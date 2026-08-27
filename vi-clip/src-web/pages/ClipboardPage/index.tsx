import { useEffect, useCallback, useMemo, useRef, useState } from "react";
import { Virtuoso, type VirtuosoHandle } from "react-virtuoso";
import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
import { useTranslation } from "react-i18next";
import { useClipboardStore } from "../../stores/clipboardStore";
import { Icons } from "../../components/Icons";
import SearchInput from "../../components/SearchInput";
import { ClipboardCard } from "./ClipboardCard";
import { TYPE_META } from "./utils";

type ClipType = "all" | "text" | "image" | "link" | "file";

TYPE_META.text.icon = Icons.clipboard;
TYPE_META.image.icon = Icons.image;
TYPE_META.link.icon = Icons.link;
TYPE_META.file.icon = Icons.file;

// Stable identity so react-virtuoso doesn't remount the list on re-render.
function ListFooter() {
  const loading = useClipboardStore((s) => s.loading);
  const hasMore = useClipboardStore((s) => s.hasMore);
  const recordsLen = useClipboardStore((s) => s.records.length);
  if (!loading || !hasMore || recordsLen === 0) return null;
  return (
    <div className="list-footer-loading" aria-hidden="true">
      <span />
      <span />
      <span />
    </div>
  );
}
const virtuosoComponents = { Footer: ListFooter };

