import { createPortal } from "react-dom";
import { useTranslation } from "react-i18next";

interface GroupDialogProps {
  open: boolean;
  editingId: string | null;
  groupName: string;
  setGroupName: (name: string) => void;
  onSave: () => void;
  onClose: () => void;
}

export function GroupDialog({
  open,
  editingId,
  groupName,
  setGroupName,
  onSave,
  onClose,
}: GroupDialogProps) {
  const { t } = useTranslation();

  if (!open) return null;

  return createPortal(
    <div className="dialog-overlay" onClick={onClose}>
      <div className="dialog-content" onClick={(e) => e.stopPropagation()}>
        <h3 className="dialog-title">
          {editingId ? t("common.edit") : t("phrases.newGroup")}
        </h3>
        <input
          className="dialog-input"
          autoFocus
          placeholder={t("phrases.groupName")}
          value={groupName}
          onChange={(e) => setGroupName(e.target.value)}
          onKeyDown={(e) => e.key === "Enter" && onSave()}
        />
        <div className="dialog-actions">
          <button className="dialog-btn secondary" onClick={onClose}>
            {t("common.cancel")}
          </button>
          <button className="dialog-btn save" onClick={onSave}>
            {t("common.save")}
          </button>
        </div>
      </div>
    </div>,
    document.body
  );
}
