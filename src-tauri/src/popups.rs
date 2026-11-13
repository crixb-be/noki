//! Gestor único de ventanas emergentes de Noki.

use std::{sync::{Arc, Mutex, OnceLock}, time::{Duration, Instant}};
use tauri::{AppHandle, Emitter, Manager, WebviewWindow, WindowEvent};
use windows::Win32::{
    System::Threading::GetCurrentProcessId,
    UI::WindowsAndMessaging::{GetForegroundWindow, GetWindowThreadProcessId},
};

const BLUR_DEBOUNCE: Duration = Duration::from_millis(80);
const OPEN_GRACE: Duration = Duration::from_millis(300);

#[derive(Default)]
struct PopupState {
    active_popup: Option<String>,
    opened_at: Option<Instant>,
    last_blur_hide: Option<(String, Instant)>,
    generation: u64,
}

static STATE: OnceLock<Mutex<PopupState>> = OnceLock::new();

fn state() -> &'static Mutex<PopupState> { STATE.get_or_init(|| Mutex::new(PopupState::default())) }

fn window_name(id: &str) -> &'static str {
    if id == "launchpad" { "launchpad" }
    else if id == "spotlight" { "spotlight" }
    else { "barpanel" }
}

fn same_window(left: &str, right: &str) -> bool {
    if left == right { return true; }
    left.starts_with("bar_panel:") && right.starts_with("bar_panel:")
}

fn active_for_window(active: Option<&str>, id: &str) -> bool {
    match active {
        Some(value) if id == "bar_panel" => value.starts_with("bar_panel:"),
        Some(value) => value == id,
        None => false,
    }
}

fn hide_window(app: &AppHandle, name: &'static str) {
    if let Some(window) = app.get_webview_window(name) {
        let ui_window = window.clone();
        let _ = window.run_on_main_thread(move || { let _ = ui_window.hide(); });
    }
}

fn hide_other_windows(app: &AppHandle, id: &str) {
    for name in ["launchpad", "spotlight", "barpanel"] {
        let keep = (name == "launchpad" && id == "launchpad")
            || (name == "spotlight" && id == "spotlight")
            || (name == "barpanel" && id.starts_with("bar_panel:"));
        if !keep { hide_window(app, name); }
    }
}

/// Abre un popup lógico y oculta los demás inmediatamente.
pub fn open(app: &AppHandle, id: &str) -> Result<(), String> {
    if !matches!(id, "launchpad" | "spotlight") && !id.starts_with("bar_panel:") {
        return Err("Popup desconocido.".into());
    }
    let mut current = state().lock().unwrap();
    let previous = current.active_popup.clone();
    current.active_popup = Some(id.to_string());
    current.opened_at = Some(Instant::now());
    current.last_blur_hide = None;
    current.generation = current.generation.wrapping_add(1);
    if previous.as_deref().is_none_or(|value| !same_window(value, id)) {
        hide_other_windows(app, id);
    }
    Ok(())
}

/// Cierra el popup indicado. Devuelve `true` si era el popup activo.
pub fn close(app: &AppHandle, id: &str) -> bool {
    let mut current = state().lock().unwrap();
    let was_active = current.active_popup.as_deref().is_some_and(|value| {
        if id == "bar_panel" { value.starts_with("bar_panel:") } else { value == id }
    });
    if was_active {
        current.active_popup = None;
        current.opened_at = None;
        current.generation = current.generation.wrapping_add(1);
    }
    if was_active || id == "bar_panel" { hide_window(app, window_name(id)); }
    was_active
}

/// Alterna un popup. Un toggle del mismo id justo después de un cierre por blur
/// se suprime para evitar la carrera blur -> click -> reapertura.
pub fn toggle(app: &AppHandle, id: &str) -> Result<bool, String> {
    if !matches!(id, "launchpad" | "spotlight") && !id.starts_with("bar_panel:") {
        return Err("Popup desconocido.".into());
    }
    let mut current = state().lock().unwrap();
    if current.active_popup.as_deref().is_some_and(|value| value == id) {
        current.active_popup = None;
        current.opened_at = None;
        current.generation = current.generation.wrapping_add(1);
        hide_window(app, window_name(id));
        return Ok(false);
    }
    if current.last_blur_hide.as_ref().is_some_and(|(last_id, at)| last_id == id && at.elapsed() < OPEN_GRACE) {
        return Ok(false);
    }
    current.active_popup = Some(id.to_string());
    current.opened_at = Some(Instant::now());
    current.last_blur_hide = None;
    current.generation = current.generation.wrapping_add(1);
    hide_other_windows(app, id);
    Ok(true)
}

pub fn close_all(app: &AppHandle) {
    {
        let mut current = state().lock().unwrap();
        current.active_popup = None;
        current.opened_at = None;
        current.generation = current.generation.wrapping_add(1);
    }
    for name in ["launchpad", "spotlight", "barpanel"] { hide_window(app, name); }
}

#[tauri::command]
pub async fn close_all_command(app: AppHandle) { close_all(&app); }

/// Registra el cierre automático por pérdida de foco para una ventana ya creada.
/// El callback solo limpia el estado específico de la vista; la lógica de blur
/// y el debounce permanecen centralizados aquí.
pub fn register_window<F>(app: &AppHandle, window: &WebviewWindow, id: &'static str, on_blur_close: F)
where
    F: Fn() + Send + Sync + 'static,
{
    let app = app.clone();
    let callback = Arc::new(on_blur_close);
    window.on_window_event(move |event| {
        if !matches!(event, WindowEvent::Focused(false)) { return; }
        let app = app.clone();
        let callback = callback.clone();
        std::thread::spawn(move || {
            let epoch = {
                let current = state().lock().unwrap();
                if !active_for_window(current.active_popup.as_deref(), id)
                    || current.opened_at.is_some_and(|at| at.elapsed() < OPEN_GRACE)
                { return; }
                current.generation
            };
            std::thread::sleep(BLUR_DEBOUNCE);
            let Some(window) = app.get_webview_window(window_name(id)) else { return };
            let should_check = {
                let current = state().lock().unwrap();
                active_for_window(current.active_popup.as_deref(), id)
                    && current.generation == epoch
                    && !current.opened_at.is_some_and(|at| at.elapsed() < OPEN_GRACE)
            };
            if !should_check { return; }
            let ui_window = window.clone();
            let _ = window.run_on_main_thread(move || unsafe {
                let foreground = GetForegroundWindow();
                let mut pid = 0u32;
                if !foreground.0.is_null() { GetWindowThreadProcessId(foreground, Some(&mut pid)); }
                if pid == GetCurrentProcessId() { return; }
                let _ = ui_window.hide();
                let mut current = state().lock().unwrap();
                if active_for_window(current.active_popup.as_deref(), id) && current.generation == epoch {
                    current.last_blur_hide = Some((current.active_popup.clone().unwrap_or_else(|| id.to_string()), Instant::now()));
                    current.active_popup = None;
                    current.opened_at = None;
                    let _ = app.emit("popup-closed", id);
                    callback();
                }
            });
        });
    });
}

