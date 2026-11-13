import { useEffect, useMemo, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { useSettings } from "../../hooks/useSettings.js";
import { useGlass, useGlassShape } from "../../lib/glass.js";

function AppsIcon() { return <span className="apps-icon">{Array.from({ length: 9 }, (_, i) => <i key={i} />)}</span>; }

export default function Dock() {
  const { settings } = useSettings();
  const [items, setItems] = useState([]);
  const [trash, setTrash] = useState({ items: 0 });
  const [menu, setMenu] = useState(null);
  const pill = useRef(null);
  const itemRefs = useRef([]);
  const iconRefs = useRef([]);
  const pointerX = useRef(null);
  const frame = useRef(0);
  const dock = settings?.dock ?? { enabled: true, iconSize: 40, magnify: true, magnifyScale: 1.6, showApps: true, showRunning: true, showTrash: true, autohide: false, pinned: [], folders: [] };
  // Tamaño y escala vienen de Ajustes (antes estaban fijos: 44-46 px y 62 px máximo).
  const size = Math.min(96, Math.max(32, Math.round(dock.iconSize || 48)));
  const scale = dock.magnify ? Math.min(2.5, Math.max(1, Number(dock.magnifyScale) || 1.6)) : 1;
  const maxSize = Math.round(size * scale);
  const sizes = useRef([]);
  const rest = useRef(null);
  const geometry = useRef({ lift: 0 });
  const reportHit = useRef(null);

  const refresh = async () => {
    try { if (!(await getCurrentWindow().isVisible())) return; } catch { /* El navegador de desarrollo no expone visibilidad nativa. */ }
    invoke("dock_items").then(setItems).catch(() => {});
    invoke("trash_state").then(setTrash).catch(() => {});
  };
  useEffect(() => { refresh(); const timer = setInterval(refresh, 1000); return () => clearInterval(timer); }, []);
  useEffect(() => { let stops = []; Promise.all([listen("settings-updated", refresh), listen("settings-changed", refresh), listen("dock-windows", (event) => setItems(event.payload ?? []))]).then((fns) => { stops = fns; }); return () => stops.forEach((stop) => stop()); }, []);
  const all = useMemo(() => [ ...(dock.showApps ? [{ key: "apps", name: "Aplicaciones", kind: "apps" }] : []), ...items, ...(dock.showTrash ? [{ key: "trash", name: "Papelera", kind: "trash", running: trash.items > 0 }] : []) ], [dock.showApps, dock.showTrash, items, trash.items]);
  // Lo que necesita el área de clic sin depender del cierre del efecto.
  geometry.current = { lift: dock.magnify ? maxSize - size : 0 };
  // Área de clic: la píldora (fija) y, mientras se magnifica, también la franja de encima donde
  // asoman los iconos ampliados (si no, al subir el cursor al icono grande el Dock lo soltaba).
  useEffect(() => {
    const node = pill.current;
    if (!node) return undefined;
    const report = () => {
      const r = node.getBoundingClientRect();
      const lift = pointerX.current != null ? geometry.current.lift : 0;
      invoke("dock_set_hit", { x: Math.round(r.left - lift * 2), y: Math.round(r.top - lift), w: Math.round(r.width + lift * 4), h: Math.round(r.height + lift) }).catch(() => {});
    };
    reportHit.current = report;
    report();
    const observer = new ResizeObserver(report);
    observer.observe(node);
    // Al redimensionar/recolocar la ventana la píldora se recentra sin cambiar de tamaño:
    // sin esto el área de clic quedaba desfasada y el Dock dejaba de responder.
    window.addEventListener("resize", report);
    const timer = setInterval(report, 2000);
    return () => { observer.disconnect(); window.removeEventListener("resize", report); clearInterval(timer); };
  }, []);
  // Desenfoque real debajo de la píldora. La píldora no cambia de tamaño al magnificar, así que el
  // vidrio nativo no se recoloca en cada fotograma (era la franja clara que asomaba encima del Dock).
  const glass = useGlass(settings, "dock");
  useGlassShape(pill, glass);
  // Tamaño de la ventana: ancho natural de la píldora en reposo + holgura para la magnificación.
  // Antes la píldora se limitaba al ancho de la ventana y la ventana a la píldora: los iconos se recortaban.
  useEffect(() => {
    const node = pill.current;
    if (!node || pointerX.current != null) return;
    const width = Math.ceil(node.scrollWidth + (dock.magnify ? (maxSize - size) * 4 : 0) + 24);
    const height = Math.ceil(maxSize + 8 + 24);
    invoke("dock_resize", { width, height }).catch(() => {});
  }, [all.length, size, maxSize, dock.magnify]);

  if (settings && !dock.enabled) return null;
  // Magnificación solo con transform (ni width ni height cambian): el icono crece desde abajo con
  // scale() y los vecinos se apartan con translateX, así la píldora y su vidrio quedan quietos.
  function apply(index, scaleValue, shift) {
    const item = itemRefs.current[index];
    const icon = iconRefs.current[index];
    if (!item || !icon) return;
    item.style.transform = shift ? `translateX(${shift.toFixed(2)}px)` : "";
    icon.style.transform = scaleValue !== 1 ? `scale(${scaleValue.toFixed(4)})` : "";
  }

  // Centros de cada icono en reposo, relativos al centro de la píldora (siempre centrada en
  // la ventana). Medir sobre posiciones en reposo evita el temblor de medir iconos en movimiento.
  function measureRest() {
    const node = pill.current;
    if (!node) return null;
    const gap = parseFloat(getComputedStyle(node).columnGap) || 8;
    const count = itemRefs.current.filter(Boolean).length;
    const total = count * size + Math.max(0, count - 1) * gap;
    return { centers: Array.from({ length: count }, (_, i) => i * (size + gap) + size / 2 - total / 2) };
  }
  // Curva tipo macOS: el icono bajo el cursor llega a tamaño × escala y los vecinos
  // crecen menos (campana coseno, alcance ≈ 3 iconos). Interpolación por fotograma.
  function tick() {
    frame.current = 0;
    const node = pill.current;
    if (!node) return;
    const box = node.getBoundingClientRect();
    const center = box.left + box.width / 2;
    const x = pointerX.current;
    const reach = size * 3;
    let moving = false;
    const items = itemRefs.current;
    items.forEach((item, index) => {
      if (!item) return;
      let target = size;
      if (x != null && dock.magnify && rest.current?.centers[index] != null) {
        const d = Math.abs(x - (center + rest.current.centers[index]));
        if (d < reach) target = size + (maxSize - size) * (0.5 + 0.5 * Math.cos(Math.PI * d / reach));
      }
      const current = sizes.current[index] ?? size;
      const next = Math.abs(target - current) < 0.4 ? target : current + (target - current) * 0.35;
      if (next !== target) moving = true;
      sizes.current[index] = next;
    });
    // Cada icono ampliado ocupa `extra` px más; se reparte alrededor del centro de la píldora.
    const extras = items.map((item, index) => (item ? (sizes.current[index] ?? size) - size : 0));
    const total = extras.reduce((sum, value) => sum + value, 0);
    let before = 0;
    items.forEach((item, index) => {
      if (!item) return;
      const value = sizes.current[index] ?? size;
      apply(index, value / size, before + extras[index] / 2 - total / 2);
      before += extras[index];
      item.style.zIndex = String(Math.round(value));
    });
    if (moving || x != null) frame.current = requestAnimationFrame(tick);
  }

  function start() { if (!frame.current) frame.current = requestAnimationFrame(tick); }

  function resetMagnification() {
    const was = pointerX.current != null;
    pointerX.current = null;
    if (was) reportHit.current?.();
    start();
  }

  function onMouseMove(event) {
    const first = pointerX.current == null;
    if (first) rest.current = measureRest();
    pointerX.current = event.clientX;
    if (first) reportHit.current?.();
    start();
  }  function activate(item) {
    if (item.kind === "apps") return invoke("toggle_launchpad");
    if (item.kind === "trash") return invoke("trash_open");
    if (item.kind === "folder") return invoke("open_path", { path: item.path });
    return invoke("dock_activate", { key: item.key });
  }

  return <div className="dock-stage" onMouseMove={onMouseMove} onMouseLeave={() => { resetMagnification(); setMenu(null); }}>
    <div ref={pill} className="dock-pill">
      {all.map((item, index) => {
        const icon = item.kind === "apps" ? <AppsIcon /> : item.kind === "trash" ? <span className="trash-icon">{item.running ? "🗑️" : "♲"}</span> : item.icon ? <img src={item.icon} alt="" /> : <ItemIcon path={item.path || (item.aumid ? `shell:AppsFolder\\${item.aumid}` : "")} name={item.name} />;
        const firstShelfIndex = all.findIndex((entry) => entry.kind === "folder" || entry.kind === "trash");
        return <button key={`${item.key}-${index}`} title={item.name} className={`dock-item ${item.kind === "folder" ? "dock-folder" : ""} ${index === firstShelfIndex ? "dock-divider" : ""}`} style={{ width: size, height: size + 8 }} ref={(node) => { itemRefs.current[index] = node; iconRefs.current[index] = node?.querySelector(".dock-icon") ?? null; }} onClick={() => activate(item)} onContextMenu={(e) => { e.preventDefault(); setMenu({ x: e.clientX, y: e.clientY, item }); }} onMouseDown={(e) => { if (e.button === 1) { e.preventDefault(); invoke("dock_close_windows", { key: item.key }); } }}>
          <span className="dock-icon" style={{ width: size, height: size }}>{icon}</span>{item.running && <span className="dock-dot" />}
        </button>;
      })}
    </div>
    {menu && <div className="dock-menu" style={{ left: menu.x, top: menu.y }} onMouseLeave={() => setMenu(null)}>
      {menu.item.kind === "trash" ? <><button onClick={() => { invoke("trash_empty"); setMenu(null); }}>Vaciar papelera</button><button onClick={() => { invoke("trash_open"); setMenu(null); }}>Abrir papelera</button></> : menu.item.kind === "apps" ? <button onClick={() => { invoke("toggle_launchpad"); setMenu(null); }}>Abrir Launchpad</button> : <><button onClick={() => { invoke("dock_activate", { key: menu.item.key }); setMenu(null); }}>Abrir</button>{menu.item.kind === "pinned" && <button onClick={() => { invoke("dock_unpin", { key: menu.item.key }); setMenu(null); }}>Quitar del Dock</button>}{menu.item.kind === "running" && <button onClick={() => { invoke("dock_close_windows", { key: menu.item.key }); setMenu(null); }}>Cerrar ventanas</button>}</>}
    </div>}
  </div>;
}

function fallbackColor(name = "") {
  let hash = 2166136261;
  for (const char of name) hash = Math.imul(hash ^ char.charCodeAt(0), 16777619);
  return `rgb(${70 + ((hash >>> 16) & 127)}, ${70 + ((hash >>> 8) & 127)}, ${70 + (hash & 127)})`;
}

function FallbackIcon({ name }) {
  return <span className="dock-fallback" style={{ background: fallbackColor(name) }}>{(name.trim()[0] || "?").toUpperCase()}</span>;
}

function ItemIcon({ path, name }) {
  const [src, setSrc] = useState(null);
  const [failed, setFailed] = useState(false);
  useEffect(() => {
    setSrc(null); setFailed(false);
    if (path) invoke("app_icon", { path }).then(setSrc).catch(() => setFailed(true));
  }, [path]);
  return src && !failed ? <img src={src} alt="" onError={() => setFailed(true)} /> : <FallbackIcon name={name} />;
}

