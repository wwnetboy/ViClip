import { useTranslation } from "react-i18next";
import { Icons } from "../../components/Icons";
import { useSettingsStore } from "../../stores/settingsStore";

interface Phrase {
  id: string;
  group_id: string;
  title: string;
  content: string;
  sort_order: number;
  created_at: string;
  updated_at: string;
}

interface PhraseListProps {
  phrases: Phrase[];
  loading: boolean;
  selectedGroupId: string | null;
  onPaste: (phrase: Phrase) => void;
  onEdit: (phrase: Phrase) => void;
  onDelete: (id: string) => void;
}

export function PhraseList({
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
        <div
          key={p.id}
          className="notification phrase-card"
          style={{ "--enter-delay": i } as React.CSSProperties}
          {...(clickMode === "double"
            ? { onDoubleClick: () => onPaste(p) }
            : { onClick: () => onPaste(p) })}
        >
          <div className="noticontent">
            <div className="notibody phrase-card-body">{p.content}</div>
            <div className="notititle phrase-card-footer">
              <span className="phrase-card-remark">{p.title}</span>
              <div className="phrase-card-actions">
                <button
                  className="card-edit-btn"
                  onClick={(e) => {
                    e.stopPropagation();
                    onEdit(p);
                  }}
                >
                  {Icons.edit}
                </button>
                <button
                  className="card-delete-btn"
                  onClick={(e) => {
                    e.stopPropagation();
                    onDelete(p.id);
                  }}
                >
                  {Icons.delete}
                </button>
              </div>
            </div>
          </div>
        </div>
      ))}
    </div>
  );
}
