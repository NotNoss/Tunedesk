import PlayButton from "../PlayButton";
import { LiveStream, EpgListing, safeAtob, fmtTime } from "./liveUtils";

interface LiveTopProps {
  selected: LiveStream | null;
  selectedListing: EpgListing | undefined;
  epg: Record<number, EpgListing[]>;
  now: number;
  onBack: () => void;
  onPlay: (ch: LiveStream) => void;
}

export default function LiveTop({ selected, selectedListing, epg, now, onBack, onPlay }: LiveTopProps) {
  return (
    <div style={{
      flexShrink: 0,
      display: "flex",
      height: "200px",
      background: "var(--color-sidebar-bg)",
      borderBottom: "1px solid var(--color-border)",
    }}>
      {/* Channel image / icon */}
      <div style={{
        width: "200px",
        flexShrink: 0,
        display: "flex",
        alignItems: "center",
        justifyContent: "center",
        padding: "16px",
        background: "var(--color-bg)",
      }}>
        {selected?.stream_icon ? (
          <img
            src={selected.stream_icon}
            alt={selected.name}
            style={{ maxWidth: "100%", maxHeight: "100%", objectFit: "contain" }}
            onError={e => { e.currentTarget.style.display = "none"; }}
          />
        ) : (
          <div style={{ width: 64, height: 64, borderRadius: 8, background: "var(--color-card-bg)" }} />
        )}
      </div>

      {/* Info */}
      <div style={{ flex: 1, padding: "20px 24px", display: "flex", flexDirection: "column", gap: "8px", overflow: "hidden" }}>
        <button
          onClick={onBack}
          style={{ alignSelf: "flex-start", background: "transparent", border: "none", color: "var(--color-text-muted)", fontSize: "13px", cursor: "pointer", padding: 0, marginBottom: "2px" }}
          onMouseEnter={e => (e.currentTarget.style.color = "var(--color-text)")}
          onMouseLeave={e => (e.currentTarget.style.color = "var(--color-text-muted)")}
        >
          ← Back
        </button>

        <div style={{ display: "flex", alignItems: "center", gap: "10px" }}>
          <h2 style={{ margin: 0, fontSize: "18px", fontWeight: 700, color: "var(--color-text)" }}>
            {selected?.name ?? "Select a channel"}
          </h2>
          <span style={{ fontSize: "10px", fontWeight: 700, color: "#e50914", border: "1px solid #e50914", borderRadius: "3px", padding: "1px 5px", letterSpacing: "0.06em" }}>
            LIVE
          </span>
        </div>

        {selectedListing ? (
          <>
            <p style={{ margin: 0, fontSize: "13px", color: "var(--color-text)", fontWeight: 600 }}>
              {safeAtob(selectedListing.title)}
              <span style={{ fontWeight: 400, color: "var(--color-text-muted)", marginLeft: 8 }}>
                {fmtTime(Number(selectedListing.start_timestamp))} – {fmtTime(Number(selectedListing.stop_timestamp))}
              </span>
            </p>
            <p style={{
              margin: 0, fontSize: "12px", color: "var(--color-text-muted)", lineHeight: 1.5,
              overflow: "hidden", display: "-webkit-box",
              WebkitLineClamp: 2, WebkitBoxOrient: "vertical",
            }}>
              {safeAtob(selectedListing.description)}
            </p>
          </>
        ) : (
          <p style={{ margin: 0, fontSize: "12px", color: "var(--color-text-muted)" }}>
            {selected && epg[selected.stream_id] !== undefined ? "No guide info available" : "Loading guide…"}
          </p>
        )}

        {selected && (
          <div style={{ alignSelf: "flex-start", marginTop: "auto" }}>
            <PlayButton onClick={() => onPlay(selected)} label="Watch Live" />
          </div>
        )}
      </div>

      {/* Current time */}
      <div style={{ padding: "20px 24px", flexShrink: 0, display: "flex", alignItems: "flex-start" }}>
        <span style={{ fontSize: "15px", fontWeight: 600, color: "var(--color-text)" }}>
          {fmtTime(now)}
        </span>
      </div>
    </div>
  );
}
