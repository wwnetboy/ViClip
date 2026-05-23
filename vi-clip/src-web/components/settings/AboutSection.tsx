import { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { invoke } from "@tauri-apps/api/core";

interface AppInfo {
  name: string;
  version: string;
  author: string;
  copyright: string;
}

export function AboutSection() {
  const { t } = useTranslation();
  const [appInfo, setAppInfo] = useState<AppInfo | null>(null);

  useEffect(() => {
    invoke<AppInfo>("get_app_info").then(setAppInfo).catch(console.error);
  }, []);

  return (
    <div className="settings-section">
      <div className="settings-section-title">{t("settings.about")}</div>
      <div className="settings-card">
        <div className="settings-row">
          <div className="settings-row-label">{t("settings.version")}</div>
          <span style={{ color: "var(--text-secondary)", fontSize: "var(--text-sm)" }}>
            {appInfo?.version ?? "—"}
          </span>
        </div>
        <div className="settings-row">
          <div className="settings-row-label">{t("settings.copyright")}</div>
          <span style={{ color: "var(--text-secondary)", fontSize: "var(--text-sm)" }}>
            {appInfo?.copyright ?? "—"}
          </span>
        </div>
      </div>
    </div>
  );
}
