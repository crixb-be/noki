import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { getVersion } from "@tauri-apps/api/app";
import { open, save } from "@tauri-apps/plugin-dialog";
import { useSettings } from "../hooks/useSettings.js";
import { useBgImage } from "../lib/background.js";
import { presetLanguage, useLanguage } from "../lib/i18n.js";
import Mascot from "./Mascot.jsx";
import { THEMES, THEME_CHOICES, resolveTheme } from "../lib/themes.js";
import NokiLogo from "./NokiLogo.jsx";
import { APP_ICONS } from "../lib/icons.js";

/** Datos del desarrollador (apartado «Desarrollador»). */
const DEV = {
  name: "Cristhian Bustamante",
  links: [
    { id: "instagram", label: "Instagram", url: "https://www.instagram.com/cristhian.billy/" },
    { id: "facebook", label: "Facebook", url: "https://www.facebook.com/cristhian.bustamanteespinoza/" },
    { id: "x", label: "X", url: "https://x.com/Cristhian_0702" },
  ],
  kofi: "https://ko-fi.com/cristhianbillybustamante",
  /** Dirección del repositorio en GitHub; vacío = no se muestra el botón. */
  repo: "https://github.com/crixb-be/noki",
};

const SECTIONS = [
  { id: "general", icon: "⚙", label: "General" },
  { id: "appearance", icon: "🎨", label: "Apariencia" },
  { id: "modules", icon: "🧩", label: "Módulos" },
  { id: "productivity", icon: "📋", label: "Agenda" },
  { id: "calendar", icon: "📅", label: "Calendario" },
  { id: "weather", icon: "🌤️", label: "Clima" },
  { id: "tools", icon: "🧰", label: "Herramientas" },
  { id: "alerts", icon: "🔔", label: "Avisos" },
  { id: "shortcuts", icon: "⌨", label: "Atajos" },
  { id: "ai", icon: "✨", label: "IA" },
  { id: "developer", icon: "👤", label: "Desarrollador" },
  { id: "about", icon: "ℹ", label: "Acerca de" },
];

// ───────────── primitivas de formulario ─────────────

function Row({ title, hint, children }) {
  return (
    <div className="flex items-center justify-between gap-6 border-b border-white/5 py-3.5">
      <div className="min-w-0">
        <div className="text-sm">{title}</div>
        {hint && <div className="mt-0.5 text-xs leading-snug text-white/40">{hint}</div>}
      </div>
      <div className="shrink-0">{children}</div>
    </div>
  );
}

function Toggle({ checked, onChange }) {
  return (
    <button
      role="switch"
      aria-checked={checked}
      onClick={() => onChange(!checked)}
      className={`relative h-6 w-11 rounded-full transition-colors duration-200 ${checked ? "bg-emerald-500" : "bg-white/15"}`}
    >
      <span
        className="absolute left-0.5 top-0.5 h-5 w-5 rounded-full bg-white shadow transition-transform duration-200"
        style={{ transform: checked ? "translateX(20px)" : "translateX(0)" }}
      />
    </button>
  );
}

function Slider({ value, min, max, step = 1, unit = "", onChange }) {
  return (
    <div className="flex items-center gap-3">
      <input
        type="range"
        min={min}
        max={max}
        step={step}
        value={value}
        onChange={(e) => onChange(Number(e.target.value))}
        className="h-1 w-44 cursor-pointer accent-white"
      />
      <span className="w-16 text-right text-xs tabular-nums text-white/60">
        {value}
        {unit}
      </span>
    </div>
  );
}

function Select({ value, options, onChange, native = false }) {
  return (
    <select
      data-no-i18n={native ? "" : undefined}
      value={value}
      onChange={(e) => onChange(e.target.value)}
      className="rounded-lg border border-white/10 bg-zinc-900 px-3 py-1.5 text-sm outline-none focus:border-white/30"
    >
      {options.map(([v, label]) => (
        <option key={v} value={v}>
          {label}
        </option>
      ))}
    </select>
  );
}

const input =
  "w-72 rounded-lg border border-white/10 bg-zinc-900 px-3 py-1.5 text-sm outline-none placeholder:text-white/30 focus:border-white/30";
const button = "rounded-lg bg-white/10 px-3 py-1.5 text-sm transition-colors hover:bg-white/20";

function Page({ title, subtitle, children }) {
  return (
    <div>
      <h1 className="text-xl font-semibold">{title}</h1>
      {subtitle && <p className="mt-1 text-sm text-white/40">{subtitle}</p>}
      <div className="mt-4">{children}</div>
    </div>
  );
}

// ───────────── apartados ─────────────

function General({ s, set, monitors }) {
  const g = s.general;
  return (
    <Page title="General" subtitle="Cómo se comporta el notch en tu escritorio.">
      <Row title="Idioma de la interfaz" hint="Español / English. Los cambios se aplican al instante.">
        <Select native value={g.language} onChange={(v) => set({ general: { language: v } })} options={[["es", "Español"], ["en", "English"]]} />
      </Row>
      <Row title="Mostrar el notch" hint="También puedes ocultarlo desde el icono de la bandeja.">
        <Toggle checked={g.visible} onChange={(v) => set({ general: { visible: v } })} />
      </Row>
      <Row title="Iniciar con Windows" hint="Se abre solo al iniciar sesión. Actívalo desde la versión instalada.">
        <Toggle checked={g.autostart} onChange={(v) => set({ general: { autostart: v } })} />
      </Row>
      <Row
        title="Reservar espacio en la parte superior"
        hint="Las demás ventanas trabajan por debajo del notch, como una barra de tareas. Si lo desactivas, el notch flota encima de ellas."
      >
        <Toggle checked={g.reserveSpace} onChange={(v) => set({ general: { reserveSpace: v } })} />
      </Row>
      <Row title="Ocultar en pantalla completa" hint="Se aparta cuando hay un juego, un vídeo o una presentación a pantalla completa.">
        <Toggle checked={g.hideOnFullscreen} onChange={(v) => set({ general: { hideOnFullscreen: v } })} />
      </Row>
      <Row title={g.allMonitors ? "Pantalla principal del notch" : "Monitor"} hint={g.allMonitors ? "El notch principal (el que genera los avisos); en las demás pantallas se crea uno más." : "Pantalla donde aparece el notch."}>
        <Select
          value={g.monitor}
          onChange={(v) => set({ general: { monitor: Number(v) } })}
          options={[
            [-1, "Principal"],
            ...monitors.map((m) => [m.index, `${m.name} (${m.width}×${m.height})${m.primary ? " · principal" : ""}`]),
          ]}
        />
      </Row>
      <Row
        title="Un notch en cada pantalla"
        hint={
          monitors.length > 1
            ? `Se muestra un notch en cada una de tus ${monitors.length} pantallas. Comparten datos, música y avisos; cada uno se abre por separado.`
            : "Solo detecto una pantalla; si conectas otra, se añade sola."
        }
      >
        <Toggle checked={g.allMonitors} onChange={(v) => set({ general: { allMonitors: v } })} />
      </Row>
      <Row title="Abrir al" hint="«Borde superior» abre al llegar con el cursor arriba, sin tener que apuntar a la píldora.">
        <Select
          value={g.trigger}
          onChange={(v) => set({ general: { trigger: v } })}
          options={[
            ["notch", "Pasar sobre el notch"],
            ["edge", "Llegar al borde superior"],
          ]}
        />
      </Row>
      <Row title="Retraso al abrir">
        <Slider value={g.hoverDelay} min={0} max={800} step={20} unit=" ms" onChange={(v) => set({ general: { hoverDelay: v } })} />
      </Row>
      <Row title="Retraso al cerrar">
        <Slider value={g.closeDelay} min={0} max={1500} step={50} unit=" ms" onChange={(v) => set({ general: { closeDelay: v } })} />
      </Row>
    </Page>
  );
}

