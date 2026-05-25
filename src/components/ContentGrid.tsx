import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import ContextMenu from "./ContextMenu";

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

interface ContextMenuState {
  x: number;
  y: number;
  item: ContentItem;
}

interface SeriesInfo {
  episodes: Record<string, { id: string }[]>;
}

export default function ContentGrid({ items, onSelect, profileName, keyPrefix }: ContentGridProps) {
  const [watched, setWatched] = useState<Set<string>>(new Set());
  const [progress, setProgress] = useState<Record<string, { position: number; duration: number }>>({});
  const [contextMenu, setContextMenu] = useState<ContextMenuState | null>(null);

  useEffect(() => {
    if (!profileName || !keyPrefix || items.length === 0) return;
    const keys = items.map(i => `${keyPrefix}_${i.id}`);
    const progressFetch = keyPrefix === "movie"
      ? invoke<Record<string, { position: number; duration: number }>>("get_progress", { profile: profileName, keys })
      : Promise.resolve({} as Record<string, { position: number; duration: number }>);
    Promise.all([
      invoke<string[]>("get_watched", { profile: profileName, keys }),
      progressFetch,
    ]).then(([w, p]) => {
      setWatched(new Set(w));
      setProgress(p);
    }).catch(() => {});
  }, [items, profileName, keyPrefix]);

  function handleContextMenu(e: React.MouseEvent, item: ContentItem) {
    e.preventDefault();
    e.stopPropagation();
    setContextMenu({ x: e.clientX, y: e.clientY, item });
  }

  async function collectKeys(item: ContentItem): Promise<string[]> {
    const primaryKey = `${keyPrefix}_${item.id}`;
    const keys = [primaryKey];
    if (keyPrefix === "series" && profileName) {
      try {
        const info = await invoke<SeriesInfo>("get_series_info", { name: profileName, seriesId: item.id });
        const episodeKeys = Object.values(info.episodes).flat().map(ep => `episode_${ep.id}`);
        keys.push(...episodeKeys);
      } catch {}
    }
    return keys;
  }

  async function markWatched(item: ContentItem) {
    if (!profileName || !keyPrefix) return;
    const keys = await collectKeys(item);
    await invoke("set_watched", { profile: profileName, keys }).catch(() => {});
    setWatched(prev => {
      const next = new Set(prev);
      next.add(`${keyPrefix}_${item.id}`);
      return next;
    });
  }

  async function markUnwatched(item: ContentItem) {
    if (!profileName || !keyPrefix) return;
    const keys = await collectKeys(item);
    await invoke("set_unwatched", { profile: profileName, keys }).catch(() => {});
    setWatched(prev => {
      const next = new Set(prev);
      next.delete(`${keyPrefix}_${item.id}`);
      return next;
    });
  }

  function buildMenuItems(item: ContentItem) {
    const key = keyPrefix ? `${keyPrefix}_${item.id}` : "";
    const isWatched = watched.has(key);
    const hasProgress = !!progress[key];
    const isSeries = keyPrefix === "series";
    const label = isSeries ? "series" : "this";
    const menuItems: { label: string; onClick: () => void }[] = [];
    if (!isWatched) {
      menuItems.push({ label: `Mark ${label} as watched`, onClick: () => markWatched(item) });
    }
    if (isWatched || hasProgress) {
      menuItems.push({ label: `Mark ${label} as unwatched`, onClick: () => markUnwatched(item) });
    }
    return menuItems;
  }

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
                onContextMenu={e => handleContextMenu(e, item)}
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
      {contextMenu && profileName && keyPrefix && (
        <ContextMenu
          x={contextMenu.x}
          y={contextMenu.y}
          items={buildMenuItems(contextMenu.item)}
          onClose={() => setContextMenu(null)}
        />
      )}
    </div>
  );
}
