import { useCallback, useEffect, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

const isObj = (v) => v && typeof v === "object" && !Array.isArray(v);

/** Fusión profunda: `patch` solo pisa las claves que trae. */
export function merge(base, patch) {
  const out = { ...base };
  for (const [k, v] of Object.entries(patch)) out[k] = isObj(v) && isObj(base?.[k]) ? merge(base[k], v) : v;
  return out;
}

/**
 * Ajustes compartidos entre ventanas. Rust es la fuente de verdad: guarda en %APPDATA%,
 * aplica los cambios (layout, atajo, autoinicio) y emite `settings-changed` a todas las ventanas.
 */
export function useSettings() {
  const [settings, setSettings] = useState(null);
  const [warning, setWarning] = useState(null);
  const latest = useRef(null);
  const timer = useRef(null);
  const inFlight = useRef(false);

  const flush = useCallback(() => {
    inFlight.current = true;
    invoke("save_settings", { settings: latest.current })
      .then((w) => setWarning(w ?? null))
      .catch((e) => setWarning(String(e)))
      .finally(() => {
        inFlight.current = false;
        // Se vuelve a leer el estado real de Rust (p. ej. una clave recién guardada pasa a verse como «guardada»),
        // salvo que haya cambios nuevos en camino.
        if (!timer.current) {
          invoke("get_settings")
            .then((s) => {
              if (timer.current) return;
              latest.current = s;
              setSettings(s);
            })
            .catch(() => {});
        }
      });
  }, []);

  useEffect(() => {
    let unlisten;
    let cancelled = false;
    invoke("get_settings").then((s) => {
      if (cancelled) return;
      latest.current = s;
      setSettings(s);
    });
    listen("settings-changed", (e) => {
      // Mientras hay cambios locales sin confirmar se ignora el eco para evitar saltos en sliders.
      if (timer.current || inFlight.current) return;
      latest.current = e.payload;
      setSettings(e.payload);
    }).then((fn) => (cancelled ? fn() : (unlisten = fn)));
    return () => {
      cancelled = true;
      unlisten?.();
    };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  /** `patch` es un objeto parcial, p. ej. `{ appearance: { opacity: 80 } }`. */
  function update(patch) {
    if (!latest.current) return;
    latest.current = merge(latest.current, patch);
    setSettings(latest.current);
    clearTimeout(timer.current);
    timer.current = setTimeout(() => {
      timer.current = null;
      flush();
    }, 120);
  }

  const reset = useCallback(async () => {
    clearTimeout(timer.current);
    timer.current = null;
    await invoke("reset_settings");
    const s = await invoke("get_settings");
    latest.current = s;
    setSettings(s);
  }, []);

  return { settings, update, reset, warning };
}
