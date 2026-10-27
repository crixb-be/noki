import { useEffect, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { open as pickFolder } from "@tauri-apps/plugin-dialog";
import { startDrag } from "@crabnebula/tauri-plugin-drag";
import { dayKey, pad2 } from "../lib/time.js";

export function fmtSize(b) {
  if (b == null) return "";
  if (b < 1024) return `${b} B`;
  if (b < 1024 ** 2) return `${(b / 1024).toFixed(1)} KB`;
  if (b < 1024 ** 3) return `${(b / 1024 ** 2).toFixed(1)} MB`;
  return `${(b / 1024 ** 3).toFixed(2)} GB`;
}

const baseName = (p) => p.split(/[\\/]/).pop();

const ICONS = [
  [/\.(png|jpe?g|gif|webp|bmp|svg|ico|heic)$/i, "🖼️"],
  [/\.(mp4|mkv|mov|avi|webm)$/i, "🎬"],
  [/\.(mp3|wav|flac|ogg|m4a|aac)$/i, "🎵"],
  [/\.(zip|rar|7z|tar|gz)$/i, "🗜️"],
  [/\.(pdf)$/i, "📕"],
  [/\.(docx?|odt|rtf|txt|md)$/i, "📄"],
  [/\.(xlsx?|csv|ods)$/i, "📊"],
  [/\.(pptx?|odp)$/i, "📽️"],
  [/\.(exe|msi|bat|cmd|ps1)$/i, "⚙️"],
  [/\.(js|jsx|ts|tsx|py|rs|json|html|css|go|java|c|cpp)$/i, "🧩"],
];
const iconFor = (f) => (f.dir ? "📁" : ICONS.find(([re]) => re.test(f.name))?.[1] ?? "📄");

/** Imagen PNG (data URL) para el "fantasma" que acompaña al cursor al arrastrar fuera del notch. */
function dragIcon(emoji, count) {
  const c = document.createElement("canvas");
  c.width = c.height = 64;
  const g = c.getContext("2d");
  g.fillStyle = "rgba(24,24,27,0.92)";
  g.beginPath();
  g.roundRect(2, 2, 60, 60, 14);
  g.fill();
  g.font = "34px 'Segoe UI Emoji', sans-serif";
  g.textAlign = "center";
  g.textBaseline = "middle";
  g.fillText(emoji, 32, 34);
  if (count > 1) {
    g.fillStyle = "#38bdf8";
    g.beginPath();
    g.arc(50, 14, 11, 0, Math.PI * 2);
    g.fill();
    g.fillStyle = "#000";
    g.font = "bold 14px 'Segoe UI', sans-serif";
    g.fillText(String(count), 50, 15);
  }
  return c.toDataURL("image/png");
}

/** Escucha el arrastre nativo de Tauri (da rutas reales). `onEnter` permite abrir el notch. */
export function useNativeDrop({ onEnter, onDrop }) {
  const handlers = useRef({ onEnter, onDrop });
  handlers.current = { onEnter, onDrop };
  const [over, setOver] = useState(false);

  useEffect(() => {
    let unlisten;
    let cancelled = false;
    try {
      // El destino de arrastre es nativo y propio (dropzone.rs): avisa con `notch-drag` a cada ventana.
      const me = getCurrentWindow().label;
      listen("notch-drag", (e) => {
        const p = e.payload;
        if (p.label !== me) return;
        if (p.kind === "enter") {
          setOver(true);
          handlers.current.onEnter?.();
        } else if (p.kind === "drop") {
          setOver(false);
          handlers.current.onDrop?.(p.paths);
        } else {
          setOver(false);
        }
      })
        .then((fn) => (cancelled ? fn() : (unlisten = fn)))
        .catch(() => {});
    } catch {
      /* fuera de Tauri (navegador) */
    }
    return () => {
      cancelled = true;
      unlisten?.();
    };
  }, []);

  return over;
}

export async function addPaths(paths, setFiles) {
  const items = await Promise.all(
    paths.map(async (path) => {
      let info = { size: null, dir: false };
      try {
        info = await invoke("file_size", { path });
      } catch {
        /* sin tamaño */
      }
      return { id: `${Date.now()}-${path}`, name: baseName(path), path, size: info.size, dir: info.dir };
    })
  );
  setFiles((l) => [...l, ...items.filter((i) => !l.some((x) => x.path === i.path))]);
}

export default function FileShelf({ files, setFiles, dragging }) {
  const [selected, setSelected] = useState(() => new Set());
  const [missing, setMissing] = useState(() => new Set());
  const [busy, setBusy] = useState(false);
  const [note, setNote] = useState(null);
  const press = useRef(null); // { id, x, y } del botón pulsado sobre una tarjeta

  // Al abrir la pestaña (y cuando cambia la lista) se comprueba qué archivos ya no existen.
  useEffect(() => {
    if (!files.length) return setMissing(new Set());
    invoke("files_exist", { paths: files.map((f) => f.path) })
      .then((ok) => setMissing(new Set(files.filter((_, i) => !ok[i]).map((f) => f.id))))
      .catch(() => {});
  }, [files]);

  const flash = (msg) => {
    setNote(msg);
    setTimeout(() => setNote(null), 2500);
  };

  const open = (f) => invoke("open_path", { path: f.path }).catch(() => flash("El archivo ya no existe"));
  const reveal = (f) => invoke("reveal_in_folder", { path: f.path }).catch(() => flash("El archivo ya no existe"));
  const remove = (id) => {
    setFiles((l) => l.filter((f) => f.id !== id));
    setSelected((s) => {
      const n = new Set(s);
      n.delete(id);
      return n;
    });
  };
  const toggle = (id) =>
    setSelected((s) => {
      const n = new Set(s);
      n.has(id) ? n.delete(id) : n.add(id);
      return n;
    });

  const targets = () => (selected.size ? files.filter((f) => selected.has(f.id)) : files).filter((f) => !missing.has(f.id));

  const zip = async () => {
    const list = targets();
    if (!list.length || busy) return;
    setBusy(true);
    try {
      const d = new Date();
      const name = `Noki ${dayKey(d)} ${pad2(d.getHours())}-${pad2(d.getMinutes())}-${pad2(d.getSeconds())}`;
      const path = await invoke("zip_files", { paths: list.map((f) => f.path), name });
      await addPaths([path], setFiles);
      setSelected(new Set());
      flash(`Comprimido: ${baseName(path)}`);
    } catch (e) {
      flash(`No se pudo comprimir: ${e}`);
    } finally {
      setBusy(false);
    }
  };

  const chosen = () => files.filter((f) => selected.has(f.id) && !missing.has(f.id));

  /** Pide una carpeta (el notch se queda abierto mientras el selector tiene el foco) y aplica la acción. */
  const withFolder = async (verb, run) => {
    const list = chosen();
    if (!list.length || busy) return;
    invoke("request_open", { open: true, sticky: true }).catch(() => {});
    let dest = null;
    try {
      dest = await pickFolder({ directory: true, title: `${verb} a…` });
    } finally {
      invoke("request_open", { open: true, sticky: false }).catch(() => {});
    }
    if (!dest) return;
    setBusy(true);
    try {
      await run(list, dest);
    } catch (e) {
      flash(String(e));
    } finally {
      setBusy(false);
    }
  };

  const copyTo = () =>
    withFolder("Copiar", async (list, dest) => {
      await invoke("shelf_copy", { paths: list.map((f) => f.path), dest });
      flash(`Copiado a ${baseName(dest)}`);
    });

  const moveTo = () =>
    withFolder("Mover", async (list, dest) => {
      const moved = await invoke("shelf_move", { paths: list.map((f) => f.path), dest });
      const byId = new Map(list.map((f, i) => [f.id, moved[i]]));
      setFiles((l) => l.map((f) => (byId.has(f.id) ? { ...f, path: byId.get(f.id), name: baseName(byId.get(f.id)) } : f)));
      flash(`Movido a ${baseName(dest)}`);
    });

  const trash = async () => {
    const list = chosen();
    if (!list.length || busy) return;
    setBusy(true);
    try {
      await invoke("shelf_trash", { paths: list.map((f) => f.path) });
      setFiles((l) => l.filter((f) => !selected.has(f.id)));
      setSelected(new Set());
      flash("Enviado a la papelera");
    } catch (e) {
      flash(String(e));
    } finally {
      setBusy(false);
    }
  };

  const zips = chosen().filter((f) => /\.zip$/i.test(f.name));
  const unzip = async () => {
    if (busy || !zips.length) return;
    setBusy(true);
    try {
      const dirs = [];
      for (const z of zips) dirs.push(await invoke("shelf_unzip", { path: z.path }));
      await addPaths(dirs, setFiles);
      flash(`Extraído en ${baseName(dirs[0])}`);
    } catch (e) {
      flash(String(e));
    } finally {
      setBusy(false);
    }
  };

  // Arrastrar fuera del notch hacia otra app o carpeta (arrastre nativo de Windows).
  const beginDrag = async (f) => {
    const group = selected.has(f.id) ? files.filter((x) => selected.has(x.id)) : [f];
    const list = group.filter((x) => !missing.has(x.id));
    if (!list.length) return;
    // Se mantiene el notch abierto mientras dura el arrastre.
    invoke("request_open", { open: true, sticky: true }).catch(() => {});
    try {
      await startDrag({ item: list.map((x) => x.path), icon: dragIcon(iconFor(list[0]), list.length), mode: "copy" });
    } catch {
      flash("No se pudo arrastrar el archivo");
    } finally {
      invoke("request_open", { open: true, sticky: false }).catch(() => {});
    }
  };

  const onCardMouseMove = (e, f) => {
    const p = press.current;
    if (!p || p.id !== f.id || e.buttons !== 1) return;
    if (Math.hypot(e.clientX - p.x, e.clientY - p.y) > 6) {
      press.current = null;
      beginDrag(f);
    }
  };

  return (
    <div className="flex h-full flex-col gap-2">
      <div
        className={`flex shrink-0 items-center justify-center rounded-3xl border border-dashed text-xs transition-all duration-300 ${
          dragging
            ? "h-16 border-sky-400 bg-sky-400/10 text-sky-300"
            : files.length
              ? "h-9 border-white/15 text-white/40"
              : "h-full border-white/20 text-white/50"
        }`}
      >
        {dragging ? "Suelta para guardar" : files.length ? "📂 Suelta más archivos aquí" : "📂 Suelta archivos aquí para guardarlos"}
      </div>
      {files.length > 0 && (
        <>
          {/* Todas las acciones en una sola fila; el mensaje de estado va debajo. */}
          <div className="flex shrink-0 flex-nowrap items-center gap-1 overflow-x-auto text-[10px] [&>button]:shrink-0 [&>button]:whitespace-nowrap [&>button]:rounded-full [&>button]:bg-white/10 [&>button]:px-2 [&>button]:py-1 [&>button:disabled]:opacity-40 [&>button:hover]:bg-white/20">
            <button onClick={() => setSelected(selected.size === files.length ? new Set() : new Set(files.map((f) => f.id)))}>
              {selected.size === files.length ? "Ninguno" : "Todos"}
            </button>
            <button onClick={zip} disabled={busy} title={selected.size ? "Comprimir los seleccionados" : "Comprimir todo el estante"}>
              {busy ? "…" : "🗜️ Comprimir"}
            </button>
            {selected.size > 0 && (
              <>
                <button onClick={copyTo} disabled={busy}>📋 Copiar a…</button>
                <button onClick={moveTo} disabled={busy}>➡️ Mover a…</button>
                {zips.length > 0 && <button onClick={unzip} disabled={busy}>📦 Extraer</button>}
                <button onClick={trash} disabled={busy} title="Envía los archivos reales a la papelera de Windows">🗑️ Papelera</button>
                <button onClick={() => setFiles((l) => l.filter((f) => !selected.has(f.id)))} title="Quitar del estante (no borra el archivo)">✕ Quitar</button>
              </>
            )}
          </div>
          <div className="shrink-0 truncate px-1 text-[10px] text-white/35">
            {note ?? (selected.size ? `${selected.size} seleccionado${selected.size === 1 ? "" : "s"} · arrastra una tarjeta fuera para copiarla` : "Arrastra una tarjeta fuera para copiarla · doble clic abre")}
          </div>
          <div className="grid min-h-0 flex-1 grid-cols-4 content-start gap-2 overflow-y-auto px-0.5 py-1">
            {files.map((f) => {
              const gone = missing.has(f.id);
              const sel = selected.has(f.id);
              return (
                <div
                  key={f.id}
                  onClick={() => toggle(f.id)}
                  onDoubleClick={() => open(f)}
                  onMouseDown={(e) => e.button === 0 && (press.current = { id: f.id, x: e.clientX, y: e.clientY })}
                  onMouseMove={(e) => onCardMouseMove(e, f)}
                  onMouseUp={() => (press.current = null)}
                  title={gone ? "El archivo ya no existe" : f.path}
                  className={`group relative flex cursor-grab flex-col gap-0.5 rounded-2xl p-2.5 transition-all duration-200 ${
                    sel ? "bg-sky-400/20 ring-2 ring-inset ring-sky-400" : "bg-white/5 hover:bg-white/10"
                  } ${gone ? "opacity-40" : ""}`}
                >
                  <span className="text-lg leading-none">{iconFor(f)}</span>
                  <span className={`truncate text-[11px] font-medium ${gone ? "line-through" : ""}`}>{f.name}</span>
                  <span className="text-[10px] text-white/40">{gone ? "No encontrado" : fmtSize(f.size)}</span>
                  <div className="absolute right-1 top-1 flex gap-0.5 opacity-0 transition-opacity group-hover:opacity-100">
                    <button onClick={(e) => { e.stopPropagation(); reveal(f); }} className="rounded-full bg-black/60 px-1.5 text-[10px] hover:bg-white/30" aria-label="Mostrar en carpeta" title="Mostrar en carpeta">📂</button>
                    <button onClick={(e) => { e.stopPropagation(); remove(f.id); }} className="rounded-full bg-black/60 px-1.5 text-[10px] hover:bg-red-500/70" aria-label="Quitar" title="Quitar del estante">✕</button>
                  </div>
                </div>
              );
            })}
          </div>
        </>
      )}
    </div>
  );
}
