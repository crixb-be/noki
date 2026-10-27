import { useMemo, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { useStore } from "../hooks/useStore.js";
import { dayKey } from "../lib/time.js";

const COLORS = ["#60a5fa", "#a78bfa", "#34d399", "#fbbf24", "#f472b6", "#22d3ee", "#fb923c", "#a3e635"];
const colorOf = (name) => COLORS[[...name].reduce((s, c) => s + c.charCodeAt(0), 0) % COLORS.length];

const fmt = (secs) => {
  const m = Math.round(secs / 60);
  if (m < 1) return secs > 0 ? "<1 min" : "0 min";
  const h = Math.floor(m / 60);
  return h ? `${h} h ${String(m % 60).padStart(2, "0")} min` : `${m} min`;
};

const WEEKDAYS = ["Dom", "Lun", "Mar", "Mié", "Jue", "Vie", "Sáb"];

/** Últimos `n` días (el más antiguo primero) con su clave local AAAA-MM-DD. */
function lastDays(n) {
  const out = [];
  for (let i = n - 1; i >= 0; i--) {
    const d = new Date();
    d.setDate(d.getDate() - i);
    out.push({ key: dayKey(d), label: WEEKDAYS[d.getDay()] });
  }
  return out;
}

function mergeApps(days) {
  const total = {};
  for (const d of days) for (const [app, s] of Object.entries(d?.apps ?? {})) total[app] = (total[app] ?? 0) + s;
  return Object.entries(total).sort((a, b) => b[1] - a[1]);
}

function AppList({ apps, total }) {
  if (!apps.length) return <div className="py-6 text-center text-xs text-white/30">Aún no hay datos. Se irán sumando mientras usas el PC.</div>;
  return (
    <ul className="flex flex-col gap-1.5">
      {apps.slice(0, 6).map(([name, s]) => (
        <li key={name} className="flex items-center gap-2 text-xs">
          <span className="w-24 shrink-0 truncate" title={name}>{name}</span>
          <span className="h-2 min-w-0 flex-1 overflow-hidden rounded-full bg-white/10">
            <span className="block h-full rounded-full transition-all duration-500" style={{ width: `${Math.max(2, (s / Math.max(1, total)) * 100)}%`, background: colorOf(name) }} />
          </span>
          <span className="w-16 shrink-0 text-right tabular-nums text-white/60">{fmt(s)}</span>
        </li>
      ))}
    </ul>
  );
}

function Bars({ values, labels, highlight }) {
  const top = Math.max(1, ...values);
  return (
    <div className="flex h-16 items-end gap-[3px]">
      {values.map((v, i) => (
        <div key={i} className="flex h-full min-w-0 flex-1 flex-col items-center justify-end gap-0.5" title={`${labels?.[i] ?? ""} ${fmt(v)}`}>
          <div
            className="w-full rounded-sm transition-all duration-500"
            style={{ height: `${v ? Math.max(6, (v / top) * 100) : 3}%`, background: i === highlight ? "#60a5fa" : v ? "rgba(96,165,250,0.45)" : "rgba(255,255,255,0.08)" }}
          />
        </div>
      ))}
    </div>
  );
}

export default function ScreenTime() {
  const [data] = useStore("screentime", { days: {} });
  const [range, setRange] = useStore("screentimeRange", "today");
  const [confirm, setConfirm] = useState(false);
  const days = data?.days ?? {};
  const today = days[dayKey()];

  const week = useMemo(() => lastDays(7).map((d) => ({ ...d, total: days[d.key]?.total ?? 0 })), [data]); // eslint-disable-line react-hooks/exhaustive-deps
  const weekTotal = week.reduce((s, d) => s + d.total, 0);
  const weekApps = useMemo(() => mergeApps(week.map((d) => days[d.key])), [data]); // eslint-disable-line react-hooks/exhaustive-deps
  const todayApps = useMemo(() => mergeApps([today]), [data]); // eslint-disable-line react-hooks/exhaustive-deps
  const hours = today?.hours?.length === 24 ? today.hours : Array(24).fill(0);

  const clear = async () => {
    await invoke("screentime_clear").catch(() => {});
    setConfirm(false);
  };

  return (
    <div className="flex h-full min-h-0 flex-col gap-2 rounded-3xl bg-white/5 p-3">
      <div className="flex shrink-0 items-center justify-between">
        <div className="flex gap-1">
          {[["today", "Hoy"], ["week", "7 días"]].map(([id, label]) => (
            <button key={id} onClick={() => setRange(id)}
              className={`rounded-full px-3 py-1 text-[11px] transition-all duration-300 ${range === id ? "bg-white/20" : "text-white/45 hover:text-white/80"}`}>
              {label}
            </button>
          ))}
        </div>
        <div className="text-right">
          <span className="text-lg font-semibold tabular-nums">{fmt(range === "today" ? today?.total ?? 0 : weekTotal)}</span>
          <span className="ml-1.5 text-[10px] text-white/40">{range === "today" ? "hoy" : `· prom. ${fmt(weekTotal / 7)}/día`}</span>
        </div>
      </div>

      <div className="min-h-0 flex-1 overflow-y-auto pr-1">
        {range === "today" ? (
          <div className="flex flex-col gap-3">
            <div>
              <Bars values={hours} labels={hours.map((_, i) => `${i}:00`)} highlight={new Date().getHours()} />
              <div className="mt-0.5 flex justify-between text-[9px] text-white/30"><span>0h</span><span>6h</span><span>12h</span><span>18h</span><span>24h</span></div>
            </div>
            <AppList apps={todayApps} total={today?.total ?? 0} />
          </div>
        ) : (
          <div className="flex flex-col gap-3">
            <div>
              <Bars values={week.map((d) => d.total)} labels={week.map((d) => d.label)} highlight={6} />
              <div className="mt-0.5 flex text-[9px] text-white/40">{week.map((d) => <span key={d.key} className="flex-1 text-center">{d.label}</span>)}</div>
            </div>
            <AppList apps={weekApps} total={weekTotal} />
          </div>
        )}
      </div>

      <div className="flex shrink-0 items-center justify-between text-[10px] text-white/30">
        <span>Solo en este equipo · no cuenta cuando estás ausente 2 min</span>
        {confirm ? (
          <span className="flex gap-1">
            <button className="rounded-full bg-red-500/30 px-2 py-0.5 text-white hover:bg-red-500/50" onClick={clear}>Borrar todo</button>
            <button className="rounded-full bg-white/10 px-2 py-0.5 hover:bg-white/20" onClick={() => setConfirm(false)}>Cancelar</button>
          </span>
        ) : (
          <button className="rounded-full px-2 py-0.5 hover:bg-white/10" onClick={() => setConfirm(true)}>Borrar historial</button>
        )}
      </div>
    </div>
  );
}
