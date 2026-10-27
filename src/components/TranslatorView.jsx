import { useCallback, useEffect, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { open as pickFile } from "@tauri-apps/plugin-dialog";
import { useStore } from "../hooks/useStore.js";

const LANGS = [
  ["es", "Español"], ["en", "Inglés"], ["fr", "Francés"], ["de", "Alemán"], ["it", "Italiano"], ["pt", "Portugués"],
  ["ja", "Japonés"], ["ko", "Coreano"], ["zh-CN", "Chino"], ["ru", "Ruso"], ["ar", "Árabe"], ["hi", "Hindi"],
  ["tr", "Turco"], ["nl", "Neerlandés"], ["pl", "Polaco"], ["uk", "Ucraniano"],
];
const nameOf = (code) => LANGS.find(([c]) => c === code)?.[1] ?? code;

const select = "rounded-full bg-white/10 px-2 py-1 text-[11px] outline-none [color-scheme:dark] hover:bg-white/15";
const chip = "rounded-full bg-white/10 px-2.5 py-1 text-[11px] transition-all duration-300 hover:bg-white/20 active:scale-95 disabled:opacity-30";

/** Traductor: texto (Google Translate, sin clave) e imágenes (OCR de Windows, en tu equipo). Traduce solo mientras escribes. */
export default function TranslatorView() {
  const [cfg, setCfg] = useStore("translatorCfg", { from: "auto", to: "es" });
  const [input, setInput] = useState("");
  const [output, setOutput] = useState("");
  const [detected, setDetected] = useState(null);
  const [status, setStatus] = useState(null); // null | "translating" | "reading" | texto de error
  const [copied, setCopied] = useState(false);
  const ticket = useRef(0); // descarta respuestas viejas si escribes más rápido de lo que contestan

  const translate = useCallback(
    async (text, from, to) => {
      const id = ++ticket.current;
      if (!text.trim()) {
        setOutput("");
        setDetected(null);
        setStatus(null);
        return;
      }
      setStatus("translating");
      try {
        const r = await invoke("translate_text", { text, to, from });
        if (id !== ticket.current) return;
        setOutput(r.text);
        setDetected(r.detected);
        setStatus(null);
      } catch (e) {
        if (id !== ticket.current) return;
        setStatus(String(e));
      }
    },
    []
  );

  // Se traduce 400 ms después de la última tecla.
  useEffect(() => {
    const id = setTimeout(() => translate(input, cfg.from, cfg.to), 400);
    return () => clearTimeout(id);
  }, [input, cfg.from, cfg.to, translate]);

  const readImage = async (run) => {
    setStatus("reading");
    try {
      const text = await run();
      setInput(text);
    } catch (e) {
      setStatus(String(e));
    }
  };

  const onPaste = (e) => {
    const file = [...(e.clipboardData?.files ?? [])].find((f) => f.type.startsWith("image/"));
    if (!file) return; // texto normal: se pega sin más
    e.preventDefault();
    readImage(
      () =>
        new Promise((resolve, reject) => {
          const reader = new FileReader();
          reader.onload = () => invoke("ocr_image", { data: reader.result }).then(resolve, reject);
          reader.onerror = () => reject("No se pudo leer la imagen pegada.");
          reader.readAsDataURL(file);
        })
    );
  };

  const chooseImage = async () => {
    // El selector de archivos le quita el foco al notch: se mantiene abierto mientras tanto.
    invoke("request_open", { open: true, sticky: true }).catch(() => {});
    let path = null;
    try {
      path = await pickFile({ multiple: false, title: "Imagen con texto", filters: [{ name: "Imágenes", extensions: ["png", "jpg", "jpeg", "bmp", "gif", "tif", "tiff", "webp"] }] });
    } finally {
      invoke("request_open", { open: true, sticky: false }).catch(() => {});
    }
    if (path) readImage(() => invoke("ocr_file", { path }));
  };

  const swap = () => {
    // «Detectar» no se puede poner como destino: se usa el idioma detectado si lo hay.
    const newTo = cfg.from === "auto" ? (detected && LANGS.some(([c]) => c === detected) ? detected : "en") : cfg.from;
    setCfg({ from: cfg.to, to: newTo });
    if (output) setInput(output);
  };

  const copy = async () => {
    try {
      await navigator.clipboard.writeText(output);
      setCopied(true);
      setTimeout(() => setCopied(false), 1200);
    } catch {
      /* sin permiso de portapapeles */
    }
  };

  const busy = status === "translating" || status === "reading";
  const error = status && !busy ? status : null;

  return (
    <div className="flex h-full min-h-0 flex-col gap-2 rounded-3xl bg-white/5 p-3">
      <div className="flex shrink-0 items-center gap-1.5">
        <select value={cfg.from} onChange={(e) => setCfg({ ...cfg, from: e.target.value })} className={select} title="Idioma del texto">
          <option value="auto" className="bg-neutral-900">Detectar idioma</option>
          {LANGS.map(([c, n]) => <option key={c} value={c} className="bg-neutral-900">{n}</option>)}
        </select>
        <button onClick={swap} className={chip} title="Intercambiar idiomas" aria-label="Intercambiar idiomas">⇄</button>
        <select value={cfg.to} onChange={(e) => setCfg({ ...cfg, to: e.target.value })} className={select} title="Traducir a">
          {LANGS.map(([c, n]) => <option key={c} value={c} className="bg-neutral-900">{n}</option>)}
        </select>
        <span className="ml-auto flex items-center gap-1.5 text-[10px] text-white/40">
          {busy && <span className="animate-pulse">{status === "reading" ? "Leyendo imagen…" : "Traduciendo…"}</span>}
          {!busy && detected && cfg.from === "auto" && input.trim() && <span>Detectado: {nameOf(detected)}</span>}
        </span>
      </div>

      <div className="grid min-h-0 flex-1 grid-cols-2 gap-2">
        <div className="relative flex min-h-0 flex-col">
          <textarea
            value={input}
            onChange={(e) => setInput(e.target.value)}
            onPaste={onPaste}
            spellCheck={false}
            placeholder="Escribe, pega texto o una imagen (Ctrl+V)…"
            className="min-h-0 flex-1 resize-none rounded-2xl bg-white/10 p-2.5 pr-8 text-xs outline-none placeholder:text-white/30 focus:bg-white/15"
          />
          {input && (
            <button onClick={() => setInput("")} className="absolute right-1.5 top-1.5 rounded-full px-1.5 text-white/40 hover:bg-white/10 hover:text-white" aria-label="Borrar">✕</button>
          )}
        </div>
        <div className="relative flex min-h-0 flex-col">
          <div className="min-h-0 flex-1 select-text overflow-y-auto whitespace-pre-wrap break-words rounded-2xl bg-black/25 p-2.5 pr-8 text-xs">
            {output || <span className="text-white/25">La traducción aparece aquí.</span>}
          </div>
          {output && (
            <button onClick={copy} className="absolute right-1.5 top-1.5 rounded-full px-1.5 text-white/40 hover:bg-white/10 hover:text-white" title="Copiar traducción" aria-label="Copiar">{copied ? "✓" : "⧉"}</button>
          )}
        </div>
      </div>

      <div className="flex shrink-0 items-center gap-1.5">
        <button onClick={chooseImage} disabled={busy} className={chip}>🖼️ Traducir imagen…</button>
        <span className="truncate text-[10px] text-white/30">{error ? <span className="text-amber-300">{error}</span> : "Win+Shift+S y Ctrl+V aquí para traducir una captura"}</span>
      </div>
    </div>
  );
}
