//! Desenfoque real del escritorio debajo de las formas de vidrio (Dock, barra, paneles, lanzador).
//!
//! Las ventanas WebView de Noki son 100 % transparentes: dibujan el tinte, el brillo y el borde con
//! CSS. El desenfoque lo pone **una sola ventana auxiliar por superficie**, colocada justo debajo de
//! la ventana visible, con el acento acrílico de Windows y las esquinas redondeadas y suavizadas por
//! DWM (`DWMWA_WINDOW_CORNER_PREFERENCE`). Sin clics, sin activarse y fuera de Alt+Tab.
//!
//! Antes la forma redondeada se aproximaba apilando ~20 ventanas rectangulares: cada esquina era una
//! escalera de 1 px y cada tira fina se difuminaba por separado (rayas horizontales y bases «sin
//! blur»). Se suponía que DWM no redondea las ventanas con acento acrílico; comprobado en Windows 11
//! que sí, con antialiasing. Por qué no el material del sistema (`DWMSBT_TRANSIENTWINDOW`):
//! comprobado también que en una ventana inactiva cae a un color plano sin desenfoque, y el Dock y
//! la barra nunca están activos. El precio de la superficie continua es que el radio lo fija Windows
//! (≈ 8 px lógicos): el CSS usa ese mismo radio en el modo nativo (`src/lib/glass.js`). Con el modo
//! «Vidrio suave sin blur nativo» no se crea ninguna ventana.

use std::{collections::{HashMap, HashSet}, sync::{Mutex, OnceLock}};
use tauri::{AppHandle, Emitter, Manager, Runtime, WebviewWindow, WindowEvent};
use windows::{
    core::w,
    Win32::{
        Foundation::{HWND, LPARAM, LRESULT, RECT, WPARAM},
        Graphics::Dwm::{
            DwmSetWindowAttribute, DWMWA_BORDER_COLOR, DWMWA_COLOR_NONE, DWMWA_WINDOW_CORNER_PREFERENCE, DWMWCP_DONOTROUND,
            DWMWCP_ROUND, DWMWCP_ROUNDSMALL, DWM_WINDOW_CORNER_PREFERENCE,
        },
        UI::{
            WindowsAndMessaging::{
                CreateWindowExW, DefWindowProcW, DestroyWindow, GetWindowRect, IsWindowVisible, RegisterClassExW, SendMessageW,
                SetWindowPos, HTTRANSPARENT, SWP_HIDEWINDOW, SWP_NOACTIVATE, SWP_NOMOVE, SWP_NOSIZE, SWP_NOZORDER, SWP_SHOWWINDOW,
                WM_NCACTIVATE, WM_NCHITTEST, WNDCLASSEXW, WS_EX_NOACTIVATE, WS_EX_TOOLWINDOW, WS_EX_TOPMOST, WS_EX_TRANSPARENT,
                WS_POPUP,
            },
        },
    },
};

#[link(name = "kernel32")]
extern "system" {
    fn GetModuleHandleW(name: *const u16) -> *mut core::ffi::c_void;
}

/// Forma de vidrio que informa cada vista, en px lógicos relativos a su ventana.
#[derive(Clone, Copy, PartialEq, Debug, Default)]
pub struct Shape { pub x: f64, pub y: f64, pub w: f64, pub h: f64, pub radius: f64 }

#[derive(Default)]
struct Entry {
    shape: Option<Shape>,
    /// Estilo de paneles (Ajustes > Apariencia): «oscuro opaco» = sin desenfoque.
    enabled: bool,
    /// Vidrio nativo de Windows (`true`) o solo CSS («Vidrio suave sin blur nativo»).
    native: bool,
    /// Material oscuro (modo noche) o claro.
    dark: bool,
    /// La ventana del desenfoque (una por superficie).
    piece: Option<isize>,
    /// Atributos aplicados a la pieza: (oscuro, esquinas).
    applied: Option<(bool, i32)>,
    /// Último rectángulo físico colocado (para no recolocar si nada cambió).
    placed: Option<RECT>,
    shown: bool,
    /// La vista ha pintado un fotograma estando visible (`glass_ready`): hasta entonces no hay vidrio.
    ready: bool,
    /// Cuándo se pidió esa confirmación (evento `glass-reveal`); se repite si no llega respuesta
    /// (antes se pedía una sola vez y, si la vista aún no escuchaba, el Dock se quedaba sin vidrio).
    reveal_at: Option<std::time::Instant>,
}

