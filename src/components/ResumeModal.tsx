interface ResumeModalProps {
  title: string;
  onResume: () => void;
  onStartOver: () => void;
  onBack: () => void;
}

const actionBtnBase: React.CSSProperties = {
  width: "100%",
  border: "none",
  borderRadius: "6px",
  padding: "12px",
  fontSize: "14px",
  fontWeight: 600,
  cursor: "pointer",
};

export default function ResumeModal({ title, onResume, onStartOver, onBack }: ResumeModalProps) {
  return (
    <div style={{
      position: "fixed",
      inset: 0,
      background: "rgba(0,0,0,0.65)",
      display: "flex",
      alignItems: "center",
      justifyContent: "center",
      zIndex: 200,
    }}>
      <div style={{
        background: "var(--color-card-bg)",
        border: "1px solid var(--color-border)",
        borderRadius: "12px",
        padding: "32px",
        width: "340px",
        display: "flex",
        flexDirection: "column",
        gap: "24px",
        boxShadow: "0 16px 48px rgba(0,0,0,0.5)",
      }}>
        <div>
          <p style={{ margin: "0 0 6px", fontSize: "11px", fontWeight: 600, letterSpacing: "0.08em", textTransform: "uppercase", color: "var(--color-text-muted)" }}>
            Continue Watching
          </p>
          <h2 style={{ margin: 0, fontSize: "17px", fontWeight: 700, color: "var(--color-text)", lineHeight: 1.3 }}>
            {title}
          </h2>
        </div>

        <div style={{ display: "flex", flexDirection: "column", gap: "10px" }}>
          <button
            onClick={onResume}
            style={{ ...actionBtnBase, background: "#fff", color: "#000" }}
            onMouseEnter={e => (e.currentTarget.style.background = "#ddd")}
            onMouseLeave={e => (e.currentTarget.style.background = "#fff")}
          >
            ▶ Resume
          </button>
          <button
            onClick={onStartOver}
            style={{ ...actionBtnBase, background: "transparent", color: "var(--color-text)", border: "1px solid var(--color-border)" }}
            onMouseEnter={e => (e.currentTarget.style.background = "var(--color-card-hover)")}
            onMouseLeave={e => (e.currentTarget.style.background = "transparent")}
          >
            ↺ Start Over
          </button>
          <button
            onClick={onBack}
            style={{ ...actionBtnBase, background: "transparent", color: "var(--color-text-muted)", fontWeight: 400 }}
            onMouseEnter={e => (e.currentTarget.style.color = "var(--color-text)")}
            onMouseLeave={e => (e.currentTarget.style.color = "var(--color-text-muted)")}
          >
            ← Back
          </button>
        </div>
      </div>
    </div>
  );
}
