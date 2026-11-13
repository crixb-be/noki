use crate::secrets;
use serde::{Deserialize, Serialize};
use std::{fs, path::PathBuf};
use tauri::{AppHandle, Manager};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(default, rename_all = "camelCase")]
pub struct General {
    /// Notch visible (se puede pausar desde la bandeja).
    pub visible: bool,
    pub autostart: bool,
    /// Registra el notch como AppBar para que las demás ventanas trabajen debajo.
    pub reserve_space: bool,
    /// "notch" = hay que tocar la píldora; "edge" = basta con llegar al borde superior.
    pub trigger: String,
    pub hover_delay: u32,
    pub close_delay: u32,
    pub hide_on_fullscreen: bool,
    /// Índice de monitor; -1 = principal.
    pub monitor: i32,
    /// Un notch en cada pantalla conectada (si no, solo en la elegida).
    pub all_monitors: bool,
    /// No molestar: silencia alertas y recordatorios (no los indicadores de volumen o brillo).
    pub dnd: bool,
    /// Idioma de la interfaz: "es" o "en".
    pub language: String,
    /// Nombre con el que Noki te saluda al arrancar (vacío = saludo sin nombre).
    pub user_name: String,
    /// Saludo de bienvenida (según la hora) al iniciar Noki.
    pub welcome_greeting: bool,
}

impl Default for General {
    fn default() -> Self {
        Self {
            visible: true,
            autostart: false,
            // Se conserva solo para leer configuraciones antiguas; la reserva nueva la hace `topbar`.
            reserve_space: false,
            trigger: "notch".into(),
            hover_delay: 120,
            close_delay: 300,
            hide_on_fullscreen: true,
            monitor: -1,
            all_monitors: false,
            dnd: false,
            language: "es".into(),
            user_name: String::new(),
            welcome_greeting: true,
        }
    }
}

/// Una superficie de vidrio (Dock, barra superior, paneles o lanzador).
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(default, rename_all = "camelCase")]
pub struct GlassSurface {
    /// Opacidad del tinte en porcentaje (0-100).
    pub tint_opacity: i32,
    /// Intensidad del desenfoque: "low" | "medium" | "high".
    pub blur: String,
    /// Radio de las esquinas en px lógicos.
    pub radius: i32,
    /// Borde brillante de 1 px.
    pub rim: bool,
    /// Sombra suave bajo la forma.
    pub shadow: bool,
}

impl GlassSurface {
    pub fn macdock(radius: i32) -> Self { Self { tint_opacity: 14, blur: "medium".into(), radius, rim: true, shadow: true } }
    fn clamp(&mut self) {
        self.tint_opacity = self.tint_opacity.clamp(0, 100);
        self.radius = self.radius.clamp(0, 40);
        if !matches!(self.blur.as_str(), "low" | "medium" | "high") { self.blur = "medium".into(); }
    }
}

impl Default for GlassSurface { fn default() -> Self { Self::macdock(18) } }

/// «Vidrio y colores» (Ajustes > Apariencia). Por defecto, los valores de MacDock.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(default, rename_all = "camelCase")]
pub struct Glass {
    /// "light" (día) | "dark" (noche) | "auto" (sigue a Windows).
    pub mode: String,
    /// "native": desenfoque acrílico de Windows 11 (una ventana continua por superficie, esquinas de
    /// DWM). "css": «Vidrio suave sin blur nativo», solo fondo CSS translúcido (cero artefactos de DWM).
    pub engine: String,
    pub dock: GlassSurface,
    pub bar: GlassSurface,
    pub panels: GlassSurface,
    pub launcher: GlassSurface,
}

impl Default for Glass {
    fn default() -> Self {
        Self { mode: "light".into(), engine: "native".into(), dock: GlassSurface::macdock(20), bar: GlassSurface::macdock(0), panels: GlassSurface::macdock(18), launcher: GlassSurface::macdock(24) }
    }
}

