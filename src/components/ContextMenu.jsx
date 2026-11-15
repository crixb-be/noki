import { useEffect, useLayoutEffect, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { useSettings } from "../hooks/useSettings.js";
import { useGlass, useGlassShape } from "../lib/glass.js";

const GAP = 6;
const MARGIN = 8;
const clamp = (value, min, max) => Math.min(Math.max(value, min), Math.max(min, max));

/** All levels share one topmost canvas; its bottom edge is strictly above the Dock. */
export default function ContextMenu() {
  const { settings } = useSettings();
  const [menu, setMenu] = useState(null);
  const [path, setPath] = useState([]);
  const [positions, setPositions] = useState([]);
  const root = useRef(null);
  const activeToken = useRef(0);
  const panels = useRef([]);
  const buttons = useRef([]);
  const glass = useGlass(settings, "panels");
  const pathKey = path.join(":");
  useGlassShape(root, glass, [menu?.token, pathKey, JSON.stringify(positions)], "[data-menu-panel]");

  useEffect(() => { document.documentElement.dataset.panels = settings?.appearance?.panelStyle === "dark" ? "dark" : "light"; }, [settings?.appearance?.panelStyle]);
  useEffect(() => {
    let disposed = false;
    let stops = [];
    const apply = (data) => {
      if (disposed || (data && data.token < activeToken.current) || (!data && activeToken.current)) return;
      activeToken.current = data?.token ?? 0;
      setMenu(data); setPath([]); setPositions([]);
    };
    Promise.all([
      listen("context-menu-open", (event) => apply(event.payload)),
      listen("context-menu-focus", (event) => { if (event.payload === activeToken.current) panels.current[0]?.querySelector("button:not(:disabled)")?.focus({ preventScroll: true }); }),
    ]).then(async (unlisten) => {
      if (disposed) { unlisten.forEach((stop) => stop()); return; }
      stops = unlisten;
      apply(await invoke("context_menu_state"));
    }).catch(console.error);
    return () => { disposed = true; stops.forEach((stop) => stop()); };
  }, []);
  useEffect(() => {
    const escape = (event) => { if (event.key === "Escape" && !event.defaultPrevented && menu) close(); };
    window.addEventListener("keydown", escape);
    return () => window.removeEventListener("keydown", escape);
  }, [menu?.token]);

  const levels = [];
  if (menu) {
    let entries = menu.entries;
    levels.push(entries);
    for (const index of path) {
      const children = entries[index]?.children;
      if (!children?.length) break;
      levels.push(children);
      entries = children;
    }
  }

  useLayoutEffect(() => {
    if (!menu) return;
    const layout = () => {
      const width = window.innerWidth;
      const height = window.innerHeight;
      const next = [];
      panels.current.slice(0, levels.length).forEach((node, depth) => {
        if (!node) return;
        const w = node.offsetWidth;
        const h = node.offsetHeight;
        if (depth === 0) {
          next.push({ left: clamp(menu.x - (menu.above ? w / 2 : 0), MARGIN, width - w - MARGIN), top: clamp(menu.above ? height - h - MARGIN : menu.y, MARGIN, height - h - MARGIN) });
        } else {
          const parent = panels.current[depth - 1];
          const anchor = buttons.current[depth - 1]?.[path[depth - 1]];
          const previous = next[depth - 1];
          const rowTop = anchor ? anchor.offsetTop - parent.scrollTop : 0;
          let left = previous.left + parent.offsetWidth + GAP;
          if (left + w + MARGIN > width) left = previous.left - w - GAP;
          next.push({ left: clamp(left, MARGIN, width - w - MARGIN), top: clamp(previous.top + rowTop - 6, MARGIN, height - h - MARGIN) });
        }
      });
      setPositions((previous) => JSON.stringify(previous) === JSON.stringify(next) ? previous : next);
    };
    layout();
    window.addEventListener("resize", layout);
    return () => window.removeEventListener("resize", layout);
  }, [menu, pathKey, levels.length]);

  useLayoutEffect(() => {
    if (!settings || !menu || positions.length !== levels.length || !levels.length) return;
    const regions = panels.current.slice(0, levels.length).filter(Boolean).map((node) => {
      const r = node.getBoundingClientRect(); return { x: r.left, y: r.top, w: r.width, h: r.height };
    });
    invoke("context_menu_present", { token: menu.token, regions }).catch(console.error);
  }, [settings, menu, pathKey, JSON.stringify(positions), levels.length]);

  useEffect(() => {
    if (menu && positions.length) panels.current[0]?.querySelector("button:not(:disabled)")?.focus({ preventScroll: true });
  }, [menu?.token, positions.length > 0]);

  function close() { if (!menu) return; setMenu(null); invoke("context_menu_close", { token: menu.token }).catch(console.error); }
  function select(entry, depth, index, focusChild = false) {
    if (!entry.enabled) return;
    if (entry.children?.length) {
      setPath((current) => [...current.slice(0, depth), index]);
      if (focusChild) requestAnimationFrame(() => panels.current[depth + 1]?.querySelector("button:not(:disabled)")?.focus({ preventScroll: true }));
    } else {
      setMenu(null);
      invoke("context_menu_execute", { token: menu.token, id: entry.id }).catch(console.error);
    }
  }
  function keyboard(event, depth) {
    if (event.key === "Escape") { event.preventDefault(); close(); return; }
    const entries = levels[depth];
    const current = buttons.current[depth] ?? [];
    const active = current.indexOf(document.activeElement);
    const enabled = current.filter((node) => node && !node.disabled);
    if (["ArrowDown", "ArrowUp", "Home", "End"].includes(event.key)) {
      event.preventDefault();
      const index = enabled.indexOf(document.activeElement);
      const next = event.key === "Home" ? 0 : event.key === "End" ? enabled.length - 1 : (index + (event.key === "ArrowDown" ? 1 : -1) + enabled.length) % enabled.length;
      enabled[next]?.focus({ preventScroll: false });
    } else if (event.key === "ArrowRight" && active >= 0) {
      event.preventDefault(); if (entries[active].children?.length) select(entries[active], depth, active, true);
    } else if (event.key === "ArrowLeft" && depth > 0) {
      event.preventDefault(); setPath((current) => current.slice(0, depth - 1));
      buttons.current[depth - 1]?.[path[depth - 1]]?.focus({ preventScroll: true });
    } else if (event.key === "Tab") { event.preventDefault(); close(); }
  }

  return <div ref={root} className="context-menu-stage" onContextMenu={(event) => event.preventDefault()} onPointerDown={(event) => { if (!event.target.closest("[data-menu-panel]")) close(); }}>
    {levels.map((entries, depth) => <div key={`${menu.token}-${depth}`} ref={(node) => { panels.current[depth] = node; }} data-menu-panel role="menu" aria-label={depth ? "Submenú" : "Menú contextual"}
      className="noki-popup-card noki-panel-surface context-menu-panel" style={{ left: positions[depth]?.left ?? MARGIN, top: positions[depth]?.top ?? MARGIN, zIndex: 100 + depth, visibility: positions[depth] ? "visible" : "hidden" }} onKeyDown={(event) => keyboard(event, depth)} onScroll={() => setPath((current) => current.slice(0, depth))}>
      {entries.map((entry, index) => entry.separator ? <div key={`separator-${index}`} className="panel-divider" role="separator" /> : <button key={`${entry.id}-${index}`} ref={(node) => { (buttons.current[depth] ??= [])[index] = node; }} type="button" role="menuitem" disabled={!entry.enabled} className={`panel-row${entry.children?.length && path[depth] === index ? " active" : ""}`} title={entry.label}
        aria-haspopup={entry.children?.length ? "menu" : undefined} aria-expanded={entry.children?.length ? path[depth] === index : undefined}
        onMouseEnter={() => { if (!entry.enabled) return; setPath((current) => entry.children?.length ? [...current.slice(0, depth), index] : current.slice(0, depth)); }}
        onFocus={() => { if (!entry.children?.length) setPath((current) => current.slice(0, depth)); }} onClick={() => select(entry, depth, index, true)}>
        <span className="context-menu-label">{entry.label}</span>{entry.children?.length > 0 && <svg className="context-menu-chevron" viewBox="0 0 16 16" aria-hidden="true"><path d="m6 3 5 5-5 5" fill="none" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" strokeLinejoin="round" /></svg>}
      </button>)}
    </div>)}
  </div>;
}
