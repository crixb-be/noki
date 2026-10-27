import { useEffect, useRef } from "react";
import { useStore } from "./useStore.js";
import { dayKey } from "../lib/time.js";

const CHECK_MS = 20000;
/** Un recordatorio solo se muestra si su hora pasó hace menos de esto (así no llegan en bloque al abrir la app). */
const FRESH_MS = 20 * 60000;

export const EMPTY_HABITS = { items: [], log: {} };

/** Cuántas veces se hizo hoy un hábito. */
export const countOn = (habits, id, day = dayKey()) => habits.log?.[day]?.[id] ?? 0;

/** Días seguidos cumpliendo la meta. Si hoy aún no se cumple, se cuenta desde ayer. */
export function streakOf(habits, item) {
  let n = 0;
  const d = new Date();
  if (countOn(habits, item.id, dayKey(d)) < item.goal) d.setDate(d.getDate() - 1);
  for (let i = 0; i < 400; i++) {
    if (countOn(habits, item.id, dayKey(d)) < item.goal) break;
    n++;
    d.setDate(d.getDate() - 1);
  }
  return n;
}

const toMin = (hhmm) => {
  const [h, m] = String(hhmm).split(":").map(Number);
  return Number.isFinite(h) && Number.isFinite(m) ? h * 60 + m : null;
};

/** Horas (en ms de hoy) a las que toca avisar un hábito. */
export function slotsToday(item, now = new Date()) {
  const r = item.reminder ?? { mode: "none" };
  const base = new Date(now.getFullYear(), now.getMonth(), now.getDate()).getTime();
  let mins = [];
  if (r.mode === "times") {
    mins = (r.times ?? []).map(toMin).filter((m) => m != null);
  } else if (r.mode === "every") {
    const from = toMin(r.from ?? "08:00") ?? 480;
    const to = toMin(r.to ?? "22:00") ?? 1320;
    const step = Math.max(15, Math.round((r.every ?? 2) * 60));
    for (let m = from; m <= to; m += step) mins.push(m);
  }
  return mins.map((m) => base + m * 60000).sort((a, b) => a - b);
}

/**
 * Recordatorios de hábitos (agua, pastillas…): solo la ventana principal los genera. Cada hábito activo avisa
 * en sus horas mientras no haya cumplido la meta del día.
 */
export function useHabits({ enabled, notify, leader = true }) {
  const [habits] = useStore("habits", EMPTY_HABITS);
  const [fired, setFired] = useStore("habitsFired", {}); // id → última hora (ms) ya avisada
  const latest = useRef({});
  latest.current = { habits, fired, notify, setFired };

  useEffect(() => {
    if (!enabled || !leader) return;
    const check = () => {
      const { habits: h, fired: f, notify: say, setFired: save } = latest.current;
      const now = Date.now();
      const updates = {};
      let first = true;
      for (const item of h.items ?? []) {
        if (item.active === false || !item.reminder || item.reminder.mode === "none") continue;
        const done = countOn(h, item.id);
        if (done >= item.goal) continue;
        const due = slotsToday(item).filter((t) => t <= now && now - t <= FRESH_MS && t > (f[item.id] ?? 0));
        if (!due.length) continue;
        updates[item.id] = due[due.length - 1];
        say({ icon: item.icon, text: `${item.name} · ${done}/${item.goal}`, tone: "alert", ms: 9000, silent: !first });
        first = false;
      }
      if (Object.keys(updates).length) save({ ...f, ...updates });
    };
    const first = setTimeout(check, 4000);
    const id = setInterval(check, CHECK_MS);
    return () => {
      clearTimeout(first);
      clearInterval(id);
    };
  }, [enabled, leader]);
}
