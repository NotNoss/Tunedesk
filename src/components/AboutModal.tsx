import { useEffect, useState } from "react";
import { getVersion } from "@tauri-apps/api/app";

interface AboutModalProps {
  onClose: () => void;
}

export default function AboutModal({ onClose }: AboutModalProps) {
  const [version, setVersion] = useState<string>("");

  useEffect(() => {
    getVersion().then(setVersion);
  }, []);

  return (
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
          padding: "36px 48px",
          display: "flex",
          flexDirection: "column",
          alignItems: "center",
          gap: "16px",
          boxShadow: "0 16px 48px rgba(0,0,0,0.6)",
          minWidth: "260px",
        }}
      >
        <img src="/Tunedesk.svg" width={64} height={64} alt="" />
        <div style={{ textAlign: "center", display: "flex", flexDirection: "column", gap: "6px" }}>
          <span style={{ color: "var(--color-text)", fontSize: "14px" }}>Author: Noss</span>
          <span style={{ color: "var(--color-text-muted)", fontSize: "13px" }}>Version: {version}</span>
        </div>
        <button
          onClick={onClose}
          style={{
            marginTop: "8px",
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
  );
}
