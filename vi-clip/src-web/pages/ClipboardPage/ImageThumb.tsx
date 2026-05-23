import { useState, useEffect, useRef } from "react";
import { useClipboardStore } from "../../stores/clipboardStore";

interface ImageThumbProps {
  record: { id: string; content: string };
  onClick: (e: React.MouseEvent) => void;
}

export function ImageThumb({ record, onClick }: ImageThumbProps) {
  const { getThumbnail, thumbnailCache } = useClipboardStore();
  const [src, setSrc] = useState<string | null>(null);
  const [visible, setVisible] = useState(false);
  const ref = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (!visible) return;
    const cached = thumbnailCache[record.id];
    if (cached) {
      setSrc(cached);
      return;
    }
    getThumbnail(record as any).then((dataUrl) => {
      if (dataUrl) setSrc(dataUrl);
    });
  }, [visible, record.id]);

  useEffect(() => {
    const el = ref.current;
    if (!el) return;
    const observer = new IntersectionObserver(
      ([entry]) => {
        if (entry.isIntersecting) setVisible(true);
      },
      { rootMargin: "200px" }
    );
    observer.observe(el);
    return () => observer.disconnect();
  }, []);

  return (
    <div
      ref={ref}
      className="clipboard-card-thumb"
      onClick={onClick}
    >
      {src ? (
        <img src={src} alt="" />
      ) : (
        <div className="thumb-spinner" />
      )}
    </div>
  );
}
