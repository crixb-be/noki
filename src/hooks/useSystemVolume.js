import { useCallback, useEffect, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

// Mientras el usuario arrastra un slider, el backend devuelve (cada ~100 ms) valores intermedios que
// ya quedaron atrás; se ignoran durante esta ventana y al terminar se reconcilia con el valor real.
const ECHO_HOLD_MS = 400;

/** Nivel de ondas para SpeakerIcon según el porcentaje. */
export const speakerLevel = (value) => (value == null ? 3 : value === 0 ? 0 : value < 34 ? 1 : 2);

/**
 * Volumen y silencio de la salida predeterminada, sincronizados con el backend: misma fuente que el
 * OSD del notch (`volume-state` se emite en cada cambio, también con las teclas físicas).
 */
export function useSystemVolume() {
  const [state, setState] = useState({ value: null, muted: false });
  const holdUntil = useRef(0);
  const reconcile = useRef(null);

  const apply = useCallback((next) => {
    if (next && Date.now() >= holdUntil.current) setState({ value: next.value, muted: !!next.muted });
  }, []);
  const refresh = useCallback(() => invoke("get_volume").then(apply).catch(() => {}), [apply]);

  useEffect(() => {
    let disposed = false;
    let stop;
    listen("volume-state", ({ payload }) => apply(payload)).then((fn) => { if (disposed) fn(); else stop = fn; }).catch(() => {});
    refresh();
    window.addEventListener("focus", refresh);
    return () => {
      disposed = true;
      stop?.();
      window.removeEventListener("focus", refresh);
      clearTimeout(reconcile.current);
    };
  }, [apply, refresh]);

  const hold = useCallback(() => {
    holdUntil.current = Date.now() + ECHO_HOLD_MS;
    clearTimeout(reconcile.current);
    reconcile.current = setTimeout(refresh, ECHO_HOLD_MS + 20);
  }, [refresh]);

  const setVolume = useCallback((value) => {
    hold();
    // Igual que el backend: subir el volumen por encima de 0 quita el silencio.
    setState((s) => ({ value, muted: value > 0 ? false : s.muted }));
    invoke("set_volume", { value }).catch(() => {});
  }, [hold]);

  const setMuted = useCallback((muted) => {
    hold();
    setState((s) => ({ ...s, muted }));
    invoke("set_mute", { muted }).catch(() => {});
  }, [hold]);

  return { volume: state.value, muted: state.muted, setVolume, setMuted, refresh };
}
