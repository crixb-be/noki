import { invoke } from "@tauri-apps/api/core";
import Clock from "../Clock.jsx";
import Battery from "../Battery.jsx";

/** Cabecera (reloj, batería, pestañas, ajustes), cuerpo de la pestaña activa y aviso flotante. */
export default function ExpandedView({ ap, tabs, activeTab, onTab, battery, banner, children }) {
  return (
    <div className="flex h-full animate-[contentIn_320ms_ease-out_80ms_both] flex-col gap-3 p-4">
      <div className="flex shrink-0 items-center justify-between">
        <div className="flex items-center gap-3">
          <Clock withDate={tabs.length < 6} hour12={!ap.clock24} />
          {tabs.length < 8 && <Battery info={battery} />}
        </div>
        <nav className="flex gap-0.5 rounded-full bg-white/10 p-1">
          {tabs.map((t) => (
            <button
              key={t.id}
              onClick={() => onTab(t.id)}
              title={t.label}
              aria-label={t.label}
              className={`rounded-full py-1 text-sm transition-all duration-300 active:scale-90 ${tabs.length >= 10 ? "px-1.5" : "px-2"} ${
                activeTab === t.id ? "bg-white/20" : "opacity-50 hover:opacity-100"
              }`}
            >
              {t.icon}
            </button>
          ))}
          <button
            onClick={() => invoke("open_settings", {})}
            title="Ajustes"
            aria-label="Ajustes"
            className="rounded-full px-2 py-1 text-sm opacity-50 transition-all duration-300 hover:opacity-100 active:scale-90"
          >
            ⚙
          </button>
        </nav>
      </div>
      {/* key: cada cambio de pestaña vuelve a montar el cuerpo y reproduce la entrada */}
      <div key={activeTab} className="min-h-0 flex-1 animate-[tabIn_220ms_ease-out_both]">
        {tabs.length === 0 && (
          <div className="flex h-full flex-col items-center justify-center gap-2 text-xs text-white/40">
            Todos los módulos están desactivados.
            <button onClick={() => invoke("open_settings", { section: "modules" })} className="text-sky-300 underline">
              Abrir ajustes
            </button>
          </div>
        )}
        {children}
      </div>
      {banner && (
        <div
          className={`pointer-events-none absolute bottom-3 left-1/2 flex max-w-[90%] -translate-x-1/2 animate-[bannerIn_260ms_cubic-bezier(0.32,0.72,0,1)_both] items-center gap-2 rounded-full px-4 py-1.5 text-xs font-medium shadow-lg ring-1 ${
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
