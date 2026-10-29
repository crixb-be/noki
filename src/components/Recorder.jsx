import { useEffect, useMemo, useRef, useState } from "react";
import { deleteRecording, getRecording, saveRecording, useRecordings } from "../lib/recordings.js";
import { transcribe } from "../lib/transcribe.js";
import { isEn, locale } from "../lib/i18n.js";

const BARS = 28;
const fmtDur = (s) => `${Math.floor(s / 60)}:${String(Math.floor(s % 60)).padStart(2, "0")}`;

/** Mejor formato que soporte el WebView. */
const mimeType = () => ["audio/webm;codecs=opus", "audio/webm", "audio/mp4"].find((m) => MediaRecorder.isTypeSupported(m)) ?? "";

/** Controla la grabación del micrófono. Al detener, entrega el audio ya guardado. */
function useRecorder(onSaved) {
  const [state, setState] = useState("idle"); // idle | recording
  const [secs, setSecs] = useState(0);
  const [levels, setLevels] = useState(() => Array(BARS).fill(0));
  const [error, setError] = useState(null);
  const ref = useRef({});

  const cleanup = () => {
    const r = ref.current;
    clearInterval(r.timer);
    cancelAnimationFrame(r.raf);
    r.stream?.getTracks().forEach((t) => t.stop());
    r.ctx?.close().catch(() => {});
    ref.current = {};
    setLevels(Array(BARS).fill(0));
  };
  useEffect(() => cleanup, []);

  const start = async () => {
    setError(null);
    try {
      const stream = await navigator.mediaDevices.getUserMedia({ audio: true });
      const rec = new MediaRecorder(stream, mimeType() ? { mimeType: mimeType() } : undefined);
      const chunks = [];
      rec.ondataavailable = (e) => e.data.size && chunks.push(e.data);
      const began = Date.now();
      rec.onstop = async () => {
        const duration = (Date.now() - began) / 1000;
        const blob = new Blob(chunks, { type: rec.mimeType });
        cleanup();
        setState("idle");
        if (blob.size) await onSaved({ blob, duration });
      };
      // Nivel de voz para el medidor.
      const ctx = new AudioContext();
      const analyser = ctx.createAnalyser();
      analyser.fftSize = 512;
      ctx.createMediaStreamSource(stream).connect(analyser);
      const data = new Uint8Array(analyser.fftSize);
      const tick = () => {
        analyser.getByteTimeDomainData(data);
        let peak = 0;
        for (const v of data) peak = Math.max(peak, Math.abs(v - 128));
        const now = performance.now();
        if (now - (ref.current.last ?? 0) > 70) {
          ref.current.last = now;
          const lv = Math.min(1, peak / 70);
          setLevels((l) => [...l.slice(1), lv]);
        }
        ref.current.raf = requestAnimationFrame(tick);
      };
      ref.current = { stream, rec, ctx, raf: requestAnimationFrame(tick), timer: setInterval(() => setSecs((Date.now() - began) / 1000), 250) };
      setSecs(0);
      rec.start();
      setState("recording");
    } catch (e) {
      cleanup();
      setError(e?.name === "NotAllowedError" ? "Noki no tiene permiso para usar el micrófono." : e?.name === "NotFoundError" ? "No se encontró ningún micrófono." : String(e?.message ?? e));
    }
  };
  const stop = () => ref.current.rec?.state === "recording" && ref.current.rec.stop();
  return { state, secs, levels, error, start, stop };
}

/** Barras decorativas, siempre iguales para cada grabación. */
const shape = (id, n) => {
  let x = Number(id) % 2147483647 || 7;
  return Array.from({ length: n }, () => {
    x = (x * 48271) % 2147483647;
    return 0.25 + 0.75 * Math.abs(Math.sin(x / 9e5));
  });
};

let playing = null; // una sola grabación suena a la vez

