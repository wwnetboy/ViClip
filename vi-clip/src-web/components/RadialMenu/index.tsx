import { useEffect, useRef, useState, useCallback } from "react";
import { useTranslation } from "react-i18next";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { invoke } from "@tauri-apps/api/core";
import { useClipboardStore, type ClipType } from "../../stores/clipboardStore";
import { usePhraseStore } from "../../stores/phraseStore";
import { resolveTheme, type ThemeMode, useSettingsStore } from "../../stores/settingsStore";
import i18n from "../../i18n";

type TabKey = "clipboard" | "phrases";

const MAX_ITEMS = 2000;

function formatTime(dateStr: string): string {
  const date = new Date(dateStr);
  const month = date.getMonth() + 1;
  const day = date.getDate();
  const hours = date.getHours().toString().padStart(2, "0");
  const minutes = date.getMinutes().toString().padStart(2, "0");
  return `${month}/${day} ${hours}:${minutes}`;
}

function ImageThumb({ recordId }: { recordId: string }) {
  const [src, setSrc] = useState("");
  const { records, getThumbnail } = useClipboardStore();

  useEffect(() => {
    const record = records.find((r) => r.id === recordId);
    if (!record || record.type !== "image") return;
    let cancelled = false;
    getThumbnail(record).then((url) => {
      if (!cancelled && url) setSrc(url);
    });
    return () => { cancelled = true; };
  }, [recordId, records, getThumbnail]);

  if (!src) return <span className="radial-menu-item-text">…</span>;
  return (
    <img
      src={src}
      alt=""
      style={{ width: 48, height: 36, objectFit: "cover", borderRadius: 5 }}
    />
  );
}

