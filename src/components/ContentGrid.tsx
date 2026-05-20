import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";

interface ContentItem {
  id: number;
  name: string;
  image: string;
}

interface ContentGridProps {
  items: ContentItem[];
  onSelect: (id: number) => void;
  profileName?: string;
  keyPrefix?: string;
}

export default function ContentGrid({ items, onSelect, profileName, keyPrefix }: ContentGridProps) {
  const [watched, setWatched] = useState<Set<string>>(new Set());

  useEffect(() => {
    if (!profileName || !keyPrefix || items.length === 0) return;
    const keys = items.map(i => `${keyPrefix}_${i.id}`);
    invoke<string[]>("get_watched", { profile: profileName, keys })
      .then(w => setWatched(new Set(w)))
      .catch(() => {});
  }, [items, profileName, keyPrefix]);

  return (
    <div style={{
      height: "100%",
      overflowY: "auto",
      padding: "20px",
      boxSizing: "border-box",
    }}>
      <div style={{
        display: "grid",
        gridTemplateColumns: "repeat(auto-fill, minmax(140px, 1fr))",
        gap: "12px",
      }}>
        {items.map((item) => {
          const isWatched = keyPrefix ? watched.has(`${keyPrefix}_${item.id}`) : false;
          return (
            <div key={item.id} style={{ position: "relative", paddingBottom: "150%" }}>
              <div
                onClick={() => onSelect(item.id)}
                style={{
                  position: "absolute",
                  inset: 0,
                  borderRadius: "8px",
                  overflow: "hidden",
                  background: "var(--color-card-bg)",
                  border: "1px solid var(--color-border)",
                  cursor: "pointer",
                  transition: "transform 0.15s ease, border-color 0.15s ease",
                }}
                onMouseEnter={(e) => {
                  e.currentTarget.style.transform = "scale(1.03)";
                  e.currentTarget.style.borderColor = "var(--color-text-muted)";
                }}
                onMouseLeave={(e) => {
                  e.currentTarget.style.transform = "scale(1)";
                  e.currentTarget.style.borderColor = "var(--color-border)";
                }}
              >
                {item.image && (
                  <img
                    src={item.image}
                    alt={item.name}
                    loading="lazy"
                    style={{ width: "100%", height: "100%", objectFit: "cover", display: "block", filter: isWatched ? "brightness(0.45)" : "none" }}
                    onError={(e) => { e.currentTarget.style.display = "none"; }}
                  />
                )}
                {isWatched && (
                  <img
                    src="/check.svg"
                    alt="Watched"
                    style={{ position: "absolute", top: "6px", left: "6px", width: "22px", height: "22px", opacity: 0.9 }}
                  />
                )}
                <div style={{
                  position: "absolute",
                  bottom: 0,
                  left: 0,
                  right: 0,
                  padding: "24px 8px 8px",
                  background: "linear-gradient(transparent, rgba(0,0,0,0.85))",
                  color: "#fff",
                  fontSize: "11px",
                  lineHeight: "1.3",
                }}>
                  {item.name}
                </div>
              </div>
            </div>
          );
        })}
      </div>
    </div>
  );
}
