import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { useStore } from "../hooks/useStore.js";

const chip = "rounded-full bg-white/10 px-3 py-1 text-xs transition-all duration-300 hover:bg-white/20 active:scale-95 disabled:opacity-40";

const gb = (b) => `${(b / 1024 ** 3).toFixed(b >= 100 * 1024 ** 3 ? 0 : 1)} GB`;
const mb = (b) => (b >= 1024 ** 3 ? `${(b / 1024 ** 3).toFixed(1)} GB` : `${Math.round(b / 1024 ** 2)} MB`);
const rate = (bps) => {
  if (bps < 1024) return `${Math.round(bps)} B/s`;
  if (bps < 1024 ** 2) return `${(bps / 1024).toFixed(0)} KB/s`;
  return `${(bps / 1024 ** 2).toFixed(1)} MB/s`;
};
const uptime = (s) => {
  const d = Math.floor(s / 86400);
  const h = Math.floor((s % 86400) / 3600);
  const m = Math.floor((s % 3600) / 60);
  return d ? `${d} d ${h} h` : h ? `${h} h ${m} min` : `${m} min`;
};
const tone = (pct) => (pct >= 90 ? "#fb7185" : pct >= 70 ? "#fbbf24" : "#34d399");
const clean = (name) => name.replace(/\.exe$/i, "");

function Ring({ pct, label, sub }) {
  const R = 30;
  const C = 2 * Math.PI * R;
  return (
    <div className="flex shrink-0 flex-col items-center gap-0.5">
      <div className="relative h-[72px] w-[72px]">
        <svg viewBox="0 0 72 72" className="h-full w-full -rotate-90">
          <circle cx="36" cy="36" r={R} fill="none" stroke="rgba(255,255,255,0.1)" strokeWidth="6" />
          <circle
            cx="36" cy="36" r={R} fill="none" stroke={tone(pct)} strokeWidth="6" strokeLinecap="round"
            strokeDasharray={C} strokeDashoffset={C * (1 - Math.min(100, pct) / 100)}
            style={{ transition: "stroke-dashoffset 500ms ease, stroke 500ms" }}
          />
        </svg>
        <div className="absolute inset-0 flex flex-col items-center justify-center">
          <span className="text-base font-semibold tabular-nums">{Math.round(pct)}%</span>
          <span className="text-[9px] uppercase tracking-wider text-white/40">{label}</span>
        </div>
      </div>
      <div className="h-3 whitespace-nowrap text-[9px] text-white/40">{sub ?? " "}</div>
    </div>
  );
}

function Spark({ data, color }) {
  const max = Math.max(1, ...data);
  const pts = data.map((v, i) => `${(i / Math.max(1, data.length - 1)) * 100},${28 - (v / max) * 26}`).join(" ");
  return (
    <svg viewBox="0 0 100 30" preserveAspectRatio="none" className="h-7 w-full">
      <polyline points={pts} fill="none" stroke={color} strokeWidth="1.5" vectorEffect="non-scaling-stroke" />
    </svg>
  );
}

// ───────────── Rendimiento ─────────────

function Performance() {
  const [s, setS] = useState(null);
  const [cpuHist, setCpuHist] = useState([]);
  const [netHist, setNetHist] = useState([]);

  useEffect(() => {
    let alive = true;
    const tick = async () => {
      try {
        const r = await invoke("system_stats");
        if (!alive) return;
        setS(r);
        setCpuHist((h) => [...h.slice(-39), r.cpu]);
        setNetHist((h) => [...h.slice(-39), r.netDown + r.netUp]);
      } catch {
        /* se reintenta en el siguiente ciclo */
      }
    };
    tick();
    const id = setInterval(tick, 1500);
    return () => {
      alive = false;
      clearInterval(id);
    };
  }, []);

  if (!s) return <div className="flex h-full items-center justify-center text-xs text-white/40">Midiendo…</div>;
  const memPct = (s.memUsed / s.memTotal) * 100;

  return (
    <div className="flex h-full min-h-0 gap-3 overflow-y-auto rounded-3xl bg-white/5 p-3">
      <div className="flex shrink-0 flex-col items-center justify-center gap-3">
        <Ring pct={s.cpu} label="CPU" />
        <Ring pct={memPct} label="RAM" sub={`${gb(s.memUsed)} / ${gb(s.memTotal)}`} />
      </div>
      <div className="flex min-w-0 flex-1 flex-col gap-2.5">
        <div>
          <div className="flex justify-between text-[10px] text-white/40"><span>CPU</span><span className="tabular-nums">{Math.round(s.cpu)}%</span></div>
          <Spark data={cpuHist} color={tone(s.cpu)} />
        </div>
        <div>
          <div className="flex justify-between text-[10px] text-white/40">
            <span>Red</span>
            <span className="tabular-nums">↓ {rate(s.netDown)} · ↑ {rate(s.netUp)}</span>
          </div>
          <Spark data={netHist} color="#60a5fa" />
        </div>
        <div className="space-y-1.5">
          {s.disks.slice(0, 3).map((d) => {
            const pct = (d.used / d.total) * 100;
            return (
              <div key={d.name}>
                <div className="flex justify-between text-[10px] text-white/45">
                  <span>Disco {d.name}</span>
                  <span className="tabular-nums">{gb(d.used)} / {gb(d.total)}</span>
                </div>
                <div className="mt-0.5 h-1.5 overflow-hidden rounded-full bg-white/10">
                  <div className="h-full rounded-full transition-all duration-500" style={{ width: `${pct}%`, background: tone(pct) }} />
                </div>
              </div>
            );
          })}
        </div>
        <div className="text-[10px] text-white/35">Encendido hace {uptime(s.uptime)}</div>
      </div>
    </div>
  );
}

