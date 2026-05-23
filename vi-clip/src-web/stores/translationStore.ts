import { create } from "zustand";
import { invoke } from "@tauri-apps/api/core";

interface TranslationResult {
  source_text: string;
  target_text: string;
  engine: string;
}

interface TranslationState {
  inputText: string;
  targetLang: string;
  result: string | null;
  engine: string | null;
  loading: boolean;
  error: string | null;

  setInputText: (text: string) => void;
  setTargetLang: (lang: string) => void;
  translate: () => Promise<void>;
}

export const useTranslationStore = create<TranslationState>((set, get) => ({
  inputText: "",
  targetLang: "zh",
  result: null,
  engine: null,
  loading: false,
  error: null,

  setInputText: (text: string) => set({ inputText: text }),
  setTargetLang: (lang: string) => set({ targetLang: lang }),

  translate: async () => {
    const { inputText, targetLang } = get();
    if (!inputText.trim()) return;

    set({ loading: true, error: null });
    try {
      const res = await invoke<TranslationResult>("translate", {
        text: inputText,
        targetLang,
      });
      set({ result: res.target_text, engine: res.engine });
    } catch (e) {
      set({ error: String(e) });
    } finally {
      set({ loading: false });
    }
  },
}));
