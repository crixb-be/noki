import { useEffect, useRef } from "react";
import { listen } from "@tauri-apps/api/event";

/** Escucha un evento de Tauri mientras `enabled`; `handler` siempre es el último, sin reenganchar el listener. */
export function useTauriEvent(name, handler, enabled = true) {
  const ref = useRef(handler);
  ref.current = handler;
  useEffect(() => {
    if (!enabled) return;
    let unlisten;
    let cancelled = false;
    listen(name, (e) => ref.current(e.payload)).then((fn) => (cancelled ? fn() : (unlisten = fn)));
    return () => {
      cancelled = true;
      unlisten?.();
    };
  }, [name, enabled]);
}