const SWATCHES = ["#000000", "#111827", "#1e1b4b", "#3b0764", "#4c0519", "#052e16", "#431407", "#27272a"];

/** Colores rápidos más un selector libre. */
function ColorPicker({ value, onChange }) {
  return (
    <div className="flex items-center gap-1.5">
      {SWATCHES.map((c) => (
        <button
          key={c}
          onClick={() => onChange(c)}
          title={c}
          aria-label={c}
          className={`h-6 w-6 rounded-full ring-1 transition-transform hover:scale-110 ${value.toLowerCase() === c ? "ring-2 ring-sky-400" : "ring-white/25"}`}
          style={{ background: c }}
        />
      ))}
      <input type="color" value={value} onChange={(e) => onChange(e.target.value)} title="Otro color" className="h-7 w-9 cursor-pointer rounded-md border-0 bg-transparent p-0" />
    </div>
  );
}

/** Elegir una imagen (se copia a la carpeta de la app) o quitarla. */
function ImagePicker({ name, onChange }) {
  const url = useBgImage(name);
  const [error, setError] = useState(null);
  const choose = async () => {
    setError(null);
    try {
      const path = await open({ multiple: false, title: "Imagen de fondo", filters: [{ name: "Imágenes", extensions: ["png", "jpg", "jpeg", "webp", "gif", "bmp"] }] });
      if (path) onChange(await invoke("bg_import", { path }));
    } catch (e) {
      setError(String(e));
    }
  };
  return (
    <div className="flex flex-col items-end gap-1">
      <div className="flex items-center gap-2">
        {name && <div className="h-8 w-14 rounded-md bg-cover bg-center ring-1 ring-white/20" style={{ backgroundImage: url ? `url("${url}")` : "none" }} />}
        <button className={button} onClick={choose}>{name ? "Cambiar…" : "Elegir imagen…"}</button>
        {name && <button className={button} onClick={() => onChange("")}>Quitar</button>}
      </div>
      {error && <span className="text-xs text-amber-300">{error}</span>}
    </div>
  );
}

/** Elegir el icono de Noki (bandeja y ventana de Ajustes). */
function IconPicker({ value, onChange }) {
  return (
    <div className="border-b border-white/5 pb-4 pt-3.5">
      <div className="text-sm">Icono de Noki</div>
      <div className="mt-0.5 text-xs leading-snug text-white/40">
        Cambia el icono de la bandeja y de Ajustes. El del instalador y el de los accesos directos lo fija Windows al instalar y no cambia.
      </div>
      <div className="mt-3 flex flex-wrap gap-3">
        {APP_ICONS.map((i) => (
          <button
            key={i.id}
            onClick={() => onChange(i.id)}
            aria-pressed={value === i.id}
            className={`flex w-24 flex-col items-center gap-1.5 rounded-2xl p-2.5 text-xs transition-all duration-200 ${
              value === i.id ? "bg-white/15 ring-2 ring-white/60" : "bg-white/5 hover:bg-white/10"
            }`}
          >
            <NokiLogo size={52} variant={i.id} />
            <span className="text-center leading-tight text-white/75">{i.name}</span>
          </button>
        ))}
      </div>
    </div>
  );
}

