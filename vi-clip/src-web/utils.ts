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

/** Detect if running on macOS. Uses navigator.userAgentData or navigator.platform. */
export function isMacOS(): boolean {
  // Modern API
  if ('userAgentData' in navigator && (navigator as any).userAgentData?.platform) {
    return (navigator as any).userAgentData.platform === 'macOS';
  }
  // Fallback
  return /Mac|Macintosh/i.test(navigator.platform || '');
}

/** Convert a shortcut string for display on current platform. */
export function displayShortcut(shortcut: string): string {
  if (!shortcut) return '';
  if (isMacOS()) {
    return shortcut
      .replace(/Super\+/g, 'Command+')
      .replace(/Alt\+/g, 'Option+')
      .replace(/Ctrl\+/g, 'Control+')
      .replace(/Shift\+/g, 'Shift+');
  }
  return shortcut.replace(/Super\+/g, 'Win+');
}
