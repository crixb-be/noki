import { useCallback, useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { useStore } from "../hooks/useStore.js";
import { weatherInfo } from "../lib/wmo.js";
import AppIcon from "./AppIcon.jsx";
import { locale } from "../lib/i18n.js";

// ───────────── iconos de línea (heredan el color del texto) ─────────────

const Svg = ({ children }) => (
  <svg viewBox="0 0 24 24" width="22" height="22" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round" aria-hidden="true">
    {children}
  </svg>
);

const ICONS = {
  wifi: <Svg><path d="M5 12.55a11 11 0 0 1 14.08 0" /><path d="M1.42 9a16 16 0 0 1 21.16 0" /><path d="M8.53 16.11a6 6 0 0 1 6.95 0" /><path d="M12 20h.01" /></Svg>,
  bluetooth: <Svg><path d="m7 7 10 10-5 5V2l5 5L7 17" /></Svg>,
  awake: <Svg><path d="M10 2v2" /><path d="M14 2v2" /><path d="M6 2v2" /><path d="M16 8a1 1 0 0 1 1 1v8a4 4 0 0 1-4 4H7a4 4 0 0 1-4-4V9a1 1 0 0 1 1-1h14a4 4 0 1 1 0 8h-1" /></Svg>,
  dark: <Svg><path d="M12 3a6 6 0 0 0 9 9 9 9 0 1 1-9-9Z" /></Svg>,
  mic: <Svg><path d="M12 2a3 3 0 0 0-3 3v7a3 3 0 0 0 6 0V5a3 3 0 0 0-3-3Z" /><path d="M19 10v2a7 7 0 0 1-14 0v-2" /><path d="M12 19v3" /></Svg>,
  micOff: <Svg><path d="m2 2 20 20" /><path d="M9 9v3a3 3 0 0 0 5.12 2.12" /><path d="M15 9.34V5a3 3 0 0 0-5.68-1.33" /><path d="M19 10v2a7 7 0 0 1-.11 1.23" /><path d="M5 10v2a7 7 0 0 0 12 5" /><path d="M12 19v3" /></Svg>,
  sound: <Svg><path d="M11 5 6 9H3v6h3l5 4V5Z" /><path d="M15.5 8.5a5 5 0 0 1 0 7" /><path d="M18.4 5.6a9 9 0 0 1 0 12.8" /></Svg>,
  soundOff: <Svg><path d="M11 5 6 9H3v6h3l5 4V5Z" /><path d="m22 9-6 6" /><path d="m16 9 6 6" /></Svg>,
  bell: <Svg><path d="M10.27 21a2 2 0 0 0 3.46 0" /><path d="M3.26 15.33A1 1 0 0 0 4 17h16a1 1 0 0 0 .74-1.67C19.41 13.96 18 12.5 18 8A6 6 0 0 0 6 8c0 4.5-1.41 5.96-2.74 7.33" /></Svg>,
  bellOff: <Svg><path d="M10.27 21a2 2 0 0 0 3.46 0" /><path d="M17 17H4a1 1 0 0 1-.74-1.67C4.59 13.96 6 12.5 6 8c0-.6.09-1.19.26-1.74" /><path d="m2 2 20 20" /><path d="M8.67 3.01A6 6 0 0 1 18 8c0 2.69.77 4.65 1.71 6.05" /></Svg>,
  screenshot: <Svg><path d="M6 2v14a2 2 0 0 0 2 2h14" /><path d="M18 22V8a2 2 0 0 0-2-2H2" /></Svg>,
  lock: <Svg><rect x="3" y="11" width="18" height="11" rx="2" /><path d="M7 11V7a5 5 0 0 1 10 0v4" /></Svg>,
};

/** Interruptores: `state` es la clave de `quick_state`; `id` lo que entiende `quick_set`. */
const TOGGLES = [
  { id: "wifi", state: "wifi", icon: "wifi", label: "Wi-Fi" },
  { id: "bluetooth", state: "bluetooth", icon: "bluetooth", label: "Bluetooth" },
  { id: "awake", state: "keepAwake", icon: "awake", label: "Mantener activo" },
  { id: "dark", state: "dark", icon: "dark", label: "Modo oscuro" },
  { id: "mic", state: "micMuted", icon: "mic", offIcon: "micOff", label: "Micrófono", invert: true },
  { id: "sound", state: "muted", icon: "sound", offIcon: "soundOff", label: "Sonido", invert: true },
  { id: "dnd", state: "dnd", icon: "bell", label: "No molestar" },
];

const ACTIONS = [
  { id: "screenshot", icon: "screenshot", label: "Captura de pantalla" },
  { id: "lock", icon: "lock", label: "Bloquear el equipo" },
];

const card = "flex min-h-0 min-w-0 flex-col rounded-3xl bg-zinc-900/90 p-3";
const heading = "mb-2 flex shrink-0 items-center gap-1.5 text-[10px] font-semibold uppercase tracking-wider text-white/45";

/** Programas que se muestran mientras no hayas marcado favoritos en Herramientas → Lanzador. */
const DEFAULT_APPS = [
  { name: "Explorador", path: "C:/Windows/explorer.exe" },
  { name: "Calculadora", path: "C:/Windows/System32/calc.exe" },
  { name: "Bloc de notas", path: "C:/Windows/System32/notepad.exe" },
];

const deg = (v) => `${Math.round(v)}°`;

// ───────────── clima (actual + horas + días) ─────────────

function WeatherCard({ weather, units, city }) {
  const data = weather?.data;
  const cur = data?.current;
  const place = city ? city.split(",")[0] : "Clima";
  if (!cur) {
    return (
      <div className={card}>
        <div className={heading}><span>📍</span><span className="truncate">{place}</span></div>
        <div className="flex flex-1 items-center justify-center text-center text-[10px] text-white/35">
          {weather?.configured ? "Cargando clima…" : "Elige tu ciudad en Ajustes → Clima."}
        </div>
      </div>
    );
  }
  const info = weatherInfo(cur.weather_code, cur.is_day);
  const nowIso = cur.time.slice(0, 13);
  const start = Math.max(0, data.hourly.time.findIndex((t) => t.slice(0, 13) >= nowIso));
  const hours = data.hourly.time.slice(start, start + 6).map((t, i) => {
    const h = Number(t.slice(11, 13));
    return {
      label: i === 0 ? "Ahora" : `${h}h`,
      temp: data.hourly.temperature_2m[start + i],
      icon: weatherInfo(data.hourly.weather_code[start + i], h >= 6 && h < 19 ? 1 : 0).icon,
      rain: data.hourly.precipitation_probability?.[start + i],
    };
  });
  const days = data.daily.time.slice(0, 5).map((t, i) => ({
    label: i === 0 ? "Hoy" : new Date(`${t}T12:00`).toLocaleDateString(locale(), { weekday: "short" }).replace(".", ""),
    icon: weatherInfo(data.daily.weather_code[i]).icon,
    max: data.daily.temperature_2m_max[i],
    min: data.daily.temperature_2m_min[i],
    rain: data.daily.precipitation_probability_max?.[i],
  }));
  const wind = `${Math.round(cur.wind_speed_10m)} ${units === "f" ? "mph" : "km/h"}`;
  const rainText = (r) => (r > 0 ? `💧${r}%` : "\u00a0");

  return (
    <div className={`${card} gap-1.5`}>
      <div className={`${heading} !mb-0`}>
        <span>📍</span>
        <span className="truncate">{place}</span>
      </div>
      <div className="flex shrink-0 items-center gap-2">
        <span className="text-[30px] leading-none">{info.icon}</span>
        <span className="text-[30px] font-semibold leading-none tabular-nums">{deg(cur.temperature_2m)}</span>
        <div className="min-w-0 pl-1">
          <div className="truncate text-[11px] font-medium">{info.text}</div>
          <div className="truncate text-[9px] tabular-nums text-white/45" title={`Viento ${wind}`}>Sens. {deg(cur.apparent_temperature)} · 💧 {cur.relative_humidity_2m}%</div>
        </div>
      </div>
      <div className="flex shrink-0 justify-between border-t border-white/10 pt-1.5">
        {hours.map((h, i) => (
          <div key={i} className="flex w-[34px] flex-col items-center gap-0.5 text-[9px]">
            <span className="text-white/40">{h.label}</span>
            <span className="text-sm leading-none">{h.icon}</span>
            <span className="tabular-nums">{deg(h.temp)}</span>
            <span className="tabular-nums text-sky-300">{rainText(h.rain)}</span>
          </div>
        ))}
      </div>
      <div className="flex shrink-0 justify-between border-t border-white/10 pt-1.5">
        {days.map((d, i) => (
          <div key={i} className="flex w-[40px] flex-col items-center gap-0.5 text-[9px]">
            <span className="capitalize text-white/40">{d.label}</span>
            <span className="text-sm leading-none">{d.icon}</span>
            <span className="tabular-nums">{deg(d.max)}</span>
            <span className="tabular-nums text-white/35">{deg(d.min)}</span>
            <span className="tabular-nums text-sky-300">{rainText(d.rain)}</span>
          </div>
        ))}
      </div>
    </div>
  );
}

// ───────────── apps ─────────────

function AppsCard() {
  const [favs] = useStore("launcherFavs", []);
  const [error, setError] = useState(null);
  const list = (favs.length ? favs : DEFAULT_APPS).slice(0, 6);
  const launch = async (f) => {
    try {
      await invoke("launch_app", { path: f.path });
      invoke("request_open", { open: false, sticky: false }).catch(() => {});
    } catch (e) {
      setError(String(e));
      setTimeout(() => setError(null), 2500);
    }
  };
  return (
    <div className={card}>
      <div className={`${heading} !mb-1.5`}><span>🚀</span><span>Apps</span></div>
      <div className="flex flex-wrap content-start items-start gap-x-2 gap-y-2">
        {list.map((f) => (
          <button key={f.path} onClick={() => launch(f)} title={f.name} className="group flex w-14 min-w-0 flex-col items-center gap-1 active:scale-95">
            <AppIcon app={f} size={30} className="transition-transform group-hover:scale-110" />
            <span className="w-full truncate text-center text-[9px] text-white/60">{f.name}</span>
          </button>
        ))}
      </div>
      {!favs.length && <div className="pt-1 text-[9px] leading-tight text-white/25">Marca ★ en Herramientas → Lanzador para poner aquí tus programas.</div>}
      {error && <div className="mt-1 text-[10px] text-amber-300">{error}</div>}
    </div>
  );
}

// ───────────── panel ─────────────

export default function PanelView({ weather, units, city, dnd, update }) {
  const [state, setState] = useState(null);
  const [busy, setBusy] = useState(null);
  const [msg, setMsg] = useState(null);

  // Si una lectura falla a medias (null), se conserva el valor anterior en vez de apagar el interruptor.
  const refresh = useCallback(
    () =>
      invoke("quick_state")
        .then((next) => setState((prev) => (prev ? { ...next, ...Object.fromEntries(Object.entries(prev).filter(([k, v]) => next[k] == null && v != null)) } : next)))
        .catch(() => {}),
    []
  );
  useEffect(() => {
    refresh();
    const id = setInterval(refresh, 4000);
    return () => clearInterval(id);
  }, [refresh]);

  const flash = (text) => {
    setMsg(text);
    setTimeout(() => setMsg(null), 3000);
  };

  const toggle = async (t) => {
    if (busy) return;
    if (t.id === "dnd") {
      update({ general: { dnd: !dnd } });
      return;
    }
    const cur = state?.[t.state];
    // Wi-Fi y Bluetooth se pueden intentar aunque Windows no haya dado su estado (se asume apagado → encender).
    if (cur == null && !["awake", "dark", "wifi", "bluetooth"].includes(t.id)) return;
    setBusy(t.id);
    try {
      await invoke("quick_set", { kind: t.id, on: !cur });
    } catch (e) {
      flash(String(e));
    }
    await refresh();
    setBusy(null);
  };

  const act = (a) => invoke("quick_action", { kind: a.id }).then(() => a.id === "screenshot" && invoke("request_open", { open: false, sticky: false })).catch((e) => flash(String(e)));

  const tile = "flex h-10 min-w-0 items-center justify-center rounded-2xl transition-all duration-300 active:scale-95 disabled:opacity-30";

  return (
    <div className="grid h-full min-h-0 grid-cols-[1.25fr_1fr] gap-2">
      <WeatherCard weather={weather} units={units} city={city} />
      <div className="flex min-h-0 min-w-0 flex-col gap-2">
      <div className={`${card} shrink-0 justify-center`}>
        <div className="grid grid-cols-5 gap-1.5">
          {TOGGLES.map((t) => {
            const v = t.id === "dnd" ? dnd : state?.[t.state];
            const available = t.id === "dnd" || v != null;
            // En «micrófono» y «sonido» el estado guardado es «silenciado»: el botón se ve encendido cuando NO lo está.
            const on = t.invert ? available && !v : Boolean(v);
            const usable = available || ["awake", "dark", "wifi", "bluetooth"].includes(t.id);
            const icon = t.id === "dnd" ? (dnd ? "bellOff" : "bell") : t.invert && !on ? t.offIcon : t.icon;
            return (
              <button
                key={t.id}
                onClick={() => toggle(t)}
                disabled={!usable}
                title={usable ? `${t.label}: ${on ? "activado" : "desactivado"}` : `${t.label}: no disponible en este equipo`}
                aria-label={t.label}
                className={`${tile} ${on ? "bg-white text-zinc-900" : "bg-zinc-800 text-white/65 hover:bg-zinc-700"} ${busy === t.id ? "animate-pulse" : ""}`}
              >
                {ICONS[icon]}
              </button>
            );
          })}
          {ACTIONS.map((a) => (
            <button key={a.id} onClick={() => act(a)} title={a.label} aria-label={a.label} className={`${tile} bg-zinc-800 text-white/65 hover:bg-zinc-700`}>
              {ICONS[a.icon]}
            </button>
          ))}
        </div>
        {msg && <div className="mt-1 rounded-xl bg-amber-500/10 px-2 py-1 text-[10px] text-amber-300">{msg}</div>}
      </div>
      <div className="min-h-0 flex-1 [&>*]:h-full"><AppsCard /></div>
      </div>
    </div>
  );
}
