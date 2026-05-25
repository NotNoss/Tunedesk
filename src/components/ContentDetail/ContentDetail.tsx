import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import ResumeModal from "../ResumeModal";
import SeasonDetail, { Episode } from "./SeasonDetail";
import PlayButton from "../PlayButton";

interface ProgressEntry {
  position: number;
  duration: number;
}

export interface ContentInfo {
  name?: string;
  image?: string;
  releaseDate?: string;
  duration?: string;
  rating?: string;
  genre?: string;
  director?: string;
  description?: string;
  cast?: string;
}

type ContentDetailProps = (
  | { type: "movie"; streamId: number; containerExtension: string }
  | { type: "series"; episodes: Record<string, Episode[]> }
) & {
  info: ContentInfo;
  profileName: string;
  onBack: () => void;
  autoPlayNext?: boolean;
};

export default function ContentDetail(props: ContentDetailProps) {
  const { info, profileName, onBack } = props;

  const movieStreamId = props.type === "movie" ? props.streamId : null;

  const [movieProgress, setMovieProgress] = useState<ProgressEntry | null>(null);
  const [showResumeModal, setShowResumeModal] = useState(false);

  useEffect(() => {
    if (movieStreamId === null) return;
    const key = `movie_${movieStreamId}`;
    invoke<Record<string, ProgressEntry>>("get_progress", { profile: profileName, keys: [key] })
      .then(r => setMovieProgress(r[key] ?? null))
      .catch(() => {});
  }, [movieStreamId]);

  const progressPct = movieProgress && movieProgress.duration > 0
    ? Math.min((movieProgress.position / movieProgress.duration) * 100, 100)
    : 0;

  function invokeMoviePlay(startOver: boolean) {
    if (props.type !== "movie") return;
    setShowResumeModal(false);
    invoke("play_vod", {
      name: profileName,
      streamId: props.streamId,
      containerExtension: props.containerExtension,
      startOver,
    })
      .then(() => {
        const key = `movie_${props.streamId}`;
        invoke<Record<string, ProgressEntry>>("get_progress", { profile: profileName, keys: [key] })
          .then(r => setMovieProgress(r[key] ?? null))
          .catch(() => {});
      })
      .catch(console.error);
  }

  function handleMoviePlay() {
    if (progressPct > 0) {
      setShowResumeModal(true);
    } else {
      invokeMoviePlay(false);
    }
  }

  return (
    <div style={{ height: "100%", overflowY: "auto" }}>
      {showResumeModal && props.type === "movie" && (
        <ResumeModal
          title={info.name || "Unknown Title"}
          onResume={() => invokeMoviePlay(false)}
          onStartOver={() => invokeMoviePlay(true)}
          onBack={() => setShowResumeModal(false)}
        />
      )}

      {/* Header */}
      <div style={{
        display: "flex",
        minHeight: "360px",
        ...(props.type === "series" ? { height: "42vh" } : {}),
      }}>

        {/* Left panel */}
        <div style={{
          width: "45%",
          flexShrink: 0,
          padding: "32px 48px 40px",
          display: "flex",
          flexDirection: "column",
          gap: "16px",
          boxSizing: "border-box",
          ...(props.type === "series" ? { overflowY: "auto" } : {}),
        }}>
          <button
            onClick={onBack}
            style={{
              alignSelf: "flex-start",
              background: "transparent",
              border: "none",
              color: "var(--color-text-muted)",
              fontSize: "13px",
              cursor: "pointer",
              padding: 0,
              marginBottom: "8px",
            }}
            onMouseEnter={(e) => (e.currentTarget.style.color = "var(--color-text)")}
            onMouseLeave={(e) => (e.currentTarget.style.color = "var(--color-text-muted)")}
          >
            ← Back
          </button>

          <h1 style={{ margin: 0, fontSize: "30px", fontWeight: 700, color: "var(--color-text)", lineHeight: 1.2 }}>
            {info.name || "Unknown Title"}
          </h1>

          <div style={{ display: "flex", flexWrap: "wrap", gap: "8px 16px", fontSize: "13px", color: "var(--color-text-muted)" }}>
            {info.releaseDate && <span>{info.releaseDate}</span>}
            {info.duration && <span>{info.duration}</span>}
            {info.rating && <span>⭐ {info.rating}</span>}
            {info.genre && <span>{info.genre}</span>}
          </div>

          {info.director && (
            <p style={{ margin: 0, fontSize: "13px", color: "var(--color-text-muted)" }}>
              <span>Directed by </span>
              <span style={{ color: "var(--color-text)" }}>{info.director}</span>
            </p>
          )}

          {info.description && (
            <p style={{ margin: 0, fontSize: "14px", color: "var(--color-text)", lineHeight: 1.7 }}>
              {info.description}
            </p>
          )}

          {info.cast && (
            <div>
              <p style={{ margin: "0 0 6px", fontSize: "11px", fontWeight: 600, letterSpacing: "0.08em", textTransform: "uppercase", color: "var(--color-text-muted)" }}>
                Cast
              </p>
              <p style={{ margin: 0, fontSize: "13px", color: "var(--color-text)", lineHeight: 1.6 }}>
                {info.cast}
              </p>
            </div>
          )}

          {props.type === "movie" && (
            <div style={{ alignSelf: "flex-start", marginTop: "8px" }}>
              <PlayButton
                onClick={handleMoviePlay}
                label={progressPct > 0 ? "Resume" : "Play"}
                progressPct={progressPct}
              />
            </div>
          )}
        </div>

        {/* Right image panel */}
        <div style={{
          flex: 1,
          position: "relative",
          overflow: "hidden",
          ...(props.type === "movie" ? { height: "50vh", minHeight: "360px", alignSelf: "flex-start" } : {}),
        }}>
          {info.image && (
            <img
              src={info.image}
              alt={info.name || ""}
              style={{ width: "100%", height: "100%", objectFit: "contain", objectPosition: "center", display: "block" }}
            />
          )}
          <div style={{
            position: "absolute",
            inset: 0,
            background: "linear-gradient(to right, var(--color-bg) 0%, transparent 25%)",
            pointerEvents: "none",
          }} />
        </div>
      </div>

      {props.type === "series" && (
        <SeasonDetail episodes={props.episodes} profileName={profileName} autoPlayNext={props.autoPlayNext ?? true} />
      )}
    </div>
  );
}
