import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";

interface ClearCacheModalProps {
  onClose: () => void;
}

export default function ClearCacheModal({ onClose }: ClearCacheModalProps) {
  const [done, setDone] = useState(false);
  const [clearing, setClearing] = useState(false);

  async function handleConfirm() {
    setClearing(true);
    try {
      await invoke("clear_cache");
      setDone(true);
    } catch {
      setDone(true);
    } finally {
      setClearing(false);
    }
  }

  return (
    <div
      onClick={done ? onClose : undefined}
      style={{
        position: "fixed",
        inset: 0,
        background: "rgba(0,0,0,0.5)",
        display: "flex",
        alignItems: "center",
        justifyContent: "center",
        zIndex: 300,
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
          gap: "20px",
          boxShadow: "0 16px 48px rgba(0,0,0,0.6)",
          minWidth: "300px",
          maxWidth: "380px",
        }}
      >
        {done ? (
          <>
            <span style={{ color: "var(--color-text)", fontSize: "15px", fontWeight: 600 }}>
              Cache cleared
            </span>
            <span style={{ color: "var(--color-text-muted)", fontSize: "13px", lineHeight: 1.5 }}>
              Please restart the application to reload your content.
            </span>
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
          </>
        ) : (
          <>
            <span style={{ color: "var(--color-text)", fontSize: "15px", fontWeight: 600 }}>
              Clear cache?
            </span>
            <span style={{ color: "var(--color-text-muted)", fontSize: "13px", lineHeight: 1.5 }}>
              This will remove all locally cached data. You will need to restart the application afterwards.
            </span>
            <div style={{ display: "flex", gap: "8px", justifyContent: "flex-end" }}>
              <button
                onClick={onClose}
                disabled={clearing}
                style={{
                  background: "var(--color-card-hover)",
                  border: "1px solid var(--color-border)",
                  color: "var(--color-text)",
                  fontSize: "13px",
                  padding: "6px 20px",
                  borderRadius: "6px",
                  cursor: "pointer",
                  opacity: clearing ? 0.5 : 1,
                }}
                onMouseEnter={e => (e.currentTarget.style.background = "var(--color-sidebar-bg)")}
                onMouseLeave={e => (e.currentTarget.style.background = "var(--color-card-hover)")}
              >
                Cancel
              </button>
              <button
                onClick={handleConfirm}
                disabled={clearing}
                style={{
                  background: "#ef4444",
                  border: "1px solid #dc2626",
                  color: "#ffffff",
                  fontSize: "13px",
                  padding: "6px 20px",
                  borderRadius: "6px",
                  cursor: clearing ? "not-allowed" : "pointer",
                  opacity: clearing ? 0.7 : 1,
                }}
                onMouseEnter={e => { if (!clearing) e.currentTarget.style.background = "#dc2626"; }}
                onMouseLeave={e => { if (!clearing) e.currentTarget.style.background = "#ef4444"; }}
              >
                {clearing ? "Clearing…" : "Yes, clear cache"}
              </button>
            </div>
          </>
        )}
      </div>
    </div>
  );
}
