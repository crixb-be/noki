import { useCallback, useEffect, useRef, useState } from "react";
import { LABEL } from "../hooks/useNotchOpen.js";
import { useTauriEvent } from "../hooks/useTauriEvent.js";

/**
 * La carita de Noki dentro del notch. Parpadea, sigue el cursor con la mirada y reacciona:
 * contenta al terminar una tarea, un hábito o un Pomodoro, baila con la música, se sorprende si la pinchas,
 * se marea con muchos clics, abre la boca al recibir un archivo y se adormece de noche o con poca batería.
 * Está dibujada en código (sin imágenes) para seguir el color y el tamaño del notch.
 */

const STROKE = { fill: "none", stroke: "#FFFFFF", strokeLinecap: "round", strokeLinejoin: "round" };

/** Hacia dónde mira: x e y entre -1 y 1 (el centro del notch es 0, 0). Rust avisa de la posición del cursor. */
export function useLook(enabled) {
  const [look, setLook] = useState({ x: 0, y: 0 });
  useTauriEvent("mascot-cursor", (p) => p.label === LABEL && setLook(p.near ? { x: p.x, y: p.y } : { x: 0, y: 0 }), enabled);
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

function Eyes({ mood, look, skin }) {
  const ox = look.x * 3.4;
  const oy = look.y * 2.6;
  const Heart = ({ cx }) => {
    const x = cx + ox;
    const y = 17 + oy;
    return (
      <g className="mascot-blink">
        <path
          d={`M${x} ${y + 7} C${x - 9} ${y + 1} ${x - 6} ${y - 6} ${x - 3} ${y - 5.4} C${x - 1.4} ${y - 5} ${x} ${y - 3.6} ${x} ${y - 2.6} C${x} ${y - 3.6} ${x + 1.4} ${y - 5} ${x + 3} ${y - 5.4} C${x + 6} ${y - 6} ${x + 9} ${y + 1} ${x} ${y + 7}Z`}
          fill="#FB7185"
        />
      </g>
    );
  };
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
      if (skin === "love")
        return (
          <>
            <Heart cx={20} />
            <Heart cx={44} />
          </>
        );
      return (
        <g transform={`translate(${ox} ${oy})`}>
          <g {...STROKE} strokeWidth="3.4">
            <path d="M14.5 20 Q20 13 25.5 20" />
            <path d="M38.5 20 Q44 13 49.5 20" />
          </g>
        </g>
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
      return <path {...STROKE} strokeWidth="3" d="M28 30.5 Q32 34.5 36 30.5" />;
  }
}

/** Gorros y accesorios (arriba, y 0–17 del lienzo). La cara se reduce un poco para dejarles sitio. */
function Costume({ skin }) {
  switch (skin) {
    case "christmas":
      return (
        <g>
          <path d="M17 17 C19 7 29 2 41 3 C46 3.5 49 7 48 12 L47 17 Z" fill="#E5393B" />
          <path d="M41 3 C46 3 50 6 49 12" fill="none" stroke="#C62B2E" strokeWidth="1.2" />
          <rect x="14" y="14.5" width="36" height="5" rx="2.5" fill="#FFFFFF" />
          <circle cx="49" cy="11.5" r="3.6" fill="#FFFFFF" />
        </g>
      );
    case "halloween":
      return (
        <g>
          <path d="M23 15 L34 1.5 C34.5 1 35.5 1.5 36 2.5 L42 15 Z" fill="#6B3FA0" />
          <path d="M34 1.5 C37 1 39 3 38 4.5" fill="none" stroke="#6B3FA0" strokeWidth="2.4" strokeLinecap="round" />
          <path d="M25 12 L41 12 L42 15 L24 15 Z" fill="#FB923C" />
          <ellipse cx="33" cy="15.8" rx="18" ry="3" fill="#7C4DB8" />
          <rect x="31.6" y="11.6" width="3.2" height="3.4" rx="0.6" fill="none" stroke="#FDE68A" strokeWidth="1" />
        </g>
      );
    case "love":
      return (
        <g>
          <path d="M32 15 C22 8.5 25 0.8 29.6 2.4 C31 2.9 32 4.2 32 5.4 C32 4.2 33 2.9 34.4 2.4 C39 0.8 42 8.5 32 15Z" fill="#FB7185" />
          <g className="mascot-confetti" fill="#F9A8D4">
            <path d="M13 12 C9.5 9.5 10.6 6.6 12.3 7.2 C12.8 7.4 13 7.9 13 8.3 C13 7.9 13.2 7.4 13.7 7.2 C15.4 6.6 16.5 9.5 13 12Z" />
            <path d="M52 15 C49.5 13.2 50.3 11.1 51.5 11.5 C51.9 11.7 52 12 52 12.3 C52 12 52.2 11.7 52.6 11.5 C53.8 11.1 54.5 13.2 52 15Z" />
          </g>
        </g>
      );
    case "birthday":
      return (
        <g>
          <path d="M24 17 L32 2.5 L40 17 Z" fill="#F472B6" />
          <path d="M27.4 11 L36.6 11 M25.7 14 L38.3 14" stroke="#FDE047" strokeWidth="2" strokeLinecap="round" />
          <circle cx="32" cy="2.4" r="2.6" fill="#38BDF8" />
          <g className="mascot-confetti">
            <circle cx="12" cy="9" r="1.4" fill="#FDE047" />
            <circle cx="52" cy="7" r="1.4" fill="#34D399" />
            <rect x="48" y="14" width="2.6" height="2.6" rx="0.5" fill="#F472B6" />
            <rect x="12.5" y="16" width="2.6" height="2.6" rx="0.5" fill="#38BDF8" />
          </g>
        </g>
      );
    default:
      return null;
  }
}

/** `size` es el alto en píxeles; el ancho sale solo. `skin`: "none" | "halloween" | "christmas" | "birthday". */
export default function Mascot({ size = 22, mood = "idle", look = { x: 0, y: 0 }, skin = "none", onPoke, className = "" }) {
  const dressed = skin !== "none";
  return (
    <span
      role={onPoke ? "button" : "img"}
      aria-label="Noki"
      onClick={onPoke}
      className={`mascot mascot-${mood} ${dressed ? `mascot-skin-${skin}` : ""} inline-flex shrink-0 items-center justify-center ${onPoke ? "cursor-pointer" : ""} ${className}`}
      style={{ height: size, width: size * 1.6 }}
    >
      <svg viewBox="0 0 64 40" width={size * 1.6} height={size} overflow="visible">
        <g className="mascot-body">
          <g transform={dressed ? "translate(6.4 9.6) scale(0.8)" : undefined}>
            <Eyes mood={mood} look={look} skin={skin} />
            <Mouth mood={mood} />
          </g>
          {dressed && <Costume skin={skin} />}
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
