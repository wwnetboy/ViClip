import { useState } from "react";
import { createPortal } from "react-dom";
import { useTranslation } from "react-i18next";
import { invoke } from "@tauri-apps/api/core";
import { relaunch } from "@tauri-apps/plugin-process";
import { emit } from "@tauri-apps/api/event";
import IosSelect from "../IosSelect";

interface StorageSectionProps {
  storagePath: string;
  setStoragePath: (path: string) => void;
  retention: string;
  onRetentionChange: (retention: string) => void;
}

export function StorageSection({
  storagePath,
  setStoragePath,
  retention,
  onRetentionChange,
}: StorageSectionProps) {
  const { t } = useTranslation();
  const [needRestart, setNeedRestart] = useState(false);
  const [clearing, setClearing] = useState(false);
  const [cleared, setCleared] = useState(false);
  const [showConfirm, setShowConfirm] = useState(false);

  const handleClear = async () => {
    setShowConfirm(false);
    setClearing(true);
    try {
      await invoke("clear_all_records");
      emit("clipboard-update");
      emit("phrase-groups-changed");
      setCleared(true);
      setTimeout(() => setCleared(false), 2000);
    } catch (e) {
      console.error("Failed to clear records:", e);
    }
    setClearing(false);
  };

  const retentionOptions = [
    { value: "1week", label: t("settings.retention1week") },
    { value: "1month", label: t("settings.retention1month") },
    { value: "3months", label: t("settings.retention3months") },
    { value: "6months", label: t("settings.retention6months") },
    { value: "1year", label: t("settings.retention1year") },
    { value: "forever", label: t("settings.retentionForever") },
  ];

  return (
    <div className="settings-section">
      <div className="settings-section-title">{t("settings.storage")}</div>
      <div className="settings-card">
        <div className="settings-row vertical">
          <div className="settings-row-label">{t("settings.storagePath")}</div>
          <div className="settings-storage-row">
            <span className="settings-storage-path">{storagePath}</span>
            <button
              className="settings-storage-btn"
              onClick={async () => {
                try {
                  const folder = await invoke<string>("select_storage_folder");
                  await invoke("set_setting", { key: "storage_path", value: folder });
                  setStoragePath(folder);
                  setNeedRestart(true);
                } catch {
                  console.error("Failed to select storage folder");
                }
              }}
            >
              {t("settings.changeFolder")}
            </button>
          </div>
          <div className="settings-storage-hint">
            {t("settings.storagePathHint")}
          </div>
          {needRestart && (
            <div className="settings-restart-hint">
              <span>{t("settings.restartHint")}</span>
              <button
                className="settings-restart-btn"
                onClick={() => relaunch()}
              >
                {t("settings.restartNow")}
              </button>
            </div>
          )}
        </div>
        <div className="settings-row">
          <div className="settings-row-label">{t("settings.fileRetention")}</div>
          <IosSelect
            value={retention}
            options={retentionOptions}
            onChange={onRetentionChange}
          />
        </div>
        <div className="settings-row">
          <div className="settings-row-label">{t("settings.clearAllRecords")}</div>
          <button
            className={`settings-clear-btn${cleared ? " cleared" : ""}`}
            disabled={clearing}
            onClick={() => setShowConfirm(true)}
          >
            {cleared ? t("common.cleared") : clearing ? t("common.loading") : t("common.clear")}
          </button>
        </div>
      </div>

      {showConfirm && createPortal(
        <div className="dialog-overlay" onClick={() => setShowConfirm(false)}>
          <div className="dialog-content" onClick={(e) => e.stopPropagation()}>
            <h3 className="dialog-title">{t("settings.clearAllRecords")}</h3>
            <p style={{ fontSize: "var(--text-sm)", color: "var(--text-secondary)", margin: "0 0 16px", lineHeight: 1.5 }}>
              {t("settings.clearAllConfirm")}
            </p>
            <div className="dialog-actions">
              <button className="dialog-btn secondary" onClick={() => setShowConfirm(false)}>
                {t("common.cancel")}
              </button>
              <button className="dialog-btn primary" onClick={handleClear}>
                {t("common.confirm")}
              </button>
            </div>
          </div>
        </div>,
        document.body
      )}
    </div>
  );
}
