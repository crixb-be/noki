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
  const menuRef = useRef(null);
  const lastMenuSpace = useRef(0);
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
  // Tras un recorte de pantalla (Impr Pant, Recortes) el backend pide reiniciar el estado de la vista.
  const recover = useRef(null);

  const refresh = async () => {
    try { if (!(await getCurrentWindow().isVisible())) return; } catch { /* El navegador de desarrollo no expone visibilidad nativa. */ }
    invoke("dock_items").then(setItems).catch(() => {});
    invoke("trash_state").then(setTrash).catch(() => {});
  };
  useEffect(() => { refresh(); const timer = setInterval(refresh, 1000); return () => clearInterval(timer); }, []);
  useEffect(() => { let stops = []; Promise.all([listen("settings-updated", refresh), listen("settings-changed", refresh), listen("dock-windows", (event) => setItems(event.payload ?? [])), listen("dock-recover", () => recover.current?.())]).then((fns) => { stops = fns; }); return () => stops.forEach((stop) => stop()); }, []);
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
      let box = { left: r.left - lift * 2, top: r.top - lift, right: r.right + lift * 2, bottom: r.bottom };
      // Con el menú contextual abierto, el área de clic también lo cubre (si no, era click-through).
      const m = menuRef.current?.getBoundingClientRect();
      if (m) box = { left: Math.min(box.left, m.left), top: Math.min(box.top, m.top), right: Math.max(box.right, m.right), bottom: box.bottom };
      invoke("dock_set_hit", { x: Math.round(box.left), y: Math.round(box.top), w: Math.round(box.right - box.left), h: Math.round(box.bottom - box.top) }).catch(() => {});
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
  // Anclada abajo: si la ventana crece hacia arriba (menú contextual), el vidrio no se mueve.
  useGlassShape(pill, glass, [], null, true);
  // Tamaño de la ventana: ancho natural de la píldora en reposo + holgura para la magnificación.
  // Antes la píldora se limitaba al ancho de la ventana y la ventana a la píldora: los iconos se recortaban.
  // Con el menú contextual abierto la ventana crece hacia arriba lo que mide el menú (la píldora no
  // se mueve: el borde inferior de la ventana queda fijo). Antes el menú se abría hacia abajo dentro
  // de una ventana de ~86 px y se cortaba en el borde de la pantalla.
  const menuSpace = menu ? menuHeight(menuActions(menu.item)) + MENU_GAP + 6 : 0;
  useEffect(() => {
    const node = pill.current;
    if (!node) return;
    const menuChanged = lastMenuSpace.current !== menuSpace;
    if (pointerX.current != null && !menuChanged) return;
    lastMenuSpace.current = menuSpace;
    const width = Math.ceil(Math.max(node.scrollWidth + (dock.magnify ? (maxSize - size) * 4 : 0) + 24, menu ? MENU_WIDTH + 16 : 0));
    const height = Math.ceil(maxSize + 8 + 24 + menuSpace);
    invoke("dock_resize", { width, height }).catch(() => {});
  }, [all.length, size, maxSize, dock.magnify, menuSpace]);
  useEffect(() => {
    if (!menu) { reportHit.current?.(); return undefined; }
    const frame = requestAnimationFrame(() => reportHit.current?.());
    const onResize = () => reportHit.current?.();
    window.addEventListener("resize", onResize);
    return () => { cancelAnimationFrame(frame); window.removeEventListener("resize", onResize); };
  }, [menu]);

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
    // Con el menú abierto no se magnifica (los iconos ampliados lo tapaban).
    if (menu) return;
    const first = pointerX.current == null;
    if (first) rest.current = measureRest();
    pointerX.current = event.clientX;
    if (first) reportHit.current?.();
    start();
  }  recover.current = () => { resetMagnification(); reportHit.current?.(); refresh(); };
  function activate(item) {
    if (item.kind === "apps") return invoke("toggle_launchpad");
    if (item.kind === "trash") return invoke("trash_open");
    if (item.kind === "folder") return invoke("open_path", { path: item.path });
    return invoke("dock_activate", { key: item.key });
  }

  function openMenu(event, item) {
    resetMagnification();
    // Encima de la píldora, centrado en el icono; medido desde abajo porque la ventana va a crecer.
    const box = pill.current?.getBoundingClientRect();
    const bottom = box ? window.innerHeight - box.top + MENU_GAP : 80;
    setMenu({ x: event.clientX, bottom, item });
  }
  const actions = menu ? menuActions(menu.item) : [];
  const menuLeft = menu ? Math.round(Math.min(Math.max(4, menu.x - MENU_WIDTH / 2), Math.max(4, window.innerWidth - MENU_WIDTH - 4))) : 0;

  return <div className="dock-stage" onMouseMove={onMouseMove} onMouseLeave={() => { resetMagnification(); setMenu(null); }} onMouseDown={(e) => { if (menu && !menuRef.current?.contains(e.target)) setMenu(null); }}>
    <div ref={pill} className="dock-pill">
      {all.map((item, index) => {
        const icon = item.kind === "apps" ? <AppsIcon /> : item.kind === "trash" ? <span className="trash-icon">{item.running ? "🗑️" : "♲"}</span> : item.icon ? <img src={item.icon} alt="" /> : <ItemIcon path={item.path || (item.aumid ? `shell:AppsFolder\\${item.aumid}` : "")} name={item.name} />;
        const firstShelfIndex = all.findIndex((entry) => entry.kind === "folder" || entry.kind === "trash");
        return <button key={`${item.key}-${index}`} title={item.name} className={`dock-item ${item.kind === "folder" ? "dock-folder" : ""} ${index === firstShelfIndex ? "dock-divider" : ""}`} style={{ width: size, height: size + 8 }} ref={(node) => { itemRefs.current[index] = node; iconRefs.current[index] = node?.querySelector(".dock-icon") ?? null; }} onClick={() => activate(item)} onContextMenu={(e) => { e.preventDefault(); openMenu(e, item); }} onMouseDown={(e) => { if (e.button === 1) { e.preventDefault(); invoke("dock_close_windows", { key: item.key }); } }}>
          <span className="dock-icon" style={{ width: size, height: size }}>{icon}</span>{item.running && <span className="dock-dot" />}
        </button>;
      })}
    </div>
    {menu && <div ref={menuRef} className="dock-menu" role="menu" style={{ left: menuLeft, bottom: menu.bottom, width: MENU_WIDTH }}>
      {actions.map((a) => <button key={a.label} role="menuitem" className={a.danger ? "danger" : ""} onClick={() => { setMenu(null); a.run(); }}>{a.label}</button>)}
    </div>}
  </div>;
}

