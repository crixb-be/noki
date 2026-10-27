import { useCallback, useEffect, useRef, useState } from "react";
import { useStore } from "./useStore.js";
import { dayKey } from "../lib/time.js";

export const PHASE_LABEL = { work: "Enfoque", short: "Descanso", long: "Descanso largo" };

const seconds = (phase, p) =>
  (phase === "work" ? p.workMin : phase === "short" ? p.shortBreakMin : p.longBreakMin) * 60;

export const fmtTime = (s) =>
  `${String(Math.floor(s / 60)).padStart(2, "0")}:${String(s % 60).padStart(2, "0")}`;

const IDLE = { phase: "work", running: false, endAt: null, pausedLeft: null, cycle: 0 };

/**
 * Pomodoro configurable y persistente. Usa una hora de fin (`endAt`) en vez de restar un segundo
 * por tick, así sigue exacto aunque la ventana esté oculta y sobrevive a reiniciar la app.
 */
export function usePomodoro(prod, notify, leader = true) {
  const [st, setSt] = useStore("pomo", IDLE);
  const [stats, setStats] = useStore("pomoStats", {});
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
    }
    const nextSecs = seconds(next, p);
    setSt(p.autoNext ? { ...begin(next, nextSecs), cycle } : { phase: next, running: false, endAt: null, pausedLeft: null, cycle });
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

  const reset = useCallback(() => {
    setAlert(false);
    setSt((s) => ({ ...s, running: false, endAt: null, pausedLeft: null }));
  }, [setSt]);

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
  return { phase: st.phase, left, total, running: st.running, cycle: st.cycle, doneToday, alert, start, pause, reset, switchPhase, skip };
}
