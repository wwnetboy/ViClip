import { memo, useCallback, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { WebviewWindow } from "@tauri-apps/api/webviewWindow";
import { useTranslation } from "react-i18next";
import type { ClipboardRecord } from "../../types";
import { Icons } from "../../components/Icons";
import { useSettingsStore } from "../../stores/settingsStore";
import { usePhraseStore } from "../../stores/phraseStore";
import { useToastStore } from "../../stores/toastStore";
import { ContextMenu, type ContextMenuItem } from "../../components/ContextMenu";
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
const MAX_PREVIEW_WINDOWS = 6;
const openPreviews: string[] = [];

async function canOpenPreview(limitMessage: string): Promise<boolean> {
  // Remove stale entries (windows already closed by user)
  for (let i = openPreviews.length - 1; i >= 0; i--) {
    const w = await WebviewWindow.getByLabel(openPreviews[i]);
    if (!w) openPreviews.splice(i, 1);
  }
  if (openPreviews.length >= MAX_PREVIEW_WINDOWS) {
    useToastStore.getState().show(limitMessage);
    return false;
  }
  return true;
}

function ClipboardCardInner({
  record,
  index,
  getTypeLabel,
  onPaste,
  onDelete,
}: ClipboardCardProps) {
  const { t } = useTranslation();
  const meta = TYPE_META[record.type] || TYPE_META.text;
  const clickMode = useSettingsStore((s) => s.clickMode);
  const groups = usePhraseStore((s) => s.groups);
  const createPhrase = usePhraseStore((s) => s.createPhrase);
  const [menuPos, setMenuPos] = useState<{ x: number; y: number } | null>(null);

  const handlePaste = useCallback(() => onPaste(record), [onPaste, record]);
  const handleDelete = useCallback(
    (e: React.MouseEvent) => {
      e.stopPropagation();
      onDelete(record.id);
    },
    [onDelete, record.id],
  );

  const openPreview = useCallback(
    async (e: React.MouseEvent, alwaysOnTop: boolean) => {
      e.stopPropagation();
      try {
        if (!(await canOpenPreview(t("imagePreview.limitReached")))) return;

        const label = `image-preview-${++previewCounter}`;
        const encodedPath = encodeURIComponent(record.content);

        new WebviewWindow(label, {
          url: `index.html?preview=1&path=${encodedPath}`,
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
          alwaysOnTop,
        });
        openPreviews.push(label);
      } catch (err) {
        console.error("Failed to open image preview:", err);
      }
    },
    [record, t],
  );

  const handleContextMenu = useCallback(
    (e: React.MouseEvent) => {
      e.preventDefault();
      e.stopPropagation();
      setMenuPos({ x: e.clientX, y: e.clientY });
    },
    [],
  );

  const handleCopyContent = useCallback(async () => {
    try {
      await navigator.clipboard.writeText(record.content);
    } catch {
      // Fallback for older environments
      const ta = document.createElement("textarea");
      ta.value = record.content;
      document.body.appendChild(ta);
      ta.select();
      document.execCommand("copy");
      document.body.removeChild(ta);
    }
  }, [record.content]);

  const handleOpenLink = useCallback(async () => {
    try {
      await invoke("open_url", { url: record.content });
    } catch (err) {
      console.error("Failed to open link:", err);
    }
  }, [record.content]);

  const handleOpenFileLocation = useCallback(async () => {
    try {
      await invoke("open_file_location", { path: record.content });
    } catch (err) {
      console.error("Failed to open file location:", err);
    }
  }, [record.content]);

  const handleSaveAsPhrase = useCallback(
    async (groupId: string) => {
      try {
        const title = record.content.slice(0, 30);
        await createPhrase(groupId, title, record.content);
      } catch (err) {
        console.error("Failed to save as phrase:", err);
      }
    },
    [record.content, createPhrase],
  );

  const menuItems: ContextMenuItem[] = [
    {
      key: "paste",
      label: t("contextMenu.paste"),
      icon: Icons.paste,
      onClick: handlePaste,
    },
    ...(record.type !== "image"
      ? [
          {
            key: "copyContent",
            label: t("contextMenu.copyContent"),
            icon: Icons.copy,
            onClick: handleCopyContent,
          } as ContextMenuItem,
        ]
      : []),
    ...(record.type === "link"
      ? [
          {
            key: "openLink",
            label: t("contextMenu.openLink"),
            icon: Icons.link,
            onClick: handleOpenLink,
          } as ContextMenuItem,
        ]
      : []),
    ...(record.type === "file"
      ? [
          {
            key: "openFileLocation",
            label: t("contextMenu.openFileLocation"),
            icon: Icons.file,
            onClick: handleOpenFileLocation,
          } as ContextMenuItem,
        ]
      : []),
    {
      key: "saveAsPhrase",
      label: t("contextMenu.saveAsPhrase"),
      icon: Icons.phrases,
      children:
        groups.length > 0
          ? groups.map((g) => ({
              key: `save-${g.id}`,
              label: g.name,
              onClick: () => handleSaveAsPhrase(g.id),
            }))
          : [
              {
                key: "noGroups",
                label: t("contextMenu.noGroups"),
                disabled: true,
              },
            ],
    },
    {
      key: "delete",
      label: t("contextMenu.delete"),
      icon: Icons.delete,
      onClick: () => onDelete(record.id),
      danger: true,
    },
  ];

  const clickProps =
    clickMode === "double"
      ? { onDoubleClick: handlePaste }
      : { onClick: handlePaste };

  return (
    <>
      <div
        className={`notification clipboard-card type-${record.type}`}
        style={{ "--enter-delay": index } as React.CSSProperties}
        {...clickProps}
        onContextMenu={handleContextMenu}
      >
        <div className="noticontent">
          <div className="notititle clipboard-card-header">
            <span className="noti-type-label">
              <span className="noti-type-icon">{meta.icon}</span>
              <span className="noti-type-text">{getTypeLabel(record.type)}</span>
            </span>
            {record.type === "image" && (
              <div className="card-header-actions">
                <button className="card-preview-btn" onClick={(e) => openPreview(e, false)} title={t("imagePreview.title")}>
                  <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
                    <path d="M1 12s4-8 11-8 11 8 11 8-4 8-11 8-11-8-11-8z" />
                    <circle cx="12" cy="12" r="3" />
                  </svg>
                </button>
                <button
                  className="card-preview-btn pin"
                  onClick={(e) => openPreview(e, true)}
                  title={t("imagePreview.pin")}
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
              <button className="card-delete-btn" onClick={handleDelete} title={t("contextMenu.delete")}>
                {Icons.delete}
              </button>
            </div>
          </div>
        </div>
      </div>
      {menuPos && (
        <ContextMenu
          items={menuItems}
          position={menuPos}
          onClose={() => setMenuPos(null)}
        />
      )}
    </>
  );
}

export const ClipboardCard = memo(ClipboardCardInner);