static ENTRIES: OnceLock<Mutex<HashMap<String, Entry>>> = OnceLock::new();
fn entries() -> &'static Mutex<HashMap<String, Entry>> { ENTRIES.get_or_init(|| Mutex::new(HashMap::new())) }

unsafe extern "system" fn wnd_proc(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    match msg {
        // Los clics atraviesan la pieza: los recibe la ventana de vidrio o lo que haya debajo.
        WM_NCHITTEST => LRESULT(HTTRANSPARENT as isize),
        // Siempre «activa» (nunca se activa de verdad: WS_EX_NOACTIVATE).
        WM_NCACTIVATE => DefWindowProcW(hwnd, msg, WPARAM(1), lparam),
        _ => DefWindowProcW(hwnd, msg, wparam, lparam),
    }
}

fn register_class() -> bool {
    static REGISTERED: OnceLock<bool> = OnceLock::new();
    *REGISTERED.get_or_init(|| unsafe {
        let class = WNDCLASSEXW {
            cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
            lpfnWndProc: Some(wnd_proc),
            hInstance: windows::Win32::Foundation::HINSTANCE(GetModuleHandleW(std::ptr::null())),
            lpszClassName: w!("NokiGlassBackdrop"),
            // Negro: con el acento acrílico, el negro deja ver el desenfoque.
            hbrBackground: windows::Win32::Graphics::Gdi::HBRUSH(windows::Win32::Graphics::Gdi::GetStockObject(windows::Win32::Graphics::Gdi::BLACK_BRUSH).0),
            ..Default::default()
        };
        RegisterClassExW(&class) != 0
    })
}

/// Tinte del acento acrílico (ABGR): gris medio uniforme como el de las capturas de referencia
/// (macdock-*.png); más oscuro y denso en modo noche. El tinte fino, el brillo y el borde, en CSS.
fn piece_tint(dark: bool) -> u32 {
    if dark { 0xA0_20_1C_1C } else { 0x88_54_50_50 }
}

/// Crea la ventana del desenfoque (en el hilo de UI). El acento se aplica en `apply_attributes`.
fn create_piece() -> Option<isize> {
    if !register_class() { return None; }
    unsafe {
        let hwnd = CreateWindowExW(
            WS_EX_TOOLWINDOW | WS_EX_NOACTIVATE | WS_EX_TRANSPARENT | WS_EX_TOPMOST,
            w!("NokiGlassBackdrop"),
            w!(""),
            WS_POPUP,
            0, 0, 1, 1,
            None, None,
            Some(windows::Win32::Foundation::HINSTANCE(GetModuleHandleW(std::ptr::null()))),
            None,
        ).ok()?;
        // Sin el borde de 1 px de DWM: el borde lo dibuja el CSS.
        let border = DWMWA_COLOR_NONE;
        let _ = DwmSetWindowAttribute(hwnd, DWMWA_BORDER_COLOR, &border as *const _ as _, std::mem::size_of_val(&border) as u32);
        let _ = SendMessageW(hwnd, WM_NCACTIVATE, Some(WPARAM(1)), Some(LPARAM(0)));
        Some(hwnd.0 as isize)
    }
}

/// Esquinas nativas según el radio pedido por la vista: sin redondeo (barra), pequeñas o normales.
fn corner_for(radius: f64) -> DWM_WINDOW_CORNER_PREFERENCE {
    if radius < 1.0 { DWMWCP_DONOTROUND } else if radius < 6.0 { DWMWCP_ROUNDSMALL } else { DWMWCP_ROUND }
}

fn apply_attributes(piece: HWND, dark: bool, corner: DWM_WINDOW_CORNER_PREFERENCE) {
    crate::window_utils::set_acrylic(piece, piece_tint(dark));
    unsafe { let _ = DwmSetWindowAttribute(piece, DWMWA_WINDOW_CORNER_PREFERENCE, &corner as *const _ as _, std::mem::size_of_val(&corner) as u32); }
}