function Appearance({ s, set }) {
  const a = s.appearance;
  const ap = (patch) => set({ appearance: patch });
  return (
    <Page title="Apariencia" subtitle="Tamaño, posición y transparencia. Los cambios se ven al instante en el notch.">
      <IconPicker value={a.appIcon} onChange={(v) => ap({ appIcon: v })} />
      <Row title="Personaje en el notch" hint="Una carita que parpadea, te mira y reacciona: se alegra al terminar tareas, hábitos o un Pomodoro, baila con la música y se adormece de noche.">
        <Toggle checked={a.mascot} onChange={(v) => ap({ mascot: v })} />
      </Row>
      {(
        <>
          <div className="border-b border-white/5 pb-4 pt-3.5">
            <div className="text-sm">Tema de temporada</div>
            <div className="mt-0.5 text-xs leading-snug text-white/40">
              «Automático» pone Halloween (24 oct–2 nov), Navidad (1–26 dic), Amor (7–14 feb) y Cumpleaños el día que indiques abajo.
            </div>
            <div className="mt-3 flex flex-wrap gap-3">
              {THEME_CHOICES.map(([id, label]) => {
                const shown = id === "auto" ? resolveTheme("auto", a.birthday) : id;
                const t = THEMES[shown];
                return (
                  <button
                    key={id}
                    onClick={() => ap({ theme: id })}
                    aria-pressed={a.theme === id}
                    className={`flex w-24 flex-col items-center gap-1.5 rounded-2xl p-2.5 text-xs transition-all duration-200 ${
                      a.theme === id ? "bg-white/15 ring-2 ring-white/60" : "bg-white/5 hover:bg-white/10"
                    }`}
                  >
                    <span
                      className="flex h-12 w-full items-center justify-center overflow-hidden rounded-xl border-t-4"
                      style={{ background: a.themeColors && t ? t.notch : "#000", borderColor: a.themeColors && t ? t.bar : "transparent" }}
                    >
                      {a.themeMascot ? <Mascot size={26} skin={shown} /> : <Mascot size={26} />}
                    </span>
                    <span className="text-center leading-tight text-white/75">{label}</span>
                  </button>
                );
              })}
            </div>
          </div>
          <Row title="Vestir al personaje" hint="Gorro de bruja, de Papá Noel, de fiesta o ojitos de corazón.">
            <Toggle checked={a.themeMascot} onChange={(v) => ap({ themeMascot: v })} />
          </Row>
          <Row title="Colores del notch y los costados" hint="Pinta el notch y la franja lateral con los colores del tema. Tus colores e imagen guardados no se pierden: al desactivarlo vuelven.">
            <Toggle checked={a.themeColors} onChange={(v) => ap({ themeColors: v })} />
          </Row>
          <Row title="Tu cumpleaños" hint="Solo día y mes; se guarda únicamente en este equipo. Ese día se activa el tema de cumpleaños.">
            <div className="flex items-center gap-2">
              <input
                type="date"
                value={a.birthday ? `2000-${a.birthday}` : ""}
                onChange={(e) => ap({ birthday: e.target.value ? e.target.value.slice(5) : "" })}
                className="rounded-lg border border-white/10 bg-zinc-900 px-3 py-1.5 text-sm outline-none focus:border-white/30 [color-scheme:dark]"
              />
              {a.birthday && (
                <button onClick={() => ap({ birthday: "" })} className="text-xs text-white/40 hover:text-white/80">
                  Quitar
                </button>
              )}
            </div>
          </Row>
        </>
      )}
      <Row title="Ancho (compacto)">
        <Slider value={a.collapsedW} min={120} max={600} step={2} unit=" px" onChange={(v) => ap({ collapsedW: v })} />
      </Row>
      <Row title="Alto (compacto)" hint="También es el espacio que se reserva arriba si esa opción está activa.">
        <Slider value={a.collapsedH} min={24} max={80} unit=" px" onChange={(v) => ap({ collapsedH: v })} />
      </Row>
      <Row title="Ancho (expandido)">
        <Slider value={a.expandedW} min={360} max={900} step={4} unit=" px" onChange={(v) => ap({ expandedW: v })} />
      </Row>
      <Row title="Alto (expandido)">
        <Slider value={a.expandedH} min={200} max={600} step={4} unit=" px" onChange={(v) => ap({ expandedH: v })} />
      </Row>
      <Row title="Desplazamiento horizontal" hint="0 = centrado en la pantalla.">
        <Slider value={a.offsetX} min={-800} max={800} step={4} unit=" px" onChange={(v) => ap({ offsetX: v })} />
      </Row>
      <Row title="Opacidad del fondo" hint="Con una imagen de fondo en el notch no se aplica; usa «Oscurecer imagen».">
        <Slider value={a.opacity} min={30} max={100} unit=" %" onChange={(v) => ap({ opacity: v })} />
      </Row>
      <div className="mb-1 mt-5 text-xs uppercase tracking-wider text-white/35">Fondo del notch</div>
      <Row title="Color automático" hint="El notch toma el color del programa que tengas abierto debajo y cambia con él. Reemplaza al color y a la imagen.">
        <Toggle checked={a.notchAuto} onChange={(v) => ap({ notchAuto: v })} />
      </Row>
      {!a.notchAuto && (
        <>
          <Row title="Color">
            <ColorPicker value={a.notchColor} onChange={(v) => ap({ notchColor: v })} />
          </Row>
          <Row title="Imagen" hint="Se usa como fondo del notch en lugar del color. Se guarda una copia, así que no importa si mueves el original.">
            <ImagePicker name={a.notchImage} onChange={(v) => ap({ notchImage: v })} />
          </Row>
        </>
      )}
      {!a.notchAuto && a.notchImage && (
        <Row title="Oscurecer imagen" hint="Ayuda a que el texto se lea sobre fotos claras.">
          <Slider value={a.notchDim} min={0} max={85} unit=" %" onChange={(v) => ap({ notchDim: v })} />
        </Row>
      )}
      <div className="mb-1 mt-5 text-xs uppercase tracking-wider text-white/35">Franja a los costados del notch</div>
      <Row title="Fondo de la franja" hint="La barra de arriba que cruza toda la pantalla (la que reserva el espacio). «Igual que el notch» la hace parecer parte de él; «Automático» toma el color del programa que tengas abierto debajo y cambia con él.">
        <Select
          value={a.barMode}
          onChange={(v) => ap({ barMode: v })}
          options={[["match", "Igual que el notch"], ["auto", "Automático (color del programa)"], ["none", "Transparente"], ["color", "Color"], ["image", "Imagen"]]}
        />
      </Row>
      {a.barMode === "color" && (
        <Row title="Color de la franja">
          <ColorPicker value={a.barColor} onChange={(v) => ap({ barColor: v })} />
        </Row>
      )}
      {a.barMode === "image" && (
        <>
          <Row title="Imagen de la franja" hint="Se estira para cubrir todo el ancho; funcionan mejor las imágenes panorámicas.">
            <ImagePicker name={a.barImage} onChange={(v) => ap({ barImage: v })} />
          </Row>
          {a.barImage && (
            <Row title="Oscurecer imagen">
              <Slider value={a.barDim} min={0} max={85} unit=" %" onChange={(v) => ap({ barDim: v })} />
            </Row>
          )}
        </>
      )}
      {(a.barMode === "color" || a.barMode === "image" || a.barMode === "auto") && (
        <Row title="Opacidad de la franja">
          <Slider value={a.barOpacity} min={0} max={100} unit=" %" onChange={(v) => ap({ barOpacity: v })} />
        </Row>
      )}
      <div className="mb-1 mt-5 text-xs uppercase tracking-wider text-white/35">Contenido</div>
      <div className="mb-1 mt-5 text-xs uppercase tracking-wider text-white/35">Qué se ve en el notch cerrado</div>
      <Row title="Hora">
        <Toggle checked={a.showClock} onChange={(v) => ap({ showClock: v })} />
      </Row>
      <Row title="Fecha" hint="Día de la semana y fecha, a la izquierda.">
        <Toggle checked={a.showDate} onChange={(v) => ap({ showDate: v })} />
      </Row>
      <Row title="Tareas pendientes" hint="Cuántas tareas tienes sin completar (✓ 3).">
        <Toggle checked={a.showTasks} onChange={(v) => ap({ showTasks: v })} />
      </Row>
      <Row title="Hábitos de hoy" hint="Cuántos hábitos completaste hoy (🎯 2/5).">
        <Toggle checked={a.showHabits} onChange={(v) => ap({ showHabits: v })} />
      </Row>
      <Row title="Temperatura" hint="El clima de tu ciudad (se configura en Ajustes → Clima).">
        <Toggle checked={a.showWeather} onChange={(v) => ap({ showWeather: v })} />
      </Row>
      <Row title="Batería">
        <Toggle checked={a.showBattery} onChange={(v) => ap({ showBattery: v })} />
      </Row>
      <Row title="Actividad en el centro" hint="Música sonando, Pomodoro, reunión próxima o No molestar.">
        <Toggle checked={a.showActivity} onChange={(v) => ap({ showActivity: v })} />
      </Row>
      <div className="mb-1 mt-5 text-xs uppercase tracking-wider text-white/35">Reloj</div>
      <Row title="Reloj de 24 horas" hint="Apagado muestra 12 horas con a. m. / p. m.">
        <Toggle checked={a.clock24} onChange={(v) => ap({ clock24: v })} />
      </Row>
      <div className="mb-1 mt-5 text-xs uppercase tracking-wider text-white/35">Otros</div>
      <Row title="Visualizador de audio" hint="Barras que reaccionan al sonido real del equipo mientras suena música. Solo se analiza el nivel por banda; no se guarda audio.">
        <Toggle checked={a.visualizer} onChange={(v) => ap({ visualizer: v })} />
      </Row>
    </Page>
  );
}

