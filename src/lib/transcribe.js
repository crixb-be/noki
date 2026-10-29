let worker = null;
let seq = 0;
const pending = new Map();

function getWorker() {
  if (worker) return worker;
  worker = new Worker(new URL("./whisperWorker.js", import.meta.url), { type: "module" });
  worker.onmessage = ({ data }) => {
    const p = pending.get(data.id);
    if (data.type === "progress") return [...pending.values()].forEach((q) => q.onProgress?.(data.progress));
    if (!p) return;
    if (data.type === "working") p.onProgress?.(null);
    else {
      pending.delete(data.id);
      data.type === "done" ? p.resolve(data.text) : p.reject(new Error(data.error));
    }
  };
  worker.onerror = (e) => {
    pending.forEach((p) => p.reject(new Error(e.message || "No se pudo iniciar la transcripción")));
    pending.clear();
    worker = null;
  };
  return worker;
}

/** Whisper pide mono a 16 kHz. */
async function decode(blob) {
  const ctx = new AudioContext({ sampleRate: 16000 });
  try {
    const buf = await ctx.decodeAudioData(await blob.arrayBuffer());
    return buf.getChannelData(0);
  } finally {
    ctx.close();
  }
}

/** Transcribe un audio grabado. `onProgress(pct)` informa la descarga del modelo (solo la primera vez); null = transcribiendo. */
export async function transcribe(blob, language, onProgress) {
  const audio = await decode(blob);
  const id = ++seq;
  return new Promise((resolve, reject) => {
    pending.set(id, { resolve, reject, onProgress });
    getWorker().postMessage({ id, audio, language }, [audio.buffer]);
  });
}
