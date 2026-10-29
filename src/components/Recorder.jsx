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

const pill = "rounded-full bg-white/10 px-3 py-1 text-[11px] transition-all duration-200 hover:bg-white/20 active:scale-95 disabled:opacity-50";

/** Panel derecho: reproductor, nombre, transcripción completa y acciones de una grabación. */
function Detail({ rec, onDeleted }) {
  const audio = useRef(null);
  const [isPlaying, setIsPlaying] = useState(false);
  const [progress, setProgress] = useState(0);
  const [busy, setBusy] = useState(null); // null | número (descarga %) | "work"
  const [error, setError] = useState(null);
  const [confirm, setConfirm] = useState(false);
  const [name, setName] = useState(rec.name);
  const bars = useMemo(() => shape(rec.id, 56), [rec.id]);

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
  const remove = () => {
    if (!confirm) {
      setConfirm(true);
      setTimeout(() => setConfirm(false), 2500);
      return;
    }
    deleteRecording(rec.id).then(onDeleted);
  };

  return (
    <div className="flex h-full min-h-0 flex-col gap-2">
      <input
        value={name}
        onChange={(e) => setName(e.target.value)}
        onBlur={rename}
        onKeyDown={(e) => e.key === "Enter" && e.currentTarget.blur()}
        aria-label="Nombre"
        className="w-full shrink-0 truncate rounded-xl bg-transparent px-1 text-sm font-medium outline-none ring-1 ring-transparent transition focus:bg-white/10 focus:ring-white/20"
      />
      <div className="flex shrink-0 items-center gap-3 rounded-2xl bg-black/20 px-3 py-2">
        <button
          onClick={toggle}
          aria-label={isPlaying ? "Pausar" : "Reproducir"}
          className="flex h-9 w-9 shrink-0 items-center justify-center rounded-full bg-gradient-to-br from-rose-400 to-fuchsia-500 text-sm text-white shadow-lg shadow-rose-500/20 transition-transform duration-200 hover:scale-105 active:scale-90"
        >
          {isPlaying ? "❚❚" : <span className="ml-0.5">▶</span>}
        </button>
        <div onClick={seek} className="flex h-6 min-w-0 flex-1 cursor-pointer items-center gap-[2px]" role="slider" aria-label="Posición" aria-valuenow={Math.round(progress * 100)}>
          {bars.map((b, i) => (
            <span key={i} className={`min-w-0 max-w-[4px] flex-1 rounded-full transition-colors duration-150 ${i / bars.length < progress ? "bg-rose-300" : "bg-white/20"}`} style={{ height: `${b * 100}%` }} />
          ))}
        </div>
        <span className="shrink-0 text-[10px] tabular-nums text-white/50">
          {fmtDur(isPlaying || progress ? progress * rec.duration : 0)} / {fmtDur(rec.duration)}
        </span>
      </div>
      <div className="flex shrink-0 items-center gap-1.5">
        <button onClick={run} disabled={busy !== null} className={pill}>
          {busy === null ? (rec.transcript ? "↻ Transcribir de nuevo" : "✨ Transcribir") : busy === "work" ? "Transcribiendo…" : `${isEn() ? "Downloading model" : "Descargando modelo"} ${busy}%`}
        </button>
        {rec.transcript && (
          <button onClick={() => navigator.clipboard.writeText(rec.transcript)} className={pill}>
            ⧉ Copiar
          </button>
        )}
        <button onClick={remove} className={`${pill} ml-auto ${confirm ? "!bg-red-500/30 text-red-300" : ""}`}>
          {confirm ? "¿Seguro?" : "🗑 Borrar"}
        </button>
      </div>
      {typeof busy === "number" && (
        <div className="h-1 shrink-0 overflow-hidden rounded-full bg-white/10">
          <div className="h-full rounded-full bg-gradient-to-r from-rose-400 to-fuchsia-500 transition-all" style={{ width: `${busy}%` }} />
        </div>
      )}
      {busy === 0 && <div className="shrink-0 text-[10px] text-white/35">La primera vez se descarga el modelo (~40 MB); después funciona sin internet.</div>}
      {error && <div className="shrink-0 text-[10px] text-amber-300">{error}</div>}
      <div className="min-h-0 flex-1 select-text overflow-y-auto whitespace-pre-wrap rounded-2xl bg-white/5 p-3 text-xs leading-relaxed text-white/85">
        {rec.transcript ?? <span className="text-white/30">Aún sin transcribir. Pulsa «Transcribir» para convertir el audio en texto, en tu equipo.</span>}
      </div>
    </div>
  );
}