function Modules({ s, set }) {
  const m = s.modules;
  const items = [
    ["panel", "🏠 Panel", "Interruptores (Wi-Fi, Bluetooth, mantener activo, modo oscuro…), clima y favoritos."],
    ["media", "🎵 Música", "Reproductor y controles multimedia."],
    ["work", "📋 Agenda", "Tareas, notas y enfoque."],
    ["files", "📁 Archivos", "Estante para soltar y guardar archivos."],
    ["calendar", "📅 Calendario", "Próximas reuniones y aviso en el notch."],
    ["weather", "🌤️ Clima", "Tiempo, horas y días dentro del Panel."],
    ["ai", "✨ Asistente IA", "Chat con Gemini, Claude, OpenAI o Ollama."],
    ["habits", "💧 Hábitos", "Agua, pastillas y lo que quieras, con recordatorios."],
    ["system", "🖥️ Sistema", "CPU, RAM, discos, red y los programas que más consumen."],
    ["screentime", "⏱️ Tiempo de pantalla", "Cuánto usas cada aplicación (apagado por defecto; se guarda solo en este equipo)."],
    ["tools", "🧰 Herramientas", "Lanzador de apps y portapapeles."],
  ];
  const enabled = items.filter(([id]) => m[id]);
  // Orden elegido: Rust garantiza una lista completa; «Clima» no es pestaña propia, va dentro del Panel.
  const rank = (id) => (m.order?.indexOf(id) ?? -1) >= 0 ? m.order.indexOf(id) : 99;
  const sorted = [...items].sort((a, b) => rank(a[0]) - rank(b[0]));
  const tabIds = sorted.map(([id]) => id).filter((id) => id !== "weather");
  const move = (id, dir) => {
    const i = tabIds.indexOf(id);
    const j = i + dir;
    if (j < 0 || j >= tabIds.length) return;
    const next = [...tabIds];
    [next[i], next[j]] = [next[j], next[i]];
    set({ modules: { order: next } });
  };
  return (
    <Page title="Módulos" subtitle="Elige qué pestañas aparecen en el notch y en qué orden.">
      {sorted.map(([id, label, hint]) => (
        <Row key={id} title={label} hint={hint}>
          <div className="flex items-center gap-3">
            {id !== "weather" && (
              <div className="flex gap-1">
                {[[-1, "▲", "Subir"], [1, "▼", "Bajar"]].map(([dir, glyph, tip]) => (
                  <button
                    key={dir}
                    onClick={() => move(id, dir)}
                    disabled={tabIds.indexOf(id) + dir < 0 || tabIds.indexOf(id) + dir >= tabIds.length}
                    title={tip}
                    aria-label={tip}
                    className="rounded-md bg-white/10 px-1.5 py-0.5 text-[10px] transition-colors hover:bg-white/20 disabled:opacity-25"
                  >
                    {glyph}
                  </button>
                ))}
              </div>
            )}
            <Toggle checked={m[id]} onChange={(v) => set({ modules: { [id]: v } })} />
          </div>
        </Row>
      ))}
      <Row title="🎤 Letras en el reproductor" hint="Botón de letra sincronizada (LRCLIB) dentro de la pestaña de música.">
        <Toggle checked={m.lyrics} onChange={(v) => set({ modules: { lyrics: v } })} />
      </Row>
      <Row title="Pestaña al abrir" hint="«Última usada» recuerda dónde te quedaste.">
        <Select
          value={m.defaultTab}
          onChange={(v) => set({ modules: { defaultTab: v } })}
          options={[["last", "Última usada"], ...enabled.filter(([id]) => id !== "weather").map(([id, label]) => [id, label.slice(3)])]}
        />
      </Row>
    </Page>
  );
}

function Productivity({ s, set }) {
  const p = s.productivity;
  const pp = (patch) => set({ productivity: patch });
  return (
    <Page title="Agenda" subtitle="Tareas, notas y enfoque.">
      <div className="mb-1 mt-2 text-xs uppercase tracking-wider text-white/35">Pomodoro</div>
      <Row title="Enfoque">
        <Slider value={p.workMin} min={5} max={90} unit=" min" onChange={(v) => pp({ workMin: v })} />
      </Row>
      <Row title="Descanso corto">
        <Slider value={p.shortBreakMin} min={1} max={30} unit=" min" onChange={(v) => pp({ shortBreakMin: v })} />
      </Row>
      <Row title="Descanso largo">
        <Slider value={p.longBreakMin} min={5} max={60} unit=" min" onChange={(v) => pp({ longBreakMin: v })} />
      </Row>
      <Row title="Ciclos hasta el descanso largo">
        <Slider value={p.cyclesBeforeLong} min={2} max={8} onChange={(v) => pp({ cyclesBeforeLong: v })} />
      </Row>
      <Row title="Empezar la siguiente fase sola" hint="Si está apagado, el notch avisa y espera a que pulses Iniciar.">
        <Toggle checked={p.autoNext} onChange={(v) => pp({ autoNext: v })} />
      </Row>
      <Row title="Sonido en las alertas" hint="Fin de fase, tarea vencida y recordatorio de agua.">
        <Toggle checked={p.sound} onChange={(v) => pp({ sound: v })} />
      </Row>

      <div className="mb-1 mt-6 text-xs uppercase tracking-wider text-white/35">Salud</div>
      <Row title="Descanso de vista (20-20-20)" hint="Cada cierto tiempo de uso te recuerda mirar algo lejano durante 20 segundos.">
        <Toggle checked={s.health.eyes} onChange={(v) => set({ health: { eyes: v } })} />
      </Row>
      {s.health.eyes && (
        <Row title="Cada">
          <Slider value={s.health.eyesEveryMin} min={10} max={60} step={5} unit=" min" onChange={(v) => set({ health: { eyesEveryMin: v } })} />
        </Row>
      )}
      <Row title="Levantarse y estirarse" hint="Recordatorio periódico para moverte.">
        <Toggle checked={s.health.stand} onChange={(v) => set({ health: { stand: v } })} />
      </Row>
      {s.health.stand && (
        <Row title="Cada">
          <Slider value={s.health.standEveryMin} min={15} max={180} step={15} unit=" min" onChange={(v) => set({ health: { standEveryMin: v } })} />
        </Row>
      )}
      <div className="mt-3 text-xs leading-relaxed text-white/35">
        Si llevas más de 5 minutos sin tocar el teclado ni el ratón, se asume que descansaste y los contadores vuelven a empezar.
      </div>
    </Page>
  );
}

function ToolsSettings({ s, set }) {
  const c = s.clipboard;
  return (
    <Page title="Herramientas" subtitle="Lanzador de aplicaciones y portapapeles.">
      <Row
        title="Historial del portapapeles"
        hint="Guarda el texto que copias para reutilizarlo. Solo en memoria (se borra al cerrar la app o al desactivarlo) y no registra lo que copian los gestores de contraseñas."
      >
        <Toggle checked={c.enabled} onChange={(v) => set({ clipboard: { enabled: v } })} />
      </Row>
      {c.enabled && (
        <Row title="Elementos que se recuerdan">
          <Slider value={c.max} min={5} max={100} step={5} onChange={(v) => set({ clipboard: { max: v } })} />
        </Row>
      )}
      <div className="mt-4 text-xs leading-relaxed text-white/35">
        El lanzador usa los accesos del menú Inicio. Marca con ★ tus favoritos para tenerlos a un clic; clic derecho sobre un favorito lo quita.
      </div>
    </Page>
  );
}