export default function ClipboardPage() {
  const { t } = useTranslation();
  const {
    records,
    search,
    loading,
    category,
    init,
    setSearch,
    setCategory,
    loadRecords,
    loadMore,
    deleteRecord,
    pasteRecord,
  } = useClipboardStore();

  const virtuosoRef = useRef<VirtuosoHandle>(null);

  // Keyboard navigation highlight + capsule state. Refs mirror these
  // so the window-level keydown listener never needs rebinding.
  const [highlightedId, setHighlightedId] = useState<string | null>(null);
  const [unseenCount, setUnseenCount] = useState(0);
  const [atTop, setAtTop] = useState(true);
  // Which card the pointer is over — drives the "retreat" of the card
  // above it (CSS :has() proved unreliable in this webview).
  const [hoveredId, setHoveredId] = useState<string | null>(null);
  // Refs mirror state/list so listeners and effects never need rebinding.
  const highlightedIdRef = useRef<string | null>(null);
  const atTopRef = useRef(true);
  // Newest record id the user has actually seen (was under their eyes).
  const topSeenIdRef = useRef<string | null>(null);

  const categories: { key: ClipType; label: string }[] = [
    { key: "all", label: t("clipboard.all") },
    { key: "text", label: t("clipboard.text") },
    { key: "image", label: t("clipboard.image") },
    { key: "link", label: t("clipboard.link") },
    { key: "file", label: t("clipboard.file") },
  ];

  const labels: Record<string, string> = useMemo(
    () => ({
      text: t("clipboard.text"),
      image: t("clipboard.image"),
      link: t("clipboard.link"),
      file: t("clipboard.file"),
    }),
    [t],
  );

  const getTypeLabel = useCallback(
    (type: string): string => labels[type] || labels.text,
    [labels],
  );

  const handlePaste = useCallback(
    (r: typeof records[number]) => pasteRecord(r),
    [pasteRecord],
  );

  const handleDelete = useCallback(
    (id: string) => deleteRecord(id),
    [deleteRecord],
  );

  const filtered = useMemo(
    () => (category === "all" ? records : records.filter((r) => r.type === category)),
    [records, category],
  );
  const filteredRef = useRef(filtered);
  useEffect(() => {
    filteredRef.current = filtered;
  }, [filtered]);
  useEffect(() => {
    highlightedIdRef.current = highlightedId;
  }, [highlightedId]);

  // Fresh-corpus reset shared by search / category switches.
  const resetForFreshLoad = useCallback(() => {
    setHighlightedId(null);
    setUnseenCount(0);
    setHoveredId(null);
    topSeenIdRef.current = null;
  }, []);

  useEffect(() => {
    init();
  }, [init]);

  const searchMountedRef = useRef(false);
  useEffect(() => {
    if (!searchMountedRef.current) {
      searchMountedRef.current = true;
      return;
    }
    const timer = setTimeout(() => {
      resetForFreshLoad();
      loadRecords();
    }, 300);
    return () => clearTimeout(timer);
  }, [search, loadRecords, resetForFreshLoad]);

  // Category switches refetch with a type filter now that the list is
  // paginated — client-side filtering can no longer see unloaded pages.
  const categoryMountedRef = useRef(false);
  useEffect(() => {
    if (!categoryMountedRef.current) {
      categoryMountedRef.current = true;
      return;
    }
    resetForFreshLoad();
    loadRecords();
  }, [category, loadRecords, resetForFreshLoad]);

  // Summoning the window (or entering the page) starts at the newest item.
  useEffect(() => {
    const resetToLatest = () => {
      setHighlightedId(null);
      setUnseenCount(0);
      topSeenIdRef.current = useClipboardStore.getState().records[0]?.id ?? null;
      virtuosoRef.current?.scrollToIndex(0);
    };
    resetToLatest();

    let disposed = false;
    const promise = getCurrentWebviewWindow().listen<void>(
      "main-window-shown",
      () => {
        if (!disposed) resetToLatest();
      },
    );
    return () => {
      disposed = true;
      void promise.then((unlisten) => unlisten());
    };
  }, []);

  // Window-level keyboard navigation.
  useEffect(() => {
    const onKeyDown = (e: KeyboardEvent) => {
      const el = e.target as HTMLElement | null;
      if (
        el &&
        (el.tagName === "INPUT" || el.tagName === "TEXTAREA" || el.isContentEditable)
      ) {
        return;
      }

      if (e.key === "Escape") {
        // Let an open context menu consume Escape (it listens on document
        // and closes itself); only hide the window when none is open.
        if (!document.querySelector(".context-menu")) {
          e.preventDefault();
          void getCurrentWebviewWindow().hide();
        }
        return;
      }

      const list = filteredRef.current;
      if (list.length === 0) return;

      if (e.key === "ArrowDown" || e.key === "ArrowUp") {
        e.preventDefault();
        const curIdx = highlightedIdRef.current
          ? list.findIndex((r) => r.id === highlightedIdRef.current)
          : -1;
        const next =
          curIdx === -1
            ? e.key === "ArrowDown"
              ? 0
              : list.length - 1
            : e.key === "ArrowDown"
              ? Math.min(curIdx + 1, list.length - 1)
              : Math.max(curIdx - 1, 0);
        setHighlightedId(list[next].id);
        virtuosoRef.current?.scrollToIndex({ index: next, align: "center" });
      } else if (e.key === "Enter") {
        e.preventDefault();
        const target =
          list.find((r) => r.id === highlightedIdRef.current) || list[0];
        pasteRecord(target);
      } else if (e.key === "Delete") {
        if (highlightedIdRef.current) {
          e.preventDefault();
          deleteRecord(highlightedIdRef.current);
        }
      }
    };
    window.addEventListener("keydown", onKeyDown);
    return () => window.removeEventListener("keydown", onKeyDown);
  }, [pasteRecord, deleteRecord]);

  // Track how many new records arrived above the viewport while the user
  // was scrolled away. While at top, prepends are seen instantly and the
  // anchor simply follows the head.
  useEffect(() => {
    const head = records[0]?.id ?? null;
    if (!head || atTopRef.current) return;
    const prev = topSeenIdRef.current;
    if (!prev || prev === head) {
      topSeenIdRef.current = head;
      return;
    }
    const idx = records.findIndex((r) => r.id === prev);
    if (idx > 0) setUnseenCount((u) => u + idx);
    topSeenIdRef.current = head;
  }, [records]);

  const jumpToLatest = useCallback(() => {
    setUnseenCount(0);
    topSeenIdRef.current = useClipboardStore.getState().records[0]?.id ?? null;
    setAtTop(true);
    atTopRef.current = true;
    virtuosoRef.current?.scrollToIndex(0);
  }, []);

  const handleAtTopChange = useCallback((isAtTop: boolean) => {
    atTopRef.current = isAtTop;
    setAtTop(isAtTop);
    if (isAtTop) {
      topSeenIdRef.current = useClipboardStore.getState().records[0]?.id ?? null;
      setUnseenCount(0);
    }
  }, []);

  // When a card is hovered, ALL cards above it slide up together as a
  // rigid block (mutual gaps unchanged) — matching the real layout push
  // on the cards below — so the whole list reads as "breathing open".
  // (index comes from Virtuoso's itemContent, not from findIndex.)
  const hoveredIdx = hoveredId ? filtered.findIndex((r) => r.id === hoveredId) : -1;
  const handleHoverChange = useCallback(
    (record: typeof records[number] | null) => {
      setHoveredId(record ? record.id : null);
    },
    [],
  );

  return (
    <div className="clipboard-page">
      <div className="page-search">
        <SearchInput
          placeholder={t("clipboard.search")}
          value={search}
          onChange={setSearch}
        />
      </div>

      <div className="clipboard-categories">
        {categories.map((c) => (
          <button
            key={c.key}
            className={`category-chip ${category === c.key ? "active" : ""}`}
            onClick={() => setCategory(c.key)}
          >
            {c.label}
          </button>
        ))}
      </div>

      {loading && records.length === 0 ? (
        <div className="clipboard-list">
          {[1, 2, 3, 4].map((i) => (
            <div key={i} className="notification skeleton">
              <div className="noticontent">
                <div className="notititle">
                  <div className="skeleton-line short" />
                </div>
                <div className="notibody">
                  <div
                    className="skeleton-line"
                    style={{ width: `${55 + ((i * 17) % 35)}%` }}
                  />
                </div>
              </div>
            </div>
          ))}
        </div>
      ) : filtered.length === 0 ? (
        <div className="page-empty-compact">
          <div className="empty-icon-compact">{Icons.clipboard}</div>
          <span>{t("clipboard.empty")}</span>
        </div>
      ) : (
        <div className="clipboard-list-wrap">
          {unseenCount > 0 && !atTop && (
            <button className="new-records-pill" onClick={jumpToLatest}>
              <span aria-hidden="true">↑</span>
              <span>{unseenCount}</span>
            </button>
          )}
          <Virtuoso
            className="clipboard-list"
            data={filtered}
            computeItemKey={(_, item) => item.id}
            endReached={() => void loadMore()}
            atTopStateChange={handleAtTopChange}
            components={virtuosoComponents}
            itemContent={(itemIndex, record) => (
              <ClipboardCard
                record={record}
                highlighted={record.id === highlightedId}
                retreat={hoveredIdx > -1 && itemIndex < hoveredIdx}
                onHoverChange={handleHoverChange}
                getTypeLabel={getTypeLabel}
                onPaste={handlePaste}
                onDelete={handleDelete}
              />
            )}
          />
        </div>
      )}

    </div>
  );
}
