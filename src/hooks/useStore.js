import { useCallback, useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";

/**
 * Almacén persistente en Rust (`data.json`). Se carga una vez al arrancar; después cada clave
 * se lee y escribe de forma síncrona contra una caché y se sincroniza entre los componentes
 * que la usan.
 */
let cache = {};
let loading = null;
const listeners = new Map(); // clave → Set<fn>

const write = (key, value) => {
  cache[key] = value;
  invoke("store_set", { key, value }).catch(() => {});
  listeners.get(key)?.forEach((fn) => fn(value));
};

/** Datos de versiones anteriores, que vivían en localStorage. */
function migrateLegacy() {
  const legacy = [
    ["notch.files", "files"],
    ["notch.todos", "todos"],
  ];
  for (const [old, key] of legacy) {
    try {
      const raw = localStorage.getItem(old);
      if (raw === null) continue;
      // Solo se importa si el almacén aún no tiene esa clave, y el dato antiguo se borra solo tras importarlo.
      if (cache[key] !== undefined) continue;
      let value = JSON.parse(raw);
      write(key, value);
      localStorage.removeItem(old);
    } catch {
      /* dato antiguo ilegible: se ignora */
    }
  }
}

export function loadStore() {
  loading ??= invoke("store_all")
    .then((all) => {
      cache = all ?? {};
      migrateLegacy();
      // Con varios notch (uno por pantalla), los cambios de otra ventana llegan aquí sin volver a guardarse.
      const me = getCurrentWindow().label;
      listen("store-changed", (e) => {
        const { key, value, source } = e.payload;
        if (source === me) return; // el cambio lo hizo esta misma ventana
        cache[key] = value;
        listeners.get(key)?.forEach((fn) => fn(value));
      }).catch(() => {});
    })
    .catch(() => {});
  return loading;
}

export function useStoreReady() {
  const [ready, setReady] = useState(false);
  useEffect(() => {
    loadStore().then(() => setReady(true));
  }, []);
  return ready;
}

export function useStore(key, initial) {
  const [value, setValue] = useState(() => cache[key] ?? initial);

  useEffect(() => {
    if (!listeners.has(key)) listeners.set(key, new Set());
    listeners.get(key).add(setValue);
    return () => listeners.get(key).delete(setValue);
  }, [key]);

  const set = useCallback(
    (v) => {
      const prev = cache[key] ?? initial;
      write(key, typeof v === "function" ? v(prev) : v);
    },
    // eslint-disable-next-line react-hooks/exhaustive-deps
    [key]
  );
  return [value, set];
}

