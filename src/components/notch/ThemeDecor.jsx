import { useMemo } from "react";

/** Adornos del tema dentro del notch (detrás del contenido). El notch cerrado solo lleva los marcados con `c`. */
export function NotchDecor({ theme, compact }) {
  if (!theme) return null;
  const items = compact ? theme.decor.filter((d) => d.c) : theme.decor;
  return (
    <div className="pointer-events-none absolute inset-0 overflow-hidden" aria-hidden="true">
      {items.map((d, i) => (
        <span
          key={`${d.e}${i}`}
          className={`absolute select-none leading-none ${d.fly ? "animate-[themeFloat_5s_ease-in-out_infinite]" : "animate-[themeBob_3.4s_ease-in-out_infinite]"}`}
          style={{
            left: `${d.x}%`,
            top: `${d.y}%`,
            fontSize: compact ? `${11 * d.s}px` : `${17 * d.s}px`,
            opacity: compact ? 0.6 : 0.5,
            animationDelay: `${(i * 0.7) % 3}s`,
            transform: "translate(-50%, -50%)",
          }}
        >
          {d.e}
        </span>
      ))}
    </div>
  );
}

/** Franja de los costados: pocos adornos del tema, bien separados y sin un ritmo fijo. */
const STEP = 7.5; // % del ancho entre un adorno y el siguiente
export function StripDecor({ theme, height }) {
  const items = useMemo(() => {
    if (!theme) return [];
    const n = Math.ceil(100 / STEP);
    return Array.from({ length: n }, (_, i) => {
      // «Ruido» fijo: el mismo en cada arranque, pero sin patrón evidente.
      const r = Math.abs(Math.sin((i + 1) * 12.9898)) ;
      const q = Math.abs(Math.sin((i + 1) * 78.233));
      return {
        e: theme.strip[(i * 3 + Math.floor(r * 2)) % theme.strip.length],
        x: i * STEP + 2 + r * 3.5,
        dy: (q - 0.5) * 6,
        s: 0.75 + r * 0.3,
      };
    });
  }, [theme]);
  if (!theme) return null;
  return (
    <div className="pointer-events-none relative h-full w-full overflow-hidden" aria-hidden="true">
      {items.map((d, i) => (
        <span
          key={i}
          className="absolute select-none leading-none opacity-85"
          style={{ left: `${d.x}%`, top: `calc(50% + ${d.dy}px)`, fontSize: Math.max(9, height * 0.5 * d.s), transform: "translate(-50%, -50%)" }}
        >
          {d.e}
        </span>
      ))}
    </div>
  );
}
