import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";

/** Valores de MacDock (punto de partida y «Restablecer valores de MacDock»). */
const surface = (radius) => ({ tintOpacity: 14, blur: "medium", radius, rim: true, shadow: false });
export const MACDOCK_GLASS = { mode: "light", dock: surface(20), bar: surface(0), panels: surface(18), launcher: surface(24) };

/** Radios por defecto (px) de las formas de vidrio. */
export const GLASS_RADIUS = { panel: 18, dock: 20, launcher: 24, bar: 0 };

const prefersDark = () => window.matchMedia?.("(prefers-color-scheme: dark)").matches ?? false;

/** ¿Vidrio de noche? «auto» sigue al tema de Windows (prefers-color-scheme del WebView). */
export function useGlassDark(mode) {
  const [system, setSystem] = useState(prefersDark);
  useEffect(() => {
    const query = window.matchMedia?.("(prefers-color-scheme: dark)");
    if (!query) return undefined;
    const onChange = () => setSystem(query.matches);
    query.addEventListener("change", onChange);
    return () => query.removeEventListener("change", onChange);
  }, []);
  return mode === "dark" || (mode === "auto" && system);
}

/** Variables CSS de una superficie de vidrio (también las usa la vista previa de Ajustes). */
export function glassVars(cfg, dark) {
  const o = Math.max(0, Math.min(100, Number(cfg?.tintOpacity ?? 14))) / 100;
  const rim = cfg?.rim !== false;
  return {
    "--glass-tint": dark ? `rgba(30,32,40,${Math.min(1, o + 0.21).toFixed(3)})` : `rgba(255,255,255,${o.toFixed(3)})`,
    "--glass-sheen": dark ? "rgba(255,255,255,.06)" : "rgba(255,255,255,.10)",
    "--glass-rim": rim ? "rgba(255,255,255,.18)" : "transparent",
    "--glass-rim-top": rim ? "rgba(255,255,255,.34)" : "transparent",
    "--glass-radius": `${Number(cfg?.radius ?? 18)}px`,
    "--glass-shadow": cfg?.shadow ? "0 10px 28px rgba(0,0,0,.28)" : "none",
  };
}

/**
 * Ajustes de vidrio de una superficie ("dock" | "bar" | "panels" | "launcher"). Aplica sus
 * variables CSS a la ventana (cada ventana tiene una sola superficie) y devuelve lo que necesita
 * el desenfoque nativo: radio, modo de noche e intensidad.
 */
export function useGlass(settings, name) {
  const glass = settings?.appearance?.glass ?? MACDOCK_GLASS;
  const cfg = glass[name] ?? MACDOCK_GLASS[name];
  const dark = useGlassDark(glass.mode);
  const vars = glassVars(cfg, dark);
  const key = JSON.stringify(vars);
  useEffect(() => {
    const root = document.documentElement;
    Object.entries(vars).forEach(([k, v]) => root.style.setProperty(k, v));
    root.dataset.glass = dark ? "dark" : "light";
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [key, dark]);
  return { radius: Number(cfg.radius ?? 18), dark, blur: cfg.blur ?? "medium" };
}

/**
 * Informa al backend de la forma de vidrio visible (rectángulo + radio, px lógicos relativos a la
 * ventana) y de su tinte (modo e intensidad). El backend coloca debajo el desenfoque real del
 * escritorio con ese tamaño exacto, así que fuera de la forma la ventana queda 100 % transparente.
 */
export function useGlassShape(ref, glass, deps = []) {
  const { radius, dark = false, blur = "medium" } = typeof glass === "number" ? { radius: glass } : glass;
  useEffect(() => {
    const node = ref.current;
    if (!node) return undefined;
    let last = "";
    let frame = 0;
    const report = () => {
      frame = 0;
      const r = node.getBoundingClientRect();
      const shape = { x: r.left, y: r.top, w: r.width, h: r.height, radius, dark, blur };
      const key = `${shape.x}|${shape.y}|${shape.w}|${shape.h}|${radius}|${dark}|${blur}`;
      if (key === last) return;
      last = key;
      invoke("glass_shape", shape).catch(() => {});
    };
    const schedule = () => { if (!frame) frame = requestAnimationFrame(report); };
    report();
    const observer = new ResizeObserver(schedule);
    observer.observe(node);
    window.addEventListener("resize", schedule);
    // La forma puede moverse dentro de la ventana sin cambiar de tamaño (p. ej. la píldora del
    // Dock al recentrarse): una comprobación ligera por segundo lo cubre.
    const timer = setInterval(report, 1000);
    return () => { observer.disconnect(); window.removeEventListener("resize", schedule); clearInterval(timer); if (frame) cancelAnimationFrame(frame); };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [ref, radius, dark, blur, ...deps]);
}
