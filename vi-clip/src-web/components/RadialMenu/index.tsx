import { useEffect, useRef, useState, useCallback, useMemo, type CSSProperties } from "react";
import { useTranslation } from "react-i18next";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { invoke } from "@tauri-apps/api/core";
import { useClipboardStore, type ClipType } from "../../stores/clipboardStore";
import { usePhraseStore } from "../../stores/phraseStore";
import { resolveTheme, type ThemeMode, useSettingsStore } from "../../stores/settingsStore";
import { useThemeSync } from "../../hooks/useThemeSync";
import { formatTime } from "../../utils";
import { ImageThumb } from "../../pages/ClipboardPage/ImageThumb";
import i18n from "../../i18n";

type TabKey = "clipboard" | "phrases";

const MAX_ITEMS = 2000;

export default function RadialMenu() {
  const { t } = useTranslation();

  const [visible, setVisible] = useState(false);
  const [activeTab, setActiveTab] = useState<TabKey>("clipboard");
  const [selectedItemId, setSelectedItemId] = useState<string | null>(null);
  const [clipboardCategory, setClipboardCategory] = useState<ClipType>("all");
  const [phraseGroupId, setPhraseGroupId] = useState<string | null>(null);
  const [searchQuery, setSearchQuery] = useState("");
  const [searchExpanded, setSearchExpanded] = useState(false);
  // List content switch animation: "out" shows a frozen snapshot of the old
  // list while it fades away, then "in" staggers the new items in.
  const switchKey = `${activeTab}:${phraseGroupId ?? clipboardCategory}`;
  const [listPhase, setListPhase] = useState<"live" | "out" | "in">("live");
  const animKeyRef = useRef(switchKey);
  const listTimerRef = useRef<number | undefined>(undefined);
  const listRef = useRef<HTMLDivElement>(null);

  const focusedRef = useRef(false);
  const selectedItemIdRef = useRef<string | null>(null);
  const activeTabRef = useRef<TabKey>("clipboard");
  const clipboardCategoryRef = useRef<ClipType>("all");
  const phraseGroupIdRef = useRef<string | null>(null);
  const searchExpandedRef = useRef(false);
  const searchQueryRef = useRef("");
  const searchInputRef = useRef<HTMLInputElement>(null);

  useEffect(() => { selectedItemIdRef.current = selectedItemId; }, [selectedItemId]);
  useEffect(() => { activeTabRef.current = activeTab; }, [activeTab]);
  useEffect(() => { clipboardCategoryRef.current = clipboardCategory; }, [clipboardCategory]);
  useEffect(() => { phraseGroupIdRef.current = phraseGroupId; }, [phraseGroupId]);
  useEffect(() => { searchExpandedRef.current = searchExpanded; }, [searchExpanded]);
  useEffect(() => { searchQueryRef.current = searchQuery; }, [searchQuery]);

  useThemeSync();

  useEffect(() => {
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

    let unlistenLang: UnlistenFn | undefined;
    listen<{ language: string }>("language-changed", (e) => {
      if (e.payload.language !== i18n.language) {
        i18n.changeLanguage(e.payload.language);
      }
    }).then((fn) => { unlistenLang = fn; });

    return () => {
      if (unlistenLang) unlistenLang();
    };
  }, []);

  // Collapse the search pill and drop the query — used when leaving search mode
  // (outside click, icon toggle) and whenever the menu is hidden or re-shown.
  const resetSearch = useCallback(() => {
    setSearchExpanded(false);
    setSearchQuery("");
    searchExpandedRef.current = false;
    searchQueryRef.current = "";
  }, []);

  const hide = useCallback(() => {
    setVisible(false);
    setSelectedItemId(null);
    selectedItemIdRef.current = null;
    resetSearch();
    window.clearTimeout(listTimerRef.current);
    setListPhase("live");
    getCurrentWindow().hide();
    invoke("radial_menu_dismissed").catch(() => {});
  }, [resetSearch]);

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
      const unDown = await listen<{ theme: string }>("radial-menu-down", async (e) => {
        document.documentElement.setAttribute("data-theme", resolveTheme(e.payload.theme as ThemeMode));
        focusedRef.current = false;
        resetSearch();
        setVisible(true);
        // Fresh popup: reset scroll and replay the staggered entrance.
        if (listRef.current) listRef.current.scrollTop = 0;
        setListPhase("live");
        window.setTimeout(() => setListPhase("in"), 30);
      });

      // Sync settings from main window via event bus, avoiding per-show IPC
      const unSettings = await listen<Record<string, string>>("settings-changed", (e) => {
        const s = e.payload;
        useSettingsStore.setState((state) => ({
          ...state,
          ...(s.theme !== undefined ? { themeMode: s.theme as ThemeMode } : {}),
          ...(s.language !== undefined ? { language: s.language } : {}),
          ...(s.shortcut_key !== undefined ? { shortcutKey: s.shortcut_key } : {}),
          ...(s.click_mode !== undefined ? { clickMode: s.click_mode } : {}),
          ...(s.radial_menu_enabled !== undefined ? { radialMenuEnabled: s.radial_menu_enabled !== "0" } : {}),
          ...(s.toast_enabled !== undefined ? { toastEnabled: s.toast_enabled !== "0" } : {}),
        }));
      });

      const unDismissed = await listen("radial-menu-dismissed", () => {
        setVisible(false);
        setSelectedItemId(null);
        selectedItemIdRef.current = null;
        resetSearch();
        window.clearTimeout(listTimerRef.current);
        setListPhase("live");
      });

      unlisteners = [unDown, unSettings, unDismissed];
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
        if (searchExpandedRef.current && searchQueryRef.current) {
          setSearchQuery("");
          searchQueryRef.current = "";
          searchInputRef.current?.focus();
        } else if (searchExpandedRef.current) {
          setSearchExpanded(false);
          searchExpandedRef.current = false;
        } else {
          hide();
        }
      }
    };

    const handleFocus = () => {
      focusedRef.current = true;
    };

    const handleBlur = () => {
      // Always hide on blur (user clicked outside or switched apps)
      hide();
    };

    document.addEventListener("contextmenu", handleContextMenu, true);
    document.addEventListener("wheel", handleWheel, { passive: false });
    document.addEventListener("keydown", handleKeyDown);
    window.addEventListener("focus", handleFocus);
    window.addEventListener("blur", handleBlur);

    return () => {
      unlisteners.forEach((fn) => fn());
      document.removeEventListener("contextmenu", handleContextMenu, true);
      document.removeEventListener("wheel", handleWheel);
      document.removeEventListener("keydown", handleKeyDown);
      window.removeEventListener("focus", handleFocus);
      window.removeEventListener("blur", handleBlur);
    };
  }, [visible, hide, resetSearch]);

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

  const filteredRecords = useMemo(() =>
    clipboardCategory === "all"
      ? records
      : records.filter((r) => r.type === clipboardCategory),
    [records, clipboardCategory]);

  const items = useMemo(() => {
    const query = searchQuery.trim().toLowerCase();
    return activeTab === "clipboard"
      ? (query
          ? filteredRecords.filter((r) => r.type !== "image" && r.content.toLowerCase().includes(query))
          : filteredRecords
        ).slice(0, MAX_ITEMS).map((r) => ({
          id: r.id,
          content: r.type === "image"
            ? `[${t("clipboard.image")}]`
            : r.type === "file"
              ? r.content.replace(/\\/g, "/").split("/").pop() || r.content
              : r.content,
          type: r.type,
          createdAt: r.created_at,
        }))
      : (query
          ? phrases.filter((p) =>
              p.content.toLowerCase().includes(query)
              || (p.title && p.title.toLowerCase().includes(query))
            )
          : phrases
        ).map((p) => ({
          id: p.id,
          content: p.content,
          type: "phrase" as string,
          title: p.title,
        }));
  }, [activeTab, filteredRecords, phrases, searchQuery, t]);

  // --- List switch animation (category / tab hover) ---
  // committedRef tracks what is on screen; at switch time it is copied into
  // state (leavingItems) and frozen while the old list fades out.
  const committedRef = useRef(items);
  const [leavingItems, setLeavingItems] = useState(items);

  useEffect(() => {
    if (animKeyRef.current === switchKey) return;
    animKeyRef.current = switchKey;
    window.clearTimeout(listTimerRef.current);
    setLeavingItems(committedRef.current);
    setListPhase("out");
    listTimerRef.current = window.setTimeout(() => {
      if (listRef.current) listRef.current.scrollTop = 0;
      setListPhase("in");
    }, 110);
  }, [switchKey]);

  // Keep the committed snapshot in sync, so the next switch always fades out
  // from what the user actually sees (incl. live search results).
  useEffect(() => {
    if (listPhase !== "out") committedRef.current = items;
  }, [items, listPhase]);

  useEffect(() => () => window.clearTimeout(listTimerRef.current), []);

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
  const displayItems = listPhase === "out" ? leavingItems : items;

  return (
    <div className={`radial-menu-overlay${visible ? "" : " radial-menu-hidden"}`} onMouseDown={hide}>
      <div
        className="radial-menu-popup"
        onMouseDown={(e) => {
          e.stopPropagation();
          // Click outside the search pill collapses it; clicks inside the
          // pill are stopped by .radial-menu-search so typing keeps it open.
          if (searchExpandedRef.current) {
            resetSearch();
          }
        }}
      >
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

        <div
          className={`radial-menu-search${searchExpanded ? " expanded" : ""}`}
          onClick={(e) => e.stopPropagation()}
          onMouseDown={(e) => e.stopPropagation()}
        >
          <div className={`radial-menu-search-pill${searchExpanded ? " expanded" : ""}`}>
            <button
              className="radial-menu-search-icon"
              onClick={() => {
                if (searchExpanded) {
                  resetSearch();
                } else {
                  setSearchExpanded(true);
                  searchExpandedRef.current = true;
                  setTimeout(() => searchInputRef.current?.focus(), 100);
                }
              }}
            >
              <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
                <circle cx="11" cy="11" r="8"/>
                <line x1="21" y1="21" x2="16.65" y2="16.65"/>
              </svg>
            </button>
            <input
              ref={searchInputRef}
              className="radial-menu-search-input"
              type="text"
              placeholder={t("radialMenu.search")}
              value={searchQuery}
              onChange={(e) => setSearchQuery(e.target.value)}
            />
          </div>
        </div>

        <div className="radial-menu-list" data-radial-list ref={listRef}>
          <div
            className={`radial-menu-list-inner${listPhase === "out" ? " leaving" : listPhase === "in" ? " entering" : ""}`}
          >
          {displayItems.length === 0 ? (
            <div className="radial-menu-empty">{t("radialMenu.empty")}</div>
          ) : (
            displayItems.map((item, index) => (
              <div
                key={item.id}
                className={`radial-menu-item ${selectedItemId === item.id ? "selected" : ""}`}
                data-radial-item-id={item.id}
                style={{ "--enter-delay": index } as CSSProperties}
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
                  if (clickModeRef.current !== "double") {
                    handleItemClick(item.id);
                  }
                }}
                onDoubleClick={() => {
                  if (clickModeRef.current === "double") {
                    handleItemClick(item.id);
                  }
                }}
              >
                {item.type === "image" ? (
                  (() => {
                    const imgRecord = records.find((r) => r.id === item.id);
                    return imgRecord ? (
                      <ImageThumb record={imgRecord} onClick={() => {}} />
                    ) : <span className="radial-menu-item-text">…</span>;
                  })()
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
    </div>
  );
}