/** Mientras se graba: cronómetro grande y onda en vivo. */
function Live({ secs, levels }) {
  return (
    <div className="flex h-full flex-col items-center justify-center gap-4">
      <span className="text-4xl font-semibold tabular-nums text-rose-200">{fmtDur(secs)}</span>
      <div className="flex h-12 w-full items-center justify-center gap-[3px]" aria-hidden="true">
        {levels.map((l, i) => (
          <span key={i} className="w-[4px] shrink-0 rounded-full bg-rose-300 transition-[height] duration-100" style={{ height: `${Math.max(10, l * 100)}%`, opacity: 0.35 + (i / levels.length) * 0.65 }} />
        ))}
      </div>
      <span className="text-[11px] text-white/40">Grabando… pulsa Detener para guardar</span>
    </div>
  );
}

/** Grabadora: lista de audios a la izquierda y, al tocar uno, su reproductor y texto a la derecha (como en Notas). */
export default function Recorder() {
  const items = useRecordings();
  const [selected, setSelected] = useState(null);
  const { state, secs, levels, error, start, stop } = useRecorder(async ({ blob, duration }) => {
    const created = Date.now();
    const name = `${isEn() ? "Recording" : "Grabación"} ${new Date(created).toLocaleString(locale(), { day: "numeric", month: "short", hour: "2-digit", minute: "2-digit" })}`;
    await saveRecording({ id: created, name, created, duration, blob, transcript: null });
    setSelected(created);
  });
  const recording = state === "recording";
  const current = items.find((r) => r.id === selected) ?? null;

  return (
    <div className="flex h-full gap-2 rounded-3xl bg-white/5 p-3">
      <div className="flex w-40 shrink-0 flex-col gap-1">
        <button
          onClick={recording ? stop : start}
          className={`flex items-center justify-center gap-2 rounded-full py-1.5 text-xs font-medium text-white transition-all duration-300 active:scale-95 ${
            recording ? "bg-rose-500" : "bg-gradient-to-r from-rose-500 to-fuchsia-500 hover:brightness-110"
          }`}
        >
          <span className={`relative h-2.5 w-2.5 bg-white ${recording ? "rounded-[2px]" : "rounded-full"}`}>
            {recording && <span className="absolute inset-0 animate-ping rounded-full bg-white/70" />}
          </span>
          {recording ? `${isEn() ? "Stop" : "Detener"} · ${fmtDur(secs)}` : "Grabar"}
        </button>
        {error && <div className="rounded-xl bg-amber-500/10 px-2 py-1 text-[10px] leading-snug text-amber-300">{error}</div>}
        <ul className="flex min-h-0 flex-1 flex-col gap-0.5 overflow-y-auto pr-1">
          {items.length === 0 && <li className="py-4 text-center text-[11px] text-white/30">Sin grabaciones</li>}
          {items.map((r) => (
            <li key={r.id}>
              <button
                onClick={() => setSelected(r.id)}
                className={`flex w-full items-center gap-2 rounded-xl px-2 py-1 text-left transition-colors ${r.id === selected && !recording ? "bg-white/15" : "text-white/60 hover:bg-white/5"}`}
              >
                <span className="shrink-0 text-sm">{r.transcript ? "📝" : "🎙"}</span>
                <span className="min-w-0 flex-1">
                  <span className="block truncate text-xs">{r.name}</span>
                  <span className="block text-[10px] tabular-nums text-white/35">
                    {new Date(r.created).toLocaleDateString(locale(), { day: "numeric", month: "short" })} · {fmtDur(r.duration)}
                  </span>
                </span>
              </button>
            </li>
          ))}
        </ul>
      </div>
      <div className="min-w-0 flex-1">
        {recording ? (
          <Live secs={secs} levels={levels} />
        ) : current ? (
          <Detail key={current.id} rec={current} onDeleted={() => setSelected(null)} />
        ) : (
          <div className="flex h-full flex-col items-center justify-center gap-1 text-white/30">
            <span className="text-2xl opacity-60">🎙</span>
            <span className="text-xs">{items.length ? "Elige una grabación" : "Pulsa Grabar para empezar"}</span>
          </div>
        )}
      </div>
    </div>
  );
}
