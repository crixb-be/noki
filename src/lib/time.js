import { locale } from "./i18n.js";
/** Clave de día en hora LOCAL ("2026-10-24"). `toISOString()` daría el día en UTC y cambiaría de día a media tarde. */
export const dayKey = (d = new Date()) =>
  `${d.getFullYear()}-${String(d.getMonth() + 1).padStart(2, "0")}-${String(d.getDate()).padStart(2, "0")}`;

export const pad2 = (n) => String(n).padStart(2, "0");

export const fmtClock = (ms) => {
  const d = new Date(ms);
  return `${pad2(d.getHours())}:${pad2(d.getMinutes())}`;
};

/** "15:30", "Mañana 09:00" o "lun 12 · 09:00" según lo lejos que esté. */
export function fmtDue(ms, now = Date.now()) {
  const d = new Date(ms);
  const today = new Date(now);
  const sameDay = (a, b) => dayKey(a) === dayKey(b);
  if (sameDay(d, today)) return fmtClock(ms);
  const tomorrow = new Date(now + 86400000);
  if (sameDay(d, tomorrow)) return `Mañana ${fmtClock(ms)}`;
  return `${d.toLocaleDateString(locale(), { weekday: "short", day: "numeric" })} · ${fmtClock(ms)}`;
}

/** Próxima ocurrencia de "HH:MM": hoy si aún no pasó (o mañana si `forceTomorrow` o ya pasó). */
export function nextOccurrence(hhmm, forceTomorrow = false, now = Date.now()) {
  const [h, m] = hhmm.split(":").map(Number);
  const d = new Date(now);
  d.setHours(h, m, 0, 0);
  if (forceTomorrow || d.getTime() <= now) d.setDate(d.getDate() + 1);
  return d.getTime();
}
