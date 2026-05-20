import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import Select from "./Select";
import ProfileIconPicker from "./ProfileIconPicker";

interface EditProfileModalProps {
  xtreamProfiles: string[];
  m3u8Profiles: Set<string>;
  onClose: () => void;
  onSaved: () => void;
}

interface XtreamCreds {
  url: string;
  username: string;
  password: string;
}

interface M3u8Creds {
  m3u_url: string;
  epg_url: string;
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

const cancelBtnStyle: React.CSSProperties = {
  background: "transparent",
  border: "1px solid var(--color-border)",
  color: "var(--color-text-muted)",
  fontSize: "13px",
  padding: "7px 18px",
  borderRadius: "6px",
  cursor: "pointer",
};

export default function EditProfileModal({ xtreamProfiles, m3u8Profiles, onClose, onSaved }: EditProfileModalProps) {
  const allProfiles = [...xtreamProfiles, ...Array.from(m3u8Profiles)];

  const [selectedProfile, setSelectedProfile] = useState("");
  const [name, setName] = useState("");
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [showDeleteConfirm, setShowDeleteConfirm] = useState(false);

  const [url, setUrl] = useState("");
  const [username, setUsername] = useState("");
  const [password, setPassword] = useState("");

  const [m3uUrl, setM3uUrl] = useState("");
  const [epgUrl, setEpgUrl] = useState("");

  const [icon, setIcon] = useState("tv.svg");
  const [origIcon, setOrigIcon] = useState("tv.svg");

  const [origName, setOrigName] = useState("");
  const [origUrl, setOrigUrl] = useState("");
  const [origUsername, setOrigUsername] = useState("");
  const [origPassword, setOrigPassword] = useState("");
  const [origM3uUrl, setOrigM3uUrl] = useState("");
  const [origEpgUrl, setOrigEpgUrl] = useState("");

  const isM3u8 = m3u8Profiles.has(selectedProfile);
  const hasProfile = selectedProfile !== "";

  const isDirty =
    hasProfile &&
    (name !== origName ||
      icon !== origIcon ||
      (isM3u8
        ? m3uUrl !== origM3uUrl || epgUrl !== origEpgUrl
        : url !== origUrl || username !== origUsername || password !== origPassword));

  async function loadProfile(profileName: string) {
    setLoading(true);
    setError(null);
    try {
      setName(profileName);
      setOrigName(profileName);
      const icons = await invoke<Record<string, string>>("get_profile_icons");
      const profileIcon = icons[profileName] ?? "tv.svg";
      setIcon(profileIcon);
      setOrigIcon(profileIcon);
      if (m3u8Profiles.has(profileName)) {
        const creds = await invoke<M3u8Creds>("get_m3u8_profile", { name: profileName });
        setM3uUrl(creds.m3u_url);
        setEpgUrl(creds.epg_url);
        setOrigM3uUrl(creds.m3u_url);
        setOrigEpgUrl(creds.epg_url);
        setUrl(""); setUsername(""); setPassword("");
        setOrigUrl(""); setOrigUsername(""); setOrigPassword("");
      } else {
        const creds = await invoke<XtreamCreds>("get_xtream_profile", { name: profileName });
        setUrl(creds.url);
        setUsername(creds.username);
        setPassword(creds.password);
        setOrigUrl(creds.url);
        setOrigUsername(creds.username);
        setOrigPassword(creds.password);
        setM3uUrl(""); setEpgUrl("");
        setOrigM3uUrl(""); setOrigEpgUrl("");
      }
    } catch (e) {
      setError(String(e));
    } finally {
      setLoading(false);
    }
  }

  function handleProfileSelect(profileName: string) {
    setSelectedProfile(profileName);
    setError(null);
    if (profileName) {
      loadProfile(profileName);
    }
  }

  async function handleSave() {
    if (!isDirty) return;
    setError(null);
    try {
      const oldName = origName;
      if (isM3u8) {
        await invoke("save_m3u8_profile", { name, m3uUrl, epgUrl });
      } else {
        await invoke("save_xtream_profile", { name, url, username, password });
      }
      if (name !== oldName) {
        await invoke("delete_profile", { name: oldName });
      }
      await invoke("set_profile_icon", { name, icon });
      onSaved();
      onClose();
    } catch (e) {
      setError(String(e));
    }
  }

  async function handleDelete() {
    try {
      await invoke("delete_profile", { name: selectedProfile });
      onSaved();
      onClose();
    } catch (e) {
      setError(String(e));
      setShowDeleteConfirm(false);
    }
  }

  return (
    <>
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
            width: "420px",
            boxShadow: "0 16px 48px rgba(0,0,0,0.6)",
            display: "flex",
            flexDirection: "column",
            gap: "16px",
          }}
        >
          <h2 style={{ margin: 0, fontSize: "16px", color: "var(--color-text)", fontWeight: 600 }}>
            Edit Profile
          </h2>

          <div>
            <label style={labelStyle}>Profile</label>
            <Select
              value={selectedProfile}
              onChange={handleProfileSelect}
              options={allProfiles.map((p) => ({ value: p, label: p }))}
              placeholder="Select a profile..."
            />
          </div>

