const PROFILE_ICONS = [
  { file: "tv.svg", label: "TV" },
  { file: "fire.svg", label: "Fire" },
  { file: "comedy.svg", label: "Comedy" },
  { file: "football.svg", label: "Football" },
];

interface ProfileIconPickerProps {
  value: string;
  onChange: (icon: string) => void;
}

export { PROFILE_ICONS };

export default function ProfileIconPicker({ value, onChange }: ProfileIconPickerProps) {
  return (
    <div style={{ display: "flex", gap: "8px" }}>
      {PROFILE_ICONS.map(({ file, label }) => {
        const selected = value === file;
        return (
          <button
            key={file}
            type="button"
            onClick={() => onChange(file)}
            title={label}
            style={{
              width: "44px",
              height: "44px",
              borderRadius: "8px",
              border: selected ? "2px solid var(--color-text)" : "1px solid var(--color-border)",
              background: selected ? "var(--color-card-hover)" : "var(--color-sidebar-bg)",
              cursor: "pointer",
              display: "flex",
              alignItems: "center",
              justifyContent: "center",
              padding: "8px",
              transition: "border-color 0.15s, background 0.15s",
              flexShrink: 0,
            }}
          >
            <img src={`/${file}`} alt={label} style={{ width: "22px", height: "22px", filter: "var(--icon-filter, none)" }} />
          </button>
        );
      })}
    </div>
  );
}
