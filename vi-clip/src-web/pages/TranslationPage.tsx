import { useTranslation } from "react-i18next";
import { useTranslationStore } from "../stores/translationStore";
import { Icons } from "../components/Icons";
import IosSelect from "../components/IosSelect";
import { useState } from "react";

const LANGUAGES = [
  { code: "zh", name: "中文", badge: "ZH" },
  { code: "en", name: "English", badge: "EN" },
  { code: "ja", name: "日本語", badge: "JA" },
  { code: "ko", name: "한국어", badge: "KO" },
  { code: "fr", name: "Français", badge: "FR" },
  { code: "de", name: "Deutsch", badge: "DE" },
  { code: "es", name: "Español", badge: "ES" },
  { code: "ru", name: "Русский", badge: "RU" },
  { code: "ar", name: "العربية", badge: "AR" },
  { code: "th", name: "ไทย", badge: "TH" },
  { code: "vi", name: "Tiếng Việt", badge: "VI" },
];

export default function TranslationPage() {
  const { t } = useTranslation();
  const {
    inputText,
    targetLang,
    result,
    engine,
    loading,
    error,
    setInputText,
    setTargetLang,
    translate,
  } = useTranslationStore();

  const [copied, setCopied] = useState(false);

  const handleCopy = async () => {
    if (!result) return;
    try {
      await navigator.clipboard.writeText(result);
      setCopied(true);
      setTimeout(() => setCopied(false), 2000);
    } catch (err) {
      console.error("Failed to copy:", err);
    }
  };

  return (
    <div className="translation-page">
      <div className="translation-input-card">
        <textarea
          className="translation-input"
          placeholder={t("translate.inputPlaceholder")}
          value={inputText}
          onChange={(e) => setInputText(e.target.value)}
        />
        <div className="translation-input-footer">
          <span className="char-count">{inputText.length}</span>
          <button
            className="translate-btn"
            onClick={translate}
            disabled={loading || !inputText.trim()}
          >
            {loading ? (
              <div className="translate-spinner" />
            ) : (
              <>
                {Icons.translate}
                <span>{t("translate.translate")}</span>
              </>
            )}
          </button>
        </div>
      </div>

      <div className="translation-lang-section">
        <span className="translation-lang-label">{t("translate.targetLang")}</span>
        <div className="translation-lang-select-wrapper">
          <IosSelect
            value={targetLang}
            options={LANGUAGES.map((l) => ({ value: l.code, label: l.name }))}
            onChange={setTargetLang}
          />
        </div>
      </div>

      {error && (
        <div className="translation-error">
          <div className="error-icon-svg">{Icons.delete}</div>
          <span className="translation-error-text">{error}</span>
        </div>
      )}

      <div className="translation-result-card">
        <div className="translation-result-header">
          <span className="section-label">{t("translate.result")}</span>
          <div className="translation-result-header-right">
            {engine && (
              <span className="engine-badge">
                {engine === "ai" ? "AI" : "Google"}
              </span>
            )}
            {result && (
              <button
                className={`translation-copy-btn ${copied ? "copied" : ""}`}
                onClick={handleCopy}
                title={copied ? t("translate.copied") : t("translate.copy")}
              >
                {copied ? Icons.check : Icons.copy}
              </button>
            )}
          </div>
        </div>
        <div className="translation-result">
          {result ? (
            <p className="result-text">{result}</p>
          ) : (
            <div className="result-placeholder">
              <div className="result-placeholder-icon">{Icons.translate}</div>
              <span>{t("translate.noResult")}</span>
            </div>
          )}
        </div>
      </div>
    </div>
  );
}
