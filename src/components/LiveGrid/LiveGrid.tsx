import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { LiveStream, EpgListing } from "./liveUtils";
import LiveTop from "./LiveTop";
import LiveChannel from "./LiveChannel";

interface LiveGridProps {
  profileName: string;
  categoryId: string;
  onBack: () => void;
}

export default function LiveGrid({ profileName, categoryId, onBack }: LiveGridProps) {
  const [channels, setChannels] = useState<LiveStream[]>([]);
  const [epg, setEpg] = useState<Record<number, EpgListing[]>>({});
  const [loading, setLoading] = useState(true);
  const [now, setNow] = useState(() => Math.floor(Date.now() / 1000));
  const [selected, setSelected] = useState<LiveStream | null>(null);

  useEffect(() => {
    invoke<LiveStream[]>("get_live_streams", { name: profileName, categoryId })
      .then(chs => {
        setChannels(chs);
        setLoading(false);
        if (chs.length > 0) setSelected(chs[0]);

        const BATCH_SIZE = 20;
        for (let i = 0; i < chs.length; i += BATCH_SIZE) {
          const batch = chs.slice(i, i + BATCH_SIZE);
          Promise.all(
            batch.map(ch =>
              invoke<{ epg_listings: EpgListing[] }>("get_channel_epg", {
                name: profileName,
                streamId: ch.stream_id,
              }).catch(() => ({ epg_listings: [] }))
            )
          ).then(results => {
            setEpg(prev => {
              const map = { ...prev };
              batch.forEach((ch, j) => { map[ch.stream_id] = results[j].epg_listings; });
              return map;
            });
          });
        }
      })
      .catch(() => setLoading(false));

    const tick = setInterval(() => setNow(Math.floor(Date.now() / 1000)), 30000);
    return () => clearInterval(tick);
  }, [categoryId]);

  function getCurrentListing(streamId: number): EpgListing | undefined {
    return epg[streamId]?.find(l => {
      const start = Number(l.start_timestamp);
      const stop = Number(l.stop_timestamp);
      return start <= now && stop > now;
    });
  }

  function handlePlay(ch: LiveStream) {
    invoke("play_live", { name: profileName, streamId: ch.stream_id }).catch(console.error);
  }

  const selectedListing = selected ? getCurrentListing(selected.stream_id) : undefined;

  if (loading) {
    return (
      <div style={{ display: "flex", alignItems: "center", justifyContent: "center", height: "100%", color: "var(--color-text-muted)", fontSize: "14px" }}>
        Loading...
      </div>
    );
  }

  return (
    <div style={{ height: "100%", display: "flex", flexDirection: "column", overflow: "hidden" }}>

      <LiveTop
        selected={selected}
        selectedListing={selectedListing}
        epg={epg}
        now={now}
        onBack={onBack}
        onPlay={handlePlay}
      />

      <LiveChannel
        channels={channels}
        epg={epg}
        selected={selected}
        now={now}
        onSelect={setSelected}
      />
    </div>
  );
}
