import { useState, useEffect, useRef, useCallback } from "react";
import { createPortal } from "react-dom";
import { useTranslation } from "react-i18next";
import { invoke } from "@tauri-apps/api/core";
import { emit } from "@tauri-apps/api/event";
import { useSettingsStore, resolveTheme, type ThemeMode } from "../stores/settingsStore";
import { StorageSection, BasicSettingsSection, ShortcutSection, TranslationSection, AboutSection } from "./settings";

interface Props {
  embedded?: boolean;
}

export default function SettingsContent({ embedded }: Props) {
  const { i18n, t } = useTranslation();
  const settings = useSettingsStore();

  const [recording, setRecording] = useState(false);
  const recordingRef = useRef(false);
  const keydownHandlerRef = useRef<((e: KeyboardEvent) => void) | null>(null);
  const [storagePath, setStoragePath] = useState("");
  const [toast, setToast] = useState<string | null>(null);
  const toastTimer = useRef<ReturnType<typeof setTimeout> | null>(null);
  const debounceTimers = useRef<Record<string, ReturnType<typeof setTimeout>>>({});

  const loadSettings = settings.loadSettings;

  useEffect(() => {
    loadSettings();
    invoke<string>("get_storage_path").then(setStoragePath).catch(console.error);
  }, [loadSettings]);

  const showToast = useCallback((msg: string) => {
    setToast(msg);
    if (toastTimer.current) clearTimeout(toastTimer.current);
    toastTimer.current = setTimeout(() => setToast(null), 1800);
  }, []);

  // ── Persist helpers ──

  const persistKey = useCallback(async (key: string, value: string) => {
    await settings.setSetting(key, value);
  }, [settings]);

  const debouncedPersist = useCallback((dbKey: string, value: string, toastKey: string) => {
    if (debounceTimers.current[dbKey]) clearTimeout(debounceTimers.current[dbKey]);
    debounceTimers.current[dbKey] = setTimeout(async () => {
      await settings.setSetting(dbKey, value);
      showToast(t(toastKey));
    }, 600);
  }, [settings, showToast, t]);

  // ── Immediate-persist callbacks ──

  const handleThemeChange = useCallback(async (theme: string) => {
    if (theme === settings.themeMode) return;
    useSettingsStore.setState({ themeMode: theme as ThemeMode });
    document.documentElement.setAttribute("data-theme", resolveTheme(theme as ThemeMode));
    emit("theme-changed", { theme });
    await settings.setSetting("theme", theme);
    showToast(t("settings.toast.theme"));
  }, [settings, showToast, t]);

  const handleLangChange = useCallback(async (lang: string) => {
    if (lang === i18n.language) return;
    i18n.changeLanguage(lang);
    emit("language-changed", { language: lang });
    await settings.setSetting("language", lang);
    invoke("update_tray_language").catch(console.error);
    showToast(t("settings.toast.language"));
  }, [i18n, settings, showToast, t]);

  const handleAutostartChange = useCallback(async (enabled: boolean) => {
    await settings.setAutostart(enabled);
    showToast(t("settings.toast.startup"));
  }, [settings, showToast, t]);

  const handleMinimizeToTrayChange = useCallback(async (enabled: boolean) => {
    await settings.setMinimizeToTray(enabled);
    showToast(t("settings.toast.minimizeToTray"));
  }, [settings, showToast, t]);

  const handleShortcutChange = useCallback(async (newKey: string) => {
    const oldKey = settings.shortcutKey;
    useSettingsStore.setState({ shortcutKey: newKey });
    await settings.setSetting("shortcut_key", newKey);
    if (oldKey !== newKey) {
      try {
        await invoke("update_shortcut", { oldShortcut: oldKey, newShortcut: newKey });
      } catch (e) {
        console.error("Failed to update shortcut:", e);
      }
    }
    showToast(t("settings.toast.shortcut"));
  }, [settings, showToast, t]);

  const handleRadialMenuChange = useCallback(async (enabled: boolean) => {
    useSettingsStore.setState({ radialMenuEnabled: enabled });
    await settings.setSetting("radial_menu_enabled", enabled ? "1" : "0");
    try {
      await invoke("set_radial_menu_enabled", { enabled });
    } catch (e) {
      console.error("Failed to set radial menu enabled:", e);
    }
    showToast(t("settings.toast.radialMenu"));
  }, [settings, showToast, t]);

  const handleClickModeChange = useCallback(async (mode: string) => {
    useSettingsStore.setState({ clickMode: mode });
    await persistKey("click_mode", mode);
    showToast(t("settings.toast.clickMode"));
  }, [persistKey, showToast, t]);

  const handleToastChange = useCallback(async (enabled: boolean) => {
    useSettingsStore.setState({ toastEnabled: enabled });
    await persistKey("toast_enabled", enabled ? "1" : "0");
    emit("toast-setting-changed", { enabled });
    showToast(t(enabled ? "settings.toast.toastOn" : "settings.toast.toastOff"));
  }, [persistKey, showToast, t]);

  const handleRetentionChange = useCallback(async (retention: string) => {
    useSettingsStore.setState({ clipboardRetention: retention });
    await persistKey("clipboard_retention", retention);
    showToast(t("settings.toast.retention"));
  }, [persistKey, showToast, t]);

  const handleEngineChange = useCallback(async (engine: string) => {
    useSettingsStore.setState({ defaultEngine: engine });
    await persistKey("default_translate_engine", engine);
    showToast(t("settings.toast.engine"));
  }, [persistKey, showToast, t]);

  // ── Text input callbacks (update store immediately, persist with debounce) ──

  const handleApiUrlChange = useCallback((value: string) => {
    useSettingsStore.setState({ apiUrl: value });
    debouncedPersist("ai_api_url", value, "settings.toast.aiApiUrl");
  }, [debouncedPersist]);

  const handleApiKeyChange = useCallback((value: string) => {
    useSettingsStore.setState({ apiKey: value });
    debouncedPersist("ai_api_key", value, "settings.toast.aiApiKey");
  }, [debouncedPersist]);

  const handleModelChange = useCallback((value: string) => {
    useSettingsStore.setState({ model: value });
    debouncedPersist("ai_model", value, "settings.toast.aiModel");
  }, [debouncedPersist]);

  const handleGoogleApiKeyChange = useCallback((value: string) => {
    useSettingsStore.setState({ googleApiKey: value });
    debouncedPersist("google_api_key", value, "settings.toast.googleApiKey");
  }, [debouncedPersist]);

  const handleTranslateProxyChange = useCallback((value: string) => {
    useSettingsStore.setState({ translateProxy: value });
    debouncedPersist("translate_proxy", value, "settings.toast.translateProxy");
  }, [debouncedPersist]);

  // ── Recording logic ──

  const startRecording = () => {
    recordingRef.current = true;
    setRecording(true);

    const cleanup = () => {
      document.removeEventListener("keydown", handler, true);
      keydownHandlerRef.current = null;
    };

    const handler = (e: KeyboardEvent) => {
      if (!recordingRef.current) {
        cleanup();
        return;
      }

      if (["Control", "Alt", "Shift", "Meta", "CapsLock", "NumLock", "ScrollLock", "Dead"].includes(e.key)) {
        return;
      }

      if (!e.ctrlKey && !e.altKey && !e.shiftKey && !e.metaKey) {
        return;
      }

      e.preventDefault();
      e.stopPropagation();

      const parts: string[] = [];
      if (e.ctrlKey) parts.push("Ctrl");
      if (e.altKey) parts.push("Alt");
      if (e.shiftKey) parts.push("Shift");
      if (e.metaKey) parts.push("Super");

      const code = e.code;
      let keyName: string;
      if (code.startsWith("Key")) {
        keyName = code[3];
      } else if (code.startsWith("Digit")) {
        keyName = code[5];
      } else if (code.startsWith("Numpad")) {
        keyName = "NumPad" + code.substring(6);
      } else {
        keyName = e.key;
        if (keyName === " ") keyName = "Space";
      }
      parts.push(keyName);

      const shortcut = parts.join("+");
      recordingRef.current = false;
      setRecording(false);
      cleanup();
      handleShortcutChange(shortcut);
    };

    keydownHandlerRef.current = handler;
    document.addEventListener("keydown", handler, true);
  };

  const stopRecording = () => {
    recordingRef.current = false;
    setRecording(false);
    if (keydownHandlerRef.current) {
      document.removeEventListener("keydown", keydownHandlerRef.current, true);
      keydownHandlerRef.current = null;
    }
  };

  const content = (
    <>
      {toast && createPortal(
        <div className="settings-toast">{toast}</div>,
        document.body
      )}

      <BasicSettingsSection
        themeMode={settings.themeMode}
        onThemeChange={handleThemeChange}
        language={i18n.language}
        onLanguageChange={handleLangChange}
        autostartEnabled={settings.autostartEnabled}
        onAutostartChange={handleAutostartChange}
        minimizeToTray={settings.minimizeToTray}
        onMinimizeToTrayChange={handleMinimizeToTrayChange}
        toastEnabled={settings.toastEnabled}
        onToastChange={handleToastChange}
      />

      <ShortcutSection
        shortcutKey={settings.shortcutKey}
        onShortcutChange={handleShortcutChange}
        recording={recording}
        onStartRecording={startRecording}
        onStopRecording={stopRecording}
        radialMenuEnabled={settings.radialMenuEnabled}
        onRadialMenuChange={handleRadialMenuChange}
        clickMode={settings.clickMode}
        onClickModeChange={handleClickModeChange}
      />

      <TranslationSection
        engine={settings.defaultEngine}
        onEngineChange={handleEngineChange}
        apiUrl={settings.apiUrl}
        onApiUrlChange={handleApiUrlChange}
        apiKey={settings.apiKey}
        onApiKeyChange={handleApiKeyChange}
        model={settings.model}
        onModelChange={handleModelChange}
        googleApiKey={settings.googleApiKey}
        onGoogleApiKeyChange={handleGoogleApiKeyChange}
        translateProxy={settings.translateProxy}
        onTranslateProxyChange={handleTranslateProxyChange}
      />

      <StorageSection
        storagePath={storagePath}
        setStoragePath={setStoragePath}
        retention={settings.clipboardRetention}
        onRetentionChange={handleRetentionChange}
      />

      <AboutSection />
    </>
  );

  if (embedded) {
    return <div className="settings-panel-content">{content}</div>;
  }

  return content;
}
