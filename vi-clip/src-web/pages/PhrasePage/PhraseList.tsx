import { memo, useCallback, useState } from "react";
import { useTranslation } from "react-i18next";
import { Icons } from "../../components/Icons";
import { useSettingsStore } from "../../stores/settingsStore";
import { ContextMenu, type ContextMenuItem } from "../../components/ContextMenu";
import type { Phrase } from "../../types";

interface PhraseListProps {
  phrases: Phrase[];
  loading: boolean;
  selectedGroupId: string | null;
  onPaste: (phrase: Phrase) => void;
  onEdit: (phrase: Phrase) => void;
  onDelete: (id: string) => void;
}

function PhraseCard({
  phrase,
  index,
  clickMode,
  onPaste,
  onEdit,
  onDelete,
}: {
  phrase: Phrase;
  index: number;
  clickMode: string;
  onPaste: (p: Phrase) => void;
  onEdit: (p: Phrase) => void;
  onDelete: (id: string) => void;
}) {
  const { t } = useTranslation();
  const [menuPos, setMenuPos] = useState<{ x: number; y: number } | null>(null);

  const handleContextMenu = useCallback((e: React.MouseEvent) => {
    e.preventDefault();
    e.stopPropagation();
    setMenuPos({ x: e.clientX, y: e.clientY });
  }, []);

  const handleCopyContent = useCallback(async () => {
    try {
      await navigator.clipboard.writeText(phrase.content);
    } catch {
      const ta = document.createElement("textarea");
      ta.value = phrase.content;
      document.body.appendChild(ta);
      ta.select();
      document.execCommand("copy");
      document.body.removeChild(ta);
    }
  }, [phrase.content]);

  const menuItems: ContextMenuItem[] = [
    {
      key: "paste",
      label: t("contextMenu.paste"),
      icon: Icons.paste,
      onClick: () => onPaste(phrase),
    },
    {
      key: "copyContent",
      label: t("contextMenu.copyContent"),
      icon: Icons.copy,
      onClick: handleCopyContent,
    },
    {
      key: "edit",
      label: t("contextMenu.edit"),
      icon: Icons.edit,
      onClick: () => onEdit(phrase),
    },
    {
      key: "delete",
      label: t("contextMenu.delete"),
      icon: Icons.delete,
      onClick: () => onDelete(phrase.id),
      danger: true,
    },
  ];

  const clickProps =
    clickMode === "double"
      ? { onDoubleClick: () => onPaste(phrase) }
      : { onClick: () => onPaste(phrase) };

  return (
    <>
      <div
        className="notification phrase-card"
        style={{ "--enter-delay": index } as React.CSSProperties}
        onContextMenu={handleContextMenu}
        {...clickProps}
      >
        <div className="noticontent">
          <div className="notibody phrase-card-body">{phrase.content}</div>
          <div className="notititle phrase-card-footer">
            <span className="phrase-card-remark">{phrase.title}</span>
            <div className="phrase-card-actions">
              <button
                className="card-edit-btn"
                onClick={(e) => {
                  e.stopPropagation();
                  onEdit(phrase);
                }}
                title={t("contextMenu.edit")}
              >
                {Icons.edit}
              </button>
              <button
                className="card-delete-btn"
                onClick={(e) => {
                  e.stopPropagation();
                  onDelete(phrase.id);
                }}
                title={t("contextMenu.delete")}
              >
                {Icons.delete}
              </button>
            </div>
          </div>
        </div>
      </div>
      {menuPos && (
        <ContextMenu
          items={menuItems}
          position={menuPos}
          onClose={() => setMenuPos(null)}
        />
      )}
    </>
  );
}

export const PhraseList = memo(function PhraseList({
  phrases,
  loading,
  selectedGroupId,
  onPaste,
  onEdit,
  onDelete,
}: PhraseListProps) {
  const { t } = useTranslation();
  const clickMode = useSettingsStore((s) => s.clickMode);

  if (loading && phrases.length === 0) {
    return (
      <div className="phrase-list">
        {[1, 2, 3, 4].map((i) => (
          <div key={i} className="notification skeleton">
            <div className="noticontent">
              <div className="notibody">
                <div className="skeleton-line" style={{ width: `${40 + ((i * 13) % 30)}%` }} />
              </div>
              <div className="notititle">
                <div className="skeleton-line short" />
              </div>
            </div>
          </div>
        ))}
      </div>
    );
  }

  if (!selectedGroupId) {
    return (
      <div className="page-empty-compact">
        <div className="empty-icon-compact">{Icons.phrases}</div>
        <span>{t("phrases.empty")}</span>
      </div>
    );
  }

  if (phrases.length === 0 && !loading) {
    return (
      <div className="page-empty-compact">
        <span>{t("phrases.emptyGroupPhrases")}</span>
      </div>
    );
  }

  return (
    <div className="phrase-list">
      {phrases.map((p, i) => (
        <PhraseCard
          key={p.id}
          phrase={p}
          index={i}
          clickMode={clickMode}
          onPaste={onPaste}
          onEdit={onEdit}
          onDelete={onDelete}
        />
      ))}
    </div>
  );
});
