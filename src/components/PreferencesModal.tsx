import { useState } from "react";
import ClearCacheModal from "./ClearCacheModal";

interface PreferencesModalProps {
  theme: "dark" | "light";
  onThemeChange: (theme: "dark" | "light") => void;
  onClose: () => void;
}

export default function PreferencesModal({ theme, onThemeChange, onClose }: PreferencesModalProps) {
  const isDark = theme === "dark";
  const [showClearCache, setShowClearCache] = useState(false);

  return (
    <>
    <div
      onClick={onClose}
      style={{
        position: "fixed",
        inset: 0,
        background: "rgba(0,0,0,0.6)",
        display: "flex",
        alignItems: "center",
        justifyContent: "center",
        zIndex: 200,
      }}
    >
      <div
        onClick={e => e.stopPropagation()}
        style={{
          background: "var(--color-card-bg)",
          border: "1px solid var(--color-border)",
          borderRadius: "12px",
          padding: "32px 40px",
          display: "flex",
          flexDirection: "column",
          gap: "24px",
          boxShadow: "0 16px 48px rgba(0,0,0,0.6)",
          minWidth: "300px",
        }}
      >
        <span style={{ color: "var(--color-text)", fontSize: "15px", fontWeight: 600 }}>
          Preferences
        </span>

        <div style={{ display: "flex", alignItems: "center", justifyContent: "space-between", gap: "24px" }}>
          <span style={{ color: "var(--color-text)", fontSize: "13px" }}>Theme</span>
          <div style={{ display: "flex", alignItems: "center", gap: "10px" }}>
            <span style={{ color: "var(--color-text-muted)", fontSize: "12px" }}>Light</span>
            <button
              onClick={() => onThemeChange(isDark ? "light" : "dark")}
              style={{
                width: "44px",
                height: "24px",
                borderRadius: "12px",
                border: "none",
                background: isDark ? "#388bfd" : "var(--color-border)",
                cursor: "pointer",
                position: "relative",
                transition: "background 0.2s",
                padding: 0,
                flexShrink: 0,
              }}
            >
              <span
                style={{
                  position: "absolute",
                  top: "3px",
                  left: isDark ? "23px" : "3px",
                  width: "18px",
                  height: "18px",
                  borderRadius: "50%",
                  background: "#ffffff",
                  transition: "left 0.2s",
                }}
              />
            </button>
            <span style={{ color: "var(--color-text-muted)", fontSize: "12px" }}>Dark</span>
          </div>
        </div>

        <div style={{ display: "flex", alignItems: "center", justifyContent: "space-between", gap: "24px" }}>
          <span style={{ color: "var(--color-text)", fontSize: "13px" }}>Cache</span>
          <button
            onClick={() => setShowClearCache(true)}
            style={{
              background: "var(--color-card-hover)",
              border: "1px solid var(--color-border)",
              color: "var(--color-text)",
              fontSize: "12px",
              padding: "5px 14px",
              borderRadius: "6px",
              cursor: "pointer",
            }}
            onMouseEnter={e => (e.currentTarget.style.background = "var(--color-sidebar-bg)")}
            onMouseLeave={e => (e.currentTarget.style.background = "var(--color-card-hover)")}
          >
            Clear cache
          </button>
        </div>

        <button
          onClick={onClose}
          style={{
            alignSelf: "flex-end",
            background: "var(--color-card-hover)",
            border: "1px solid var(--color-border)",
            color: "var(--color-text)",
            fontSize: "13px",
            padding: "6px 20px",
            borderRadius: "6px",
            cursor: "pointer",
          }}
          onMouseEnter={e => (e.currentTarget.style.background = "var(--color-sidebar-bg)")}
          onMouseLeave={e => (e.currentTarget.style.background = "var(--color-card-hover)")}
        >
          Close
        </button>
      </div>
    </div>

    {showClearCache && <ClearCacheModal onClose={() => setShowClearCache(false)} />}
    </>
  );
}