function Alerts({ s, set }) {
  const h = s.hud;
  const items = [
    ["volume", "🔊 Volumen", "Slider al cambiar el volumen; también puedes arrastrarlo."],
    ["brightness", "🔆 Brillo", "Slider al cambiar el brillo de la pantalla del portátil."],
    ["charger", "⚡ Cargador", "Al conectar o desconectar el cargador."],
    ["lowBattery", "🪫 Batería baja", "Al bajar al 20 % y al 10 % sin cargador."],
    ["network", "📶 Internet", "Cuando se pierde o vuelve la conexión."],
    ["audioDevice", "🎧 Dispositivo de audio", "Al conectar auriculares o cambiar la salida de sonido."],
    ["capsLock", "⇪ Bloq Mayús", "Al activar o desactivar las mayúsculas."],
    ["language", "🌐 Idioma del teclado", "Al cambiar la distribución del teclado."],
  ];
  return (
    <Page title="Avisos" subtitle="Mensajes breves que el notch muestra sin abrirse.">
      {items.map(([id, label, hint]) => (
        <Row key={id} title={label} hint={hint}>
          <Toggle checked={h[id]} onChange={(v) => set({ hud: { [id]: v } })} />
        </Row>
      ))}
      <Row title="Duración de cada aviso">
        <Slider value={h.duration} min={800} max={6000} step={100} unit=" ms" onChange={(v) => set({ hud: { duration: v } })} />
      </Row>
      <div className="mt-4 text-xs leading-relaxed text-white/35">
        Windows seguirá mostrando su propio indicador de volumen y brillo junto al del notch.
      </div>
    </Page>
  );
}

const KEY_NAMES = { " ": "Space", ArrowUp: "Up", ArrowDown: "Down", ArrowLeft: "Left", ArrowRight: "Right", Escape: "Escape" };

function ShortcutRecorder({ value, onChange }) {
  const [recording, setRecording] = useState(false);

  useEffect(() => {
    if (!recording) return;
    const onKey = (e) => {
      e.preventDefault();
      if (["Control", "Shift", "Alt", "Meta"].includes(e.key)) return;
      if (e.key === "Escape") return setRecording(false);
      const mods = [e.ctrlKey && "Ctrl", e.altKey && "Alt", e.shiftKey && "Shift", e.metaKey && "Super"].filter(Boolean);
      if (!mods.length) return; // sin modificador chocaría con la escritura normal
      const raw = KEY_NAMES[e.key] ?? (e.key.length === 1 ? e.key.toUpperCase() : e.key);
      onChange([...mods, raw].join("+"));
      setRecording(false);
    };
    window.addEventListener("keydown", onKey, true);
    return () => window.removeEventListener("keydown", onKey, true);
  }, [recording, onChange]);

  return (
    <button
      onClick={() => setRecording((r) => !r)}
      onBlur={() => setRecording(false)}
      className={`min-w-40 rounded-lg border px-3 py-1.5 text-sm ${
        recording ? "animate-pulse border-sky-400 bg-sky-400/10 text-sky-200" : "border-white/10 bg-zinc-900 hover:border-white/30"
      }`}
    >
      {recording ? "Pulsa las teclas… (Esc cancela)" : value}
    </button>
  );
}

function Shortcuts({ s, set, warning }) {
  return (
    <Page title="Atajos" subtitle="Atajos globales: funcionan aunque otra app tenga el foco.">
      <Row title="Abrir / cerrar el notch" hint="Se cierra con el mismo atajo, con Esc o haciendo clic fuera.">
        <ShortcutRecorder value={s.shortcuts.toggle} onChange={(v) => set({ shortcuts: { toggle: v } })} />
      </Row>
      {warning && <div className="mt-4 rounded-lg bg-amber-500/10 p-3 text-sm text-amber-300">{warning}</div>}
    </Page>
  );
}

const PROVIDERS = {
  gemini: { name: "Gemini (Google)", key: true, baseHint: null, keyHint: "Consíguela gratis en Google AI Studio (aistudio.google.com)." },
  claude: { name: "Claude (Anthropic)", key: true, baseHint: null, keyHint: "Desde console.anthropic.com." },
  openai: {
    name: "OpenAI y compatibles",
    key: true,
    baseHint: "https://api.openai.com/v1",
    keyHint: "Sirve también para OpenRouter, Groq, LM Studio… cambiando la dirección base.",
  },
  ollama: { name: "Ollama (local)", key: false, baseHint: "http://localhost:11434", keyHint: null },
};

/**
 * La clave se escribe en un campo local y solo se envía al pulsar «Guardar» (así no se guardan claves a medias).
 * Rust la guarda en el Administrador de credenciales de Windows y a partir de ahí esta pantalla solo ve «guardada».
 */
function KeyField({ stored, onSave }) {
  const [draft, setDraft] = useState("");
  const [show, setShow] = useState(false);
  const [changing, setChanging] = useState(false);

  if (stored && !changing) {
    return (
      <div className="flex items-center gap-2">
        <span className="text-sm text-emerald-300">🔒 Guardada en Windows</span>
        <button className={button} onClick={() => setChanging(true)}>Cambiar</button>
        <button className={`${button} hover:bg-red-500/40`} onClick={() => onSave("")}>Borrar</button>
      </div>
    );
  }
  const submit = (e) => {
    e.preventDefault();
    if (!draft.trim()) return;
    onSave(draft.trim());
    setDraft("");
    setChanging(false);
  };
  return (
    <form onSubmit={submit} className="flex gap-2">
      <input
        type={show ? "text" : "password"}
        value={draft}
        onChange={(e) => setDraft(e.target.value)}
        placeholder="Pega aquí tu clave"
        className={input}
        autoComplete="off"
        spellCheck={false}
      />
      <button type="button" onClick={() => setShow((v) => !v)} className={button} aria-label="Mostrar u ocultar">
        {show ? "🙈" : "👁"}
      </button>
      <button className={`${button} ${draft.trim() ? "bg-emerald-500/30 hover:bg-emerald-500/50" : "opacity-50"}`} disabled={!draft.trim()}>Guardar</button>
      {changing && <button type="button" className={button} onClick={() => { setChanging(false); setDraft(""); }}>Cancelar</button>}
    </form>
  );
}

function Ai({ s, set }) {
  const provider = s.ai.provider;
  const info = PROVIDERS[provider];
  const cfg = s.ai[provider];
  const ai = (patch) => set({ ai: patch });
  const cfgPatch = (patch) => ai({ [provider]: patch });
  return (
    <Page title="Asistente IA" subtitle="Usa tu propia clave. Las consultas salen directo de tu PC al proveedor que elijas.">
      <Row title="Proveedor" hint="Lo usan el chat y el traductor.">
        <Select value={provider} onChange={(v) => ai({ provider: v })} options={Object.entries(PROVIDERS).map(([id, p]) => [id, p.name])} />
      </Row>
      {info.key && (
        <Row title="API key" hint={info.keyHint}>
          <KeyField key={provider} stored={cfg.apiKey !== ""} onSave={(k) => cfgPatch({ apiKey: k })} />
        </Row>
      )}
      <Row title="Modelo" hint={provider === "gemini" ? "Para respuestas rápidas usa un modelo «flash-lite» (por ejemplo gemini-3.5-flash-lite: ~2 s). Los «flash» grandes pueden tardar mucho más." : undefined}>
        <input value={cfg.model} onChange={(e) => cfgPatch({ model: e.target.value })} className={input} spellCheck={false} />
      </Row>
      {info.baseHint && (
        <Row title="Dirección del servidor" hint={`Déjalo vacío para usar ${info.baseHint}`}>
          <input
            value={cfg.baseUrl}
            onChange={(e) => cfgPatch({ baseUrl: e.target.value })}
            placeholder={info.baseHint}
            className={input}
            spellCheck={false}
          />
        </Row>
      )}
      <Row title="Instrucciones del asistente" hint="Se envían al inicio de cada conversación (no al traductor).">
        <textarea
          value={s.ai.systemPrompt}
          onChange={(e) => ai({ systemPrompt: e.target.value })}
          rows={3}
          className={`${input} resize-none`}
        />
      </Row>
      <div className="mt-4 text-xs leading-relaxed text-white/35">
        Las claves se guardan en el Administrador de credenciales de Windows: no aparecen en settings.json ni en las copias de seguridad
        (en otro equipo habrá que escribirlas de nuevo).
      </div>
    </Page>
  );
}

