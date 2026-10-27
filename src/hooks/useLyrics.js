import { useEffect, useState } from "react";

const cache = new Map(); // trackKey → { status, lines, plain }

/** "[01:23.45]texto" (y varias marcas por línea) → [{ t, text }] ordenado por tiempo. */
export function parseLrc(lrc) {
  const lines = [];
  for (const raw of lrc.split("\n")) {
    const stamps = [...raw.matchAll(/\[(\d+):(\d+(?:\.\d+)?)\]/g)];
    if (!stamps.length) continue;
    const text = raw.replace(/\[[^\]]*\]/g, "").trim();
    for (const m of stamps) lines.push({ t: Number(m[1]) * 60 + Number(m[2]), text });
  }
  return lines.sort((a, b) => a.t - b.t);
}

/** Quita ruido típico de títulos de YouTube: «(Official Video)», «[HD]», «- Topic»… */
const clean = (s) =>
  s
    .replace(/\s*[(\[][^)\]]*(official|video|lyric|audio|hd|4k|remaster|visualizer|letra)[^)\]]*[)\]]/gi, "")
    .replace(/\s*-\s*topic$/i, "")
    .replace(/\s+/g, " ")
    .trim();

async function lrclib(path, params) {
  const res = await fetch(`https://lrclib.net/api/${path}?${new URLSearchParams(params)}`);
  if (!res.ok) return null;
  return res.json();
}

const pick = (r) =>
  r?.syncedLyrics
    ? { status: "synced", lines: parseLrc(r.syncedLyrics), plain: r.plainLyrics ?? "" }
    : r?.plainLyrics
      ? { status: "plain", lines: [], plain: r.plainLyrics }
      : null;

async function find({ title, artist, album, duration }) {
  const t = clean(title);
  const a = clean(artist);
  if (!t) return { status: "none", lines: [], plain: "" };

  const exact = await lrclib("get", {
    track_name: t,
    artist_name: a,
    ...(album ? { album_name: album } : {}),
    ...(duration ? { duration: Math.round(duration) } : {}),
  }).catch(() => null);
  const direct = pick(exact);
  if (direct?.status === "synced") return direct;

  // Sin coincidencia exacta: búsqueda aproximada, prefiriendo letra sincronizada y duración parecida.
  const list = (await lrclib("search", { q: `${a} ${t}`.trim() }).catch(() => null)) ?? [];
  const ranked = list
    .filter((r) => r.syncedLyrics || r.plainLyrics)
    .map((r) => ({ r, score: (r.syncedLyrics ? 0 : 100) + (duration ? Math.abs((r.duration ?? 0) - duration) : 0) }))
    .sort((x, y) => x.score - y.score);
  return pick(ranked[0]?.r) ?? direct ?? { status: "none", lines: [], plain: "" };
}

/** Letra sincronizada de LRCLIB para la canción actual; solo consulta si `enabled`. */
export function useLyrics(track, trackKey, enabled) {
  const [state, setState] = useState({ status: "idle", lines: [], plain: "" });

  useEffect(() => {
    if (!enabled || !trackKey || !track.title) return setState({ status: "idle", lines: [], plain: "" });
    const hit = cache.get(trackKey);
    if (hit) return setState(hit);
    let cancelled = false;
    setState({ status: "loading", lines: [], plain: "" });
    find(track)
      .then((r) => {
        cache.set(trackKey, r);
        if (!cancelled) setState(r);
      })
      .catch(() => !cancelled && setState({ status: "error", lines: [], plain: "" }));
    return () => {
      cancelled = true;
    };
    // `track.duration` suele llegar tarde: se consulta una vez por pista, con lo que haya al empezar.
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [enabled, trackKey]);

  return state;
}

/** Índice de la línea activa para una posición en segundos (-1 antes de la primera). */
export function activeLine(lines, position) {
  let lo = 0;
  let hi = lines.length - 1;
  let ans = -1;
  while (lo <= hi) {
    const mid = (lo + hi) >> 1;
    if (lines[mid].t <= position + 0.25) {
      ans = mid;
      lo = mid + 1;
    } else hi = mid - 1;
  }
  return ans;
}
