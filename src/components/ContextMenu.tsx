import { useEffect, useRef } from "react";
import { createPortal } from "react-dom";

interface ContextMenuProps {
  x: number;
  y: number;
  items: { label: string; onClick: () => void }[];
  onClose: () => void;
}

export default function ContextMenu({ x, y, items, onClose }: ContextMenuProps) {
  const menuRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (menuRef.current) {
      const rect = menuRef.current.getBoundingClientRect();
      if (rect.right > window.innerWidth) {
        menuRef.current.style.left = `${window.innerWidth - rect.width - 8}px`;
      }
      if (rect.bottom > window.innerHeight) {
        menuRef.current.style.top = `${window.innerHeight - rect.height - 8}px`;
      }
    }
  }, []);

  useEffect(() => {
    function handleMouseDown(e: MouseEvent) {
      if (menuRef.current && !menuRef.current.contains(e.target as Node)) {
        onClose();
      }
    }
    function handleKey(e: KeyboardEvent) {
      if (e.key === "Escape") onClose();
    }
    document.addEventListener("mousedown", handleMouseDown);
    document.addEventListener("keydown", handleKey);
    return () => {
      document.removeEventListener("mousedown", handleMouseDown);
      document.removeEventListener("keydown", handleKey);
    };
  }, [onClose]);

  return createPortal(
    <div
      ref={menuRef}
      style={{
        position: "fixed",
        top: y,
        left: x,
        background: "var(--color-card-bg)",
        border: "1px solid var(--color-border)",
        borderRadius: "6px",
        boxShadow: "0 4px 16px rgba(0,0,0,0.4)",
        zIndex: 9999,
        minWidth: "180px",
        overflow: "hidden",
      }}
    >
      {items.map((item, i) => (
        <button
          key={i}
          onClick={() => { item.onClick(); onClose(); }}
          style={{
            display: "block",
            width: "100%",
            padding: "10px 14px",
            background: "transparent",
            border: "none",
            borderBottom: i < items.length - 1 ? "1px solid var(--color-border)" : "none",
            color: "var(--color-text)",
            fontSize: "13px",
            textAlign: "left",
            cursor: "pointer",
          }}
          onMouseEnter={e => (e.currentTarget.style.background = "var(--color-card-hover)")}
          onMouseLeave={e => (e.currentTarget.style.background = "transparent")}
        >
          {item.label}
        </button>
      ))}
    </div>,
    document.body
  );
}
