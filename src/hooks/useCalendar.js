import { useCallback, useEffect, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { useStore } from "./useStore.js";

const REFRESH_MS = 15 * 60 * 1000;
const CHECK_MS = 30 * 1000;

/**
 * Eventos de los calendarios iCal configurados (Rust descarga y expande las repeticiones).
 * Refresca cada 15 min y al cambiar las direcciones, avisa antes de cada reunión y expone la
 * próxima para el estado compacto del notch.
 */
export function useCalendar({ calendar, enabled, notify, leader = true }) {
  const [cache, setCache] = useStore("calCache", { events: [], at: 0, key: "" });
  const [notified, setNotified] = useStore("calNotified", {});
  const [errors, setErrors] = useState([]);
  const [loading, setLoading] = useState(false);
  const [now, setNow] = useState(Date.now());
  const latest = useRef({});
  latest.current = { calendar, notify, notified, cache, leader };

  const key = JSON.stringify(calendar.sources.map((s) => [s.name, s.url]));
  const hasSources = calendar.sources.some((s) => s.url);

  const refresh = useCallback(async () => {
    if (!hasSources) {
      setCache({ events: [], at: Date.now(), key });
      setErrors([]);
      return;
    }
    setLoading(true);
    try {
      const r = await invoke("calendar_fetch");
      setErrors(r.errors);
      // Si todo falló se conserva lo último que se sabía en vez de vaciar la lista.
      if (r.events.length || !r.errors.length) setCache({ events: r.events, at: Date.now(), key });
    } catch (e) {
      setErrors([String(e)]);
    } finally {
      setLoading(false);
    }
  }, [hasSources, key, setCache]);

  useEffect(() => {
    if (!enabled || !leader) return;
    const stale = latest.current.cache.key !== key || Date.now() - latest.current.cache.at > REFRESH_MS;
    if (stale) refresh();
    const id = setInterval(refresh, REFRESH_MS);
    return () => clearInterval(id);
  }, [enabled, leader, key, refresh]);

  // Reloj interno + aviso previo a cada reunión.
  useEffect(() => {
    const id = setInterval(() => {
      const t = Date.now();
      setNow(t);
      const { calendar: c, notify: say, notified: done, cache: ca, leader: isLeader } = latest.current;
      if (!isLeader || !c.reminderMin) return; // el aviso previo lo genera solo la ventana principal
      const due = ca.events.filter(
        (e) => !e.allDay && !done[e.id] && e.start > t && e.start - t <= c.reminderMin * 60000
      );
      if (!due.length) return;
      setNotified((m) => {
        const next = { ...m };
        due.forEach((e) => (next[e.id] = e.start));
        // Se olvidan los avisos de eventos que ya pasaron hace más de un día.
        Object.keys(next).forEach((k) => next[k] < t - 86400000 && delete next[k]);
        return next;
      });
      due.forEach((e, i) =>
        say({
          icon: "📅",
          text: `${e.title} · en ${Math.max(1, Math.round((e.start - t) / 60000))} min`,
          tone: "alert",
          ms: 9000,
          silent: i > 0,
        })
      );
    }, CHECK_MS);
    return () => clearInterval(id);
  }, [setNotified]);

  const upcoming = cache.events.filter((e) => e.end > now || (e.end === e.start && e.start > now - 60000));
  const ongoing = upcoming.find((e) => !e.allDay && e.start <= now && e.end > now);
  const next = upcoming.find((e) => !e.allDay && e.start > now);

  return { events: upcoming, allEvents: cache.events, ongoing, next, errors, loading, refresh, hasSources, now };
}