          {hasProfile && (
            <div style={{ display: "flex", flexDirection: "column", gap: "12px" }}>
              <div>
                <label style={labelStyle}>Name</label>
                <input
                  style={fieldStyle}
                  value={name}
                  onChange={(e) => setName(e.target.value)}
                  placeholder="Profile name"
                />
              </div>

              <div>
                <label style={labelStyle}>Icon</label>
                <ProfileIconPicker value={icon} onChange={setIcon} />
              </div>

              {loading && (
                <p style={{ margin: 0, fontSize: "13px", color: "var(--color-text-muted)" }}>Loading...</p>
              )}

              {!loading && isM3u8 && (
                <>
                  <div>
                    <label style={labelStyle}>M3U URL</label>
                    <input
                      style={fieldStyle}
                      value={m3uUrl}
                      onChange={(e) => setM3uUrl(e.target.value)}
                      placeholder="http://example.com/playlist.m3u"
                    />
                  </div>
                  <div>
                    <label style={labelStyle}>EPG URL</label>
                    <input
                      style={fieldStyle}
                      value={epgUrl}
                      onChange={(e) => setEpgUrl(e.target.value)}
                      placeholder="http://example.com/epg.xml"
                    />
                  </div>
                </>
              )}

              {!loading && !isM3u8 && (
                <>
                  <div>
                    <label style={labelStyle}>URL</label>
                    <input
                      style={fieldStyle}
                      value={url}
                      onChange={(e) => setUrl(e.target.value)}
                      placeholder="http://example.com"
                    />
                  </div>
                  <div>
                    <label style={labelStyle}>Username</label>
                    <input
                      style={fieldStyle}
                      value={username}
                      onChange={(e) => setUsername(e.target.value)}
                      placeholder="Enter username"
                    />
                  </div>
                  <div>
                    <label style={labelStyle}>Password</label>
                    <input
                      style={fieldStyle}
                      type="password"
                      value={password}
                      onChange={(e) => setPassword(e.target.value)}
                      placeholder="Enter password"
                    />
                  </div>
                </>
              )}
            </div>
          )}

          {error && (
            <p style={{ margin: 0, fontSize: "12px", color: "#f87171" }}>{error}</p>
          )}

          <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center", marginTop: "4px" }}>
            <button
              onClick={onClose}
              style={cancelBtnStyle}
              onMouseEnter={(e) => (e.currentTarget.style.color = "var(--color-text)")}
              onMouseLeave={(e) => (e.currentTarget.style.color = "var(--color-text-muted)")}
            >
              Cancel
            </button>
            <div style={{ display: "flex", gap: "8px" }}>
              <button
                onClick={() => hasProfile && setShowDeleteConfirm(true)}
                disabled={!hasProfile}
                style={{
                  background: "transparent",
                  border: `1px solid ${hasProfile ? "#dc2626" : "#7f1d1d"}`,
                  color: hasProfile ? "#dc2626" : "#7f1d1d",
                  fontSize: "13px",
                  padding: "7px 18px",
                  borderRadius: "6px",
                  cursor: hasProfile ? "pointer" : "not-allowed",
                  opacity: hasProfile ? 1 : 0.5,
                  transition: "color 0.15s, border-color 0.15s",
                }}
              >
                Delete
              </button>
              <button
                disabled={!isDirty}
                onClick={handleSave}
                style={{
                  background: isDirty ? "var(--color-card-hover)" : "var(--color-border)",
                  border: "1px solid " + (isDirty ? "var(--color-text-muted)" : "var(--color-border)"),
                  color: isDirty ? "var(--color-text)" : "var(--color-text-muted)",
                  fontSize: "13px",
                  padding: "7px 18px",
                  borderRadius: "6px",
                  cursor: isDirty ? "pointer" : "not-allowed",
                  opacity: isDirty ? 1 : 0.5,
                  transition: "opacity 0.15s, background 0.15s",
                }}
              >
                Save
              </button>
            </div>
          </div>
        </div>
      </div>

      {showDeleteConfirm && (
        <div
          onClick={() => setShowDeleteConfirm(false)}
          style={{
            position: "fixed",
            inset: 0,
            background: "rgba(0,0,0,0.7)",
            display: "flex",
            alignItems: "center",
            justifyContent: "center",
            zIndex: 300,
          }}
        >
          <div
            onClick={(e) => e.stopPropagation()}
            style={{
              background: "var(--color-card-bg)",
              border: "1px solid var(--color-border)",
              borderRadius: "12px",
              padding: "28px 32px",
              width: "360px",
              boxShadow: "0 16px 48px rgba(0,0,0,0.6)",
              display: "flex",
              flexDirection: "column",
              gap: "16px",
            }}
          >
            <h2 style={{ margin: 0, fontSize: "16px", color: "var(--color-text)", fontWeight: 600 }}>
              Delete Profile
            </h2>
            <p style={{ margin: 0, fontSize: "13px", color: "var(--color-text-muted)", lineHeight: 1.5 }}>
              Are you sure you want to delete{" "}
              <strong style={{ color: "var(--color-text)" }}>{selectedProfile}</strong>? This action cannot be undone.
            </p>
            <div style={{ display: "flex", justifyContent: "flex-end", gap: "8px" }}>
              <button
                onClick={() => setShowDeleteConfirm(false)}
                style={cancelBtnStyle}
                onMouseEnter={(e) => (e.currentTarget.style.color = "var(--color-text)")}
                onMouseLeave={(e) => (e.currentTarget.style.color = "var(--color-text-muted)")}
              >
                No
              </button>
              <button
                onClick={handleDelete}
                style={{
                  background: "#dc2626",
                  border: "1px solid #dc2626",
                  color: "#fff",
                  fontSize: "13px",
                  padding: "7px 18px",
                  borderRadius: "6px",
                  cursor: "pointer",
                }}
                onMouseEnter={(e) => (e.currentTarget.style.background = "#b91c1c")}
                onMouseLeave={(e) => (e.currentTarget.style.background = "#dc2626")}
              >
                Yes, Delete
              </button>
            </div>
          </div>
        </div>
      )}
    </>
  );
}
