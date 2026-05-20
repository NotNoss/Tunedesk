interface SidebarCardProps {
  icon: string;
  label: string;
  onClick?: () => void;
}

export default function SidebarCard({ icon, label, onClick }: SidebarCardProps) {
  return (
    <div onClick={onClick} style={{
      display: "flex",
      alignItems: "center",
      gap: "12px",
      background: "var(--color-card-bg)",
      borderRadius: "8px",
      padding: "10px 14px",
      cursor: "pointer",
      transition: "background 0.15s ease",
    }}
      onMouseEnter={e => (e.currentTarget.style.background = "var(--color-card-hover)")}
      onMouseLeave={e => (e.currentTarget.style.background = "var(--color-card-bg)")}
    >
      <img src={icon} width={20} height={20} alt="" style={{ filter: "var(--icon-filter)" }} />
      <span style={{ color: "var(--color-text)", fontSize: "14px", whiteSpace: "nowrap" }}>
        {label}
      </span>
    </div>
  );
}
