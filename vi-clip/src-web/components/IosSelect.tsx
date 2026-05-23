import { useState, useRef, useEffect, useCallback } from "react";

interface Option {
  value: string;
  label: string;
}

interface IosSelectProps {
  value: string;
  options: Option[];
  onChange: (value: string) => void;
  placeholder?: string;
}

export default function IosSelect({ value, options, onChange, placeholder }: IosSelectProps) {
  const [open, setOpen] = useState(false);
  const ref = useRef<HTMLDivElement>(null);
  const dropdownRef = useRef<HTMLDivElement>(null);

  const selected = options.find((o) => o.value === value);

  const close = useCallback(() => setOpen(false), []);

  useEffect(() => {
    if (!open) return;
    const handler = (e: MouseEvent) => {
      if (ref.current && !ref.current.contains(e.target as Node)) {
        close();
      }
    };
    document.addEventListener("mousedown", handler);
    return () => document.removeEventListener("mousedown", handler);
  }, [open, close]);

  useEffect(() => {
    if (!open) return;
    const dropdown = dropdownRef.current;
    if (!dropdown) return;
    const handler = (e: WheelEvent) => {
      const el = e.target as HTMLElement;
      if (dropdown.contains(el)) {
        const { scrollTop, scrollHeight, clientHeight } = dropdown;
        const atTop = scrollTop <= 0 && e.deltaY < 0;
        const atBottom = scrollTop + clientHeight >= scrollHeight && e.deltaY > 0;
        if (!atTop && !atBottom) return;
      }
      e.preventDefault();
      e.stopPropagation();
    };
    window.addEventListener("wheel", handler, { passive: false });
    return () => window.removeEventListener("wheel", handler);
  }, [open]);

  return (
    <div className="ios-select" ref={ref}>
      <button
        className={`ios-select-trigger${open ? " open" : ""}`}
        onClick={() => setOpen(!open)}
        type="button"
      >
        <span className="ios-select-value">{selected?.label || placeholder}</span>
        <span className={`ios-select-arrow${open ? " open" : ""}`}>
          <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2.5" strokeLinecap="round" strokeLinejoin="round">
            <polyline points="6 9 12 15 18 9" />
          </svg>
        </span>
      </button>
      {open && (
        <div className="ios-select-dropdown">
          <div className="ios-select-dropdown-inner" ref={dropdownRef}>
            {options.map((opt) => (
              <button
                key={opt.value}
                className={`ios-select-option${opt.value === value ? " selected" : ""}`}
                onClick={() => {
                  onChange(opt.value);
                  close();
                }}
                type="button"
              >
                <span className="ios-select-option-label">{opt.label}</span>
                {opt.value === value && (
                  <span className="ios-select-check">
                    <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="3" strokeLinecap="round" strokeLinejoin="round">
                      <polyline points="20 6 9 17 4 12" />
                    </svg>
                  </span>
                )}
              </button>
            ))}
          </div>
        </div>
      )}
    </div>
  );
}
