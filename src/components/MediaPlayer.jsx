import { useEffect, useRef, useState } from "react";
import { fmtTime } from "../hooks/usePomodoro.js";
import { useStore } from "../hooks/useStore.js";
import { activeLine, useLyrics } from "../hooks/useLyrics.js";
import Visualizer from "./Visualizer.jsx";

const ctl =
  "flex h-10 w-10 items-center justify-center rounded-full text-lg transition-all duration-300 hover:bg-white/10 active:scale-90 disabled:pointer-events-none disabled:opacity-25";

/** Color dominante de la carátula (promedio ponderado por saturación) para el brillo del reproductor. */
function useArtColor(art) {
  const [color, setColor] = useState(null);
  useEffect(() => {
    if (!art) return setColor(null);
    let cancelled = false;
    const img = new Image();
    img.onload = () => {
      if (cancelled) return;
      try {
        const c = document.createElement("canvas");
        c.width = c.height = 16;
        const ctx = c.getContext("2d", { willReadFrequently: true });
        ctx.drawImage(img, 0, 0, 16, 16);
        const d = ctx.getImageData(0, 0, 16, 16).data;
        let r = 0, g = 0, b = 0, w = 0;
        for (let i = 0; i < d.length; i += 4) {
          const mx = Math.max(d[i], d[i + 1], d[i + 2]);
          const mn = Math.min(d[i], d[i + 1], d[i + 2]);
          const wt = 0.1 + (mx - mn) / 255; // los píxeles saturados pesan más
          r += d[i] * wt; g += d[i + 1] * wt; b += d[i + 2] * wt; w += wt;
        }
        setColor(`${Math.round(r / w)}, ${Math.round(g / w)}, ${Math.round(b / w)}`);
      } catch {
        setColor(null);
      }
    };
    img.onerror = () => !cancelled && setColor(null);
    img.src = art;
    return () => {
      cancelled = true;
    };
  }, [art]);
  return color;
}

/** Panel de letra: la línea activa se resalta y se mantiene centrada; clic en una línea salta a ella. */
function LyricsPanel({ lyrics, position, canSeek, seek }) {
  const box = useRef(null);
  const active = activeLine(lyrics.lines, position);

  useEffect(() => {
    const el = box.current?.querySelector(`[data-i="${active}"]`);
    if (el && box.current) {
      box.current.scrollTo({ top: el.offsetTop - box.current.clientHeight / 2 + el.clientHeight / 2, behavior: "smooth" });
    }
  }, [active]);

  const note = (text) => <div className="flex h-full items-center justify-center text-center text-xs text-white/35">{text}</div>;
  if (lyrics.status === "loading") return note("Buscando letra…");
  if (lyrics.status === "error") return note("No se pudo buscar la letra. Revisa tu conexión.");
  if (lyrics.status === "none" || lyrics.status === "idle") return note("No se encontró letra para esta canción");
  if (lyrics.status === "plain") {
    return (
      <div className="h-full select-text overflow-y-auto whitespace-pre-wrap pr-2 text-xs leading-relaxed text-white/70">
        <div className="mb-2 text-[10px] uppercase tracking-wider text-white/30">Sin sincronizar</div>
        {lyrics.plain}
      </div>
    );
  }
  return (
    <div
      ref={box}
      className="no-scrollbar relative h-full overflow-y-auto py-16 pr-2 [mask-image:linear-gradient(to_bottom,transparent,black_22%,black_78%,transparent)]"
    >
      {lyrics.lines.map((l, i) => (
        <button
          key={i}
          data-i={i}
          disabled={!canSeek}
          onClick={() => seek(l.t)}
          className={`block w-full py-0.5 text-left leading-snug transition-all duration-300 disabled:cursor-default ${
            i === active ? "text-[15px] font-semibold text-white" : i < active ? "text-xs text-white/30" : "text-xs text-white/45"
          } ${canSeek ? "hover:text-white/80" : ""}`}
        >
          {l.text || "♪"}
        </button>
      ))}
    </div>
  );
}

