import { create } from "zustand";
import { invoke } from "@tauri-apps/api/core";
import { useSettingsStore } from "./settingsStore";

interface TranslationResult {
  source_text: string;
  target_text: string;
  engine: string;
  detected_lang?: string;
}

interface TranslationState {
  inputText: string;
  targetLang: string;
  result: string | null;
  engine: string | null;
  detectedLang: string | null;
  loading: boolean;
  error: string | null;

  setInputText: (text: string) => void;
  setTargetLang: (lang: string) => void;
  clearInput: () => void;
  translate: () => Promise<void>;
}

let nextRequestId = 0;
let debounceTimer: ReturnType<typeof setTimeout> | null = null;

export const useTranslationStore = create<TranslationState>((set, get) => ({
  inputText: "",
  targetLang: useSettingsStore.getState().defaultTargetLang || "zh",
  result: null,
  engine: null,
  detectedLang: null,
  loading: false,
  error: null,

  setInputText: (text: string) => {
    set({ inputText: text, error: null });
    if (debounceTimer) clearTimeout(debounceTimer);
    if (!text.trim()) {
      set({ result: null, engine: null, detectedLang: null, loading: false });
      return;
    }
    set({ loading: true });
    debounceTimer = setTimeout(() => {
      get().translate();
    }, 400);
  },

  setTargetLang: (lang: string) => {
    set({ targetLang: lang });
    if (get().inputText.trim()) {
      if (debounceTimer) clearTimeout(debounceTimer);
      set({ loading: true });
      get().translate();
    }
  },

  clearInput: () => {
    if (debounceTimer) clearTimeout(debounceTimer);
    set({ inputText: "", result: null, engine: null, detectedLang: null, loading: false, error: null });
  },

  translate: async () => {
    const { inputText, targetLang } = get();
    if (!inputText.trim()) return;

    const requestId = ++nextRequestId;
    set({ loading: true, error: null });
    try {
      const res = await invoke<TranslationResult>("translate", {
        text: inputText,
        targetLang,
      });
      if (requestId !== nextRequestId) return;
      set({ result: res.target_text, engine: res.engine, detectedLang: res.detected_lang || null });
    } catch (e) {
      if (requestId !== nextRequestId) return;
      set({ error: String(e) });
    } finally {
      if (requestId === nextRequestId) {
        set({ loading: false });
      }
    }
  },
}));
