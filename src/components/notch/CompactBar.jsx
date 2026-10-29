import Clock from "../Clock.jsx";
import Battery from "../Battery.jsx";
import Visualizer from "../Visualizer.jsx";
import Mascot from "../Mascot.jsx";
import { fmtTime } from "../../hooks/usePomodoro.js";

/** Indicador discreto del estado compacto: reunión inminente > temporizador > canción > no molestar. */
function Indicator({ cal, calendarOn, pomo, media, visualizer, dnd }) {
  const soon =
    calendarOn &&
    (cal.ongoing
      ? { text: "Ahora" }
      : cal.next && cal.next.start - cal.now <= 15 * 60000
        ? { text: `${Math.max(1, Math.round((cal.next.start - cal.now) / 60000))} min` }
        : null);
  if (soon) {
    return (
      <span className="max-w-[110px] truncate text-xs text-amber-300" title={(cal.ongoing ?? cal.next).title}>
        📅 {soon.text}
      </span>
    );
  }
  if (pomo.running) {
    return (
      <span className={`text-xs tabular-nums ${pomo.phase === "work" ? "text-rose-400" : "text-emerald-400"}`}>
        ⏱ {fmtTime(pomo.left)}
      </span>
    );
  }
  if (media.available && media.playing) {
    return (
      <span className="flex items-center gap-1.5" aria-label="Reproduciendo">
        {media.track.art && <img src={media.track.art} alt="" className="h-5 w-5 rounded-md object-cover" draggable={false} />}
        {visualizer ? (
          <Visualizer bars={6} className="h-3.5 w-9" />
        ) : (
          <span className="flex h-3 items-end gap-[2px]">
            {[0, 1, 2].map((i) => (
              <span
                key={i}
                className="w-[3px] animate-bounce rounded-full bg-fuchsia-400"
                style={{ height: "100%", animationDelay: `${i * 150}ms`, animationDuration: "700ms" }}
              />
            ))}
          </span>
        )}
      </span>
    );
  }
  if (dnd) return <span className="text-xs text-white/50" title="No molestar activado">🔕</span>;
  return null;
}

/** Contenido del notch cerrado: reloj, mascota/indicador y los datos que elegiste mostrar. */
export default function CompactBar({ ap, modules, mascot, indicator, pending, habits, weatherNow, temperature, battery }) {
  return (
    <div className="relative flex h-full animate-[compactIn_260ms_cubic-bezier(0.2,0.8,0.2,1)_both] items-center justify-between gap-2 px-4">
      <div className="flex shrink-0 items-center gap-2">
        {(ap.showClock || ap.showDate) && <Clock showTime={ap.showClock} withDate={ap.showDate} hour12={!ap.clock24} />}
      </div>
      <div className="flex min-w-0 flex-1 items-center justify-center gap-2.5">
        {ap.mascot && <Mascot size={20} mood={mascot.mood} look={mascot.look} skin={mascot.skin} />}
        {ap.showActivity ? <Indicator {...indicator} visualizer={ap.visualizer} /> : null}
      </div>
      <div className="flex shrink-0 items-center gap-2 text-xs tabular-nums">
        {ap.showTasks && (
          <span className="text-white/80" title={`${pending} tarea${pending === 1 ? "" : "s"} pendiente${pending === 1 ? "" : "s"}`}>✓ {pending}</span>
        )}
        {ap.showHabits && modules.habits && habits.total > 0 && (
          <span className="text-white/80" title="Hábitos completados hoy">🎯 {habits.done}/{habits.total}</span>
        )}
        {ap.showWeather && weatherNow && (
          <span className="text-white/80" title={weatherNow.text}>{weatherNow.icon} {temperature}°</span>
        )}
        {ap.showBattery && <Battery info={battery} />}
      </div>
    </div>
  );
}
