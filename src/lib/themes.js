/**
 * Temas de temporada. Cada uno viste al personaje y, si quieres, pinta el notch y la franja de los costados.
 * `notch` y `bar` son los colores; `hint` dice cuándo se activa en modo «Automático».
 */
export const THEMES = {
  halloween: {
    label: "Halloween 🎃", notch: "#1A0B2B", bar: "#F97316",
    // Decoración del notch (x, y en %, s = tamaño relativo, c = también en el notch cerrado) y de la franja lateral.
    decor: [
      { e: "🎃", x: 4, y: 78, s: 1.5, c: 1 }, { e: "🦇", x: 90, y: 12, s: 1.2, c: 1, fly: 1 }, { e: "🔥", x: 94, y: 80, s: 1.3, c: 1 },
      { e: "👻", x: 78, y: 70, s: 1.1, fly: 1 }, { e: "🕸️", x: 1, y: 2, s: 1.6 }, { e: "🦇", x: 14, y: 20, s: 0.9, fly: 1 },
      { e: "🔥", x: 4, y: 40, s: 0.9 }, { e: "🎃", x: 88, y: 46, s: 1 },
    ],
    strip: ["🎃", "🦇", "🔥", "👻"],
  },
  christmas: {
    label: "Navidad 🎅", notch: "#0E2A1C", bar: "#C62B2E",
    decor: [
      { e: "🎄", x: 3, y: 72, s: 1.6, c: 1 }, { e: "🦌", x: 90, y: 66, s: 1.4, c: 1 }, { e: "❄️", x: 92, y: 8, s: 1, c: 1, fly: 1 },
      { e: "🧝", x: 80, y: 74, s: 1.2 }, { e: "🎅", x: 12, y: 60, s: 1.2 }, { e: "🎁", x: 22, y: 82, s: 1 },
      { e: "❄️", x: 8, y: 10, s: 0.9, fly: 1 }, { e: "⭐", x: 70, y: 6, s: 0.9 }, { e: "❄️", x: 52, y: 84, s: 0.8, fly: 1 },
    ],
    strip: ["🎄", "🦌", "🎅", "🧝", "❄️", "🎁"],
  },
  birthday: {
    label: "Cumpleaños 🎂", notch: "#2B1055", bar: "#F472B6",
    decor: [
      { e: "🎈", x: 4, y: 62, s: 1.5, c: 1, fly: 1 }, { e: "🎉", x: 92, y: 70, s: 1.4, c: 1 }, { e: "🎂", x: 84, y: 8, s: 1.1, c: 1 },
      { e: "🎁", x: 16, y: 78, s: 1.1 }, { e: "🎈", x: 74, y: 56, s: 1.2, fly: 1 }, { e: "✨", x: 10, y: 14, s: 0.9 },
      { e: "🎊", x: 60, y: 84, s: 1 }, { e: "✨", x: 94, y: 40, s: 0.8 },
    ],
    strip: ["🎈", "🎉", "🎂", "🎁", "🎊"],
  },
  love: {
    label: "Amor 💖", notch: "#3A0B20", bar: "#F43F5E",
    decor: [
      { e: "💖", x: 4, y: 70, s: 1.5, c: 1, fly: 1 }, { e: "💕", x: 91, y: 12, s: 1.2, c: 1, fly: 1 }, { e: "❤️", x: 93, y: 74, s: 1.3, c: 1 },
      { e: "💗", x: 20, y: 80, s: 1, fly: 1 }, { e: "💘", x: 80, y: 62, s: 1.2 }, { e: "💞", x: 8, y: 20, s: 0.9, fly: 1 },
      { e: "🌹", x: 68, y: 82, s: 1.1 }, { e: "💕", x: 50, y: 6, s: 0.8, fly: 1 },
    ],
    strip: ["💖", "💕", "❤️", "💗", "🌹"],
  },
};

/** `none` significa «lo que tengas en Ajustes» (sin disfraz en el personaje, tus colores en el notch y los costados). */
export const THEME_CHOICES = [
  ["auto", "Automático"],
  ["none", "Ninguno"],
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
