import { create } from "zustand";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import type { ClipboardRecord } from "../types";

type UnlistenFn = () => void;

export const CLIP_TYPES = ["all", "text", "image", "link", "file"] as const;
export type ClipType = (typeof CLIP_TYPES)[number];

const MAX_THUMB_CACHE = 50;
const MAX_IMAGE_CACHE = 20;
const PAGE_SIZE = 50;

// Bumped on every page fetch so responses from a superseded
// query (search/category changed mid-flight) are discarded.
let fetchSeq = 0;

interface ClipboardState {
  records: ClipboardRecord[];
  search: string;
  loading: boolean;
  hasMore: boolean;
  thumbnailCache: Record<string, string>;
  thumbnailCacheOrder: string[];
  imageCache: Record<string, string>;
  imageCacheOrder: string[];
  category: ClipType;
  initialized: boolean;

  init: () => void;
  setSearch: (s: string) => void;
  setCategory: (c: ClipType) => void;
  loadRecords: () => Promise<void>;
  loadMore: () => Promise<void>;
  deleteRecord: (id: string) => Promise<void>;
  pasteRecord: (record: ClipboardRecord) => Promise<void>;
  getThumbnail: (record: ClipboardRecord) => Promise<string>;
  getImageData: (record: ClipboardRecord) => Promise<string>;
}

let unlisten: UnlistenFn | null = null;

const MAX_CONCURRENT = 3;
let running = 0;
const queue: (() => void)[] = [];

function enqueue<T>(fn: () => Promise<T>): Promise<T> {
  return new Promise((resolve, reject) => {
    const run = async () => {
      running++;
      try {
        resolve(await fn());
      } catch (e) {
        reject(e);
      } finally {
        running--;
        if (queue.length > 0 && running < MAX_CONCURRENT) {
          const next = queue.shift()!;
          next();
        }
      }
    };
    if (running < MAX_CONCURRENT) {
      run();
    } else {
      queue.push(run);
    }
  });
}

function addToCache(
  cache: Record<string, string>,
  order: string[],
  key: string,
  value: string,
  max: number,
): { cache: Record<string, string>; order: string[] } {
  const newCache = { ...cache, [key]: value };
  const newOrder = order.filter((k) => k !== key);
  newOrder.push(key);
  for (let i = 0; newOrder.length > max; i++) {
    delete newCache[newOrder[i]];
    newOrder.shift();
    i--;
  }
  return { cache: newCache, order: newOrder };
}

type SetState = (
  partial: Partial<ClipboardState> | ((state: ClipboardState) => Partial<ClipboardState>),
) => void;

// One page of records. reset=true replaces the list (first page /
// search / category change), otherwise results are appended.
async function fetchPage(set: SetState, get: () => ClipboardState, offset: number, reset: boolean) {
  const seq = ++fetchSeq;
  set({ loading: true });
  try {
    const s = get().search || undefined;
    const cat = get().category !== "all" ? get().category : undefined;
    const page = await invoke<ClipboardRecord[]>("get_clipboard_records", {
      search: s,
      limit: PAGE_SIZE,
      recordType: cat,
      offset,
    });
    if (seq !== fetchSeq) return;
    set((state) => {
      if (reset) return { records: page, hasMore: page.length === PAGE_SIZE };
      // clipboard-update may have prepended rows while the user was
      // paging; drop anything already present instead of duplicating.
      const known = new Set(state.records.map((r) => r.id));
      return {
        records: [...state.records, ...page.filter((r) => !known.has(r.id))],
        hasMore: page.length === PAGE_SIZE,
      };
    });
  } catch (e) {
    console.error("Failed to load clipboard records:", e);
  } finally {
    if (seq === fetchSeq) set({ loading: false });
  }
}

