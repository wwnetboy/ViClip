import { useTranslation } from "react-i18next";
import { displayShortcut, isMacOS } from "../../utils";

interface ShortcutSectionProps {
  shortcutKey: string;
  onShortcutChange: (key: string) => void;
  recording: boolean;
  onStartRecording: () => void;
  onStopRecording: () => void;
  radialMenuEnabled: boolean;
  onRadialMenuChange: (enabled: boolean) => void;
  clickMode: string;
  onClickModeChange: (mode: string) => void;
}

export function ShortcutSection({
  shortcutKey,
  onShortcutChange,
  recording,
  onStartRecording,
  onStopRecording,
  radialMenuEnabled,
  onRadialMenuChange,
  clickMode,
  onClickModeChange,
}: ShortcutSectionProps) {
  const { t } = useTranslation();
  const macOS = isMacOS();

  // Win+V interception is a Windows-only feature. On macOS, Super+V is a
  // normal customizable shortcut and should not disable the record button.
  const winVEnabled = !macOS && shortcutKey === "Super+V";
  const displayKey = displayShortcut(shortcutKey);

  return (
    <div className="settings-section">
      <div className="settings-section-title">{t("settings.shortcut")}</div>
      <div className="settings-card">
        <div className="settings-row">
          <div className="settings-row-label">{t("settings.windowShortcut")}</div>
          <div className="shortcut-setting">
            <div className="shortcut-keyboard-row">
              <span className={`shortcut-display${recording ? " recording" : ""}${winVEnabled ? " disabled" : ""}`}>
                {recording ? t("settings.recording") : (displayKey || t("settings.shortcutPlaceholder"))}
              </span>
              <button
                className="shortcut-record-btn"
                onClick={recording ? onStopRecording : onStartRecording}
                disabled={winVEnabled}
              >
                {recording ? t("settings.stopRecord") : t("settings.recordShortcut")}
              </button>
            </div>
          </div>
        </div>
        {!macOS && (
        <div className="settings-row">
          <div className="settings-row-label">{t("settings.winVShortcut")}</div>
          <div className="radial-shortcut-right">
            <button
              className={`toggle-switch ${winVEnabled ? "on" : "off"}`}
              onClick={() => onShortcutChange(winVEnabled ? "" : "Super+V")}
              title={winVEnabled ? t("common.on") : t("common.off")}
            >
              <span className="toggle-thumb" />
            </button>
          </div>
        </div>
        )}
        <div className="settings-row">
          <div className="settings-row-label">{t("settings.radialShortcut")}</div>
          <div className="radial-shortcut-right">
            <span className="radial-shortcut-key">
              {macOS ? "Command+Shift+V" : t("settings.radialShortcutDesc")}
            </span>
            <button
              className={`toggle-switch ${radialMenuEnabled ? "on" : "off"}`}
              onClick={() => onRadialMenuChange(!radialMenuEnabled)}
              title={radialMenuEnabled ? t("common.on") : t("common.off")}
            >
              <span className="toggle-thumb" />
            </button>
          </div>
        </div>
        <div className="settings-row">
          <div className="settings-row-label">{t("settings.clickMode")}</div>
          <div className="settings-row-right">
            <div className="ios-segment">
              <button
                className={`ios-segment-btn ${clickMode === "single" ? "active" : ""}`}
                onClick={() => onClickModeChange("single")}
              >
                {t("settings.clickModeSingle")}
              </button>
              <button
                className={`ios-segment-btn ${clickMode === "double" ? "active" : ""}`}
                onClick={() => onClickModeChange("double")}
              >
                {t("settings.clickModeDouble")}
              </button>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}
