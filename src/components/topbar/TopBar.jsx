import { useEffect, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { useSettings } from "../../hooks/useSettings.js";
import { isEn, presetLanguage, useLanguage } from "../../lib/i18n.js";
import { BatteryIcon, BellIcon, BluetoothIcon, CameraIcon, ControlCenterIcon, RecordIcon, SearchIcon, SpeakerIcon, WifiIcon } from "./SymbolIcons.jsx";
import { BarPaint, useBarPaint } from "./barPaint.jsx";
import { useGlass, useGlassShape } from "../../lib/glass.js";

function Item({ title, onClick, children, className = "" }) {
  return <button type="button" title={title} aria-label={title} onClick={onClick} className={`topbar-item ${className}`}>{children}</button>;
}

function formatClock(date) {
  const parts = Object.fromEntries(new Intl.DateTimeFormat(isEn() ? "en-US" : "es-ES", { weekday: "short", day: "numeric", month: "short", hour: "numeric", minute: "2-digit", hour12: true }).formatToParts(date).map((p) => [p.type, p.value]));
  if (isEn()) return `${parts.weekday} ${parts.month} ${parts.day} ${parts.hour}:${parts.minute} ${parts.dayPeriod}`;
  const weekdays = { dom: "dom.", lun: "lun.", mar: "mar.", mié: "mié.", jue: "jue.", vie: "vie.", sáb: "sáb.", sun: "dom.", mon: "lun.", tue: "mar.", wed: "mié.", thu: "jue.", fri: "vie.", sat: "sáb." };
  const weekday = weekdays[parts.weekday?.toLowerCase()] ?? `${parts.weekday}.`;
  return `${weekday} ${parts.day} ${parts.month} ${parts.hour}:${parts.minute} ${parts.dayPeriod}`;
}

export default function TopBar() {
  const { settings } = useSettings();
  const shell = useRef(null);
  // Desenfoque real debajo de la barra (solo en modo vidrio; el backend decide si se muestra).
  const glass = useGlass(settings, "bar");
  useGlassShape(shell, glass);
  const [now, setNow] = useState(() => new Date());
  const [avatar, setAvatar] = useState(null);
  const [battery, setBattery] = useState(null);
  const [wifi, setWifi] = useState({ level: 3, off: false });
  const [bt, setBt] = useState([]);
  const [user, setUser] = useState({ displayName: "Usuario" });
  const [stats, setStats] = useState(null);
  const t = settings?.topbar ?? {};
  const paint = useBarPaint(settings);

  presetLanguage(settings?.general?.language);
  useLanguage(settings?.general?.language);

  useEffect(() => {
    const timer = setInterval(() => setNow(new Date()), 1000);
    return () => clearInterval(timer);
  }, []);
  useEffect(() => {
    if (!settings) return;
    let stop;
    const timer = setTimeout(() => {
      invoke("command_user_avatar").then(setAvatar).catch(() => {});
      invoke("command_user_info").then(setUser).catch(() => {});
      invoke("command_battery_info").then(setBattery).catch(() => {});
      invoke("system_stats").then(setStats).catch(() => {});
      invoke("quick_state").then((state) => {
        setBt(state?.bluetooth ? ["Bluetooth"] : []);
        if (typeof state?.wifi === "boolean" && !state.wifi) setWifi({ level: 0, off: true });
      }).catch(() => {});
    }, 0);
    listen("avatar-changed", (event) => setAvatar(event.payload || null)).then((fn) => { stop = fn; });
    return () => { clearTimeout(timer); stop?.(); };
  }, [settings?.topbar?.avatarPath]);

  // Sobre el vidrio gris el texto va siempre en blanco (como en macOS).
  const dark = true;
  const open = (kind, event) => invoke("command_open_bar_panel", { kind, anchorX: Math.round(event.currentTarget.getBoundingClientRect().left + event.currentTarget.offsetWidth / 2) }).catch(() => {});
  const initials = (user.displayName || "U").trim().slice(0, 1).toUpperCase();
  const batteryPct = battery?.level ?? 100;
  const reserve = Math.max(180, settings?.appearance?.collapsedW ?? 240);

  if (!settings) return null;
  return (
    <div ref={shell} className={`topbar-shell ${dark ? "topbar-dark" : "topbar-light"}`} style={{ "--notch-reserve": `${reserve}px` }}>
      <BarPaint settings={settings} tint={paint.tint} />
      <div className="topbar-left" />
      <div className="topbar-center-reserve" aria-hidden="true" />
      <div className="topbar-actions">
        {t.showCpu && <Item title="CPU" onClick={() => invoke("command_open_task_manager")}><span className="topbar-text">CPU {stats ? Math.round(stats.cpu) : "--"}%</span></Item>}
        {t.showRam && <Item title="RAM" onClick={() => invoke("command_open_task_manager")}><span className="topbar-text">RAM {stats?.memTotal ? Math.round((stats.memUsed / stats.memTotal) * 100) : "--"}%</span></Item>}
        {t.showBluetooth && <Item title="Bluetooth" onClick={(e) => open("bluetooth", e)}><BluetoothIcon off={!bt.length} /></Item>}
        {t.showSound && <Item title="Sonido" onClick={(e) => open("sound", e)}><SpeakerIcon level={3} /></Item>}
        {t.showWifi && <Item title="Wi-Fi" onClick={(e) => open("wifi", e)}><WifiIcon level={wifi.level} off={wifi.off} /></Item>}
        {t.showBattery && battery && <Item title="Bateria" onClick={(e) => open("battery", e)}><BatteryIcon pct={batteryPct} charging={battery.charging} />{t.showBatteryPct && <span className="topbar-text">{batteryPct}%</span>}</Item>}
        {t.showSearch && <Item title="Buscar" onClick={() => invoke("toggle_spotlight")}><SearchIcon /></Item>}
        {t.showControlCenter && <Item title="Centro de control" onClick={(e) => open("control", e)}><ControlCenterIcon /></Item>}
        {t.showCapture && <><Item title="Grabar" onClick={() => invoke("command_send_keys", { combo: "record" })}><RecordIcon /></Item><Item title="Captura" onClick={() => invoke("command_open_system_uri", { uri: "ms-screenclip:" })}><CameraIcon /></Item></>}
        {t.showNotifications && <Item title="Notificaciones" onClick={() => invoke("command_open_system_uri", { uri: "ms-actioncenter:" }).catch(() => invoke("command_send_keys", { combo: "notifications" }))}><BellIcon /></Item>}
        {t.showAvatar && <Item title="Usuario" onClick={(e) => open("user", e)}><span className="topbar-avatar">{avatar ? <img src={avatar} alt="" /> : initials}</span></Item>}
        {t.showClock && <Item title="Fecha y hora" onClick={() => invoke("command_open_system_uri", { uri: "ms-settings:dateandtime" })}><span className="topbar-clock">{formatClock(now)}</span></Item>}
      </div>
    </div>
  );
}

