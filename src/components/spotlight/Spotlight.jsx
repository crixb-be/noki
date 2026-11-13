import { useEffect, useMemo, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";

export default function Spotlight() {
  const [query, setQuery] = useState("");
  const [results, setResults] = useState([]);
  const [selected, setSelected] = useState(0);
  const input = useRef(null);
  const requestId = useRef(0);
  useEffect(() => {
    input.current?.focus();
    const id = ++requestId.current;
    const timer = setTimeout(() => invoke("spotlight_search", { query }).then((value) => {
      if (id === requestId.current) setResults(value ?? []);
    }).catch(() => {}), 150);
    return () => clearTimeout(timer);
  }, [query]);
  useEffect(() => setSelected(0), [results]);
  useEffect(() => { invoke("spotlight_resize", { height: Math.min(460, 118 + results.length * 50) }).catch(() => {}); }, [results.length]);
  const apps = useMemo(() => results.filter((item) => item.kind === "app"), [results]);
  const files = useMemo(() => results.filter((item) => item.kind === "file"), [results]);
  const flat = [...apps, ...files];
  const choose = (item, reveal = false) => {
    if (!item) return;
    const action = reveal && item.kind === "file" ? invoke("reveal_in_folder", { path: item.path }) : item.kind === "app" ? invoke("launch_app", { path: item.path }) : invoke("open_path", { path: item.path });
    action.finally(() => invoke("close_spotlight"));
  };
  const onKey = (event) => {
    if (event.key === "Escape") { if (query) setQuery(""); else invoke("close_spotlight"); event.preventDefault(); }
    if (event.key === "ArrowDown") { setSelected((i) => Math.min(i + 1, Math.max(0, flat.length - 1))); event.preventDefault(); }
    if (event.key === "ArrowUp") { setSelected((i) => Math.max(i - 1, 0)); event.preventDefault(); }
    if (event.key === "Enter") { choose(flat[selected], event.ctrlKey); event.preventDefault(); }
  };
  return <div className="spotlight-shell" onKeyDown={onKey}><div className="spotlight-card noki-popup-card">
    <div className="spotlight-input"><span>⌕</span><input ref={input} value={query} onChange={(event) => setQuery(event.target.value)} placeholder="Buscar en Noki" autoFocus /></div>
    <div className="spotlight-results">
      {apps.length > 0 && <ResultGroup title="Aplicaciones" items={apps} offset={0} selected={selected} onClick={choose} />}
      {files.length > 0 && <ResultGroup title="Archivos y carpetas" items={files} offset={apps.length} selected={selected} onClick={choose} />}
      {!flat.length && <p className="spotlight-empty">Escribe para buscar aplicaciones, archivos y carpetas</p>}
    </div>
  </div></div>;
}

function ResultGroup({ title, items, offset, selected, onClick }) {
  return <section><h2>{title}</h2>{items.map((item, index) => <Result key={`${item.kind}-${item.path}`} item={item} active={selected === offset + index} onClick={() => onClick(item)} />)}</section>;
}

function Result({ item, active, onClick }) {
  const [icon, setIcon] = useState(null);
  const [failed, setFailed] = useState(false);
  useEffect(() => { setIcon(null); setFailed(false); invoke("app_icon", { path: item.path }).then(setIcon).catch(() => setFailed(true)); }, [item.path]);
  const fallback = <span className="spotlight-fallback" style={{ background: fallbackColor(item.name) }}>{(item.name.trim()[0] || "?").toUpperCase()}</span>;
  return <button className={`spotlight-result ${active ? "active" : ""}`} onClick={onClick}><span className="spotlight-icon">{icon && !failed ? <img src={icon} alt="" onError={() => setFailed(true)} /> : fallback}</span><span className="spotlight-label"><strong>{item.name}</strong><small>{item.path}</small></span></button>;
}

function fallbackColor(name = "") {
  let hash = 2166136261;
  for (const char of name) hash = Math.imul(hash ^ char.charCodeAt(0), 16777619);
  return `rgb(${70 + ((hash >>> 16) & 127)}, ${70 + ((hash >>> 8) & 127)}, ${70 + (hash & 127)})`;
}
