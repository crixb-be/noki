import { useSyncExternalStore } from "react";
import { saveRecording } from "./recordings.js";
import { isEn, locale } from "./i18n.js";

/**
 * Grabadora de voz compartida: vive fuera de los componentes para que siga grabando aunque se cierre el notch
 * y para poder iniciarla desde un atajo global.
 */
export const BARS = 28;
export const MIC_KEY = "noki.micId";

let snap = { recording: false, secs: 0, levels: Array(BARS).fill(0), error: null, lastSaved: null };
const subs = new Set();
const set = (patch) => {
  snap = { ...snap, ...patch };
  subs.forEach((f) => f());
};
let run = {};

const mimeType = () => ["audio/webm;codecs=opus", "audio/webm", "audio/mp4"].find((m) => MediaRecorder.isTypeSupported(m)) ?? "";
const savedMic = () => {
  try {
    return localStorage.getItem(MIC_KEY) || "";
  } catch {
    return "";
  }
};

function cleanup() {
  clearInterval(run.timer);
  cancelAnimationFrame(run.raf);
  run.stream?.getTracks().forEach((t) => t.stop());
  run.ctx?.close().catch(() => {});
  run = {};
}

async function openStream() {
  const id = savedMic();
  if (id) {
    try {
      return await navigator.mediaDevices.getUserMedia({ audio: { deviceId: { exact: id } } });
    } catch {
      /* el micrófono guardado ya no está: se usa el predeterminado */
    }
  }
  return navigator.mediaDevices.getUserMedia({ audio: true });
}

export async function startRecording() {
  if (snap.recording) return;
  set({ error: null });
  try {
    const stream = await openStream();
    const rec = new MediaRecorder(stream, mimeType() ? { mimeType: mimeType() } : undefined);
    const chunks = [];
    rec.ondataavailable = (e) => e.data.size && chunks.push(e.data);
    const began = Date.now();
    rec.onstop = async () => {
      const duration = (Date.now() - began) / 1000;
      const blob = new Blob(chunks, { type: rec.mimeType });
      cleanup();
      set({ recording: false, levels: Array(BARS).fill(0) });
      if (!blob.size) return;
      const created = Date.now();
      const name = `${isEn() ? "Recording" : "Grabación"} ${new Date(created).toLocaleString(locale(), { day: "numeric", month: "short", hour: "2-digit", minute: "2-digit" })}`;
      await saveRecording({ id: created, name, created, duration, blob, transcript: null });
      set({ lastSaved: created });
    };
    // Nivel de voz para la onda.
    const ctx = new AudioContext();
    const analyser = ctx.createAnalyser();
    analyser.fftSize = 512;
    ctx.createMediaStreamSource(stream).connect(analyser);
    const data = new Uint8Array(analyser.fftSize);
    let last = 0;
    const tick = () => {
      analyser.getByteTimeDomainData(data);
      let peak = 0;
      for (const v of data) peak = Math.max(peak, Math.abs(v - 128));
      const now = performance.now();
      if (now - last > 70) {
        last = now;
        set({ levels: [...snap.levels.slice(1), Math.min(1, peak / 70)] });
      }
      run.raf = requestAnimationFrame(tick);
    };
    run = { stream, rec, ctx, raf: requestAnimationFrame(tick), timer: setInterval(() => set({ secs: (Date.now() - began) / 1000 }), 250) };
    rec.start();
    set({ recording: true, secs: 0 });
  } catch (e) {
    cleanup();
    set({
      recording: false,
      error: e?.name === "NotAllowedError" ? "Noki no tiene permiso para usar el micrófono." : e?.name === "NotFoundError" ? "No se encontró ningún micrófono." : String(e?.message ?? e),
    });
  }
}

export function stopRecording() {
  if (run.rec?.state === "recording") run.rec.stop();
}

export const isRecording = () => snap.recording;
export const toggleRecording = () => (snap.recording ? stopRecording() : startRecording());

export function useRecorder() {
  return useSyncExternalStore(
    (f) => (subs.add(f), () => subs.delete(f)),
    () => snap
  );
}

/** Micrófonos disponibles (las etiquetas solo aparecen cuando ya hubo permiso). */
export async function listMics() {
  try {
    return (await navigator.mediaDevices.enumerateDevices()).filter((d) => d.kind === "audioinput" && d.deviceId);
  } catch {
    return [];
  }
}
