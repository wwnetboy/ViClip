import { useEffect, useRef, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import { useToastStore } from "../stores/toastStore";
import { useTranslation } from "react-i18next";
import "../i18n";

interface ToastPayload {
  copy_type: "text" | "image" | "file";
  preview: string;
}

const isToastWindow = window.location.search.includes("toast=1");

export default function Toast() {
  // Main app: use Zustand store for settings feedback
  const { visible: mainVisible, message: mainMessage } = useToastStore();

  // Toast window: local state for paste notifications
  const { i18n } = useTranslation();
  const [toastVisible, setToastVisible] = useState(false);
  const [toastPayload, setToastPayload] = useState<ToastPayload | null>(null);
  const timerRef = useRef<ReturnType<typeof setTimeout> | null>(null);
  const unlistenRef = useRef<(() => void) | null>(null);
  const toastEnabledRef = useRef(true);
  const [langLoaded, setLangLoaded] = useState(false);

  // Toast window: load language and theme
  useEffect(() => {
    if (!isToastWindow) return;
    const loadSettings = async () => {
      try {
        const { invoke } = await import("@tauri-apps/api/core");
        const lang = await invoke<string>("get_setting", { key: "language" });
        if (lang && lang !== i18n.language) {
          await i18n.changeLanguage(lang);
        }
        const theme = await invoke<string>("get_setting", { key: "theme" });
        const resolved = theme === "auto"
          ? (window.matchMedia("(prefers-color-scheme: dark)").matches ? "dark" : "light")
          : (theme || "light");
        document.documentElement.setAttribute("data-theme", resolved);
        const toastEnabled = await invoke<string>("get_setting", { key: "toast_enabled" });
        toastEnabledRef.current = toastEnabled !== "0";
      } catch (e) {
        console.error("Toast: failed to load settings:", e);
      }
      setLangLoaded(true);
    };
    loadSettings();

    let unlistenSetting: (() => void) | undefined;
    listen<{ enabled: boolean }>("toast-setting-changed", (e) => {
      toastEnabledRef.current = e.payload.enabled;
    }).then((fn) => { unlistenSetting = fn; });

    return () => {
      if (unlistenSetting) unlistenSetting();
    };
  }, []);

  // Toast window: listen for toast-show events
  useEffect(() => {
    if (!isToastWindow || !langLoaded) return;

    const setup = async () => {
      const unlisten = await listen<ToastPayload>("toast-show", (event) => {
        if (!toastEnabledRef.current) return;
        if (timerRef.current) clearTimeout(timerRef.current);
        setToastPayload(event.payload);
        setToastVisible(true);
        timerRef.current = setTimeout(() => {
          setToastVisible(false);
        }, 1800);
      });
      unlistenRef.current = unlisten;
    };
    setup();

    return () => {
      if (unlistenRef.current) unlistenRef.current();
      if (timerRef.current) clearTimeout(timerRef.current);
    };
  }, [langLoaded]);

  const labelKey =
    toastPayload?.copy_type === "image" ? "settings.copyImage" :
    toastPayload?.copy_type === "file" ? "settings.copyFile" :
    "settings.copyLabel";

  if (isToastWindow) {
    return (
      <div className="toast-window-container">
        <div className={`toast-window-popup${toastVisible ? " toast-window-visible" : ""}`}>
          <span className="toast-window-label">{langLoaded ? i18n.t(labelKey) : ""}</span>
          <span className="toast-window-message">{toastPayload?.preview}</span>
        </div>
      </div>
    );
  }

  return (
    <div className={`toast-container${mainVisible ? " toast-visible" : ""}`}>
      <div className="toast-inner">
        <svg className="toast-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
          <polyline points="20 6 9 17 4 12" />
        </svg>
        <span className="toast-message">{mainMessage}</span>
      </div>
    </div>
  );
}
