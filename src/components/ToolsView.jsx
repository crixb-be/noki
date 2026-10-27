import { useEffect, useMemo, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { useStore } from "../hooks/useStore.js";
import AppIcon from "./AppIcon.jsx";
import TranslatorView from "./TranslatorView.jsx";

const chip = "rounded-full bg-white/10 px-3 py-1 text-xs transition-all duration-300 hover:bg-white/20 active:scale-95";
const inputCls = "min-w-0 flex-1 rounded-full bg-white/10 px-3 py-1.5 text-xs outline-none placeholder:text-white/30 focus:bg-white/15";

const ago = (ms) => {
  const m = Math.floor((Date.now() - ms) / 60000);
  if (m < 1) return "ahora";
  if (m < 60) return `hace ${m} min`;
  const h = Math.floor(m / 60);
  return h < 24 ? `hace ${h} h` : `hace ${Math.floor(h / 24)} d`;
};

// ───────────── Portapapeles ─────────────

function Clipboard({ clip, update }) {
  const [items, setItems] = useState([]);
  const [pins, setPins] = useStore("clipPins", []);
  const [q, setQ] = useState("");
  const [copied, setCopied] = useState(null);

  useEffect(() => {
    if (!clip.enabled) return setItems([]);
    let unlisten;
    let cancelled = false;
    invoke("clip_list").then((l) => !cancelled && setItems(l)).catch(() => {});
    listen("clip-changed", (e) => setItems(e.payload)).then((fn) => (cancelled ? fn() : (unlisten = fn)));
    return () => {
      cancelled = true;
      unlisten?.();
    };
  }, [clip.enabled]);

  const copy = async (text, key) => {
    await invoke("clip_copy", { text }).catch(() => {});
    setCopied(key);
    setTimeout(() => setCopied(null), 1200);
  };
  const pin = (text) => setPins((p) => (p.some((x) => x.text === text) ? p : [{ id: Date.now(), text }, ...p].slice(0, 20)));
  const unpin = (id) => setPins((p) => p.filter((x) => x.id !== id));

  const filter = (list) => (q.trim() ? list.filter((x) => x.text.toLowerCase().includes(q.trim().toLowerCase())) : list);
  const shownItems = useMemo(() => filter(items).filter((i) => !pins.some((p) => p.text === i.text)), [items, pins, q]); // eslint-disable-line react-hooks/exhaustive-deps
  const shownPins = useMemo(() => filter(pins), [pins, q]); // eslint-disable-line react-hooks/exhaustive-deps

  if (!clip.enabled) {
    return (
      <div className="flex h-full flex-col items-center justify-center gap-2 rounded-3xl bg-white/5 px-10 text-center">
        <span className="text-3xl">📋</span>
        <span className="text-sm text-white/70">Historial del portapapeles</span>
        <span className="text-xs leading-relaxed text-white/35">
          Guarda lo que copias para volver a usarlo. Solo texto y solo en memoria (se borra al cerrar la app);
          ignora lo que copian los gestores de contraseñas. Está desactivado porque puede capturar datos sensibles.
        </span>
        <button className={`${chip} mt-1`} onClick={() => update({ clipboard: { enabled: true } })}>Activar historial</button>
      </div>
    );
  }

  const renderRow = ({ text, at, id, pinned }) => (
    <li key={`${pinned ? "p" : "i"}${id}`} className="group flex items-start gap-2 rounded-2xl bg-white/5 px-3 py-1.5 text-xs transition-colors hover:bg-white/10">
      <button onClick={() => copy(text, id)} className="min-w-0 flex-1 text-left" title="Clic para copiar de nuevo">
        <span className="line-clamp-2 break-words">{text}</span>
        <span className="mt-0.5 block text-[10px] text-white/30">{copied === id ? "✓ Copiado" : at ? ago(at) : "Fijado"}</span>
      </button>
      <div className="flex shrink-0 gap-1 opacity-0 transition-opacity group-hover:opacity-100">
        <button onClick={() => (pinned ? unpin(id) : pin(text))} className="rounded-full px-1.5 hover:bg-white/20" title={pinned ? "Quitar de fijados" : "Fijar"}>
          {pinned ? "📌" : "📍"}
        </button>
        {!pinned && (
          <button onClick={() => invoke("clip_remove", { id })} className="rounded-full px-1.5 hover:bg-red-500/40" title="Quitar">✕</button>
        )}
      </div>
    </li>
  );

  return (
    <div className="flex h-full flex-col gap-2 rounded-3xl bg-white/5 p-3">
      <div className="flex shrink-0 gap-1.5">
        <input value={q} onChange={(e) => setQ(e.target.value)} placeholder="Buscar en el historial…" className={inputCls} />
        {items.length > 0 && <button className={chip} onClick={() => invoke("clip_clear")}>Borrar</button>}
      </div>
      <ul className="flex min-h-0 flex-1 flex-col gap-1 overflow-y-auto pr-1">
        {shownPins.map((p) => renderRow({ id: p.id, text: p.text, pinned: true }))}
        {shownItems.map((i) => renderRow({ id: i.id, text: i.text, at: i.at }))}
        {shownPins.length + shownItems.length === 0 && (
          <li className="py-6 text-center text-xs text-white/30">{q ? "Sin coincidencias" : "Copia algo (Ctrl+C) y aparecerá aquí"}</li>
        )}
      </ul>
    </div>
  );
}

// ───────────── Lanzador ─────────────


function Launcher() {
  const [apps, setApps] = useState(null);
  const [favs, setFavs] = useStore("launcherFavs", []);
  const [q, setQ] = useState("");
  const [error, setError] = useState(null);

  useEffect(() => {
    invoke("launcher_apps").then(setApps).catch(() => setApps([]));
  }, []);

  const launch = async (app) => {
    try {
      await invoke("launch_app", { path: app.path });
      invoke("request_open", { open: false, sticky: false }).catch(() => {});
    } catch (e) {
      setError(String(e));
      setTimeout(() => setError(null), 2500);
    }
  };
  const isFav = (app) => favs.some((f) => f.path === app.path);
  const toggleFav = (app) =>
    setFavs((l) => (l.some((f) => f.path === app.path) ? l.filter((f) => f.path !== app.path) : [...l, { name: app.name, path: app.path }].slice(0, 12)));

  const query = q.trim().toLowerCase();
  const results = useMemo(() => {
    if (!apps) return [];
    const list = query ? apps.filter((a) => a.name.toLowerCase().includes(query)) : apps;
    // Los que empiezan por la búsqueda van primero.
    return [...list].sort((a, b) => Number(!a.name.toLowerCase().startsWith(query)) - Number(!b.name.toLowerCase().startsWith(query))).slice(0, 40);
  }, [apps, query]);

  const onKey = (e) => e.key === "Enter" && results[0] && launch(results[0]);

  return (
    <div className="flex h-full flex-col gap-2 rounded-3xl bg-white/5 p-3">
      <input
        value={q}
        onChange={(e) => setQ(e.target.value)}
        onKeyDown={onKey}
        placeholder={apps ? `Buscar entre ${apps.length} programas… (Enter abre el primero)` : "Cargando programas…"}
        className={`${inputCls} flex-none`}
      />
      {favs.length > 0 && !query && (
        <div className="flex shrink-0 flex-wrap gap-1.5">
          {favs.map((f) => (
            <button key={f.path} onClick={() => launch(f)} title={`${f.name} (clic derecho: quitar)`} onContextMenu={(e) => { e.preventDefault(); toggleFav(f); }}
              className="flex items-center gap-1.5 rounded-full bg-white/10 py-1 pl-1 pr-3 text-xs transition-colors hover:bg-white/20">
              <AppIcon app={f} size={24} />
              <span className="max-w-[90px] truncate">{f.name}</span>
            </button>
          ))}
        </div>
      )}
      <ul className="grid min-h-0 flex-1 grid-cols-2 content-start gap-1 overflow-y-auto pr-1">
        {results.map((a) => (
          <li key={a.path} className="group flex items-center gap-2 rounded-xl px-2 py-1 text-xs transition-colors hover:bg-white/10">
            <button onClick={() => launch(a)} className="flex min-w-0 flex-1 items-center gap-2 text-left">
              <AppIcon app={a} size={26} />
              <span className="truncate">{a.name}</span>
            </button>
            <button onClick={() => toggleFav(a)} className={`shrink-0 rounded-full px-1 transition-opacity hover:bg-white/20 ${isFav(a) ? "" : "opacity-0 group-hover:opacity-100"}`} title={isFav(a) ? "Quitar de favoritos" : "Añadir a favoritos"}>
              {isFav(a) ? "★" : "☆"}
            </button>
          </li>
        ))}
        {apps && results.length === 0 && <li className="col-span-2 py-6 text-center text-xs text-white/30">Sin resultados</li>}
      </ul>
      {error && <div className="shrink-0 text-[11px] text-amber-300">{error}</div>}
    </div>
  );
}

// ───────────── contenedor ─────────────

const SUBTABS = [
  { id: "launcher", label: "Lanzador", icon: "🚀" },
  { id: "clip", label: "Portapapeles", icon: "📋" },
  { id: "translate", label: "Traductor", icon: "🌐" },
];

export default function ToolsView({ clip, update }) {
  const [sub, setSub] = useStore("toolsTab", "launcher");
  return (
    <div className="flex h-full flex-col gap-2">
      <div className="flex shrink-0 gap-1">
        {SUBTABS.map((t) => (
          <button
            key={t.id}
            onClick={() => setSub(t.id)}
            className={`rounded-full px-3 py-1 text-[11px] transition-all duration-300 ${sub === t.id ? "bg-white/20" : "text-white/45 hover:text-white/80"}`}
          >
            {t.icon} {t.label}
          </button>
        ))}
      </div>
      <div className="min-h-0 flex-1">
        {sub === "launcher" && <Launcher />}
        {sub === "clip" && <Clipboard clip={clip} update={update} />}
        {sub === "translate" && <TranslatorView />}
      </div>
    </div>
  );
}