function Item({ rec, open, onToggle }) {
  const audio = useRef(null);
  const [isPlaying, setIsPlaying] = useState(false);
  const [progress, setProgress] = useState(0);
  const [busy, setBusy] = useState(null); // null | número (descarga %) | "work"
  const [error, setError] = useState(null);
  const [confirm, setConfirm] = useState(false);
  const [name, setName] = useState(rec.name);
  const bars = useMemo(() => shape(rec.id, 48), [rec.id]);

  useEffect(
    () => () => {
      const a = audio.current;
      if (!a) return;
      a.pause();
      URL.revokeObjectURL(a.src);
      if (playing === a) playing = null;
    },
    []
  );

  const ensureAudio = async () => {
    if (audio.current) return audio.current;
    const full = await getRecording(rec.id);
    const a = new Audio(URL.createObjectURL(full.blob));
    a.ontimeupdate = () => setProgress(a.duration && isFinite(a.duration) ? a.currentTime / a.duration : a.currentTime / rec.duration);
    a.onplay = () => setIsPlaying(true);
    a.onpause = () => setIsPlaying(false);
    a.onended = () => {
      setIsPlaying(false);
      setProgress(0);
    };
    audio.current = a;
    return a;
  };
  const toggle = async () => {
    const a = await ensureAudio();
    if (a.paused) {
      if (playing && playing !== a) playing.pause();
      playing = a;
      a.play().catch(() => {});
    } else a.pause();
  };
  const seek = async (e) => {
    const a = await ensureAudio();
    const r = e.currentTarget.getBoundingClientRect();
    const f = Math.min(1, Math.max(0, (e.clientX - r.left) / r.width));
    a.currentTime = f * (isFinite(a.duration) ? a.duration : rec.duration);
    setProgress(f);
  };

  const run = async () => {
    setError(null);
    setBusy(0);
    try {
      const full = await getRecording(rec.id);
      const text = await transcribe(full.blob, isEn() ? "english" : "spanish", (p) => setBusy(p == null ? "work" : Math.round(p)));
      await saveRecording({ ...full, transcript: text || "(sin texto)" });
    } catch (e) {
      setError(String(e?.message ?? e));
    }
    setBusy(null);
  };
  const rename = () => name.trim() && name.trim() !== rec.name && getRecording(rec.id).then((r) => saveRecording({ ...r, name: name.trim() }));
  const remove = () => (confirm ? deleteRecording(rec.id) : (setConfirm(true), setTimeout(() => setConfirm(false), 2500)));

  const pill = "rounded-full bg-white/10 px-3 py-1 text-[11px] transition-all duration-200 hover:bg-white/20 active:scale-95 disabled:opacity-50";

  return (
    <li className={`overflow-hidden rounded-2xl ring-1 transition-colors duration-300 ${open ? "bg-white/[0.09] ring-white/15" : "bg-white/[0.05] ring-white/[0.06] hover:bg-white/[0.08]"}`}>
      <div className="flex items-center gap-3 px-3 py-2">
        <button
          onClick={toggle}
          aria-label={isPlaying ? "Pausar" : "Reproducir"}
          className="flex h-9 w-9 shrink-0 items-center justify-center rounded-full bg-gradient-to-br from-rose-400 to-fuchsia-500 text-sm text-white shadow-lg shadow-rose-500/20 transition-transform duration-200 hover:scale-105 active:scale-90"
        >
          {isPlaying ? "❚❚" : <span className="ml-0.5">▶</span>}
        </button>
        <div className="min-w-0 flex-1">
          <div className="flex items-center gap-2">
            <span className="min-w-0 flex-1 truncate text-xs font-medium">{rec.name}</span>
            {rec.transcript && <span className="shrink-0 rounded-full bg-emerald-400/15 px-1.5 py-px text-[9px] font-medium text-emerald-300">Transcrita</span>}
          </div>
          <div className="mt-1 flex items-center gap-2">
            <div onClick={seek} className="flex h-5 min-w-0 flex-1 cursor-pointer items-center gap-[2px]" role="slider" aria-label="Posición" aria-valuenow={Math.round(progress * 100)}>
              {bars.map((b, i) => (
                <span
                  key={i}
                  className={`max-w-[4px] min-w-0 flex-1 rounded-full transition-colors duration-150 ${i / bars.length < progress ? "bg-rose-300" : "bg-white/20"}`}
                  style={{ height: `${b * 100}%` }}
                />
              ))}
            </div>
            <span className="w-8 shrink-0 text-right text-[10px] tabular-nums text-white/50">{fmtDur(isPlaying || progress ? progress * rec.duration : rec.duration)}</span>
          </div>
        </div>
        <div className="flex shrink-0 flex-col items-end gap-1">
          <span className="text-[10px] text-white/35">{new Date(rec.created).toLocaleDateString(locale(), { day: "numeric", month: "short" })}</span>
          <button onClick={onToggle} aria-label="Detalles" className={`text-[10px] text-white/40 transition-transform duration-300 hover:text-white ${open ? "rotate-180" : ""}`}>▼</button>
        </div>
      </div>
      {open && (
        <div className="flex animate-[tabIn_220ms_ease-out_both] flex-col gap-2 border-t border-white/5 px-3 pb-3 pt-2">
          <div className="flex items-center gap-1.5">
            <input
              value={name}
              onChange={(e) => setName(e.target.value)}
              onBlur={rename}
              onKeyDown={(e) => e.key === "Enter" && e.currentTarget.blur()}
              aria-label="Nombre"
              className="min-w-0 flex-1 rounded-full bg-black/25 px-3 py-1 text-[11px] outline-none ring-1 ring-white/10 focus:ring-white/30"
            />
            <button onClick={run} disabled={busy !== null} className={`${pill} shrink-0`}>
              {busy === null ? (rec.transcript ? "↻ Transcribir de nuevo" : "✨ Transcribir") : busy === "work" ? "Transcribiendo…" : `${isEn() ? "Downloading model" : "Descargando modelo"} ${busy}%`}
            </button>
            <button onClick={remove} className={`${pill} shrink-0 ${confirm ? "!bg-red-500/30 text-red-300" : ""}`} aria-label="Borrar">
              {confirm ? "¿Seguro?" : "🗑"}
            </button>
          </div>
          {typeof busy === "number" && <div className="h-1 overflow-hidden rounded-full bg-white/10"><div className="h-full rounded-full bg-gradient-to-r from-rose-400 to-fuchsia-500 transition-all" style={{ width: `${busy}%` }} /></div>}
          {busy === 0 && <div className="text-[10px] text-white/35">La primera vez se descarga el modelo (~40 MB); después funciona sin internet.</div>}
          {error && <div className="text-[10px] text-amber-300">{error}</div>}
          {rec.transcript && (
            <div className="relative">
              <p className="max-h-24 select-text overflow-y-auto rounded-xl bg-black/25 p-2.5 pr-9 text-[11px] leading-relaxed text-white/85">{rec.transcript}</p>
              <button onClick={() => navigator.clipboard.writeText(rec.transcript)} title="Copiar" aria-label="Copiar" className="absolute right-1.5 top-1.5 rounded-full bg-white/10 px-1.5 text-[11px] hover:bg-white/20">
                ⧉
              </button>
            </div>
          )}
        </div>
      )}
    </li>
  );
}

