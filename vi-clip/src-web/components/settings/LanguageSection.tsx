import { useTranslation } from "react-i18next";

interface LanguageSectionProps {
  localLang: string;
  setLocalLang: (lang: string) => void;
}

export function LanguageSection({
  localLang,
  setLocalLang,
}: LanguageSectionProps) {
  const { t } = useTranslation();

  return (
    <div className="settings-section">
      <div className="settings-section-title">{t("settings.language")}</div>
      <div className="settings-card">
        <div className="settings-row">
          <div className="settings-row-label">{t("settings.language")}</div>
          <div className="settings-lang-toggle">
            <button
              className={`lang-toggle-btn${localLang === "zh-CN" ? " active" : ""}`}
              onClick={() => setLocalLang("zh-CN")}
            >
              ZH
            </button>
            <button
              className={`lang-toggle-btn${localLang === "en" ? " active" : ""}`}
              onClick={() => setLocalLang("en")}
            >
              EN
            </button>
          </div>
        </div>
      </div>
    </div>
  );
}
