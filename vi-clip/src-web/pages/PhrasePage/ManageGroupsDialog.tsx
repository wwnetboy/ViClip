import { createPortal } from "react-dom";
import { useTranslation } from "react-i18next";
import { Icons } from "../../components/Icons";

interface PhraseGroup {
  id: string;
  name: string;
  sort_order: number;
  created_at: string;
  updated_at: string;
}

interface ManageGroupsDialogProps {
  open: boolean;
  groups: PhraseGroup[];
  renameId: string | null;
  renameName: string;
  setRenameName: (name: string) => void;
  onStartRename: (id: string, name: string) => void;
  onRename: () => void;
  onDeleteGroup: (id: string) => void;
  onClose: () => void;
}

export function ManageGroupsDialog({
  open,
  groups,
  renameId,
  renameName,
  setRenameName,
  onStartRename,
  onRename,
  onDeleteGroup,
  onClose,
}: ManageGroupsDialogProps) {
  const { t } = useTranslation();

  if (!open) return null;

  return createPortal(
    <div className="dialog-overlay" onClick={onClose}>
      <div className="dialog-content large" onClick={(e) => e.stopPropagation()}>
        <h3 className="dialog-title">{t("phrases.manageGroups")}</h3>
        <div className="phrase-group-manage-list">
          {groups.map((g) => (
            <div key={g.id} className="phrase-group-manage-row">
              {renameId === g.id ? (
                <input
                  className="dialog-input"
                  autoFocus
                  value={renameName}
                  onChange={(e) => setRenameName(e.target.value)}
                  onKeyDown={(e) => {
                    if (e.key === "Enter") onRename();
                    if (e.key === "Escape") {
                      onStartRename("", "");
                    }
                  }}
                  onBlur={onRename}
                />
              ) : (
                <span className="phrase-group-manage-name">{g.name}</span>
              )}
              <div className="phrase-group-manage-actions">
                <button
                  className="card-edit-btn"
                  style={{ opacity: 1 }}
                  onClick={() => onStartRename(g.id, g.name)}
                  title={t("phrases.rename")}
                >
                  {Icons.edit}
                </button>
                <button
                  className="card-delete-btn"
                  style={{ opacity: 1 }}
                  onClick={() => onDeleteGroup(g.id)}
                  title={t("common.delete")}
                >
                  {Icons.delete}
                </button>
              </div>
            </div>
          ))}
        </div>
      </div>
    </div>,
    document.body
  );
}
