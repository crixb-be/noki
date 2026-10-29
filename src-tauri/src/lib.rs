mod appicon;
mod ai;
mod backgrounds;
mod calendar;
mod clipboard;
mod icons;
mod launcher;
mod legacy;
mod media;
mod dropzone;
mod fileops;
mod notch;
mod quick;
mod settings;
mod secrets;
mod screentime;
mod store;
mod sysmon;
mod system;
mod translate;
mod updates;
mod viz;

use notch::NotchState;
use serde::Serialize;
use settings::Settings;
use system::{SysCmd, SysHandle};
use std::sync::atomic::Ordering::SeqCst;
use tauri::{
    menu::{CheckMenuItem, Menu, MenuItem, PredefinedMenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Emitter, Manager, RunEvent, State, Theme, WebviewUrl, WebviewWindow,
    WebviewWindowBuilder,
};
use tauri_plugin_autostart::{MacosLauncher, ManagerExt as _};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut, ShortcutState};

// ───────────────────────────── ajustes ─────────────────────────────

#[tauri::command]
fn get_settings(state: State<NotchState>) -> Settings {
    // Las claves de API no salen de Rust: el frontend solo ve si hay una guardada.
    state.settings.lock().unwrap().masked()
}

/// Guarda, aplica (layout, atajo, autoinicio) y avisa a todas las ventanas.
/// Devuelve un aviso si algo no pudo aplicarse (p. ej. atajo ocupado).
fn commit_settings(app: &AppHandle, mut new: Settings) -> Result<Option<String>, String> {
    let state = app.state::<NotchState>();
    let prev = state.settings.lock().unwrap().clone();
    new.restore_masked_keys(&prev);
    new.clamp();
    if prev == new {
        return Ok(None);
    }
    // Primero las claves (si Windows las rechaza no se guarda nada a medias).
    new.sync_secrets(&prev)?;
    settings::save(app, &new)?;
    *state.settings.lock().unwrap() = new.clone();
    state.dirty.store(true, SeqCst);

    let mut warning = None;
    if new.shortcuts != prev.shortcuts {
        warning = apply_shortcut(app, &new);
    }
    if new.general.autostart != prev.general.autostart {
        if let Err(e) = apply_autostart(app, new.general.autostart) {
            warning = Some(format!("No se pudo cambiar el inicio con Windows: {e}"));
        }
    }
    if let Some(item) = app.try_state::<TrayDnd>() {
        let _ = item.0.set_checked(new.general.dnd);
    }
    if new.appearance.app_icon != prev.appearance.app_icon {
        appicon::apply(app, &new.appearance.app_icon);
    }
    let _ = app.emit("settings-changed", &new.masked());
    Ok(warning)
}

#[tauri::command]
fn save_settings(app: AppHandle, settings: Settings) -> Result<Option<String>, String> {
    commit_settings(&app, settings)
}

#[tauri::command]
fn reset_settings(app: AppHandle) -> Result<Option<String>, String> {
    commit_settings(&app, Settings::default())
}

/// Atajos activos como (combinación, acción). Las pestañas son «tab:0»…«tab:8».
fn shortcut_list(sc: &settings::Shortcuts) -> Vec<(String, String)> {
    let mut v = Vec::new();
    for (on, key, action) in [
        (sc.toggle_enabled, &sc.toggle, "toggle"),
        (sc.record_enabled, &sc.record, "record"),
        (sc.dnd_enabled, &sc.dnd, "dnd"),
        (sc.add_task_enabled, &sc.add_task, "addTask"),
    ] {
        if on {
            v.push((key.clone(), action.to_string()));
        }
    }
    if sc.tabs_enabled {
        for i in 1..=9 {
            v.push((format!("{}+{i}", sc.tabs_modifier), format!("tab:{}", i - 1)));
        }
    }
    v
}

