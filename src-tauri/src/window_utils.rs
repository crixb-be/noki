//! Utilidades comunes para mantener las ventanas emergentes sin marco de Windows.

use std::{
    collections::HashSet,
    sync::{Mutex, OnceLock},
    time::Duration,
};
use tauri::{webview::PageLoadEvent, Url, WebviewUrl, WebviewWindow};

static READY_VIEWS: OnceLock<Mutex<HashSet<String>>> = OnceLock::new();

fn ready_views() -> &'static Mutex<HashSet<String>> {
    READY_VIEWS.get_or_init(|| Mutex::new(HashSet::new()))
}

pub fn mark_frontend_ready(view: &str) {
    ready_views().lock().unwrap().insert(view.to_string());
}

pub fn frontend_ready(view: &str) -> bool {
    ready_views().lock().unwrap().contains(view)
}

pub fn app_url(view: &str) -> WebviewUrl {
    WebviewUrl::App(format!("index.html?view={view}").into())
}

pub fn route_is_valid(url: &Url, view: &str) -> bool {
    let app_origin = url.scheme() == "tauri"
        || (matches!(url.scheme(), "http" | "https")
            && matches!(url.host_str(), Some("localhost") | Some("127.0.0.1") | Some("tauri.localhost")));
    app_origin && url.path().ends_with("index.html") && url.query_pairs().any(|(key, value)| key == "view" && value == view)
}

fn corrected_app_url(current: &Url, view: &str) -> Url {
    let mut url = current.clone();
    if url.scheme() == "tauri" {
        url.set_path("/index.html");
    } else {
        url.set_path("/index.html");
    }
    url.set_query(Some(&format!("view={view}")));
    url
}

pub fn repair_page_load(window: WebviewWindow, event: PageLoadEvent, url: &Url, view: &'static str) {
    if event != PageLoadEvent::Finished || route_is_valid(url, view) {
        return;
    }
    let target = corrected_app_url(url, view);
    let ui_window = window.clone();
    let _ = window.run_on_main_thread(move || {
        let _ = ui_window.navigate(target);
    });
}

/// Reintenta la ruta de la aplicación mientras Vite/WebView2 termina de estar disponible.
/// El evento `frontend-ready` detiene la secuencia tan pronto como React ha pintado la vista.
pub fn schedule_app_recovery(window: &WebviewWindow, view: &'static str) {
    let Some(target) = window.url().ok() else { return };
    let window = window.clone();
    std::thread::spawn(move || {
        for delay in [120u64, 250, 500, 1_000, 2_000] {
            std::thread::sleep(Duration::from_millis(delay));
            if frontend_ready(view) { break; }
            let ui_window = window.clone();
            let target = target.clone();
            let _ = window.run_on_main_thread(move || {
                if !frontend_ready(view) {
                    let _ = ui_window.navigate(target);
                }
            });
        }
    });
}
use windows::Win32::{
    Foundation::HWND,
    Graphics::Gdi::{CreateRoundRectRgn, SetWindowRgn},
    Graphics::Dwm::{DwmSetWindowAttribute, DWMWA_BORDER_COLOR, DWMWA_COLOR_NONE, DWMWA_WINDOW_CORNER_PREFERENCE, DWMWCP_DONOTROUND},
    UI::WindowsAndMessaging::{
        GetWindowLongPtrW, SetWindowLongPtrW, SetWindowPos, GWL_EXSTYLE, GWL_STYLE, SWP_FRAMECHANGED,
        SWP_NOACTIVATE, SWP_NOMOVE, SWP_NOSIZE, SWP_NOZORDER, WS_BORDER, WS_CAPTION, WS_DLGFRAME, WS_EX_CLIENTEDGE,
        WS_EX_DLGMODALFRAME, WS_EX_NOACTIVATE, WS_EX_STATICEDGE, WS_EX_WINDOWEDGE, WS_MAXIMIZEBOX,
        WS_MINIMIZEBOX, WS_SYSMENU, WS_THICKFRAME,
    },
};

