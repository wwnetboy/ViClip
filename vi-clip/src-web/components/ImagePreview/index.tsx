import { useState, useEffect, useCallback } from "react";
import { useTranslation } from "react-i18next";
import i18n from "../../i18n";
import { invoke } from "@tauri-apps/api/core";
import { getCurrentWindow, currentMonitor, LogicalSize, LogicalPosition } from "@tauri-apps/api/window";
import { listen, emit } from "@tauri-apps/api/event";

const TITLE_BAR_HEIGHT = 32;

export default function ImagePreview() {
  const { t } = useTranslation();
  const [imageSrc, setImageSrc] = useState<string | null>(null);
  const [alwaysOnTop, setAlwaysOnTop] = useState(false);
  const [naturalSize, setNaturalSize] = useState<{ w: number; h: number } | null>(null);
  const [isMaximized, setIsMaximized] = useState(false);
  const win = getCurrentWindow();

  const fitWindowToImage = useCallback(async (imgW: number, imgH: number) => {
    try {
      const monitor = await currentMonitor();
      const scale = monitor?.scaleFactor ?? 1;
      const availW = (monitor?.size.width ?? 1920) / scale;
      const availH = (monitor?.size.height ?? 1080) / scale;

      const maxW = Math.floor(availW * 0.9);
      const maxH = Math.floor(availH * 0.9);

      const contentW = imgW;
      const contentH = imgH + TITLE_BAR_HEIGHT;

      let winW: number;
      let winH: number;

      if (contentW <= maxW && contentH <= maxH) {
        winW = contentW;
        winH = contentH;
      } else {
        const scaleW = maxW / contentW;
        const scaleH = maxH / contentH;
        const s = Math.min(scaleW, scaleH);
        winW = Math.floor(contentW * s);
        winH = Math.floor(contentH * s);
      }

      await win.setSize(new LogicalSize(winW, winH));
      const x = Math.floor((availW - winW) / 2);
      const y = Math.floor((availH - winH) / 2);
      await win.setPosition(new LogicalPosition(Math.max(0, x), Math.max(0, y)));
    } catch {
      // ignore resize errors
    }
  }, [win]);

  const winLabel = win.label;

  const lockAspectRatio = useCallback(async (ratio: number) => {
    try {
      await invoke("set_preview_aspect_ratio", { windowLabel: winLabel, ratio, titlebarHLogical: TITLE_BAR_HEIGHT });
    } catch { /* ignore */ }
  }, [winLabel]);

  const unlockAspectRatio = useCallback(async () => {
    try {
      await invoke("set_preview_aspect_ratio", { windowLabel: winLabel, ratio: null, titlebarHLogical: 0 });
    } catch { /* ignore */ }
  }, [winLabel]);

  useEffect(() => {
    invoke("apply_preview_backdrop", { windowLabel: winLabel }).catch(() => {});
    // Sync pin state from the window's actual always-on-top state
    win.isAlwaysOnTop().then(setAlwaysOnTop).catch(() => {});
  }, []);

  useEffect(() => {
    const setup = async () => {
      const unlistenClose = await win.onCloseRequested(async (event) => {
        event.preventDefault();
        await unlockAspectRatio();
        win.destroy();
      });

      const unlistenResize = await win.onResized(async () => {
        try {
          const maximized = await win.isMaximized();
          setIsMaximized(maximized);
        } catch { /* ignore */ }
      });

      return () => {
        unlistenClose();
        unlistenResize();
      };
    };
    const cleanupPromise = setup();

    return () => {
      cleanupPromise.then((fn) => fn());
    };
  }, []);

  // Sync theme and language with main window
  useEffect(() => {
    const sync = async () => {
      try {
        const theme = await invoke<string>("get_setting", { key: "theme" });
        if (theme) document.documentElement.setAttribute("data-theme", theme);
      } catch { /* ignore */ }

      try {
        const lang = await invoke<string>("get_setting", { key: "language" });
        if (lang && lang !== i18n.language) {
          i18n.changeLanguage(lang);
        }
      } catch { /* ignore */ }

      const unlistenTheme = await listen<{ theme: string }>("theme-changed", (event) => {
        document.documentElement.setAttribute("data-theme", event.payload.theme);
      });

      const unlistenLang = await listen<{ language: string }>("language-changed", (event) => {
        if (event.payload.language !== i18n.language) {
          i18n.changeLanguage(event.payload.language);
        }
      });

      return () => {
        unlistenTheme();
        unlistenLang();
      };
    };
    const cleanupPromise = sync();

    return () => {
      cleanupPromise.then((fn) => fn?.());
    };
  }, []);

  // Load image data from the Rust store using token from URL
  useEffect(() => {
    const params = new URLSearchParams(window.location.search);
    const token = params.get("token");
    if (token) {
      invoke<string | null>("fetch_preview_image", { token })
        .then((base64) => {
          if (base64) {
            setImageSrc(`data:image/png;base64,${base64}`);
          }
        })
        .catch(console.error);
    }
  }, []);

  const handleTogglePin = useCallback(async () => {
    const next = !alwaysOnTop;
    setAlwaysOnTop(next);
    await win.setAlwaysOnTop(next);
    emit("image-preview-pin-changed", { pinned: next });
  }, [alwaysOnTop, win]);

  const handleMinimize = useCallback(async () => {
    await win.minimize();
  }, [win]);

  const handleToggleMaximize = useCallback(async () => {
    await win.toggleMaximize();
  }, [win]);

  const handleClose = useCallback(async () => {
    await win.close();
  }, [win]);

  return (
    <div className="image-preview-window">
      <div className="preview-titlebar" data-tauri-drag-region>
        <div className="preview-titlebar-title">
          {naturalSize ? (
            <span className="preview-titlebar-size">
              {naturalSize.w} × {naturalSize.h}
            </span>
          ) : (
            <span className="preview-titlebar-label">{t("imagePreview.title")}</span>
          )}
        </div>
        <div className="preview-titlebar-controls">
          <button
            className={`preview-ctrl-btn pin ${alwaysOnTop ? "active" : ""}`}
            onClick={handleTogglePin}
            title={alwaysOnTop ? t("imagePreview.unpin") : t("imagePreview.pin")}
          >
            <svg viewBox="0 0 24 24" fill={alwaysOnTop ? "currentColor" : "none"} stroke="currentColor" strokeWidth="2">
              <path d="M16 12V4h1a1 1 0 0 0 0-2H7a1 1 0 0 0 0 2h1v8l-2.5 3.5a1 1 0 0 0 .8 1.5h4.7v4.5a1 1 0 0 0 2 0V17h4.7a1 1 0 0 0 .8-1.5L16 12z" />
            </svg>
          </button>
          <button className="preview-ctrl-btn" onClick={handleMinimize} title={t("imagePreview.minimize")}>
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
              <line x1="5" y1="12" x2="19" y2="12" />
            </svg>
          </button>
          <button className="preview-ctrl-btn" onClick={handleToggleMaximize} title={isMaximized ? t("imagePreview.restore") : t("imagePreview.maximize")}>
            {isMaximized ? (
              <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
                <rect x="5" y="7" width="10" height="10" rx="1" />
                <rect x="9" y="3" width="10" height="10" rx="1" />
              </svg>
            ) : (
              <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
                <rect x="4" y="4" width="16" height="16" rx="1" />
              </svg>
            )}
          </button>
          <button className="preview-ctrl-btn close" onClick={handleClose} title={t("imagePreview.close")}>
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
              <line x1="18" y1="6" x2="6" y2="18" />
              <line x1="6" y1="6" x2="18" y2="18" />
            </svg>
          </button>
        </div>
      </div>
      <div className="image-preview-content">
        {imageSrc ? (
          <img
            key={imageSrc}
            src={imageSrc}
            alt="Preview"
            className="image-preview-img"
            onLoad={(e) => {
              const img = e.currentTarget;
              const w = img.naturalWidth;
              const h = img.naturalHeight;
              setNaturalSize({ w, h });
              lockAspectRatio(w / h);
              fitWindowToImage(w, h);
            }}
          />
        ) : (
          <div className="image-preview-empty">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" strokeLinejoin="round">
              <rect x="3" y="3" width="18" height="18" rx="2" ry="2" />
              <circle cx="8.5" cy="8.5" r="1.5" />
              <polyline points="21 15 16 10 5 21" />
            </svg>
            <span>{t("imagePreview.empty")}</span>
          </div>
        )}
      </div>
    </div>
  );
}
