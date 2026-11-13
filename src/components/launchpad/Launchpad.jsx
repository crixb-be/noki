import { memo, useEffect, useMemo, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { useSettings } from "../../hooks/useSettings.js";
import { isEn, presetLanguage, useLanguage } from "../../lib/i18n.js";

const CATEGORIES = ["Todas", "Productividad y finanzas", "Creatividad", "Utilidades", "Información y lectura", "Juegos", "Entretenimiento", "Social"];
const WORDS = {
  "Productividad y finanzas": "word excel powerpoint outlook onenote teams notion calendar calendario correo mail todo pdf office libreoffice obsidian bank finance banco",
  Creatividad: "photoshop illustrator premiere affinity gimp inkscape blender autocad paint capcut canva figma obs audacity",
  Utilidades: "system32 control terminal powershell cmd explorer explorador calculator calculadora notepad configuración settings winrar recortes herramienta monitor driver update backup vpn defender seguridad 7zip",
  "Información y lectura": "reader reading pdf kindle notion obsidian wiki browser docs documentation acrobat edge chrome firefox news prensa revistas book libros help manual calibre zotero",
  Juegos: "game juego steam epic xbox gog riot ubisoft minecraft roblox bluestacks",
  Entretenimiento: "netflix spotify vlc music música video vídeo tv disney youtube plex itunes fotos photos media player",
  Social: "whatsapp telegram discord messenger instagram facebook twitter tiktok skype zoom slack signal line",
};
const fold = (s) => (s ?? "").normalize("NFD").replace(/[\u0300-\u036f]/g, "").toLowerCase();
let cachedApps = null;
// Iconos: caché por ruta, peticiones deduplicadas y como mucho 6 a la vez (evita congelar la primera apertura).
const iconCache = new Map();
const iconWaiters = new Map();
const iconQueue = [];
let iconActive = 0;
function pumpIcons() {
  while (iconActive < 6 && iconQueue.length) {
    const path = iconQueue.shift();
    iconActive += 1;
    invoke("app_icon", { path }).then((src) => src, () => "").then((src) => {
      iconCache.set(path, src);
      (iconWaiters.get(path) ?? []).forEach((fn) => fn(src));
      iconWaiters.delete(path);
      iconActive -= 1;
      pumpIcons();
    });
  }
}
function requestIcon(path, fn) {
  if (iconCache.has(path)) { fn(iconCache.get(path)); return () => {}; }
  if (!iconWaiters.has(path)) { iconWaiters.set(path, []); iconQueue.push(path); pumpIcons(); }
  iconWaiters.get(path).push(fn);
  return () => {
    const list = iconWaiters.get(path);
    if (!list) return;
    const rest = list.filter((x) => x !== fn);
    if (rest.length) { iconWaiters.set(path, rest); return; }
    // Nadie lo espera ya (salió de la vista): se retira de la cola si aún no empezó.
    const queued = iconQueue.indexOf(path);
    if (queued >= 0) { iconQueue.splice(queued, 1); iconWaiters.delete(path); } else iconWaiters.set(path, []);
  };
}
let appsRequest = null;

// Estable entre renders para que `memo` evite repintar toda la rejilla.
function launch(app) { invoke("launch_app", { path: app.path }).catch(() => {}); invoke("close_launchpad").catch(() => {}); }

function loadApps() {
  if (cachedApps) return Promise.resolve(cachedApps);
  if (!appsRequest) appsRequest = invoke("launcher_apps").then((value) => { cachedApps = value ?? []; return cachedApps; }).finally(() => { appsRequest = null; });
  return appsRequest;
}

export default function Launchpad() {
  const { settings } = useSettings();
  const [apps, setApps] = useState(() => cachedApps ?? []);
  const [query, setQuery] = useState("");
  const [category, setCategory] = useState("Todas");
  const [sort, setSort] = useState("az");

  presetLanguage(settings?.general?.language);
  useLanguage(settings?.general?.language);

  useEffect(() => { loadApps().then(setApps).catch(() => {}); }, []);
  const filtered = useMemo(() => apps.filter((app) => {
    const text = fold(`${app.name} ${app.path}`);
    const matchesQuery = !fold(query) || text.includes(fold(query));
    const words = WORDS[category];
    const matchesCategory = category === "Todas" || (words && fold(words).split(" ").some((word) => text.includes(word)));
    return matchesQuery && matchesCategory;
  }).sort((a, b) => sort === "az" ? a.name.localeCompare(b.name) : b.name.localeCompare(a.name)), [apps, query, category, sort]);

  useEffect(() => {
    const onKey = (event) => {
      if (event.key === "Escape") { if (query) setQuery(""); else invoke("close_launchpad").catch(() => {}); event.preventDefault(); }
      if (event.key === "Enter" && filtered[0]) launch(filtered[0]);
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [query, filtered]);

  return <div className="launchpad-backdrop" onMouseDown={(event) => { if (event.target === event.currentTarget) invoke("close_launchpad").catch(() => {}); }}>
    <div className="launchpad-layout launchpad-card noki-popup-card">
      <div className="launchpad-head">
        <label className="launchpad-search"><span className="launchpad-app-mark">A</span><input autoFocus value={query} onChange={(event) => setQuery(event.target.value)} placeholder={isEn() ? "Applications" : "Aplicaciones"} /></label>
        <button className="launchpad-sort" onClick={() => setSort(sort === "az" ? "za" : "az")} aria-label="Ordenar aplicaciones">☷ {sort === "az" ? "A-Z" : "Z-A"}</button>
      </div>
      <>
        <div className="launchpad-chips">{CATEGORIES.map((item) => <button key={item} className={category === item ? "selected" : ""} onClick={() => setCategory(item)}>{item}</button>)}</div>
        <div className="launchpad-grid">{filtered.length ? filtered.map((app) => <LaunchItem key={`${app.name}-${app.path}`} app={app} onLaunch={launch} />) : <div className="launchpad-empty">No hay aplicaciones en esta categoría</div>}</div>
      </>
    </div>
  </div>;
}

const LaunchItem = memo(function LaunchItem({ app, onLaunch }) {
  const ref = useRef(null);
  const [icon, setIcon] = useState(() => iconCache.get(app.path) ?? null);
  useEffect(() => {
    if (icon !== null && iconCache.has(app.path)) return undefined;
    let cancel = () => {};
    // Solo se piden los iconos que están (o van a estar) a la vista.
    const observer = new IntersectionObserver(([entry]) => {
      if (!entry.isIntersecting) return;
      observer.disconnect();
      cancel = requestIcon(app.path, setIcon);
    }, { rootMargin: "200px" });
    if (ref.current) observer.observe(ref.current);
    return () => { observer.disconnect(); cancel(); };
  }, [app.path]);
  const pin = (event) => {
    event.preventDefault();
    if (window.confirm(`Anclar ${app.name} al Dock?`)) invoke("dock_pin", { path: app.path, name: app.name, args: "", aumid: "" }).catch(() => {});
  };
  return <button ref={ref} className="launchpad-app" onClick={() => onLaunch(app)} onContextMenu={pin}>{icon ? <img src={icon} alt="" draggable="false" /> : <span className="launchpad-icon-placeholder" />}<span title={app.name}>{app.name}</span></button>;
});