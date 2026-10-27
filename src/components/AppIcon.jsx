import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";

const COLORS = ["#60a5fa", "#a78bfa", "#34d399", "#fbbf24", "#f472b6", "#22d3ee", "#fb923c", "#a3e635"];
const colorOf = (name) => COLORS[[...name].reduce((n, c) => n + c.charCodeAt(0), 0) % COLORS.length];

/** Iconos ya pedidos a Windows: se comparten entre todas las vistas de esta ventana. */
const cache = new Map(); // ruta → data URL, o null si no se pudo obtener
const failedAt = new Map(); // ruta → cuándo falló por última vez
const RETRY_AFTER_MS = 20000;
const wait = (ms) => new Promise((r) => setTimeout(r, ms));

/** Pide el icono a Windows; a veces falla la primera vez (arranque en frío), así que se reintenta un par de veces. */
async function fetchIcon(path) {
  let last;
  for (const delay of [0, 700, 2000]) {
    if (delay) await wait(delay);
    try {
      return await invoke("app_icon", { path });
    } catch (e) {
      last = e;
      if (/0x80070002|0x80070003/.test(String(e))) break; // el archivo no existe: reintentar no sirve
    }
  }
  throw last;
}

/** Icono real del programa; mientras llega (o si Windows no lo da) se muestra su inicial en un cuadro de color. */
export default function AppIcon({ app, size = 28, className = "" }) {
  const [src, setSrc] = useState(() => cache.get(app.path) ?? null);

  useEffect(() => {
    const failed = cache.get(app.path) === null && Date.now() - (failedAt.get(app.path) ?? 0) < RETRY_AFTER_MS;
    if (cache.has(app.path) && (cache.get(app.path) || failed)) {
      setSrc(cache.get(app.path));
      return;
    }
    let alive = true;
    fetchIcon(app.path)
      .then((url) => {
        cache.set(app.path, url);
        if (alive) setSrc(url);
      })
      .catch(() => {
        cache.set(app.path, null);
        failedAt.set(app.path, Date.now());
      });
    return () => {
      alive = false;
    };
  }, [app.path]);

  if (src) return <img src={src} alt="" draggable={false} style={{ width: size, height: size }} className={`shrink-0 object-contain ${className}`} />;
  return (
    <span
      className={`flex shrink-0 items-center justify-center rounded-lg font-bold text-black ${className}`}
      style={{ width: size, height: size, background: colorOf(app.name), fontSize: size * 0.45 }}
    >
      {app.name.trim().charAt(0).toUpperCase()}
    </span>
  );
}
