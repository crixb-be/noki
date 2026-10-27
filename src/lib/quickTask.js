/**
 * Sintaxis rápida al escribir una tarea: «llamar a Ana 15m», «descanso 1h30», «reunión 14:30», «pagar mañana 9:00».
 * Devuelve el texto sin la parte de tiempo y el aviso (ms) o null. Las horas «HH:MM» usan `next` para
 * decidir el día (la próxima vez que den esas horas; «mañana» fuerza el día siguiente).
 */
export function parseQuickTask(input, now = Date.now(), next) {
  let text = input;
  const take = (re) => {
    const m = text.match(re);
    if (m) text = text.replace(m[0], " ");
    return m;
  };

  // «1h30», «2h», «1 h 15 min»
  let m = take(/(?:^|\s)(?:en\s+)?(\d{1,2})\s?h(?:\s?(\d{1,2})\s?(?:m|min)?)?(?=\s|$)/i);
  if (m) {
    const mins = Number(m[1]) * 60 + Number(m[2] ?? 0);
    return finish(text, mins > 0 ? now + mins * 60000 : null, input);
  }
  // «15m», «45 min»
  m = take(/(?:^|\s)(?:en\s+)?(\d{1,3})\s?(?:m|min|mins)(?=\s|$)/i);
  if (m) return finish(text, Number(m[1]) > 0 ? now + Number(m[1]) * 60000 : null, input);
  // «14:30», «a las 9:05», «mañana 9:00»
  const tomorrow = /(?:^|\s)ma[ñn]ana(?=\s|$)/i.test(text);
  m = take(/(?:^|\s)(?:a\s+las\s+|@)?([01]?\d|2[0-3]):([0-5]\d)(?=\s|$)/i);
  if (m) {
    if (tomorrow) text = text.replace(/(?:^|\s)ma[ñn]ana(?=\s|$)/i, " ");
    const hh = `${String(m[1]).padStart(2, "0")}:${m[2]}`;
    return finish(text, next ? next(hh, tomorrow) : null, input);
  }
  return { text: input.trim(), due: null };
}

function finish(text, due, original) {
  const clean = text.replace(/\s+/g, " ").trim();
  // Si solo era la hora («15m»), la tarea conserva el texto original.
  return { text: clean || original.trim(), due };
}
