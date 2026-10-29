import Clock from "../Clock.jsx";
import Battery from "../Battery.jsx";
import Visualizer from "../Visualizer.jsx";
import { fmtTime } from "../../hooks/usePomodoro.js";

/** Indicador discreto del estado compacto: reunión inminente > temporizador > canción. */
function Indicator({ ap, cal, pomo, media, dnd, calendarOn }) {
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
      <span className="flex animate-[popIn_250ms_ease-out_both] items-center gap-1.5" aria-label="Reproduciendo">
        {media.track.art && <img src={media.track.art} alt="" className="h-5 w-5 rounded-md object-cover" draggable={false} />}
        {ap.visualizer ? (
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

/** Contenido del notch cerrado: hora/fecha a la izquierda, actividad al centro y datos rápidos a la derecha. */
export default function CompactBar({ ap, modules, dnd, cal, pomo, media, battery, pendingTasks, activeHabits, habitsDone, weatherNow, temperature }) {
  return (
    <div className="flex h-full animate-[contentIn_260ms_ease-out_both] items-center justify-between gap-2 px-4">
      <div className="flex shrink-0 items-center gap-2">
        {(ap.showClock || ap.showDate) && <Clock showTime={ap.showClock} withDate={ap.showDate} hour12={!ap.clock24} />}
      </div>
      <div className="flex min-w-0 flex-1 justify-center">
        {ap.showActivity ? <Indicator ap={ap} cal={cal} pomo={pomo} media={media} dnd={dnd} calendarOn={modules.calendar} /> : null}
      </div>
      <div className="flex shrink-0 items-center gap-2 text-xs tabular-nums">
        {ap.showTasks && (
          <span className="text-white/80" title={`${pendingTasks} tarea${pendingTasks === 1 ? "" : "s"} pendiente${pendingTasks === 1 ? "" : "s"}`}>✓ {pendingTasks}</span>
        )}
        {ap.showHabits && modules.habits && activeHabits.length > 0 && (
          <span className="text-white/80" title="Hábitos completados hoy">🎯 {habitsDone}/{activeHabits.length}</span>
        )}
        {ap.showWeather && weatherNow && (
          <span className="text-white/80" title={weatherNow.text}>{weatherNow.icon} {Math.round(temperature)}°</span>
        )}
        {ap.showBattery && <Battery info={battery} />}
      </div>
    </div>
  );
}
