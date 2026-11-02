import { invoke } from "@tauri-apps/api/core";
import Visualizer from "../Visualizer.jsx";

/** Alto extra (px) que se despliega bajo el notch al pasar el cursor: título + controles. */
export const PILL_EXTRA_H = 58;

/** Ecualizador sutil de CSS (cuando el visualizador de audio real está apagado). */
function Eq({ active }) {
  return (
    <span className="flex h-3.5 items-end gap-[2px]" aria-hidden>
      {[0, 1, 2, 3].map((i) => (
        <span
          key={i}
          className="w-[3px] origin-bottom rounded-full bg-fuchsia-400/90"
          style={{
            height: "100%",
            transform: active ? undefined : "scaleY(0.25)",
            animation: active ? `eqBar ${700 + i * 130}ms ease-in-out ${i * 90}ms infinite alternate` : "none",
            transition: "transform 300ms ease",
          }}
        />
      ))}
    </span>
  );
}

function Ctl({ label, onClick, disabled, big, children }) {
  return (
    <button
      type="button"
      aria-label={label}
      title={label}
      disabled={disabled}
      onClick={(e) => {
        e.stopPropagation();
        onClick();
      }}
      className={`flex items-center justify-center rounded-full transition-[background-color,transform,opacity] hover:bg-white/15 active:scale-90 disabled:opacity-30 ${
        big ? "h-8 w-8 bg-white/15 hover:bg-white/25" : "h-7 w-7"
      }`}
    >
      {children}
    </button>
  );
}

const Icon = ({ d }) => (
  <svg viewBox="0 0 24 24" className="h-4 w-4 fill-white" aria-hidden>
    <path d={d} />
  </svg>
);
const PREV = "M6 6h2v12H6zM9.5 12 18 6v12z";
const NEXT = "M16 6h2v12h-2zM6 18V6l8.5 6z";
const PLAY = "M8 5v14l11-7z";
const PAUSE = "M6 5h4v14H6zM14 5h4v14h-4z";

/** Píldora multimedia: carátula circular + ecualizador; al pasar el cursor despliega título y controles. */
export default function MediaPill({ media, collapsedH, visualizer, hover, setHover }) {
  const { track, playing, can } = media;
  return (
    <div
      className="relative h-full animate-[fadeIn_260ms_ease-out_both] cursor-default"
      onMouseEnter={() => setHover(true)}
      onMouseLeave={() => setHover(false)}
      onClick={() => invoke("request_open", { open: true, sticky: false }).catch(() => {})}
      role="group"
      aria-label="Reproduciendo"
    >
      <div className="absolute inset-x-0 top-0 flex items-center justify-between px-3.5" style={{ height: collapsedH }}>
        <span
          className={`h-[22px] w-[22px] shrink-0 overflow-hidden rounded-full bg-gradient-to-br from-fuchsia-500 to-indigo-500 ring-1 ring-white/20 ${playing ? "" : "opacity-60"}`}
          style={{ transition: "opacity 300ms ease" }}
        >
          {track.art ? (
            <img src={track.art} alt="" className="h-full w-full object-cover" draggable={false} />
          ) : (
            <span className="flex h-full w-full items-center justify-center text-[11px]">🎵</span>
          )}
        </span>
        {visualizer && playing ? <Visualizer bars={5} className="h-3.5 w-8" /> : <Eq active={playing} />}
      </div>

      <div
        className={`absolute inset-x-0 flex flex-col items-center gap-1 px-4 transition-opacity duration-200 ${hover ? "opacity-100 delay-100" : "pointer-events-none opacity-0"}`}
        style={{ top: collapsedH - 2 }}
      >
        <div className="w-full truncate text-center text-xs leading-tight">
          <span className="font-medium text-white">{track.title || "Sin título"}</span>
          {track.artist && <span className="text-white/55"> · {track.artist}</span>}
        </div>
        <div className="flex items-center gap-2">
          <Ctl label="Anterior" onClick={media.prev} disabled={!can.prev}>
            <Icon d={PREV} />
          </Ctl>
          <Ctl label={playing ? "Pausar" : "Reproducir"} onClick={media.toggle} disabled={!can.playPause} big>
            <Icon d={playing ? PAUSE : PLAY} />
          </Ctl>
          <Ctl label="Siguiente" onClick={media.next} disabled={!can.next}>
            <Icon d={NEXT} />
          </Ctl>
        </div>
      </div>
    </div>
  );
}
