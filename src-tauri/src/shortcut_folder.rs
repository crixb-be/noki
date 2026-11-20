//! Panel del Dock para abrir el contenido de la carpeta elegida en Ajustes.
use crate::{dock, notch::NotchState, popups, window_utils};
use std::path::Path;
use tauri::{
    window::Color, AppHandle, Emitter, Manager, PhysicalPosition, PhysicalSize, WebviewWindow,
    WebviewWindowBuilder,
};

const LABEL: &str = "shortcutfolder";

pub(crate) fn is_open() -> bool {
    popups::is_active(LABEL)
}

pub(crate) fn close(app: &AppHandle) {
    if is_open() {
        popups::close(app, LABEL);
    }
}

fn configured_path(window: &WebviewWindow) -> String {
    window
        .state::<NotchState>()
        .settings
        .lock()
        .unwrap()
        .dock
        .shortcut_folder
        .clone()
}

fn popup(app: &AppHandle) -> tauri::Result<WebviewWindow> {
    if let Some(window) = app.get_webview_window(LABEL) {
        return Ok(window);
    }
    let window = WebviewWindowBuilder::new(app, LABEL, window_utils::app_url(LABEL))
        .title("Carpeta de accesos directos")
        .inner_size(340.0, 320.0)
        .decorations(false)
        .transparent(true)
        .background_color(Color(0, 0, 0, 0))
        .always_on_top(true)
        .resizable(false)
        .skip_taskbar(true)
        .shadow(false)
        .focused(false)
        .visible(false)
        .on_navigation(|url| window_utils::route_is_valid(url, LABEL))
        .on_page_load(|window, payload| {
            window_utils::repair_page_load(window, payload.event(), payload.url(), LABEL)
        })
        .build()?;
    popups::register_window(app, &window, LABEL, || {});
    Ok(window)
}

#[tauri::command]
pub async fn toggle_shortcut_folder(window: WebviewWindow, x: i32, y: i32) -> Result<(), String> {
    if window.label() != "dock"
        || !dock::can_show()
        || dock::capture_in_progress()
        || configured_path(&window).is_empty()
    {
        return Ok(());
    }
    let app = window.app_handle();
    let hide_taskbar = window
        .state::<NotchState>()
        .settings
        .lock()
        .unwrap()
        .dock
        .hide_taskbar;
    let (work, scale) = dock::work_area_with_monitor(app, hide_taskbar)
        .ok_or("No se encontró el monitor del Dock.")?;
    let origin = window.inner_position().map_err(|error| error.to_string())?;
    let px = |value: f64| (value * scale).round() as i32;
    let width = px(340.0).min(work.width()).max(1);
    let bottom = (origin.y + y - px(8.0)).clamp(work.top + 1, work.bottom);
    let height = px(320.0).min(bottom - work.top).max(1);
    let left = (origin.x + x - width / 2).clamp(work.left, work.right - width);
    let panel = popup(app).map_err(|error| error.to_string())?;
    if !popups::toggle(app, LABEL)? {
        return Ok(());
    }
    let positioned = panel
        .set_position(PhysicalPosition::new(left, bottom - height))
        .and_then(|_| panel.set_size(PhysicalSize::new(width as u32, height as u32)));
    if let Err(error) = positioned {
        close(app);
        return Err(error.to_string());
    }
    let _ = panel.emit_to(LABEL, "shortcut-folder-opened", ());
    popups::present(&panel, LABEL);
    Ok(())
}

#[tauri::command]
pub fn close_shortcut_folder(window: WebviewWindow) {
    if matches!(window.label(), "dock" | LABEL) {
        close(window.app_handle());
    }
}

#[derive(serde::Serialize)]
pub struct Contents {
    path: String,
    entries: Vec<Entry>,
}

#[derive(serde::Serialize)]
struct Entry {
    name: String,
    directory: bool,
    icon_path: Option<String>,
}

#[tauri::command]
pub async fn shortcut_folder_contents(window: WebviewWindow) -> Result<Contents, String> {
    if window.label() != LABEL {
        return Err("Panel no disponible.".into());
    }
    let path = configured_path(&window);
    tauri::async_runtime::spawn_blocking(move || {
        let entries = std::fs::read_dir(&path)
            .map_err(|error| error.to_string())?
            .map(|entry| {
                let entry = entry.map_err(|error| error.to_string())?;
                let kind = entry.file_type().map_err(|error| error.to_string())?;
                let directory = kind.is_dir() || (kind.is_symlink() && entry.path().is_dir());
                Ok(Entry {
                    name: entry.file_name().to_string_lossy().into_owned(),
                    directory,
                    icon_path: directory.then(|| entry.path().to_string_lossy().into_owned()),
                })
            })
            .collect::<Result<Vec<_>, String>>()?;
        Ok(Contents { path, entries })
    })
    .await
    .map_err(|error| error.to_string())?
}

#[tauri::command]
pub async fn open_shortcut_entry(
    window: WebviewWindow,
    path: String,
    name: Option<String>,
) -> Result<(), String> {
    if window.label() != LABEL
        || !is_open()
        || configured_path(&window) != path
        || !Path::new(&path).is_absolute()
    {
        return Err("La carpeta seleccionada cambió.".into());
    }
    tauri::async_runtime::spawn_blocking(move || {
        let root = Path::new(&path);
        let target = match name {
            Some(name)
                if !name.contains(':') && Path::new(&name).file_name() == Some(name.as_ref()) =>
            {
                root.join(name)
            }
            Some(_) => return Err("Nombre de archivo no válido.".into()),
            None => root.to_path_buf(),
        };
        if !root.is_dir() || !target.exists() {
            return Err("El archivo o la carpeta ya no existe.".into());
        }
        dock::dock_launch_blocking(
            target.to_string_lossy().into_owned(),
            String::new(),
            String::new(),
        )
    })
    .await
    .map_err(|error| error.to_string())?
}