export const useClipboardStore = create<ClipboardState>((set, get) => ({  records: [],
  search: "",
  loading: false,
  hasMore: false,
  thumbnailCache: {},
  thumbnailCacheOrder: [],
  imageCache: {},
  imageCacheOrder: [],
  category: "all",
  initialized: false,

  init: () => {
    if (get().initialized) return;
    set({ initialized: true });

    listen<ClipboardRecord>("clipboard-update", (event) => {
      const newRecord = event.payload;
      set((state) => ({
        // Backend bumps re-copied content instead of cloning it: a known
        // id arriving here carries a fresh created_at, so lift it to top.
        records: [
          newRecord,
          ...state.records.filter((r) => r.id !== newRecord.id),
        ].slice(0, 2000),
      }));
    }).then((fn) => {
      unlisten = fn;
    });

    listen<string>("clipboard-deleted", (event) => {
      const deletedId = event.payload;
      set((state) => ({
        records: state.records.filter((r) => r.id !== deletedId),
      }));
    });

    get().loadRecords();
  },

  setSearch: (s) => set({ search: s }),
  setCategory: (c) => set({ category: c }),

  loadRecords: () => fetchPage(set, get, 0, true),

  loadMore: async () => {
    const { records, hasMore, loading } = get();
    if (!hasMore || loading) return;
    await fetchPage(set, get, records.length, false);
  },

  deleteRecord: async (id: string) => {
    try {
      await invoke("delete_clipboard_record", { id });
      const thumbCache = { ...get().thumbnailCache };
      delete thumbCache[id];
      const cache = { ...get().imageCache };
      delete cache[id];
      set({
        records: get().records.filter((r) => r.id !== id),
        thumbnailCache: thumbCache,
        thumbnailCacheOrder: get().thumbnailCacheOrder.filter((k) => k !== id),
        imageCache: cache,
        imageCacheOrder: get().imageCacheOrder.filter((k) => k !== id),
      });
    } catch (e) {
      console.error("Failed to delete record:", e);
    }
  },

  pasteRecord: async (record: ClipboardRecord) => {
    try {
      if (record.type === "image") {
        await invoke("paste_image", { path: record.content });
      } else if (record.type === "file") {
        await invoke("paste_file", { path: record.content });
      } else {
        await invoke("paste_text", { text: record.content });
      }
    } catch (e) {
      console.error("[frontend] Paste failed:", e);
    }
  },

  getThumbnail: async (record: ClipboardRecord): Promise<string> => {
    const cached = get().thumbnailCache[record.id];
    if (cached) return cached;

    return enqueue(async () => {
      const cached2 = get().thumbnailCache[record.id];
      if (cached2) return cached2;

      try {
        // Use base64 data URI for reliable cross-platform display
        const base64 = await invoke<string>("get_image_thumbnail", {
          path: record.content,
          maxSize: 200,
        });
        const url = `data:image/png;base64,${base64}`;
        const { cache, order } = addToCache(
          get().thumbnailCache, get().thumbnailCacheOrder,
          record.id, url, MAX_THUMB_CACHE,
        );
        set({ thumbnailCache: cache, thumbnailCacheOrder: order });
        return url;
      } catch (e) {
        console.error("Failed to load thumbnail:", e);
        return "";
      }
    });
  },

  getImageData: async (record: ClipboardRecord): Promise<string> => {
    const cached = get().imageCache[record.id];
    if (cached) return cached;

    try {
      const base64 = await invoke<string>("get_image_base64", {
        path: record.content,
      });
      const url = `data:image/png;base64,${base64}`;
      const { cache, order } = addToCache(
        get().imageCache, get().imageCacheOrder,
        record.id, url, MAX_IMAGE_CACHE,
      );
      set({ imageCache: cache, imageCacheOrder: order });
      return url;
    } catch (e) {
      console.error("Failed to load image:", e);
      return "";
    }
  },
}));

if (typeof window !== "undefined") {
  window.addEventListener("beforeunload", () => {
    if (unlisten) unlisten();
  });
}
