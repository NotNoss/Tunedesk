import { useState, useEffect } from "react";
import { listen } from "@tauri-apps/api/event";

interface FetchNotif {
  id: string;
  message: string;
}

export default function FetchToast() {
  const [notifs, setNotifs] = useState<FetchNotif[]>([]);

  useEffect(() => {
    const unlistenStart = listen<FetchNotif>("fetch:start", (e) => {
      setNotifs(prev =>
        prev.some(n => n.id === e.payload.id) ? prev : [...prev, e.payload]
      );
    });

    const unlistenEnd = listen<{ id: string }>("fetch:end", (e) => {
      setNotifs(prev => prev.filter(n => n.id !== e.payload.id));
    });

    return () => {
      unlistenStart.then(fn => fn());
      unlistenEnd.then(fn => fn());
    };
  }, []);

  if (notifs.length === 0) return null;

  return (
    <div style={{
      position: "fixed",
      bottom: 16,
      right: 16,
      zIndex: 500,
      display: "flex",
      flexDirection: "column",
      gap: 6,
      maxWidth: 280,
      pointerEvents: "none",
    }}>
      {notifs.map(n => (
        <div
          key={n.id}
          style={{
            background: "var(--color-card-bg)",
            border: "1px solid var(--color-border)",
            borderRadius: 8,
            padding: "7px 10px 7px 12px",
            display: "flex",
            alignItems: "center",
            gap: 8,
            boxShadow: "0 4px 16px rgba(0,0,0,0.4)",
            pointerEvents: "auto",
          }}
        >
          <div className="fetch-toast-spinner" />
          <span style={{
            flex: 1,
            fontSize: 12,
            color: "var(--color-text-muted)",
            lineHeight: 1.4,
            overflow: "hidden",
            textOverflow: "ellipsis",
            whiteSpace: "nowrap",
          }}>
            {n.message}
          </span>
          <button
            onClick={() => setNotifs(prev => prev.filter(x => x.id !== n.id))}
            style={{
              background: "transparent",
              border: "none",
              color: "var(--color-text-muted)",
              fontSize: 16,
              lineHeight: 1,
              cursor: "pointer",
              padding: "0 2px",
              flexShrink: 0,
            }}
            onMouseEnter={e => (e.currentTarget.style.color = "var(--color-text)")}
            onMouseLeave={e => (e.currentTarget.style.color = "var(--color-text-muted)")}
          >
            ×
          </button>
        </div>
      ))}
    </div>
  );
}
