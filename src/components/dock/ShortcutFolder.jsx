import { useEffect, useMemo, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { useSettings } from "../../hooks/useSettings.js";
import { useGlass, useGlassShape } from "../../lib/glass.js";
import { useLanguage } from "../../lib/i18n.js";
import FolderIcon from "./FolderIcon.jsx";

export default function ShortcutFolder() {
  const { settings } = useSettings();
  const [contents, setContents] = useState(null);
  const [error, setError] = useState(null);
  const [busy, setBusy] = useState(false);
  const panel = useRef(null);
  const list = useRef(null);
  const generation = useRef(0);
  const language = settings?.general?.language;
  const glass = useGlass(settings, "panels");
  useGlassShape(panel, glass);
  useLanguage(language);

  useEffect(() => {
    let disposed = false;
    async function load() {
      const request = ++generation.current;
      setContents(null);
      setError(null);
      setBusy(false);
      if (list.current) list.current.scrollTop = 0;
      try {
        const result = await invoke("shortcut_folder_contents");
        if (!disposed && generation.current === request) setContents(result);
      } catch (failure) {
        if (!disposed && generation.current === request) setError(String(failure));
      }
    }

    const subscription = listen("shortcut-folder-opened", load);
    subscription.then(() => { if (!disposed) load(); }).catch(console.error);
    return () => {
      disposed = true;
      subscription.then((unlisten) => unlisten()).catch(console.error);
    };
  }, []);

  const entries = useMemo(() => {
    const collator = new Intl.Collator(language, { numeric: true, sensitivity: "base" });
    return contents?.entries.slice().sort((a, b) => Number(b.directory) - Number(a.directory) || collator.compare(a.name, b.name));
  }, [contents, language]);
  const path = contents?.path ?? settings?.dock?.shortcutFolder;
  const close = () => invoke("close_shortcut_folder").catch(console.error);

  async function openEntry(name = null) {
    const request = generation.current;
    setError(null);
    setBusy(true);
    try {
      await invoke("open_shortcut_entry", { path, name });
      if (generation.current === request) await close();
    } catch (failure) {
      if (generation.current === request) setError(String(failure));
    } finally {
      if (generation.current === request) setBusy(false);
    }
  }

  return <section ref={panel} role="dialog" aria-labelledby="shortcut-folder-title" className="dock-preview shortcut-folder-panel" onKeyDown={(event) => { if (event.key === "Escape") close(); }}>
    <header className="shortcut-folder-header">
      <div className="min-w-0">
        <h1 id="shortcut-folder-title" className="font-semibold">Carpeta de accesos directos</h1>
        {path && <p data-no-i18n title={path} className="truncate text-xs text-white/65">{path}</p>}
      </div>
      <button type="button" className="shortcut-folder-close" aria-label="Cerrar" onClick={close}>×</button>
    </header>
    <div ref={list} className="shortcut-folder-list" aria-busy={!contents && !error}>
      {error && <p role="alert" className="px-2 py-1 text-xs text-amber-200">{error}</p>}
      {!contents && !error && <p role="status" className="px-2 py-1 text-white/65">Cargando carpeta…</p>}
      {entries?.length === 0 && <p className="px-2 py-1 text-white/65">La carpeta está vacía.</p>}
      {entries?.map(({ name, directory, icon_path }) => <button type="button" key={name} className="shortcut-folder-action" disabled={busy} onClick={() => openEntry(name)}>
        {directory ? <FolderIcon path={icon_path} fallback="📁" className="shortcut-folder-entry-icon" /> : <span aria-hidden="true" className="shortcut-folder-entry-icon">📄</span>}
        <span data-no-i18n title={name} className="truncate">{name}</span>
      </button>)}
    </div>
    <footer className="shortcut-folder-footer">
      <button type="button" className="shortcut-folder-action" disabled={!path || busy} onClick={() => openEntry()}>Abrir en el Explorador</button>
    </footer>
  </section>;
}
