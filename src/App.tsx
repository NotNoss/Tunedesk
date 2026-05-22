import { useState, useEffect, useRef } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import AboutModal from "./components/AboutModal";
import FetchToast from "./components/FetchToast";
import ContentDetail from "./components/ContentDetail/ContentDetail";
import EditProfileModal from "./components/EditProfileModal";
import MenuBar from "./components/MenuBar";
import PreferencesModal from "./components/PreferencesModal";
import ContentGrid from "./components/ContentGrid";
import NewProfileModal from "./components/NewProfileModal";
import LiveGrid from "./components/LiveGrid/LiveGrid";
import SearchResults, { ProfileSearchResults } from "./components/SearchResults";
import Sidebar from "./components/Sidebar";
import "./App.css";

interface VodStream {
  stream_id: number;
  name: string;
  stream_icon: string;
}

interface VodInfo {
  info: {
    name?: string;
    movie_image?: string;
    releasedate?: string;
    director?: string;
    actors?: string;
    description?: string;
    plot?: string;
    genre?: string;
    duration?: string;
    rating?: string | number;
    container_extension?: string;
  };
}

interface SeriesItem {
  series_id: number;
  name: string;
  cover: string;
}

interface Episode {
  id: string;
  episode_num: number;
  title: string;
  season: number;
  container_extension?: string;
  info?: {
    movie_image?: string;
    plot?: string;
    duration?: string;
    releasedate?: string;
  };
}

interface SeriesInfo {
  info: {
    name?: string;
    cover?: string;
    plot?: string;
    cast?: string;
    director?: string;
    genre?: string;
    releaseDate?: string;
    rating?: string | number;
    episode_run_time?: string;
  };
  episodes: Record<string, Episode[]>;
}

