import { useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";

/** Radios por defecto (px) de las formas de vidrio, como en MacDock. */
export const GLASS_RADIUS = { panel: 18, dock: 20, launcher: 24, bar: 0 };

/**
 * Informa al backend de la forma de vidrio visible (rectángulo + radio, px lógicos relativos a la
 * ventana). El backend coloca debajo el desenfoque real del escritorio con ese tamaño exacto, así que
 * fuera de la forma redondeada la ventana queda 100 % transparente.
 */
export function useGlassShape(ref, radius, deps = []) {
  useEffect(() => {
    const node = ref.current;
    if (!node) return undefined;
    let last = "";
    let frame = 0;
    const report = () => {
      frame = 0;
      const r = node.getBoundingClientRect();
      const shape = { x: r.left, y: r.top, w: r.width, h: r.height, radius };
      const key = `${shape.x}|${shape.y}|${shape.w}|${shape.h}|${radius}`;
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
  }, [ref, radius, ...deps]);
}
