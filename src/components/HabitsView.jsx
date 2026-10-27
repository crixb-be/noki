import { useState } from "react";
import { useStore } from "../hooks/useStore.js";
import { EMPTY_HABITS, countOn, slotsToday, streakOf } from "../hooks/useHabits.js";
import { dayKey } from "../lib/time.js";

const ICONS = ["💧", "💊", "🚶", "📖", "🧘", "🍎", "😴", "🏋️", "🦷", "🥗", "☀️", "✍️", "🧠", "🌿", "🐶", "📵"];

const PRESETS = [
  { name: "Tomar agua", icon: "💧", goal: 8, reminder: { mode: "every", every: 2, from: "08:00", to: "20:00" } },
  { name: "Tomar pastilla", icon: "💊", goal: 1, reminder: { mode: "times", times: ["08:00"] } },
  { name: "Caminar", icon: "🚶", goal: 1, reminder: { mode: "none" } },
  { name: "Leer", icon: "📖", goal: 1, reminder: { mode: "times", times: ["21:00"] } },
  { name: "Meditar", icon: "🧘", goal: 1, reminder: { mode: "none" } },
  { name: "Comer fruta", icon: "🍎", goal: 2, reminder: { mode: "none" } },
];

const chip = "rounded-full bg-white/10 px-3 py-1 text-xs transition-all duration-300 hover:bg-white/20 active:scale-95 disabled:opacity-30";
const field = "min-w-0 rounded-full bg-white/10 px-3 py-1.5 text-xs outline-none placeholder:text-white/30 focus:bg-white/15 [color-scheme:dark]";

const blank = () => ({ id: Date.now(), name: "", icon: "💧", goal: 1, active: true, reminder: { mode: "none", every: 2, from: "08:00", to: "20:00", times: ["08:00"] } });

function Editor({ draft, setDraft }) {
  const r = draft.reminder;
  const setR = (patch) => setDraft({ ...draft, reminder: { ...r, ...patch } });
  const times = r.times ?? [];
  return (
    <div className="flex min-h-0 flex-1 flex-col gap-2 overflow-y-auto rounded-2xl bg-black/20 p-3 text-xs">
      <div className="flex gap-1.5">
        <input value={draft.name} onChange={(e) => setDraft({ ...draft, name: e.target.value })} placeholder="Nombre del hábito (ej. Tomar agua)" className={`${field} flex-1`} autoFocus />
        <label className="flex shrink-0 items-center gap-1.5 text-white/60" title="Veces por día">
          Meta
          <input type="number" min={1} max={30} value={draft.goal} onChange={(e) => setDraft({ ...draft, goal: Math.max(1, Math.min(30, Number(e.target.value) || 1)) })} className={`${field} w-14 text-center`} />
        </label>
      </div>
      <div className="flex flex-wrap gap-1">
        {ICONS.map((i) => (
          <button key={i} onClick={() => setDraft({ ...draft, icon: i })} className={`h-6 w-6 rounded-lg text-sm transition-colors ${draft.icon === i ? "bg-sky-400/30 ring-1 ring-sky-400" : "bg-white/5 hover:bg-white/10"}`}>
            {i}
          </button>
        ))}
      </div>
      <div className="flex flex-wrap items-center gap-1.5">
        <span className="text-white/50">Recordar</span>
        <select value={r.mode} onChange={(e) => setR({ mode: e.target.value })} className={field}>
          <option value="none" className="bg-neutral-900">Nunca</option>
          <option value="every" className="bg-neutral-900">Cada cierto tiempo</option>
          <option value="times" className="bg-neutral-900">A estas horas</option>
        </select>
        {r.mode === "every" && (
          <>
            <span className="text-white/50">cada</span>
            <input type="number" min={0.5} max={12} step={0.5} value={r.every} onChange={(e) => setR({ every: Math.max(0.5, Number(e.target.value) || 1) })} className={`${field} w-14 text-center`} />
            <span className="text-white/50">h, de</span>
            <input type="time" value={r.from} onChange={(e) => setR({ from: e.target.value })} className={`${field} w-[84px]`} />
            <span className="text-white/50">a</span>
            <input type="time" value={r.to} onChange={(e) => setR({ to: e.target.value })} className={`${field} w-[84px]`} />
          </>
        )}
        {r.mode === "times" && (
          <>
            {times.map((t, i) => (
              <span key={i} className="flex items-center gap-0.5">
                <input type="time" value={t} onChange={(e) => setR({ times: times.map((x, j) => (j === i ? e.target.value : x)) })} className={`${field} w-[84px]`} />
                {times.length > 1 && <button onClick={() => setR({ times: times.filter((_, j) => j !== i) })} className="text-white/40 hover:text-red-400" aria-label="Quitar hora">✕</button>}
              </span>
            ))}
            {times.length < 8 && <button onClick={() => setR({ times: [...times, "12:00"] })} className={chip}>＋ Hora</button>}
          </>
        )}
      </div>
      <label className="flex items-center gap-2 text-white/60">
        <input type="checkbox" checked={draft.active !== false} onChange={(e) => setDraft({ ...draft, active: e.target.checked })} />
        Activo (se muestra y avisa; apágalo para pausarlo sin borrarlo)
      </label>
    </div>
  );
}

