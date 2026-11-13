import { useCallback, useEffect, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

const isObj = (v) => v && typeof v === "object" && !Array.isArray(v);

const DEFAULTS = {
  topbar: { enabled: false, height: 30, tint: "auto", showCpu: false, showRam: false, showWifi: true, showBluetooth: true, showSound: true, showBattery: true, showBatteryPct: true, showSearch: true, showControlCenter: true, showCapture: true, showNotifications: true, showAvatar: true, showClock: true, avatarPath: "" },
  dock: { enabled: false, iconSize: 48, magnify: true, magnifyScale: 1.6, showApps: true, showRunning: true, showTrash: true, autohide: false, hideTaskbar: false, reserveSpace: false, pinned: [], folders: [] },
};

/** Fusión profunda: `patch` solo pisa las claves que trae. */
export function merge(base, patch) {
  const out = { ...base };
  for (const [k, v] of Object.entries(patch)) out[k] = isObj(v) && isObj(base?.[k]) ? merge(base[k], v) : v;
  return out;
}

/**
 * Ajustes compartidos entre ventanas. Rust es la fuente de verdad: guarda en %APPDATA%,
 * aplica los cambios (layout, atajo, autoinicio) y emite `settings-updated` a todas las ventanas.
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
      });
  }, []);

  useEffect(() => {
    const unlisten = [];
    let cancelled = false;
    invoke("get_settings").then((s) => {
      if (cancelled) return;
      const merged = merge(merge({}, DEFAULTS), s);
      latest.current = merged;
      setSettings(merged);
    });
    const apply = (payload) => {
      const merged = merge(merge({}, DEFAULTS), payload);
      latest.current = merged;
      setSettings(merged);
    };
    listen("settings-updated", (e) => {
      // Este evento es la fuente de verdad: se aplica incluso mientras termina el guardado local.
      apply(e.payload);
    }).then((fn) => (cancelled ? fn() : unlisten.push(fn)));
    listen("settings-changed", (e) => {
      // Compatibilidad con versiones anteriores; no pisa un slider local en curso.
      if (timer.current || inFlight.current) return;
      apply(e.payload);
    }).then((fn) => (cancelled ? fn() : unlisten.push(fn)));
    return () => {
      cancelled = true;
      unlisten.forEach((fn) => fn());
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
      const merged = merge(merge({}, DEFAULTS), s);
      latest.current = merged;
      setSettings(merged);
  }, []);

  return { settings, update, reset, warning };
}
