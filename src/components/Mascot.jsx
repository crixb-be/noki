import { useCallback, useEffect, useRef, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";

/**
 * La carita de Noki dentro del notch. Parpadea, sigue el cursor con la mirada y reacciona:
 * contenta al terminar una tarea, un hábito o un Pomodoro, baila con la música, se sorprende si la pinchas,
 * se marea con muchos clics, abre la boca al recibir un archivo y se adormece de noche o con poca batería.
 * Está dibujada en código (sin imágenes) para seguir el color y el tamaño del notch.
 */

const LABEL = getCurrentWindow().label;

const STROKE = { fill: "none", stroke: "#FFFFFF", strokeLinecap: "round", strokeLinejoin: "round" };

/** Hacia dónde mira: x e y entre -1 y 1 (el centro del notch es 0, 0). Rust avisa de la posición del cursor. */
export function useLook(enabled) {
  const [look, setLook] = useState({ x: 0, y: 0 });
  useEffect(() => {
    if (!enabled) return;
    let unlisten;
    let cancelled = false;
    listen("mascot-cursor", (e) => {
      const p = e.payload;
      if (p.label !== LABEL) return;
      setLook(p.near ? { x: p.x, y: p.y } : { x: 0, y: 0 });
    }).then((fn) => (cancelled ? fn() : (unlisten = fn)));
    return () => {
      cancelled = true;
      unlisten?.();
    };
  }, [enabled]);
  return look;
}

const isNight = () => {
  const h = new Date().getHours();
  return h >= 23 || h < 6;
};

/**
 * Estado de ánimo según lo que pasa en Noki. Prioridad: mareada > sorprendida > recibiendo archivo >
 * contenta (aviso o saludo) > bailando > adormilada > tranquila.
 */
export function useMascotMood({ enabled, doneTasks, habitsDone, pomoAlert, playing, dragging, battery }) {
  const [happy, setHappy] = useState(true); // saluda al abrir Noki
  const [poked, setPoked] = useState(false);
  const [dizzy, setDizzy] = useState(false);
  const [night, setNight] = useState(isNight());
  const happyTimer = useRef(null);
  const pokeTimer = useRef(null);
  const dizzyTimer = useRef(null);
  const clicks = useRef([]);
  const prev = useRef({ doneTasks, habitsDone, pomoAlert });

  const cheer = useCallback((ms = 3200) => {
    setHappy(true);
    clearTimeout(happyTimer.current);
    happyTimer.current = setTimeout(() => setHappy(false), ms);
  }, []);

  // Saludo inicial.
  useEffect(() => {
    if (!enabled) return;
    cheer(2600);
    return () => clearTimeout(happyTimer.current);
  }, [enabled, cheer]);

  // Se alegra cuando sube lo hecho (tarea, hábito) o suena el aviso del Pomodoro. Nunca al arrancar.
  useEffect(() => {
    const p = prev.current;
    if (enabled && (doneTasks > p.doneTasks || habitsDone > p.habitsDone || (pomoAlert && !p.pomoAlert))) cheer();
    prev.current = { doneTasks, habitsDone, pomoAlert };
  }, [enabled, doneTasks, habitsDone, pomoAlert, cheer]);

  useEffect(() => {
    const id = setInterval(() => setNight(isNight()), 60000);
    return () => clearInterval(id);
  }, []);
  useEffect(() => () => [pokeTimer, dizzyTimer].forEach((t) => clearTimeout(t.current)), []);

  /** Un clic la sorprende; tres seguidos la marean. */
  const poke = useCallback(() => {
    const now = Date.now();
    clicks.current = [...clicks.current.filter((t) => now - t < 1500), now];
    if (clicks.current.length >= 3) {
      clicks.current = [];
      setDizzy(true);
      clearTimeout(dizzyTimer.current);
      dizzyTimer.current = setTimeout(() => setDizzy(false), 2600);
      return;
    }
    setPoked(true);
    clearTimeout(pokeTimer.current);
    pokeTimer.current = setTimeout(() => setPoked(false), 700);
  }, []);

  const lowBattery = battery && battery.level <= 15 && !battery.charging;
  let mood = "idle";
  if (dizzy) mood = "dizzy";
  else if (poked) mood = "poke";
  else if (dragging) mood = "drop";
  else if (happy) mood = "happy";
  else if (playing) mood = "dance";
  else if (night || lowBattery) mood = "sleepy";
  return { mood, poke };
}

function Eyes({ mood, look }) {
  const ox = look.x * 3.4;
  const oy = look.y * 2.6;
  const Dot = ({ cx }) => (
    <g className="mascot-blink">
      <ellipse cx={cx + ox} cy={17 + oy} rx="5.2" ry="6.4" fill="#FFFFFF" />
    </g>
  );
  switch (mood) {
    case "happy":
    case "dance":
      return (
        <g {...STROKE} strokeWidth="3.4">
          <path d="M14 21 Q20 12 26 21" />
          <path d="M38 21 Q44 12 50 21" />
        </g>
      );
    case "sleepy":
      return (
        <g {...STROKE} strokeWidth="3.2">
          <path d="M14.5 18 Q20 22 25.5 18" />
          <path d="M38.5 18 Q44 22 49.5 18" />
        </g>
      );
    case "poke":
      return (
        <g {...STROKE} strokeWidth="3.2">
          <path d="M15 13 L25 17.5 L15 22" />
          <path d="M49 13 L39 17.5 L49 22" />
        </g>
      );
    case "dizzy":
      return (
        <g {...STROKE} strokeWidth="3">
          <path d="M15 12.5 L25 22.5 M25 12.5 L15 22.5" />
          <path d="M39 12.5 L49 22.5 M49 12.5 L39 22.5" />
        </g>
      );
    case "drop":
      return (
        <g fill="#FFFFFF">
          <ellipse cx="20" cy="16" rx="6.6" ry="7.6" />
          <ellipse cx="44" cy="16" rx="6.6" ry="7.6" />
        </g>
      );
    default:
      return (
        <>
          <Dot cx={20} />
          <Dot cx={44} />
        </>
      );
  }
}

function Mouth({ mood }) {
  switch (mood) {
    case "happy":
    case "dance":
      return <path {...STROKE} strokeWidth="3.2" d="M25 29.5 Q32 37.5 39 29.5" />;
    case "sleepy":
      return <path {...STROKE} strokeWidth="2.8" d="M28.5 31 Q32 33 35.5 31" />;
    case "poke":
      return <ellipse cx="32" cy="32" rx="3" ry="3.6" fill="#FFFFFF" />;
    case "dizzy":
      return <path {...STROKE} strokeWidth="2.8" d="M25 32 Q28.5 28.5 32 32 T39 32" />;
    case "drop":
      return <ellipse cx="32" cy="32" rx="4.6" ry="5.6" fill="#FFFFFF" />;
    default:
      return <path {...STROKE} strokeWidth="3" d="M26 30 Q32 35 38 30" />;
  }
}

/** `size` es el alto en píxeles; el ancho sale solo. */
export default function Mascot({ size = 22, mood = "idle", look = { x: 0, y: 0 }, onPoke, className = "" }) {
  return (
    <span
      role={onPoke ? "button" : "img"}
      aria-label="Noki"
      onClick={onPoke}
      className={`mascot mascot-${mood} inline-flex shrink-0 items-center justify-center ${onPoke ? "cursor-pointer" : ""} ${className}`}
      style={{ height: size, width: size * 1.6 }}
    >
      <svg viewBox="0 0 64 40" width={size * 1.6} height={size} overflow="visible">
        <g className="mascot-body">
          <Eyes mood={mood} look={look} />
          <Mouth mood={mood} />
        </g>
        {mood === "sleepy" && (
          <g fill="#FFFFFF" fontFamily="Segoe UI, sans-serif" fontWeight="700">
            <text className="mascot-z mascot-z1" x="50" y="10" fontSize="9">z</text>
            <text className="mascot-z mascot-z2" x="56" y="5" fontSize="7">z</text>
          </g>
        )}
      </svg>
    </span>
  );
}