export default function HabitsView() {
  const [habits, setHabits] = useStore("habits", EMPTY_HABITS);
  const [draft, setDraft] = useState(null); // hábito que se crea o edita
  const today = dayKey();
  const items = habits.items ?? [];
  const active = items.filter((i) => i.active !== false);
  const doneCount = active.filter((i) => countOn(habits, i.id) >= i.goal).length;

  const bump = (item, delta) =>
    setHabits((h) => {
      const log = { ...(h.log ?? {}) };
      const day = { ...(log[today] ?? {}) };
      day[item.id] = Math.max(0, (day[item.id] ?? 0) + delta);
      log[today] = day;
      // Solo se guardan los últimos 120 días.
      const keys = Object.keys(log).sort();
      if (keys.length > 120) keys.slice(0, keys.length - 120).forEach((k) => delete log[k]);
      return { ...h, log };
    });

  const save = () => {
    const clean = { ...draft, name: draft.name.trim() };
    setHabits((h) => {
      const exists = (h.items ?? []).some((i) => i.id === clean.id);
      return { ...h, items: exists ? h.items.map((i) => (i.id === clean.id ? clean : i)) : [...(h.items ?? []), clean] };
    });
    setDraft(null);
  };
  const remove = () => {
    setHabits((h) => ({ ...h, items: h.items.filter((i) => i.id !== draft.id) }));
    setDraft(null);
  };

  if (draft) {
    const isNew = !items.some((i) => i.id === draft.id);
    return (
      <div className="flex h-full flex-col gap-2 rounded-3xl bg-white/5 p-3">
        <div className="flex shrink-0 items-center gap-1.5">
          <span className="mr-auto text-xs font-medium text-white/70">{isNew ? "Nuevo hábito" : "Editar hábito"}</span>
          {!isNew && <button className={`${chip} hover:bg-red-500/50`} onClick={remove}>🗑 Borrar</button>}
          <button className={chip} onClick={() => setDraft(null)}>Cancelar</button>
          <button className={`${chip} bg-sky-400/25 hover:bg-sky-400/40`} onClick={save} disabled={!draft.name.trim()}>Guardar</button>
        </div>
        <Editor draft={draft} setDraft={setDraft} />
      </div>
    );
  }

  return (
    <div className="flex h-full min-h-0 flex-col gap-2 rounded-3xl bg-white/5 p-3">
      <div className="flex shrink-0 items-center justify-between">
        <div className="text-xs text-white/60">
          {active.length ? (
            <>Hoy: <span className="font-semibold text-white">{doneCount}/{active.length}</span> completados</>
          ) : (
            "Crea tus hábitos y el notch te lo recuerda"
          )}
        </div>
        <button className={chip} onClick={() => setDraft(blank())}>＋ Nuevo</button>
      </div>

      {items.length === 0 && (
        <div className="flex min-h-0 flex-1 flex-col items-center justify-center gap-3 text-center">
          <div className="text-xs text-white/40">Empieza con uno de estos o crea el tuyo</div>
          <div className="flex flex-wrap justify-center gap-1.5">
            {PRESETS.map((p) => (
              <button key={p.name} className={chip} onClick={() => setDraft({ ...blank(), ...p, reminder: { ...blank().reminder, ...p.reminder } })}>
                {p.icon} {p.name}
              </button>
            ))}
          </div>
        </div>
      )}

      <ul className="flex min-h-0 flex-1 flex-col gap-1.5 overflow-y-auto pr-1">
        {items.map((item) => {
          const n = countOn(habits, item.id);
          const done = n >= item.goal;
          const streak = streakOf(habits, item);
          const off = item.active === false;
          const next = slotsToday(item).find((t) => t > Date.now());
          return (
            <li key={item.id} className={`group flex items-center gap-3 rounded-2xl px-3 py-2 transition-colors ${done ? "bg-emerald-400/10" : "bg-white/5"} ${off ? "opacity-40" : ""}`}>
              <span className="text-2xl leading-none">{item.icon}</span>
              <div className="min-w-0 flex-1">
                <div className="flex items-center gap-2 text-xs">
                  <span className="truncate font-medium">{item.name}</span>
                  {streak > 0 && <span className="shrink-0 text-[10px] text-amber-300" title="Días seguidos cumpliendo la meta">🔥 {streak}</span>}
                  <button onClick={() => setDraft({ ...item, reminder: { mode: "none", every: 2, from: "08:00", to: "20:00", times: ["08:00"], ...item.reminder } })} className="shrink-0 text-white/30 opacity-0 transition-opacity hover:text-white group-hover:opacity-100" aria-label="Editar">✎</button>
                </div>
                <div className="mt-1 flex items-center gap-1.5">
                  <div className="h-1.5 min-w-0 flex-1 overflow-hidden rounded-full bg-white/10">
                    <div className="h-full rounded-full transition-all duration-500" style={{ width: `${Math.min(100, (n / item.goal) * 100)}%`, background: done ? "#34d399" : "#60a5fa" }} />
                  </div>
                  <span className="shrink-0 text-[10px] tabular-nums text-white/55">{n}/{item.goal}</span>
                </div>
                {next && !off && !done && <div className="mt-0.5 text-[9px] text-white/30">Próximo aviso: {new Date(next).toLocaleTimeString([], { hour: "2-digit", minute: "2-digit" })}</div>}
              </div>
              <div className="flex shrink-0 items-center gap-1">
                <button onClick={() => bump(item, -1)} disabled={n === 0 || off} className="h-7 w-7 rounded-full bg-white/10 text-sm text-white/60 hover:bg-white/20 disabled:opacity-25" aria-label="Quitar una">−</button>
                <button onClick={() => bump(item, 1)} disabled={off} className={`h-8 w-8 rounded-full text-base font-semibold transition-all active:scale-90 disabled:opacity-25 ${done ? "bg-emerald-400/30 text-emerald-200" : "bg-sky-400/30 text-sky-100 hover:bg-sky-400/50"}`} aria-label="Sumar una">
                  {done ? "✓" : "＋"}
                </button>
              </div>
            </li>
          );
        })}
      </ul>
    </div>
  );
}
