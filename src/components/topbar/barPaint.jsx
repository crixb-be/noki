import { useEffect, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import { fillFor, rgba, useBgImage } from "../../lib/background.js";
import { THEMES, resolveTheme } from "../../lib/themes.js";
import { StripDecor } from "../notch/ThemeDecor.jsx";

/** Calcula una sola vez el fondo de la barra y lo comparte con el notch. */
export function useBarPaint(settings) {
  const a = settings?.appearance ?? {};
  const [tint, setTint] = useState(null);
  const automatic = a.barMode === "auto" || a.notchAuto;
  const theme = THEMES[resolveTheme(a.barTheme, a.birthday, a.themeEvents)] ?? null;
  const image = useBgImage(a.barMode === "image" ? a.barImage : "");
  useEffect(() => {
    if (!automatic) return undefined;
    let stop;
    listen("bar-tint", (event) => setTint(event.payload?.color ?? null)).then((fn) => { stop = fn; });
    return () => stop?.();
  }, [automatic]);
  const fill = theme ? rgba(theme.bar, a.barThemeOpacity)
    : a.barMode === "none" ? null
      : a.barMode === "match" ? rgba(a.notchColor, a.opacity)
        : a.barMode === "auto" ? rgba(tint ?? a.notchColor, a.barOpacity)
          : a.barMode === "color" ? rgba(a.barColor, a.barOpacity)
            : fillFor({ color: a.barColor, opacity: a.barOpacity, image, dim: a.barDim });
  return { fill, theme, tint };
}

export function BarPaint({ settings, tint }) {
  const a = settings?.appearance ?? {};
  const theme = THEMES[resolveTheme(a.barTheme, a.birthday, a.themeEvents)] ?? null;
  const image = useBgImage(a.barMode === "image" ? a.barImage : "");
  const fill = theme ? rgba(theme.bar, a.barThemeOpacity)
    : a.barMode === "none" ? "transparent"
      : a.barMode === "match" ? rgba(a.notchColor, a.opacity)
        : a.barMode === "auto" ? rgba(tint ?? a.notchColor, a.barOpacity)
          : a.barMode === "color" ? rgba(a.barColor, a.barOpacity)
            : fillFor({ color: a.barColor, opacity: a.barOpacity, image, dim: a.barDim });
  // Liquid glass en todos los modos: el desenfoque real lo pone backdrop.rs y el tinte, el brillo y
  // el borde los dibuja CSS (.topbar-glass). Los temas solo añaden su decoración.
  // «Automático»: el color de la cabecera de la ventana activa se pone sobre el vidrio (con la
  // transición suave de .topbar-paint), como hacía la barra antes del vidrio.
  void fill;
  const auto = !theme && a.barMode === "auto" && tint;
  const style = auto ? { background: `linear-gradient(to bottom, var(--glass-sheen), rgba(255,255,255,0) 60%), ${rgba(tint, a.barOpacity)}` } : undefined;
  return <div className="topbar-paint topbar-glass" style={style} aria-hidden="true">{theme && <StripDecor theme={theme} height={settings?.topbar?.height ?? 30} />}</div>;
}

/** Gris del vidrio nativo debajo de la barra (backdrop.rs `piece_tint`, modo claro). */
const GLASS_BASE = [0x46, 0x48, 0x4c];

/** Luminancia relativa WCAG 2.x de un color sRGB (0-255). */
function luminance([r, g, b]) {
  const lin = (c) => { const v = c / 255; return v <= 0.03928 ? v / 12.92 : ((v + 0.055) / 1.055) ** 2.4; };
  return 0.2126 * lin(r) + 0.7152 * lin(g) + 0.0722 * lin(b);
}

/**
 * Color del texto y los iconos sobre la barra automática: el tinte se mezcla con el vidrio según su
 * opacidad y, si el fondo resultante es claro (blanco, gris claro, amarillo…), la tinta es negra
 * (#111111); si es oscuro o saturado (azul, gris oscuro…), blanca (#FFFFFF). `null` = sin cambio.
 */
export function barInk(settings, tint) {
  const a = settings?.appearance ?? {};
  if (a.barMode !== "auto" || !/^#[0-9a-f]{6}$/i.test(tint ?? "")) return null;
  const alpha = Math.max(0, Math.min(100, Number(a.barOpacity ?? 100))) / 100;
  const color = [1, 3, 5].map((i, k) => parseInt(tint.slice(i, i + 2), 16) * alpha + GLASS_BASE[k] * (1 - alpha));
  return luminance(color) > 0.45 ? "#111111" : "#FFFFFF";
}
