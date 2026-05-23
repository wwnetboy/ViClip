import { memo, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import { WebviewWindow } from "@tauri-apps/api/webviewWindow";
import type { ClipboardRecord } from "../../types";
import { Icons } from "../../components/Icons";
import { useSettingsStore } from "../../stores/settingsStore";
import { ImageThumb } from "./ImageThumb";
import { formatTime, getFileName, TYPE_META } from "./utils";

interface ClipboardCardProps {
  record: ClipboardRecord;
  index: number;
  getTypeLabel: (type: string) => string;
  onPaste: (r: ClipboardRecord) => void;
  onDelete: (id: string) => void;
}

let previewCounter = 0;

function ClipboardCardInner({
  record,
  index,
  getTypeLabel,
  onPaste,
  onDelete,
}: ClipboardCardProps) {
  const meta = TYPE_META[record.type] || TYPE_META.text;
  const clickMode = useSettingsStore((s) => s.clickMode);

  const handlePaste = useCallback(() => onPaste(record), [onPaste, record]);
  const handleDelete = useCallback(
    (e: React.MouseEvent) => {
      e.stopPropagation();
      onDelete(record.id);
    },
    [onDelete, record.id],
  );

  const handlePreview = useCallback(
    async (e: React.MouseEvent) => {
      e.stopPropagation();
      try {
        const base64: string = await invoke("get_image_base64", { path: record.content });
        const token: string = await invoke("store_preview_image", { base64 });
        const label = `image-preview-${++previewCounter}`;

        new WebviewWindow(label, {
          url: `index.html?preview=1&token=${token}`,
          title: "ViClip - Image Preview",
          width: 400,
          height: 300,
          decorations: false,
          transparent: true,
          visible: true,
          resizable: true,
          shadow: false,
          skipTaskbar: false,
          focus: true,
        });
      } catch (err) {
        console.error("Failed to open image preview:", err);
      }
    },
    [record],
  );

  const handlePinPreview = useCallback(
    async (e: React.MouseEvent) => {
      e.stopPropagation();
      try {
        const base64: string = await invoke("get_image_base64", { path: record.content });
        const token: string = await invoke("store_preview_image", { base64 });
        const label = `image-preview-${++previewCounter}`;

        new WebviewWindow(label, {
          url: `index.html?preview=1&token=${token}`,
          title: "ViClip - Image Preview",
          width: 400,
          height: 300,
          decorations: false,
          transparent: true,
          visible: true,
          resizable: true,
          shadow: false,
          skipTaskbar: false,
          focus: true,
          alwaysOnTop: true,
        });
      } catch (err) {
        console.error("Failed to open pinned image preview:", err);
      }
    },
    [record],
  );

  const clickProps = clickMode === "double"
    ? { onDoubleClick: handlePaste }
    : { onClick: handlePaste };

  return (
    <div
      className={`notification clipboard-card type-${record.type}`}
      style={{ "--color": meta.color, "--enter-delay": index } as React.CSSProperties}
      {...clickProps}
    >
      <div className="noticontent">
        <div className="notititle clipboard-card-header">
          <span className="noti-type-label">
            <span className="noti-type-icon">{meta.icon}</span>
            <span className="noti-type-text">{getTypeLabel(record.type)}</span>
          </span>
          {record.type === "image" && (
            <div className="card-header-actions">
              <button className="card-preview-btn" onClick={handlePreview} title="Preview image">
                <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
                  <path d="M1 12s4-8 11-8 11 8 11 8-4 8-11 8-11-8-11-8z" />
                  <circle cx="12" cy="12" r="3" />
                </svg>
              </button>
              <button
                className="card-preview-btn pin"
                onClick={handlePinPreview}
                title="Always on top"
              >
                <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
                  <path d="M16 12V4h1a1 1 0 0 0 0-2H7a1 1 0 0 0 0 2h1v8l-2.5 3.5a1 1 0 0 0 .8 1.5h4.7v4.5a1 1 0 0 0 2 0V17h4.7a1 1 0 0 0 .8-1.5L16 12z" />
                </svg>
              </button>
            </div>
          )}
        </div>
        <div className="notibody clipboard-card-body">
          {record.type === "image" ? (
            <ImageThumb
              record={record}
              onClick={(e) => {
                e.stopPropagation();
                onPaste(record);
              }}
            />
          ) : record.type === "link" ? (
            <span className="clipboard-link-content">{record.content}</span>
          ) : record.type === "file" ? (
            <span className="clipboard-file-content">{getFileName(record.content)}</span>
          ) : (
            <span className="clipboard-text-content">{record.content}</span>
          )}
        </div>
        <div className="notititle clipboard-card-footer">
          <span className="clipboard-card-time">{formatTime(record.created_at)}</span>
          <div className="clipboard-card-actions">
            <button className="card-delete-btn" onClick={handleDelete}>
              {Icons.delete}
            </button>
          </div>
        </div>
      </div>
    </div>
  );
}

export const ClipboardCard = memo(ClipboardCardInner);
