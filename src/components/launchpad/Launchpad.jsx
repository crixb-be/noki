import { useEffect, useMemo, useState } from "react";
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
let appsRequest = null;

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
  const [icons, setIcons] = useState({});

  presetLanguage(settings?.general?.language);
  useLanguage(settings?.general?.language);

  useEffect(() => { loadApps().then(setApps).catch(() => {}); }, []);
  useEffect(() => {
    const onKey = (event) => {
      if (event.key === "Escape") { if (query) setQuery(""); else invoke("close_launchpad").catch(() => {}); event.preventDefault(); }
      if (event.key === "Enter" && filtered[0]) launch(filtered[0]);
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [query, category, sort, apps]);

  const filtered = useMemo(() => apps.filter((app) => {
    const text = fold(`${app.name} ${app.path}`);
    const matchesQuery = !fold(query) || text.includes(fold(query));
    const words = WORDS[category];
    const matchesCategory = category === "Todas" || (words && fold(words).split(" ").some((word) => text.includes(word)));
    return matchesQuery && matchesCategory;
  }).sort((a, b) => sort === "az" ? a.name.localeCompare(b.name) : b.name.localeCompare(a.name)), [apps, query, category, sort]);
  const launch = (app) => { invoke("launch_app", { path: app.path }).catch(() => {}); invoke("close_launchpad").catch(() => {}); };

  return <div className="launchpad-backdrop" onMouseDown={(event) => { if (event.target === event.currentTarget) invoke("close_launchpad").catch(() => {}); }}>
    <div className="launchpad-layout">
      <div className="launchpad-head noki-popup-card">
        <label className="launchpad-search"><span className="launchpad-app-mark">A</span><input autoFocus value={query} onChange={(event) => setQuery(event.target.value)} placeholder={isEn() ? "Applications" : "Aplicaciones"} /></label>
        <button className="launchpad-sort" onClick={() => setSort(sort === "az" ? "za" : "az")} aria-label="Ordenar aplicaciones">☷ {sort === "az" ? "A-Z" : "Z-A"}</button>
      </div>
      <div className="launchpad-card noki-popup-card">
        <div className="launchpad-chips">{CATEGORIES.map((item) => <button key={item} className={category === item ? "selected" : ""} onClick={() => setCategory(item)}>{item}</button>)}</div>
        <div className="launchpad-grid">{filtered.length ? filtered.map((app) => <LaunchItem key={`${app.name}-${app.path}`} app={app} icon={icons[app.path]} onIcon={(src) => setIcons((old) => ({ ...old, [app.path]: src }))} onClick={() => launch(app)} onContextMenu={(event) => { event.preventDefault(); if (window.confirm(`Anclar ${app.name} al Dock?`)) invoke("dock_pin", { path: app.path, name: app.name, args: "", aumid: "" }); }} />) : <div className="launchpad-empty">No hay aplicaciones en esta categoría</div>}</div>
      </div>
    </div>
  </div>;
}

function LaunchItem({ app, icon, onIcon, onClick, onContextMenu }) {
  useEffect(() => { if (!icon) invoke("app_icon", { path: app.path }).then(onIcon).catch(() => {}); }, [app.path, icon, onIcon]);
  return <button className="launchpad-app" onClick={onClick} onContextMenu={onContextMenu}>{icon ? <img src={icon} alt="" /> : <span className="launchpad-icon-placeholder">◈</span>}<span title={app.name}>{app.name}</span></button>;
}
