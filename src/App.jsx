import { useCallback, useEffect, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { useBattery } from "./components/Battery.jsx";
import MediaPlayer from "./components/MediaPlayer.jsx";
import Productivity from "./components/Productivity.jsx";
import FileShelf, { addPaths, useNativeDrop } from "./components/FileShelf.jsx";
import AIChat from "./components/AIChat.jsx";
import CalendarView from "./components/CalendarView.jsx";
import PanelView from "./components/PanelView.jsx";
import ToolsView from "./components/ToolsView.jsx";
import HabitsView from "./components/HabitsView.jsx";
import SystemView from "./components/SystemView.jsx";
import ScreenTime from "./components/ScreenTime.jsx";
import Ear from "./components/notch/Ear.jsx";
import CompactBar from "./components/notch/CompactBar.jsx";
import ExpandedView from "./components/notch/ExpandedView.jsx";
import PeekContent, { peekSize } from "./components/Peek.jsx";
import { useCalendar } from "./hooks/useCalendar.js";
import { useWeather } from "./hooks/useWeather.js";
import { useStore, useStoreReady } from "./hooks/useStore.js";
import { useReminders } from "./hooks/useReminders.js";
import { usePomodoro } from "./hooks/usePomodoro.js";
import { useMedia } from "./hooks/useMedia.js";
import { useSettings } from "./hooks/useSettings.js";
import { usePeeks } from "./hooks/usePeeks.js";
import { useAiChat } from "./hooks/useAiChat.js";
import { useNotchOpen } from "./hooks/useNotchOpen.js";
import { EMPTY_HABITS, countOn, useHabits } from "./hooks/useHabits.js";
import { fillFor, rgba, useBgImage } from "./lib/background.js";
import { weatherInfo } from "./lib/wmo.js";
import { NOTCH_EASE, NOTCH_MS, compactWidth, notchRadius } from "./lib/notchGeometry.js";
import { presetLanguage, useLanguage } from "./lib/i18n.js";

/** Etiqueta de esta ventana: "main" o "main-1", "main-2"… (un notch por pantalla). */
const LABEL = getCurrentWindow().label;

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
  const expanded = useNotchOpen(LABEL);
  const [tab, setTab] = useState("media");
  const prod = settings.productivity;

  const battery = useBattery();
  const media = useMedia();
  const [files, setFiles] = useStore("files", []);
  const unpinTimer = useRef(null);

  const dnd = settings.general.dnd;
  const { peek: rawPeek, setHold, show } = usePeeks({ hud: settings.hud, battery, dnd });

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
  useEffect(() => {
    let unlisten;
    let cancelled = false;
    listen("notch-notify", (e) => {
      const { ms, ...p } = e.payload;
      show({ kind: "text", ...p }, ms);
    }).then((fn) => (cancelled ? fn() : (unlisten = fn)));
    return () => {
      cancelled = true;
      unlisten?.();
    };
  }, [show]);

  const pomo = usePomodoro(prod, notify, leader);
  useReminders({ health: settings.health, notify, leader });
  useHabits({ enabled: settings.modules.habits, notify, leader });
  const cal = useCalendar({ calendar: settings.calendar, enabled: settings.modules.calendar, notify, leader });
  const [todos] = useStore("todos", []);
  const pendingTasks = todos.filter((t) => !t.done).length;
  const [habits] = useStore("habits", EMPTY_HABITS);
  const activeHabits = (habits.items ?? []).filter((i) => i.active !== false);
  const habitsDone = activeHabits.filter((i) => countOn(habits, i.id) >= i.goal).length;
  const aiChat = useAiChat(); // el chat sigue respondiendo aunque cambies de pestaña
  const weather = useWeather({ weather: settings.weather, enabled: settings.modules.weather, leader });

  // Aviso cuando la comprobación automática encuentra una versión nueva (nunca se instala sola).
  useEffect(() => {
    if (!leader) return;
    let unlisten;
    let cancelled = false;
    listen("update-available", (e) =>
      notify({ icon: "⬆️", text: `Noki ${e.payload.version} disponible · Ajustes → Acerca de`, ms: 9000, silent: true })
    ).then((fn) => (cancelled ? fn() : (unlisten = fn)));
    return () => {
      cancelled = true;
      unlisten?.();
    };
  }, [notify, leader]);

  const dragging = useNativeDrop({
    onEnter: () => {
      setTab("files");
      invoke("request_open", { open: true, sticky: false }).catch(() => {});
    },
    onDrop: (paths) => addPaths(paths, setFiles),
  });

  // Esc cierra el notch (útil cuando se abrió con el atajo).
  useEffect(() => {
    const onKey = (e) => e.key === "Escape" && invoke("request_open", { open: false, sticky: false }).catch(() => {});
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, []);

  // Pestaña inicial según los ajustes (cada vez que se abre, salvo mientras se arrastra un archivo).
  const defaultTab = settings.modules.defaultTab;
  useEffect(() => {
    if (expanded && !dragging && defaultTab && defaultTab !== "last") setTab(defaultTab);
  }, [expanded]); // eslint-disable-line react-hooks/exhaustive-deps

  const peek = expanded ? null : rawPeek;
  const banner = expanded && rawPeek?.kind === "text" ? rawPeek : null;
  const apx = settings.appearance;
  const peekDims = peek ? peekSize(peek, settings.appearance.collapsedW, settings.appearance.collapsedH) : null;
  const compactW = compactWidth(apx);

  // Rust necesita saber el tamaño del aviso para que su zona reciba clics (slider de volumen/brillo).
  useEffect(() => {
    invoke("set_peek", { width: peekDims?.w ?? (compactW > apx.collapsedW ? compactW : 0), height: peekDims?.h ?? 0 }).catch(() => {});
  }, [peekDims?.w, peekDims?.h, compactW, apx.collapsedW]);
  useEffect(() => {
    if (expanded) setHold(false);
  }, [expanded, setHold]);

  const { appearance: ap, modules } = settings;
  const tabs = TABS.filter((t) => modules[t.id]);
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
  useEffect(() => {
    if (ap.barMode !== "auto" && !ap.notchAuto) return;
    let unlisten;
    let cancelled = false;
    listen("bar-tint", (e) => e.payload.label === LABEL && setTint(e.payload.color)).then((fn) => (cancelled ? fn() : (unlisten = fn)));
    return () => {
      cancelled = true;
      unlisten?.();
    };
  }, [ap.barMode, ap.notchAuto]);
  const notchImg = useBgImage(ap.notchAuto ? "" : ap.notchImage);
  const barImg = useBgImage(ap.barMode === "image" ? ap.barImage : "");
  const notchColor = ap.notchAuto ? tint ?? ap.notchColor : ap.notchColor;
  const bg = rgba(notchColor, ap.opacity); // color plano: también lo usan las «orejas»
  const notchFill = fillFor({ color: notchColor, opacity: ap.opacity, image: notchImg, dim: ap.notchDim });
  const barFill =
    ap.barMode === "none"
      ? null
      : ap.barMode === "match"
        ? bg
        : ap.barMode === "auto"
          ? rgba(tint ?? ap.notchColor, ap.barOpacity)
          : ap.barMode === "color"
            ? rgba(ap.barColor, ap.barOpacity)
            : fillFor({ color: ap.barColor, opacity: ap.barOpacity, image: barImg, dim: ap.barDim });

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

  return (
    <div className="relative h-full w-full" onFocus={onFocus} onBlur={onBlur}>
      {barFill && <div className="pointer-events-none absolute left-0 right-0 top-0" style={{ height: ap.collapsedH, background: barFill, transition: "background-color 350ms ease" }} />}
      <div className="flex h-full w-full items-start justify-center" style={{ transform: `translateX(${ap.offsetX}px)` }}>
      <div
        className="relative"
        style={{
          width: expanded ? ap.expandedW : peekDims?.w ?? compactW,
          height: expanded ? ap.expandedH : peekDims?.h ?? ap.collapsedH,
          transition: `width ${NOTCH_MS}ms ${NOTCH_EASE}, height ${NOTCH_MS}ms ${NOTCH_EASE}`,
        }}
      >
        {!notchImg && <Ear side="left" bg={bg} />}
        {!notchImg && <Ear side="right" bg={bg} />}
        <div
          className={`absolute inset-0 overflow-hidden text-white transition-[border-radius,box-shadow,background-color] duration-300 ${
            pomo.alert || peek?.tone === "alert" ? "animate-[ringPulse_1.1s_ease-out_infinite] ring-2 ring-amber-400" : ""
          } ${dragging ? "ring-2 ring-sky-400" : ""}`}
          style={{
            background: notchFill,
            borderRadius: notchRadius(expanded, peek),
            boxShadow: expanded ? "0 14px 36px rgba(0,0,0,0.55)" : "none",
          }}
        >
          {expanded ? (
            <ExpandedView ap={ap} tabs={tabs} activeTab={activeTab} onTab={setTab} battery={battery} banner={banner}>
              {activeTab === "panel" && <PanelView weather={weather} units={settings.weather.units} city={settings.weather.city} dnd={dnd} update={update} />}
              {activeTab === "media" && <MediaPlayer media={media} lyricsAvailable={modules.lyrics} visualizer={ap.visualizer} />}
              {activeTab === "work" && <Productivity pomo={pomo} prod={prod} />}
              {activeTab === "habits" && <HabitsView />}
              {activeTab === "system" && <SystemView />}
              {activeTab === "screentime" && <ScreenTime />}
              {activeTab === "files" && <FileShelf files={files} setFiles={setFiles} dragging={dragging} />}
              {activeTab === "calendar" && <CalendarView cal={cal} />}
              {activeTab === "ai" && <AIChat ai={settings.ai} chat={aiChat} />}
              {activeTab === "tools" && <ToolsView clip={settings.clipboard} update={update} />}
            </ExpandedView>
          ) : peek ? (
            <div className="h-full animate-[contentIn_220ms_ease-out_both]">
              <PeekContent peek={peek} setHold={setHold} />
            </div>
          ) : (
            <CompactBar
              ap={ap}
              modules={modules}
              dnd={dnd}
              cal={cal}
              pomo={pomo}
              media={media}
              battery={battery}
              pendingTasks={pendingTasks}
              activeHabits={activeHabits}
              habitsDone={habitsDone}
              weatherNow={weatherNow}
              temperature={weather.data?.current?.temperature_2m}
            />
          )}
        </div>
      </div>
      </div>
    </div>
  );
}