/** Grabadora: graba la voz, guarda los audios en el equipo y, si quieres, los transcribe en local (Whisper). */
export default function Recorder() {
  const items = useRecordings();
  const [openId, setOpenId] = useState(null);
  const { state, secs, levels, error, start, stop } = useRecorder(async ({ blob, duration }) => {
    const created = Date.now();
    const name = `${isEn() ? "Recording" : "Grabación"} ${new Date(created).toLocaleString(locale(), { day: "numeric", month: "short", hour: "2-digit", minute: "2-digit" })}`;
    await saveRecording({ id: created, name, created, duration, blob, transcript: null });
    setOpenId(created);
  });
  const recording = state === "recording";
  const total = items.reduce((t, r) => t + r.duration, 0);

  return (
    <div className="flex h-full flex-col gap-2">
      <div className={`flex shrink-0 items-center gap-3 rounded-3xl px-3 py-2.5 ring-1 transition-colors duration-500 ${recording ? "bg-gradient-to-r from-rose-500/25 to-fuchsia-500/15 ring-rose-400/30" : "bg-white/[0.06] ring-white/[0.06]"}`}>
        <button
          onClick={recording ? stop : start}
          aria-label={recording ? "Detener" : "Grabar"}
          className="relative flex h-11 w-11 shrink-0 items-center justify-center rounded-full bg-gradient-to-br from-rose-400 to-rose-600 shadow-lg shadow-rose-500/30 transition-transform duration-200 hover:scale-105 active:scale-90"
        >
          {recording && <span className="absolute inset-0 animate-ping rounded-full bg-rose-400/50" />}
          <span className={`relative bg-white transition-all duration-300 ${recording ? "h-3.5 w-3.5 rounded-[3px]" : "h-3.5 w-3.5 rounded-full"}`} />
        </button>
        {recording ? (
          <>
            <span className="w-12 shrink-0 text-lg font-semibold tabular-nums text-rose-200">{fmtDur(secs)}</span>
            <div className="flex h-8 min-w-0 flex-1 items-center justify-end gap-[3px]" aria-hidden="true">
              {levels.map((l, i) => (
                <span key={i} className="w-[3px] shrink-0 rounded-full bg-rose-300 transition-[height] duration-100" style={{ height: `${Math.max(12, l * 100)}%`, opacity: 0.35 + (i / levels.length) * 0.65 }} />
              ))}
            </div>
          </>
        ) : (
          <div className="min-w-0 flex-1">
            <div className="text-xs font-medium">Grabar un audio</div>
            <div className="truncate text-[10px] text-white/40">
              {items.length ? `${items.length} ${isEn() ? (items.length === 1 ? "recording" : "recordings") : items.length === 1 ? "grabación" : "grabaciones"} · ${fmtDur(total)} · ` : ""}Tus audios se guardan solo en este equipo.
            </div>
          </div>
        )}
      </div>
      {error && <div className="shrink-0 rounded-xl bg-amber-500/10 px-3 py-1.5 text-[10px] text-amber-300">{error}</div>}
      <ul className="flex min-h-0 flex-1 flex-col gap-1.5 overflow-y-auto pr-1">
        {items.length === 0 && (
          <li className="flex flex-1 flex-col items-center justify-center gap-1 text-white/30">
            <span className="text-2xl opacity-60">🎙</span>
            <span className="text-[11px]">Sin grabaciones</span>
          </li>
        )}
        {items.map((r) => (
          <Item key={r.id} rec={r} open={openId === r.id} onToggle={() => setOpenId(openId === r.id ? null : r.id)} />
        ))}
      </ul>
    </div>
  );
}
