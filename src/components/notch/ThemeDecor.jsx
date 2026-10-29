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

/** Franja de los costados: los adornos del tema repetidos a lo largo. */
export function StripDecor({ theme, height }) {
  const row = useMemo(() => (theme ? Array.from({ length: 80 }, (_, i) => theme.strip[i % theme.strip.length]) : []), [theme]);
  if (!theme) return null;
  return (
    <div className="pointer-events-none flex h-full w-full items-center justify-around overflow-hidden" aria-hidden="true" style={{ fontSize: Math.max(10, height * 0.55) }}>
      {row.map((e, i) => (
        <span key={i} className="shrink-0 select-none leading-none opacity-80" style={{ transform: `translateY(${i % 2 ? 1 : -1}px)` }}>
          {e}
        </span>
      ))}
    </div>
  );
}