impl Glass {
    pub fn clamp(&mut self) {
        if !matches!(self.mode.as_str(), "light" | "dark" | "auto") { self.mode = "light".into(); }
        if !matches!(self.engine.as_str(), "native" | "css") { self.engine = "native".into(); }
        for s in [&mut self.dock, &mut self.bar, &mut self.panels, &mut self.launcher] { s.clamp(); }
    }
}
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(default, rename_all = "camelCase")]
pub struct Appearance {
    pub collapsed_w: i32,
    pub collapsed_h: i32,
    pub expanded_w: i32,
    pub expanded_h: i32,
    pub offset_x: i32,
    /// Opacidad del fondo en porcentaje.
    pub opacity: i32,
    pub show_clock: bool,
    pub show_battery: bool,
    /// Qué más se ve en el notch cerrado: fecha, tareas pendientes, temperatura y actividad (música, Pomodoro, reunión).
    pub show_date: bool,
    pub show_tasks: bool,
    /// Hábitos completados hoy (🎯 2/5) en el notch cerrado.
    pub show_habits: bool,
    pub show_weather: bool,
    pub show_activity: bool,
    /// El notch toma el color del programa que hay debajo (en lugar del color o la imagen elegidos).
    pub notch_auto: bool,
    /// Reloj de 24 horas (apagado: 12 horas con a. m./p. m.).
    pub clock24: bool,
    /// Barras del visualizador reaccionando al audio real del sistema.
    pub visualizer: bool,
    /// Píldora multimedia: con música sonando el notch se ensancha con carátula y ecualizador, y al pasar el cursor muestra controles.
    pub media_pill: bool,
    /// Color del notch (#rrggbb). La opacidad sale de `opacity`.
    pub notch_color: String,
    /// Imagen de fondo del notch (nombre de archivo dentro de la carpeta `backgrounds`); vacío = ninguna.
    pub notch_image: String,
    /// Cuánto se oscurece la imagen del notch para que el texto se lea (0-85 %).
    pub notch_dim: i32,
    /// Franja de los costados del notch: "none" (transparente), "match" (como el notch), "auto" (color del programa abierto), "color" o "image".
    pub bar_mode: String,
    pub bar_color: String,
    pub bar_image: String,
    pub bar_dim: i32,
    pub bar_opacity: i32,
    /// Icono de Noki (bandeja y ventana de Ajustes): "oceano" (oficial), "atardecer", "aurora", "pastel-atardecer" o "pastel-aurora".
    pub app_icon: String,
    /// La carita animada de Noki dentro del notch.
    pub mascot: bool,
    /// Tema del personaje, del notch central y de la franja de los costados, cada uno por separado:
    /// "auto" (según la fecha), "none" (tus ajustes), "halloween", "christmas", "birthday" o "love".
    pub mascot_theme: String,
    pub notch_theme: String,
    pub bar_theme: String,
    /// Opacidad (%) del color del tema en el notch central y en la franja de los costados.
    pub notch_theme_opacity: i32,
    pub bar_theme_opacity: i32,
    /// Estilo de los paneles (Launchpad, paneles de la barra, buscador): «light» = vidrio claro difuminado (macOS), «dark» = oscuro opaco.
    pub panel_style: String,
    /// Vidrio y colores (modo y ajustes por superficie).
    pub glass: Glass,
    /// Cumpleaños de quien usa Noki, "MM-DD" (vacío si no se indica). Solo se usa en este equipo.
    pub birthday: String,
    /// Fechas propias que activan un tema (p. ej. un aniversario) además de las de siempre.
    pub theme_events: Vec<ThemeEvent>,
    /// Año de nacimiento (0 = no indicado); solo sirve para decir tu edad el día del cumpleaños.
    pub birth_year: i32,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(default, rename_all = "camelCase")]
pub struct TopBar {
    pub enabled: bool,
    pub height: i32,
    pub tint: String,
    pub show_cpu: bool,
    pub show_ram: bool,
    pub show_wifi: bool,
    pub show_bluetooth: bool,
    pub show_sound: bool,
    pub show_battery: bool,
    pub show_battery_pct: bool,
    pub show_search: bool,
    pub show_control_center: bool,
    pub show_capture: bool,
    pub show_notifications: bool,
    pub show_avatar: bool,
    pub show_clock: bool,
    pub avatar_path: String,
}

