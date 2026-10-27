import { useMemo, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { useStore } from "../hooks/useStore.js";
import { dayKey, fmtClock } from "../lib/time.js";
import { locale, weekdayLetters } from "../lib/i18n.js";

const dayLabel = (key, now) => {
  const today = dayKey(new Date(now));
  const tomorrow = dayKey(new Date(now + 86400000));
  if (key === today) return "Hoy";
  if (key === tomorrow) return "Mañana";
  const [y, m, d] = key.split("-").map(Number);
  const s = new Date(y, m - 1, d).toLocaleDateString(locale(), { weekday: "long", day: "numeric", month: "long" });
  return s.charAt(0).toUpperCase() + s.slice(1);
};

export const untilText = (ms, now) => {
  const m = Math.round((ms - now) / 60000);
  if (m <= 0) return "ahora";
  if (m < 60) return `en ${m} min`;
  const h = Math.floor(m / 60);
  return `en ${h} h${m % 60 ? ` ${m % 60} min` : ""}`;
};

const monthTitle = (date) => date.toLocaleDateString(locale(), { month: "long", year: "numeric" }).replace(/^./, (c) => c.toUpperCase());

export default function CalendarView({ cal }) {
  const { events, allEvents = events, ongoing, errors, loading, refresh, hasSources, now } = cal;
  const [todos] = useStore("todos", []);
  const [cursor, setCursor] = useState(() => new Date(now));
  const [selected, setSelected] = useState(dayKey(new Date(now)));

  const month = useMemo(() => new Date(cursor.getFullYear(), cursor.getMonth(), 1), [cursor]);
  const cells = useMemo(() => {
    const offset = (month.getDay() + 6) % 7;
    const count = new Date(month.getFullYear(), month.getMonth() + 1, 0).getDate();
    return [...Array(offset).fill(null), ...Array.from({ length: count }, (_, i) => new Date(month.getFullYear(), month.getMonth(), i + 1))];
  }, [month]);

  const pendingByDay = useMemo(() => {
    const map = new Map();
    todos.filter((t) => !t.done).forEach((t) => {
      const key = t.dueDate ?? (t.due ? dayKey(new Date(t.due)) : null);
      if (key) map.set(key, (map.get(key) ?? 0) + 1);
    });
    return map;
  }, [todos]);

  const eventsByDay = useMemo(() => {
    const map = new Map();
    allEvents.forEach((e) => {
      const key = dayKey(new Date(e.start));
      map.set(key, (map.get(key) ?? 0) + 1);
    });
    return map;
  }, [allEvents]);

  const selectedTasks = todos.filter((t) => !t.done && (t.dueDate ?? (t.due ? dayKey(new Date(t.due)) : null)) === selected);
  const selectedEvents = allEvents.filter((e) => dayKey(new Date(e.start)) === selected);

  const changeMonth = (delta) => {
    const next = new Date(month.getFullYear(), month.getMonth() + delta, 1);
    setCursor(next);
    setSelected(dayKey(next));
  };
  const goToday = () => {
    const today = new Date(now);
    setCursor(today);
    setSelected(dayKey(today));
  };

  return (
    <div className="flex h-full min-h-0 gap-2 overflow-hidden">
      {/* Mitad izquierda: el mes */}
      <div className="flex min-h-0 w-1/2 shrink-0 flex-col gap-1 rounded-3xl bg-white/5 p-2">
        <div className="flex shrink-0 items-center justify-between">
          <div className="flex items-center">
            <button onClick={() => changeMonth(-1)} className="rounded-full px-1.5 py-0.5 text-white/55 hover:bg-white/10" aria-label="Mes anterior">‹</button>
            <button onClick={goToday} className="rounded-full px-1.5 py-1 text-center text-[11px] font-medium hover:bg-white/10" title="Ir a hoy">{monthTitle(month)}</button>
            <button onClick={() => changeMonth(1)} className="rounded-full px-1.5 py-0.5 text-white/55 hover:bg-white/10" aria-label="Mes siguiente">›</button>
          </div>
          <div className="flex items-center text-[11px] text-white/40">
            <button onClick={() => invoke("open_settings", { section: "calendar" })} className="rounded-full px-1.5 py-0.5 hover:bg-white/10" title="Configurar calendarios" aria-label="Configurar calendarios">⚙</button>
            <button onClick={refresh} disabled={loading} className="rounded-full px-1.5 py-0.5 hover:bg-white/10 disabled:opacity-40" title="Actualizar calendario">{loading ? "…" : "↻"}</button>
          </div>
        </div>
        <div className="grid shrink-0 grid-cols-7 gap-px text-center text-[9px] uppercase leading-3 tracking-wider text-white/35">
          {weekdayLetters().map((label, i) => <span key={i}>{label}</span>)}
        </div>
        <div className="grid min-h-0 flex-1 grid-cols-7 content-start gap-px">
          {cells.map((date, i) => {
            if (!date) return <span key={`empty-${i}`} className="h-[26px]" />;
            const key = dayKey(date);
            const taskCount = pendingByDay.get(key) ?? 0;
            const eventCount = eventsByDay.get(key) ?? 0;
            const marked = taskCount > 0 || eventCount > 0;
            const isToday = key === dayKey(new Date(now));
            return (
              <button key={key} onClick={() => setSelected(key)} className={`relative flex h-[26px] items-center justify-center rounded-lg text-[11px] transition-colors ${selected === key ? "bg-sky-400/25 text-white ring-1 ring-sky-400/70" : isToday ? "bg-white/10 text-sky-200" : "text-white/65 hover:bg-white/10"}`} title={`${taskCount} tarea(s), ${eventCount} evento(s)`}>
                {date.getDate()}
                {marked && <span className={`absolute bottom-0.5 h-1 w-1 rounded-full ${taskCount ? "bg-red-400" : "bg-sky-300"}`} />}
              </button>
            );
          })}
        </div>
      </div>

      {/* Mitad derecha: pendientes y eventos del día elegido */}
      <div className="flex min-h-0 min-w-0 flex-1 flex-col rounded-3xl bg-white/5 p-2.5">
        <div className="mb-1.5 flex shrink-0 items-center justify-between text-[11px] text-white/55">
          <span className="truncate font-medium text-white/75">{dayLabel(selected, now)}</span>
          {selectedTasks.length > 0 && <span className="shrink-0 text-red-300">{`${selectedTasks.length} pendiente${selectedTasks.length === 1 ? "" : "s"}`}</span>}
        </div>
        <div className="min-h-0 flex-1 overflow-y-auto overscroll-contain pr-1">
          {!hasSources && <div className="mb-2 text-[10px] text-white/30">Conecta Google Calendar u Outlook desde Ajustes para ver sus eventos.</div>}
          {selectedTasks.length === 0 && selectedEvents.length === 0 && <div className="py-6 text-center text-[11px] text-white/30">Nada pendiente para esta fecha.</div>}
          <ul className="flex flex-col gap-1">
            {selectedTasks.map((t) => (
              <li key={`task-${t.id}`} className="flex items-center gap-2 rounded-xl bg-red-400/10 px-2 py-1.5 text-[11px]">
                <span className="h-1.5 w-1.5 shrink-0 rounded-full bg-red-400" />
                <span className="min-w-0 flex-1 truncate" title={t.text}>{t.text}</span>
                {t.category && <span className="shrink-0 text-[9px] text-white/40">{t.category}</span>}
              </li>
            ))}
            {selectedEvents.map((e) => (
              <li key={`event-${e.id}`} className="flex items-center gap-2 rounded-xl bg-white/5 px-2 py-1.5 text-[11px]">
                <span className="h-1.5 w-1.5 shrink-0 rounded-full bg-sky-300" />
                <span className="w-10 shrink-0 text-white/40">{e.allDay ? "Todo" : fmtClock(e.start)}</span>
                <span className="min-w-0 flex-1 truncate" title={e.title}>{e.title}</span>
              </li>
            ))}
          </ul>
          {errors.map((er) => <div key={er} className="mt-2 rounded-xl bg-amber-500/10 px-2 py-1 text-[10px] text-amber-300">⚠ {er}</div>)}
        </div>
      </div>
    </div>
  );
}
