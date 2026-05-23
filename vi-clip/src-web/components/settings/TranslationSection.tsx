import { useTranslation } from "react-i18next";
import IosSelect from "../IosSelect";

interface TranslationSectionProps {
  engine: string;
  onEngineChange: (engine: string) => void;
  apiUrl: string;
  onApiUrlChange: (url: string) => void;
  apiKey: string;
  onApiKeyChange: (key: string) => void;
  model: string;
  onModelChange: (model: string) => void;
  googleApiKey: string;
  onGoogleApiKeyChange: (key: string) => void;
  translateProxy: string;
  onTranslateProxyChange: (proxy: string) => void;
}

export function TranslationSection({
  engine,
  onEngineChange,
  apiUrl,
  onApiUrlChange,
  apiKey,
  onApiKeyChange,
  model,
  onModelChange,
  googleApiKey,
  onGoogleApiKeyChange,
  translateProxy,
  onTranslateProxyChange,
}: TranslationSectionProps) {
  const { t } = useTranslation();

  const engineOptions = [
    { value: "google", label: t("settings.googleTranslation") },
    { value: "ai", label: t("settings.aiTranslation") },
  ];

  return (
    <div className="settings-section">
      <div className="settings-section-title">{t("settings.translation")}</div>
      <div className="settings-card">
        <div className="settings-row">
          <div className="settings-row-label">{t("settings.defaultEngine")}</div>
          <IosSelect
            value={engine}
            options={engineOptions}
            onChange={onEngineChange}
          />
        </div>
        {engine === "google" && (
          <>
            <div className="settings-row vertical">
              <div className="settings-row-label">{t("settings.googleApiKey")}</div>
              <input
                className="settings-input"
                type="password"
                value={googleApiKey}
                onChange={(e) => onGoogleApiKeyChange(e.target.value)}
                placeholder={t("settings.googleNote")}
              />
            </div>
            <div className="settings-row vertical">
              <div className="settings-row-label">{t("settings.translateProxy")}</div>
              <input
                className="settings-input"
                value={translateProxy}
                onChange={(e) => onTranslateProxyChange(e.target.value)}
                placeholder={t("settings.translateProxyPlaceholder")}
              />
            </div>
          </>
        )}

        {engine === "ai" && (
          <>
            <div className="settings-row vertical">
              <div className="settings-row-label">{t("settings.apiUrl")}</div>
              <input
                className="settings-input"
                value={apiUrl}
                onChange={(e) => onApiUrlChange(e.target.value)}
                placeholder={t("settings.apiUrlPlaceholder")}
              />
            </div>
            <div className="settings-row vertical">
              <div className="settings-row-label">{t("settings.apiKey")}</div>
              <input
                className="settings-input"
                type="password"
                value={apiKey}
                onChange={(e) => onApiKeyChange(e.target.value)}
                placeholder={t("settings.apiKey")}
              />
            </div>
            <div className="settings-row vertical">
              <div className="settings-row-label">{t("settings.model")}</div>
              <input
                className="settings-input"
                value={model}
                onChange={(e) => onModelChange(e.target.value)}
                placeholder={t("settings.model")}
              />
            </div>
          </>
        )}
      </div>
    </div>
  );
}