/// Radio (px lógicos) de las esquinas de los paneles: Launchpad, paneles de la barra y buscador.
pub const PANEL_RADIUS: f64 = 16.0;

/// Recorta la ventana entera con una región redondeada: así ni el fondo de la ventana
/// ni el vidrio nativo dejan asomar un rectángulo detrás de la tarjeta, y no hay sombra
/// de DWM (el redondeo nativo de Windows 11 la añade y su radio es fijo de 8 px).
/// La región se recalcula en cada cambio de tamaño.
pub fn mark_rounded(window: &WebviewWindow) {
    enforce_chrome_free(window);
    let w = window.clone();
    let _ = window.run_on_main_thread(move || apply_round_region(&w));
    let w = window.clone();
    window.on_window_event(move |event| {
        if matches!(event, tauri::WindowEvent::Resized(_) | tauri::WindowEvent::ScaleFactorChanged { .. }) { apply_round_region(&w); }
    });
}

fn apply_round_region(window: &WebviewWindow) {
    let (Ok(hwnd), Ok(size), Ok(scale)) = (window.hwnd(), window.inner_size(), window.scale_factor()) else { return };
    // Algo menos redondeada que la tarjeta CSS: el borde visible es el suavizado del CSS,
    // no el escalonado de la región.
    let r = ((PANEL_RADIUS - 4.0) * scale * 2.0).round() as i32;
    unsafe {
        let region = CreateRoundRectRgn(0, 0, size.width as i32 + 1, size.height as i32 + 1, r, r);
        // El sistema pasa a ser dueño de la región: no se libera aquí.
        let _ = SetWindowRgn(hwnd, Some(region), true);
    }
}
/// Aplica el estilo de ventana sin marco en el hilo de UI.
///
/// Tao/WebView2 puede restaurar parte del estilo al crear la ventana o al mostrarla,
/// por eso los callers lo invocan al crearla y justo antes de cada show.
pub fn enforce_chrome_free(window: &WebviewWindow) {
    let w = window.clone();
    let _ = window.run_on_main_thread(move || {
        if let Ok(hwnd) = w.hwnd() {
            enforce_chrome_free_hwnd(hwnd);
        }
    });
}

/// Variante para una clausura que ya está ejecutándose en el hilo de UI.
pub fn enforce_chrome_free_hwnd(hwnd: HWND) {
    unsafe {
        let style = GetWindowLongPtrW(hwnd, GWL_STYLE);
        let standard = (WS_CAPTION | WS_THICKFRAME | WS_BORDER | WS_SYSMENU | WS_DLGFRAME | WS_MINIMIZEBOX | WS_MAXIMIZEBOX).0 as isize;
        SetWindowLongPtrW(hwnd, GWL_STYLE, style & !standard);

        let ex_style = GetWindowLongPtrW(hwnd, GWL_EXSTYLE);
        let framed = (WS_EX_CLIENTEDGE | WS_EX_DLGMODALFRAME | WS_EX_STATICEDGE | WS_EX_WINDOWEDGE | WS_EX_NOACTIVATE).0 as isize;
        SetWindowLongPtrW(hwnd, GWL_EXSTYLE, ex_style & !framed);

        // Sin redondeo de DWM: en Windows 11 trae una sombra gris y un radio fijo de 8 px.
        // Los paneles se recortan con una región propia (ver `mark_rounded`).
        let preference = DWMWCP_DONOTROUND;
        let _ = DwmSetWindowAttribute(hwnd, DWMWA_WINDOW_CORNER_PREFERENCE, &preference as *const _ as _, std::mem::size_of_val(&preference) as u32);
        let border = DWMWA_COLOR_NONE;
        let _ = DwmSetWindowAttribute(hwnd, DWMWA_BORDER_COLOR, &border as *const _ as _, std::mem::size_of_val(&border) as u32);

        let _ = SetWindowPos(
            hwnd,
            None,
            0,
            0,
            0,
            0,
            SWP_FRAMECHANGED | SWP_NOMOVE | SWP_NOSIZE | SWP_NOZORDER | SWP_NOACTIVATE,
        );
    }
}
