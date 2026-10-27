import { locale } from "../lib/i18n.js";
import { useEffect, useState } from "react";

export default function Clock({ withDate = false, hour12 = false, showTime = true }) {
  const [now, setNow] = useState(new Date());

  useEffect(() => {
    const id = setInterval(() => setNow(new Date()), 1000);
    return () => clearInterval(id);
  }, []);

  const time = now.toLocaleTimeString(locale(), { hour: hour12 ? "numeric" : "2-digit", minute: "2-digit", hour12 });
  const date = now.toLocaleDateString(locale(), { weekday: "short", day: "numeric", month: "short" });

  return (
    <div className="flex items-baseline gap-2">
      {showTime && <span className="text-sm font-semibold tabular-nums">{time}</span>}
      {withDate && <span className={`text-xs capitalize ${showTime ? "text-white/50" : "font-medium text-white/80"}`}>{date}</span>}
    </div>
  );
}
