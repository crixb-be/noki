import { useEffect, useRef } from "react";
import { invoke } from "@tauri-apps/api/core";
import { useStore } from "./useStore.js";

const CHECK_MS = 15000;

/**
 * Recordatorios en segundo plano: tareas con hora vencida y avisos de salud.
 * Funciona mientras la app esté abierta (el notch vive en la bandeja, así que casi siempre).
 */
export function useReminders({ health, notify, leader = true }) {
  const [todos, setTodos] = useStore("todos", []);
  // Salud: se cuenta tiempo de uso real; si te alejas del PC (inactividad) los contadores se reinician.
  const healthAt = useRef({ eyes: Date.now(), stand: Date.now() });
  const latest = useRef({});
  latest.current = { health, notify, todos };

  useEffect(() => {
    const check = async () => {
      const { health: h, notify: say, todos: list } = latest.current;
      const now = Date.now();

      if (h.eyes || h.stand) {
        const idle = await invoke("idle_seconds").catch(() => 0);
        const t = healthAt.current;
        if (idle > 300) {
          t.eyes = now;
          t.stand = now;
        } else {
          if (h.eyes && now - t.eyes >= h.eyesEveryMin * 60000) {
            t.eyes = now;
            say({ icon: "👀", text: "Descansa la vista: mira algo a 6 m durante 20 s", tone: "alert", ms: 9000 });
          }
          if (h.stand && now - t.stand >= h.standEveryMin * 60000) {
            t.stand = now;
            say({ icon: "🧍", text: "Levántate y estírate un momento", tone: "alert", ms: 9000, silent: now - t.eyes < 2000 });
          }
        }
      }

      // Tareas con hora: se avisa una vez al vencer.
      const due = list.filter((t) => !t.done && t.due && !t.notified && t.due <= now);
      if (due.length) {
        setTodos((l) => l.map((t) => (due.some((d) => d.id === t.id) ? { ...t, notified: true } : t)));
        due.forEach((t, i) => say({ icon: "⏰", text: t.text, tone: "alert", ms: 10000, silent: i > 0 }));
      }

    };
    if (!leader) return; // los avisos los genera solo la ventana principal
    const first = setTimeout(check, 3000);
    const id = setInterval(check, CHECK_MS);
    return () => {
      clearTimeout(first);
      clearInterval(id);
    };
  }, [setTodos, leader]);
}
