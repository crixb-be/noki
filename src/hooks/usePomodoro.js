import { useCallback, useEffect, useRef, useState } from "react";
import { useStore } from "./useStore.js";
import { dayKey } from "../lib/time.js";

export const PHASE_LABEL = { work: "Enfoque", short: "Descanso", long: "Descanso largo" };

const seconds = (phase, p) =>
  (phase === "work" ? p.workMin : phase === "short" ? p.shortBreakMin : p.longBreakMin) * 60;

export const fmtTime = (s) =>
  `${String(Math.floor(s / 60)).padStart(2, "0")}:${String(s % 60).padStart(2, "0")}`;

const IDLE = { phase: "work", running: false, endAt: null, pausedLeft: null, cycle: 0, taskId: null, taskTitle: "" };

/**
 * Suma minutos de enfoque al registro del día (`pomoTime`): total, por tarea y por hora del día.
 * `pomoStats` sigue contando ciclos completos; aquí además cuenta el tiempo real (también el de sesiones cortadas).
 */
export function addFocusTime(prev, minutes, task, at = new Date()) {
  if (!(minutes > 0)) return prev;
  const day = dayKey(at);
  const cur = prev[day] ?? { min: 0, byTask: {}, hours: Array(24).fill(0) };
  const byTask = { ...cur.byTask };
  if (task?.id != null) {
    const t = byTask[task.id] ?? { title: task.title, min: 0 };
    byTask[task.id] = { title: task.title || t.title, min: Math.round((t.min + minutes) * 10) / 10 };
  }
  const hours = cur.hours?.length === 24 ? [...cur.hours] : Array(24).fill(0);
  hours[at.getHours()] = Math.round((hours[at.getHours()] + minutes) * 10) / 10;
  return { ...prev, [day]: { min: Math.round((cur.min + minutes) * 10) / 10, byTask, hours } };
}

/**
 * Pomodoro configurable y persistente. Usa una hora de fin (`endAt`) en vez de restar un segundo
 * por tick, así sigue exacto aunque la ventana esté oculta y sobrevive a reiniciar la app.
 */
export function usePomodoro(prod, notify, leader = true) {
  const [st, setSt] = useStore("pomo", IDLE);
  const [stats, setStats] = useStore("pomoStats", {});
  const [, setTime] = useStore("pomoTime", {});
  const [now, setNow] = useState(Date.now());
  const [alert, setAlert] = useState(false);
  const alertTimer = useRef(null);
  const latest = useRef({ st, prod, notify });
  latest.current = { st, prod, notify };

  const total = seconds(st.phase, prod);
  const left = st.running
    ? Math.max(0, Math.ceil((st.endAt - now) / 1000))
    : Math.min(st.pausedLeft ?? total, total);

  useEffect(() => {
    if (!st.running) return;
    const id = setInterval(() => setNow(Date.now()), 500);
    return () => clearInterval(id);
  }, [st.running]);

  const begin = useCallback((phase, secs) => ({
    phase,
    running: true,
    endAt: Date.now() + secs * 1000,
    pausedLeft: null,
  }), []);

  // Fin de fase.
  useEffect(() => {
    // Con varios notch solo la ventana principal cierra las fases (si no, se contarían dos veces).
    if (!leader || !st.running || left > 0) return;
    const { prod: p, notify: say } = latest.current;
    let cycle = st.cycle;
    let next = "work";
    if (st.phase === "work") {
      cycle += 1;
      next = cycle % p.cyclesBeforeLong === 0 ? "long" : "short";
      const day = dayKey();
      setStats((s) => ({ ...s, [day]: (s[day] ?? 0) + 1 }));
      setTime((t) => addFocusTime(t, p.workMin, { id: st.taskId, title: st.taskTitle }));
    }
    const nextSecs = seconds(next, p);
    // La tarea vinculada se conserva entre fases: el siguiente bloque de enfoque sigue siendo para ella.
    const task = { taskId: st.taskId ?? null, taskTitle: st.taskTitle ?? "" };
    setSt(p.autoNext ? { ...begin(next, nextSecs), cycle, ...task } : { phase: next, running: false, endAt: null, pausedLeft: null, cycle, ...task });
    setNow(Date.now());

    setAlert(true);
    clearTimeout(alertTimer.current);
    alertTimer.current = setTimeout(() => setAlert(false), 6000);
    say?.({
      icon: st.phase === "work" ? "☕" : "🎯",
      text:
        st.phase === "work"
          ? `Enfoque terminado · ${next === "long" ? "descanso largo" : "descanso"} de ${Math.round(nextSecs / 60)} min`
          : "Descanso terminado · a enfocarse",
      tone: "alert",
      ms: 7000,
    });
  }, [left, st.running]); // eslint-disable-line react-hooks/exhaustive-deps

  useEffect(() => () => clearTimeout(alertTimer.current), []);

  const start = useCallback(() => {
    setAlert(false);
    setSt((s) => ({ ...s, ...begin(s.phase, s.pausedLeft ?? seconds(s.phase, latest.current.prod)) }));
    setNow(Date.now());
  }, [begin, setSt]);

  const pause = useCallback(() => {
    setSt((s) => ({ ...s, running: false, pausedLeft: Math.max(1, Math.ceil((s.endAt - Date.now()) / 1000)), endAt: null }));
  }, [setSt]);

  /** Detiene la sesión. Si era de enfoque, guarda los minutos que llevaba (≥ 1) para la tarea vinculada. */
  const reset = useCallback(() => {
    setAlert(false);
    const s = latest.current.st;
    if (s.phase === "work" && (s.running || s.pausedLeft != null)) {
      const total = seconds("work", latest.current.prod);
      const leftNow = s.running ? Math.max(0, Math.ceil((s.endAt - Date.now()) / 1000)) : s.pausedLeft;
      const mins = Math.floor((total - leftNow) / 60);
      if (mins >= 1) setTime((t) => addFocusTime(t, mins, { id: s.taskId, title: s.taskTitle }));
    }
    setSt((x) => ({ ...x, running: false, endAt: null, pausedLeft: null }));
  }, [setSt, setTime]);

  /** Tarea a la que se dedica el enfoque (`null` = enfoque libre). */
  const setTask = useCallback((task) => setSt((x) => ({ ...x, taskId: task?.id ?? null, taskTitle: task?.text ?? "" })), [setSt]);

  const switchPhase = useCallback(
    (phase) => {
      setAlert(false);
      setSt((s) => ({ ...s, phase, running: false, endAt: null, pausedLeft: null }));
    },
    [setSt]
  );

  /** Salta a la siguiente fase sin contar el ciclo. */
  const skip = useCallback(() => {
    setAlert(false);
    setSt((s) => {
      const next = s.phase === "work" ? "short" : "work";
      return { ...s, phase: next, running: false, endAt: null, pausedLeft: null };
    });
  }, [setSt]);

  const doneToday = stats[dayKey()] ?? 0;
  const paused = !st.running && st.pausedLeft != null;
  return { phase: st.phase, left, total, running: st.running, paused, taskId: st.taskId, taskTitle: st.taskTitle, setTask, cycle: st.cycle, doneToday, alert, start, pause, reset, switchPhase, skip };
}
