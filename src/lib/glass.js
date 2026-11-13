import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";

/** Dos fotogramas: el primero se programa antes de pintar y el segundo, cuando ya se ha pintado. */
const afterPaint = (fn) => requestAnimationFrame(() => requestAnimationFrame(fn));

/** Valores de MacDock (punto de partida y «Restablecer valores de MacDock»). */
const surface = (radius) => ({ tintOpacity: 14, blur: "medium", radius, rim: true, shadow: true });
export const MACDOCK_GLASS = { mode: "light", engine: "native", dock: surface(20), bar: surface(0), panels: surface(18), launcher: surface(24) };

/** Radios por defecto (px) de las formas de vidrio. */
export const GLASS_RADIUS = { panel: 18, dock: 20, launcher: 24, bar: 0 };

/**
 * Radio (px lógicos) de las esquinas redondeadas nativas de Windows 11 (`DWMWCP_ROUND`). Con el
 * desenfoque nativo el CSS usa exactamente este radio: si fuera mayor, el material de Windows
 * asomaría por fuera de la curva en cada esquina.
 */
export const NATIVE_RADIUS = 8;

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

/** Radio efectivo: el de Ajustes con el vidrio CSS; el nativo de Windows con el desenfoque nativo. */
export function glassRadius(cfg, engine) {
  const radius = Number(cfg?.radius ?? 18);
  return engine === "css" || radius === 0 ? radius : NATIVE_RADIUS;
}

/**
 * Variables CSS de una superficie de vidrio (también las usa la vista previa de Ajustes).
 * Acabado de las capturas de referencia (macdock-*.png): tinte gris medio uniforme, borde de 1 px
 * rgba(255,255,255,.22) algo más vivo arriba, apenas brillo y sombra suave.
 * - «native»: el material acrílico de Windows difumina el escritorio; el tinte es ligero.
 * - «css» («Vidrio suave sin blur nativo»): sin desenfoque, así que el tinte es casi opaco para que
 *   el texto y los iconos de detrás no se transparenten como rayas.
 */
export function glassVars(cfg, dark, engine = "native") {
  const o = Math.max(0, Math.min(100, Number(cfg?.tintOpacity ?? 14))) / 100;
  const rim = cfg?.rim !== false;
  const css = engine === "css";
  // Nativo: el acento acrílico ya trae el gris medio (backdrop.rs `piece_tint`); aquí solo un velo.
  const alpha = css ? Math.min(0.96, 0.74 + o * 0.5) : Math.min(0.9, 0.06 + o * 0.5);
  return {
    "--glass-tint": dark ? `rgba(24,24,28,${(alpha + 0.04).toFixed(3)})` : `rgba(64,64,70,${alpha.toFixed(3)})`,
    "--glass-sheen": css ? "rgba(255,255,255,.07)" : "rgba(255,255,255,.04)",
    "--glass-rim": rim ? "rgba(255,255,255,.22)" : "transparent",
    "--glass-rim-top": rim ? "rgba(255,255,255,.30)" : "transparent",
    "--glass-radius": `${glassRadius(cfg, engine)}px`,
    "--glass-shadow": cfg?.shadow ? "0 8px 24px rgba(0,0,0,.22)" : "0 0 #0000",
  };
}

/**
 * Ajustes de vidrio de una superficie ("dock" | "bar" | "panels" | "launcher"). Aplica sus
 * variables CSS a la ventana (cada ventana tiene una sola superficie) y devuelve lo que necesita
 * el desenfoque nativo: radio, modo de noche y si se usa el desenfoque nativo.
 */
export function useGlass(settings, name) {
  const glass = settings?.appearance?.glass ?? MACDOCK_GLASS;
  const cfg = glass[name] ?? MACDOCK_GLASS[name];
  const engine = glass.engine === "css" ? "css" : "native";
  const dark = useGlassDark(glass.mode);
  const vars = glassVars(cfg, dark, engine);
  const key = JSON.stringify(vars);
  useEffect(() => {
    const root = document.documentElement;
    Object.entries(vars).forEach(([k, v]) => root.style.setProperty(k, v));
    root.dataset.glass = dark ? "dark" : "light";
    root.dataset.glassEngine = engine;
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [key, dark, engine]);
  return { radius: glassRadius(cfg, engine), dark, native: engine === "native" };
}

/**
 * Informa al backend de la forma de vidrio visible (rectángulo + radio, px lógicos relativos a la
 * ventana) y de su tinte (modo e intensidad). El backend coloca debajo el desenfoque real del
 * escritorio con ese tamaño exacto, así que fuera de la forma la ventana queda 100 % transparente.
 */
export function useGlassShape(ref, glass, deps = []) {
  const { radius, dark = false, native = true } = typeof glass === "number" ? { radius: glass } : glass;
  useEffect(() => {
    let node = null;
    let last = "";
    let frame = 0;
    let attachFrame = 0;
    let observer = null;
    let timer = 0;
    let stopReveal = null;
    let disposed = false;
    const report = () => {
      frame = 0;
      if (!node?.isConnected) { node = null; attach(); return; }
      const r = node.getBoundingClientRect();
      const shape = { x: r.left, y: r.top, w: r.width, h: r.height, radius, dark, native };
      const key = `${shape.x}|${shape.y}|${shape.w}|${shape.h}|${radius}|${dark}|${native}`;
      if (key === last) return;
      last = key;
      invoke("glass_shape", shape).catch(() => {});
    };
    const schedule = () => { if (!frame) frame = requestAnimationFrame(report); };
    // El vidrio nativo solo se muestra cuando la vista ya ha pintado un fotograma estando visible
    // (backdrop.rs `glass_ready`): así no aparece un recuadro gris vacío antes que el contenido.
    const confirmPainted = () => afterPaint(() => { if (!disposed) invoke("glass_ready").catch(() => {}); });
    // El nodo puede no existir al montar (p. ej. la barra devuelve null hasta tener los ajustes):
    // antes el efecto salía y no volvía a ejecutarse nunca, y la barra se quedaba sin vidrio.
    const attach = () => {
      attachFrame = 0;
      if (disposed || node) return;
      node = ref.current;
      if (!node) { attachFrame = requestAnimationFrame(attach); return; }
      last = "";
      report();
      confirmPainted();
      observer?.disconnect();
      observer = new ResizeObserver(schedule);
      observer.observe(node);
    };
    attach();
    window.addEventListener("resize", schedule);
    // La forma puede moverse dentro de la ventana sin cambiar de tamaño (p. ej. la píldora del
    // Dock al recentrarse): una comprobación ligera por segundo lo cubre.
    timer = setInterval(() => { if (node) report(); }, 1000);
    // Al volver a mostrarse la ventana, el backend pide confirmar un fotograma pintado.
    const label = getCurrentWindow().label;
    listen("glass-reveal", (event) => { if (event.payload === label && node) confirmPainted(); })
      .then((stop) => { if (disposed) stop(); else stopReveal = stop; })
      .catch(() => {});
    return () => {
      disposed = true;
      observer?.disconnect();
      window.removeEventListener("resize", schedule);
      clearInterval(timer);
      if (frame) cancelAnimationFrame(frame);
      if (attachFrame) cancelAnimationFrame(attachFrame);
      stopReveal?.();
    };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [ref, radius, dark, native, ...deps]);
}
