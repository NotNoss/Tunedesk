import { useState, useEffect, useRef } from "react";
import { getCurrentWindow } from "@tauri-apps/api/window";

interface MenuDefinition {
  label: string;
  items: { label: string; action?: () => void }[];
}

interface MenuBarProps {
  onExit: () => void;
  onAbout: () => void;
  onNewProfile: () => void;
  onEditProfile: () => void;
  onPreferences: () => void;
}

const winBtnStyle: React.CSSProperties = {
  background: "transparent",
  border: "none",
  color: "var(--color-text-muted)",
  fontSize: "13px",
  cursor: "pointer",
  padding: "4px 10px",
  borderRadius: "4px",
  lineHeight: 1,
};

export default function MenuBar({ onExit, onAbout, onNewProfile, onEditProfile, onPreferences }: MenuBarProps) {
  const menus: MenuDefinition[] = [
    {
      label: "File",
      items: [
        { label: "New Profile", action: onNewProfile },
        { label: "Edit Profile", action: onEditProfile },
        { label: "Exit", action: onExit },
      ],
    },
    {
      label: "Edit",
      items: [
        { label: "Preferences", action: onPreferences },
      ],
    },
    {
      label: "Help",
      items: [
        { label: "About", action: onAbout },
      ],
    },
  ];

  const [openMenu, setOpenMenu] = useState<string | null>(null);
  const barRef = useRef<HTMLDivElement>(null);
  const win = getCurrentWindow();

  useEffect(() => {
    function handleClickOutside(e: MouseEvent) {
      if (barRef.current && !barRef.current.contains(e.target as Node)) {
        setOpenMenu(null);
      }
    }
    document.addEventListener("mousedown", handleClickOutside);
    return () => document.removeEventListener("mousedown", handleClickOutside);
  }, []);

  return (
    <div
      ref={barRef}
      style={{
        display: "flex",
        alignItems: "center",
        height: "32px",
        background: "var(--color-sidebar-bg)",
        borderBottom: "1px solid var(--color-border)",
        padding: "0 8px",
        flexShrink: 0,
        userSelect: "none",
      }}
    >
      {menus.map((menu) => (
        <div key={menu.label} style={{ position: "relative" }}>
          <button
            onClick={() => setOpenMenu(openMenu === menu.label ? null : menu.label)}
            style={{
              background: openMenu === menu.label ? "var(--color-card-bg)" : "transparent",
              border: "none",
              color: "var(--color-text)",
              fontSize: "13px",
              padding: "4px 10px",
              borderRadius: "4px",
              cursor: "pointer",
            }}
            onMouseEnter={e => {
              if (openMenu && openMenu !== menu.label) setOpenMenu(menu.label);
              (e.currentTarget as HTMLButtonElement).style.background = "var(--color-card-bg)";
            }}
            onMouseLeave={e => {
              if (openMenu !== menu.label)
                (e.currentTarget as HTMLButtonElement).style.background = "transparent";
            }}
          >
            {menu.label}
          </button>

          {openMenu === menu.label && (
            <div
              style={{
                position: "absolute",
                top: "calc(100% + 2px)",
                left: 0,
                minWidth: "160px",
                background: "var(--color-card-bg)",
                border: "1px solid var(--color-border)",
                borderRadius: "6px",
                padding: "4px",
                zIndex: 100,
                boxShadow: "0 8px 24px rgba(0,0,0,0.4)",
              }}
            >
              {menu.items.map((item) => (
                <div
                  key={item.label}
                  onClick={() => { item.action?.(); setOpenMenu(null); }}
                  style={{
                    padding: "6px 12px",
                    fontSize: "13px",
                    color: "var(--color-text)",
                    borderRadius: "4px",
                    cursor: "pointer",
                  }}
                  onMouseEnter={e => (e.currentTarget.style.background = "var(--color-card-hover)")}
                  onMouseLeave={e => (e.currentTarget.style.background = "transparent")}
                >
                  {item.label}
                </div>
              ))}
            </div>
          )}
        </div>
      ))}

      {/* Drag region — fills space between menus and window controls */}
      <div data-tauri-drag-region style={{ flex: 1, height: "100%" }} />

      {/* Window controls */}
      <div style={{ display: "flex", gap: "2px" }}>
        <button
          onClick={() => win.minimize()}
          style={winBtnStyle}
          onMouseEnter={e => (e.currentTarget.style.background = "var(--color-card-bg)")}
          onMouseLeave={e => (e.currentTarget.style.background = "transparent")}
        >
          &#x2212;
        </button>
        <button
          onClick={() => win.toggleMaximize()}
          style={winBtnStyle}
          onMouseEnter={e => (e.currentTarget.style.background = "var(--color-card-bg)")}
          onMouseLeave={e => (e.currentTarget.style.background = "transparent")}
        >
          &#x25A1;
        </button>
        <button
          onClick={onExit}
          style={winBtnStyle}
          onMouseEnter={e => (e.currentTarget.style.color = "#f87171")}
          onMouseLeave={e => (e.currentTarget.style.color = "var(--color-text-muted)")}
        >
          &#x2715;
        </button>
      </div>
    </div>
  );
}
