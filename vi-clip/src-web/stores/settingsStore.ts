import { create } from "zustand";
import { invoke } from "@tauri-apps/api/core";
import { enable, disable, isEnabled } from "@tauri-apps/plugin-autostart";

export type ThemeMode = "light" | "dark" | "auto" | "deep-blue" | "dark-solid";

export function resolveTheme(mode: ThemeMode): "light" | "dark" | "deep-blue" | "dark-solid" {
  if (mode === "auto") {
    return window.matchMedia("(prefers-color-scheme: dark)").matches ? "dark" : "light";
  }
  return mode;
}

interface SettingsState {
  themeMode: ThemeMode;
  clipboardRetention: string;
  defaultEngine: string;
  apiUrl: string;
  apiKey: string;
  model: string;
  baiduAppId: string;
  baiduSecret: string;
  googleApiKey: string;
  translateProxy: string;
  language: string;
  shortcutKey: string;
  radialMenuEnabled: boolean;
  autostartEnabled: boolean;
  minimizeToTray: boolean;
  toastEnabled: boolean;
  clickMode: string;

  loadSettings: () => Promise<void>;
  setSetting: (key: string, value: string) => Promise<void>;
  setSettingsBatch: (settings: Record<string, string>) => Promise<void>;
  setAutostart: (enabled: boolean) => Promise<void>;
  setMinimizeToTray: (enabled: boolean) => Promise<void>;
}

export const useSettingsStore = create<SettingsState>((set) => ({
  themeMode: "light",
  clipboardRetention: "1month",
  defaultEngine: "google",
  apiUrl: "",
  apiKey: "",
  model: "",
  baiduAppId: "",
  baiduSecret: "",
  googleApiKey: "",
  translateProxy: "",
  language: "zh-CN",
  shortcutKey: "",
  radialMenuEnabled: true,
  autostartEnabled: false,
  minimizeToTray: false,
  toastEnabled: true,
  clickMode: "single",

  loadSettings: async () => {
    try {
      const settings = await invoke<Record<string, string>>("get_all_settings");

      set({
        themeMode: (settings.theme as ThemeMode) || "light",
        clipboardRetention: settings.clipboard_retention || "1month",
        defaultEngine: settings.default_translate_engine || "google",
        apiUrl: settings.ai_api_url || "",
        apiKey: settings.ai_api_key || "",
        model: settings.ai_model || "",
        baiduAppId: settings.baidu_appid || "",
        baiduSecret: settings.baidu_secret || "",
        googleApiKey: settings.google_api_key || "",
        translateProxy: settings.translate_proxy || "",
        language: settings.language || "zh-CN",
        shortcutKey: settings.shortcut_key || "",
        radialMenuEnabled: settings.radial_menu_enabled !== "0",
        toastEnabled: settings.toast_enabled !== "0",
        clickMode: settings.click_mode || "single",
        minimizeToTray: settings.minimize_to_tray === "1",
      });

      // Read autostart state from the OS (plugin)
      try {
        const auto = await isEnabled();
        set({ autostartEnabled: auto });
      } catch { /* plugin not available */ }
    } catch {
      // Settings not yet initialized, use defaults
    }
  },

  setSetting: async (key: string, value: string) => {
    try {
      await invoke("set_setting", { key, value });
    } catch (e) {
      console.error("Failed to save setting:", e);
    }
  },

  setSettingsBatch: async (settings: Record<string, string>) => {
    try {
      await invoke("set_settings_batch", { settings });
    } catch (e) {
      console.error("Failed to batch save settings:", e);
    }
  },

  setAutostart: async (enabled: boolean) => {
    try {
      if (enabled) {
        await enable();
      } else {
        await disable();
      }
      set({ autostartEnabled: enabled });
    } catch (e) {
      console.error("Failed to set autostart:", e);
    }
  },

  setMinimizeToTray: async (enabled: boolean) => {
    try {
      await invoke("set_setting", { key: "minimize_to_tray", value: enabled ? "1" : "0" });
      set({ minimizeToTray: enabled });
    } catch (e) {
      console.error("Failed to set minimize to tray:", e);
    }
  },
}));
