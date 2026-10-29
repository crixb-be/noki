import { invoke } from "@tauri-apps/api/core";
import Clock from "../Clock.jsx";
import Battery from "../Battery.jsx";
import MediaPlayer from "../MediaPlayer.jsx";
import Productivity from "../Productivity.jsx";
import FileShelf from "../FileShelf.jsx";
import AIChat from "../AIChat.jsx";
import CalendarView from "../CalendarView.jsx";
import PanelView from "../PanelView.jsx";
import ToolsView from "../ToolsView.jsx";
import HabitsView from "../HabitsView.jsx";
import SystemView from "../SystemView.jsx";
import ScreenTime from "../ScreenTime.jsx";
import Mascot from "../Mascot.jsx";

/** Notch abierto: cabecera (reloj, batería, mascota, pestañas), la vista activa y el aviso flotante. */
/** Primera vez: un vistazo rápido a lo esencial. */
function Welcome({ shortcuts, done }) {
  const tips = [
    ["🖱️", "Pasa el cursor por el notch (o pulsa " + (shortcuts.toggleEnabled ? shortcuts.toggle : "el atajo") + ") para abrirlo."],
    ["🎙", shortcuts.recordEnabled ? `${shortcuts.record} graba tu voz desde cualquier lugar.` : "En Agenda → Grabadora guardas tu voz y la transcribes."],
    ["⚙", "Cambia colores, temas, módulos y atajos en Ajustes (el engranaje)."],
    ["🎭", "Prueba los temas de temporada en Ajustes → Apariencia."],
  ];
  return (
    <div className="absolute inset-0 z-20 flex animate-[fadeIn_250ms_ease-out_both] items-center justify-center bg-black/80 p-6 backdrop-blur-sm">
      <div className="flex max-w-sm flex-col gap-3">
        <div className="text-base font-semibold">¡Hola, soy Noki! 👋</div>
        <ul className="flex flex-col gap-2 text-xs text-white/75">
          {tips.map(([icon, text]) => (
            <li key={text} className="flex items-start gap-2.5">
              <span className="text-base leading-none">{icon}</span>
              <span>{text}</span>
            </li>
          ))}
        </ul>
        <button onClick={done} className="self-start rounded-full bg-white/15 px-4 py-1.5 text-xs font-medium transition-all hover:bg-white/25 active:scale-95">Empezar</button>
      </div>
    </div>
  );
}

export default function ExpandedView({ tabs, activeTab, setTab, settings, update, data, mascot, banner, welcome }) {
  const { appearance: ap, modules } = settings;
  const { battery, media, pomo, cal, weather, files, setFiles, dragging, aiChat } = data;
  return (
    <div className="relative flex h-full animate-[expandIn_320ms_cubic-bezier(0.2,0.8,0.2,1)_both] flex-col gap-3 p-4">
      <div className="flex shrink-0 animate-[dropIn_360ms_cubic-bezier(0.2,0.8,0.2,1)_60ms_both] items-center justify-between">
        <div className="flex items-center gap-3">
          <Clock withDate={tabs.length < 6} hour12={!ap.clock24} />
          {tabs.length < 8 && <Battery info={battery} />}
          {ap.mascot && tabs.length < 10 && <Mascot size={22} mood={mascot.mood} look={mascot.look} skin={mascot.skin} onPoke={mascot.poke} />}
        </div>
        <nav className="flex gap-0.5 rounded-full bg-white/10 p-1">
          {tabs.map((t) => (
            <button
              key={t.id}
              onClick={() => setTab(t.id)}
              title={t.label}
              aria-label={t.label}
              className={`rounded-full py-1 text-sm transition-all duration-300 ${tabs.length >= 10 ? "px-1.5" : "px-2"} ${
                activeTab === t.id ? "scale-110 bg-white/20" : "opacity-50 hover:scale-105 hover:opacity-100"
              }`}
            >
              {t.icon}
            </button>
          ))}
          <button
            onClick={() => invoke("open_settings", {})}
            title="Ajustes"
            aria-label="Ajustes"
            className="rounded-full px-2 py-1 text-sm opacity-50 transition-all duration-300 hover:rotate-45 hover:opacity-100"
          >
            ⚙
          </button>
        </nav>
      </div>
      {/* La clave reinicia la animación al cambiar de pestaña. */}
      <div key={activeTab} className="min-h-0 flex-1 animate-[tabIn_260ms_cubic-bezier(0.2,0.8,0.2,1)_both]">
        {tabs.length === 0 && (
          <div className="flex h-full flex-col items-center justify-center gap-2 text-xs text-white/40">
            Todos los módulos están desactivados.
            <button onClick={() => invoke("open_settings", { section: "modules" })} className="text-sky-300 underline">
              Abrir ajustes
            </button>
          </div>
        )}
        {activeTab === "panel" && <PanelView weather={weather} units={settings.weather.units} city={settings.weather.city} dnd={settings.general.dnd} update={update} />}
        {activeTab === "media" && <MediaPlayer media={media} lyricsAvailable={modules.lyrics} visualizer={ap.visualizer} />}
        {activeTab === "work" && <Productivity pomo={pomo} prod={settings.productivity} />}
        {activeTab === "habits" && <HabitsView />}
        {activeTab === "system" && <SystemView />}
        {activeTab === "screentime" && <ScreenTime />}
        {activeTab === "files" && <FileShelf files={files} setFiles={setFiles} dragging={dragging} />}
        {activeTab === "calendar" && <CalendarView cal={cal} />}
        {activeTab === "ai" && <AIChat ai={settings.ai} chat={aiChat} />}
        {activeTab === "tools" && <ToolsView clip={settings.clipboard} update={update} />}
      </div>
      {welcome && <Welcome {...welcome} />}
      {banner && (
        <div
          className={`pointer-events-none absolute bottom-3 left-1/2 flex max-w-[90%] -translate-x-1/2 animate-[bannerIn_320ms_cubic-bezier(0.34,1.56,0.64,1)_both] items-center gap-2 rounded-full px-4 py-1.5 text-xs font-medium shadow-lg ring-1 ${
            banner.tone === "alert" ? "bg-amber-400/20 text-amber-100 ring-amber-400/50" : "bg-white/15 text-white ring-white/20"
          }`}
        >
          <span>{banner.icon}</span>
          <span className="truncate">{banner.text}</span>
        </div>
      )}
    </div>
  );
}