// ───────────── Procesos ─────────────

function Processes() {
  const [by, setBy] = useStore("procSort", "mem");
  const [list, setList] = useState(null);
  const [confirm, setConfirm] = useState(null); // nombre del programa pendiente de confirmar
  const [msg, setMsg] = useState(null);

  useEffect(() => {
    let alive = true;
    const tick = async () => {
      try {
        const r = await invoke("top_processes", { by });
        if (alive) setList(r);
      } catch {
        /* se reintenta */
      }
    };
    tick();
    const id = setInterval(tick, 2500);
    return () => {
      alive = false;
      clearInterval(id);
    };
  }, [by]);

  const end = async (name) => {
    if (confirm !== name) {
      setConfirm(name);
      setTimeout(() => setConfirm((c) => (c === name ? null : c)), 3000);
      return;
    }
    setConfirm(null);
    try {
      const n = await invoke("end_task", { name });
      setMsg(`Cerrado: ${clean(name)}${n > 1 ? ` (${n} procesos)` : ""}`);
    } catch (e) {
      setMsg(String(e));
    }
    setTimeout(() => setMsg(null), 3500);
  };

  const top = list?.[0] ? (by === "cpu" ? Math.max(1, list[0].cpu) : list[0].mem) : 1;

  return (
    <div className="flex h-full min-h-0 flex-col gap-1.5 rounded-3xl bg-white/5 p-3">
      <div className="flex shrink-0 items-center justify-between">
        <div className="flex gap-1">
          {[["mem", "Memoria"], ["cpu", "CPU"]].map(([id, label]) => (
            <button key={id} onClick={() => setBy(id)} className={`rounded-full px-2.5 py-0.5 text-[11px] ${by === id ? "bg-white/20" : "text-white/45 hover:text-white/80"}`}>
              {label}
            </button>
          ))}
        </div>
        <span className="truncate text-[10px] text-white/35">{msg ?? "Programas que más consumen"}</span>
      </div>
      <ul className="flex min-h-0 flex-1 flex-col gap-1 overflow-y-auto pr-1">
        {!list && <li className="py-6 text-center text-xs text-white/35">Midiendo…</li>}
        {list?.map((p) => {
          const frac = by === "cpu" ? p.cpu / top : p.mem / top;
          return (
            <li key={p.name} className="group flex items-center gap-2 rounded-xl px-2 py-1 text-xs hover:bg-white/5">
              <span className="w-28 shrink-0 truncate" title={p.name}>{clean(p.name)}{p.count > 1 && <span className="text-white/35"> ×{p.count}</span>}</span>
              <span className="h-1.5 min-w-0 flex-1 overflow-hidden rounded-full bg-white/10">
                <span className="block h-full rounded-full bg-sky-400/70 transition-all duration-500" style={{ width: `${Math.max(2, frac * 100)}%` }} />
              </span>
              <span className="w-16 shrink-0 text-right tabular-nums text-white/65">{by === "cpu" ? `${p.cpu.toFixed(1)}%` : mb(p.mem)}</span>
              <button
                onClick={() => end(p.name)}
                className={`w-[58px] shrink-0 rounded-full py-0.5 text-[10px] transition-all ${confirm === p.name ? "bg-red-500/70 text-white" : "bg-white/10 text-white/60 opacity-0 hover:bg-white/20 group-hover:opacity-100"}`}
                title="Cerrar este programa"
              >
                {confirm === p.name ? "¿Seguro?" : "Cerrar"}
              </button>
            </li>
          );
        })}
      </ul>
    </div>
  );
}

// ───────────── contenedor ─────────────

const SUBTABS = [
  { id: "perf", label: "Rendimiento", icon: "📊" },
  { id: "proc", label: "Procesos", icon: "🧮" },
];

export default function SystemView() {
  const [saved, setSub] = useStore("systemTab", "perf");
  const sub = SUBTABS.some((t) => t.id === saved) ? saved : "perf";
  return (
    <div className="flex h-full flex-col gap-2">
      <div className="flex shrink-0 gap-1">
        {SUBTABS.map((t) => (
          <button
            key={t.id}
            onClick={() => setSub(t.id)}
            className={`rounded-full px-3 py-1 text-[11px] transition-all duration-300 ${sub === t.id ? "bg-white/20" : "text-white/45 hover:text-white/80"}`}
          >
            {t.icon} {t.label}
          </button>
        ))}
      </div>
      <div className="min-h-0 flex-1">{sub === "perf" ? <Performance /> : <Processes />}</div>
    </div>
  );
}
