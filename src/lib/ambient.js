import { useSyncExternalStore } from "react";

/**
 * Sonidos ambientales para concentrarse, generados al vuelo con WebAudio (sin archivos de audio):
 * ruido blanco, lluvia y café. Un único motor por ventana; la UI lo lee con `useAmbient()`.
 */
export const SOUNDS = [
  { id: "white", label: "Ruido blanco", icon: "🌫️" },
  { id: "rain", label: "Lluvia", icon: "🌧️" },
  { id: "cafe", label: "Café", icon: "☕" },
];

let ctx = null;
let master = null;
let active = null; // { id, stop() }
let state = { kind: null, volume: 0.5 };
const subs = new Set();
const emit = (patch) => {
  state = { ...state, ...patch };
  subs.forEach((f) => f());
};

function noiseBuffer(c, kind = "white", seconds = 6) {
  const buf = c.createBuffer(1, c.sampleRate * seconds, c.sampleRate);
  const d = buf.getChannelData(0);
  let last = 0;
  for (let i = 0; i < d.length; i++) {
    const w = Math.random() * 2 - 1;
    if (kind === "brown") {
      last = (last + 0.02 * w) / 1.02;
      d[i] = last * 3.5;
    } else d[i] = w;
  }
  // Empalme suave para que el bucle no «clicke».
  const fade = Math.floor(c.sampleRate * 0.05);
  for (let i = 0; i < fade; i++) d[d.length - 1 - i] *= i / fade;
  return buf;
}

const loop = (c, kind) => {
  const s = c.createBufferSource();
  s.buffer = noiseBuffer(c, kind);
  s.loop = true;
  s.start();
  return s;
};

function build(c, out, id) {
  const nodes = [];
  const timers = [];
  const gain = (v) => {
    const g = c.createGain();
    g.gain.value = v;
    nodes.push(g);
    return g;
  };
  const filter = (type, f, q = 0.7) => {
    const n = c.createBiquadFilter();
    n.type = type;
    n.frequency.value = f;
    n.Q.value = q;
    nodes.push(n);
    return n;
  };
  const blip = (freq, vol, dur) => {
    const o = c.createOscillator();
    const g = c.createGain();
    o.frequency.value = freq;
    o.type = "sine";
    const t = c.currentTime;
    g.gain.setValueAtTime(0, t);
    g.gain.linearRampToValueAtTime(vol, t + 0.004);
    g.gain.exponentialRampToValueAtTime(0.0001, t + dur);
    o.connect(g).connect(out);
    o.start(t);
    o.stop(t + dur + 0.05);
  };

  if (id === "white") {
    const n = loop(c, "white");
    const lp = filter("lowpass", 9000);
    const g = gain(0.35);
    n.connect(lp).connect(g).connect(out);
    nodes.push(n);
  } else if (id === "rain") {
    const n = loop(c, "white");
    const hp = filter("highpass", 700);
    const lp = filter("lowpass", 7000);
    const g = gain(0.4);
    n.connect(hp).connect(lp).connect(g).connect(out);
    nodes.push(n);
    // Gotas sueltas
    timers.push(setInterval(() => Math.random() < 0.7 && blip(1800 + Math.random() * 2600, 0.02 + Math.random() * 0.03, 0.04), 90));
  } else {
    // Café: rumor grave de sala + murmullo que sube y baja + algún tintineo de tazas.
    const room = loop(c, "brown");
    const lpRoom = filter("lowpass", 700);
    const gRoom = gain(0.55);
    room.connect(lpRoom).connect(gRoom).connect(out);
    const talk = loop(c, "white");
    const bp = filter("bandpass", 1100, 0.6);
    const gTalk = gain(0.12);
    const lfo = c.createOscillator();
    const lfoG = c.createGain();
    lfo.frequency.value = 0.23;
    lfoG.gain.value = 0.07;
    lfo.connect(lfoG).connect(gTalk.gain);
    lfo.start();
    talk.connect(bp).connect(gTalk).connect(out);
    nodes.push(room, talk, lfo);
    timers.push(setInterval(() => Math.random() < 0.18 && blip(2400 + Math.random() * 1800, 0.035, 0.22), 700));
  }
  return () => {
    timers.forEach(clearInterval);
    nodes.forEach((n) => {
      try { n.stop?.(); } catch { /* ya parado */ }
      try { n.disconnect(); } catch { /* ya desconectado */ }
    });
  };
}

export function stopAmbient() {
  if (!active) return;
  const { stop } = active;
  active = null;
  if (master && ctx) {
    const t = ctx.currentTime;
    master.gain.cancelScheduledValues(t);
    master.gain.setValueAtTime(master.gain.value, t);
    master.gain.linearRampToValueAtTime(0, t + 0.4);
  }
  setTimeout(stop, 450);
  emit({ kind: null });
}

export function playAmbient(id) {
  if (active?.id === id) return stopAmbient();
  const prev = active;
  active = null;
  prev?.stop();
  try {
    ctx ??= new (window.AudioContext || window.webkitAudioContext)();
    if (ctx.state === "suspended") ctx.resume();
    master = ctx.createGain();
    master.gain.value = 0;
    master.connect(ctx.destination);
    master.gain.linearRampToValueAtTime(state.volume, ctx.currentTime + 0.6);
    active = { id, stop: build(ctx, master, id) };
    emit({ kind: id });
  } catch {
    emit({ kind: null });
  }
}

export function setAmbientVolume(v) {
  if (master && ctx) master.gain.setTargetAtTime(v, ctx.currentTime, 0.05);
  emit({ volume: v });
}

/** Siguiente sonido del ciclo blanco → lluvia → café → apagado (para el botón compacto de la píldora). */
export function cycleAmbient() {
  const i = SOUNDS.findIndex((s) => s.id === state.kind);
  if (i === SOUNDS.length - 1) stopAmbient();
  else playAmbient(SOUNDS[i + 1].id);
}

export function useAmbient() {
  return useSyncExternalStore(
    (f) => (subs.add(f), () => subs.delete(f)),
    () => state
  );
}
