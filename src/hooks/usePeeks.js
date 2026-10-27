import { useCallback, useEffect, useRef, useState } from "react";
import { listen } from "@tauri-apps/api/event";

const langName = (tag) => {
  try {
    const n = new Intl.DisplayNames([navigator.language || "es"], { type: "language" }).of(tag);
    return n.charAt(0).toUpperCase() + n.slice(1);
  } catch {
    return tag;
  }
};

/**
 * Avisos temporales ("peeks") del notch cerrado: volumen, brillo, cargador, internet,
 * dispositivo de audio, Bloq Mayús e idioma. Rust emite los eventos del sistema; aquí se
 * filtran según los ajustes y se decide qué mostrar y por cuánto tiempo.
 */
export function usePeeks({ hud, battery, dnd }) {
  const [peek, setPeek] = useState(null);
  const cfg = useRef(hud);
  cfg.current = hud;
  // No molestar: se silencian los avisos automáticos; el volumen y el brillo (acción tuya) se siguen mostrando.
  const quiet = useRef(dnd);
  quiet.current = dnd;
  const timer = useRef(null);
  const hold = useRef(false); // el cursor está sobre el aviso o se arrastra su slider
  const prevCharging = useRef(null);
  const lowWarned = useRef(100); // último umbral de batería baja avisado (20 o 10); 100 = ninguno

  const arm = useCallback((ms) => {
    clearTimeout(timer.current);
    const tick = () => {
      if (hold.current) timer.current = setTimeout(tick, 500);
      else setPeek(null);
    };
    timer.current = setTimeout(tick, ms ?? cfg.current?.duration ?? 2000);
  }, []);

  const show = useCallback(
    (p, ms) => {
      if (quiet.current) return;
      setPeek(p);
      arm(ms);
    },
    [arm]
  );

  const setHold = useCallback((v) => {
    hold.current = v;
  }, []);

  useEffect(() => {
    const subs = [];
    const on = (name, fn) => {
      let un;
      let dead = false;
      listen(name, (e) => cfg.current && fn(e.payload)).then((f) => (dead ? f() : (un = f)));
      subs.push(() => {
        dead = true;
        un?.();
      });
    };

    on("hud", ({ kind, value, muted }) => {
      if (!cfg.current[kind]) return;
      // Mientras se arrastra el slider del propio aviso solo se mantiene vivo, sin pisar el valor.
      setPeek((cur) => (hold.current && cur?.kind === kind ? cur : { kind, value, muted }));
      arm();
    });
    on("audio-device", ({ name, kind }) => cfg.current.audioDevice && show({ kind: "text", icon: kind === "headphones" ? "🎧" : "🔊", text: name }));
    on("caps-lock", (isOn) => cfg.current.capsLock && show({ kind: "text", icon: "⇪", text: isOn ? "Bloq Mayús activado" : "Bloq Mayús desactivado", tone: isOn ? "warn" : undefined }));
    on("keyboard-language", (tag) => cfg.current.language && show({ kind: "text", icon: "🌐", text: langName(tag) }));

    const online = () => cfg.current?.network && show({ kind: "text", icon: "📶", text: "Conexión restablecida" });
    const offline = () => cfg.current?.network && show({ kind: "text", icon: "📵", text: "Sin conexión a internet", tone: "warn" });
    window.addEventListener("online", online);
    window.addEventListener("offline", offline);

    return () => {
      subs.forEach((f) => f());
      window.removeEventListener("online", online);
      window.removeEventListener("offline", offline);
      clearTimeout(timer.current);
    };
  }, [arm, show]);

  // Cargador conectado / desconectado (la primera lectura solo fija el estado inicial).
  useEffect(() => {
    if (!battery) return;
    if (prevCharging.current !== null && prevCharging.current !== battery.charging && cfg.current?.charger) {
      show({
        kind: "text",
        icon: battery.charging ? "⚡" : "🔌",
        text: battery.charging ? `Cargando · ${battery.level}%` : `Cargador desconectado · ${battery.level}%`,
        tone: !battery.charging && battery.level <= 20 ? "warn" : undefined,
      });
    }
    prevCharging.current = battery.charging;
  }, [battery?.charging]); // eslint-disable-line react-hooks/exhaustive-deps

  // Batería baja: un aviso al cruzar el 20 % y otro al 10 %, solo sin cargador. Se rearma al cargar.
  useEffect(() => {
    if (!battery) return;
    if (battery.charging || battery.level > 20) {
      if (battery.charging || battery.level > 25) lowWarned.current = 100;
      return;
    }
    const threshold = battery.level <= 10 ? 10 : 20;
    if (threshold < lowWarned.current) {
      lowWarned.current = threshold;
      if (cfg.current?.lowBattery) {
        show({ kind: "text", icon: "🪫", text: `Batería baja · ${battery.level}%`, tone: "warn" }, 4000);
      }
    }
  }, [battery?.level, battery?.charging]); // eslint-disable-line react-hooks/exhaustive-deps

  return { peek, setHold, show };
}
