//! Ventana reutilizable del Launchpad y su activación fiable en Windows.

use crate::{dock, notch, popups, settings::Settings, window_utils};
use tauri::{window::Color, AppHandle, Emitter, Manager, WebviewWindow, WebviewWindowBuilder};
use window_vibrancy::{clear_acrylic, clear_blur};

fn create(app: &AppHandle) -> Option<WebviewWindow> {
    if let Some(w) = app.get_webview_window("launchpad") { return Some(w); }
    let window = WebviewWindowBuilder::new(app, "launchpad", window_utils::app_url("launchpad"))
        .title("").inner_size(760.0, 560.0).decorations(false).transparent(true).background_color(Color(0, 0, 0, 0))
        .always_on_top(true).resizable(false).skip_taskbar(true).shadow(false).focused(false).visible(false)
        .on_navigation(|url| window_utils::route_is_valid(url, "launchpad"))
        .on_page_load(|window, payload| window_utils::repair_page_load(window, payload.event(), payload.url(), "launchpad"))
        .build().ok()?;
    let _ = window.set_background_color(Some(Color(0, 0, 0, 0)));
    let _ = clear_acrylic(&window);
    let _ = clear_blur(&window);
    popups::register_window(app, &window, "launchpad", || {});
    window_utils::mark_rounded(&window);
    let style = app.state::<crate::notch::NotchState>().settings.try_lock().map(|s| s.appearance.panel_style.clone()).unwrap_or_else(|_| "light".into());
    window_utils::sync_panel_glass(app, &style);
    window_utils::schedule_app_recovery(&window, "launchpad");
    Some(window)
}

pub fn ensure(app: &AppHandle) { let _ = create(app); }

pub fn open(app: &AppHandle) -> Result<(), String> {
    let window = create(app).ok_or("No se pudo crear Launchpad.")?;
    popups::open(app, "launchpad")?;
    let settings = app.state::<notch::NotchState>().settings.lock().unwrap().clone();
    position(app, &window, &settings);
    popups::present(&window, "launchpad");
    let _ = app.emit("launchpad-open", true);
    Ok(())
}

fn position(app: &AppHandle, window: &WebviewWindow, settings: &Settings) {
    if let Some((_, mon)) = notch::assignments(settings, &notch::monitors(app)).into_iter().find(|(l, _)| l == "main") {
        let (work, scale) = dock::work_area(app).unwrap_or((mon.rect, mon.scale));
        let width = 760;
        let height = 560;
        let w = (width as f64 * scale).round() as i32;
        let h = (height as f64 * scale).round() as i32;
        // El Dock puede estar oculto o en transición; el borde inferior del área de trabajo
        // es una referencia estable y no obliga a consultar otra ventana desde este hilo.
        let dock_top = work.bottom - (80.0 * scale) as i32;
        let bottom_gap = (12.0 * scale) as i32;
        let top = (dock_top - bottom_gap - h).max(work.top + (12.0 * scale) as i32);
        let left = work.left + (work.width() - w) / 2;
        let ui_window = window.clone();
        let _ = window.run_on_main_thread(move || {
            let _ = ui_window.set_position(tauri::PhysicalPosition::new(left, top));
            let _ = ui_window.set_size(tauri::PhysicalSize::new(w as u32, h as u32));
        });
    }
}

#[tauri::command]
pub async fn open_launchpad(app: AppHandle) -> Result<(), String> { open(&app) }

#[tauri::command]
pub async fn toggle_launchpad(app: AppHandle) -> Result<(), String> {
    if popups::toggle(&app, "launchpad")? { open(&app) } else { let _ = app.emit("launchpad-open", false); Ok(()) }
}

#[tauri::command]
pub async fn close_launchpad(app: AppHandle) {
    popups::close(&app, "launchpad");
    let _ = app.emit("launchpad-open", false);
}

pub fn sync(app: &AppHandle, settings: &Settings, fullscreen: bool) {
    if !window_utils::frontend_ready("notch") { return; }
    let Some(window) = create(app) else { return };
    if fullscreen || !settings.dock.enabled {
        let ui_window = window.clone();
        let _ = window.run_on_main_thread(move || { let _ = ui_window.hide(); });
    }
}

