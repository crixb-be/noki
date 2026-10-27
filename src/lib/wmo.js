/** Códigos meteorológicos WMO (los que usa Open-Meteo) → icono y texto en español. */
const TABLE = [
  [[0], "☀️", "🌙", "Despejado"],
  [[1], "🌤️", "🌙", "Mayormente despejado"],
  [[2], "⛅", "☁️", "Parcialmente nublado"],
  [[3], "☁️", "☁️", "Nublado"],
  [[45, 48], "🌫️", "🌫️", "Niebla"],
  [[51, 53, 55], "🌦️", "🌧️", "Llovizna"],
  [[56, 57], "🌧️", "🌧️", "Llovizna helada"],
  [[61], "🌧️", "🌧️", "Lluvia débil"],
  [[63], "🌧️", "🌧️", "Lluvia"],
  [[65], "🌧️", "🌧️", "Lluvia fuerte"],
  [[66, 67], "🌧️", "🌧️", "Lluvia helada"],
  [[71, 73, 75, 77], "🌨️", "🌨️", "Nieve"],
  [[80, 81, 82], "🌦️", "🌧️", "Chubascos"],
  [[85, 86], "🌨️", "🌨️", "Chubascos de nieve"],
  [[95], "⛈️", "⛈️", "Tormenta"],
  [[96, 99], "⛈️", "⛈️", "Tormenta con granizo"],
];

export function weatherInfo(code, isDay = 1) {
  const row = TABLE.find((r) => r[0].includes(code));
  if (!row) return { icon: "🌡️", text: "—" };
  return { icon: isDay ? row[1] : row[2], text: row[3] };
}
