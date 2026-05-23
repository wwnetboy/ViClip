import { createPortal } from "react-dom";
import { useTranslation } from "react-i18next";

interface PhraseDialogProps {
  open: boolean;
  editingId: string | null;
  phraseRemark: string;
  phraseContent: string;
  phraseError: boolean;
  setPhraseRemark: (remark: string) => void;
  setPhraseContent: (content: string) => void;
  onSave: () => void;
  onClose: () => void;
}

export function PhraseDialog({
  open,
  editingId,
  phraseRemark,
  phraseContent,
  phraseError,
  setPhraseRemark,
  setPhraseContent,
  onSave,
  onClose,
}: PhraseDialogProps) {
  const { t } = useTranslation();

  if (!open) return null;

  return createPortal(
    <div className="dialog-overlay" onClick={onClose}>
      <div className="dialog-content large" onClick={(e) => e.stopPropagation()}>
        <h3 className="dialog-title">
          {editingId ? t("common.edit") : t("phrases.newPhrase")}
        </h3>
        <textarea
          className={`dialog-textarea${phraseError ? " error" : ""}`}
          autoFocus
          placeholder={t("phrases.content")}
          value={phraseContent}
          onChange={(e) => {
            setPhraseContent(e.target.value);
          }}
        />
        {phraseError && (
          <span className="dialog-error-text">{t("phrases.contentRequired")}</span>
        )}
        <input
          className="dialog-input"
          placeholder={t("phrases.remark")}
          value={phraseRemark}
          onChange={(e) => setPhraseRemark(e.target.value)}
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
