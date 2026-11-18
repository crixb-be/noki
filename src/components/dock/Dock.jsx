import { useEffect, useMemo, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";
import TrashIcon from "./TrashIcon.jsx";
import { useSettings } from "../../hooks/useSettings.js";
import { useGlass, useGlassShape } from "../../lib/glass.js";

function physicalBounds(node) {
  const { left, top, right, bottom } = node.getBoundingClientRect();
  const px = (value) => Math.round(value * window.devicePixelRatio);
  return { left: px(left), top: px(top), right: px(right), bottom: px(bottom) };
}

function AppsIcon() { return <span className="apps-icon">{Array.from({ length: 9 }, (_, i) => <i key={i} />)}</span>; }

export default function Dock() {
  const { settings } = useSettings();
  const [items, setItems] = useState([]);
  const [trash, setTrash] = useState({ items: 0 });
  const [menuOpen, setMenuOpen] = useState(false);
  const [dropTarget, setDropTarget] = useState(null);
  const [dropError, setDropError] = useState(null);
  const hoverTimer = useRef(0);
  const pill = useRef(null);
  const itemRefs = useRef([]);
  const iconRefs = useRef([]);
  const pointerX = useRef(null);
  const frame = useRef(0);
  const dock = settings?.dock ?? { enabled: true, iconSize: 44, magnify: true, magnifyScale: 1.6, showApps: true, showRunning: true, showTrash: true, autohide: false, pinned: [], folders: [] };
  // Tamaño y escala vienen de Ajustes (antes estaban fijos: 44-46 px y 62 px máximo).
  // «Tamaño del Dock» (Ajustes): tamaño base de los iconos, 36-64 px. El relleno y el hueco de la
  // píldora escalan con él (variables CSS), así que la píldora, el área de clic y el vidrio cambian juntos.
  const size = Math.min(64, Math.max(36, Math.round(dock.iconSize || 44)));
  const pillVars = { "--dock-gap": `${Math.round(size * 0.17)}px`, "--dock-pad-y": `${Math.max(3, Math.round(size * 0.09))}px`, "--dock-pad-x": `${Math.round(size * 0.18)}px` };
  const scale = dock.magnify ? Math.min(2.5, Math.max(1, Number(dock.magnifyScale) || 1.6)) : 1;
  const maxSize = Math.round(size * scale);
  const sizes = useRef([]);
  const rest = useRef(null);
  const geometry = useRef({ lift: 0 });
  const reportHit = useRef(null);
  // Tras un recorte de pantalla (Impr Pant, Recortes) el backend pide reiniciar el estado de la vista.
  const recover = useRef(null);
  const leave = useRef(null);

  const refresh = async () => {
    try { if (!(await getCurrentWindow().isVisible())) return; } catch { /* El navegador de desarrollo no expone visibilidad nativa. */ }
    invoke("dock_items").then(setItems).catch(() => {});
    invoke("trash_state").then(setTrash).catch(() => {});
  };
  useEffect(() => { refresh(); const timer = setInterval(refresh, 1000); return () => clearInterval(timer); }, []);
  useEffect(() => {
    const subscriptions = [
      listen("context-menu-visibility", (event) => setMenuOpen(!!event.payload)),
      listen("settings-updated", refresh),
      listen("settings-changed", refresh),
      listen("dock-windows", (event) => setItems(event.payload ?? [])),
      listen("dock-recover", () => recover.current?.()),
      listen("dock-leave", () => leave.current?.()),
      listen("dock-file-hover", ({ payload }) => {
        setDropTarget(payload);
        if (payload) {
          clearTimeout(hoverTimer.current);
          invoke("dock_preview_close").catch(console.error);
        }
      }),
      listen("dock-file-result", ({ payload }) => { setDropError(payload); refresh(); }),
    ];
    return () => {
      clearTimeout(hoverTimer.current);
      subscriptions.forEach((subscription) => subscription.then((stop) => stop()).catch(console.error));
    };
  }, []);
  useEffect(() => {
    if (!dropError) return;
    const timer = setTimeout(() => setDropError(null), 3500);
    return () => clearTimeout(timer);
  }, [dropError]);
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
      const icons = itemRefs.current.flatMap((item) => item?.dataset.dropKey
        ? [{ key: item.dataset.dropKey, ...physicalBounds(item) }]
        : []);
      invoke("dock_drop_targets", { icons }).catch(console.error);
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
  useEffect(() => { reportHit.current?.(); }, [all, size]);
  // Desenfoque real debajo de la píldora. La píldora no cambia de tamaño al magnificar, así que el
  // vidrio nativo no se recoloca en cada fotograma (era la franja clara que asomaba encima del Dock).
  const glass = useGlass(settings, "dock");
  // Anclada abajo: si la ventana crece hacia arriba (menú contextual), el vidrio no se mueve.
  useGlassShape(pill, glass, [], null, true);
  // Context menus have their own topmost window, so the Dock never grows or clips them.
  useEffect(() => {
    const node = pill.current;
    if (!node || pointerX.current != null) return;
    const width = Math.ceil(node.scrollWidth + (dock.magnify ? (maxSize - size) * 4 : 0) + 24);
    invoke("dock_resize", { width, height: Math.ceil(maxSize + 32) }).catch(() => {});
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
    else reportHit.current?.();
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
    if (menuOpen) return;
    const first = pointerX.current == null;
    if (first) rest.current = measureRest();
    pointerX.current = event.clientX;
    if (first) reportHit.current?.();
    start();
  }  recover.current = () => { resetMagnification(); reportHit.current?.(); refresh(); };
  leave.current = () => resetMagnification();
  function activate(item) {
    if (item.kind === "apps") return invoke("toggle_launchpad");
    if (item.kind === "trash") return invoke("trash_open");
    if (item.kind === "folder") return invoke("open_path", { path: item.path });
    return invoke("dock_activate", { key: item.key });
  }

  function closePreview() {
    clearTimeout(hoverTimer.current);
    invoke("dock_preview_close").catch(console.error);
  }

  function schedulePreview(event, item) {
    clearTimeout(hoverTimer.current);
    if (event.buttons || menuOpen || dropTarget) return;
    const node = event.currentTarget;
    hoverTimer.current = setTimeout(() => {
      if (node.isConnected) {
        const icon = node.querySelector(".dock-icon") ?? node;
        const a = physicalBounds(node);
        const b = physicalBounds(icon);
        const anchor = { left: Math.min(a.left, b.left), top: Math.min(a.top, b.top), right: Math.max(a.right, b.right), bottom: Math.max(a.bottom, b.bottom) };
        invoke("dock_preview_open", { key: item.key, name: item.name, anchor }).catch(console.error);
      }
    }, 250);
  }

  function openMenu(event, item) {
    event.stopPropagation();
    closePreview();
    resetMagnification();
    invoke("dock_context_menu", { key: item.key }).catch(console.error);
  }

  return <div className="dock-stage" onContextMenu={(e) => { e.preventDefault(); resetMagnification(); invoke("desktop_context_menu", { section: "dock" }).catch(console.error); }} onMouseMove={onMouseMove} onMouseLeave={resetMagnification}>
    {dropError && <div role="status" className="dock-drop-error">{dropError}</div>}
    <div ref={pill} className="dock-pill" style={pillVars}>
      {all.map((item, index) => {
        const icon = item.kind === "apps" ? <AppsIcon /> : item.kind === "trash" ? <TrashIcon full={trash.items > 0} /> : item.icon ? <img src={item.icon} alt="" /> : <ItemIcon path={item.path || (item.aumid ? `shell:AppsFolder\\${item.aumid}` : "")} name={item.name} />;
        const firstShelfIndex = all.findIndex((entry) => entry.kind === "folder" || entry.kind === "trash");
        return <button key={item.key} aria-label={item.name} data-drop-key={item.kind === "apps" ? undefined : item.key} className={`dock-item ${dropTarget === item.key ? "dock-drop-target" : ""} ${item.kind === "folder" ? "dock-folder" : ""} ${index === firstShelfIndex ? "dock-divider" : ""}`} style={{ width: size, height: size + 8 }} ref={(node) => { itemRefs.current[index] = node; iconRefs.current[index] = node?.querySelector(".dock-icon") ?? null; }} onMouseEnter={(event) => schedulePreview(event, item)} onMouseLeave={() => clearTimeout(hoverTimer.current)} onClick={() => { closePreview(); activate(item); }} onContextMenu={(e) => { e.preventDefault(); openMenu(e, item); }} onMouseDown={(e) => { if (e.button === 1) { e.preventDefault(); invoke("dock_close_windows", { key: item.key }); } }}>
          <span className="dock-icon" style={{ width: size, height: size }}>{icon}</span>{item.running && item.kind !== "trash" && <span className="dock-dot" />}
        </button>;
      })}
    </div>
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

