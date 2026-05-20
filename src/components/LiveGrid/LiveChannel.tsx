import { useRef, useEffect } from "react";
import { LiveStream, EpgListing, safeAtob, fmtTime } from "./liveUtils";

const PX_PER_MIN = 8;
const MINS_BEFORE = 60;
const MINS_AFTER = 300;
const MINS_TOTAL = MINS_BEFORE + MINS_AFTER;
const CHANNEL_COL_W = 90;
const ROW_H = 64;

interface LiveChannelProps {
  channels: LiveStream[];
  epg: Record<number, EpgListing[]>;
  selected: LiveStream | null;
  now: number;
  onSelect: (ch: LiveStream) => void;
}

export default function LiveChannel({ channels, epg, selected, now, onSelect }: LiveChannelProps) {
  const gridRef = useRef<HTMLDivElement>(null);

  const windowStart = now - MINS_BEFORE * 60;
  const nowX = MINS_BEFORE * PX_PER_MIN;
  const totalW = MINS_TOTAL * PX_PER_MIN;

  useEffect(() => {
    if (gridRef.current && channels.length > 0) {
      gridRef.current.scrollLeft = nowX - 16;
    }
  }, [channels.length]);

  const timeLabels: { x: number; label: string }[] = [];
  let t = Math.ceil(windowStart / 1800) * 1800;
  while (t <= windowStart + MINS_TOTAL * 60) {
    timeLabels.push({ x: ((t - windowStart) / 60) * PX_PER_MIN, label: fmtTime(t) });
    t += 1800;
  }

  return (
    <div ref={gridRef} style={{ flex: 1, overflow: "auto" }}>
      <div style={{ minWidth: CHANNEL_COL_W + totalW, width: "100%" }}>

        {/* Sticky time header row */}
        <div style={{
          display: "flex",
          height: 28,
          position: "sticky",
          top: 0,
          zIndex: 10,
          background: "var(--color-bg)",
          borderBottom: "1px solid var(--color-border)",
        }}>
          <div style={{ width: CHANNEL_COL_W, flexShrink: 0, background: "var(--color-bg)", borderRight: "1px solid var(--color-border)" }} />
          <div style={{ position: "relative", flex: 1, minWidth: totalW }}>
            {timeLabels.map(({ x, label }) => (
              <span key={x} style={{
                position: "absolute",
                left: x + 6,
                top: "50%",
                transform: "translateY(-50%)",
                fontSize: "11px",
                color: "var(--color-text-muted)",
                whiteSpace: "nowrap",
              }}>
                {label}
              </span>
            ))}
            <div style={{ position: "absolute", left: nowX, top: 0, bottom: 0, width: 2, background: "#e50914", opacity: 0.7 }} />
          </div>
        </div>

        {/* Channel rows */}
        {channels.map(ch => {
          const isSelected = selected?.stream_id === ch.stream_id;
          const listings = epg[ch.stream_id] ?? [];
          const visible = listings.filter(l => {
            const stop = Number(l.stop_timestamp);
            const start = Number(l.start_timestamp);
            return stop > windowStart && start < windowStart + MINS_TOTAL * 60;
          });

          return (
            <div
              key={ch.stream_id}
              onClick={() => onSelect(ch)}
              style={{
                display: "flex",
                height: ROW_H,
                borderBottom: "1px solid var(--color-border)",
                background: isSelected ? "var(--color-card-hover)" : "transparent",
                cursor: "pointer",
              }}
            >
              {/* Channel info — sticky left */}
              <div style={{
                width: CHANNEL_COL_W,
                flexShrink: 0,
                position: "sticky",
                left: 0,
                zIndex: 2,
                display: "flex",
                alignItems: "center",
                gap: "6px",
                padding: "0 8px",
                background: isSelected ? "var(--color-card-hover)" : "var(--color-sidebar-bg)",
                borderRight: "1px solid var(--color-border)",
              }}>
                <span style={{ fontSize: "10px", color: "var(--color-text-muted)", minWidth: 18, textAlign: "right", flexShrink: 0 }}>
                  {ch.num || ""}
                </span>
                {ch.stream_icon ? (
                  <img
                    src={ch.stream_icon}
                    alt={ch.name}
                    style={{ width: 30, height: 30, objectFit: "contain", borderRadius: 4, flexShrink: 0 }}
                    onError={e => { e.currentTarget.style.display = "none"; }}
                  />
                ) : (
                  <div style={{ width: 30, height: 30, borderRadius: 4, background: "var(--color-card-bg)", flexShrink: 0 }} />
                )}
              </div>

              {/* Shows area */}
              <div style={{ position: "relative", flex: 1, minWidth: totalW }}>
                <div style={{ position: "absolute", left: nowX, top: 0, bottom: 0, width: 2, background: "#e50914", opacity: 0.5, zIndex: 1, pointerEvents: "none" }} />

                {visible.length > 0 ? visible.map(l => {
                  const start = Number(l.start_timestamp);
                  const stop = Number(l.stop_timestamp);
                  const clampedStart = Math.max(start, windowStart);
                  const clampedStop = Math.min(stop, windowStart + MINS_TOTAL * 60);
                  const x = ((clampedStart - windowStart) / 60) * PX_PER_MIN;
                  const w = ((clampedStop - clampedStart) / 60) * PX_PER_MIN;
                  const isCurrent = start <= now && stop > now;
                  const isPast = stop <= now;
                  const progress = isCurrent ? Math.min(((now - start) / (stop - start)) * 100, 100) : 0;

                  return (
                    <div
                      key={start}
                      style={{
                        position: "absolute",
                        left: x + 1,
                        width: Math.max(w - 2, 0),
                        top: 6,
                        bottom: 6,
                        background: isPast ? "rgba(26,46,69,0.5)" : isCurrent ? "var(--color-card-bg)" : "var(--color-sidebar-bg)",
                        borderRadius: 4,
                        border: `1px solid ${isCurrent ? "var(--color-text-muted)" : "var(--color-border)"}`,
                        overflow: "hidden",
                        boxSizing: "border-box",
                        display: "flex",
                        flexDirection: "column",
                        justifyContent: "center",
                        padding: "0 6px",
                      }}
                    >
                      {w > 36 && (
                        <span style={{
                          fontSize: "11px",
                          color: isPast ? "var(--color-text-muted)" : "var(--color-text)",
                          overflow: "hidden",
                          textOverflow: "ellipsis",
                          whiteSpace: "nowrap",
                        }}>
                          {safeAtob(l.title)}
                        </span>
                      )}
                      {isCurrent && (
                        <div style={{ height: 2, background: "rgba(255,255,255,0.15)", marginTop: 3, flexShrink: 0 }}>
                          <div style={{ width: `${progress}%`, height: "100%", background: "#e50914" }} />
                        </div>
                      )}
                    </div>
                  );
                }) : (
                  <div style={{
                    position: "absolute",
                    left: nowX + 1,
                    width: MINS_AFTER * PX_PER_MIN - 2,
                    top: 6,
                    bottom: 6,
                    background: "var(--color-card-bg)",
                    borderRadius: 4,
                    border: "1px solid var(--color-border)",
                    display: "flex",
                    alignItems: "center",
                    padding: "0 8px",
                    overflow: "hidden",
                  }}>
                    <span style={{ fontSize: "11px", color: "var(--color-text-muted)", overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap" }}>
                      {ch.name}
                    </span>
                  </div>
                )}
              </div>
            </div>
          );
        })}
      </div>
    </div>
  );
}
