import { invoke } from "@tauri-apps/api/core";

export interface SearchResult {
  kind: "movie" | "show" | "live";
  id: number;
  name: string;
  icon: string;
}

export interface ProfileSearchResults {
  profile: string;
  results: SearchResult[];
}

interface SearchResultsProps {
  profileResults: ProfileSearchResults[];
  query: string;
  onMovieSelect: (id: number, profileName: string) => void;
  onShowSelect: (id: number, profileName: string) => void;
}

const kindBadge: Record<string, { label: string; color: string }> = {
  live:  { label: "Live",  color: "#e50914" },
  movie: { label: "Movie", color: "#3b82f6" },
  show:  { label: "Show",  color: "#8b5cf6" },
};

function ResultCard({
  result,
  profileName,
  onMovieSelect,
  onShowSelect,
}: {
  result: SearchResult;
  profileName: string;
  onMovieSelect: (id: number, profileName: string) => void;
  onShowSelect: (id: number, profileName: string) => void;
}) {
  const badge = kindBadge[result.kind];

  function handleClick() {
    if (result.kind === "movie") onMovieSelect(result.id, profileName);
    else if (result.kind === "show") onShowSelect(result.id, profileName);
    else invoke("play_live", { name: profileName, streamId: result.id }).catch(console.error);
  }

  return (
    <div
      onClick={handleClick}
      style={{
        width: 120,
        flexShrink: 0,
        cursor: "pointer",
        borderRadius: 8,
        overflow: "hidden",
        background: "var(--color-card-bg)",
        transition: "transform 0.1s",
      }}
      onMouseEnter={e => (e.currentTarget.style.transform = "scale(1.04)")}
      onMouseLeave={e => (e.currentTarget.style.transform = "scale(1)")}
    >
      <div style={{
        height: 80,
        background: "var(--color-sidebar-bg)",
        display: "flex",
        alignItems: "center",
        justifyContent: "center",
        overflow: "hidden",
        position: "relative",
      }}>
        {result.icon ? (
          <img
            src={result.icon}
            alt={result.name}
            style={{ width: "100%", height: "100%", objectFit: "cover" }}
            onError={e => { e.currentTarget.style.display = "none"; }}
          />
        ) : (
          <div style={{ width: 36, height: 36, borderRadius: 6, background: "var(--color-border)" }} />
        )}
        <span style={{
          position: "absolute",
          top: 5,
          right: 5,
          fontSize: "9px",
          fontWeight: 700,
          letterSpacing: "0.05em",
          color: "#fff",
          background: badge.color,
          borderRadius: 3,
          padding: "2px 4px",
        }}>
          {badge.label.toUpperCase()}
        </span>
      </div>
      <div style={{ padding: "5px 7px 7px" }}>
        <p style={{
          margin: 0,
          fontSize: "11px",
          color: "var(--color-text)",
          fontWeight: 600,
          overflow: "hidden",
          textOverflow: "ellipsis",
          whiteSpace: "nowrap",
        }}>
          {result.name}
        </p>
      </div>
    </div>
  );
}

function ProfileRow({
  psr,
  onMovieSelect,
  onShowSelect,
}: {
  psr: ProfileSearchResults;
  onMovieSelect: (id: number, profileName: string) => void;
  onShowSelect: (id: number, profileName: string) => void;
}) {
  return (
    <div style={{ marginBottom: 24 }}>
      <p style={{
        margin: "0 0 8px",
        fontSize: "11px",
        fontWeight: 700,
        color: "var(--color-text-muted)",
        textTransform: "uppercase",
        letterSpacing: "0.08em",
      }}>
        {psr.profile} — {psr.results.length} result{psr.results.length !== 1 ? "s" : ""}
      </p>
      <div style={{
        display: "flex",
        gap: 10,
        overflowX: "auto",
        paddingBottom: 6,
      }}>
        {psr.results.map(r => (
          <ResultCard
            key={`${r.kind}-${r.id}`}
            result={r}
            profileName={psr.profile}
            onMovieSelect={onMovieSelect}
            onShowSelect={onShowSelect}
          />
        ))}
      </div>
    </div>
  );
}

export default function SearchResults({ profileResults, query, onMovieSelect, onShowSelect }: SearchResultsProps) {
  if (profileResults.length === 0) {
    return (
      <div style={{
        flex: 1,
        display: "flex",
        alignItems: "center",
        justifyContent: "center",
        color: "var(--color-text-muted)",
        fontSize: "14px",
      }}>
        No results for &ldquo;{query}&rdquo;
      </div>
    );
  }

  return (
    <div style={{ flex: 1, overflowY: "auto", padding: "16px 20px" }}>
      {profileResults.map(psr => (
        <ProfileRow
          key={psr.profile}
          psr={psr}
          onMovieSelect={onMovieSelect}
          onShowSelect={onShowSelect}
        />
      ))}
    </div>
  );
}
