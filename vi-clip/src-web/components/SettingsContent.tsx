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
  const themeMode = useSettingsStore((s) => s.themeMode);
  const autostartEnabled = useSettingsStore((s) => s.autostartEnabled);
  const minimizeToTray = useSettingsStore((s) => s.minimizeToTray);
  const toastEnabled = useSettingsStore((s) => s.toastEnabled);
  const shortcutKey = useSettingsStore((s) => s.shortcutKey);
  const radialMenuEnabled = useSettingsStore((s) => s.radialMenuEnabled);
  const clickMode = useSettingsStore((s) => s.clickMode);
  const defaultEngine = useSettingsStore((s) => s.defaultEngine);
  const clipboardRetention = useSettingsStore((s) => s.clipboardRetention);
  const loadSettings = useSettingsStore((s) => s.loadSettings);
  const setSetting = useSettingsStore((s) => s.setSetting);
  const setAutostart = useSettingsStore((s) => s.setAutostart);
  const setMinimizeToTray = useSettingsStore((s) => s.setMinimizeToTray);

  const [recording, setRecording] = useState(false);
  const recordingRef = useRef(false);
  const keydownHandlerRef = useRef<((e: KeyboardEvent) => void) | null>(null);
  const [storagePath, setStoragePath] = useState("");
  const [toast, setToast] = useState<string | null>(null);
  const toastTimer = useRef<ReturnType<typeof setTimeout> | null>(null);
  const debounceTimers = useRef<Record<string, ReturnType<typeof setTimeout>>>({});

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
    await setSetting(key, value);
  }, [setSetting]);

  const debouncedPersist = useCallback((dbKey: string, value: string, toastKey: string) => {
    if (debounceTimers.current[dbKey]) clearTimeout(debounceTimers.current[dbKey]);
    debounceTimers.current[dbKey] = setTimeout(async () => {
      await setSetting(dbKey, value);
      showToast(t(toastKey));
    }, 600);
  }, [setSetting, showToast, t]);

  // ── Immediate-persist callbacks ──

  const handleThemeChange = useCallback(async (theme: string) => {
    if (theme === themeMode) return;
    useSettingsStore.setState({ themeMode: theme as ThemeMode });
    document.documentElement.setAttribute("data-theme", resolveTheme(theme as ThemeMode));
    emit("theme-changed", { theme });
    await setSetting("theme", theme);
    showToast(t("settings.toast.theme"));
  }, [themeMode, setSetting, showToast, t]);

  const handleLangChange = useCallback(async (lang: string) => {
    if (lang === i18n.language) return;
    i18n.changeLanguage(lang);
    emit("language-changed", { language: lang });
    await setSetting("language", lang);
    invoke("update_tray_language").catch(console.error);
    showToast(t("settings.toast.language"));
  }, [i18n, setSetting, showToast, t]);

  const handleAutostartChange = useCallback(async (enabled: boolean) => {
    await setAutostart(enabled);
    showToast(t("settings.toast.startup"));
  }, [setAutostart, showToast, t]);

  const handleMinimizeToTrayChange = useCallback(async (enabled: boolean) => {
    await setMinimizeToTray(enabled);
    showToast(t("settings.toast.minimizeToTray"));
  }, [setMinimizeToTray, showToast, t]);

  const handleShortcutChange = useCallback(async (newKey: string) => {
    const oldKey = shortcutKey;
    useSettingsStore.setState({ shortcutKey: newKey });
    await setSetting("shortcut_key", newKey);
    if (oldKey !== newKey) {
      try {
        await invoke("update_shortcut", { oldShortcut: oldKey, newShortcut: newKey });
      } catch (e) {
        console.error("Failed to update shortcut:", e);
      }
    }
    showToast(t("settings.toast.shortcut"));
  }, [shortcutKey, setSetting, showToast, t]);

  const handleRadialMenuChange = useCallback(async (enabled: boolean) => {
    useSettingsStore.setState({ radialMenuEnabled: enabled });
    await setSetting("radial_menu_enabled", enabled ? "1" : "0");
    try {
      await invoke("set_radial_menu_enabled", { enabled });
    } catch (e) {
      console.error("Failed to set radial menu enabled:", e);
    }
    showToast(t("settings.toast.radialMenu"));
  }, [setSetting, showToast, t]);

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

  // ── Config persist (store update + debounced DB write) ──

  const persistConfig = useCallback((dbKey: string, value: string, toastKey: string) => {
    debouncedPersist(dbKey, value, toastKey);
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
        themeMode={themeMode}
        onThemeChange={handleThemeChange}
        language={i18n.language}
        onLanguageChange={handleLangChange}
        autostartEnabled={autostartEnabled}
        onAutostartChange={handleAutostartChange}
        minimizeToTray={minimizeToTray}
        onMinimizeToTrayChange={handleMinimizeToTrayChange}
        toastEnabled={toastEnabled}
        onToastChange={handleToastChange}
      />

      <ShortcutSection
        shortcutKey={shortcutKey}
        onShortcutChange={handleShortcutChange}
        recording={recording}
        onStartRecording={startRecording}
        onStopRecording={stopRecording}
        radialMenuEnabled={radialMenuEnabled}
        onRadialMenuChange={handleRadialMenuChange}
        clickMode={clickMode}
        onClickModeChange={handleClickModeChange}
      />

      <TranslationSection
        engine={defaultEngine}
        onEngineChange={handleEngineChange}
        persistConfig={persistConfig}
      />

      <StorageSection
        storagePath={storagePath}
        setStoragePath={setStoragePath}
        retention={clipboardRetention}
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
