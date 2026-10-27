import { useEffect, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";

/** Tamaño (px lógicos) que ocupa el notch mientras muestra un aviso. */
export function peekSize(peek, collapsedW, collapsedH) {
  if (peek.kind === "text") {
    return { w: Math.round(Math.min(440, Math.max(collapsedW, peek.text.length * 7.4 + 70))), h: collapsedH };
  }
  return { w: Math.max(collapsedW, 300), h: collapsedH + 4 };
}

const volumeIcon = (v, muted) => (muted || v === 0 ? "🔇" : v < 35 ? "🔈" : v < 70 ? "🔉" : "🔊");

function SliderPeek({ peek, setHold }) {
  const isVolume = peek.kind === "volume";
  const [value, setValue] = useState(peek.value);
  const dragging = useRef(false);

  useEffect(() => {
    if (!dragging.current) setValue(peek.value);
  }, [peek.value]);

  const change = (v) => {
    setValue(v);
    invoke(isVolume ? "set_volume" : "set_brightness", { value: v }).catch(() => {});
  };
  const end = () => {
    dragging.current = false;
    setHold(false);
  };

  return (
    <div className="flex h-full items-center gap-3 px-4">
      <button
        className="w-5 shrink-0 text-center text-sm"
        onClick={() => isVolume && invoke("set_mute", { muted: !peek.muted }).catch(() => {})}
        aria-label={isVolume ? "Silenciar" : "Brillo"}
      >
        {isVolume ? volumeIcon(value, peek.muted) : "🔆"}
      </button>
      <input
        type="range"
        min={0}
        max={100}
        value={peek.muted ? 0 : value}
        onChange={(e) => change(Number(e.target.value))}
        onPointerDown={() => {
          dragging.current = true;
          setHold(true);
        }}
        onPointerUp={end}
        onPointerCancel={end}
        className="h-1.5 min-w-0 flex-1 cursor-pointer appearance-none rounded-full"
        style={{
          accentColor: "#fff",
          background: `linear-gradient(to right, #fff ${peek.muted ? 0 : value}%, rgba(255,255,255,0.22) ${peek.muted ? 0 : value}%)`,
        }}
      />
      <span className="w-9 shrink-0 text-right text-xs tabular-nums text-white/70">{peek.muted ? "—" : `${value}%`}</span>
    </div>
  );
}

export default function PeekContent({ peek, setHold }) {
  if (peek.kind === "text") {
    return (
      <div
        className={`flex h-full items-center justify-center gap-2.5 px-4 text-xs font-medium ${
          peek.tone === "warn" ? "text-amber-300" : "text-white"
        }`}
        onMouseEnter={() => setHold(true)}
        onMouseLeave={() => setHold(false)}
      >
        <span className="text-sm">{peek.icon}</span>
        <span className="truncate">{peek.text}</span>
      </div>
    );
  }
  return (
    <div onMouseEnter={() => setHold(true)} onMouseLeave={() => setHold(false)} className="h-full">
      <SliderPeek peek={peek} setHold={setHold} />
    </div>
  );
}
