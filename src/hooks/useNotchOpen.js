import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

/** Rust decide cuándo abrir/cerrar (hover, atajo, clic fuera); aquí solo se refleja. */
export function useNotchOpen(label) {
  const [open, setOpen] = useState(false);
  useEffect(() => {
    let unlisten;
    let cancelled = false;
    invoke("notch_state").then((o) => !cancelled && setOpen(o)).catch(() => {});
    listen("notch-open", (e) => e.payload.label === label && setOpen(e.payload.open)).then((fn) => (cancelled ? fn() : (unlisten = fn)));
    return () => {
      cancelled = true;
      unlisten?.();
    };
  }, [label]);
  return open;
}