function CalendarSettings({ s, set }) {
  const c = s.calendar;
  const [test, setTest] = useState(null);
  const [busy, setBusy] = useState(false);
  const sources = c.sources;
  const setSources = (next) => set({ calendar: { sources: next } });

  const check = async () => {
    setBusy(true);
    setTest(null);
    // Se espera a que el guardado de los cambios llegue a Rust antes de descargar.
    await new Promise((r) => setTimeout(r, 400));
    try {
      const r = await invoke("calendar_fetch");
      setTest(r);
    } catch (e) {
      setTest({ events: [], errors: [String(e)] });
    } finally {
      setBusy(false);
    }
  };

  return (
    <Page title="Calendario" subtitle="Conecta calendarios con su dirección iCal (.ics). Solo lectura; se actualizan cada 15 minutos.">
      {sources.map((src, i) => (
        <div key={i} className="mb-3 flex gap-2">
          <input
            value={src.name}
            onChange={(e) => setSources(sources.map((x, j) => (j === i ? { ...x, name: e.target.value } : x)))}
            placeholder="Nombre"
            className={`${input} w-36`}
          />
          <input
            value={src.url}
            onChange={(e) => setSources(sources.map((x, j) => (j === i ? { ...x, url: e.target.value } : x)))}
            placeholder="https://… o webcal://…"
            className={`${input} min-w-0 flex-1`}
            spellCheck={false}
          />
          <button onClick={() => setSources(sources.filter((_, j) => j !== i))} className={`${button} hover:bg-red-500/40`} aria-label="Quitar">✕</button>
        </div>
      ))}
      <div className="flex gap-2">
        {sources.length < 5 && (
          <button onClick={() => setSources([...sources, { name: "", url: "" }])} className={button}>＋ Añadir calendario</button>
        )}
        {sources.some((x) => x.url) && (
          <button onClick={check} disabled={busy} className={`${button} disabled:opacity-50`}>
            {busy ? "Comprobando…" : "Probar conexión"}
          </button>
        )}
      </div>
      {test && (
        <div className={`mt-3 rounded-lg p-3 text-sm ${test.errors.length ? "bg-amber-500/10 text-amber-300" : "bg-emerald-500/10 text-emerald-300"}`}>
          {test.errors.length === 0 && `Conectado: ${test.events.length} evento(s) en los próximos 14 días.`}
          {test.errors.map((e) => (
            <div key={e}>⚠ {e}</div>
          ))}
          {test.errors.length > 0 && test.events.length > 0 && <div className="mt-1">Del resto: {test.events.length} evento(s).</div>}
        </div>
      )}
      <Row title="Aviso antes de cada reunión" hint="El notch te avisa con sonido. 0 = sin aviso.">
        <Slider value={c.reminderMin} min={0} max={60} step={5} unit=" min" onChange={(v) => set({ calendar: { reminderMin: v } })} />
      </Row>
      <div className="mt-4 space-y-2 text-xs leading-relaxed text-white/35">
        <div>
          <b className="text-white/50">Google Calendar:</b> Configuración → tu calendario → «Integrar calendario» → copia la «Dirección secreta en formato iCal».
        </div>
        <div>
          <b className="text-white/50">Outlook:</b> Configuración → Calendario → Calendarios compartidos → Publicar un calendario → copia el enlace ICS.
        </div>
        <div>Cualquiera que tenga la dirección puede ver tu calendario: no la compartas. Las horas con zona horaria se leen en la zona de este equipo.</div>
      </div>
    </Page>
  );
}

function WeatherSettings({ s, set }) {
  const w = s.weather;
  const [q, setQ] = useState("");
  const [results, setResults] = useState(null);
  const [busy, setBusy] = useState(false);
  const [detect, setDetect] = useState(null); // null | "busy" | texto de error

  const detectCity = async () => {
    setDetect("busy");
    try {
      const c = await invoke("detect_city");
      set({ weather: { city: c.label, lat: c.lat, lon: c.lon } });
      setDetect(null);
    } catch (e) {
      setDetect(String(e));
    }
  };

  const search = async (e) => {
    e.preventDefault();
    if (!q.trim()) return;
    setBusy(true);
    try {
      const res = await fetch(
        `https://geocoding-api.open-meteo.com/v1/search?${new URLSearchParams({ name: q.trim(), count: 6, language: "es" })}`
      );
      setResults((await res.json()).results ?? []);
    } catch {
      setResults([]);
    } finally {
      setBusy(false);
    }
  };

  return (
    <Page title="Clima" subtitle="Datos de Open-Meteo (gratuito, sin clave).">
      <Row title="Ciudad actual">
        <span className="text-sm text-white/70">{w.city || "Sin elegir"}</span>
      </Row>
      <form onSubmit={search} className="mt-3 flex gap-2">
        <input value={q} onChange={(e) => setQ(e.target.value)} placeholder="Buscar ciudad…" className={`${input} flex-1`} />
        <button className={button} disabled={busy}>{busy ? "Buscando…" : "Buscar"}</button>
        <button type="button" className={button} onClick={detectCity} disabled={detect === "busy"} title="Usa tu dirección IP para estimar la ciudad (se consulta ipwho.is solo al pulsar)">
          {detect === "busy" ? "Detectando…" : "📍 Detectar"}
        </button>
      </form>
      {detect && detect !== "busy" && <div className="mt-2 text-xs text-amber-300">{detect}</div>}
      {results && (
        <ul className="mt-3 flex flex-col gap-1">
          {results.length === 0 && <li className="text-sm text-white/40">Sin resultados.</li>}
          {results.map((r) => {
            const label = [r.name, r.admin1, r.country].filter(Boolean).join(", ");
            return (
              <li key={r.id}>
                <button
                  onClick={() => {
                    set({ weather: { city: label, lat: r.latitude, lon: r.longitude } });
                    setResults(null);
                    setQ("");
                  }}
                  className="w-full rounded-lg bg-white/5 px-3 py-2 text-left text-sm transition-colors hover:bg-white/10"
                >
                  {label}
                  <span className="ml-2 text-xs text-white/35">{r.latitude.toFixed(2)}, {r.longitude.toFixed(2)}</span>
                </button>
              </li>
            );
          })}
        </ul>
      )}
      <Row title="Unidades">
        <Select
          value={w.units}
          onChange={(v) => set({ weather: { units: v } })}
          options={[["c", "°C · km/h"], ["f", "°F · mph"]]}
        />
      </Row>
    </Page>
  );
}

const mb = (b) => `${(b / 1024 / 1024).toFixed(1)} MB`;

