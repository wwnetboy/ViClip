import { create } from "zustand";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

type UnlistenFn = () => void;

export const CLIP_TYPES = ["all", "text", "image", "link", "file"] as const;
export type ClipType = (typeof CLIP_TYPES)[number];

interface ClipboardRecord {
  id: string;
  type: "text" | "image" | "link" | "file";
  content: string;
  source_app: string;
  created_at: string;
}

interface ClipboardState {
  records: ClipboardRecord[];
  search: string;
  loading: boolean;
  thumbnailCache: Record<string, string>;
  imageCache: Record<string, string>;
  category: ClipType;
  initialized: boolean;

  init: () => void;
  setSearch: (s: string) => void;
  setCategory: (c: ClipType) => void;
  loadRecords: () => Promise<void>;
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

export const useClipboardStore = create<ClipboardState>((set, get) => ({
  records: [],
  search: "",
  loading: false,
  thumbnailCache: {},
  imageCache: {},
  category: "all",
  initialized: false,

  init: () => {
    if (get().initialized) return;
    set({ initialized: true });

    listen<ClipboardRecord>("clipboard-update", (event) => {
      const newRecord = event.payload;
      set((state) => {
        // Skip if record with same ID already exists (prevents loadRecords race)
        if (state.records.some((r) => r.id === newRecord.id)) return state;
        return { records: [newRecord, ...state.records].slice(0, 2000) };
      });
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

  loadRecords: async () => {
    set({ loading: true });
    try {
      const s = get().search || undefined;
      const records = await invoke<ClipboardRecord[]>("get_clipboard_records", {
        search: s,
        limit: 2000,
      });
      set({ records });
    } catch (e) {
      console.error("Failed to load clipboard records:", e);
    } finally {
      set({ loading: false });
    }
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
        imageCache: cache,
      });
    } catch (e) {
      console.error("Failed to delete record:", e);
    }
  },

  pasteRecord: async (record: ClipboardRecord) => {
    try {
      console.error("[frontend] pasteRecord called, type=", record.type, "content len=", record.content?.length);
      if (record.type === "image") {
        await invoke("paste_image", { path: record.content });
      } else if (record.type === "file") {
        await invoke("paste_file", { path: record.content });
      } else {
        await invoke("paste_text", { text: record.content });
      }
      console.error("[frontend] pasteRecord invoke completed");
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
        set({ thumbnailCache: { ...get().thumbnailCache, [record.id]: url } });
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
      set({ imageCache: { ...get().imageCache, [record.id]: url } });
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
