export interface ClipboardRecord {
  id: string;
  type: "text" | "image" | "link" | "file";
  content: string;
  source_app: string;
  created_at: string;
}

export interface PhraseGroup {
  id: string;
  name: string;
  sort_order: number;
  created_at: string;
  updated_at: string;
}

export interface Phrase {
  id: string;
  group_id: string;
  title: string;
  content: string;
  sort_order: number;
  created_at: string;
  updated_at: string;
}

export interface TranslationRecord {
  id: string;
  source_text: string;
  target_text: string;
  source_lang: string;
  target_lang: string;
  engine: "ai" | "google";
  created_at: string;
}
