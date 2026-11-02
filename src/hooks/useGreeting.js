import { useCallback, useEffect, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { isEn } from "../lib/i18n.js";
import { LABEL } from "./useNotchOpen.js";
import { useTauriEvent } from "./useTauriEvent.js";

export const GREETING_MS = 4000;
/** A los 2 s el saludo da paso al micro-resumen de tareas (si hay pendientes). */
export const SUMMARY_AT_MS = 2000;

// Una sola vez por sesión de Noki: sobrevive a remontajes de React, no a reiniciar la app.
let greeted = false;

/** Saludo según la hora local: 06–11 mañana, 12–18 tarde, 19–05 noche. */
export function greetingFor(name, date = new Date()) {
  const h = date.getHours();
  const en = isEn();
  const [text, icon] = h >= 6 && h < 12 ? [en ? "Good morning" : "Buenos días", "☀️"] : h >= 12 && h < 19 ? [en ? "Good afternoon" : "Buenas tardes", "🌤️"] : [en ? "Good evening" : "Buenas noches", "🌙"];
  const who = (name ?? "").replace(/\s+/g, " ").trim();
  return `${text}${who ? `, ${who}` : ""} ${icon}`;
}

const clip = (t, n = 26) => (t.length > n ? `${t.slice(0, n - 1).trimEnd()}…` : t);

/**
 * Eventos de HOY que aún no han terminado, de todos los calendarios a la vez (Rust ya junta cualquier iCal: Google,
 * Outlook, Apple, etc.). Se quitan duplicados (el mismo evento en dos calendarios) y se ordenan por cercanía:
 * primero los con hora (en curso o próximos, por hora de inicio) y al final los de todo el día.
 */
export function todayEvents(events, now = Date.now()) {
  const d = new Date(now);
  const from = new Date(d.getFullYear(), d.getMonth(), d.getDate()).getTime();
  const to = from + 86400000;
  const seen = new Set();
  return (events ?? [])
    .filter((e) => e && Number.isFinite(e.start) && e.start >= from && e.start < to && (e.end > now || (e.end === e.start && e.start > now - 60000)))
    .filter((e) => {
      const k = `${(e.title ?? "").trim().toLowerCase()}|${e.start}|${!!e.allDay}`;
      return seen.has(k) ? false : seen.add(k);
    })
    .sort((x, y) => Number(!!x.allDay) - Number(!!y.allDay) || x.start - y.start);
}

/**
 * Micro-resumen: primero el calendario de hoy (el evento más cercano, con «+N» si hay más), si no las tareas
 * pendientes, si no nada. Devuelve { text, target } (target = pestaña a la que lleva el clic) o null.
 */
export function summaryFor({ events, pending, hour12 }, now = Date.now()) {
  const en = isEn();
  const list = todayEvents(events, now);
  if (list.length) {
    const e = list[0];
    const more = list.length > 1 ? ` · +${list.length - 1}` : "";
    const title = clip(e.title?.trim() || (en ? "Event" : "Evento"));
    let text;
    if (e.allDay) text = `${title} ${en ? "today" : "hoy"}`;
    else if (e.start <= now) text = `${title} ${en ? "now" : "en curso"}`;
    else {
      const hour = new Date(e.start).toLocaleTimeString(en ? "en-US" : "es-ES", { hour: "numeric", minute: "2-digit", hour12 });
      text = en ? `${title} today at ${hour}` : `${title} hoy a las ${hour}`;
    }
    return { target: "calendar", text: `${text}${more} 📅` };
  }
  if (pending > 0) {
    return { target: "work", text: en ? `You have ${pending} pending task${pending === 1 ? "" : "s"} 📋` : `Tienes ${pending} tarea${pending === 1 ? "" : "s"} pendiente${pending === 1 ? "" : "s"} 📋` };
  }
  return null;
}

/**
 * Saludo de bienvenida al arrancar. Devuelve `{ text, summary, showSummary, dismiss }`: `text` es el saludo (o null si no se muestra), y a los 2 s
 * pasa a `showSummary` si hay eventos de hoy o tareas pendientes (`summary`; `target` = pestaña del clic). Dura 4 s en total.
 * Se cierra con clic (`dismiss`), con Esc, o si el usuario abre el notch. Solo se dispara una vez por sesión.
 */
export function useGreeting({ enabled, name, pending, events, hour12, busy }) {
  const [text, setText] = useState(null);
  const [summary, setSummary] = useState(null);
  const [showSummary, setShowSummary] = useState(false);
  const dataRef = useRef({});
  dataRef.current = { pending, events, hour12 };
  const summaryTimer = useRef(null);
  const nameRef = useRef(name);
  nameRef.current = name;
  const timer = useRef(null);

  const dismiss = useCallback(() => {
    clearTimeout(timer.current);
    clearTimeout(summaryTimer.current);
    setText(null);
    setShowSummary(false);
  }, []);

  useEffect(() => {
    if (!enabled || greeted) return;
    // Pequeña espera para que la ventana termine de cargar y la animación se vea.
    const start = setTimeout(() => {
      if (greeted) return;
      greeted = true;
      const sum = summaryFor(dataRef.current);
      setText(greetingFor(nameRef.current));
      setSummary(sum);
      setShowSummary(false);
      if (sum) summaryTimer.current = setTimeout(() => setShowSummary(true), SUMMARY_AT_MS);
      timer.current = setTimeout(() => {
        setText(null);
        setShowSummary(false);
      }, GREETING_MS);
    }, 900);
    return () => clearTimeout(start);
  }, [enabled]);

  useEffect(() => () => [timer, summaryTimer].forEach((t) => clearTimeout(t.current)), []);

  // Si el usuario abre el notch, el saludo cede el paso.
  useEffect(() => {
    if (busy) dismiss();
  }, [busy, dismiss]);

  // Rust vigila Esc y los clics fuera aunque la ventana no tenga el foco; el listener local es un respaldo.
  useEffect(() => {
    invoke("set_greeting", { active: !!text }).catch(() => {});
    return () => {
      if (text) invoke("set_greeting", { active: false }).catch(() => {});
    };
  }, [text]);
  useTauriEvent("greeting-dismiss", (p) => p.label === LABEL && dismiss(), !!text);

  useEffect(() => {
    if (!text) return;
    const onKey = (e) => e.key === "Escape" && dismiss();
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [text, dismiss]);

  return { text, summary: summary?.text ?? null, target: summary?.target ?? null, showSummary: showSummary && !!summary, dismiss };
}
