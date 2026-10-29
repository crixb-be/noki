/**
 * Temas de temporada. Cada uno viste al personaje y, si quieres, pinta el notch y la franja de los costados.
 * `notch` y `bar` son los colores; `hint` dice cuándo se activa en modo «Automático».
 */
export const THEMES = {
  halloween: { label: "Halloween 🎃", notch: "#1A0B2B", bar: "#F97316", hint: "24 oct–2 nov" },
  christmas: { label: "Navidad 🎅", notch: "#0E2A1C", bar: "#C62B2E", hint: "1–26 dic" },
  birthday: { label: "Cumpleaños 🎂", notch: "#2B1055", bar: "#F472B6", hint: "tu día" },
  love: { label: "Amor 💖", notch: "#3A0B20", bar: "#F43F5E", hint: "7–14 feb" },
};

export const THEME_CHOICES = [
  ["auto", "Automático"],
  ["none", "Sin tema"],
  ...Object.entries(THEMES).map(([id, t]) => [id, t.label]),
];

const md = (m, d) => `${String(m).padStart(2, "0")}-${String(d).padStart(2, "0")}`;

/** `birthday` es "MM-DD" (o vacío). Devuelve el id del tema que toca ahora: "none" o una clave de THEMES. */
export function resolveTheme(setting, birthday, now = new Date()) {
  if (setting && setting !== "auto") return setting;
  const month = now.getMonth() + 1;
  const day = now.getDate();
  if (birthday && birthday === md(month, day)) return "birthday";
  if ((month === 10 && day >= 24) || (month === 11 && day <= 2)) return "halloween";
  if (month === 12 && day <= 26) return "christmas";
  if (month === 2 && day >= 7 && day <= 14) return "love";
  return "none";
}
