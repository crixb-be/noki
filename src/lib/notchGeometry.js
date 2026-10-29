// Deben coincidir con EAR y SHADOW en src-tauri/src/notch.rs
export const EAR = 16;

/** Curva y duración compartidas por el tamaño, el radio y las orejas del notch. */
export const NOTCH_EASE = "cubic-bezier(0.32, 0.72, 0, 1)";
export const NOTCH_MS = 300;

export const notchRadius = (expanded, peek) => (expanded ? "0 0 28px 28px" : peek ? "0 0 20px 20px" : "0 0 16px 16px");

/** El notch cerrado se ensancha lo justo para que quepa lo que elegiste mostrar (hora, fecha, tareas, temperatura, batería). */
export function compactWidth(ap) {
  const items = [ap.showClock, ap.showDate, ap.showTasks, ap.showHabits, ap.showWeather, ap.showBattery].filter(Boolean).length;
  const needed =
    32 + (ap.showClock ? 46 : 0) + (ap.showDate ? 78 : 0) + (ap.showTasks ? 34 : 0) + (ap.showHabits ? 44 : 0) + (ap.showWeather ? 54 : 0) + (ap.showBattery ? 66 : 0) + (ap.showActivity ? 36 : 0) + 8 * items;
  return Math.min(Math.max(ap.collapsedW, needed), ap.expandedW);
}
