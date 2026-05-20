export interface LiveStream {
  num: number;
  name: string;
  stream_id: number;
  stream_icon: string;
}

export interface EpgListing {
  title: string;
  description: string;
  start_timestamp: string | number;
  stop_timestamp: string | number;
}

export function safeAtob(s: string): string {
  try { return atob(s); } catch { return s; }
}

export function fmtTime(ts: number): string {
  return new Date(ts * 1000).toLocaleTimeString([], { hour: "numeric", minute: "2-digit" });
}
