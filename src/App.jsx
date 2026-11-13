import { useCallback, useEffect, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { useBattery } from "./components/Battery.jsx";
import { addPaths, useNativeDrop } from "./components/FileShelf.jsx";
import { NotchDecor } from "./components/notch/ThemeDecor.jsx";
import Ear from "./components/notch/Ear.jsx";
import CompactBar from "./components/notch/CompactBar.jsx";
import MediaPill, { PILL_EXTRA_H } from "./components/notch/MediaPill.jsx";
import ZenPill, { ZEN_EXTRA_H } from "./components/notch/ZenPill.jsx";
import QuickAsk, { QUICK_H, QUICK_W } from "./components/notch/QuickAsk.jsx";
import ExpandedView from "./components/notch/ExpandedView.jsx";
import Mascot, { useLook, useMascotMood } from "./components/Mascot.jsx";
import PeekContent, { peekSize } from "./components/Peek.jsx";
import { useCalendar } from "./hooks/useCalendar.js";
import { useWeather } from "./hooks/useWeather.js";
import { isRecording, toggleRecording, useRecorder } from "./lib/recorder.js";
import { useStore, useStoreReady } from "./hooks/useStore.js";
import { useReminders } from "./hooks/useReminders.js";
import { usePomodoro } from "./hooks/usePomodoro.js";
import { useMedia } from "./hooks/useMedia.js";
import { useSettings } from "./hooks/useSettings.js";
import { usePeeks } from "./hooks/usePeeks.js";
import { useGreeting } from "./hooks/useGreeting.js";
import { useTauriEvent } from "./hooks/useTauriEvent.js";
import { LABEL, useNotchOpen } from "./hooks/useNotchOpen.js";
import { useAiChat } from "./hooks/useAiChat.js";
import { EMPTY_HABITS, countOn, useHabits } from "./hooks/useHabits.js";
import { fillFor, rgba, useBgImage } from "./lib/background.js";
import { weatherInfo } from "./lib/wmo.js";
import { THEMES, resolveTheme } from "./lib/themes.js";
import { presetLanguage, useLanguage } from "./lib/i18n.js";

// Misma curva y duración para ancho y alto: el notch se abre y cierra como una sola pieza.
const NOTCH_MS = 340;
const NOTCH_EASE = "cubic-bezier(0.32, 0.72, 0, 1)";

const TABS = [
  { id: "panel", icon: "🏠", label: "Panel" },
  { id: "media", icon: "🎵", label: "Música" },
  { id: "work", icon: "✅", label: "Agenda" },
  { id: "habits", icon: "💧", label: "Hábitos" },
  { id: "files", icon: "📁", label: "Archivos" },
  { id: "calendar", icon: "📅", label: "Calendario" },
  { id: "ai", icon: "✨", label: "Asistente IA" },
  { id: "system", icon: "🖥️", label: "Sistema" },
  { id: "screentime", icon: "⏱️", label: "Tiempo de pantalla" },
  { id: "tools", icon: "🧰", label: "Herramientas" },
];

const isTextField = (el) =>
  el?.tagName === "TEXTAREA" || (el?.tagName === "INPUT" && !["range", "checkbox"].includes(el.type));

/** Espera a tener ajustes y datos cargados para montar el notch con todo disponible. */
export default function App() {
  const { settings, update } = useSettings();
  presetLanguage(settings?.general.language);
  useLanguage(settings?.general.language);
  const ready = useStoreReady();
  if (!settings || !ready) return null;
  return <Notch settings={settings} update={update} />;
}

function Notch({ settings, update }) {
  const expanded = useNotchOpen();
  const [tab, setTab] = useState("media");
  const prod = settings.productivity;

  const battery = useBattery();
  const media = useMedia();
  const [files, setFiles] = useStore("files", []);
  const unpinTimer = useRef(null);

  const dnd = settings.general.dnd;

  // Con un notch por pantalla hay varias ventanas con esta misma app. La principal ("main") es la «líder»: solo ella
  // genera avisos (Pomodoro, tareas, agua, salud, calendario) y descarga datos; las demás reflejan el estado
  // compartido (almacén sincronizado) y muestran los avisos que la líder difunde a todas.
  const leader = LABEL === "main";

  // Alertas (fin de Pomodoro, tarea vencida, agua, salud): aviso en TODOS los notch + sonido del sistema (una vez).
  // Con «No molestar» no se muestra ni suena nada.
  const notify = useCallback(
    ({ silent, ms, ...p }) => {
      if (dnd) return;
      invoke("broadcast_notify", { payload: { ...p, ms } }).catch(() => {});
      if (prod.sound && !silent) invoke("alert_sound").catch(() => {});
    },
    [prod.sound, dnd]
  );

  const pomo = usePomodoro(prod, notify, leader);
  // Modo Zen: mientras dura un bloque de enfoque se silencian las alertas de sistema (pasan las críticas).
  const focusDnd = !!prod.zenDnd && pomo.running && pomo.phase === "work";
  const { peek: rawPeek, setHold, show } = usePeeks({ hud: settings.hud, battery, dnd, focus: focusDnd });
  useTauriEvent("notch-notify", ({ ms, ...p }) => show({ kind: "text", ...p }, ms));
  useReminders({ health: settings.health, notify, leader });
  useHabits({ enabled: settings.modules.habits, notify, leader });
  const cal = useCalendar({ calendar: settings.calendar, enabled: settings.modules.calendar, notify, leader });
  const [todos] = useStore("todos", []);
  const pendingTasks = todos.filter((t) => !t.done).length;
  const [habits] = useStore("habits", EMPTY_HABITS);
  const activeHabits = (habits.items ?? []).filter((i) => i.active !== false);
  const habitsDone = activeHabits.filter((i) => countOn(habits, i.id) >= i.goal).length;
  const doneTasks = todos.filter((t) => t.done).length;
  const aiChat = useAiChat(); // el chat sigue respondiendo aunque cambies de pestaña
  const weather = useWeather({ weather: settings.weather, enabled: settings.modules.weather, leader });

  // Aviso cuando la comprobación automática encuentra una versión nueva (nunca se instala sola).
  useTauriEvent(
    "update-available",
    (p) => notify({ icon: "⬆️", text: `Noki ${p.version} disponible · Ajustes → Acerca de`, ms: 9000, silent: true }),
    leader
  );

  const dragging = useNativeDrop({
    onEnter: () => {
      setTab("files");
      invoke("request_open", { open: true, sticky: false }).catch(() => {});
    },
    onDrop: (paths) => addPaths(paths, setFiles),
  });

  // Felicitación el día del cumpleaños (una vez al año; solo la ventana líder).
  const [greeted, setGreeted] = useStore("birthdayGreeted", 0);
  useEffect(() => {
    if (!leader || !settings.appearance.birthday) return;
    const now = new Date();
    if (settings.appearance.birthday !== `${String(now.getMonth() + 1).padStart(2, "0")}-${String(now.getDate()).padStart(2, "0")}`) return;
    if (greeted === now.getFullYear()) return;
    const age = settings.appearance.birthYear ? now.getFullYear() - settings.appearance.birthYear : 0;
    notify({ icon: "🎂", text: age > 0 ? `¡Feliz cumpleaños! Hoy cumples ${age}` : "¡Feliz cumpleaños!", ms: 9000, silent: true });
    setGreeted(now.getFullYear());
  }, [leader, settings.appearance.birthday]); // eslint-disable-line react-hooks/exhaustive-deps

  // Atajos globales que Rust reenvía a esta ventana (grabar, «No molestar», tarea rápida, ir a una pestaña).
  const [, setWorkTab] = useStore("workTab", "tasks");
  const tabsRef = useRef([]);
  const [, setToolsTab] = useStore("toolsTab", "launcher");
  // Pregunta rápida a la IA (atajo): el notch abierto muestra solo un campo de texto, no el panel entero.
  const [quick, setQuick] = useState(false);
  const [quickAsked, setQuickAsked] = useState(false);
  useEffect(() => {
    if (!expanded) {
      setQuick(false);
      setQuickAsked(false);
    }
  }, [expanded]);
  useTauriEvent("shortcut-action", async (p) => {
    if (p.label !== LABEL) return;
    if (p.action === "record") {
      const was = isRecording();
      await toggleRecording();
      show(was ? { kind: "text", icon: "✅", text: "Grabación guardada" } : { kind: "text", icon: "🔴", text: "Grabando… (el mismo atajo la detiene)" }, 2200);
    } else if (p.action === "dnd") {
      update({ general: { dnd: !dnd } });
      show({ kind: "text", icon: dnd ? "🔔" : "🔕", text: dnd ? "No molestar desactivado" : "No molestar activado" }, 1800);
    } else if (p.action === "addTask") {
      setTab("work");
      setWorkTab("tasks");
      setTimeout(() => document.querySelector("[data-task-input]")?.focus(), 380);
    } else if (p.action === "ai") {
      setQuickAsked(false);
      setQuick(true);
    } else if (p.action === "clip") {
      wantTab.current = "tools";
      setTab("tools");
      setToolsTab("clip");
    } else if (p.action.startsWith("tab:")) {
      const t = tabsRef.current[Number(p.action.slice(4))];
      if (t) setTab(t.id);
    }
  });

  // Esc cierra el notch (útil cuando se abrió con el atajo).
  useEffect(() => {
    const onKey = (e) => e.key === "Escape" && invoke("request_open", { open: false, sticky: false }).catch(() => {});
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, []);

  // Pestaña inicial según los ajustes (cada vez que se abre, salvo mientras se arrastra un archivo).
  const defaultTab = settings.modules.defaultTab;
  const wantTab = useRef(null); // el saludo pidió abrir directamente Calendario o Tareas: gana a la pestaña inicial
  useEffect(() => {
    if (expanded && wantTab.current) {
      setTab(wantTab.current);
      wantTab.current = null;
    } else if (expanded && !dragging && defaultTab && defaultTab !== "last") setTab(defaultTab);
  }, [expanded]); // eslint-disable-line react-hooks/exhaustive-deps

  const mascotOn = settings.appearance.mascot;
  const look = useLook(mascotOn);
  const { mood, poke } = useMascotMood({
    enabled: mascotOn,
    doneTasks,
    habitsDone,
    pomoAlert: pomo.alert,
    playing: media.available && media.playing,
    dragging,
    battery,
  });

  // Saludo de bienvenida al arrancar: solo la ventana líder, una vez por sesión y sin pisar el notch abierto.
  const greet = useGreeting({ enabled: leader && settings.general.welcomeGreeting, name: settings.general.userName, pending: pendingTasks, events: settings.modules.calendar ? cal.allEvents : [], hour12: !settings.appearance.clock24, busy: expanded });
  const greeting = expanded ? null : greet.showSummary ? greet.summary : greet.text;
  const greetingW = Math.max(greet.text?.length ?? 0, greet.summary?.length ?? 0); // ancho fijo: no salta al cambiar de estado

  // Clic en el resumen: abre el notch directamente en Calendario (eventos) o Agenda → Tareas. En el saludo normal, solo lo cierra.
  const openFromGreeting = () => {
    const target = greet.showSummary ? greet.target : null;
    greet.dismiss();
    if (!target) return;
    wantTab.current = target;
    setTab(target);
    if (target === "work") setWorkTab("tasks");
    invoke("request_open", { open: true, sticky: true }).catch(() => {});
  };

  const rec = useRecorder();
  const [mic, setMic] = useState({ active: false, app: "" });
  useTauriEvent("mic-state", setMic);

  // Píldora multimedia: con música sonando el notch se ensancha; al parar/cerrar la app vuelve al estado compacto tras unos segundos.
  const [pillLinger, setPillLinger] = useState(false);
  const [pillHover, setPillHover] = useState(false);
  const pillLeave = useRef(null);
  const mediaLive = settings.appearance.mediaPill && media.available && media.playing;
  useEffect(() => {
    if (mediaLive) {
      setPillLinger(true);
      return;
    }
    const id = setTimeout(() => setPillLinger(false), 5000);
    return () => clearTimeout(id);
  }, [mediaLive]);
  // Las alertas de sistema ceden ante una grabación y, durante una llamada (micrófono en uso), ante todo salvo lo urgente (tono de advertencia).
  const sysYield = !!rawPeek?.sys && (rec.recording || (mic.active && rawPeek.tone !== "warn"));
  const peek = expanded || greeting || sysYield ? null : rawPeek;
  const meetingSoon = settings.modules.calendar && (!!cal.ongoing || (cal.next && cal.next.start - cal.now <= 15 * 60000));
  const pillWanted = (mediaLive || (pillLinger && settings.appearance.mediaPill)) && !rec.recording && !pomo.running && !pomo.paused && !pomo.alert && !meetingSoon;
  const zenWanted = !!settings.productivity.zen && (pomo.running || pomo.paused) && !rec.recording;
  const pillShown = pillWanted && !expanded && !greeting && !peek && !rawPeek;
  const zenShown = zenWanted && !expanded && !greeting && !peek && !rawPeek;
  const hoverShown = pillShown || zenShown;
  const setHover = (h) => {
    clearTimeout(pillLeave.current);
    if (h) setPillHover(true);
    else pillLeave.current = setTimeout(() => setPillHover(false), 220);
  };
  useEffect(() => {
    if (!hoverShown) setPillHover(false);
    invoke("set_media_pill", { active: hoverShown }).catch(() => {});
  }, [hoverShown]);
  // Modo Zen: Rust bloquea la apertura por hover/borde mientras dure la sesión (aunque un aviso tape la píldora un instante).
  const zenLock = (zenWanted || rec.recording) && !expanded;
  useEffect(() => {
    invoke("set_zen_pill", { active: zenLock }).catch(() => {});
  }, [zenLock]);
  const banner = expanded && rawPeek?.kind === "text" ? rawPeek : null;
  const peekDims = greeting
    ? { w: Math.round(Math.min(settings.appearance.expandedW, Math.max(settings.appearance.collapsedW + 60, greetingW * 8 + 110))), h: settings.appearance.collapsedH + 40 }
    : peek
      ? peekSize(peek, settings.appearance.collapsedW, settings.appearance.collapsedH)
      : zenShown
        ? {
            w: Math.max(150, pillHover ? 250 : 0),
            h: settings.appearance.collapsedH + (pillHover ? ZEN_EXTRA_H : 0),
          }
      : pillShown
        ? {
            w: Math.min(settings.appearance.expandedW, Math.max(settings.appearance.collapsedW + 80, pillHover ? 300 : 0)),
            h: settings.appearance.collapsedH + (pillHover ? PILL_EXTRA_H : 0),
          }
        : null;
  // El notch cerrado se ensancha lo justo para que quepa lo que elegiste mostrar (hora, fecha, tareas, temperatura, batería).
  const apx = settings.appearance;
  const compactItems = [apx.showClock, apx.showDate, apx.showTasks, apx.showHabits, apx.showWeather, apx.showBattery].filter(Boolean).length;
  const compactNeeded =
    32 + (apx.showClock ? 46 : 0) + (apx.showDate ? 78 : 0) + (apx.showTasks ? 34 : 0) + (apx.showHabits ? 44 : 0) + (apx.showWeather ? 54 : 0) + (apx.showBattery ? 66 : 0) + (apx.showActivity ? 36 : 0) + (apx.mascot ? 40 : 0) + 8 * compactItems;
  const compactW = Math.min(Math.max(apx.collapsedW, compactNeeded), apx.expandedW);

  // Rust necesita saber el tamaño del aviso para que su zona reciba clics (slider de volumen/brillo).
  useEffect(() => {
    invoke("set_peek", { width: peekDims?.w ?? (compactW > apx.collapsedW ? compactW : 0), height: peekDims?.h ?? 0 }).catch(() => {});
  }, [peekDims?.w, peekDims?.h, compactW, apx.collapsedW]);
  useEffect(() => {
    if (expanded) setHold(false);
  }, [expanded, setHold]);

  const { appearance: ap, modules } = settings;
  const rank = (id) => {
    const i = modules.order?.indexOf(id) ?? -1;
    return i < 0 ? TABS.length : i;
  };
  const tabs = TABS.filter((t) => modules[t.id]).sort((a, b) => rank(a.id) - rank(b.id));
  tabsRef.current = tabs;
  const activeTab = tabs.some((t) => t.id === tab) ? tab : tabs[0]?.id;

  // Agenda y Calendario contienen selectores nativos que pueden abrirse fuera de la
  // ventana del notch. Mientras se trabaja en esas vistas se conserva abierta; al
  // cambiar a otra pestaña vuelve el cierre automático por cursor.
  useEffect(() => {
    if (!expanded) return;
    const interactive = activeTab === "work" || activeTab === "calendar";
    invoke("request_open", { open: true, sticky: interactive }).catch(() => {});
  }, [expanded, activeTab]);

  // El visualizador solo captura audio mientras suena música y se ve (notch cerrado o pestaña de música).
  const wantViz = ap.visualizer && media.available && media.playing && (!expanded || activeTab === "media");
  useEffect(() => {
    invoke("set_visualizer", { enabled: wantViz }).catch(() => {});
  }, [wantViz]);
  // Fondo del notch y de la franja a sus costados (configurables en Ajustes → Apariencia).
  const weatherNow = weather.data?.current ? weatherInfo(weather.data.current.weather_code, weather.data.current.is_day) : null;
  const [tint, setTint] = useState(null); // color bajo la franja en modo automático (lo mide Rust)
  useTauriEvent("bar-tint", (p) => p.label === LABEL && setTint(p.color), ap.barMode === "auto" || ap.notchAuto);
  // Temas de temporada, cada parte el suyo: personaje, notch central y franja de los costados. No tocan tus colores guardados.
  const themeFor = (setting) => resolveTheme(setting, ap.birthday, ap.themeEvents);
  const notchTheme = THEMES[themeFor(ap.notchTheme)] ?? null;
  const notchImg = useBgImage(ap.notchAuto || notchTheme ? "" : ap.notchImage);
  const notchColor = notchTheme ? notchTheme.notch : ap.notchAuto ? tint ?? ap.notchColor : ap.notchColor;
  const notchOpacity = notchTheme ? ap.notchThemeOpacity : ap.opacity;
  const bg = rgba(notchColor, notchOpacity); // color plano: también lo usan las «orejas»
  const notchFill = fillFor({ color: notchColor, opacity: notchOpacity, image: notchImg, dim: ap.notchDim });

  const onFocus = (e) => {
    if (!isTextField(e.target)) return;
    clearTimeout(unpinTimer.current);
    invoke("set_pinned", { pinned: true }).catch(() => {});
  };
  const onBlur = (e) => {
    if (!isTextField(e.target)) return;
    // Los selectores nativos de fecha/hora viven fuera del WebView y pueden provocar
    // un blur momentáneo. Esperamos un poco antes de permitir el cierre automático.
    clearTimeout(unpinTimer.current);
    unpinTimer.current = setTimeout(() => invoke("set_pinned", { pinned: false }).catch(() => {}), 700);
  };

  const [welcomed, setWelcomed] = useStore("welcomed", false);
  const mascotState = { mood, look, poke, skin: themeFor(ap.mascotTheme) };
  const habitStats = { done: habitsDone, total: activeHabits.length };

  return (
    <div className="relative h-full w-full" onFocus={onFocus} onBlur={onBlur}>
      {/* Sin espacio reservado la franja taparía la parte de arriba de las apps maximizadas: solo queda el notch. */}
      <div className="flex h-full w-full items-start justify-center" style={{ transform: `translateX(${ap.offsetX}px)` }}>
      <div
        className="relative"
        style={{
          width: expanded ? (quick ? Math.min(ap.expandedW, QUICK_W) : ap.expandedW) : peekDims?.w ?? compactW,
          height: expanded ? (quick ? ap.collapsedH + (quickAsked || aiChat.error ? QUICK_H.answer : QUICK_H.input) : ap.expandedH) : peekDims?.h ?? ap.collapsedH,
          transition: `width ${NOTCH_MS}ms ${NOTCH_EASE}, height ${NOTCH_MS}ms ${NOTCH_EASE}`,
        }}
      >
        {/* Con la barra superior activa la pastilla ya está dentro de la barra: las orejas
            solo añadían esquinas de otro tono (bordes oscuros) junto a la pastilla. */}
        {!notchImg && !settings.topbar.enabled && <Ear side="left" bg={bg} />}
        {!notchImg && !settings.topbar.enabled && <Ear side="right" bg={bg} />}
        <div
          className={`absolute inset-0 overflow-hidden text-white transition-[border-radius,box-shadow,background-color] duration-300 ${
            pomo.alert || peek?.tone === "alert" ? "animate-[ringPulse_1.1s_ease-out_infinite] ring-2 ring-amber-400" : ""
          } ${dragging ? "ring-2 ring-sky-400" : ""}`}
          style={{
            background: notchFill,
            borderRadius: expanded || greeting ? "0 0 28px 28px" : peek || (hoverShown && pillHover) ? "0 0 20px 20px" : "0 0 16px 16px",
            // Sin sombra: alrededor de la pastilla se veía un halo oscuro sobre la barra y las apps.
            boxShadow: "none",
          }}
        >
          {notchTheme && !peek && !greeting && <NotchDecor theme={notchTheme} compact={!expanded} />}
          {expanded && quick ? (
            <QuickAsk
              chat={aiChat}
              asked={quickAsked}
              onAsk={() => setQuickAsked(true)}
              onFull={() => {
                wantTab.current = "ai";
                setQuick(false);
                setTab("ai");
              }}
              collapsedH={ap.collapsedH}
            />
          ) : expanded ? (
            <ExpandedView
              tabs={tabs}
              activeTab={activeTab}
              setTab={setTab}
              settings={settings}
              update={update}
              data={{ battery, media, pomo, cal, weather, files, setFiles, dragging, aiChat }}
              mascot={mascotState}
              banner={banner}
              welcome={!welcomed ? { shortcuts: settings.shortcuts, done: () => setWelcomed(true) } : null}
            />
          ) : greeting ? (
            <button
              type="button"
              onClick={openFromGreeting}
              className="flex h-full w-full animate-[fadeIn_300ms_ease-out_120ms_both] cursor-default items-end justify-center gap-3 px-5 pb-2.5"
              style={{ paddingTop: ap.collapsedH }}
              aria-label="Saludo de bienvenida"
            >
              <Mascot size={28} mood="happy" look={look} skin={mascotState.skin} />
              <span className="truncate text-sm font-medium text-white">{greeting}</span>
            </button>
          ) : peek ? (
            <div className="relative h-full animate-[peekIn_240ms_cubic-bezier(0.2,0.8,0.2,1)_both]">
              <PeekContent peek={peek} setHold={setHold} />
            </div>
          ) : pillShown ? (
            <MediaPill media={media} collapsedH={ap.collapsedH} visualizer={ap.visualizer} hover={pillHover} setHover={setHover} />
          ) : zenShown ? (
            <ZenPill pomo={pomo} collapsedH={ap.collapsedH} hover={pillHover} setHover={setHover} />
          ) : (
            <CompactBar
              ap={ap}
              modules={modules}
              mascot={mascotState}
              indicator={{ cal, calendarOn: modules.calendar, pomo, media, dnd, rec, mic: settings.hud.microphone ? mic : null }}
              pending={pendingTasks}
              habits={habitStats}
              weatherNow={weatherNow}
              temperature={weatherNow ? Math.round(weather.data.current.temperature_2m) : null}
              battery={battery}
            />
          )}
        </div>
      </div>
      </div>
    </div>
  );
}
