import { invoke } from "@tauri-apps/api/core";
import { fmtTime } from "../../hooks/usePomodoro.js";
import { SOUNDS, cycleAmbient, useAmbient } from "../../lib/ambient.js";

/** Alto extra (px) que se despliega bajo el notch al pasar el cursor: fila de controles. */
export const ZEN_EXTRA_H = 68;

const PLAY = "M8 5v14l11-7z";
const PAUSE = "M6 5h4v14H6zM14 5h4v14h-4z";
const STOP = "M7 7h10v10H7z";

function Ctl({ label, onClick, big, children, active }) {
  return (
    <button
      type="button"
      aria-label={label}
      title={label}
      onClick={(e) => {
        e.stopPropagation();
        onClick();
      }}
      className={`flex items-center justify-center rounded-full transition-[background-color,transform] hover:bg-white/20 active:scale-90 ${big ? "h-8 w-8 bg-white/15" : "h-7 w-7"} ${active ? "bg-white/25" : ""}`}
    >
      {typeof children === "string" && children.startsWith("M") ? (
        <svg viewBox="0 0 24 24" className="h-4 w-4 fill-white" aria-hidden>
          <path d={children} />
        </svg>
      ) : (
        <span className="text-sm leading-none">{children}</span>
      )}
    </button>
  );
}

/**
 * Modo Zen: durante una sesión el notch cerrado queda en un punto pulsante + cuenta atrás (24:59 🍅).
 * Al pasar el cursor se despliegan los controles de pausar/reanudar y detener; un clic abre el panel.
 */
export default function ZenPill({ pomo, collapsedH, hover, setHover }) {
  const work = pomo.phase === "work";
  const dot = work ? "bg-rose-400" : "bg-emerald-400";
  const amb = useAmbient();
  const sound = SOUNDS.find((s) => s.id === amb.kind);
  return (
    <div
      className="relative h-full animate-[fadeIn_260ms_ease-out_both] cursor-default"
      onMouseEnter={() => setHover(true)}
      onMouseLeave={() => setHover(false)}
      onClick={() => invoke("request_open", { open: true, sticky: false }).catch(() => {})}
      role="group"
      aria-label={work ? "Sesión de enfoque" : "Descanso"}
    >
      <div className="absolute inset-x-0 top-0 flex items-center justify-center gap-2" style={{ height: collapsedH }}>
        <span className="relative flex h-2 w-2" aria-hidden>
          {pomo.running && <span className={`absolute inline-flex h-full w-full animate-ping rounded-full opacity-60 ${dot}`} style={{ animationDuration: "2.2s" }} />}
          <span className={`relative inline-flex h-2 w-2 rounded-full transition-opacity duration-300 ${dot} ${pomo.running ? "" : "opacity-40"}`} />
        </span>
        <span className={`text-sm font-medium tabular-nums transition-colors duration-300 ${pomo.running ? "text-white" : "text-white/55"}`}>
          {fmtTime(pomo.left)} <span className="text-xs">{work ? "🍅" : "☕"}</span>
        </span>
      </div>

      <div
        className={`absolute inset-x-0 flex flex-col items-center gap-1 px-4 transition-opacity duration-200 ${hover ? "opacity-100 delay-100" : "pointer-events-none opacity-0"}`}
        style={{ top: collapsedH - 2 }}
      >
        <div className="w-full truncate text-center text-[11px] leading-tight text-white/70" title={pomo.taskTitle || undefined}>
          {pomo.taskTitle ? <>🎯 <span className="font-medium text-white">{pomo.taskTitle}</span></> : "Enfoque libre"}
        </div>
        <div className="flex items-center gap-2">
          <Ctl label={pomo.running ? "Pausar" : "Reanudar"} onClick={pomo.running ? pomo.pause : pomo.start} big>
            {pomo.running ? PAUSE : PLAY}
          </Ctl>
          <Ctl label="Detener sesión" onClick={pomo.reset}>
            {STOP}
          </Ctl>
          <Ctl label={sound ? `Sonido: ${sound.label} (clic para cambiar)` : "Sonido ambiental"} onClick={cycleAmbient} active={!!sound}>
            {sound ? sound.icon : "🎧"}
          </Ctl>
        </div>
      </div>
    </div>
  );
}