/** Búsqueda e instalación de actualizaciones. La firma del instalador la verifica Rust antes de ejecutarlo. */
function UpdatesBlock({ s, set, version }) {
  const u = s.updates;
  const [state, setState] = useState({ phase: "idle" }); // idle | checking | uptodate | available | installing | error
  const [url, setUrl] = useState(u.url);

  useEffect(() => setUrl(u.url), [u.url]);
  useEffect(() => {
    let unlisten;
    let cancelled = false;
    listen("update-progress", (e) => setState((st) => (st.phase === "installing" ? { ...st, ...e.payload } : st))).then((fn) =>
      cancelled ? fn() : (unlisten = fn)
    );
    // Si la comprobación automática ya encontró algo, se muestra al abrir.
    listen("update-available", (e) => setState({ phase: "available", info: e.payload })).then((fn) => (cancelled ? fn() : (unlisten = fn)));
    return () => {
      cancelled = true;
      unlisten?.();
    };
  }, []);

  const check = async () => {
    setState({ phase: "checking" });
    // Se espera a que una dirección recién escrita llegue a Rust.
    await new Promise((r) => setTimeout(r, 450));
    try {
      const info = await invoke("update_check");
      setState(info ? { phase: "available", info } : { phase: "uptodate" });
    } catch (e) {
      setState({ phase: "error", message: String(e) });
    }
  };

  const install = async () => {
    setState((st) => ({ ...st, phase: "installing", downloaded: 0, total: null }));
    try {
      await invoke("update_install"); // si todo va bien, la app se cierra y se reabre sola
    } catch (e) {
      setState({ phase: "error", message: String(e) });
    }
  };

  const saveUrl = () => url.trim() !== u.url && set({ updates: { url: url.trim() } });

  return (
    <>
      <Row
        title="Actualizaciones"
        hint={
          state.phase === "available" || state.phase === "installing"
            ? `Hay una versión nueva: ${state.info?.version} (tienes la ${version}).`
            : state.phase === "uptodate"
              ? "Estás al día."
              : state.phase === "error"
                ? state.message
                : "Busca versiones nuevas en la dirección de abajo."
        }
      >
        <div className="flex items-center gap-2">
          {state.phase === "installing" ? (
            <span className="text-xs tabular-nums text-sky-300">
              {state.total ? `Descargando ${mb(state.downloaded)} / ${mb(state.total)}` : "Descargando…"}
            </span>
          ) : state.phase === "available" ? (
            <button className={`${button} bg-emerald-500/30 hover:bg-emerald-500/50`} onClick={install}>Instalar y reiniciar</button>
          ) : (
            <button className={button} onClick={check} disabled={state.phase === "checking"}>
              {state.phase === "checking" ? "Buscando…" : "Buscar ahora"}
            </button>
          )}
        </div>
      </Row>
      {state.phase === "available" && state.info?.notes && (
        <div className="mt-2 whitespace-pre-wrap rounded-lg bg-white/5 p-3 text-xs leading-relaxed text-white/60">{state.info.notes}</div>
      )}
      <Row
        title="Dirección de las actualizaciones"
        hint="Dirección HTTPS del archivo latest.json que publicas con cada versión (por ejemplo en GitHub Releases). Cada instalador va firmado y la app solo instala los que coinciden con su clave."
      >
        <input
          value={url}
          onChange={(e) => setUrl(e.target.value)}
          onBlur={saveUrl}
          onKeyDown={(e) => e.key === "Enter" && e.currentTarget.blur()}
          placeholder="https://…/latest.json"
          className={`${input} w-80`}
          spellCheck={false}
        />
      </Row>
      <Row title="Buscar automáticamente" hint="Comprueba a los pocos segundos de abrir y cada 6 horas; si hay algo nuevo te avisa en el notch (nunca instala sin que lo pidas).">
        <Toggle checked={u.auto} onChange={(v) => set({ updates: { auto: v } })} />
      </Row>
    </>
  );
}

function About({ reset, s, set }) {
  const [version, setVersion] = useState("");
  const [dir, setDir] = useState("");
  const [confirm, setConfirm] = useState(false);
  const [backupMsg, setBackupMsg] = useState(null);

  const exportBackup = async () => {
    try {
      const d = new Date();
      const stamp = `${d.getFullYear()}-${String(d.getMonth() + 1).padStart(2, "0")}-${String(d.getDate()).padStart(2, "0")}`;
      const path = await save({ title: "Guardar copia de seguridad", defaultPath: `Noki-copia-${stamp}.json`, filters: [{ name: "Copia de Noki", extensions: ["json"] }] });
      if (!path) return;
      await invoke("backup_export", { path });
      setBackupMsg({ ok: true, text: `Copia guardada en ${path}` });
    } catch (e) {
      setBackupMsg({ ok: false, text: String(e) });
    }
  };

  const importBackup = async () => {
    try {
      const path = await open({ title: "Elegir copia de seguridad", multiple: false, filters: [{ name: "Copia de Noki", extensions: ["json"] }] });
      if (!path) return;
      if (!window.confirm("Se reemplazarán tus ajustes y datos actuales por los de la copia y Noki se reiniciará. ¿Continuar?")) return;
      await invoke("backup_import", { path });
    } catch (e) {
      setBackupMsg({ ok: false, text: String(e) });
    }
  };

  useEffect(() => {
    getVersion().then(setVersion).catch(() => {});
    invoke("data_dir").then((d) => setDir(d ?? "")).catch(() => {});
  }, []);
  return (
    <Page title="Acerca de Noki">
      <Row title="Versión">
        <span className="text-sm text-white/60">{version}</span>
      </Row>
      <Row title="Licencia" hint="Noki es software libre: puedes usarlo, estudiarlo y mejorarlo.">
        <span className="text-sm text-white/60">GPL-3.0</span>
      </Row>
      <Row title="Carpeta de datos" hint="Aquí se guardan tus ajustes. Cópiala para llevarlos a otro equipo.">
        <span className="select-text break-all text-xs text-white/50">{dir}</span>
      </Row>
      <UpdatesBlock s={s} set={set} version={version} />
      <Row
        title="Copia de seguridad"
        hint="Guarda ajustes y datos (tareas, notas, agua, chat…) en un archivo, o restáuralos en otro equipo. No incluye las claves de API."
      >
        <div className="flex gap-2">
          <button className={button} onClick={exportBackup}>Exportar…</button>
          <button className={button} onClick={importBackup}>Importar…</button>
        </div>
      </Row>
      {backupMsg && <div className={`mt-2 rounded-lg p-3 text-sm ${backupMsg.ok ? "bg-emerald-500/10 text-emerald-300" : "bg-amber-500/10 text-amber-300"}`}>{backupMsg.text}</div>}
      <Row title="Restablecer ajustes" hint="Vuelve todo a los valores de fábrica (incluida la clave de IA).">
        {confirm ? (
          <div className="flex gap-2">
            <button className={`${button} bg-red-500/30 hover:bg-red-500/50`} onClick={() => { reset(); setConfirm(false); }}>
              Confirmar
            </button>
            <button className={button} onClick={() => setConfirm(false)}>Cancelar</button>
          </div>
        ) : (
          <button className={button} onClick={() => setConfirm(true)}>Restablecer…</button>
        )}
      </Row>
      <Row title="Cerrar Noki" hint="Libera el espacio reservado y cierra la aplicación.">
        <button className={`${button} bg-red-500/20 hover:bg-red-500/40`} onClick={() => invoke("quit_app")}>
          Salir
        </button>
      </Row>
    </Page>
  );
}

