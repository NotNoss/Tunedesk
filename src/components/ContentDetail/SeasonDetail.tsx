import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import ResumeModal from "../ResumeModal";

export interface Episode {
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

interface ProgressEntry {
  position: number;
  duration: number;
}

interface SeasonDetailProps {
  episodes: Record<string, Episode[]>;
  profileName: string;
  autoPlayNext: boolean;
}

export default function SeasonDetail({ episodes, profileName, autoPlayNext }: SeasonDetailProps) {
  const [episodeProgress, setEpisodeProgress] = useState<Record<string, ProgressEntry>>({});
  const [watched, setWatched] = useState<Set<string>>(new Set());
  const [pendingEpisode, setPendingEpisode] = useState<Episode | null>(null);

  const seasonKeys = Object.keys(episodes).sort((a, b) => parseInt(a) - parseInt(b));

  async function fetchProgress() {
    const keys = Object.values(episodes).flat().map(ep => `episode_${ep.id}`);
    if (keys.length === 0) return;
    const [progress, watchedList] = await Promise.all([
      invoke<Record<string, ProgressEntry>>("get_progress", { profile: profileName, keys }).catch(() => ({} as Record<string, ProgressEntry>)),
      invoke<string[]>("get_watched", { profile: profileName, keys }).catch(() => [] as string[]),
    ]);
    setEpisodeProgress(progress);
    setWatched(new Set(watchedList));
  }

  useEffect(() => { fetchProgress(); }, [episodes]);

  function findNextEpisode(ep: Episode): Episode | null {
    const currentSeasonKey = String(ep.season);
    const currentSeasonEps = episodes[currentSeasonKey] ?? [];
    const currentIdx = currentSeasonEps.findIndex(e => e.id === ep.id);

    if (currentIdx >= 0 && currentIdx < currentSeasonEps.length - 1) {
      return currentSeasonEps[currentIdx + 1];
    }

    const currentSeasonIdx = seasonKeys.indexOf(currentSeasonKey);
    if (currentSeasonIdx >= 0 && currentSeasonIdx < seasonKeys.length - 1) {
      const nextSeasonKey = seasonKeys[currentSeasonIdx + 1];
      const nextSeasonEps = episodes[nextSeasonKey];
      if (nextSeasonEps?.length > 0) {
        return nextSeasonEps[0];
      }
    }

    return null;
  }

  async function invokeEpisodePlay(ep: Episode, startOver: boolean) {
    setPendingEpisode(null);

    try {
      await invoke("play_episode", {
        name: profileName,
        episodeId: ep.id,
        containerExtension: ep.container_extension ?? "",
        startOver,
      });
    } catch (e) {
      invoke("log_event", { level: "info", module: "autoplay", message: `Episode playback error for S${ep.season}E${ep.episode_num}: ${e}` }).catch(() => {});
      console.error(e);
      fetchProgress();
      return;
    }

    const keys = Object.values(episodes).flat().map(e => `episode_${e.id}`);
    const [progress, watchedList] = await Promise.all([
      invoke<Record<string, ProgressEntry>>("get_progress", { profile: profileName, keys }).catch(() => ({} as Record<string, ProgressEntry>)),
      invoke<string[]>("get_watched", { profile: profileName, keys }).catch(() => [] as string[]),
    ]);
    setEpisodeProgress(progress);
    const newWatched = new Set(watchedList);
    setWatched(newWatched);

    if (autoPlayNext) {
      const epKey = `episode_${ep.id}`;
      const prog = progress[epKey];
      const isWatched = newWatched.has(epKey);
      const pct = prog && prog.duration > 0 ? prog.position / prog.duration : 0;

      if (isWatched || pct > 0.9) {
        const next = findNextEpisode(ep);
        if (next) {
          invoke("log_event", { level: "info", module: "autoplay", message: `Auto-playing next episode: S${next.season}E${next.episode_num} "${next.title || `Episode ${next.episode_num}`}"` }).catch(() => {});
          invokeEpisodePlay(next, false);
        } else {
          invoke("log_event", { level: "info", module: "autoplay", message: `Auto-play: no next episode after S${ep.season}E${ep.episode_num} — end of series` }).catch(() => {});
        }
      }
    }
  }

  function handleEpisodePlay(ep: Episode) {
    const prog = episodeProgress[`episode_${ep.id}`];
    if (prog && prog.position > 5) {
      setPendingEpisode(ep);
    } else {
      invokeEpisodePlay(ep, false);
    }
  }

  return (
    <>
      {pendingEpisode && (
        <ResumeModal
          title={pendingEpisode.title || `Episode ${pendingEpisode.episode_num}`}
          onResume={() => invokeEpisodePlay(pendingEpisode, false)}
          onStartOver={() => invokeEpisodePlay(pendingEpisode, true)}
          onBack={() => setPendingEpisode(null)}
        />
      )}

      <div style={{ padding: "8px 32px 48px", background: "var(--color-bg)" }}>
        {seasonKeys.map((seasonKey) => {
          const eps = episodes[seasonKey];
          if (!eps?.length) return null;
          return (
            <div key={seasonKey} style={{ marginBottom: "36px" }}>
              <h3 style={{ margin: "0 0 14px", fontSize: "16px", fontWeight: 600, color: "var(--color-text)" }}>
                Season {seasonKey}
              </h3>
              <div style={{ display: "flex", gap: "12px", overflowX: "auto", paddingBottom: "10px" }}>
                {eps.map((ep) => {
                  const prog = episodeProgress[`episode_${ep.id}`];
                  const episodePct = prog && prog.duration > 0
                    ? Math.min((prog.position / prog.duration) * 100, 100)
                    : 0;
                  return (
                    <div
                      key={ep.id}
                      onClick={() => handleEpisodePlay(ep)}
                      style={{ flexShrink: 0, width: "200px", cursor: "pointer" }}
                    >
                      <div style={{ position: "relative", paddingBottom: "56.25%", borderRadius: "6px", overflow: "hidden", background: "var(--color-card-bg)", border: "1px solid var(--color-border)", marginBottom: "8px" }}>
                        {ep.info?.movie_image ? (
                          <img
                            src={ep.info.movie_image}
                            alt={ep.title}
                            style={{ position: "absolute", inset: 0, width: "100%", height: "100%", objectFit: "cover", display: "block", filter: watched.has(`episode_${ep.id}`) ? "brightness(0.45)" : "none" }}
                            onError={(e) => { e.currentTarget.style.display = "none"; }}
                          />
                        ) : (
                          <div style={{ position: "absolute", inset: 0, display: "flex", alignItems: "center", justifyContent: "center", color: "var(--color-text-muted)", fontSize: "12px" }}>
                            Ep {ep.episode_num}
                          </div>
                        )}
                        {watched.has(`episode_${ep.id}`) && (
                          <img
                            src="/check.svg"
                            alt="Watched"
                            style={{ position: "absolute", top: "6px", left: "6px", width: "20px", height: "20px", opacity: 0.9 }}
                          />
                        )}
                        {episodePct > 0 && (
                          <div style={{ position: "absolute", bottom: 0, left: 0, right: 0, height: "3px", background: "rgba(255,255,255,0.15)" }}>
                            <div style={{ width: `${episodePct}%`, height: "100%", background: "#e50914" }} />
                          </div>
                        )}
                      </div>
                      <p style={{ margin: "0 0 2px", fontSize: "11px", color: "var(--color-text-muted)" }}>
                        Episode {ep.episode_num}{ep.info?.duration ? ` · ${ep.info.duration}` : ""}
                      </p>
                      <p style={{ margin: 0, fontSize: "12px", color: "var(--color-text)", lineHeight: 1.4, whiteSpace: "nowrap", overflow: "hidden", textOverflow: "ellipsis" }}>
                        {ep.title || `Episode ${ep.episode_num}`}
                      </p>
                    </div>
                  );
                })}
              </div>
            </div>
          );
        })}
      </div>
    </>
  );
}
