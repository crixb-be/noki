import { useEffect } from "react";
import { EN, PATTERNS } from "./en.js";

/**
 * Idiomas de la interfaz. El texto original está en español; para el inglés se recorre el DOM y se
 * sustituye cada texto (y los atributos title / placeholder / aria-label) por su traducción de en.js.
 * Lo que no tenga traducción se queda en español. Funciona igual en el notch y en Ajustes, y también para
 * lo que React pinte más tarde (un observador vigila los cambios).
 */
let lang = "es";
let started = false;
let domLang = "es"; // idioma con el que está pintado el DOM ahora mismo

export const isEn = () => lang === "en";
/** Configuración regional para fechas y horas. */
export const locale = () => (lang === "en" ? "en-US" : "es-ES");
/** Iniciales de lunes a domingo para las cabeceras del calendario. */
export const weekdayLetters = () => (lang === "en" ? ["M", "T", "W", "T", "F", "S", "S"] : ["L", "M", "X", "J", "V", "S", "D"]);

const ATTRS = ["title", "placeholder", "aria-label", "alt"];

/** Inglés de un texto (sin espacios sobrantes en los extremos) o null si no hay traducción. */
function translate(core) {
  if (Object.prototype.hasOwnProperty.call(EN, core)) return EN[core];
  for (const [re, fn] of PATTERNS) {
    const m = core.match(re);
    if (m) return fn(m);
  }
  return null;
}

/** Mantiene el texto original (en español) de cada nodo/atributo y la última traducción que le pusimos. */
const textState = new WeakMap(); // Text → { orig, en }
const attrState = new WeakMap(); // Element → { [attr]: { orig, en } }

const skip = (el) => !el || el.closest?.("[data-no-i18n], script, style, textarea, code");

function applyText(node) {
  if (skip(node.parentElement)) return;
  const raw = node.nodeValue;
  const st = textState.get(node);
  // Si el valor actual es el que pusimos nosotros, el original es el guardado; si no, React escribió algo nuevo.
  const orig = st && st.en === raw ? st.orig : raw;
  const target = lang === "en" ? withSpaces(orig) : orig;
  if (target === raw) {
    if (!st || st.orig !== orig) textState.set(node, { orig, en: raw });
    return;
  }
  textState.set(node, { orig, en: target });
  node.nodeValue = target;
}

function withSpaces(s) {
  const m = s.match(/^(\s*)([\s\S]*?)(\s*)$/);
  if (!m || !m[2]) return s;
  const en = translate(m[2].replace(/\s+/g, " "));
  return en == null ? s : `${m[1]}${en}${m[3]}`;
}

function applyAttrs(el) {
  if (skip(el)) return;
  for (const a of ATTRS) {
    if (!el.hasAttribute?.(a)) continue;
    const raw = el.getAttribute(a);
    const all = attrState.get(el) ?? {};
    const st = all[a];
    const orig = st && st.en === raw ? st.orig : raw;
    const target = lang === "en" ? withSpaces(orig) : orig;
    all[a] = { orig, en: target };
    attrState.set(el, all);
    if (target !== raw) el.setAttribute(a, target);
  }
}

function walk(root) {
  if (root.nodeType === Node.TEXT_NODE) return applyText(root);
  if (root.nodeType !== Node.ELEMENT_NODE) return;
  applyAttrs(root);
  const tw = document.createTreeWalker(root, NodeFilter.SHOW_ELEMENT | NodeFilter.SHOW_TEXT);
  let n = tw.nextNode();
  while (n) {
    if (n.nodeType === Node.TEXT_NODE) applyText(n);
    else applyAttrs(n);
    n = tw.nextNode();
  }
}

function start() {
  if (started) return;
  started = true;
  const obs = new MutationObserver((records) => {
    for (const r of records) {
      if (r.type === "characterData") applyText(r.target);
      else if (r.type === "attributes") applyAttrs(r.target);
      else r.addedNodes.forEach(walk);
    }
  });
  obs.observe(document.body, { subtree: true, childList: true, characterData: true, attributes: true, attributeFilter: ATTRS });
}

/** Fija el idioma antes de pintar (para que las fechas salgan ya en el idioma correcto); el DOM lo traduce `useLanguage`. */
export function presetLanguage(next) {
  lang = next === "en" ? "en" : "es";
}

/** Cambia el idioma de esta ventana (y deja el observador activo). */
export function setLanguage(next) {
  const value = next === "en" ? "en" : "es";
  const changed = value !== domLang;
  lang = value;
  domLang = value;
  document.documentElement.lang = value;
  start();
  if (changed) walk(document.body);
}

/** Aplica el idioma de los ajustes a la ventana actual. */
export function useLanguage(value) {
  useEffect(() => {
    setLanguage(value);
  }, [value]);
}
