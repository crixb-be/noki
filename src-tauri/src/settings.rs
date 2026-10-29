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
}

impl Default for General {
    fn default() -> Self {
        Self {
            visible: true,
            autostart: false,
            reserve_space: true,
            trigger: "notch".into(),
            hover_delay: 120,
            close_delay: 300,
            hide_on_fullscreen: true,
            monitor: -1,
            all_monitors: false,
            dnd: false,
            language: "es".into(),
        }
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
    /// Cumpleaños de quien usa Noki, "MM-DD" (vacío si no se indica). Solo se usa en este equipo.
    pub birthday: String,
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
            birthday: String::new(),
        }
    }
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
    pub toggle: String,
}

impl Default for Shortcuts {
    fn default() -> Self {
        Self { toggle: "Alt+N".into() }
    }
}

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
        for t in [&mut a.mascot_theme, &mut a.notch_theme, &mut a.bar_theme] {
            if !["auto", "none", "halloween", "christmas", "birthday", "love"].contains(&t.as_str()) {
                *t = "auto".into();
            }
        }
        let valid_birthday = a.birthday.split_once('-').is_some_and(|(m, d)| {
            m.len() == 2
                && d.len() == 2
                && matches!(m.parse::<u32>(), Ok(1..=12))
                && matches!(d.parse::<u32>(), Ok(1..=31))
        });
        if !valid_birthday {
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
    let mut s: Settings = path(app)
        .and_then(|p| fs::read_to_string(p).ok())
        // Se ignora el BOM por si el archivo se editó con el Bloc de notas.
        .and_then(|t| serde_json::from_str(t.trim_start_matches('\u{feff}')).ok())
        .unwrap_or_default();
    s.clamp();

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
