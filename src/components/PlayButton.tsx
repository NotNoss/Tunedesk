interface PlayButtonProps {
  onClick: () => void;
  label: string;
  progressPct?: number;
}

export default function PlayButton({ onClick, label, progressPct = 0 }: PlayButtonProps) {
  return (
    <div>
      <button
        onClick={onClick}
        style={{
          display: "flex",
          alignItems: "center",
          gap: "10px",
          background: "#fff",
          color: "#000",
          border: "none",
          borderRadius: "6px",
          padding: "12px 32px",
          fontSize: "15px",
          fontWeight: 700,
          cursor: "pointer",
        }}
        onMouseEnter={(e) => (e.currentTarget.style.background = "#ddd")}
        onMouseLeave={(e) => (e.currentTarget.style.background = "#fff")}
      >
        ▶ {label}
      </button>
      {progressPct > 0 && (
        <div style={{ marginTop: "6px", height: "3px", background: "var(--color-border)", borderRadius: "2px" }}>
          <div style={{ width: `${progressPct}%`, height: "100%", background: "#e50914", borderRadius: "2px" }} />
        </div>
      )}
    </div>
  );
}
