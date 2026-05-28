import { useTranslation } from "react-i18next";
import IosSelect from "../IosSelect";

interface BasicSettingsSectionProps {
  themeMode: string;
  onThemeChange: (theme: string) => void;
  language: string;
  onLanguageChange: (lang: string) => void;
  autostartEnabled: boolean;
  onAutostartChange: (enabled: boolean) => void;
  minimizeToTray: boolean;
  onMinimizeToTrayChange: (enabled: boolean) => void;
  toastEnabled: boolean;
  onToastChange: (enabled: boolean) => void;
}

export function BasicSettingsSection({
  themeMode,
  onThemeChange,
  language,
  onLanguageChange,
  autostartEnabled,
  onAutostartChange,
  minimizeToTray,
  onMinimizeToTrayChange,
  toastEnabled,
  onToastChange,
}: BasicSettingsSectionProps) {
  const { t } = useTranslation();

  const themeOptions = [
    { value: "light", label: t("settings.light") },
    { value: "dark-solid", label: t("settings.darkSolid") },
    { value: "deep-blue", label: t("settings.deepBlue") },
    { value: "dark", label: t("settings.dark") },
    { value: "auto", label: t("settings.auto") },
  ];

  const languageOptions = [
    { value: "zh-CN", label: "简体中文" },
    { value: "zh-HK", label: "繁體中文（香港）" },
    { value: "zh-TW", label: "繁體中文（台灣）" },
    { value: "en-US", label: "English" },
    { value: "ja-JP", label: "日本語" },
    { value: "de-DE", label: "Deutsch" },
    { value: "fr-FR", label: "Français" },
    { value: "es-ES", label: "Español" },
    { value: "it-IT", label: "Italiano" },
    { value: "pt-BR", label: "Português (Brasil)" },
    { value: "ru-RU", label: "Русский" },
    { value: "ko-KR", label: "한국어" },
    { value: "th-TH", label: "ไทย" },
    { value: "vi-VN", label: "Tiếng Việt" },
    { value: "id-ID", label: "Bahasa Indonesia" },
    { value: "ms-MY", label: "Bahasa Melayu" },
    { value: "hi-IN", label: "हिन्दी" },
  ];

  return (
    <div className="settings-section">
      <div className="settings-section-title">{t("settings.basicSettings")}</div>
      <div className="settings-card">
        <div className="settings-row">
          <div className="settings-row-label">{t("settings.theme")}</div>
          <IosSelect
            value={themeMode}
            options={themeOptions}
            onChange={onThemeChange}
          />
        </div>
        <div className="settings-row">
          <div className="settings-row-label">{t("settings.language")}</div>
          <IosSelect
            value={language}
            options={languageOptions}
            onChange={onLanguageChange}
          />
        </div>
        <div className="settings-row">
          <div className="settings-row-label">{t("settings.startup")}</div>
          <button
            className={`toggle-switch ${autostartEnabled ? "on" : "off"}`}
            onClick={() => onAutostartChange(!autostartEnabled)}
            title={autostartEnabled ? t("common.on") : t("common.off")}
          >
            <span className="toggle-thumb" />
          </button>
        </div>
        <div className="settings-row">
          <div className="settings-row-label">{t("settings.minimizeToTray")}</div>
          <button
            className={`toggle-switch ${minimizeToTray ? "on" : "off"}`}
            onClick={() => onMinimizeToTrayChange(!minimizeToTray)}
            title={minimizeToTray ? t("common.on") : t("common.off")}
          >
            <span className="toggle-thumb" />
          </button>
        </div>
        <div className="settings-row">
          <div className="settings-row-label">{t("settings.toastPaste")}</div>
          <button
            className={`toggle-switch ${toastEnabled ? "on" : "off"}`}
            onClick={() => onToastChange(!toastEnabled)}
            title={toastEnabled ? t("common.on") : t("common.off")}
          >
            <span className="toggle-thumb" />
          </button>
        </div>
      </div>
    </div>
  );
}
