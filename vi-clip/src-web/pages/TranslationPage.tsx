import { useTranslation } from "react-i18next";
import { useTranslationStore } from "../stores/translationStore";
import { useSettingsStore } from "../stores/settingsStore";
import { Icons } from "../components/Icons";
import IosSelect from "../components/IosSelect";
import { useState, useEffect } from "react";
import { getLanguagesForEngine, normalizeLangCode } from "../data/languages";
import i18next from "i18next";

function localizeError(error: string, t: (key: string) => string): string {
  const lower = error.toLowerCase();
  if (lower.includes("connect") || lower.includes("network") || lower.includes("proxy")) {
    return t("translate.errors.network");
  }
  if (lower.includes("timeout") || lower.includes("timed out")) {
    return t("translate.errors.timeout");
  }
  if (lower.includes("http")) {
    return t("translate.errors.http");
  }
  if (lower.includes("unexpected") || lower.includes("parse") || lower.includes("format error")) {
    return t("translate.errors.parse");
  }
  if (lower.includes("not configured") || lower.includes("fill in the")) {
    return t("translate.errors.config");
  }
  // Show API-specific errors directly so the user can see the actual reason
  if (lower.includes("translation error") || lower.includes("translate error")) {
    return error;
  }
  return t("translate.errors.generic");
}

function langName(code: string): string {
  const c = normalizeLangCode(code);
  const found = getLanguagesForEngine("google").find((l) => l.code === c);
  const name = found ? found.name : code;
  return i18next.t(`langs.${c}`, name);
}

export default function TranslationPage() {
  const { t } = useTranslation();
  const {
    inputText,
    targetLang,
    result,
    engine,
    loading,
    error,
    detectedLang,
    setInputText,
    setTargetLang,
    clearInput,
  } = useTranslationStore();

  const defaultEngine = useSettingsStore((s) => s.defaultEngine);
  const defaultTargetLang = useSettingsStore((s) => s.defaultTargetLang);
  const engineLangs = getLanguagesForEngine(defaultEngine);

  useEffect(() => {
    setTargetLang(defaultTargetLang);
  }, [defaultTargetLang]);

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
      {/* ── 上半区：源语言输入 ── */}
      <div className="trans-panel trans-source">
        <div className="trans-panel-header">
          <span className="trans-panel-label">
            {detectedLang && inputText
              ? `${t("translate.sourceLang")} - ${langName(detectedLang)}`
              : t("translate.sourceLang")}
          </span>
          {inputText && (
            <button className="trans-clear-btn" onClick={clearInput} title={t("translate.clear")}>
              {Icons.delete}
            </button>
          )}
        </div>
        <textarea
          className="trans-textarea"
          placeholder={t("translate.inputPlaceholder")}
          value={inputText}
          onChange={(e) => setInputText(e.target.value)}
        />
        <div className="trans-panel-footer">
          <span className="trans-char-count">{inputText.length}/5000</span>
        </div>
      </div>

      {/* ── 下半区：目标语言输出 ── */}
      <div className="trans-panel trans-target">
        <div className="trans-panel-header">
          <div className="trans-lang-select">
            <IosSelect
              value={targetLang}
              options={engineLangs.map((l) => ({ value: l.code, label: t(`langs.${l.code}`, l.name) }))}
              onChange={setTargetLang}
            />
          </div>
          <div className="trans-panel-actions">
            {loading && <div className="translate-spinner" />}
            {engine && !loading && (
              <span className="engine-badge">
                {engine === "ai" ? t("translate.engineAi") : engine === "baidu" ? t("translate.engineBaidu") : engine === "youdao" ? t("translate.engineYoudao") : engine === "tencent" ? t("translate.engineTencent") : engine === "volctrans" ? t("translate.engineVolctrans") : t("translate.engineGoogle")}
              </span>
            )}
            {result && (
              <button
                className={`trans-copy-btn ${copied ? "copied" : ""}`}
                onClick={handleCopy}
                title={copied ? t("translate.copied") : t("translate.copy")}
              >
                {copied ? Icons.check : Icons.copy}
              </button>
            )}
          </div>
        </div>
        <div className="trans-result-body">
          {error ? (
            <div className="trans-error-banner">
              <div className="trans-error-top">
                <div className="error-icon-svg">{Icons.delete}</div>
                <span className="trans-error-text">{localizeError(error, t)}</span>
              </div>
            </div>
          ) : result ? (
            <p className="trans-result-text">{result}</p>
          ) : (
            <div className="trans-result-placeholder">
              <span>{t("translate.noResult")}</span>
            </div>
          )}
        </div>
      </div>
    </div>
  );
}
