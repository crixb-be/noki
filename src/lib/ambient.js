import { useSyncExternalStore } from "react";

/**
 * Sonidos ambientales para concentrarse, generados al vuelo con WebAudio (sin archivos de audio).
 * Cuatro sonidos bien distintos entre sí:
 *  - Lluvia: ruido rosa filtrado, con una modulación suave de estática.
 *  - Olas de mar: ruido blanco cuyo filtro y volumen suben y bajan despacio (marea).
 *  - Fogata: ruido marrón grave con crujidos de madera aleatorios.
 *  - Ruido blanco: constante y puro.
 * Un único motor por ventana; la UI lo lee con `useAmbient()`.
 */
export const SOUNDS = [
  { id: "rain", label: "Lluvia", icon: "🌧️" },
  { id: "waves", label: "Olas de mar", icon: "🌊" },
  { id: "fire", label: "Fogata", icon: "🔥" },
  { id: "white", label: "Ruido blanco", icon: "🌫️" },
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

/** Buffer de ruido: blanco, rosa (Paul Kellet, −3 dB/octava) o marrón (−6 dB/octava). */
function noiseBuffer(c, kind = "white", seconds = 8) {
  const buf = c.createBuffer(1, c.sampleRate * seconds, c.sampleRate);
  const d = buf.getChannelData(0);
  let last = 0;
  let b0 = 0, b1 = 0, b2 = 0, b3 = 0, b4 = 0, b5 = 0, b6 = 0;
  for (let i = 0; i < d.length; i++) {
    const w = Math.random() * 2 - 1;
    if (kind === "brown") {
      last = (last + 0.02 * w) / 1.02;
      d[i] = last * 3.5;
    } else if (kind === "pink") {
      b0 = 0.99886 * b0 + w * 0.0555179;
      b1 = 0.99332 * b1 + w * 0.0750759;
      b2 = 0.969 * b2 + w * 0.153852;
      b3 = 0.8665 * b3 + w * 0.3104856;
      b4 = 0.55 * b4 + w * 0.5329522;
      b5 = -0.7616 * b5 - w * 0.016898;
      d[i] = (b0 + b1 + b2 + b3 + b4 + b5 + b6 + w * 0.5362) * 0.11;
      b6 = w * 0.115926;
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
  /** Oscilador lento que mueve un parámetro: valor base ± `depth`. */
  const lfo = (param, hz, depth) => {
    const o = c.createOscillator();
    const g = c.createGain();
    o.frequency.value = hz;
    g.gain.value = depth;
    o.connect(g).connect(param);
    o.start();
    nodes.push(o, g);
  };

  if (id === "white") {
    // Ruido blanco constante y puro: sin filtros ni modulación.
    const n = loop(c, "white");
    n.connect(gain(0.3)).connect(out);
    nodes.push(n);
  } else if (id === "rain") {
    // Ruido rosa filtrado + estática suave (dos ondulaciones lentas e incomensurables de volumen).
    const n = loop(c, "pink");
    const hp = filter("highpass", 500);
    const lp = filter("lowpass", 7500);
    const g = gain(0.55);
    lfo(g.gain, 0.31, 0.06);
    lfo(g.gain, 0.83, 0.035);
    n.connect(hp).connect(lp).connect(g).connect(out);
    nodes.push(n);
  } else if (id === "waves") {
    // Marea: ruido blanco con el corte del filtro y el volumen subiendo y bajando cada ~12 s.
    const n = loop(c, "white");
    const lp = filter("lowpass", 1300, 0.9);
    lfo(lp.frequency, 0.085, 1000);
    const g = gain(0.3);
    lfo(g.gain, 0.085, 0.22);
    n.connect(lp).connect(g).connect(out);
    nodes.push(n);
  } else {
    // Fogata: base grave de ruido marrón + crujidos de madera (ráfagas cortas y agudas, a intervalos aleatorios).
    const base = loop(c, "brown");
    const lp = filter("lowpass", 900);
    base.connect(lp).connect(gain(0.9)).connect(out);
    nodes.push(base);
    const sparks = c.createBuffer(1, c.sampleRate, c.sampleRate);
    const sd = sparks.getChannelData(0);
    for (let i = 0; i < sd.length; i++) sd[i] = Math.random() * 2 - 1;
    const crackle = (loud) => {
      const s = c.createBufferSource();
      s.buffer = sparks;
      const bp = c.createBiquadFilter();
      bp.type = "bandpass";
      bp.frequency.value = 1200 + Math.random() * 3200;
      bp.Q.value = 1.2;
      const g = c.createGain();
      const t = c.currentTime;
      const dur = loud ? 0.05 + Math.random() * 0.05 : 0.012 + Math.random() * 0.03;
      const peak = loud ? 0.5 + Math.random() * 0.25 : 0.1 + Math.random() * 0.2;
      g.gain.setValueAtTime(0, t);
      g.gain.linearRampToValueAtTime(peak, t + 0.002);
      g.gain.exponentialRampToValueAtTime(0.0001, t + dur);
      s.connect(bp).connect(g).connect(out);
      s.start(t, Math.random() * 0.8, dur + 0.02);
    };
    let alive = true;
    const next = () => {
      if (!alive) return;
      crackle(Math.random() < 0.12);
      if (Math.random() < 0.35) setTimeout(() => alive && crackle(false), 30 + Math.random() * 70); // chasquidos en racha
      timers.push(setTimeout(next, 90 + Math.random() * 650));
    };
    timers.push(setTimeout(next, 200));
    timers.push({ stopFire: () => (alive = false) });
  }
  return () => {
    timers.forEach((t) => (t?.stopFire ? t.stopFire() : clearTimeout(t)));
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

/** Siguiente sonido del ciclo lluvia → olas → fogata → ruido blanco → apagado (botón compacto de la píldora). */
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