// ───────────── desarrollador ─────────────

const glyph = { fill: "none", stroke: "currentColor", strokeWidth: 2, strokeLinecap: "round", strokeLinejoin: "round" };
const SOCIAL_ICONS = {
  instagram: (
    <svg viewBox="0 0 24 24" width="22" height="22" {...glyph} aria-hidden="true">
      <rect x="3" y="3" width="18" height="18" rx="5" />
      <circle cx="12" cy="12" r="4" />
      <circle cx="17.3" cy="6.7" r="0.6" fill="currentColor" />
    </svg>
  ),
  facebook: (
    <svg viewBox="0 0 24 24" width="22" height="22" {...glyph} aria-hidden="true">
      <path d="M14 8h3V4h-3a4 4 0 0 0-4 4v3H7v4h3v6h4v-6h3l1-4h-4V8.6c0-.4.2-.6.6-.6Z" />
    </svg>
  ),
  x: (
    <svg viewBox="0 0 24 24" width="22" height="22" {...glyph} aria-hidden="true">
      <path d="M4 4l16 16" strokeWidth="2.6" />
      <path d="M20 4 13.5 10.8M10.5 13.7 4 20" />
    </svg>
  ),
  kofi: (
    <svg viewBox="0 0 24 24" width="22" height="22" {...glyph} aria-hidden="true">
      <path d="M4 8h13v6a5 5 0 0 1-5 5H9a5 5 0 0 1-5-5V8Z" />
      <path d="M17 9.5h1.5a2.5 2.5 0 0 1 0 5H17" />
      <path d="M10.5 15.5s-2-1.2-2-2.6a1.2 1.2 0 0 1 2-.8 1.2 1.2 0 0 1 2 .8c0 1.4-2 2.6-2 2.6Z" fill="currentColor" stroke="none" />
    </svg>
  ),
};

function SocialButton({ id, label, url }) {
  return (
    <button
      onClick={() => invoke("open_url", { url }).catch(() => {})}
      title={label}
      aria-label={label}
      className="flex h-11 w-11 items-center justify-center rounded-full bg-white/10 text-white/80 transition-all duration-200 hover:scale-110 hover:bg-white/20 hover:text-white active:scale-95"
    >
      {SOCIAL_ICONS[id]}
    </button>
  );
}

function Developer({ icon }) {
  return (
    <Page title="Desarrollador" subtitle="La persona detrás de Noki.">
      <div className="mt-2 flex items-center gap-5 rounded-2xl bg-white/5 p-5">
        <NokiLogo size={72} variant={icon} />
        <div className="min-w-0">
          <div className="text-lg font-semibold">{DEV.name}</div>
          <div className="mt-0.5 text-sm text-white/55">Creador de Noki</div>
          <div className="mt-3 flex items-center gap-2">
            {DEV.links.map((l) => (
              <SocialButton key={l.id} {...l} />
            ))}
          </div>
        </div>
      </div>

      <Row title="Apoya el proyecto" hint="Noki es gratis y de código abierto. Si te gusta, un café ayuda a seguir mejorándolo.">
        <button
          onClick={() => invoke("open_url", { url: DEV.kofi }).catch(() => {})}
          className="flex items-center gap-2 rounded-full bg-[#ff5e5b] px-4 py-2 text-sm font-medium text-white transition-all duration-200 hover:scale-105 hover:bg-[#ff7673] active:scale-95"
        >
          {SOCIAL_ICONS.kofi}
          Invítame un café
        </button>
      </Row>
      {DEV.repo && (
        <Row title="Código fuente" hint="Reporta errores, propón mejoras o échale un vistazo al código.">
          <button className={button} onClick={() => invoke("open_url", { url: DEV.repo }).catch(() => {})}>
            Ver en GitHub
          </button>
        </Row>
      )}
      <div className="mt-6 space-y-2 text-xs leading-relaxed text-white/40">
        <p>Inspirado en el notch de los Mac y en las apps de «notch» para escritorio: gracias a esa idea me animé a hacer mi propia versión.</p>
        <p>Hecho por Cristhian Bustamante con ayuda de inteligencia artificial (Claude Code).</p>
      </div>
    </Page>
  );
}

// ───────────── ventana ─────────────

export default function SettingsWindow() {
  const { settings, update, reset, warning } = useSettings();
  presetLanguage(settings?.general.language);
  useLanguage(settings?.general.language);
  const [section, setSection] = useState(() => new URLSearchParams(window.location.search).get("section") || "general");
  const [monitors, setMonitors] = useState([]);

  useEffect(() => {
    invoke("list_monitors").then(setMonitors).catch(() => {});
    let unlisten;
    let cancelled = false;
    listen("settings-section", (e) => setSection(e.payload)).then((fn) => (cancelled ? fn() : (unlisten = fn)));
    return () => {
      cancelled = true;
      unlisten?.();
    };
  }, []);

  if (!settings) return null;

  return (
    <div className="flex h-full select-none">
      <aside className="flex w-52 shrink-0 flex-col gap-0.5 overflow-y-auto border-r border-white/5 bg-zinc-950 p-3">
        <div className="mb-2 flex items-center gap-2.5 px-3 pt-1 text-sm font-semibold tracking-wide text-white/80">
          <NokiLogo size={26} variant={settings.appearance.appIcon} />
          Noki
        </div>
        {SECTIONS.map((x) => (
          <button
            key={x.id}
            onClick={() => setSection(x.id)}
            className={`flex shrink-0 items-center gap-3 rounded-lg px-3 py-1.5 text-left text-sm transition-colors ${
              section === x.id ? "bg-white/10 text-white" : "text-white/55 hover:bg-white/5 hover:text-white"
            }`}
          >
            <span className="w-5 text-center">{x.icon}</span>
            {x.label}
          </button>
        ))}
      </aside>
      <main className="min-w-0 flex-1 overflow-y-auto px-8 py-6">
        {section === "general" && <General s={settings} set={update} monitors={monitors} />}
        {section === "appearance" && <Appearance s={settings} set={update} />}
        {section === "modules" && <Modules s={settings} set={update} />}
        {section === "productivity" && <Productivity s={settings} set={update} />}
        {section === "tools" && <ToolsSettings s={settings} set={update} />}
        {section === "alerts" && <Alerts s={settings} set={update} />}
        {section === "shortcuts" && <Shortcuts s={settings} set={update} warning={warning} />}
        {section === "calendar" && <CalendarSettings s={settings} set={update} />}
        {section === "weather" && <WeatherSettings s={settings} set={update} />}
        {section === "ai" && <Ai s={settings} set={update} />}
        {section === "developer" && <Developer icon={settings.appearance.appIcon} />}
        {section === "about" && <About reset={reset} s={settings} set={update} />}
        {warning && section !== "shortcuts" && (
          <div className="mt-4 rounded-lg bg-amber-500/10 p-3 text-sm text-amber-300">{warning}</div>
        )}
      </main>
    </div>
  );
}
