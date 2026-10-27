import { useEffect, useState } from "react";

export function useBattery() {
  const [info, setInfo] = useState(null);

  useEffect(() => {
    if (!navigator.getBattery) return;
    let battery;
    let cancelled = false;
    const update = () => setInfo({ level: Math.round(battery.level * 100), charging: battery.charging });
    navigator.getBattery().then((b) => {
      if (cancelled) return;
      battery = b;
      update();
      b.addEventListener("levelchange", update);
      b.addEventListener("chargingchange", update);
    });
    return () => {
      cancelled = true;
      if (!battery) return;
      battery.removeEventListener("levelchange", update);
      battery.removeEventListener("chargingchange", update);
    };
  }, []);

  return info;
}

export default function Battery({ info }) {
  if (!info) return <span className="text-xs text-white/40">🔋 N/D</span>;

  const color = info.level <= 20 && !info.charging ? "bg-red-500" : "bg-green-400";
  return (
    <div className="flex items-center gap-1.5">
      <div className="relative h-3 w-6 rounded-[3px] border border-white/60 p-[1.5px]">
        <div className={`h-full rounded-[1px] ${color}`} style={{ width: `${info.level}%` }} />
        <div className="absolute -right-[3px] top-1/2 h-1.5 w-[2px] -translate-y-1/2 rounded-r bg-white/60" />
      </div>
      <span className="text-xs tabular-nums text-white/80">
        {info.level}%{info.charging ? " ⚡" : ""}
      </span>
    </div>
  );
}