export default function MediaPlayer({ media, lyricsAvailable = true, visualizer = true }) {
  const { available, track, trackKey, position, playing, can, toggle, next, prev, seek } = media;
  const color = useArtColor(track.art);
  const [lyricsOn, setLyricsOn] = useStore("lyricsOn", false);
  const showLyrics = lyricsAvailable && lyricsOn && available;
  const lyrics = useLyrics(track, trackKey, showLyrics);

  if (!available) {
    return (
      <div className="flex h-full flex-col items-center justify-center gap-2 text-center text-white/40">
        <span className="text-3xl">🎵</span>
        <span className="text-sm">No hay nada reproduciéndose</span>
        <span className="text-xs text-white/30">Reproduce algo en Spotify, YouTube Music, tu navegador o VLC y aparecerá aquí.</span>
      </div>
    );
  }

  const hasTimeline = track.duration > 0;
  const pct = hasTimeline ? Math.min(100, (position / track.duration) * 100) : 0;
  const tint = color ? `rgb(${color})` : "#fff";

  const progress = (
    <div className="flex flex-col gap-1">
      <input
        type="range"
        min={0}
        max={hasTimeline ? track.duration : 1}
        step={0.5}
        value={hasTimeline ? position : 0}
        disabled={!can.seek || !hasTimeline}
        onChange={(e) => seek(Number(e.target.value))}
        className="h-1 w-full cursor-pointer appearance-none rounded-full disabled:cursor-default"
        style={{ accentColor: tint, background: `linear-gradient(to right, ${tint} ${pct}%, rgba(255,255,255,0.2) ${pct}%)` }}
      />
      <div className="flex justify-between text-[11px] tabular-nums text-white/50">
        <span>{fmtTime(Math.floor(position))}</span>
        <span>{hasTimeline ? fmtTime(Math.floor(track.duration)) : "en vivo"}</span>
      </div>
    </div>
  );

  const controls = (
    <div className="flex items-center justify-center gap-2">
      <button className={ctl} onClick={prev} disabled={!can.prev} aria-label="Anterior">⏮</button>
      <button className={`${ctl} bg-white text-black hover:bg-white/90`} onClick={toggle} disabled={!can.playPause} aria-label="Reproducir o pausar">
        {playing ? "⏸" : "▶"}
      </button>
      <button className={ctl} onClick={next} disabled={!can.next} aria-label="Siguiente">⏭</button>
    </div>
  );

  const lyricsButton = lyricsAvailable && (
    <button
      onClick={() => setLyricsOn(!lyricsOn)}
      title={lyricsOn ? "Ocultar letra" : "Mostrar letra"}
      aria-label="Letra"
      className={`absolute right-0 top-0 rounded-full px-2 py-1 text-xs transition-all duration-300 ${lyricsOn ? "bg-white/20" : "opacity-50 hover:bg-white/10 hover:opacity-100"}`}
    >
      🎤
    </button>
  );

  if (showLyrics) {
    return (
      <div className="relative flex h-full gap-4">
        <div className="flex w-52 shrink-0 flex-col justify-between">
          <div className="flex items-center gap-3">
            <div
              className="flex h-14 w-14 shrink-0 items-center justify-center overflow-hidden rounded-2xl bg-gradient-to-br from-fuchsia-500 via-violet-600 to-indigo-700 text-xl"
              style={{ boxShadow: color && playing ? `0 0 22px rgba(${color}, 0.5)` : undefined }}
            >
              {track.art ? <img src={track.art} alt="" className="h-full w-full object-cover" draggable={false} /> : "♪"}
            </div>
            <div className="min-w-0">
              <div className="truncate text-sm font-semibold" title={track.title}>{track.title || "Sin título"}</div>
              <div className="truncate text-xs text-white/55">{track.artist || track.album || "—"}</div>
            </div>
          </div>
          {progress}
          {controls}
        </div>
        <div className="min-w-0 flex-1 pr-7">
          <LyricsPanel lyrics={lyrics} position={position} canSeek={can.seek} seek={seek} />
        </div>
        {lyricsButton}
      </div>
    );
  }

  return (
    <div className="relative flex h-full items-center gap-5">
      <div
        className={`flex h-28 w-28 shrink-0 items-center justify-center overflow-hidden rounded-3xl bg-gradient-to-br from-fuchsia-500 via-violet-600 to-indigo-700 text-4xl transition-all duration-500 ${
          playing ? "scale-100" : "scale-95 opacity-80"
        }`}
        style={{ boxShadow: color && playing ? `0 0 38px rgba(${color}, 0.55)` : "0 8px 20px rgba(0,0,0,0.4)" }}
      >
        {track.art ? <img src={track.art} alt="" className="h-full w-full object-cover" draggable={false} /> : "♪"}
      </div>
      <div className="flex min-w-0 flex-1 flex-col gap-3">
        <div className="min-w-0 pr-8">
          <div className="mb-0.5 flex items-center justify-between gap-3">
            <span className="truncate text-[10px] uppercase tracking-wider text-white/35">{track.source}</span>
            {visualizer && playing && <Visualizer bars={12} className="h-4 w-20 shrink-0" color="bg-white/70" />}
          </div>
          <div className="truncate text-lg font-semibold" title={track.title}>{track.title || "Sin título"}</div>
          <div className="truncate text-sm text-white/60" title={track.artist}>{track.artist || track.album || "—"}</div>
        </div>
        {progress}
        {controls}
      </div>
      {lyricsButton}
    </div>
  );
}