fn hide_piece(entry: &mut Entry) {
    if !entry.shown { return; }
    if let Some(p) = entry.piece {
        unsafe { let _ = SetWindowPos(HWND(p as *mut _), None, 0, 0, 0, 0, SWP_HIDEWINDOW | SWP_NOACTIVATE | SWP_NOMOVE | SWP_NOSIZE | SWP_NOZORDER); }
    }
    entry.shown = false;
    entry.placed = None;
}

/// Recoloca (o esconde) el desenfoque de una ventana. Solo en el hilo de UI. La pieza va siempre
/// justo debajo de su ventana y nunca pasa por HWND_TOPMOST (un fotograma encima tapaba el Dock).
fn apply<R: Runtime>(window: &WebviewWindow<R>, label: &str) {
    let Ok(owner) = window.hwnd() else { return };
    let scale = window.scale_factor().unwrap_or(1.0);
    let mut reveal = false;
    {
        let mut map = entries().lock().unwrap();
        let Some(entry) = map.get_mut(label) else { return };
        let visible = unsafe { IsWindowVisible(owner).as_bool() };
        if !visible {
            // Oculta: al volver a mostrarse, el vidrio espera a que la vista haya pintado otra vez.
            entry.ready = false;
            entry.reveal_at = None;
        }
        let target = match (entry.enabled && entry.native && visible, entry.shape) {
            (true, Some(s)) if s.w > 1.0 && s.h > 1.0 => {
                let mut win = RECT::default();
                if unsafe { GetWindowRect(owner, &mut win) }.is_err() { None } else {
                    // Nunca fuera de su ventana (el redondeo a 125 % podía dar 1 px de más).
                    Some((RECT {
                        left: (win.left + (s.x * scale).round() as i32).max(win.left),
                        top: (win.top + (s.y * scale).round() as i32).max(win.top),
                        right: (win.left + ((s.x + s.w) * scale).round() as i32).min(win.right),
                        bottom: (win.top + ((s.y + s.h) * scale).round() as i32).min(win.bottom),
                    }, corner_for(s.radius)))
                }
            }
            _ => None,
        };
        let Some((rect, corner)) = target else { hide_piece(entry); return };
        if !entry.ready {
            // Visible pero sin un fotograma pintado confirmado: sin vidrio (si no, se veía medio
            // segundo un recuadro gris vacío al abrir el lanzador). Se pide la confirmación una vez.
            hide_piece(entry);
            if entry.reveal_at.is_none_or(|at| at.elapsed() >= std::time::Duration::from_millis(400)) {
                entry.reveal_at = Some(std::time::Instant::now());
                reveal = true;
            }
        } else {
            if entry.piece.is_none() { entry.piece = create_piece(); }
            let Some(p) = entry.piece else { return };
            let piece = HWND(p as *mut _);
            if entry.applied != Some((entry.dark, corner.0)) {
                apply_attributes(piece, entry.dark, corner);
                entry.applied = Some((entry.dark, corner.0));
            }
            unsafe {
                if entry.shown && entry.placed == Some(rect) {
                    // Solo reafirmar el orden: justo debajo de la ventana visible.
                    let _ = SetWindowPos(piece, Some(owner), 0, 0, 0, 0, SWP_NOACTIVATE | SWP_NOMOVE | SWP_NOSIZE);
                } else {
                    let _ = SetWindowPos(piece, Some(owner), rect.left, rect.top, rect.right - rect.left, rect.bottom - rect.top, SWP_NOACTIVATE | SWP_SHOWWINDOW);
                    entry.shown = true;
                    entry.placed = Some(rect);
                }
            }
        }
    }
    if reveal { let _ = window.emit("glass-reveal", label); }
}

fn schedule<R: Runtime>(app: &AppHandle<R>, label: &str) {
    let Some(window) = app.get_webview_window(label) else { return };
    let w = window.clone();
    let label = label.to_string();
    let _ = window.run_on_main_thread(move || apply(&w, &label));
}