fn apply_shortcut(app: &AppHandle, s: &Settings) -> Option<String> {
    let gs = app.global_shortcut();
    let _ = gs.unregister_all();
    let mut problems: Vec<String> = Vec::new();
    for (key, _) in shortcut_list(&s.shortcuts) {
        match key.parse::<Shortcut>() {
            Ok(sc) => {
                if let Err(e) = gs.register(sc) {
                    problems.push(format!("«{key}»: {e}"));
                }
            }
            Err(e) => problems.push(format!("«{key}» no es válido: {e}")),
        }
    }
    (!problems.is_empty()).then(|| format!("No se pudo registrar el atajo {}", problems.join("; ")))
}

/// Ejecuta la acción de un atajo. «toggle» lo resuelve Rust; el resto lo atiende la ventana de JS que corresponda.
fn run_shortcut(app: &AppHandle, action: &str) {
    match action {
        "toggle" => notch::toggle(app),
        // Estado global (grabación, «No molestar»): lo gestiona la ventana principal.
        "record" | "dnd" => {
            let _ = app.emit("shortcut-action", serde_json::json!({ "label": "main", "action": action }));
        }
        // Abrir una pestaña o la tarea rápida: antes se abre el notch de la pantalla del cursor.
        _ => {
            if let Some(label) = notch::open_here(app) {
                let _ = app.emit("shortcut-action", serde_json::json!({ "label": label, "action": action }));
            }
        }
    }
}

/// Guarda texto en un archivo elegido por el usuario (exportar transcripciones, apariencia…).
#[tauri::command]
fn save_text(path: String, text: String) -> Result<(), String> {
    std::fs::write(path, text).map_err(|e| e.to_string())
}

/// Guarda bytes (en base64) en un archivo elegido por el usuario (exportar audios).
#[tauri::command]
fn save_bytes(path: String, data: String) -> Result<(), String> {
    use base64::Engine;
    let bytes = base64::engine::general_purpose::STANDARD.decode(data).map_err(|e| e.to_string())?;
    std::fs::write(path, bytes).map_err(|e| e.to_string())
}

/// Lee un archivo de texto pequeño elegido por el usuario (importar apariencia).
#[tauri::command]
fn read_text(path: String) -> Result<String, String> {
    let m = std::fs::metadata(&path).map_err(|e| e.to_string())?;
    if m.len() > 1_000_000 {
        return Err("El archivo es demasiado grande.".into());
    }
    std::fs::read_to_string(path).map_err(|e| e.to_string())
}