function App() {
  const [showAbout, setShowAbout] = useState(false);
  const [showNewProfile, setShowNewProfile] = useState(false);
  const [showEditProfile, setShowEditProfile] = useState(false);
  const [showPreferences, setShowPreferences] = useState(false);
  const [theme, setTheme] = useState<"dark" | "light">(() => {
    return (localStorage.getItem("theme") as "dark" | "light") ?? "dark";
  });
  const [updateVersion, setUpdateVersion] = useState<string | null>(null);

  useEffect(() => {
    document.documentElement.setAttribute("data-theme", theme === "light" ? "light" : "");
    localStorage.setItem("theme", theme);
  }, [theme]);
  const [profiles, setProfiles] = useState<string[]>([]);
  const [m3u8Profiles, setM3u8Profiles] = useState<Set<string>>(new Set());
  const [profileIcons, setProfileIcons] = useState<Record<string, string>>({});

  const [currentProfileName, setCurrentProfileName] = useState("");
  const [currentCategoryId, setCurrentCategoryId] = useState("");
  const [loading, setLoading] = useState(false);
  const [loadingDetail, setLoadingDetail] = useState(false);

  const [movies, setMovies] = useState<VodStream[]>([]);
  const [series, setSeries] = useState<SeriesItem[]>([]);
  const [contentType, setContentType] = useState<"movies" | "shows" | "live" | null>(null);

  const [movieInfo, setMovieInfo] = useState<VodInfo | null>(null);
  const [selectedStreamId, setSelectedStreamId] = useState<number | null>(null);
  const [seriesInfo, setSeriesInfo] = useState<SeriesInfo | null>(null);

  const [searchQuery, setSearchQuery] = useState("");
  const [searchResults, setSearchResults] = useState<ProfileSearchResults[] | null>(null);
  const searchRef = useRef<HTMLInputElement>(null);

  async function loadProfiles() {
    try {
      const [list, m3u8List, icons] = await Promise.all([
        invoke<string[]>("get_xtream_profiles"),
        invoke<string[]>("get_m3u8_profiles"),
        invoke<Record<string, string>>("get_profile_icons"),
      ]);
      setProfiles(list);
      setM3u8Profiles(new Set(m3u8List));
      setProfileIcons(icons);
      list.forEach(name => invoke("prime_cache", { name }).catch(console.error));
    } catch (e) {
      console.error("loadProfiles failed:", e);
    }
  }

  async function handleSearch() {
    const q = searchQuery.trim();
    if (!q) return;
    const results = await invoke<ProfileSearchResults[]>("search_all_profiles", { query: q });
    setSearchResults(results);
  }

  function clearSearch() {
    setSearchQuery("");
    setSearchResults(null);
  }

  async function handleCategorySelect(profileName: string, categoryId: string, section: "movies" | "shows" | "live") {
    clearSearch();
    setCurrentProfileName(profileName);
    setCurrentCategoryId(categoryId);
    setContentType(section);
    setMovieInfo(null);
    setSeriesInfo(null);
    setMovies([]);
    setSeries([]);
    if (section === "live") return;
    setLoading(true);
    try {
      if (section === "movies") {
        const result = await invoke<VodStream[]>("get_vod_streams", { name: profileName, categoryId });
        setMovies(result);
      } else {
        const result = await invoke<SeriesItem[]>("get_series_items", { name: profileName, categoryId });
        setSeries(result);
      }
    } catch (e) {
      console.error("handleCategorySelect failed:", e);
    } finally {
      setLoading(false);
    }
  }

  async function handleMovieSelect(streamId: number, profileName: string = currentProfileName) {
    clearSearch();
    setCurrentProfileName(profileName);
    setLoadingDetail(true);
    setMovieInfo(null);
    setSelectedStreamId(streamId);
    try {
      const info = await invoke<VodInfo>("get_vod_info", { name: profileName, vodId: streamId });
      setMovieInfo(info);
    } catch (e) {
      console.error("get_vod_info failed:", e);
    } finally {
      setLoadingDetail(false);
    }
  }

  async function handleSeriesSelect(seriesId: number, profileName: string = currentProfileName) {
    clearSearch();
    setCurrentProfileName(profileName);
    setLoadingDetail(true);
    setSeriesInfo(null);
    try {
      const info = await invoke<SeriesInfo>("get_series_info", { name: profileName, seriesId });
      setSeriesInfo(info);
    } catch (e) {
      console.error("get_series_info failed:", e);
    } finally {
      setLoadingDetail(false);
    }
  }

  useEffect(() => {
    loadProfiles();
    const unlisten = listen<string>("update-ready", (e) => setUpdateVersion(e.payload));
    return () => { unlisten.then(fn => fn()); };
  }, []);

  function handleExit() {
    invoke("exit_app");
  }

  const isLoading = loading || loadingDetail;
  const showSearch = searchResults !== null;

  return (
    <div className="app">
      <MenuBar
        onExit={handleExit}
        onAbout={() => setShowAbout(true)}
        onNewProfile={() => setShowNewProfile(true)}
        onEditProfile={() => setShowEditProfile(true)}
        onPreferences={() => setShowPreferences(true)}
      />
      {updateVersion && (
        <div className="update-banner">
          <span>Version {updateVersion} is ready to install.</span>
          <button onClick={() => invoke("restart_to_update")}>Restart now</button>
          <button className="update-banner-dismiss" onClick={() => setUpdateVersion(null)}>✕</button>
        </div>
      )}
      <div className="layout">
        <Sidebar profiles={profiles} m3u8Profiles={m3u8Profiles} profileIcons={profileIcons} onCategorySelect={handleCategorySelect} />
        <main style={{ flex: 1, minHeight: 0, overflow: "hidden", display: "flex", flexDirection: "column" }}>

          {/* Search bar — always visible */}
          <div style={{
            flexShrink: 0,
            padding: "8px 14px",
            borderBottom: "1px solid var(--color-border)",
            display: "flex",
            alignItems: "center",
            gap: 8,
            background: "var(--color-bg)",
          }}>
            <input
              ref={searchRef}
              type="text"
              value={searchQuery}
              onChange={e => {
                setSearchQuery(e.target.value);
                if (!e.target.value) setSearchResults(null);
              }}
              onKeyDown={e => { if (e.key === "Enter") handleSearch(); }}
              placeholder="Search movies, shows, channels across all profiles..."
              style={{
                flex: 1,
                background: "var(--color-card-bg)",
                border: "1px solid var(--color-border)",
                borderRadius: 6,
                padding: "7px 12px",
                color: "var(--color-text)",
                fontSize: "13px",
                outline: "none",
              }}
            />
            {searchQuery && (
              <button
                onClick={clearSearch}
                style={{
                  background: "transparent",
                  border: "none",
                  color: "var(--color-text-muted)",
                  fontSize: "18px",
                  lineHeight: 1,
                  cursor: "pointer",
                  padding: "0 4px",
                }}
                onMouseEnter={e => (e.currentTarget.style.color = "var(--color-text)")}
                onMouseLeave={e => (e.currentTarget.style.color = "var(--color-text-muted)")}
              >
                ×
              </button>
            )}
          </div>

          {/* Search results */}
          {showSearch && (
            <SearchResults
              profileResults={searchResults!}
              query={searchQuery}
              onMovieSelect={handleMovieSelect}
              onShowSelect={handleSeriesSelect}
            />
          )}

          {/* Normal content */}
          {!showSearch && (
            <>
              {isLoading && (
                <div style={{ display: "flex", alignItems: "center", justifyContent: "center", height: "100%", color: "var(--color-text-muted)", fontSize: "14px" }}>
                  Loading...
                </div>
              )}
              {!isLoading && movieInfo && (
                <ContentDetail
                  type="movie"
                  info={{
                    name: movieInfo.info.name,
                    image: movieInfo.info.movie_image,
                    releaseDate: movieInfo.info.releasedate?.substring(0, 4),
                    duration: movieInfo.info.duration,
                    rating: movieInfo.info.rating != null ? String(movieInfo.info.rating) : undefined,
                    genre: movieInfo.info.genre,
                    director: movieInfo.info.director,
                    description: movieInfo.info.description || movieInfo.info.plot,
                    cast: movieInfo.info.actors,
                  }}
                  streamId={selectedStreamId ?? 0}
                  containerExtension={movieInfo.info.container_extension ?? ""}
                  profileName={currentProfileName}
                  onBack={() => setMovieInfo(null)}
                />
              )}
              {!isLoading && seriesInfo && (
                <ContentDetail
                  type="series"
                  info={{
                    name: seriesInfo.info.name,
                    image: seriesInfo.info.cover,
                    releaseDate: seriesInfo.info.releaseDate,
                    duration: seriesInfo.info.episode_run_time ? `${seriesInfo.info.episode_run_time} min / ep` : undefined,
                    rating: seriesInfo.info.rating != null ? String(seriesInfo.info.rating) : undefined,
                    genre: seriesInfo.info.genre,
                    director: seriesInfo.info.director,
                    description: seriesInfo.info.plot,
                    cast: seriesInfo.info.cast,
                  }}
                  episodes={seriesInfo.episodes}
                  profileName={currentProfileName}
                  onBack={() => setSeriesInfo(null)}
                />
              )}
              {!isLoading && !movieInfo && !seriesInfo && contentType === "movies" && movies.length > 0 && (
                <ContentGrid
                  items={movies.map(m => ({ id: m.stream_id, name: m.name, image: m.stream_icon }))}
                  onSelect={handleMovieSelect}
                  profileName={currentProfileName}
                  keyPrefix="movie"
                />
              )}
              {!isLoading && !movieInfo && !seriesInfo && contentType === "shows" && series.length > 0 && (
                <ContentGrid
                  items={series.map(s => ({ id: s.series_id, name: s.name, image: s.cover }))}
                  onSelect={handleSeriesSelect}
                />
              )}
              {!movieInfo && !seriesInfo && contentType === "live" && (
                <LiveGrid profileName={currentProfileName} categoryId={currentCategoryId} onBack={() => setContentType(null)} />
              )}
            </>
          )}

        </main>
      </div>
      {showPreferences && (
        <PreferencesModal
          theme={theme}
          onThemeChange={setTheme}

          onClose={() => setShowPreferences(false)}
        />
      )}
      {showAbout && <AboutModal onClose={() => setShowAbout(false)} />}
      {showNewProfile && (
        <NewProfileModal
          onClose={() => setShowNewProfile(false)}
          onSaved={loadProfiles}
        />
      )}
      {showEditProfile && (
        <EditProfileModal
          xtreamProfiles={profiles}
          m3u8Profiles={m3u8Profiles}
          onClose={() => setShowEditProfile(false)}
          onSaved={loadProfiles}
        />
      )}
      <FetchToast />
    </div>
  );
}

export default App;
