import { useEffect, useCallback, useMemo } from "react";
import { Virtuoso } from "react-virtuoso";
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
    deleteRecord,
    pasteRecord,
  } = useClipboardStore();

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

  useEffect(() => {
    init();
  }, []);

  useEffect(() => {
    const timer = setTimeout(() => loadRecords(), 300);
    return () => clearTimeout(timer);
  }, [search]);

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
        <Virtuoso
          className="clipboard-list"
          data={filtered}
          computeItemKey={(_, item) => item.id}
          itemContent={(_, record) => (
            <ClipboardCard
              key={record.id}
              record={record}
              index={0}
              getTypeLabel={getTypeLabel}
              onPaste={handlePaste}
              onDelete={handleDelete}
            />
          )}
        />
      )}

    </div>
  );
}
