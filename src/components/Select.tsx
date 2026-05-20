import { useState, useRef, useEffect } from "react";

interface SelectProps {
  value: string;
  onChange: (value: string) => void;
  options: { value: string; label: string }[];
  placeholder?: string;
}

export default function Select({ value, onChange, options, placeholder = "Select..." }: SelectProps) {
  const [open, setOpen] = useState(false);
  const ref = useRef<HTMLDivElement>(null);

  useEffect(() => {
    function handleClickOutside(e: MouseEvent) {
      if (ref.current && !ref.current.contains(e.target as Node)) {
        setOpen(false);
      }
    }
    document.addEventListener("mousedown", handleClickOutside);
    return () => document.removeEventListener("mousedown", handleClickOutside);
  }, []);

  const selected = options.find((o) => o.value === value);

  return (
    <div ref={ref} style={{ position: "relative", userSelect: "none" }}>
      <div
        onClick={() => setOpen((o) => !o)}
        style={{
          display: "flex",
          alignItems: "center",
          justifyContent: "space-between",
          background: "var(--color-sidebar-bg)",
          border: "1px solid var(--color-border)",
          borderRadius: open ? "6px 6px 0 0" : "6px",
          padding: "8px 10px",
          color: selected ? "var(--color-text)" : "var(--color-text-muted)",
          fontSize: "13px",
          cursor: "pointer",
        }}
      >
        <span>{selected ? selected.label : placeholder}</span>
        <span style={{
          display: "inline-block",
          transform: open ? "rotate(180deg)" : "rotate(0deg)",
          transition: "transform 0.15s",
          fontSize: "10px",
          color: "var(--color-text-muted)",
        }}>▼</span>
      </div>

      {open && (
        <div style={{
          position: "absolute",
          top: "100%",
          left: 0,
          right: 0,
          background: "var(--color-sidebar-bg)",
          border: "1px solid var(--color-border)",
          borderTop: "none",
          borderRadius: "0 0 6px 6px",
          zIndex: 50,
          overflow: "hidden",
        }}>
          {options.map((option) => (
            <div
              key={option.value}
              onClick={() => { onChange(option.value); setOpen(false); }}
              style={{
                padding: "8px 10px",
                fontSize: "13px",
                color: option.value === value ? "var(--color-text)" : "var(--color-text-muted)",
                background: option.value === value ? "var(--color-card-hover)" : "transparent",
                cursor: "pointer",
              }}
              onMouseEnter={(e) => {
                if (option.value !== value)
                  e.currentTarget.style.background = "var(--color-card-bg)";
              }}
              onMouseLeave={(e) => {
                if (option.value !== value)
                  e.currentTarget.style.background = "transparent";
              }}
            >
              {option.label}
            </div>
          ))}
        </div>
      )}
    </div>
  );
}
