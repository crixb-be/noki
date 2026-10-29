/** Disfraces de Noki. "auto" elige según la fecha; el resto se fuerza desde Ajustes → Apariencia. */
export const SKINS = [
  ["auto", "Automático"],
  ["none", "Sin disfraz"],
  ["halloween", "Halloween 🎃"],
  ["christmas", "Navidad 🎅"],
  ["birthday", "Cumpleaños 🎂"],
];

/** `birthday` es "MM-DD" (o vacío). Devuelve el disfraz que toca ahora: "none", "halloween", "christmas" o "birthday". */
export function resolveSkin(setting, birthday, now = new Date()) {
  if (setting && setting !== "auto") return setting;
  const month = now.getMonth() + 1;
  const day = now.getDate();
  if (birthday && birthday === `${String(month).padStart(2, "0")}-${String(day).padStart(2, "0")}`) return "birthday";
  if ((month === 10 && day >= 24) || (month === 11 && day <= 2)) return "halloween";
  if (month === 12 && day <= 26) return "christmas";
  return "none";
}
