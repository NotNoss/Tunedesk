import { invoke } from "@tauri-apps/api/core";

interface PlaybackLoadingModalProps {
  onCancel: () => void;
}

export default function PlaybackLoadingModal({ onCancel }: PlaybackLoadingModalProps) {
  function handleCancel() {
    invoke("cancel_playback").catch(console.error);
    onCancel();
  }

  return (
    <div style={{
      position: "fixed",
      inset: 0,
      background: "rgba(0,0,0,0.75)",
      display: "flex",
      alignItems: "center",
      justifyContent: "center",
      zIndex: 400,
    }}>
      <div style={{
        background: "var(--color-card-bg)",
        border: "1px solid var(--color-border)",
        borderRadius: "12px",
        padding: "40px 48px",
        display: "flex",
        flexDirection: "column",
        alignItems: "center",
        gap: "24px",
        boxShadow: "0 16px 48px rgba(0,0,0,0.5)",
      }}>
        <div className="playback-loading-spinner" />
        <p style={{ margin: 0, fontSize: "15px", fontWeight: 600, color: "var(--color-text)" }}>
          Loading playback...
        </p>
        <button
          onClick={handleCancel}
          style={{
            background: "#e50914",
            color: "#fff",
            border: "none",
            borderRadius: "6px",
            padding: "10px 32px",
            fontSize: "14px",
            fontWeight: 600,
            cursor: "pointer",
          }}
          onMouseEnter={e => (e.currentTarget.style.background = "#c0070f")}
          onMouseLeave={e => (e.currentTarget.style.background = "#e50914")}
        >
          Cancel
        </button>
      </div>
    </div>
  );
}
