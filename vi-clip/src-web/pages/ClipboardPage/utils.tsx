import React from "react";

export function formatTime(dateStr: string): string {
  const date = new Date(dateStr);
  const month = date.getMonth() + 1;
  const day = date.getDate();
  const hours = date.getHours().toString().padStart(2, "0");
  const minutes = date.getMinutes().toString().padStart(2, "0");
  return `${month}/${day} ${hours}:${minutes}`;
}

export function getFileName(path: string): string {
  const parts = path.replace(/\\/g, "/").split("/");
  return parts[parts.length - 1] || path;
}

export const TYPE_META: Record<string, { icon: React.ReactElement | null; color: string }> = {
  text: { icon: null, color: "#007AFF" },
  image: { icon: null, color: "#34C759" },
  link: { icon: null, color: "#FF9500" },
  file: { icon: null, color: "#AF52DE" },
};
