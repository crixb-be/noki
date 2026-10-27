import { useEffect, useRef } from "react";
import { listen } from "@tauri-apps/api/event";

const MIN = 0.1;

/**
 * Barras del visualizador. Reciben los niveles reales del audio del sistema (`audio-levels`, 12 bandas)
 * y se animan directamente en el DOM, sin pasar por el estado de React (unos 25 cambios por segundo).
 */
export default function Visualizer({ bars = 12, className = "h-4 w-24", color = "bg-fuchsia-400" }) {
  const refs = useRef([]);
  const levels = useRef(new Array(bars).fill(0));

  useEffect(() => {
    let unlisten;
    let cancelled = false;
    const paint = () => {
      refs.current.forEach((el, i) => {
        if (el) el.style.transform = `scaleY(${Math.max(MIN, levels.current[i])})`;
      });
    };
    paint();
    listen("audio-levels", (e) => {
      const bands = e.payload;
      const group = bands.length / bars;
      for (let i = 0; i < bars; i++) {
        // Se promedian las bandas que caen en cada barra.
        let sum = 0;
        let n = 0;
        for (let b = Math.floor(i * group); b < Math.max(Math.floor(i * group) + 1, Math.floor((i + 1) * group)); b++) {
          sum += bands[b] ?? 0;
          n++;
        }
        const v = n ? sum / n : 0;
        const prev = levels.current[i];
        // Sube al instante, baja con suavidad.
        levels.current[i] = v > prev ? v : prev * 0.6 + v * 0.4;
      }
      paint();
    }).then((fn) => (cancelled ? fn() : (unlisten = fn)));
    return () => {
      cancelled = true;
      unlisten?.();
    };
  }, [bars]);

  return (
    <div className={`flex items-end justify-between gap-[2px] ${className}`} aria-hidden>
      {Array.from({ length: bars }, (_, i) => (
        <div
          key={i}
          ref={(el) => (refs.current[i] = el)}
          className={`h-full flex-1 origin-bottom rounded-full transition-transform duration-75 ${color}`}
          style={{ transform: `scaleY(${MIN})` }}
        />
      ))}
    </div>
  );
}
