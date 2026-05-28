import { useTranslation } from "react-i18next";
import IosSelect from "../IosSelect";
import { useSettingsStore } from "../../stores/settingsStore";

interface TranslationSectionProps {
  engine: string;
  onEngineChange: (engine: string) => void;
  persistConfig: (dbKey: string, value: string, toastKey: string) => void;
}

export function TranslationSection({
  engine,
  onEngineChange,
  persistConfig,
}: TranslationSectionProps) {
  const { t } = useTranslation();

  const engineOptions = [
    { value: "google", label: t("settings.googleTranslation") },
    { value: "ai", label: t("settings.aiTranslation") },
    { value: "baidu", label: t("settings.baiduTranslation") },
    { value: "youdao", label: t("settings.youdaoTranslation") },
    { value: "tencent", label: t("settings.tencentTranslation") },
    { value: "volctrans", label: t("settings.volctransTranslation") },
  ];

  const googleApiKey = useSettingsStore((s) => s.googleApiKey);
  const translateProxy = useSettingsStore((s) => s.translateProxy);
  const apiUrl = useSettingsStore((s) => s.apiUrl);
  const apiKey = useSettingsStore((s) => s.apiKey);
  const model = useSettingsStore((s) => s.model);
  const baiduAppid = useSettingsStore((s) => s.baiduAppid);
  const baiduSecretKey = useSettingsStore((s) => s.baiduSecretKey);
  const youdaoAppKey = useSettingsStore((s) => s.youdaoAppKey);
  const youdaoAppSecret = useSettingsStore((s) => s.youdaoAppSecret);
  const tencentSecretId = useSettingsStore((s) => s.tencentSecretId);
  const tencentSecretKey = useSettingsStore((s) => s.tencentSecretKey);
  const volctransAccessKeyId = useSettingsStore((s) => s.volctransAccessKeyId);
  const volctransSecretAccessKey = useSettingsStore((s) => s.volctransSecretAccessKey);

  const field = (
    dbKey: string,
    storeKey: string,
    toastKey: string,
  ) => (value: string) => {
    useSettingsStore.setState({ [storeKey]: value } as any);
    persistConfig(dbKey, value, toastKey);
  };

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

        {/* ── Google ── */}
        {engine === "google" && (
          <>
            <div className="settings-row vertical">
              <div className="settings-row-label">{t("settings.googleApiKey")}</div>
              <input
                className="settings-input"
                type="password"
                value={googleApiKey}
                onChange={(e) => field("google_api_key", "googleApiKey", "settings.toast.googleApiKey")(e.target.value)}
                placeholder={t("settings.googleNote")}
              />
            </div>
            <div className="settings-row vertical">
              <div className="settings-row-label">{t("settings.translateProxy")}</div>
              <input
                className="settings-input"
                value={translateProxy}
                onChange={(e) => field("translate_proxy", "translateProxy", "settings.toast.translateProxy")(e.target.value)}
                placeholder={t("settings.translateProxyPlaceholder")}
              />
            </div>
          </>
        )}

        {/* ── AI ── */}
        {engine === "ai" && (
          <>
            <div className="settings-row vertical">
              <div className="settings-row-label">{t("settings.apiUrl")}</div>
              <input
                className="settings-input"
                value={apiUrl}
                onChange={(e) => field("ai_api_url", "apiUrl", "settings.toast.aiApiUrl")(e.target.value)}
                placeholder={t("settings.apiUrlPlaceholder")}
              />
            </div>
            <div className="settings-row vertical">
              <div className="settings-row-label">{t("settings.apiKey")}</div>
              <input
                className="settings-input"
                type="password"
                value={apiKey}
                onChange={(e) => field("ai_api_key", "apiKey", "settings.toast.aiApiKey")(e.target.value)}
                placeholder={t("settings.apiKey")}
              />
            </div>
            <div className="settings-row vertical">
              <div className="settings-row-label">{t("settings.model")}</div>
              <input
                className="settings-input"
                value={model}
                onChange={(e) => field("ai_model", "model", "settings.toast.aiModel")(e.target.value)}
                placeholder={t("settings.model")}
              />
            </div>
          </>
        )}

        {/* ── Baidu ── */}
        {engine === "baidu" && (
          <>
            <div className="settings-row vertical">
              <div className="settings-row-label">{t("settings.baiduAppid")}</div>
              <input
                className="settings-input"
                value={baiduAppid}
                onChange={(e) => field("baidu_appid", "baiduAppid", "settings.toast.baiduAppid")(e.target.value)}
                placeholder={t("settings.baiduAppidPlaceholder")}
              />
            </div>
            <div className="settings-row vertical">
              <div className="settings-row-label">{t("settings.baiduSecretKey")}</div>
              <input
                className="settings-input"
                type="password"
                value={baiduSecretKey}
                onChange={(e) => field("baidu_secret_key", "baiduSecretKey", "settings.toast.baiduSecretKey")(e.target.value)}
                placeholder={t("settings.baiduSecretKeyPlaceholder")}
              />
            </div>
          </>
        )}

        {/* ── Youdao ── */}
        {engine === "youdao" && (
          <>
            <div className="settings-row vertical">
              <div className="settings-row-label">{t("settings.youdaoAppKey")}</div>
              <input
                className="settings-input"
                value={youdaoAppKey}
                onChange={(e) => field("youdao_app_key", "youdaoAppKey", "settings.toast.youdaoAppKey")(e.target.value)}
                placeholder={t("settings.youdaoAppKeyPlaceholder")}
              />
            </div>
            <div className="settings-row vertical">
              <div className="settings-row-label">{t("settings.youdaoAppSecret")}</div>
              <input
                className="settings-input"
                type="password"
                value={youdaoAppSecret}
                onChange={(e) => field("youdao_app_secret", "youdaoAppSecret", "settings.toast.youdaoAppSecret")(e.target.value)}
                placeholder={t("settings.youdaoAppSecretPlaceholder")}
              />
            </div>
          </>
        )}

        {/* ── Tencent ── */}
        {engine === "tencent" && (
          <>
            <div className="settings-row vertical">
              <div className="settings-row-label">{t("settings.tencentSecretId")}</div>
              <input
                className="settings-input"
                value={tencentSecretId}
                onChange={(e) => field("tencent_secret_id", "tencentSecretId", "settings.toast.tencentSecretId")(e.target.value)}
                placeholder={t("settings.tencentSecretIdPlaceholder")}
              />
            </div>
            <div className="settings-row vertical">
              <div className="settings-row-label">{t("settings.tencentSecretKey")}</div>
              <input
                className="settings-input"
                type="password"
                value={tencentSecretKey}
                onChange={(e) => field("tencent_secret_key", "tencentSecretKey", "settings.toast.tencentSecretKey")(e.target.value)}
                placeholder={t("settings.tencentSecretKeyPlaceholder")}
              />
            </div>
          </>
        )}

        {/* ── Volctrans ── */}
        {engine === "volctrans" && (
          <>
            <div className="settings-row vertical">
              <div className="settings-row-label">{t("settings.volctransAccessKeyId")}</div>
              <input
                className="settings-input"
                value={volctransAccessKeyId}
                onChange={(e) => field("volctrans_access_key_id", "volctransAccessKeyId", "settings.toast.volctransAccessKeyId")(e.target.value)}
                placeholder={t("settings.volctransAccessKeyIdPlaceholder")}
              />
            </div>
            <div className="settings-row vertical">
              <div className="settings-row-label">{t("settings.volctransSecretAccessKey")}</div>
              <input
                className="settings-input"
                type="password"
                value={volctransSecretAccessKey}
                onChange={(e) => field("volctrans_secret_access_key", "volctransSecretAccessKey", "settings.toast.volctransSecretAccessKey")(e.target.value)}
                placeholder={t("settings.volctransSecretAccessKeyPlaceholder")}
              />
            </div>
          </>
        )}
      </div>
    </div>
  );
}
