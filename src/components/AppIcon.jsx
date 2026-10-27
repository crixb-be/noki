import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";

const COLORS = ["#60a5fa", "#a78bfa", "#34d399", "#fbbf24", "#f472b6", "#22d3ee", "#fb923c", "#a3e635"];
const colorOf = (name) => COLORS[[...name].reduce((n, c) => n + c.charCodeAt(0), 0) % COLORS.length];

/** Iconos ya pedidos a Windows: se comparten entre todas las vistas de esta ventana. */
const cache = new Map(); // ruta → data URL, o null si no se pudo obtener

/** Icono real del programa; mientras llega (o si Windows no lo da) se muestra su inicial en un cuadro de color. */
export default function AppIcon({ app, size = 28, className = "" }) {
  const [src, setSrc] = useState(() => cache.get(app.path) ?? null);

  useEffect(() => {
    if (cache.has(app.path)) {
      setSrc(cache.get(app.path));
      return;
    }
    let alive = true;
    invoke("app_icon", { path: app.path })
      .then((url) => {
        cache.set(app.path, url);
        if (alive) setSrc(url);
      })
      .catch(() => cache.set(app.path, null));
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
