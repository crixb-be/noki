import { useEffect, useMemo, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { useSettings } from "../../hooks/useSettings.js";

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
  const size = Math.min(46, Math.max(44, dock.iconSize || 44));
  const maxSize = 62;

  const refresh = async () => {
    try { if (!(await getCurrentWindow().isVisible())) return; } catch { /* El navegador de desarrollo no expone visibilidad nativa. */ }
    invoke("dock_items").then(setItems).catch(() => {});
    invoke("trash_state").then(setTrash).catch(() => {});
  };
  useEffect(() => { refresh(); const timer = setInterval(refresh, 1000); return () => clearInterval(timer); }, []);
  useEffect(() => { let stops = []; Promise.all([listen("settings-updated", refresh), listen("settings-changed", refresh), listen("dock-windows", (event) => setItems(event.payload ?? []))]).then((fns) => { stops = fns; }); return () => stops.forEach((stop) => stop()); }, []);
  useEffect(() => {
    if (!pill.current) return;
    const report = () => {
      const r = pill.current?.getBoundingClientRect();
      if (!r) return;
      invoke("dock_set_hit", { x: Math.round(r.left), y: Math.round(r.top), w: Math.round(r.width), h: Math.round(r.height) }).catch(() => {});
      invoke("dock_resize", { width: Math.ceil(r.width + 16), height: Math.ceil(Math.max(r.height + 8, maxSize + 16)) }).catch(() => {});
    };
    report();
    const observer = new ResizeObserver(report);
    observer.observe(pill.current);
    return () => observer.disconnect();
  }, [items, dock.iconSize, maxSize]);

  const all = useMemo(() => [ ...(dock.showApps ? [{ key: "apps", name: "Aplicaciones", kind: "apps" }] : []), ...items, ...(dock.showTrash ? [{ key: "trash", name: "Papelera", kind: "trash", running: trash.items > 0 }] : []) ], [dock.showApps, dock.showTrash, items, trash.items]);
  if (settings && !dock.enabled) return null;
  function resetMagnification() {
    pointerX.current = null;
    iconRefs.current.forEach((node) => {
      if (!node) return;
      node.style.transition = "width 180ms ease-out, height 180ms ease-out";
      node.style.width = `${size}px`;
      node.style.height = `${size}px`;
      node.style.zIndex = "";
    });
    itemRefs.current.forEach((node) => {
      if (!node) return;
      node.style.transition = "width 180ms ease-out, height 180ms ease-out";
      node.style.width = `${size}px`;
      node.style.height = `${size + 8}px`;
    });
  }

  function updateMagnification() {
    frame.current = 0;
    const x = pointerX.current;
    if (x == null || !dock.magnify) { resetMagnification(); return; }
    itemRefs.current.forEach((item, index) => {
      const icon = iconRefs.current[index];
      if (!item || !icon) return;
      const rect = item.getBoundingClientRect();
      const distance = Math.abs(x - (rect.left + rect.width / 2));
      const ratio = Math.min(1, distance / (size * 3));
      const itemSize = size + (maxSize - size) * Math.cos(ratio * Math.PI / 2);
      item.style.transition = "none";
      item.style.width = `${itemSize}px`;
      item.style.height = `${itemSize + 8}px`;
      icon.style.transition = "none";
      icon.style.width = `${itemSize}px`;
      icon.style.height = `${itemSize}px`;
      item.style.zIndex = String(Math.round(itemSize));
    });
  }

  function onMouseMove(event) {
    pointerX.current = event.clientX;
    if (!frame.current) frame.current = requestAnimationFrame(updateMagnification);
  }
  function activate(item) {
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

