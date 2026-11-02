import { useMemo } from "react";
import { useStore } from "../hooks/useStore.js";
import { dayKey } from "../lib/time.js";

const fmtMin = (m) => {
  const r = Math.round(m);
  if (r < 60) return `${r} min`;
  return `${Math.floor(r / 60)} h ${String(r % 60).padStart(2, "0")} min`;
};
const hourLabel = (h) => `${String(h).padStart(2, "0")}:00`;
const DAY = ["D", "L", "M", "X", "J", "V", "S"];

/** Últimos `n` días terminando hoy, del más antiguo al más reciente. */
const lastDays = (n) =>
  Array.from({ length: n }, (_, i) => {
    const d = new Date();
    d.setDate(d.getDate() - (n - 1 - i));
    return { key: dayKey(d), label: DAY[d.getDay()], today: i === n - 1 };
  });

/**
 * Estadísticas de enfoque: racha de días, minutos de la semana, horas más productivas y tareas a las que más
 * tiempo dedicaste. Sale de `pomoStats` (ciclos), `pomoTime` (minutos reales) y `screentime` (si está activado).
 */
export default function FocusStats() {
  const [stats] = useStore("pomoStats", {});
  const [time] = useStore("pomoTime", {});
  const [screen] = useStore("screentime", { days: {} });

  const data = useMemo(() => {
    const minOf = (key) => time[key]?.min ?? 0; // los ciclos de antes de existir `pomoTime` no tienen minutos
    const active = (key) => (stats[key] ?? 0) > 0 || (time[key]?.min ?? 0) > 0;
    const week = lastDays(7).map((d) => ({ ...d, min: minOf(d.key), cycles: stats[d.key] ?? 0 }));

    // Racha: días seguidos con enfoque hasta hoy (si hoy aún no hay, cuenta desde ayer sin romperla).
    let streak = 0;
    const cursor = new Date();
    if (!active(dayKey(cursor))) cursor.setDate(cursor.getDate() - 1);
    while (active(dayKey(cursor)) && streak < 3650) {
      streak += 1;
      cursor.setDate(cursor.getDate() - 1);
    }

    // Horas de enfoque (últimos 14 días); sin datos de enfoque se recurre al tiempo de pantalla.
    const hours = Array(24).fill(0);
    let fromFocus = false;
    for (const d of lastDays(14)) {
      const h = time[d.key]?.hours;
      if (h?.length === 24) h.forEach((v, i) => ((hours[i] += v), v > 0 && (fromFocus = true)));
    }
    if (!fromFocus) {
      for (const d of lastDays(14)) {
        const h = screen?.days?.[d.key]?.hours;
        if (h?.length === 24) h.forEach((v, i) => (hours[i] += v));
      }
    }
    const best = hours.map((v, h) => ({ h, v })).filter((x) => x.v > 0).sort((a, b) => b.v - a.v).slice(0, 3);

    const tasks = {};
    for (const d of lastDays(7)) {
      for (const [id, t] of Object.entries(time[d.key]?.byTask ?? {})) {
        tasks[id] = { title: t.title, min: (tasks[id]?.min ?? 0) + t.min };
      }
    }
    const topTasks = Object.values(tasks).sort((a, b) => b.min - a.min).slice(0, 3);

    const screenWeek = lastDays(7).reduce((s, d) => s + (screen?.days?.[d.key]?.total ?? 0), 0);
    const weekMin = week.reduce((s, d) => s + d.min, 0);
    const weekCycles = week.reduce((s, d) => s + d.cycles, 0);
    return { week, streak, best, fromFocus, topTasks, screenWeek, weekMin, weekCycles, todayMin: week[6].min };
  }, [stats, time, screen]);

  const top = Math.max(1, ...data.week.map((d) => d.min));
  const tile = "flex flex-col rounded-2xl bg-white/5 px-3 py-2";
  return (
    <div className="flex h-full min-h-0 flex-col gap-2 overflow-y-auto rounded-3xl bg-white/5 p-3">
      <div className="grid shrink-0 grid-cols-4 gap-2">
        <div className={tile}>
          <span className="text-lg font-semibold tabular-nums">🔥 {data.streak}</span>
          <span className="text-[10px] text-white/40">{data.streak === 1 ? "día de racha" : "días de racha"}</span>
        </div>
        <div className={tile}>
          <span className="text-lg font-semibold tabular-nums">{fmtMin(data.todayMin)}</span>
          <span className="text-[10px] text-white/40">hoy</span>
        </div>
        <div className={tile}>
          <span className="text-lg font-semibold tabular-nums">{fmtMin(data.weekMin)}</span>
          <span className="text-[10px] text-white/40">esta semana</span>
        </div>
        <div className={tile}>
          <span className="text-lg font-semibold tabular-nums">🍅 {data.weekCycles}</span>
          <span className="text-[10px] text-white/40">ciclos en 7 días</span>
        </div>
      </div>

      <div className="flex min-h-0 flex-1 gap-3">
        <div className="flex min-w-0 flex-1 flex-col">
          <span className="mb-1 text-[10px] uppercase tracking-wider text-white/35">Minutos por día</span>
          <div className="flex min-h-[64px] flex-1 items-end gap-1.5">
            {data.week.map((d) => (
              <div key={d.key} className="flex h-full flex-1 flex-col items-center justify-end gap-1" title={`${d.key}: ${fmtMin(d.min)}`}>
                <div
                  className="w-full rounded-t-md transition-all duration-500"
                  style={{ height: `${d.min ? Math.max(8, (d.min / top) * 100) : 3}%`, background: d.today ? "#fb7185" : d.min ? "rgba(251,113,133,0.45)" : "rgba(255,255,255,0.08)" }}
                />
                <span className={`text-[10px] ${d.today ? "text-white/80" : "text-white/35"}`}>{d.label}</span>
              </div>
            ))}
          </div>
        </div>

        <div className="flex w-[42%] shrink-0 flex-col gap-2 text-xs">
          <div>
            <span className="text-[10px] uppercase tracking-wider text-white/35">
              {data.fromFocus ? "Tus mejores horas de enfoque" : "Horas con más actividad"}
            </span>
            {data.best.length ? (
              <ul className="mt-1 space-y-0.5">
                {data.best.map((b) => (
                  <li key={b.h} className="flex justify-between text-white/80">
                    <span>{hourLabel(b.h)}–{hourLabel((b.h + 1) % 24)}</span>
                    <span className="tabular-nums text-white/40">{data.fromFocus ? fmtMin(b.v) : fmtMin(b.v / 60)}</span>
                  </li>
                ))}
              </ul>
            ) : (
              <div className="mt-1 text-white/30">Completa un enfoque para verlas.</div>
            )}
          </div>
          {data.topTasks.length > 0 && (
            <div>
              <span className="text-[10px] uppercase tracking-wider text-white/35">Tareas con más tiempo</span>
              <ul className="mt-1 space-y-0.5">
                {data.topTasks.map((t) => (
                  <li key={t.title} className="flex justify-between gap-2 text-white/80">
                    <span className="truncate">{t.title}</span>
                    <span className="shrink-0 tabular-nums text-white/40">{fmtMin(t.min)}</span>
                  </li>
                ))}
              </ul>
            </div>
          )}
          {data.screenWeek > 0 && (
            <div className="text-[11px] text-white/40">🖥️ Pantalla 7 días: {fmtMin(data.screenWeek / 60)}</div>
          )}
        </div>
      </div>
    </div>
  );
}
