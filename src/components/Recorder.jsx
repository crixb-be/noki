import { useEffect, useMemo, useRef, useState } from "react";
import { deleteRecording, getRecording, saveRecording, useRecordings } from "../lib/recordings.js";
import { transcribe } from "../lib/transcribe.js";
import { isEn, locale } from "../lib/i18n.js";

const fmtDur = (s) => `${Math.floor(s / 60)}:${String(Math.floor(s % 60)).padStart(2, "0")}`;

/** Mejor formato que soporte el WebView. */
const mimeType = () => ["audio/webm;codecs=opus", "audio/webm", "audio/mp4"].find((m) => MediaRecorder.isTypeSupported(m)) ?? "";

/** Controla la grabación del micrófono. Al detener, entrega el audio ya guardado. */
function useRecorder(onSaved) {
  const [state, setState] = useState("idle"); // idle | recording
  const [secs, setSecs] = useState(0);
  const [level, setLevel] = useState(0);
  const [error, setError] = useState(null);
  const ref = useRef({});

  const cleanup = () => {
    const r = ref.current;
    clearInterval(r.timer);
    cancelAnimationFrame(r.raf);
    r.stream?.getTracks().forEach((t) => t.stop());
    r.ctx?.close().catch(() => {});
    ref.current = {};
    setLevel(0);
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
        setLevel(Math.min(1, peak / 80));
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
  return { state, secs, level, error, start, stop };
}

function Item({ rec, open, onToggle }) {
  const [url, setUrl] = useState(null);
  const [busy, setBusy] = useState(null); // null | número (descarga %) | "work"
  const [error, setError] = useState(null);
  const [confirm, setConfirm] = useState(false);
  const [name, setName] = useState(rec.name);

  useEffect(() => {
    if (!open) return;
    let u;
    let off = false;
    getRecording(rec.id).then((r) => {
      if (off || !r) return;
      u = URL.createObjectURL(r.blob);
      setUrl(u);
    });
    return () => {
      off = true;
      u && URL.revokeObjectURL(u);
      setUrl(null);
    };
  }, [open, rec.id]);

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

  return (
    <li className="rounded-2xl bg-white/5">
      <button onClick={onToggle} className="flex w-full items-center gap-2 px-3 py-1.5 text-left text-xs">
        <span>{rec.transcript ? "📝" : "🎙"}</span>
        <span className="min-w-0 flex-1 truncate">{rec.name}</span>
        <span className="shrink-0 tabular-nums text-[10px] text-white/40">
          {new Date(rec.created).toLocaleDateString(locale(), { day: "numeric", month: "short" })} · {fmtDur(rec.duration)}
        </span>
      </button>
      {open && (
        <div className="flex flex-col gap-1.5 px-3 pb-2">
          {url && <audio src={url} controls className="h-8 w-full" />}
          <div className="flex items-center gap-1.5">
            <input
              value={name}
              onChange={(e) => setName(e.target.value)}
              onBlur={rename}
              onKeyDown={(e) => e.key === "Enter" && e.currentTarget.blur()}
              className="min-w-0 flex-1 rounded-full bg-white/10 px-3 py-1 text-[11px] outline-none focus:bg-white/15"
            />
            <button onClick={run} disabled={busy !== null} className="shrink-0 rounded-full bg-white/10 px-3 py-1 text-[11px] hover:bg-white/20 disabled:opacity-50">
              {busy === null ? (rec.transcript ? "↻ Transcribir de nuevo" : "📝 Transcribir") : busy === "work" ? "Transcribiendo…" : `${isEn() ? "Downloading model" : "Descargando modelo"} ${busy}%`}
            </button>
            <button onClick={remove} className={`shrink-0 rounded-full px-2 py-1 text-[11px] ${confirm ? "bg-red-500/30 text-red-300" : "bg-white/10 hover:bg-white/20"}`} aria-label="Borrar">
              {confirm ? "¿Seguro?" : "🗑"}
            </button>
          </div>
          {busy === 0 && <div className="text-[10px] text-white/35">La primera vez se descarga el modelo (~40 MB); después funciona sin internet.</div>}
          {error && <div className="text-[10px] text-amber-300">{error}</div>}
          {rec.transcript && (
            <div className="relative">
              <p className="max-h-24 select-text overflow-y-auto rounded-xl bg-white/5 p-2 pr-8 text-[11px] leading-relaxed text-white/80">{rec.transcript}</p>
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
  const { state, secs, level, error, start, stop } = useRecorder(async ({ blob, duration }) => {
    const created = Date.now();
    const name = `${isEn() ? "Recording" : "Grabación"} ${new Date(created).toLocaleString(locale(), { day: "numeric", month: "short", hour: "2-digit", minute: "2-digit" })}`;
    await saveRecording({ id: created, name, created, duration, blob, transcript: null });
    setOpenId(created);
  });
  const recording = state === "recording";
  const bars = useMemo(() => [0.5, 0.8, 1, 0.8, 0.5], []);

  return (
    <div className="flex h-full flex-col gap-2 rounded-3xl bg-white/5 p-3">
      <div className="flex shrink-0 items-center gap-3">
        <button
          onClick={recording ? stop : start}
          className={`flex h-10 shrink-0 items-center gap-2 rounded-full px-4 text-xs font-medium transition-all duration-300 active:scale-95 ${
            recording ? "bg-rose-500 text-white" : "bg-white/10 hover:bg-white/20"
          }`}
        >
          <span className={`h-2.5 w-2.5 ${recording ? "rounded-sm bg-white" : "rounded-full bg-rose-400"}`} />
          {recording ? "Detener" : "Grabar"}
        </button>
        {recording ? (
          <>
            <span className="text-sm tabular-nums text-rose-300">{fmtDur(secs)}</span>
            <span className="flex h-6 items-center gap-[3px]" aria-hidden="true">
              {bars.map((b, i) => (
                <span key={i} className="w-[3px] rounded-full bg-rose-300 transition-[height] duration-100" style={{ height: `${Math.max(14, level * b * 100)}%` }} />
              ))}
            </span>
          </>
        ) : (
          <span className="text-[11px] text-white/35">Tus audios se guardan solo en este equipo.</span>
        )}
      </div>
      {error && <div className="shrink-0 rounded-xl bg-amber-500/10 px-2 py-1 text-[10px] text-amber-300">{error}</div>}
      <ul className="flex min-h-0 flex-1 flex-col gap-1 overflow-y-auto pr-1">
        {items.length === 0 && <li className="py-6 text-center text-[11px] text-white/30">Sin grabaciones</li>}
        {items.map((r) => (
          <Item key={r.id} rec={r} open={openId === r.id} onToggle={() => setOpenId(openId === r.id ? null : r.id)} />
        ))}
      </ul>
    </div>
  );
}