const MENU_WIDTH = 200;
const MENU_ROW = 30;
const MENU_PAD = 12;
/** Separación entre la píldora y el menú. */
const MENU_GAP = 8;
const menuHeight = (actions) => actions.length * MENU_ROW + MENU_PAD;

/** Opciones del menú contextual de cada elemento del Dock. */
function menuActions(item) {
  const run = (cmd, args) => () => invoke(cmd, args).catch(() => {});
  if (item.kind === "trash") return [{ label: "Abrir papelera", run: run("trash_open") }, { label: "Vaciar papelera", run: run("trash_empty"), danger: true }];
  if (item.kind === "apps") return [{ label: "Abrir Launchpad", run: run("toggle_launchpad") }];
  if (item.kind === "folder") return [{ label: "Abrir", run: run("open_path", { path: item.path }) }];
  const list = [{ label: "Abrir", run: run("dock_activate", { key: item.key }) }];
  if (item.path || item.aumid) list.push({ label: "Nueva ventana", run: run("dock_launch", { path: item.path ?? "", args: item.args ?? "", aumid: item.aumid ?? "" }) });
  if (item.kind === "pinned") list.push({ label: "Quitar del Dock", run: run("dock_unpin", { key: item.key }) });
  else list.push({ label: "Anclar al Dock", run: run("dock_pin", { path: item.path ?? "", name: item.name ?? "", args: "", aumid: item.aumid ?? "" }) });
  if (item.running) list.push({ label: (item.windowCount ?? 1) > 1 ? `Cerrar ${item.windowCount} ventanas` : "Cerrar programa", run: run("dock_close_windows", { key: item.key }), danger: true });
  return list;
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