export default function RadialMenu() {
  const { t } = useTranslation();

  const [visible, setVisible] = useState(false);
  const [activeTab, setActiveTab] = useState<TabKey>("clipboard");
  const [selectedItemId, setSelectedItemId] = useState<string | null>(null);
  const [clipboardCategory, setClipboardCategory] = useState<ClipType>("all");
  const [phraseGroupId, setPhraseGroupId] = useState<string | null>(null);

  const showTimestampRef = useRef(0);
  const selectedItemIdRef = useRef<string | null>(null);
  const activeTabRef = useRef<TabKey>("clipboard");
  const clipboardCategoryRef = useRef<ClipType>("all");
  const phraseGroupIdRef = useRef<string | null>(null);

  useEffect(() => { selectedItemIdRef.current = selectedItemId; }, [selectedItemId]);
  useEffect(() => { activeTabRef.current = activeTab; }, [activeTab]);
  useEffect(() => { clipboardCategoryRef.current = clipboardCategory; }, [clipboardCategory]);
  useEffect(() => { phraseGroupIdRef.current = phraseGroupId; }, [phraseGroupId]);

  useEffect(() => {
    let mqCleanup: (() => void) | undefined;

    const applyTheme = (mode: string) => {
      document.documentElement.setAttribute("data-theme", resolveTheme(mode as ThemeMode));

      if (mqCleanup) { mqCleanup(); mqCleanup = undefined; }
      if (mode === "auto") {
        const mq = window.matchMedia("(prefers-color-scheme: dark)");
        const handler = () => {
          document.documentElement.setAttribute("data-theme", resolveTheme("auto" as ThemeMode));
        };
        mq.addEventListener("change", handler);
        mqCleanup = () => mq.removeEventListener("change", handler);
      }
    };

    // Initial theme load
    invoke<string>("get_setting", { key: "theme" }).then((theme) => {
      if (theme) applyTheme(theme);
    }).catch(() => {});

    // Initial language load
    invoke<string>("get_setting", { key: "language" }).then((lang) => {
      if (lang && lang !== i18n.language) {
        i18n.changeLanguage(lang);
      }
    }).catch(() => {});

    // Pre-load data so it's ready when the menu first shows
    useSettingsStore.getState().loadSettings();
    useClipboardStore.getState().init();
    usePhraseStore.getState().init();

    let unlistenTheme: UnlistenFn | undefined;
    listen<{ theme: string }>("theme-changed", (e) => {
      applyTheme(e.payload.theme);
    }).then((fn) => { unlistenTheme = fn; });

    let unlistenLang: UnlistenFn | undefined;
    listen<{ language: string }>("language-changed", (e) => {
      if (e.payload.language !== i18n.language) {
        i18n.changeLanguage(e.payload.language);
      }
    }).then((fn) => { unlistenLang = fn; });

    return () => {
      if (unlistenTheme) unlistenTheme();
      if (unlistenLang) unlistenLang();
      if (mqCleanup) mqCleanup();
    };
  }, []);

  const hide = useCallback(() => {
    setVisible(false);
    setSelectedItemId(null);
    selectedItemIdRef.current = null;
    getCurrentWindow().hide();
  }, []);

  const handleTabSwitch = useCallback((key: string) => {
    const tab = key as TabKey;
    setActiveTab(tab);
    setSelectedItemId(null);
    selectedItemIdRef.current = null;
    if (tab === "phrases") {
      const { groups, loadPhrases } = usePhraseStore.getState();
      if (groups.length > 0) {
        const firstId = groups[0].id;
        setPhraseGroupId(firstId);
        phraseGroupIdRef.current = firstId;
        loadPhrases(firstId);
      }
    }
  }, []);

  const handleCategorySwitch = useCallback((key: string) => {
    if (activeTabRef.current === "clipboard") {
      setClipboardCategory(key as ClipType);
      clipboardCategoryRef.current = key as ClipType;
    } else {
      setPhraseGroupId(key);
      phraseGroupIdRef.current = key;
      usePhraseStore.getState().loadPhrases(key);
    }
    setSelectedItemId(null);
    selectedItemIdRef.current = null;
  }, []);

  const handleItemClick = useCallback(async (itemId: string) => {
    const { records, pasteRecord } = useClipboardStore.getState();
    const record = records.find((r) => r.id === itemId);
    if (record) {
      await pasteRecord(record);
      hide();
      return;
    }
    const { phrases, pastePhrase } = usePhraseStore.getState();
    const phrase = phrases.find((p) => p.id === itemId);
    if (phrase) {
      await pastePhrase(phrase);
      hide();
    }
  }, [hide]);

  useEffect(() => {
    let unlisteners: UnlistenFn[] = [];

    const setup = async () => {
      const unDown = await listen<{ x: number; y: number; theme: string }>("radial-menu-down", async (e) => {
        // Reload settings each time before showing, since the main window
        // and radial menu have separate zustand store instances.
        await useSettingsStore.getState().loadSettings();
        document.documentElement.setAttribute("data-theme", resolveTheme(e.payload.theme as ThemeMode));
        showTimestampRef.current = Date.now();
        setVisible(true);
      });

      unlisteners = [unDown];
    };

    setup();

    const handleContextMenu = (e: Event) => {
      e.preventDefault();
    };

    const handleWheel = (e: WheelEvent) => {
      if (!visible) return;

      const el = document.elementFromPoint(e.clientX, e.clientY);
      if (!el) return;

      const catContainer = (el as HTMLElement).closest("[data-radial-categories]");
      if (catContainer) {
        catContainer.scrollLeft += e.deltaY;
        return;
      }

      const listContainer = (el as HTMLElement).closest("[data-radial-list]");
      if (listContainer) {
        listContainer.scrollTop += e.deltaY;
      }
    };

    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key === "Escape") {
        hide();
      }
    };

    const handleBlur = () => {
      if (Date.now() - showTimestampRef.current < 500) return;
      hide();
    };

    document.addEventListener("contextmenu", handleContextMenu, true);
    document.addEventListener("wheel", handleWheel, { passive: false });
    document.addEventListener("keydown", handleKeyDown);
    window.addEventListener("blur", handleBlur);

    return () => {
      unlisteners.forEach((fn) => fn());
      document.removeEventListener("contextmenu", handleContextMenu, true);
      document.removeEventListener("wheel", handleWheel);
      document.removeEventListener("keydown", handleKeyDown);
      window.removeEventListener("blur", handleBlur);
    };
  }, [visible, hide]);

  const records = useClipboardStore((s) => s.records);
  const phraseGroups = usePhraseStore((s) => s.groups);
  const phrases = usePhraseStore((s) => s.phrases);
  const loadPhrases = usePhraseStore((s) => s.loadPhrases);
  const clickMode = useSettingsStore((s) => s.clickMode);
  const clickModeRef = useRef(clickMode);
  useEffect(() => { clickModeRef.current = clickMode; }, [clickMode]);

  useEffect(() => {
    if (visible && activeTab === "phrases" && !phraseGroupId && phraseGroups.length > 0) {
      const firstId = phraseGroups[0].id;
      setPhraseGroupId(firstId);
      phraseGroupIdRef.current = firstId;
      loadPhrases(firstId);
    }
  }, [visible, activeTab, phraseGroupId, phraseGroups, loadPhrases]);

  const filteredRecords = clipboardCategory === "all"
    ? records
    : records.filter((r) => r.type === clipboardCategory);

  const items = activeTab === "clipboard"
    ? filteredRecords.slice(0, MAX_ITEMS).map((r) => ({
        id: r.id,
        content: r.type === "image"
          ? `[${t("clipboard.image")}]`
          : r.type === "file"
            ? r.content.replace(/\\/g, "/").split("/").pop() || r.content
            : r.content,
        type: r.type,
        createdAt: r.created_at,
      }))
    : phrases.map((p) => ({
        id: p.id,
        content: p.content,
        type: "phrase" as string,
        title: p.title,
      }));

  const categories = activeTab === "clipboard"
    ? [
        { key: "all", label: t("clipboard.all") },
        { key: "text", label: t("clipboard.text") },
        { key: "image", label: t("clipboard.image") },
        { key: "link", label: t("clipboard.link") },
        { key: "file", label: t("clipboard.file") },
      ]
    : phraseGroups.map((g) => ({
        key: g.id,
        label: g.name,
      }));

  const activeCategory = activeTab === "clipboard" ? clipboardCategory : phraseGroupId;

  return (
    <div className={`radial-menu-overlay${visible ? "" : " radial-menu-hidden"}`}>
      <div className="radial-menu-popup">
        <div className="radial-menu-nav">
          {(["clipboard", "phrases"] as TabKey[]).map((tab) => (
            <button
              key={tab}
              className={`radial-menu-nav-tab ${activeTab === tab ? "active" : ""}`}
              data-radial-nav={tab}
              onMouseEnter={() => handleTabSwitch(tab)}
              onClick={() => handleTabSwitch(tab)}
            >
              <span className="radial-menu-nav-label">{t(`tabs.${tab}`)}</span>
            </button>
          ))}
        </div>

        {categories.length > 0 && (
          <div className="radial-menu-categories" data-radial-categories>
            {categories.map((cat) => (
              <button
                key={cat.key}
                className={`radial-menu-category-chip ${activeCategory === cat.key ? "active" : ""}`}
                data-radial-category={cat.key}
                onMouseEnter={() => handleCategorySwitch(cat.key)}
                onClick={() => handleCategorySwitch(cat.key)}
              >
                {cat.label}
              </button>
            ))}
          </div>
        )}

        <div className="radial-menu-list" data-radial-list>
          {items.length === 0 ? (
            <div className="radial-menu-empty">{t("radialMenu.empty")}</div>
          ) : (
            items.map((item) => (
              <div
                key={item.id}
                className={`radial-menu-item ${selectedItemId === item.id ? "selected" : ""}`}
                data-radial-item-id={item.id}
                onMouseEnter={() => {
                  setSelectedItemId(item.id);
                  selectedItemIdRef.current = item.id;
                }}
                onMouseLeave={() => {
                  if (selectedItemIdRef.current === item.id) {
                    setSelectedItemId(null);
                    selectedItemIdRef.current = null;
                  }
                }}
                onClick={() => {
                  console.error("[radial] click, clickMode=", clickModeRef.current);
                  if (clickModeRef.current !== "double") {
                    handleItemClick(item.id);
                  }
                }}
                onDoubleClick={() => {
                  console.error("[radial] dblclick, clickMode=", clickModeRef.current);
                  if (clickModeRef.current === "double") {
                    handleItemClick(item.id);
                  }
                }}
              >
                {item.type === "image" ? (
                  <ImageThumb recordId={item.id} />
                ) : (
                  <span className="radial-menu-item-text">
                    {item.content.length > 80
                      ? item.content.slice(0, 80) + "…"
                      : item.content}
                  </span>
                )}
                {"createdAt" in item && item.createdAt && (
                  <span className="radial-menu-item-time">{formatTime(item.createdAt)}</span>
                )}
                {"title" in item && item.title && (
                  <span className="radial-menu-item-remark">{item.title}</span>
                )}
              </div>
            ))
          )}
        </div>
      </div>
    </div>
  );
}
