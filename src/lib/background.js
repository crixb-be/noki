import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";

/** "#rrggbb" + opacidad 0-100 → "rgba(r, g, b, a)". */
export function rgba(hex, opacity = 100) {
  const n = /^#[0-9a-f]{6}$/i.test(hex) ? hex : "#000000";
  const r = parseInt(n.slice(1, 3), 16);
  const g = parseInt(n.slice(3, 5), 16);
  const b = parseInt(n.slice(5, 7), 16);
  return `rgba(${r}, ${g}, ${b}, ${Math.max(0, Math.min(100, opacity)) / 100})`;
}

const cache = new Map(); // nombre de archivo → data URL ("" si ya no existe)

/** Imagen de fondo guardada por Rust, como data URL ("" mientras carga o si no existe). */
export function useBgImage(name) {
  const [url, setUrl] = useState(() => (name ? cache.get(name) ?? "" : ""));
  useEffect(() => {
    if (!name) return setUrl("");
    if (cache.has(name)) return setUrl(cache.get(name));
    let alive = true;
    invoke("bg_data", { name })
      .then((u) => {
        cache.set(name, u);
        if (alive) setUrl(u);
      })
      .catch(() => {});
    return () => {
      alive = false;
    };
  }, [name]);
  return url;
}

/** Valor para la propiedad CSS `background`: la imagen (oscurecida `dim` %) o, si no hay, el color plano. */
export function fillFor({ color, opacity, image, dim }) {
  if (image) {
    const shade = `rgba(0, 0, 0, ${Math.max(0, Math.min(85, dim)) / 100})`;
    return `linear-gradient(${shade}, ${shade}), url("${image}") center / cover no-repeat`;
  }
  return rgba(color, opacity);
}