fn apply_autostart(app: &AppHandle, enable: bool) -> Result<(), String> {
    let al = app.autolaunch();
    let current = al.is_enabled().map_err(|e| e.to_string())?;
    if enable != current {
        if enable { al.enable() } else { al.disable() }.map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct MonitorDto {
    index: usize,
    name: String,
    width: u32,
    height: u32,
    primary: bool,
}

#[tauri::command]
fn list_monitors(app: AppHandle) -> Vec<MonitorDto> {
    notch::list_monitors(&app)
        .into_iter()
        .map(|m| MonitorDto { index: m.index, name: m.name, width: m.width, height: m.height, primary: m.primary })
        .collect()
}

#[tauri::command]
fn data_dir(app: AppHandle) -> Option<String> {
    settings::config_dir(&app).map(|p| p.to_string_lossy().into_owned())
}

// ───────────────────────────── estado del notch ─────────────────────────────

#[tauri::command]
fn notch_state(window: tauri::Window, state: State<NotchState>) -> bool {
    state.win(window.label()).map(|w| w.open.load(SeqCst)).unwrap_or(false)
}

#[tauri::command]
fn set_pinned(window: tauri::Window, state: State<NotchState>, pinned: bool) {
    if let Some(w) = state.win(window.label()) {
        w.pinned.store(pinned && w.open.load(SeqCst), SeqCst);
    }
}

#[tauri::command]
fn request_open(app: AppHandle, window: tauri::Window, open: bool, sticky: bool) {
    let label = window.label().to_string();
    if let Some(w) = app.state::<NotchState>().win(&label) {
        w.sticky.store(open && sticky, SeqCst);
    }
    notch::set_open(&app, &label, open);
}

#[tauri::command]
fn set_peek(window: tauri::Window, state: State<NotchState>, width: i32, height: i32) {
    if let Some(w) = state.win(window.label()) {
        w.peek_w.store(width.clamp(0, 900), SeqCst);
        w.peek_h.store(height.clamp(0, 200), SeqCst);
    }
}

/// Un aviso (tarea vencida, fin de Pomodoro…) generado por la ventana principal se muestra en TODOS los notch.
#[tauri::command]
fn broadcast_notify(app: AppHandle, payload: serde_json::Value) {
    let _ = app.emit("notch-notify", payload);
}

// ───────────────────────────── música y sistema ─────────────────────────────

#[tauri::command]
fn media_state(media: State<media::MediaHandle>) -> media::MediaState {
    media.latest()
}

#[tauri::command]
fn media_control(media: State<media::MediaHandle>, action: String) {
    use media::Cmd;
    let cmd = match action.as_str() {
        "toggle" => Cmd::Toggle,
        "play" => Cmd::Play,
        "pause" => Cmd::Pause,
        "next" => Cmd::Next,
        "prev" => Cmd::Prev,
        _ => return,
    };
    media.send(cmd);
}

#[tauri::command]
fn media_seek(media: State<media::MediaHandle>, seconds: f64) {
    media.send(media::Cmd::Seek(seconds));
}

#[tauri::command]
fn set_volume(sys: State<SysHandle>, value: f32) {
    sys.send(SysCmd::Volume(value));
}

#[tauri::command]
fn set_mute(sys: State<SysHandle>, muted: bool) {
    sys.send(SysCmd::Mute(muted));
}

#[tauri::command]
fn set_brightness(sys: State<SysHandle>, value: u8) {
    sys.send(SysCmd::Brightness(value));
}

// ───────────────────────────── ventanas ─────────────────────────────

fn show_settings(app: &AppHandle, section: Option<String>) -> Result<(), String> {
    if let Some(w) = app.get_webview_window("settings") {
        let _ = w.unminimize();
        let _ = w.show();
        let _ = w.set_focus();
        if let Some(s) = section {
            let _ = w.emit("settings-section", s);
        }
        return Ok(());
    }
    let url = format!("index.html?view=settings&section={}", section.unwrap_or_else(|| "general".into()));
    WebviewWindowBuilder::new(app, "settings", WebviewUrl::App(url.into()))
        .title("Noki — Ajustes")
        .inner_size(880.0, 600.0)
        .min_inner_size(720.0, 480.0)
        .center()
        .theme(Some(Theme::Dark))
        .build()
        .map(|w| {
            let id = app.state::<NotchState>().settings.lock().unwrap().appearance.app_icon.clone();
            if let Some(img) = appicon::image(&id) {
                let _ = w.set_icon(img);
            }
        })
        .map_err(|e| e.to_string())
}

/// Crear ventanas desde el hilo principal puede bloquear en Windows: siempre desde otro hilo.
fn open_settings_async(app: &AppHandle, section: Option<String>) {
    let app = app.clone();
    std::thread::spawn(move || {
        let _ = show_settings(&app, section);
    });
}

#[tauri::command]
async fn open_settings(app: AppHandle, section: Option<String>) -> Result<(), String> {
    show_settings(&app, section)
}

#[tauri::command]
fn quit_app(app: AppHandle) {
    app.exit(0);
}

// ───────────────────────────── datos ─────────────────────────────

#[tauri::command]
fn store_all(store: State<store::Store>) -> serde_json::Map<String, serde_json::Value> {
    store.all()
}

#[tauri::command]
fn store_set(
    app: AppHandle,
    window: tauri::Window,
    store: State<store::Store>,
    key: String,
    value: serde_json::Value,
) -> Result<(), String> {
    store.set(key.clone(), value.clone())?;
    // Las demás ventanas (otros notch) reflejan el cambio al instante; la que lo originó lo ignora por `source`.
    let _ = app.emit("store-changed", serde_json::json!({ "key": key, "value": value, "source": window.label() }));
    Ok(())
}

/// Sonido de notificación del sistema (más fiable que WebAudio con la ventana sin foco).
#[tauri::command]
fn alert_sound() {
    use windows::Win32::{System::Diagnostics::Debug::MessageBeep, UI::WindowsAndMessaging::MB_ICONASTERISK};
    unsafe {
        let _ = MessageBeep(MB_ICONASTERISK);
    }
}

// ───────────────────────────── archivos del estante ─────────────────────────────

/// Para cada ruta, si todavía existe (los archivos del estante pueden haberse movido o borrado).
#[tauri::command]
fn files_exist(paths: Vec<String>) -> Vec<bool> {
    paths.iter().map(|p| std::path::Path::new(p).exists()).collect()
}

/// Abre el Explorador con el archivo seleccionado.
#[tauri::command]
fn reveal_in_folder(path: String) -> Result<(), String> {
    let p = std::path::Path::new(&path);
    if !p.exists() {
        return Err("el archivo ya no existe".into());
    }
    std::process::Command::new("explorer")
        .arg(format!("/select,{}", p.display()))
        .spawn()
        .map(|_| ())
        .map_err(|e| e.to_string())
}

fn add_to_zip<W: std::io::Write + std::io::Seek>(
    zip: &mut zip::ZipWriter<W>,
    base: &std::path::Path,
    path: &std::path::Path,
    opts: zip::write::SimpleFileOptions,
) -> Result<(), String> {
    let name = path.strip_prefix(base).unwrap_or(path).to_string_lossy().replace('\\', "/");
    if path.is_dir() {
        zip.add_directory(format!("{name}/"), opts).map_err(|e| e.to_string())?;
        for entry in std::fs::read_dir(path).map_err(|e| e.to_string())? {
            add_to_zip(zip, base, &entry.map_err(|e| e.to_string())?.path(), opts)?;
        }
    } else {
        zip.start_file(name, opts).map_err(|e| e.to_string())?;
        let mut f = std::fs::File::open(path).map_err(|e| e.to_string())?;
        std::io::copy(&mut f, zip).map_err(|e| e.to_string())?;
    }
    Ok(())
}

/// Comprime los archivos indicados en un .zip dentro de la carpeta de datos y devuelve su ruta.
#[tauri::command]
async fn zip_files(app: AppHandle, paths: Vec<String>, name: Option<String>) -> Result<String, String> {
    if paths.is_empty() {
        return Err("no hay archivos".into());
    }
    let dir = app
        .path()
        .app_local_data_dir()
        .map_err(|e| e.to_string())?
        .join("zips");
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    // Nombre sin caracteres que Windows no admite en archivos.
    let name: String = name
        .unwrap_or_else(|| "Noki".into())
        .chars()
        .filter(|c| !r#"\/:*?"<>|"#.contains(*c))
        .collect();
    let out = dir.join(format!("{name}.zip"));

    let file = std::fs::File::create(&out).map_err(|e| e.to_string())?;
    let mut zip = zip::ZipWriter::new(file);
    let opts = zip::write::SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);
    for p in &paths {
        let path = std::path::Path::new(p);
        let base = path.parent().unwrap_or(path);
        add_to_zip(&mut zip, base, path, opts)?;
    }
    zip.finish().map_err(|e| e.to_string())?;
    Ok(out.to_string_lossy().into_owned())
}

// ───────────────────────────── archivos (utilidades previas) ─────────────────────────────

#[derive(serde::Serialize)]
struct FileInfo {
    /// Bytes; `None` en carpetas.
    size: Option<u64>,
    dir: bool,
}

/// Tamaño y tipo de un archivo o carpeta soltado en el estante.
#[tauri::command]
fn file_size(path: String) -> Result<FileInfo, String> {
    let m = std::fs::metadata(path).map_err(|e| e.to_string())?;
    Ok(FileInfo { size: if m.is_dir() { None } else { Some(m.len()) }, dir: m.is_dir() })
}

/// Abre un archivo con la aplicación predeterminada de Windows.
#[tauri::command]
fn open_path(path: String) -> Result<(), String> {
    if !std::path::Path::new(&path).exists() {
        return Err("el archivo ya no existe".into());
    }
    std::process::Command::new("explorer")
        .arg(&path)
        .spawn()
        .map(|_| ())
        .map_err(|e| e.to_string())
}

/// Sitios a los que la app puede mandarte (redes del desarrollador, donaciones y el repositorio).
const ALLOWED_HOSTS: [&str; 7] = ["instagram.com", "www.instagram.com", "facebook.com", "www.facebook.com", "x.com", "ko-fi.com", "github.com"];

/// Abre un enlace permitido en el navegador predeterminado.
#[tauri::command]
fn open_url(url: String) -> Result<(), String> {
    let parsed = reqwest::Url::parse(&url).map_err(|_| "Enlace no válido".to_string())?;
    if parsed.scheme() != "https" || !parsed.host_str().is_some_and(|h| ALLOWED_HOSTS.contains(&h)) {
        return Err("Enlace no permitido".into());
    }
    std::process::Command::new("explorer")
        .arg(parsed.as_str())
        .spawn()
        .map(|_| ())
        .map_err(|e| e.to_string())
}

// ───────────────────────────── inactividad y copias de seguridad ─────────────────────────────

#[derive(serde::Serialize)]
struct DetectedCity {
    label: String,
    lat: f64,
    lon: f64,
}

/// Estima la ciudad por la dirección IP pública (ipwho.is, gratuito y sin clave). Solo se consulta cuando
/// el usuario lo pide desde Ajustes → Clima.
#[tauri::command]
async fn detect_city() -> Result<DetectedCity, String> {
    let fail = || "No se pudo detectar la ciudad. Revisa tu conexión o búscala a mano.".to_string();
    let client = reqwest::Client::builder().timeout(std::time::Duration::from_secs(8)).build().map_err(|_| fail())?;
    let v: serde_json::Value = client.get("https://ipwho.is/").send().await.map_err(|_| fail())?.json().await.map_err(|_| fail())?;
    if v["success"].as_bool() != Some(true) {
        return Err(fail());
    }
    let part = |k: &str| v[k].as_str().filter(|s| !s.is_empty()).map(str::to_string);
    let label = [part("city"), part("region"), part("country")].into_iter().flatten().collect::<Vec<_>>().join(", ");
    match (v["latitude"].as_f64(), v["longitude"].as_f64()) {
        (Some(lat), Some(lon)) if !label.is_empty() => Ok(DetectedCity { label, lat, lon }),
        _ => Err(fail()),
    }
}

/// Segundos sin tocar teclado ni ratón (para pausar recordatorios de salud cuando te has ido).
#[tauri::command]
fn idle_seconds() -> u64 {
    use windows::Win32::{
        System::SystemInformation::GetTickCount,
        UI::Input::KeyboardAndMouse::{GetLastInputInfo, LASTINPUTINFO},
    };
    let mut info = LASTINPUTINFO { cbSize: std::mem::size_of::<LASTINPUTINFO>() as u32, dwTime: 0 };
    unsafe {
        if GetLastInputInfo(&mut info).as_bool() {
            (GetTickCount().wrapping_sub(info.dwTime) / 1000) as u64
        } else {
            0
        }
    }
}

/// Guarda ajustes y datos en un solo archivo JSON. Las claves de API NO se incluyen: viven en el
/// Administrador de credenciales de este equipo y hay que escribirlas de nuevo en el otro.
#[tauri::command]
fn backup_export(state: State<NotchState>, store: State<store::Store>, path: String) -> Result<(), String> {
    let doc = serde_json::json!({
        "app": "noki",
        "version": env!("CARGO_PKG_VERSION"),
        "settings": state.settings.lock().unwrap().without_keys(),
        "data": store.all(),
    });
    let text = serde_json::to_string_pretty(&doc).map_err(|e| e.to_string())?;
    std::fs::write(path, text).map_err(|e| e.to_string())
}

/// Restaura una copia: reemplaza ajustes y datos y reinicia la app para aplicarlos.
#[tauri::command]
fn backup_import(app: AppHandle, state: State<NotchState>, store: State<store::Store>, path: String) -> Result<(), String> {
    let text = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;
    let doc: serde_json::Value = serde_json::from_str(text.trim_start_matches('\u{feff}'))
        .map_err(|_| "El archivo no es una copia válida.".to_string())?;
    // Se aceptan también las copias hechas con el nombre anterior de la app («mi-notch»).
    if !matches!(doc.get("app").and_then(|v| v.as_str()), Some("noki" | "mi-notch")) {
        return Err("Este archivo no es una copia de seguridad de Noki.".into());
    }
    let mut new: Settings =
        serde_json::from_value(doc.get("settings").cloned().unwrap_or_default()).map_err(|e| e.to_string())?;
    new.clamp();
    // Copias nuevas no traen claves: se conservan las de este equipo. Copias antiguas (≤0.5) sí las traen:
    // se pasan al Administrador de credenciales.
    let current = state.settings.lock().unwrap().clone();
    new.keep_keys_missing_in(&current);
    new.sync_secrets(&current)?;
    let data = doc.get("data").and_then(|v| v.as_object()).cloned().unwrap_or_default();
    settings::save(&app, &new)?;
    store.replace(data)?;
    app.restart()
}

// ───────────────────────────── bandeja ─────────────────────────────

/// Casilla «No molestar» de la bandeja, para mantenerla sincronizada con los ajustes.
struct TrayDnd(CheckMenuItem<tauri::Wry>);

fn build_tray(app: &AppHandle) -> tauri::Result<()> {
    let settings_i = MenuItem::with_id(app, "settings", "Ajustes…", true, None::<&str>)?;
    let toggle_i = MenuItem::with_id(app, "toggle", "Abrir / cerrar notch", true, None::<&str>)?;
    let visible_i = MenuItem::with_id(app, "visible", "Mostrar / ocultar notch", true, None::<&str>)?;
    let dnd_i = CheckMenuItem::with_id(app, "dnd", "No molestar", true, false, None::<&str>)?;
    let quit_i = MenuItem::with_id(app, "quit", "Salir de Noki", true, None::<&str>)?;
    let sep = PredefinedMenuItem::separator(app)?;
    let menu = Menu::with_items(app, &[&settings_i, &toggle_i, &visible_i, &dnd_i, &sep, &quit_i])?;
    app.manage(TrayDnd(dnd_i));

    let mut builder = TrayIconBuilder::with_id("main")
        .tooltip("Noki")
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| match event.id.as_ref() {
            "settings" => open_settings_async(app, None),
            "toggle" => notch::toggle(app),
            "visible" => {
                let mut s = app.state::<NotchState>().settings.lock().unwrap().clone();
                s.general.visible = !s.general.visible;
                let _ = commit_settings(app, s);
            }
            "dnd" => {
                let mut s = app.state::<NotchState>().settings.lock().unwrap().clone();
                s.general.dnd = !s.general.dnd;
                let _ = commit_settings(app, s);
            }
            "quit" => app.exit(0),
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click { button: MouseButton::Left, button_state: MouseButtonState::Up, .. } = event {
                open_settings_async(tray.app_handle(), None);
            }
        });
    let chosen = app.try_state::<NotchState>().map(|st| st.settings.lock().unwrap().appearance.app_icon.clone());
    if let Some(icon) = chosen.as_deref().and_then(appicon::image).or_else(|| app.default_window_icon().cloned()) {
        builder = builder.icon(icon);
    }
    builder.build(app)?;
    Ok(())
}

// ───────────────────────────── arranque ─────────────────────────────

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let app = tauri::Builder::default()
        // Debe ser el primer plugin: una segunda ejecución solo abre los Ajustes de la primera.
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            open_settings_async(app, None);
        }))
        .plugin(tauri_plugin_drag::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_autostart::init(MacosLauncher::LaunchAgent, None))
        .plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_handler(|app, shortcut, event| {
                    if event.state() != ShortcutState::Pressed {
                        return;
                    }
                    let list = shortcut_list(&app.state::<notch::NotchState>().settings.lock().unwrap().shortcuts);
                    let hit = list.into_iter().find(|(key, _)| key.parse::<Shortcut>().map(|k| &k == shortcut).unwrap_or(false));
                    if let Some((_, action)) = hit {
                        run_shortcut(app, &action);
                    }
                })
                .build(),
        )
        .invoke_handler(tauri::generate_handler![
            get_settings,
            save_settings,
            reset_settings,
            list_monitors,
            data_dir,
            notch_state,
            set_pinned,
            request_open,
            set_peek,
            broadcast_notify,
            media_state,
            media_control,
            media_seek,
            set_volume,
            set_mute,
            set_brightness,
            open_settings,
            quit_app,
            file_size,
            save_text,
            save_bytes,
            read_text,
            ai::ai_chat,
            ai::ai_cancel,
            updates::update_check,
            updates::update_install,
            clipboard::clip_list,
            clipboard::clip_copy,
            clipboard::clip_remove,
            clipboard::clip_clear,
            launcher::launcher_apps,
            launcher::launch_app,
            quick::quick_state,
            quick::quick_set,
            quick::quick_action,
            viz::set_visualizer,
            idle_seconds,
            detect_city,
            icons::app_icon,
            backgrounds::bg_import,
            backgrounds::bg_data,
            translate::translate_text,
            translate::ocr_image,
            translate::ocr_file,
            fileops::shelf_copy,
            fileops::shelf_move,
            fileops::shelf_trash,
            fileops::shelf_unzip,
            backup_export,
            backup_import,
            calendar::calendar_fetch,
            open_path,
            open_url,
            sysmon::system_stats,
            sysmon::top_processes,
            sysmon::end_task,
            screentime::screentime_clear,
            store_all,
            store_set,
            alert_sound,
            files_exist,
            reveal_in_folder,
            zip_files,
        ])
        .setup(|app| {
            let handle = app.handle().clone();
            legacy::migrate_data(&handle);
            let loaded = settings::load(&handle);
            app.manage(NotchState::new(loaded.clone()));
            app.manage(store::Store::load(&handle));
            app.manage(ai::AiJobs::default());
            quick::start_keep_awake_thread();
            app.manage(clipboard::History::default());
            app.manage(launcher::AppCache::default());
            app.manage(viz::Viz::default());
            app.manage(updates::Pending::default());
            app.manage(sysmon::SysMonState::new(sysmon::SysMon::new()));
            screentime::start(handle.clone());

            if let Some(msg) = apply_shortcut(&handle, &loaded) {
                eprintln!("{msg}");
            }
            let _ = apply_autostart(&handle, loaded.general.autostart);
            build_tray(&handle)?;

            let (media_handle, media_rx) = media::MediaHandle::new();
            app.manage(media_handle);
            media::start(handle.clone(), media_rx);
            let (sys_handle, sys_rx) = SysHandle::new();
            app.manage(sys_handle);
            system::start(handle.clone(), sys_rx);

            // La ventana nace oculta; el hilo del notch la coloca y la muestra sin robar el foco.
            let _window: WebviewWindow = app.get_webview_window("main").expect("ventana main");
            clipboard::start(handle.clone());
            viz::start(handle.clone());
            updates::start_auto(handle.clone());
            // La casilla de la bandeja arranca con el valor guardado.
            if let Some(item) = handle.try_state::<TrayDnd>() {
                let _ = item.0.set_checked(loaded.general.dnd);
            }
            notch::start(handle);
            Ok(())
        })
        .build(tauri::generate_context!())
        .expect("error al construir la aplicación");

    app.run(|app, event| {
        if let RunEvent::Exit = event {
            // Libera el espacio reservado en el escritorio.
            notch::appbar_remove(&app.state::<NotchState>());
        }
    });
}
