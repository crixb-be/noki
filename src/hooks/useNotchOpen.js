import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { useTauriEvent } from "./useTauriEvent.js";

/** Etiqueta de esta ventana: "main" o "main-1", "main-2"… (un notch por pantalla). */
export const LABEL = getCurrentWindow().label;

/** Rust decide cuándo abrir/cerrar (hover, atajo, clic fuera); aquí solo se refleja. */
export function useNotchOpen() {
  const [open, setOpen] = useState(false);
  useEffect(() => {
    let cancelled = false;
    invoke("notch_state").then((o) => !cancelled && setOpen(o)).catch(() => {});
    return () => {
      cancelled = true;
    };
  }, []);
  useTauriEvent("notch-open", (p) => p.label === LABEL && setOpen(p.open));
  return open;
}
