import { useEffect, useLayoutEffect, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { BatteryIcon, BluetoothIcon, SpeakerIcon, WifiIcon } from "./SymbolIcons.jsx";
import { useMedia } from "../../hooks/useMedia.js";
import { useSettings } from "../../hooks/useSettings.js";
import { useGlass, useGlassShape } from "../../lib/glass.js";

const action = (uri) => invoke("command_open_system_uri", { uri }).catch(() => {});
const Switch = ({ checked, onChange }) => <button type="button" role="switch" aria-checked={checked} onClick={() => onChange(!checked)} className={`panel-switch ${checked ? "on" : ""}`}><span /></button>;

/** Píldora del Centro de control: círculo de icono (blanco con icono azul si está activa) + título y estado. */
function CcPill({ title, subtitle, icon, onClick, on = false }) {
  return <button type="button" data-glass onClick={onClick} className={`cc-module cc-pill ${on ? "cc-on" : ""}`}><span className="cc-icon">{icon}</span><span><strong>{title}</strong><small>{subtitle}</small></span></button>;
}

function CcCircle({ title, icon, onClick }) {
  return <button type="button" data-glass title={title} aria-label={title} onClick={onClick} className="cc-module cc-circle">{icon}</button>;
}

/**
 * Centro de control modular, como Capturas/macdock-centro-control.png: sin contenedor, cada módulo
 * flota como una píldora o tarjeta de vidrio con su propio desenfoque (`data-glass`).
 */
function ControlCenter({ brightness, setBrightness, volume, setVolume }) {
  const [quick, setQuick] = useState({});
  const [airplane, setAirplane] = useState(false);
  const media = useMedia();
  useEffect(() => { invoke("quick_state").then((s) => setQuick(s ?? {})).catch(() => {}); }, []);
  const toggle = (kind, value) => { setQuick((q) => ({ ...q, [kind]: value })); invoke("quick_set", { kind, on: value }).catch(() => {}); };
  const dark = !!quick.dark;
  const track = media.track ?? {};
  return <>
    <div className="cc-grid">
      <div className="cc-col">
        <CcPill title="Wi-Fi" subtitle={quick.wifi === false ? "Desactivado" : "Activado"} on={quick.wifi !== false} icon={<WifiIcon level={3} />} onClick={() => toggle("wifi", quick.wifi === false)} />
        <CcPill title="Bluetooth" subtitle={quick.bluetooth ? "Activado" : "Desactivado"} on={!!quick.bluetooth} icon={<BluetoothIcon />} onClick={() => toggle("bluetooth", !quick.bluetooth)} />
        <CcPill title="Concentración" subtitle="Ajustes" icon="◐" onClick={() => action("ms-settings:quiethours")} />
        <CcPill title="Luz nocturna" subtitle="Ajustes" icon="☼" onClick={() => action("ms-settings:nightlight")} />
        <CcPill title="Ahorro de batería" subtitle="Ajustes" icon="▭" onClick={() => action("ms-settings:batterysaver")} />
        <CcPill title="Captura" subtitle="Recortar pantalla" icon="⌗" onClick={() => action("ms-screenclip:")} />
        <CcPill title="Modo oscuro" subtitle={dark ? "Activado" : "Desactivado"} on={dark} icon="◑" onClick={() => { setQuick((q) => ({ ...q, dark: !dark })); invoke("command_toggle_dark", { dark: !dark }).catch(() => {}); }} />
      </div>
      <div className="cc-col">
        <div data-glass className="cc-module cc-media">
          <div className="cc-media-top">
            <span className="cc-art" style={track.art ? { backgroundImage: `url(${track.art})` } : undefined} />
            <span className="cc-media-text"><strong>{media.available && track.title ? track.title : "Sin reproducción"}</strong><small>{media.available ? track.artist || track.source || "" : "Multimedia"}</small></span>
          </div>
          <div className="cc-media-controls">
            <button type="button" aria-label="Anterior" disabled={!media.can?.prev} onClick={() => media.control("prev")}>⏮</button>
            <button type="button" aria-label={media.playing ? "Pausar" : "Reproducir"} disabled={!media.available} onClick={() => media.control("toggle")}>{media.playing ? "⏸" : "▶"}</button>
            <button type="button" aria-label="Siguiente" disabled={!media.can?.next} onClick={() => media.control("next")}>⏭</button>
          </div>
        </div>
        <div className="cc-row"><CcCircle title="Configuración" icon="⚙" onClick={() => action("ms-settings:")} /><CcCircle title="Notificaciones" icon="🔔" onClick={() => action("ms-actioncenter:")} /></div>
        <div className="cc-row"><CcCircle title="Bloquear" icon="🔒" onClick={() => invoke("command_lock_user").catch(() => {})} /><CcCircle title="Administrador de tareas" icon="▤" onClick={() => invoke("command_open_task_manager").catch(() => {})} /></div>
        <CcPill title="Proyectar" subtitle="Pantallas" icon="▣" onClick={() => action("ms-settings-displays-topology:projection")} />
        <CcPill title="Modo avión" subtitle={airplane ? "Activado" : "Desactivado"} on={airplane} icon="✈" onClick={() => { setAirplane(!airplane); invoke("command_toggle_airplane", { on: !airplane }).catch(() => {}); }} />
        <CcPill title="Grabar" subtitle="Pantalla" icon="●" onClick={() => invoke("command_send_keys", { combo: "record" }).catch(() => {})} />
      </div>
    </div>
    <div data-glass className="cc-module cc-slider">
      <span>Pantalla</span>
      <div className="cc-slider-row"><span>☀</span><input className="cc-range" type="range" min="0" max="100" value={brightness} style={{ "--v": `${brightness}%` }} onChange={(e) => { const v = Number(e.target.value); setBrightness(v); invoke("set_brightness", { value: v }).catch(() => {}); }} /></div>
    </div>
    <div data-glass className="cc-module cc-slider">
      <span>Sonido</span>
      <div className="cc-slider-row"><SpeakerIcon level={3} /><input className="cc-range" type="range" min="0" max="100" value={volume} style={{ "--v": `${volume}%` }} onChange={(e) => { const v = Number(e.target.value); setVolume(v); invoke("set_volume", { value: v }).catch(() => {}); }} /></div>
    </div>
  </>;
}

export default function BarPanel() {
  const { settings } = useSettings();
  // Estilo de paneles (Ajustes > Apariencia): «light» vidrio claro difuminado, «dark» oscuro opaco.
  useEffect(() => { document.documentElement.dataset.panels = settings?.appearance?.panelStyle === "dark" ? "dark" : "light"; }, [settings?.appearance?.panelStyle]);
  const [kind, setKind] = useState(null);
  const [wifi, setWifi] = useState([]);
  const [devices, setDevices] = useState([]);
  const [battery, setBattery] = useState(null);
  const [volume, setVolume] = useState(50);
  const [brightness, setBrightness] = useState(70);
  const [muted, setMuted] = useState(false);
  const [airplane, setAirplane] = useState(false);
  const [output, setOutput] = useState("Salida predeterminada");
  const [dark, setDark] = useState(false);
  const [loading, setLoading] = useState(false);
  const ref = useRef(null);
  const body = useRef(null);
  const [openCount, setOpenCount] = useState(0);

  useEffect(() => {
    let stop;
    invoke("command_bar_panel_state").then((value) => value && setKind(value)).catch(() => {});
    listen("bar-panel", (event) => { setKind(typeof event.payload === "string" ? event.payload : event.payload?.kind ?? null); setOpenCount((n) => n + 1); }).then((fn) => { stop = fn; });
    return () => stop?.();
  }, []);
  useEffect(() => {
    const onKey = (e) => { if (e.key === "Escape") invoke("command_close_bar_panel").catch(() => {}); };
    window.addEventListener("keydown", onKey); return () => window.removeEventListener("keydown", onKey);
  }, []);
  useEffect(() => {
    if (!kind) return;
    setLoading(true);
    const jobs = [];
    if (kind === "wifi") jobs.push(invoke("command_wifi_networks").then(setWifi));
    if (kind === "bluetooth") jobs.push(invoke("command_bt_devices").then(setDevices));
    if (kind === "battery") jobs.push(invoke("command_battery_info").then(setBattery));
    if (kind === "sound") jobs.push(invoke("command_audio_output_name").then(setOutput));
    Promise.allSettled(jobs).finally(() => setLoading(false));
  }, [kind]);
  // La ventana mide exactamente lo que la tarjeta: sin huecos transparentes ni zonas sin pintar.
  useLayoutEffect(() => {
    const node = ref.current;
    const inner = body.current;
    if (!node || !inner) return undefined;
    let last = 0;
    const report = () => {
      const style = getComputedStyle(node);
      const frame = ["paddingTop", "paddingBottom", "borderTopWidth", "borderBottomWidth"].reduce((sum, key) => sum + (parseFloat(style[key]) || 0), 0);
      const height = Math.ceil(inner.offsetHeight + frame);
      if (height && height !== last) { last = height; invoke("command_resize_bar_panel", { height }).catch(() => {}); }
    };
    report();
    const observer = new ResizeObserver(report);
    observer.observe(inner);
    return () => observer.disconnect();
  }, [kind, openCount]);

  const glass = useGlass(settings, "panels");
  // Centro de control: una forma de vidrio por módulo; el resto de paneles, la tarjeta entera.
  useGlassShape(ref, glass, [kind, openCount], kind === "control" ? "[data-glass]" : null);
  if (!kind) return null;
  const panelDark = settings?.topbar?.tint === "dark" || (settings?.topbar?.tint !== "light" && window.matchMedia?.("(prefers-color-scheme: dark)").matches !== false);
  const card = kind === "control" ? "cc-root" : `noki-popup-card ${panelDark ? "bar-panel-dark" : "bar-panel-light"}`;
  return <div ref={ref} className={`bar-panel ${card}`} onMouseDown={(e) => e.stopPropagation()}><div ref={body} className="bar-panel-body">
    {kind === "wifi" && <>
      <header className="panel-head-mac"><h2>Wi-Fi</h2><Switch checked onChange={(v) => invoke("quick_set", { kind: "wifi", on: v })} /></header>
      <div className="panel-status"><span>Estado</span><span>{wifi.find((x) => x.connected)?.ssid ? `Conectado a ${wifi.find((x) => x.connected).ssid}` : "Sin conexión"}</span></div>
      <div className="panel-divider" />
      <PanelLabel>REDES</PanelLabel>
      {loading && !wifi.length ? <p className="panel-empty">Buscando redes…</p> : wifi.length ? wifi.map((n) => <button className="panel-row" key={n.ssid} onClick={() => invoke("command_wifi_connect", { ssid: n.ssid })}><span className="panel-check-slot">{n.connected ? <span className="panel-check">✓</span> : null}</span><span className="panel-name">{n.ssid}</span></button>) : <p className="panel-empty">No se ven redes cerca</p>}
      <div className="panel-divider" />
      <button type="button" className="panel-link panel-link-plain" onClick={() => action("ms-settings:network-wifi")}>Configuración de red…</button>
    </>}
    {kind === "bluetooth" && <><header><BluetoothIcon /><div><h2>Bluetooth</h2><p>{loading ? "Buscando…" : devices.length ? "Activado" : "Ningún dispositivo conectado"}</p></div><Switch checked onChange={(v) => invoke("quick_set", { kind: "bluetooth", on: v })} /></header><PanelLabel>DISPOSITIVOS CONECTADOS</PanelLabel>{loading ? <p className="panel-empty">Buscando…</p> : devices.map((d) => <div className="panel-row" key={d}><BluetoothIcon /><span>{d}</span></div>)}{!loading && !devices.length && <p className="panel-empty">Ninguno conectado</p>}<PanelLink onClick={() => action("ms-settings:bluetooth")}>Configuración de Bluetooth…</PanelLink></>}
    {kind === "sound" && <><header><SpeakerIcon level={3} muted={muted} /><div><h2>Sonido</h2><p>{output}</p></div><Switch checked={!muted} onChange={(v) => { setMuted(!v); invoke("set_mute", { muted: !v }); }} /></header><PanelLabel>VOLUMEN</PanelLabel><div className="panel-slider"><SpeakerIcon level={3} muted={muted} /><input type="range" min="0" max="100" value={volume} onChange={(e) => { const v = Number(e.target.value); setVolume(v); invoke("set_volume", { value: v }); }} /><span>{volume}%</span></div><PanelLink onClick={() => action("ms-settings:sound")}>Configuración de sonido…</PanelLink></>}
    {kind === "battery" && <><header><BatteryIcon pct={battery?.level ?? 100} charging={battery?.charging} /><div><h2>Batería</h2><p>{battery?.charging ? "Conectado, cargando" : "Usando batería"}</p></div></header><div className="battery-big">{battery?.level ?? "—"}%</div><p className="panel-muted">Fuente de energía: {battery?.ac ? "Corriente alterna" : "Batería"}</p><PanelLink onClick={() => action("ms-settings:batterysaver")}>Configuración de batería…</PanelLink></>}
    {kind === "user" && <><header><span className="panel-user">U</span><div><h2>Usuario</h2><p>Cambiar opciones de cuenta</p></div></header><PanelLink onClick={() => invoke("command_lock_user")}>Bloquear pantalla</PanelLink><PanelLink onClick={() => invoke("command_logoff_user")}>Cerrar sesión…</PanelLink><PanelLink onClick={() => action("ms-settings:yourinfo")}>Configuración de cuenta…</PanelLink></>}
    {kind === "control" && <ControlCenter brightness={brightness} setBrightness={setBrightness} volume={volume} setVolume={setVolume} />}
  </div></div>;
}

function PanelLabel({ children }) { return <div className="panel-label">{children}</div>; }
function PanelLink({ children, onClick }) { return <button type="button" className="panel-link" onClick={onClick}>{children}<span>›</span></button>; }