impl Default for TopBar {
    fn default() -> Self {
        Self {
            enabled: false,
            height: 30,
            tint: "auto".into(),
            show_cpu: false,
            show_ram: false,
            show_wifi: true,
            show_bluetooth: true,
            show_sound: true,
            show_battery: true,
            show_battery_pct: true,
            show_search: true,
            show_control_center: true,
            show_capture: true,
            show_notifications: true,
            show_avatar: true,
            show_clock: true,
            avatar_path: String::new(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq)]
#[serde(default, rename_all = "camelCase")]
pub struct DockPinned {
    pub name: String,
    pub path: String,
    pub args: String,
    pub aumid: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq)]
#[serde(default, rename_all = "camelCase")]
pub struct DockFolder {
    pub name: String,
    pub path: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(default, rename_all = "camelCase")]
pub struct Dock {
    pub enabled: bool,
    pub icon_size: i32,
    pub magnify: bool,
    pub magnify_scale: f64,
    pub show_apps: bool,
    pub show_running: bool,
    pub show_trash: bool,
    pub autohide: bool,
    #[serde(default)]
    pub hide_taskbar: bool,
    pub reserve_space: bool,
    pub pinned: Vec<DockPinned>,
    pub folders: Vec<DockFolder>,
}

impl Default for Dock {
    fn default() -> Self {
        let mut pinned = vec![DockPinned {
            name: "Explorador".into(),
            path: "explorer.exe".into(),
            args: String::new(),
            aumid: String::new(),
        }];
        let edge = std::env::var("ProgramFiles(x86)")
            .ok()
            .map(|p| std::path::PathBuf::from(p).join("Microsoft\\Edge\\Application\\msedge.exe"))
            .filter(|p| p.exists());
        if let Some(path) = edge {
            pinned.push(DockPinned { name: "Microsoft Edge".into(), path: path.to_string_lossy().into_owned(), args: String::new(), aumid: String::new() });
        }
        pinned.push(DockPinned { name: "Bloc de notas".into(), path: "notepad.exe".into(), args: String::new(), aumid: String::new() });
        let downloads = std::env::var("USERPROFILE").map(|p| std::path::PathBuf::from(p).join("Downloads")).unwrap_or_default();
        Self {
            enabled: false,
            icon_size: 40,
            magnify: true,
            magnify_scale: 1.6,
            show_apps: true,
            show_running: true,
            show_trash: true,
            autohide: false,
            hide_taskbar: false,
            reserve_space: false,
            pinned,
            folders: vec![DockFolder { name: "Descargas".into(), path: downloads.to_string_lossy().into_owned() }],
        }
    }
}

impl Default for Appearance {
    fn default() -> Self {
        Self {
            collapsed_w: 240,
            collapsed_h: 34,
            expanded_w: 540,
            expanded_h: 320,
            offset_x: 0,
            opacity: 100,
            show_clock: true,
            show_battery: true,
            show_date: false,
            show_tasks: false,
            show_habits: false,
            show_weather: false,
            show_activity: true,
            notch_auto: false,
            clock24: true,
            visualizer: true,
            media_pill: true,
            notch_color: "#000000".into(),
            notch_image: String::new(),
            notch_dim: 35,
            bar_mode: "match".into(),
            bar_color: "#000000".into(),
            bar_image: String::new(),
            bar_dim: 35,
            bar_opacity: 100,
            app_icon: crate::appicon::DEFAULT.into(),
            mascot: true,
            mascot_theme: "auto".into(),
            notch_theme: "auto".into(),
            bar_theme: "auto".into(),
            notch_theme_opacity: 100,
            bar_theme_opacity: 90,
            panel_style: "light".into(),
            glass: Glass::default(),
            birthday: String::new(),
            birth_year: 0,
            theme_events: Vec::new(),
        }
    }
}

/// Un tema que se activa solo entre dos fechas ("MM-DD"; si `from` > `to` cruza el fin de año).
#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq)]
#[serde(default, rename_all = "camelCase")]
pub struct ThemeEvent {
    pub name: String,
    pub from: String,
    pub to: String,
    pub theme: String,
}

fn valid_md(v: &str) -> bool {
    v.split_once('-').is_some_and(|(m, d)| {
        m.len() == 2 && d.len() == 2 && matches!(m.parse::<u32>(), Ok(1..=12)) && matches!(d.parse::<u32>(), Ok(1..=31))
    })
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(default, rename_all = "camelCase")]
pub struct Modules {
    /// Interruptores rápidos, clima y favoritos.
    pub panel: bool,
    pub media: bool,
    pub work: bool,
    pub files: bool,
    pub ai: bool,
    pub calendar: bool,
    pub weather: bool,
    /// Botón de letras sincronizadas en el reproductor.
    pub lyrics: bool,
    /// Portapapeles y lanzador de apps.
    pub tools: bool,
    /// CPU, memoria, discos, red y programas que más consumen.
    pub system: bool,
    /// Tiempo de pantalla: qué aplicación usas y cuánto (solo en este equipo). Apagado por defecto.
    pub screentime: bool,
    /// Hábitos con recordatorios (agua, pastillas…).
    pub habits: bool,
    /// "last" recuerda la última pestaña; o el id de un módulo.
    pub default_tab: String,
    /// Orden de las pestañas del notch (ids de módulo).
    pub order: Vec<String>,
}

/// Pestañas del notch en su orden original.
pub const TAB_IDS: [&str; 10] = [
    "panel", "media", "work", "habits", "files", "calendar", "ai", "system", "screentime", "tools",
];

impl Default for Modules {
    fn default() -> Self {
        Self {
            panel: true,
            media: true,
            work: true,
            files: true,
            ai: true,
            calendar: true,
            weather: true,
            lyrics: true,
            tools: true,
            system: true,
            screentime: false,
            habits: true,
            default_tab: "last".into(),
            order: TAB_IDS.iter().map(|s| s.to_string()).collect(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(default, rename_all = "camelCase")]
pub struct Shortcuts {
    /// Abrir / cerrar el notch.
    pub toggle: String,
    pub toggle_enabled: bool,
    /// Empezar / detener una grabación de voz.
    pub record: String,
    pub record_enabled: bool,
    /// Activar / desactivar «No molestar».
    pub dnd: String,
    pub dnd_enabled: bool,
    /// Abrir el notch listo para escribir una tarea.
    pub add_task: String,
    pub add_task_enabled: bool,
    /// Abrir el notch directamente en el historial del portapapeles.
    pub clip: String,
    pub clip_enabled: bool,
    /// Campo flotante para preguntar a la IA sin abrir todo el panel.
    pub ai: String,
    pub ai_enabled: bool,
    /// Ir directo a una pestaña con <modificador>+1…9.
    pub tabs_enabled: bool,
    pub tabs_modifier: String,
}

impl Default for Shortcuts {
    fn default() -> Self {
        Self {
            toggle: "Alt+N".into(),
            toggle_enabled: true,
            record: "Alt+R".into(),
            record_enabled: true,
            dnd: "Alt+D".into(),
            dnd_enabled: false,
            add_task: "Alt+T".into(),
            add_task_enabled: false,
            clip: "Ctrl+Alt+V".into(),
            clip_enabled: true,
            ai: "Ctrl+Alt+A".into(),
            ai_enabled: true,
            tabs_enabled: false,
            tabs_modifier: "Alt".into(),
        }
    }
}

pub const TAB_MODIFIERS: [&str; 5] = ["Alt", "Ctrl", "Ctrl+Alt", "Ctrl+Shift", "Alt+Shift"];

/// Configuración de un proveedor de IA. `base_url` vacío = el servidor oficial.
#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq)]
#[serde(default, rename_all = "camelCase")]
pub struct ProviderCfg {
    pub api_key: String,
    pub model: String,
    pub base_url: String,
}

fn provider(model: &str, base_url: &str) -> ProviderCfg {
    ProviderCfg { api_key: String::new(), model: model.into(), base_url: base_url.into() }
}

/// Modelos de la versión 0.3 y anteriores (solo Gemini y Claude con una sola clave).
#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq)]
#[serde(default)]
pub struct LegacyModels {
    pub gemini: String,
    pub claude: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(default, rename_all = "camelCase")]
pub struct Ai {
    /// "gemini" | "claude" | "openai" (compatible con OpenAI) | "ollama" (local).
    pub provider: String,
    pub system_prompt: String,
    pub gemini: ProviderCfg,
    pub claude: ProviderCfg,
    pub openai: ProviderCfg,
    pub ollama: ProviderCfg,
    // Campos de ≤0.3: se migran a su proveedor en `Settings::clamp` y dejan de guardarse.
    #[serde(skip_serializing_if = "String::is_empty")]
    pub api_key: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub models: Option<LegacyModels>,
}

const OLD_PROMPT: &str = "Responde en español, de forma breve y clara.";
/// Pensado para un chat de consultas rápidas: respuestas cortas y directas.
const QUICK_PROMPT: &str = "Responde en español, directo y breve (1 a 3 frases), sin rodeos ni introducciones. Amplía solo si te lo piden.";

impl Default for Ai {
    fn default() -> Self {
        Self {
            provider: "gemini".into(),
            system_prompt: QUICK_PROMPT.into(),
            gemini: provider("gemini-3.5-flash-lite", ""),
            claude: provider("claude-haiku-5-5", ""),
            openai: provider("gpt-4o-mini", ""),
            ollama: provider("llama3.2", ""),
            api_key: String::new(),
            models: None,
        }
    }
}

impl Ai {
    /// Configuración del proveedor activo.
    pub fn active(&self) -> &ProviderCfg {
        match self.provider.as_str() {
            "claude" => &self.claude,
            "openai" => &self.openai,
            "ollama" => &self.ollama,
            _ => &self.gemini,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq)]
#[serde(default, rename_all = "camelCase")]
pub struct CalSource {
    pub name: String,
    /// Dirección iCal (.ics): la «dirección secreta» de Google Calendar, el enlace publicado de Outlook, etc.
    pub url: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(default, rename_all = "camelCase")]
pub struct Calendar {
    pub sources: Vec<CalSource>,
    /// Minutos de antelación del aviso de reunión (0 = sin aviso).
    pub reminder_min: u32,
}

impl Default for Calendar {
    fn default() -> Self {
        Self { sources: vec![], reminder_min: 10 }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(default, rename_all = "camelCase")]
pub struct Weather {
    pub city: String,
    pub lat: f64,
    pub lon: f64,
    /// "c" o "f".
    pub units: String,
}

impl Default for Weather {
    fn default() -> Self {
        Self { city: String::new(), lat: 0.0, lon: 0.0, units: "c".into() }
    }
}

/// Avisos temporales (HUD y "peeks") que aparecen en el notch cerrado.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(default, rename_all = "camelCase")]
pub struct Hud {
    pub volume: bool,
    pub brightness: bool,
    pub charger: bool,
    /// Aviso al bajar al 20 % y al 10 % sin cargador.
    pub low_battery: bool,
    pub network: bool,
    /// Cambios de red Wi-Fi (conectado a una red / desconectado).
    pub wifi: bool,
    /// Dispositivos Bluetooth que se conectan o desconectan.
    pub bluetooth: bool,
    /// Aviso (y punto rojo en el notch) cuando una app usa el micrófono.
    pub microphone: bool,
    pub audio_device: bool,
    pub caps_lock: bool,
    pub language: bool,
    /// Duración de cada aviso en milisegundos.
    pub duration: u32,
}

impl Default for Hud {
    fn default() -> Self {
        Self {
            volume: true,
            brightness: true,
            charger: true,
            low_battery: true,
            network: true,
            wifi: true,
            bluetooth: true,
            microphone: true,
            audio_device: true,
            caps_lock: true,
            language: true,
            duration: 2000,
        }
    }
}

/// Recordatorios de salud mientras usas el PC (se pausan si estás inactivo).
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(default, rename_all = "camelCase")]
pub struct Health {
    /// Regla 20-20-20: cada 20 minutos, mirar algo lejano durante 20 segundos.
    pub eyes: bool,
    pub eyes_every_min: u32,
    pub stand: bool,
    pub stand_every_min: u32,
}

impl Default for Health {
    fn default() -> Self {
        Self { eyes: false, eyes_every_min: 20, stand: false, stand_every_min: 60 }
    }
}

/// Actualizaciones automáticas: dirección del `latest.json` publicado y si se busca sola.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(default, rename_all = "camelCase")]
pub struct Updates {
    /// Dirección HTTPS del archivo `latest.json` (vacía = actualizaciones sin configurar).
    pub url: String,
    pub auto: bool,
}

/// Archivo `latest.json` de la última versión publicada en GitHub Releases.
pub const DEFAULT_UPDATE_URL: &str = "https://github.com/crixb-be/noki/releases/latest/download/latest.json";

impl Default for Updates {
    fn default() -> Self {
        Self { url: DEFAULT_UPDATE_URL.into(), auto: true }
    }
}

/// Historial del portapapeles (solo texto, solo en memoria). Desactivado por defecto: puede contener contraseñas.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(default, rename_all = "camelCase")]
pub struct Clipboard {
    pub enabled: bool,
    pub max: u32,
}

impl Default for Clipboard {
    fn default() -> Self {
        Self { enabled: false, max: 30 }
    }
}

/// Pomodoro, tareas y avisos de sonido.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(default, rename_all = "camelCase")]
pub struct Productivity {
    pub work_min: u32,
    pub short_break_min: u32,
    pub long_break_min: u32,
    /// Cada cuántos ciclos de enfoque toca descanso largo.
    pub cycles_before_long: u32,
    /// Pasar solo a la siguiente fase al terminar la actual.
    pub auto_next: bool,
    /// Sonido del sistema en alertas (fin de fase, tarea vencida, agua).
    pub sound: bool,
    /// Modo Zen: mientras hay una sesión en marcha, el notch cerrado se reduce a un punto y la cuenta atrás.
    pub zen: bool,
    /// Durante el enfoque del Pomodoro se silencian las alertas de sistema (Wi-Fi, Bluetooth…); solo pasan las críticas.
    pub zen_dnd: bool,
}

impl Default for Productivity {
    fn default() -> Self {
        Self {
            work_min: 25,
            short_break_min: 5,
            long_break_min: 15,
            cycles_before_long: 4,
            auto_next: false,
            sound: true,
            zen: true,
            zen_dnd: true,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq)]
#[serde(default)]
pub struct Settings {
    pub general: General,
    pub appearance: Appearance,
    pub modules: Modules,
    pub shortcuts: Shortcuts,
    pub hud: Hud,
    pub health: Health,
    pub clipboard: Clipboard,
    pub updates: Updates,
    pub productivity: Productivity,
    pub calendar: Calendar,
    pub weather: Weather,
    pub ai: Ai,
    pub topbar: TopBar,
    pub dock: Dock,
}

impl Settings {
    /// Mantiene los valores dentro de rangos que el diseño soporta.
    pub fn clamp(&mut self) {
        // Orden de pestañas: sin repetidas ni desconocidas, y las que falten (p. ej. módulos nuevos) al final.
        let mut seen: Vec<String> = Vec::new();
        for id in self.modules.order.drain(..) {
            if TAB_IDS.contains(&id.as_str()) && !seen.contains(&id) {
                seen.push(id);
            }
        }
        for id in TAB_IDS {
            if !seen.iter().any(|s| s == id) {
                seen.push(id.to_string());
            }
        }
        self.modules.order = seen;
        let g = &mut self.general;
        if g.trigger != "notch" && g.trigger != "edge" {
            g.trigger = "notch".into();
        }
        g.hover_delay = g.hover_delay.min(2000);
        g.close_delay = g.close_delay.min(3000);
        g.monitor = g.monitor.clamp(-1, 15);
        if g.language != "en" {
            g.language = "es".into();
        }
        g.user_name = g.user_name.chars().take(30).collect();

        let a = &mut self.appearance;
        a.collapsed_w = a.collapsed_w.clamp(120, 600);
        a.collapsed_h = a.collapsed_h.clamp(24, 80);
        a.expanded_w = a.expanded_w.clamp(360, 900).max(a.collapsed_w);
        a.expanded_h = a.expanded_h.clamp(200, 600).max(a.collapsed_h);
        a.offset_x = a.offset_x.clamp(-800, 800);
        a.opacity = a.opacity.clamp(30, 100);
        a.notch_dim = a.notch_dim.clamp(0, 85);
        a.bar_dim = a.bar_dim.clamp(0, 85);
        if !crate::appicon::valid(&a.app_icon) {
            a.app_icon = crate::appicon::DEFAULT.into();
        }
        a.bar_opacity = a.bar_opacity.clamp(0, 100);
        a.notch_theme_opacity = a.notch_theme_opacity.clamp(20, 100);
        a.bar_theme_opacity = a.bar_theme_opacity.clamp(0, 100);
        if a.panel_style != "light" && a.panel_style != "dark" { a.panel_style = "light".into(); }
        a.glass.clamp();
        for t in [&mut a.mascot_theme, &mut a.notch_theme, &mut a.bar_theme] {
            if !["auto", "none", "halloween", "christmas", "birthday", "love"].contains(&t.as_str()) {
                *t = "auto".into();
            }
        }
        let valid_birthday = valid_md(&a.birthday);
        a.theme_events.retain(|e| valid_md(&e.from) && valid_md(&e.to) && ["halloween", "christmas", "birthday", "love"].contains(&e.theme.as_str()));
        a.theme_events.truncate(12);
        for e in &mut a.theme_events {
            e.name = e.name.trim().chars().take(40).collect();
        }
        if a.birth_year != 0 && !(1900..=2100).contains(&a.birth_year) {
            a.birth_year = 0;
        }
        if !valid_birthday {
            a.birth_year = 0;
            a.birthday.clear();
        }
        for c in [&mut a.notch_color, &mut a.bar_color] {
            if !crate::backgrounds::valid_color(c) {
                *c = "#000000".into();
            }
        }
        for img in [&mut a.notch_image, &mut a.bar_image] {
            if !crate::backgrounds::valid_name(img) {
                img.clear();
            }
        }
        if !["none", "match", "auto", "color", "image"].contains(&a.bar_mode.as_str()) {
            a.bar_mode = "match".into();
        }

        let b = &mut self.topbar;
        b.height = b.height.clamp(28, 32);
        if !["auto", "dark", "light"].contains(&b.tint.as_str()) {
            b.tint = "auto".into();
        }
        b.avatar_path = b.avatar_path.chars().take(4096).collect();

        let d = &mut self.dock;
        d.icon_size = d.icon_size.clamp(32, 96);
        d.magnify_scale = d.magnify_scale.clamp(1.0, 2.5);
        d.pinned.truncate(64);
        d.folders.truncate(32);
        for p in &mut d.pinned {
            p.name = p.name.trim().chars().take(120).collect();
            p.path = p.path.trim().chars().take(4096).collect();
            p.args = p.args.chars().take(4096).collect();
            p.aumid = p.aumid.trim().chars().take(512).collect();
        }
        for f in &mut d.folders {
            f.name = f.name.trim().chars().take(120).collect();
            f.path = f.path.trim().chars().take(4096).collect();
        }

        // gemini-2.5-flash ya no está disponible para cuentas nuevas; el «lite» responde en ~2 s.
        if self.ai.gemini.model == "gemini-2.5-flash" {
            self.ai.gemini.model = "gemini-3.5-flash-lite".into();
        }
        if self.ai.system_prompt == OLD_PROMPT {
            self.ai.system_prompt = QUICK_PROMPT.into();
        }
        self.hud.duration = self.hud.duration.clamp(800, 6000);
        self.health.eyes_every_min = self.health.eyes_every_min.clamp(5, 120);
        self.health.stand_every_min = self.health.stand_every_min.clamp(15, 240);
        self.clipboard.max = self.clipboard.max.clamp(5, 100);

        let p = &mut self.productivity;
        p.work_min = p.work_min.clamp(1, 180);
        p.short_break_min = p.short_break_min.clamp(1, 60);
        p.long_break_min = p.long_break_min.clamp(1, 120);
        p.cycles_before_long = p.cycles_before_long.clamp(2, 10);
        // Migración de la IA de ≤0.3 (una clave + dos modelos) a la configuración por proveedor.
        let ai = &mut self.ai;
        if !ai.api_key.is_empty() {
            let key = std::mem::take(&mut ai.api_key);
            let target = match ai.provider.as_str() {
                "claude" => &mut ai.claude,
                _ => &mut ai.gemini,
            };
            if target.api_key.is_empty() {
                target.api_key = key;
            }
        }
        if let Some(m) = ai.models.take() {
            if !m.gemini.is_empty() {
                ai.gemini.model = m.gemini;
            }
            if !m.claude.is_empty() {
                ai.claude.model = m.claude;
            }
        }
        if !["gemini", "claude", "openai", "ollama"].contains(&ai.provider.as_str()) {
            ai.provider = "gemini".into();
        }

        let c = &mut self.calendar;
        c.sources.truncate(5);
        for s in &mut c.sources {
            s.url = s.url.trim().to_string();
            s.name = s.name.trim().to_string();
        }
        c.reminder_min = c.reminder_min.min(60);

        let w = &mut self.weather;
        w.lat = w.lat.clamp(-90.0, 90.0);
        w.lon = w.lon.clamp(-180.0, 180.0);
        if w.units != "c" && w.units != "f" {
            w.units = "c".into();
        }
        self.updates.url = self.updates.url.trim().to_string();
        if !self.updates.url.is_empty() && !self.updates.url.starts_with("https://") && !self.updates.url.starts_with("http://") {
            self.updates.url.clear();
        }
        // Sin dirección propia se usan las versiones publicadas en GitHub (se puede apagar con «Buscar automáticamente»).
        if self.updates.url.is_empty() {
            self.updates.url = DEFAULT_UPDATE_URL.into();
        }

        let sc = Shortcuts::default();
        for (cur, def) in [
            (&mut self.shortcuts.record, &sc.record),
            (&mut self.shortcuts.dnd, &sc.dnd),
            (&mut self.shortcuts.add_task, &sc.add_task),
            (&mut self.shortcuts.clip, &sc.clip),
            (&mut self.shortcuts.ai, &sc.ai),
        ] {
            if cur.trim().is_empty() {
                *cur = def.clone();
            }
        }
        if !TAB_MODIFIERS.contains(&self.shortcuts.tabs_modifier.as_str()) {
            self.shortcuts.tabs_modifier = sc.tabs_modifier.clone();
        }
        if self.shortcuts.toggle.trim().is_empty() {
            self.shortcuts.toggle = Shortcuts::default().toggle;
        }
    }
}

fn path(app: &AppHandle) -> Option<PathBuf> {
    app.path().app_config_dir().ok().map(|d| d.join("settings.json"))
}

impl Settings {
    fn key(&self, provider: &str) -> &str {
        match provider {
            "gemini" => &self.ai.gemini.api_key,
            "claude" => &self.ai.claude.api_key,
            _ => &self.ai.openai.api_key,
        }
    }

    fn key_mut(&mut self, provider: &str) -> &mut String {
        match provider {
            "gemini" => &mut self.ai.gemini.api_key,
            "claude" => &mut self.ai.claude.api_key,
            _ => &mut self.ai.openai.api_key,
        }
    }

    /// Copia para el frontend: las claves guardadas se sustituyen por `secrets::MASK`.
    pub fn masked(&self) -> Settings {
        let mut s = self.clone();
        for p in secrets::PROVIDERS {
            if !self.key(p).is_empty() {
                *s.key_mut(p) = secrets::MASK.to_string();
            }
        }
        s
    }

    /// Copia sin claves (lo que se escribe en disco y en las copias de seguridad).
    pub fn without_keys(&self) -> Settings {
        let mut s = self.clone();
        for p in secrets::PROVIDERS {
            s.key_mut(p).clear();
        }
        s
    }

    /// El frontend devuelve `MASK` para las claves que no tocó: se recupera la real.
    pub fn restore_masked_keys(&mut self, current: &Settings) {
        for p in secrets::PROVIDERS {
            if self.key(p) == secrets::MASK {
                *self.key_mut(p) = current.key(p).to_string();
            }
        }
    }

    /// Escribe en el Administrador de credenciales las claves que cambiaron respecto a `previous`.
    pub fn sync_secrets(&self, previous: &Settings) -> Result<(), String> {
        for p in secrets::PROVIDERS {
            if self.key(p) != previous.key(p) {
                secrets::set(p, self.key(p))?;
            }
        }
        Ok(())
    }

    /// Mantiene las claves actuales para los proveedores en los que `other` no trae ninguna.
    pub fn keep_keys_missing_in(&mut self, current: &Settings) {
        for p in secrets::PROVIDERS {
            if self.key(p).is_empty() {
                *self.key_mut(p) = current.key(p).to_string();
            }
        }
    }
}

pub fn load(app: &AppHandle) -> Settings {
    let text = path(app).and_then(|p| fs::read_to_string(p).ok());
    // El campo `topbar` permite distinguir una configuración nueva de una antigua. En las antiguas,
    // `general.reserveSpace` era precisamente la franja superior.
    let had_topbar = text.as_deref()
        .and_then(|raw| serde_json::from_str::<serde_json::Value>(raw.trim_start_matches('\u{feff}')).ok())
        .and_then(|value| value.get("topbar").cloned())
        .is_some();
    let mut s: Settings = text
        .and_then(|t| serde_json::from_str(t.trim_start_matches('\u{feff}')).ok())
        .unwrap_or_default();
    let migrated_topbar = !had_topbar && s.general.reserve_space;
    if migrated_topbar {
        s.topbar.enabled = true;
        s.general.reserve_space = false;
    }
    s.clamp();
    if migrated_topbar {
        let _ = save(app, &s);
    }

    // Migración: si el archivo trae claves en texto plano (versiones ≤0.5) se pasan a Windows y se reescribe
    // el archivo sin ellas. Si Windows no las acepta, se conservan en el archivo para no perderlas.
    let mut all_moved = true;
    let mut had_plaintext = false;
    for p in secrets::PROVIDERS {
        let plain = s.key(p).to_string();
        if plain.is_empty() {
            continue;
        }
        had_plaintext = true;
        if secrets::set(p, &plain).is_err() {
            all_moved = false;
        }
    }
    for p in secrets::PROVIDERS {
        if let Some(stored) = secrets::get(p) {
            *s.key_mut(p) = stored;
        }
    }
    if had_plaintext && all_moved {
        let _ = save(app, &s);
    }
    s
}

/// Escribe `settings.json` sin las claves de API (esas viven en el Administrador de credenciales).
pub fn save(app: &AppHandle, s: &Settings) -> Result<(), String> {
    let p = path(app).ok_or("sin carpeta de configuración")?;
    if let Some(dir) = p.parent() {
        fs::create_dir_all(dir).map_err(|e| e.to_string())?;
    }
    let json = serde_json::to_string_pretty(&s.without_keys()).map_err(|e| e.to_string())?;
    // Escritura atómica: evita dejar un JSON a medias si se corta la energía.
    let tmp = p.with_extension("json.tmp");
    fs::write(&tmp, json).map_err(|e| e.to_string())?;
    fs::rename(&tmp, &p).map_err(|e| e.to_string())
}

pub fn config_dir(app: &AppHandle) -> Option<PathBuf> {
    app.path().app_config_dir().ok()
}
