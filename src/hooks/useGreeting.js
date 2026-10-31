import { useCallback, useEffect, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { isEn } from "../lib/i18n.js";
import { LABEL } from "./useNotchOpen.js";
import { useTauriEvent } from "./useTauriEvent.js";

export const GREETING_MS = 4000;

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

/**
 * Saludo de bienvenida al arrancar. Devuelve el texto mientras debe verse (4 s) o null.
 * Se cierra con clic (`dismiss`), con Esc, o si el usuario abre el notch. Solo se dispara una vez por sesión.
 */
export function useGreeting({ enabled, name, busy }) {
  const [text, setText] = useState(null);
  const nameRef = useRef(name);
  nameRef.current = name;
  const timer = useRef(null);

  const dismiss = useCallback(() => {
    clearTimeout(timer.current);
    setText(null);
  }, []);

  useEffect(() => {
    if (!enabled || greeted) return;
    // Pequeña espera para que la ventana termine de cargar y la animación se vea.
    const start = setTimeout(() => {
      if (greeted) return;
      greeted = true;
      setText(greetingFor(nameRef.current));
      timer.current = setTimeout(() => setText(null), GREETING_MS);
    }, 900);
    return () => clearTimeout(start);
  }, [enabled]);

  useEffect(() => () => clearTimeout(timer.current), []);

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

  return { text, dismiss };
}
