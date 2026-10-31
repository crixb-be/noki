import { useMemo, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { useStore } from "../hooks/useStore.js";
import { PHASE_LABEL, fmtTime } from "../hooks/usePomodoro.js";
import { dayKey, fmtDue, nextOccurrence } from "../lib/time.js";
import { parseQuickTask } from "../lib/quickTask.js";
import Notes from "./Notes.jsx";
import Recorder from "./Recorder.jsx";

const chip = "rounded-full bg-white/10 px-3 py-1 text-xs transition-all duration-300 hover:bg-white/20 active:scale-95 disabled:opacity-30";
const inputCls = "min-w-0 rounded-full bg-white/10 px-3 py-1.5 text-xs outline-none placeholder:text-white/30 focus:bg-white/15";

// ───────────── Enfoque (Pomodoro) ─────────────

function FocusView({ pomo, prod }) {
  const { phase, left, total, running, cycle, doneToday, alert, start, pause, reset, switchPhase, skip } = pomo;
  const pct = total ? ((total - left) / total) * 100 : 0;
  const accent = phase === "work" ? "#fb7185" : "#34d399";
  const R = 52;
  const C = 2 * Math.PI * R;
  const dots = prod.cyclesBeforeLong;
  const filled = cycle % dots;

  return (
    <div className={`flex h-full items-center gap-6 rounded-3xl bg-white/5 px-6 ${alert ? "animate-[ringPulse_1.1s_ease-out_infinite] ring-2 ring-amber-400" : ""}`}>
      <div className="relative h-32 w-32 shrink-0">
        <svg viewBox="0 0 120 120" className="h-full w-full -rotate-90">
          <circle cx="60" cy="60" r={R} fill="none" stroke="rgba(255,255,255,0.1)" strokeWidth="7" />
          <circle
            cx="60" cy="60" r={R} fill="none" stroke={accent} strokeWidth="7" strokeLinecap="round"
            strokeDasharray={C} strokeDashoffset={C * (1 - pct / 100)}
            style={{ transition: "stroke-dashoffset 600ms linear" }}
          />
        </svg>
        <div className="absolute inset-0 flex flex-col items-center justify-center">
          <span className="text-3xl font-semibold tabular-nums" style={{ color: accent }}>{fmtTime(left)}</span>
          <span className="text-[10px] uppercase tracking-wider text-white/40">{PHASE_LABEL[phase]}</span>
        </div>
      </div>
      <div className="flex min-w-0 flex-1 flex-col gap-3">
        <div className="flex gap-1 text-[11px]">
          {Object.entries(PHASE_LABEL).map(([k, label]) => (
            <button
              key={k}
              onClick={() => switchPhase(k)}
              className={`rounded-full px-2.5 py-1 transition-all duration-300 ${phase === k ? "bg-white/20" : "text-white/40 hover:text-white/70"}`}
            >
              {label}
            </button>
          ))}
        </div>
        <div className="flex items-center gap-2">
          <button className={`${chip} px-4 py-1.5 text-sm`} onClick={running ? pause : start}>{running ? "⏸ Pausa" : "▶ Iniciar"}</button>
          <button className={chip} onClick={reset} aria-label="Reiniciar" title="Reiniciar fase">⟲</button>
          <button className={chip} onClick={skip} aria-label="Saltar" title="Saltar fase">⏭</button>
        </div>
        <div className="flex items-center gap-3 text-[11px] text-white/45">
          <span className="flex gap-1" title="Ciclos hasta el descanso largo">
            {Array.from({ length: dots }, (_, i) => (
              <span key={i} className={`h-1.5 w-4 rounded-full ${i < filled ? "bg-rose-400" : "bg-white/15"}`} />
            ))}
          </span>
          <span>Hoy: {doneToday} {doneToday === 1 ? "ciclo" : "ciclos"}</span>
        </div>
        <div className="text-[10px] text-white/30">
          {prod.workMin}/{prod.shortBreakMin}/{prod.longBreakMin} min · cambia los tiempos en Ajustes
        </div>
      </div>
    </div>
  );
}

// ───────────── Tareas ─────────────

function TasksView() {
  const [todos, setTodos] = useStore("todos", []);
  const [text, setTextState] = useState(() => sessionStorage.getItem("noki.taskDraft") ?? "");
  const [time, setTime] = useState("");
  const [date, setDate] = useState(dayKey());
  const [category, setCategory] = useState("Personal");
  const [filter, setFilter] = useState("all");
  const [categoryOpen, setCategoryOpen] = useState(false);
  const [showSchedule, setShowSchedule] = useState(false);
  const [taskView, setTaskView] = useState("active");
  const now = Date.now();
  const setText = (value) => {
    setTextState(value);
    if (value) sessionStorage.setItem("noki.taskDraft", value);
    else sessionStorage.removeItem("noki.taskDraft");
  };

  const localDateTime = (dateValue, timeValue) => {
    if (!dateValue || !timeValue) return null;
    const [year, month, day] = dateValue.split("-").map(Number);
    const [hours, minutes] = timeValue.split(":").map(Number);
    return new Date(year, month - 1, day, hours, minutes, 0, 0).getTime();
  };

  const setReminder = (timestamp) => {
    const d = new Date(timestamp);
    setDate(dayKey(d));
    setTime(`${String(d.getHours()).padStart(2, "0")}:${String(d.getMinutes()).padStart(2, "0")}`);
    setShowSchedule(true);
  };
  const remindIn = (minutes) => setReminder(Date.now() + minutes * 60000);
  const remindAt = (hours, tomorrow = false) => {
    const d = new Date();
    d.setHours(hours, 0, 0, 0);
    if (tomorrow || d.getTime() <= Date.now()) d.setDate(d.getDate() + 1);
    setReminder(d.getTime());
  };
  const toggleSchedule = () => {
    setShowSchedule((value) => {
      const next = !value;
      invoke("request_open", { open: true, sticky: next }).catch(() => {});
      return next;
    });
  };

  const add = (e) => {
    e.preventDefault();
    const raw = text.trim();
    if (!raw) return;
    // Sin hora elegida a mano, se entiende la sintaxis rápida: «llamar 15m», «reunión 14:30», «pagar mañana 9:00».
    const quick = showSchedule || time || date !== dayKey() ? null : parseQuickTask(raw, Date.now(), nextOccurrence);
    const t = quick ? quick.text : raw;
    const due = quick?.due ?? localDateTime(date, time);
    const dueDate = quick?.due ? dayKey(new Date(quick.due)) : date;
    setTodos((l) => [...l, { id: Date.now(), text: t, done: false, due, dueDate, category, notified: false }]);
    setText("");
    setTime("");
    setShowSchedule(false);
    invoke("request_open", { open: true, sticky: false }).catch(() => {});
  };
  const toggle = (id) => setTodos((l) => l.map((x) => (x.id === id ? { ...x, done: !x.done } : x)));
  const remove = (id) => setTodos((l) => l.filter((x) => x.id !== id));
  const snooze = (id) => setTodos((l) => l.map((x) => (x.id === id ? { ...x, due: Date.now() + 10 * 60000, notified: false } : x)));
  const clearDone = () => setTodos((l) => l.filter((x) => !x.done));

  const sorted = useMemo(
    () => [...todos]
      .filter((t) => taskView === "active" ? !t.done : taskView === "done" ? t.done : false)
      .filter((t) => filter === "all" || (t.category ?? "Personal") === filter)
      .sort((a, b) => a.done - b.done || (a.dueDate ?? dayKey()) .localeCompare(b.dueDate ?? dayKey()) || (a.due ?? Infinity) - (b.due ?? Infinity) || a.id - b.id),
    [todos, filter, taskView]
  );
  const doneCount = todos.filter((t) => t.done).length;
  const openCount = todos.filter((t) => !t.done).length;

  return (
    <div className="flex h-full flex-col gap-2 rounded-3xl bg-white/5 p-3">
      <form onSubmit={add} className="flex shrink-0 flex-col gap-1.5">
        <div className="flex items-center gap-1.5">
          <input data-task-input value={text} onChange={(e) => setText(e.target.value)} placeholder="Escribe una tarea…" className={`${inputCls} h-10 min-w-0 flex-1 rounded-xl`} />
          <button type="button" onMouseDown={() => !showSchedule && invoke("request_open", { open: true, sticky: true }).catch(() => {})} onClick={toggleSchedule} title="Programar fecha y hora" aria-label="Programar fecha y hora" className={`flex h-10 w-10 shrink-0 items-center justify-center rounded-xl text-lg transition-colors ${showSchedule ? "bg-sky-400/25 text-sky-100 ring-1 ring-sky-400/60" : "bg-white/10 text-white/70 hover:bg-white/15"}`}>◷</button>
          <button className="flex h-10 w-10 shrink-0 items-center justify-center rounded-xl bg-white/10 text-lg text-white/70 transition-colors hover:bg-white/15" aria-label="Agregar">＋</button>
        </div>
        <div className="flex items-center justify-between gap-2 overflow-x-auto">
          <div className="flex shrink-0 items-center gap-1">
            <span className="mr-0.5 text-[10px] text-white/35">Filtrar</span>
            {TASK_CATEGORIES.map((x) => (
              <button type="button" key={x} onClick={() => setFilter(x === "Todas" ? "all" : x)} className={`shrink-0 rounded-full px-1.5 py-1 text-[9px] transition-colors ${filter === (x === "Todas" ? "all" : x) ? "bg-sky-400/25 text-sky-100 ring-1 ring-sky-400/50" : "bg-white/5 text-white/55 hover:bg-white/10"}`}>{x}</button>
            ))}
          </div>
          <div className="flex shrink-0 items-center gap-1 rounded-xl bg-white/5 p-0.5">
            <button type="button" onClick={() => setTaskView("active")} className={`rounded-lg px-1.5 py-1 text-[9px] ${taskView === "active" ? "bg-white/15 text-white" : "text-white/45"}`}>Activas {openCount}</button>
            <button type="button" onClick={() => setTaskView("done")} className={`rounded-lg px-1.5 py-1 text-[9px] ${taskView === "done" ? "bg-white/15 text-white" : "text-white/45"}`}>Hechas {doneCount}</button>
            <button type="button" onClick={() => setTaskView("trash")} className={`rounded-lg px-1.5 py-1 text-[9px] ${taskView === "trash" ? "bg-white/15 text-white" : "text-white/45"}`}>Papelera</button>
          </div>
        </div>
        {showSchedule && (
          <div className="flex flex-wrap items-center gap-1.5 rounded-2xl border border-white/10 bg-white/5 px-2 py-1.5 text-[10px] text-white/45">
            <span className="mr-0.5">Recordarme</span>
            {[5, 15, 30].map((minutes) => <button type="button" key={minutes} onClick={() => remindIn(minutes)} className={chip}>{minutes}m</button>)}
            {[1, 2].map((hours) => <button type="button" key={hours} onClick={() => remindIn(hours * 60)} className={chip}>{hours}h</button>)}
            <button type="button" onClick={() => remindAt(20)} className={chip}>Hoy 20:00</button>
            <button type="button" onClick={() => remindAt(9, true)} className={chip}>Mañana 09:00</button>
            <input type="date" value={date} onChange={(e) => setDate(e.target.value)} title="Fecha de la tarea" className="w-[118px] rounded-full bg-white/10 px-2 py-1 text-[10px] text-white outline-none [color-scheme:dark]" />
            <input type="time" value={time} onChange={(e) => setTime(e.target.value)} title="Hora del aviso" className="w-[72px] rounded-full bg-white/10 px-2 py-1 text-[10px] text-white outline-none [color-scheme:dark]" />
            <div className="relative">
              <button type="button" onClick={() => setCategoryOpen((v) => !v)} title="Categoría de la tarea" className="rounded-full bg-white/10 px-2.5 py-1 text-[10px] text-white">{category}⌄</button>
              {categoryOpen && <div className="absolute bottom-full right-0 z-20 mb-1 min-w-[92px] overflow-hidden rounded-xl bg-zinc-900 p-1 shadow-xl ring-1 ring-white/15">{TASK_CATEGORIES.filter((x) => x !== "Todas").map((x) => <button type="button" key={x} onClick={() => { setCategory(x); setCategoryOpen(false); }} className={`block w-full rounded-lg px-2 py-1.5 text-left text-[11px] ${category === x ? "bg-sky-500/30 text-sky-100" : "text-white/70 hover:bg-white/10 hover:text-white"}`}>{x}</button>)}</div>}
            </div>
          </div>
        )}
      </form>
      <ul className="flex min-h-0 flex-1 flex-col gap-0.5 overflow-y-auto pr-1">
        {sorted.length === 0 && (
          <li className="py-6 text-center text-xs text-white/30">{taskView === "trash" ? "La papelera está vacía." : taskView === "done" ? "Aún no hay tareas completadas." : "Sin tareas para este filtro."}</li>
        )}
        {sorted.map((t) => {
          const overdue = t.due && !t.done && t.due <= now;
          const taskDate = t.dueDate ?? (t.due ? dayKey(new Date(t.due)) : null);
          return (
            <li key={t.id} className="group flex items-center gap-2 rounded-full px-2 py-1 text-xs hover:bg-white/5">
              <button type="button" onClick={() => toggle(t.id)} className={`flex h-5 w-5 shrink-0 items-center justify-center rounded-full border ${t.done ? "border-emerald-400 bg-emerald-400/20 text-emerald-300" : "border-white/45"}`} aria-label={t.done ? "Marcar pendiente" : "Completar tarea"}>{t.done ? "✓" : ""}</button>
              <span className={`min-w-0 flex-1 truncate ${t.done ? "text-white/30 line-through" : ""}`} title={t.text}>{t.text}</span>
              {t.category && <span className="shrink-0 rounded-full bg-white/10 px-2 py-0.5 text-[10px] text-white/55">{t.category}</span>}
              {taskDate && <span className="shrink-0 text-[10px] text-white/40">📅 {taskDate === dayKey() ? "Hoy" : taskDate}</span>}
              {t.due && !t.done && (
                <span className={`shrink-0 rounded-full px-2 py-0.5 text-[10px] tabular-nums ${overdue ? "bg-rose-500/25 text-rose-300" : "bg-white/10 text-white/60"}`}>
                  ⏰ {fmtDue(t.due, now)}
                </span>
              )}
              {overdue && (
                <button onClick={() => snooze(t.id)} className="shrink-0 text-[10px] text-sky-300 hover:underline" title="Posponer 10 minutos">+10 min</button>
              )}
              <button onClick={() => remove(t.id)} className="shrink-0 text-white/30 opacity-0 hover:text-red-400 group-hover:opacity-100" aria-label="Eliminar">✕</button>
            </li>
          );
        })}
      </ul>
      {doneCount > 0 && (
        <button onClick={clearDone} className="shrink-0 self-end text-[10px] text-white/35 hover:text-white/70">
          {`Limpiar ${doneCount} completada${doneCount > 1 ? "s" : ""}`}
        </button>
      )}
    </div>
  );
}

// ───────────── contenedor ─────────────

const TASK_CATEGORIES = ["Todas", "Personal", "Trabajo", "Uni", "Casa", "Otros"];

const SUBTABS = [
  { id: "tasks", label: "Tareas", icon: "✅" },
  { id: "notes", label: "Notas", icon: "📝" },
  { id: "voice", label: "Grabadora", icon: "🎙" },
  { id: "focus", label: "Enfoque", icon: "🍅" },
];

export default function Productivity({ pomo, prod }) {
  const [sub, setSub] = useStore("workTab", "tasks");
  const [order, setOrder] = useStore("workOrder", SUBTABS.map((t) => t.id));
  const [editing, setEditing] = useState(false);
  const tabs = SUBTABS.map((t) => t).sort((a, b) => {
    const rank = (id) => (order.includes(id) ? order.indexOf(id) : order.length);
    return rank(a.id) - rank(b.id);
  });
  const move = (id, dir) => {
    const ids = tabs.map((t) => t.id);
    const i = ids.indexOf(id);
    const j = i + dir;
    if (j < 0 || j >= ids.length) return;
    [ids[i], ids[j]] = [ids[j], ids[i]];
    setOrder(ids);
  };
  const arrow = "rounded-full px-1 text-[10px] text-white/50 transition-colors hover:text-white disabled:opacity-20";
  return (
    <div className="flex h-full flex-col gap-2">
      <div className="flex shrink-0 items-center gap-1">
        {tabs.map((t, i) => (
          <div key={t.id} className={`flex items-center rounded-full transition-all duration-300 ${sub === t.id ? "bg-white/20" : ""}`}>
            {editing && (
              <button className={arrow} onClick={() => move(t.id, -1)} disabled={i === 0} aria-label="Mover a la izquierda" title="Mover a la izquierda">◀</button>
            )}
            <button
              onClick={() => setSub(t.id)}
              className={`rounded-full px-3 py-1 text-[11px] transition-all duration-300 ${sub === t.id ? "" : "text-white/45 hover:text-white/80"}`}
            >
              {t.icon} {t.label}
            </button>
            {editing && (
              <button className={arrow} onClick={() => move(t.id, 1)} disabled={i === tabs.length - 1} aria-label="Mover a la derecha" title="Mover a la derecha">▶</button>
            )}
          </div>
        ))}
        <button
          onClick={() => setEditing(!editing)}
          className={`ml-auto rounded-full px-2 py-1 text-[11px] transition-all duration-300 ${editing ? "bg-white/20" : "text-white/35 hover:text-white/70"}`}
          title={editing ? "Listo" : "Reordenar pestañas"}
          aria-label={editing ? "Listo" : "Reordenar pestañas"}
        >
          {editing ? "✓" : "⇄"}
        </button>
      </div>
      <div className="min-h-0 flex-1">
        {sub === "focus" && <FocusView pomo={pomo} prod={prod} />}
        {sub === "tasks" && <TasksView />}
        {sub === "notes" && <Notes />}
        {sub === "voice" && <Recorder />}
      </div>
    </div>
  );
}
