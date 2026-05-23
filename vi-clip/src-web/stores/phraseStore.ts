import { create } from "zustand";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

interface PhraseGroup {
  id: string;
  name: string;
  sort_order: number;
  created_at: string;
  updated_at: string;
}

interface Phrase {
  id: string;
  group_id: string;
  title: string;
  content: string;
  sort_order: number;
  created_at: string;
  updated_at: string;
}

interface PhraseState {
  groups: PhraseGroup[];
  phrases: Phrase[];
  selectedGroupId: string | null;
  search: string;
  loading: boolean;

  setSearch: (s: string) => void;
  setSelectedGroup: (id: string | null) => void;
  init: () => void;
  loadGroups: () => Promise<void>;
  loadPhrases: (groupId: string) => Promise<void>;
  createGroup: (name: string) => Promise<void>;
  updateGroup: (id: string, name: string) => Promise<void>;
  deleteGroup: (id: string) => Promise<void>;
  createPhrase: (
    groupId: string,
    title: string,
    content: string
  ) => Promise<void>;
  updatePhrase: (
    id: string,
    title: string,
    content: string
  ) => Promise<void>;
  deletePhrase: (id: string) => Promise<void>;
  pastePhrase: (phrase: Phrase) => Promise<void>;
}

export const usePhraseStore = create<PhraseState>()((set, get) => {
  let initialized = false;

  return {
  groups: [],
  phrases: [],
  selectedGroupId: null,
  search: "",
  loading: false,

  setSearch: (s: string) => set({ search: s }),
  setSelectedGroup: (id: string | null) => set({ selectedGroupId: id }),

  loadGroups: async () => {
    try {
      const groups = await invoke<PhraseGroup[]>("get_phrase_groups");
      set({ groups });
      if (groups.length > 0 && !get().selectedGroupId) {
        get().loadPhrases(groups[0].id);
      }
    } catch (e) {
      console.error("Failed to load phrase groups:", e);
    }
  },

  init: () => {
    if (initialized) return;
    initialized = true;

    listen("phrase-groups-changed", () => {
      get().loadGroups();
    });

    get().loadGroups();
  },

  loadPhrases: async (groupId: string) => {
    set({ loading: true });
    try {
      const phrases = await invoke<Phrase[]>("get_phrases", { groupId });
      set({ phrases, selectedGroupId: groupId });
    } catch (e) {
      console.error("Failed to load phrases:", e);
    } finally {
      set({ loading: false });
    }
  },

  createGroup: async (name: string) => {
    try {
      const group = await invoke<PhraseGroup>("create_phrase_group", { name });
      set({ groups: [...get().groups, group] });
    } catch (e) {
      console.error("Failed to create group:", e);
    }
  },

  updateGroup: async (id: string, name: string) => {
    try {
      await invoke("update_phrase_group", { id, name });
      set({
        groups: get().groups.map((g) => (g.id === id ? { ...g, name } : g)),
      });
    } catch (e) {
      console.error("Failed to update group:", e);
    }
  },

  deleteGroup: async (id: string) => {
    try {
      await invoke("delete_phrase_group", { id });
      set({
        groups: get().groups.filter((g) => g.id !== id),
        phrases:
          get().selectedGroupId === id ? [] : get().phrases,
        selectedGroupId:
          get().selectedGroupId === id ? null : get().selectedGroupId,
      });
    } catch (e) {
      console.error("Failed to delete group:", e);
    }
  },

  createPhrase: async (groupId: string, title: string, content: string) => {
    try {
      const phrase = await invoke<Phrase>("create_phrase", {
        groupId,
        title,
        content,
      });
      set({ phrases: [...get().phrases, phrase] });
    } catch (e) {
      console.error("Failed to create phrase:", e);
    }
  },

  updatePhrase: async (id: string, title: string, content: string) => {
    try {
      await invoke("update_phrase", { id, title, content });
      set({
        phrases: get().phrases.map((p) =>
          p.id === id ? { ...p, title, content } : p
        ),
      });
    } catch (e) {
      console.error("Failed to update phrase:", e);
    }
  },

  deletePhrase: async (id: string) => {
    try {
      await invoke("delete_phrase", { id });
      set({ phrases: get().phrases.filter((p) => p.id !== id) });
    } catch (e) {
      console.error("Failed to delete phrase:", e);
    }
  },

  pastePhrase: async (phrase: Phrase) => {
    try {
      console.error("[frontend] pastePhrase called, content len=", phrase.content?.length);
      await invoke("paste_text", { text: phrase.content });
      console.error("[frontend] pastePhrase invoke completed");
    } catch (e) {
      console.error("[frontend] Paste failed:", e);
    }
  },
  };
});
