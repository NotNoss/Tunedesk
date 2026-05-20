import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import SidebarCard from "./SidebarCard";

interface VodCategory {
  category_id: string;
  category_name: string;
}

interface SidebarProps {
  profiles: string[];
  m3u8Profiles: Set<string>;
  profileIcons: Record<string, string>;
  onCategorySelect: (profileName: string, categoryId: string, section: "movies" | "shows" | "live") => void;
}

type View = "profiles" | "nav" | "categories";

const navItems = ["Live TV", "Movies", "Shows"];

const backButtonStyle: React.CSSProperties = {
  display: "flex",
  alignItems: "center",
  gap: "6px",
  background: "transparent",
  border: "none",
  color: "var(--color-text-muted)",
  fontSize: "12px",
  cursor: "pointer",
  padding: "4px 2px",
  marginBottom: "4px",
  width: "fit-content",
};

const sectionHeaderStyle: React.CSSProperties = {
  margin: "0 0 4px",
  fontSize: "13px",
  fontWeight: 600,
  color: "var(--color-text)",
  padding: "0 2px",
};

const navItemStyle: React.CSSProperties = {
  padding: "10px 14px",
  borderRadius: "8px",
  background: "var(--color-card-bg)",
  color: "var(--color-text)",
  fontSize: "14px",
  cursor: "pointer",
  transition: "background 0.15s ease",
};

export default function Sidebar({ profiles, m3u8Profiles, profileIcons, onCategorySelect }: SidebarProps) {
  const [view, setView] = useState<View>("profiles");
  const [selectedProfile, setSelectedProfile] = useState<string | null>(null);
  const [contentSection, setContentSection] = useState<"movies" | "shows" | "live">("movies");
  const [categories, setCategories] = useState<VodCategory[]>([]);
  const [loadingItem, setLoadingItem] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);

  async function handleNavClick(item: string) {
    if (item !== "Movies" && item !== "Shows" && item !== "Live TV") return;
    const section: "movies" | "shows" | "live" =
      item === "Movies" ? "movies" : item === "Shows" ? "shows" : "live";
    const command =
      item === "Movies" ? "get_vod_categories" :
      item === "Shows" ? "get_series_categories" :
      "get_live_categories";

    setLoadingItem(item);
    setError(null);
    setContentSection(section);
    try {
      const result = await invoke<VodCategory[]>(command, { name: selectedProfile });
      setCategories(result);
      setView("categories");
    } catch (e) {
      setError(String(e));
    } finally {
      setLoadingItem(null);
    }
  }

  return (
    <aside style={{
      width: "240px",
      flexShrink: 0,
      height: "100%",
      background: "var(--color-sidebar-bg)",
      borderRight: "1px solid var(--color-border)",
      padding: "16px 12px",
      display: "flex",
      flexDirection: "column",
      gap: "8px",
      overflowY: "auto",
    }}>
      <div style={{
        padding: "8px 2px 16px",
        borderBottom: "1px solid var(--color-border)",
        marginBottom: "8px",
        display: "flex",
        justifyContent: "center",
        flexShrink: 0,
      }}>
        <img src="/Tunedesk.svg" width={64} height={64} alt="" />
      </div>

      {view === "profiles" && (
        profiles.map((name) => (
          <SidebarCard
            key={name}
            icon={`/${profileIcons[name] ?? "tv.svg"}`}
            label={name}
            onClick={() => { setSelectedProfile(name); setView("nav"); }}
          />
        ))
      )}

      {view === "nav" && (
        <>
          <button
            onClick={() => setView("profiles")}
            style={backButtonStyle}
            onMouseEnter={(e) => (e.currentTarget.style.color = "var(--color-text)")}
            onMouseLeave={(e) => (e.currentTarget.style.color = "var(--color-text-muted)")}
          >
            ← Back
          </button>
          <p style={sectionHeaderStyle}>{selectedProfile}</p>
          {navItems.filter(item => !m3u8Profiles.has(selectedProfile!) || item === "Live TV").map((item) => (
            <div
              key={item}
              onClick={() => handleNavClick(item)}
              style={{ ...navItemStyle, opacity: loadingItem === item ? 0.5 : 1 }}
              onMouseEnter={(e) => (e.currentTarget.style.background = "var(--color-card-hover)")}
              onMouseLeave={(e) => (e.currentTarget.style.background = "var(--color-card-bg)")}
            >
              {loadingItem === item ? "Loading..." : item}
            </div>
          ))}
          {error && <p style={{ fontSize: "12px", color: "#f87171", margin: 0 }}>{error}</p>}
        </>
      )}

      {view === "categories" && (
        <>
          <button
            onClick={() => setView("nav")}
            style={backButtonStyle}
            onMouseEnter={(e) => (e.currentTarget.style.color = "var(--color-text)")}
            onMouseLeave={(e) => (e.currentTarget.style.color = "var(--color-text-muted)")}
          >
            ← Back
          </button>
          <p style={sectionHeaderStyle}>
            {contentSection === "movies" ? "Movies" : contentSection === "shows" ? "Shows" : "Live TV"}
          </p>
          {categories.map((cat) => (
            <div
              key={cat.category_id}
              onClick={() => onCategorySelect(selectedProfile!, cat.category_id, contentSection)}
              style={navItemStyle}
              onMouseEnter={(e) => (e.currentTarget.style.background = "var(--color-card-hover)")}
              onMouseLeave={(e) => (e.currentTarget.style.background = "var(--color-card-bg)")}
            >
              {cat.category_name}
            </div>
          ))}
        </>
      )}
    </aside>
  );
}