/// Recoloca el vidrio en cuanto su ventana se mueve, cambia de tamaño o de escala.
fn follow_window(window: &WebviewWindow, label: &str) {
    static FOLLOWED: OnceLock<Mutex<HashSet<String>>> = OnceLock::new();
    if !FOLLOWED.get_or_init(|| Mutex::new(HashSet::new())).lock().unwrap().insert(label.to_string()) { return; }
    let app = window.app_handle().clone();
    let label = label.to_string();
    window.on_window_event(move |event| {
        if matches!(event, WindowEvent::Moved(_) | WindowEvent::Resized(_) | WindowEvent::ScaleFactorChanged { .. }) {
            schedule(&app, &label);
        }
    });
}

/// Activa o desactiva el desenfoque de una ventana (p. ej. estilo oscuro opaco = sin desenfoque).
pub fn set_enabled<R: Runtime>(app: &AppHandle<R>, label: &str, enabled: bool) {
    {
        let mut map = entries().lock().unwrap();
        let entry = map.entry(label.to_string()).or_insert_with(|| Entry { native: true, ..Default::default() });
        if entry.enabled == enabled { return; }
        entry.enabled = enabled;
    }
    schedule(app, label);
}

/// La vista informa de su forma de vidrio (px lógicos relativos a su ventana), del modo (noche o
/// día) y de si quiere el desenfoque nativo o solo el vidrio CSS.
#[tauri::command]
pub fn glass_shape(window: WebviewWindow, x: f64, y: f64, w: f64, h: f64, radius: f64, dark: Option<bool>, native: Option<bool>) {
    let label = window.label().to_string();
    let shape = Shape { x, y, w, h, radius };
    let (dark, native) = (dark.unwrap_or(false), native.unwrap_or(true));
    let stale = {
        let mut map = entries().lock().unwrap();
        let entry = map.entry(label.clone()).or_insert_with(|| Entry { enabled: true, native: true, ..Default::default() });
        let unchanged = entry.shape == Some(shape) && entry.dark == dark && entry.native == native;
        entry.shape = Some(shape);
        entry.dark = dark;
        entry.native = native;
        // Sin desenfoque nativo, la ventana auxiliar sobra: se destruye.
        let stale = if native { None } else { entry.shown = false; entry.placed = None; entry.applied = None; entry.piece.take() };
        if unchanged && stale.is_none() && (entry.shown || !native) { return; }
        stale
    };
    follow_window(&window, &label);
    let w = window.clone();
    let _ = window.run_on_main_thread(move || {
        if let Some(p) = stale { unsafe { let _ = DestroyWindow(HWND(p as *mut _)); } }
        apply(&w, &label);
    });
}

/// La vista confirma que ya ha pintado un fotograma estando visible: desde ahora puede verse su vidrio.
#[tauri::command]
pub fn glass_ready(window: WebviewWindow) {
    let label = window.label().to_string();
    {
        let mut map = entries().lock().unwrap();
        // Puede llegar antes que la primera forma: se recuerda igualmente.
        let entry = map.entry(label.clone()).or_insert_with(|| Entry { enabled: true, native: true, ..Default::default() });
        entry.ready = true;
        entry.reveal_at = None;
    }
    let w = window.clone();
    let _ = window.run_on_main_thread(move || apply(&w, &label));
}

/// Sigue a las ventanas visibles: si se muestran o se esconden, el desenfoque las acompaña (los
/// movimientos llegan al momento por `follow_window`). Sondeo ligero cada 120 ms.
pub fn start(app: AppHandle) {
    std::thread::spawn(move || {
        let mut last: HashMap<String, (bool, RECT)> = HashMap::new();
        let mut tick = 0u32;
        loop {
            std::thread::sleep(std::time::Duration::from_millis(120));
            // Cada ~1 s se reafirma el orden (justo debajo de su ventana) aunque nada haya cambiado.
            tick = tick.wrapping_add(1);
            if tick % 8 == 0 { last.clear(); }
            let labels: Vec<String> = entries().lock().unwrap().keys().cloned().collect();
            for label in labels {
                let Some(window) = app.get_webview_window(&label) else { continue };
                let Ok(h) = window.hwnd() else { continue };
                let mut rect = RECT::default();
                let visible = unsafe { IsWindowVisible(h).as_bool() && GetWindowRect(h, &mut rect).is_ok() };
                let state = (visible, rect);
                if last.get(&label) == Some(&state) { continue; }
                last.insert(label.clone(), state);
                schedule(&app, &label);
            }
        }
    });
}
