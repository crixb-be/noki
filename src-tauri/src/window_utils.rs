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

/// Reintenta la ruta de la aplicación si la vista no llega a pintar. Antes recargaba a los 120, 370
/// y 870 ms: una WebView2 nueva (barra, lanzador, buscador y panel se crean casi a la vez) tarda más,
/// y cada recarga abortaba la carga anterior (la barra se quedaba vacía al arrancar). Ahora solo
/// recarga tras 2,5 s sin pintar. El evento `frontend-ready` detiene la secuencia.
pub fn schedule_app_recovery(window: &WebviewWindow, view: &'static str) {
    let Some(target) = window.url().ok() else { return };
    let window = window.clone();
    std::thread::spawn(move || {
        for delay in [2_500u64, 4_000, 8_000] {
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
    Graphics::Gdi::SetWindowRgn,
    Graphics::Dwm::{DwmSetWindowAttribute, DWMWA_BORDER_COLOR, DWMWA_COLOR_NONE, DWMWA_WINDOW_CORNER_PREFERENCE, DWMWCP_DONOTROUND},
    UI::WindowsAndMessaging::{GetWindowLongPtrW, SetWindowLongPtrW, GWL_EXSTYLE, WS_EX_TOOLWINDOW},
};

/// Ventana emergente transparente (sin efectos de Windows): sin acento y con el aspecto nativo de `enforce_chrome_free_hwnd`.
pub fn mark_rounded<R: tauri::Runtime>(window: &tauri::WebviewWindow<R>) {
    let w = window.clone();
    let _ = window.run_on_main_thread(move || {
        if let Ok(h) = w.hwnd() {
            unsafe { let _ = SetWindowRgn(h, None, true); }
            set_acrylic(h, 0); // sin acento: la ventana visible es 100 % transparente
            enforce_chrome_free_hwnd(h);
        }
    });
}

#[repr(C)]
struct AccentPolicy { state: u32, flags: u32, gradient: u32, animation: u32 }
#[repr(C)]
struct CompositionData { attribute: u32, data: *mut core::ffi::c_void, size: usize }
// SetWindowCompositionAttribute no está en la biblioteca de importación de user32:
// se resuelve en tiempo de ejecución.
#[link(name = "kernel32")]
extern "system" {
    fn GetModuleHandleW(name: *const u16) -> *mut core::ffi::c_void;
    fn GetProcAddress(module: *mut core::ffi::c_void, name: *const u8) -> *mut core::ffi::c_void;
}
type SetWca = unsafe extern "system" fn(HWND, *mut CompositionData) -> i32;

fn set_wca() -> Option<SetWca> {
    static PROC: OnceLock<usize> = OnceLock::new();
    let address = *PROC.get_or_init(|| unsafe {
        let name: Vec<u16> = "user32.dll\0".encode_utf16().collect();
        let module = GetModuleHandleW(name.as_ptr());
        if module.is_null() { 0 } else { GetProcAddress(module, b"SetWindowCompositionAttribute\0".as_ptr()) as usize }
    });
    (address != 0).then(|| unsafe { std::mem::transmute::<usize, SetWca>(address) })
}

const WCA_ACCENT_POLICY: u32 = 19;
const ACCENT_DISABLED: u32 = 0;
const ACCENT_ENABLE_ACRYLICBLURBEHIND: u32 = 4;

/// Acento acrílico (tinte ABGR) o ninguno si `tint == 0`. Solo para las piezas de `backdrop`.
pub fn set_acrylic(hwnd: HWND, tint: u32) {
    let mut policy = AccentPolicy {
        state: if tint == 0 { ACCENT_DISABLED } else { ACCENT_ENABLE_ACRYLICBLURBEHIND },
        flags: 2,
        gradient: tint,
        animation: 0,
    };
    let mut data = CompositionData { attribute: WCA_ACCENT_POLICY, data: &mut policy as *mut _ as _, size: std::mem::size_of::<AccentPolicy>() };
    if let Some(set) = set_wca() { unsafe { let _ = set(hwnd, &mut data); } }
}

/// Etiquetas de las ventanas de panel.
pub const PANEL_WINDOWS: [&str; 3] = ["barpanel", "launchpad", "spotlight"];

/// Estilo de paneles: «light» = desenfoque real debajo (ventanas auxiliares); «dark» = sin desenfoque.
pub fn sync_panel_glass<R: tauri::Runtime>(app: &tauri::AppHandle<R>, panel_style: &str) {
    let light = panel_style != "dark";
    for label in PANEL_WINDOWS { crate::backdrop::set_enabled(app, label, light); }
}

/// Aspecto nativo de las ventanas de Noki. Solo en el hilo de UI y siempre *después* de show/hide/
/// set_ignore_cursor_events: tao (window_state.rs, `apply_diff`) reescribe GWL_STYLE y GWL_EXSTYLE enteros
/// en cada cambio de sus flags. Su WM_NCCALCSIZE ya deja la ventana sin área no cliente, así que el estilo
/// de ventana (WS_CAPTION o WS_POPUP) no se toca: forzarlo solo provocaba una guerra de estilos con
/// SWP_FRAMECHANGED en cada vuelta. Aquí solo va lo que tao no gestiona, y es idempotente.
pub fn enforce_chrome_free_hwnd(hwnd: HWND) {
    unsafe {
        // Fuera de Alt+Tab (tao borra este bit en cada reescritura de GWL_EXSTYLE).
        let ex_style = GetWindowLongPtrW(hwnd, GWL_EXSTYLE);
        let tool = WS_EX_TOOLWINDOW.0 as isize;
        if ex_style & tool == 0 {
            SetWindowLongPtrW(hwnd, GWL_EXSTYLE, ex_style | tool);
        }
        // Sin redondeo ni borde de DWM: en Windows 11 traen una sombra gris y un radio fijo de 8 px.
        // Son atributos de DWM: sobreviven a los cambios de estilo y no necesitan SWP_FRAMECHANGED.
        let preference = DWMWCP_DONOTROUND;
        let _ = DwmSetWindowAttribute(hwnd, DWMWA_WINDOW_CORNER_PREFERENCE, &preference as *const _ as _, std::mem::size_of_val(&preference) as u32);
        let border = DWMWA_COLOR_NONE;
        let _ = DwmSetWindowAttribute(hwnd, DWMWA_BORDER_COLOR, &border as *const _ as _, std::mem::size_of_val(&border) as u32);
    }
}
