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
  return <div className="topbar-paint" style={{ background: fill }} aria-hidden="true">{theme && <StripDecor theme={theme} height={settings?.topbar?.height ?? 30} />}</div>;
}
