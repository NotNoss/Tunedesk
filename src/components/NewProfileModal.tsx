import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import Select from "./Select";
import ProfileIconPicker from "./ProfileIconPicker";

interface NewProfileModalProps {
  onClose: () => void;
  onSaved: () => void;
}

const fieldStyle: React.CSSProperties = {
  width: "100%",
  background: "var(--color-sidebar-bg)",
  border: "1px solid var(--color-border)",
  borderRadius: "6px",
  padding: "8px 10px",
  color: "var(--color-text)",
  fontSize: "13px",
  outline: "none",
  boxSizing: "border-box",
};

const labelStyle: React.CSSProperties = {
  display: "block",
  fontSize: "12px",
  color: "var(--color-text-muted)",
  marginBottom: "4px",
};

const fieldGroupStyle: React.CSSProperties = {
  display: "flex",
  flexDirection: "column",
  gap: "12px",
};

export default function NewProfileModal({ onClose, onSaved }: NewProfileModalProps) {
  const [name, setName] = useState("");
  const [icon, setIcon] = useState("tv.svg");
  const [type, setType] = useState<"" | "xtream" | "m3u8">("");

  const [xtreamUrl, setXtreamUrl] = useState("");
  const [xtreamUsername, setXtreamUsername] = useState("");
  const [xtreamPassword, setXtreamPassword] = useState("");

  const [m3uUrl, setM3uUrl] = useState("");
  const [epgUrl, setEpgUrl] = useState("");
  const [error, setError] = useState<string | null>(null);

  async function handleSave() {
    if (!canSave) return;
    try {
      if (type === "xtream") {
        await invoke("save_xtream_profile", {
          name,
          url: xtreamUrl,
          username: xtreamUsername,
          password: xtreamPassword,
        });
      } else if (type === "m3u8") {
        await invoke("save_m3u8_profile", {
          name,
          m3uUrl,
          epgUrl,
        });
      }
      await invoke("set_profile_icon", { name, icon });
      onSaved();
      onClose();
    } catch (e) {
      setError(String(e));
    }
  }

  const canSave =
    name.trim() !== "" &&
    type !== "" &&
    (type === "xtream"
      ? xtreamUrl.trim() !== "" && xtreamUsername.trim() !== "" && xtreamPassword.trim() !== ""
      : m3uUrl.trim() !== "" && epgUrl.trim() !== "");

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
        onClick={(e) => e.stopPropagation()}
        style={{
          background: "var(--color-card-bg)",
          border: "1px solid var(--color-border)",
          borderRadius: "12px",
          padding: "28px 32px",
          width: "400px",
          boxShadow: "0 16px 48px rgba(0,0,0,0.6)",
          display: "flex",
          flexDirection: "column",
          gap: "16px",
        }}
      >
        <h2 style={{ margin: 0, fontSize: "16px", color: "var(--color-text)", fontWeight: 600 }}>
          New Profile
        </h2>

        <div style={fieldGroupStyle}>
          <div>
            <label style={labelStyle}>Name</label>
            <input
              style={fieldStyle}
              value={name}
              onChange={(e) => setName(e.target.value)}
              placeholder="Enter profile name"
            />
          </div>

          <div>
            <label style={labelStyle}>Icon</label>
            <ProfileIconPicker value={icon} onChange={setIcon} />
          </div>

          <div>
            <label style={labelStyle}>Type of playlist</label>
            <Select
              value={type}
              onChange={(v) => setType(v as "" | "xtream" | "m3u8")}
              options={[
                { value: "xtream", label: "Xtream" },
                { value: "m3u8", label: "M3U8" },
              ]}
              placeholder="Select type..."
            />
          </div>

          {type === "xtream" && (
            <>
              <div>
                <label style={labelStyle}>URL</label>
                <input
                  style={fieldStyle}
                  value={xtreamUrl}
                  onChange={(e) => setXtreamUrl(e.target.value)}
                  placeholder="http://example.com"
                />
              </div>
              <div>
                <label style={labelStyle}>Username</label>
                <input
                  style={fieldStyle}
                  value={xtreamUsername}
                  onChange={(e) => setXtreamUsername(e.target.value)}
                  placeholder="Enter username"
                />
              </div>
              <div>
                <label style={labelStyle}>Password</label>
                <input
                  style={fieldStyle}
                  type="password"
                  value={xtreamPassword}
                  onChange={(e) => setXtreamPassword(e.target.value)}
                  placeholder="Enter password"
                />
              </div>
            </>
          )}

          {type === "m3u8" && (
            <>
              <div>
                <label style={labelStyle}>URL to M3U</label>
                <input
                  style={fieldStyle}
                  value={m3uUrl}
                  onChange={(e) => setM3uUrl(e.target.value)}
                  placeholder="http://example.com/playlist.m3u"
                />
              </div>
              <div>
                <label style={labelStyle}>URL to EPG</label>
                <input
                  style={fieldStyle}
                  value={epgUrl}
                  onChange={(e) => setEpgUrl(e.target.value)}
                  placeholder="http://example.com/epg.xml"
                />
              </div>
            </>
          )}
        </div>

        {error && (
          <p style={{ margin: 0, fontSize: "12px", color: "#f87171" }}>{error}</p>
        )}

        <div style={{ display: "flex", justifyContent: "flex-end", gap: "8px", marginTop: "4px" }}>
          <button
            onClick={onClose}
            style={{
              background: "transparent",
              border: "1px solid var(--color-border)",
              color: "var(--color-text-muted)",
              fontSize: "13px",
              padding: "7px 18px",
              borderRadius: "6px",
              cursor: "pointer",
            }}
            onMouseEnter={(e) => (e.currentTarget.style.color = "var(--color-text)")}
            onMouseLeave={(e) => (e.currentTarget.style.color = "var(--color-text-muted)")}
          >
            Cancel
          </button>
          <button
            disabled={!canSave}
            onClick={handleSave}
            style={{
              background: canSave ? "var(--color-card-hover)" : "var(--color-border)",
              border: "1px solid " + (canSave ? "var(--color-text-muted)" : "var(--color-border)"),
              color: canSave ? "var(--color-text)" : "var(--color-text-muted)",
              fontSize: "13px",
              padding: "7px 18px",
              borderRadius: "6px",
              cursor: canSave ? "pointer" : "not-allowed",
              opacity: canSave ? 1 : 0.5,
              transition: "opacity 0.15s, background 0.15s",
            }}
          >
            Save
          </button>
        </div>
      </div>
    </div>
  );
}
