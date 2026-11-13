//! Desenfoque real del escritorio debajo de las formas de vidrio (Dock, barra, paneles, lanzador).
//!
//! Las ventanas WebView de Noki son 100 % transparentes y sin efectos de Windows: dibujan el tinte,
//! el brillo y el borde con CSS. El desenfoque lo ponen ventanas auxiliares propias, colocadas justo
//! debajo de cada ventana visible, sin clics (WS_EX_TRANSPARENT + HTTRANSPARENT), sin activarse y
//! fuera de Alt+Tab, con el acento acrílico de Windows.
//!
//! El acrílico ignora las regiones de ventana y el redondeo nativo de Windows 11 es de radio fijo con
//! sombra, así que la forma redondeada se aproxima con un rectángulo central y franjas finas cuya
//! anchura sigue el arco de cada esquina; las franjas quedan siempre DENTRO de la forma, de modo que
//! fuera de ella no se ve nada. (Técnica descrita en MacDock; implementación propia.)

use std::{collections::HashMap, sync::{Mutex, OnceLock}};
use tauri::{AppHandle, Manager, Runtime, WebviewWindow};
use windows::{
    core::w,
    Win32::{
        Foundation::{HWND, LPARAM, LRESULT, RECT, WPARAM},
        Graphics::Dwm::{DwmSetWindowAttribute, DWMWA_BORDER_COLOR, DWMWA_COLOR_NONE, DWMWA_WINDOW_CORNER_PREFERENCE, DWMWCP_DONOTROUND},
        UI::WindowsAndMessaging::{
            CreateWindowExW, DefWindowProcW, GetWindowRect, IsWindowVisible, RegisterClassExW, SetWindowPos, HTTRANSPARENT,
            HWND_TOPMOST, SWP_HIDEWINDOW, SWP_NOACTIVATE, SWP_NOMOVE, SWP_NOSIZE, SWP_NOZORDER, SWP_SHOWWINDOW, WM_NCACTIVATE,
            WM_NCHITTEST, WNDCLASSEXW, WS_EX_NOACTIVATE, WS_EX_TOOLWINDOW, WS_EX_TOPMOST, WS_EX_TRANSPARENT, WS_POPUP,
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
    enabled: bool,
    /// HWND de las piezas: [cuerpo, franjas...]
    pieces: Vec<isize>,
    /// Último rectángulo físico y radio colocados (para no recolocar si nada cambió).
    placed: Option<(RECT, i32)>,
    shown: bool,
}

static ENTRIES: OnceLock<Mutex<HashMap<String, Entry>>> = OnceLock::new();
fn entries() -> &'static Mutex<HashMap<String, Entry>> { ENTRIES.get_or_init(|| Mutex::new(HashMap::new())) }

unsafe extern "system" fn wnd_proc(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    match msg {
        // Los clics atraviesan la pieza: los recibe la ventana de vidrio o lo que haya debajo.
        WM_NCHITTEST => LRESULT(HTTRANSPARENT as isize),
        // Siempre «activa»: si no, el acrílico cae a un relleno plano al perder el foco.
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
            ..Default::default()
        };
        RegisterClassExW(&class) != 0
    })
}

/// Crea una pieza (en el hilo de UI).
fn create_piece(tint: u32) -> Option<isize> {
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
        let corner = DWMWCP_DONOTROUND;
        let _ = DwmSetWindowAttribute(hwnd, DWMWA_WINDOW_CORNER_PREFERENCE, &corner as *const _ as _, std::mem::size_of_val(&corner) as u32);
        let border = DWMWA_COLOR_NONE;
        let _ = DwmSetWindowAttribute(hwnd, DWMWA_BORDER_COLOR, &border as *const _ as _, std::mem::size_of_val(&border) as u32);
        crate::window_utils::set_acrylic(hwnd, tint);
        Some(hwnd.0 as isize)
    }
}

/// Tinte del acrílico de las piezas: casi transparente; el tinte visible lo pinta el CSS encima.
const PIECE_TINT: u32 = 0x10_FF_FF_FF;

/// Rectángulos (físicos) que cubren la forma redondeada sin salirse de ella.
fn pieces_for(rect: RECT, r: i32) -> Vec<RECT> {
    let (w, h) = (rect.right - rect.left, rect.bottom - rect.top);
    let r = r.min(w / 2).min(h / 2).max(0);
    if r < 2 { return vec![rect]; }
    let mut out = vec![RECT { left: rect.left, top: rect.top + r, right: rect.right, bottom: rect.bottom - r }];
    // Hueco del arco a la altura y (desde el borde): cuánto hay que meter la franja hacia dentro.
    let inset = |y: i32| -> f64 { let dy = (r - y) as f64; r as f64 - ((r * r) as f64 - dy * dy).max(0.0).sqrt() };
    // Franjas adaptativas: cada una termina cuando el arco se ha desplazado ≤ 1,5 px, así que son
    // finas junto al vértice (donde la curva cambia rápido) y gruesas donde es casi vertical.
    // Se usa el hueco del borde más cercano al vértice: la franja nunca sobresale de la forma.
    let mut y0 = 0;
    while y0 < r {
        let start = inset(y0);
        let mut y1 = y0 + 1;
        while y1 < r && start - inset(y1) <= 1.5 { y1 += 1; }
        let i = start.ceil() as i32;
        out.push(RECT { left: rect.left + i, top: rect.top + y0, right: rect.right - i, bottom: rect.top + y1 });
        out.push(RECT { left: rect.left + i, top: rect.bottom - y1, right: rect.right - i, bottom: rect.bottom - y0 });
        y0 = y1;
    }    out
}

/// Recoloca (o esconde) las piezas de una ventana. Solo en el hilo de UI.
fn apply<R: Runtime>(window: &WebviewWindow<R>, label: &str) {
    let Ok(owner) = window.hwnd() else { return };
    let scale = window.scale_factor().unwrap_or(1.0);
    let mut map = entries().lock().unwrap();
    let Some(entry) = map.get_mut(label) else { return };
    let visible = unsafe { IsWindowVisible(owner).as_bool() };
    let target = match (entry.enabled && visible, entry.shape) {
        (true, Some(s)) if s.w > 1.0 && s.h > 1.0 => {
            let mut win = RECT::default();
            if unsafe { GetWindowRect(owner, &mut win) }.is_err() { None } else {
                let rect = RECT {
                    left: win.left + (s.x * scale).round() as i32,
                    top: win.top + (s.y * scale).round() as i32,
                    right: win.left + ((s.x + s.w) * scale).round() as i32,
                    bottom: win.top + ((s.y + s.h) * scale).round() as i32,
                };
                Some((rect, (s.radius * scale).round() as i32))
            }
        }
        _ => None,
    };
    let Some((rect, radius)) = target else {
        if entry.shown {
            for &p in &entry.pieces { unsafe { let _ = SetWindowPos(HWND(p as *mut _), None, 0, 0, 0, 0, SWP_HIDEWINDOW | SWP_NOACTIVATE | SWP_NOMOVE | SWP_NOSIZE | SWP_NOZORDER); } }
            entry.shown = false;
            entry.placed = None;
        }
        return;
    };
    if entry.shown && entry.placed == Some((rect, radius)) {
        // Solo reafirmar el orden: justo debajo de la ventana visible.
        for &p in &entry.pieces { unsafe { let _ = SetWindowPos(HWND(p as *mut _), Some(owner), 0, 0, 0, 0, SWP_NOACTIVATE | SWP_NOMOVE | SWP_NOSIZE); } }
        return;
    }
    let rects = pieces_for(rect, radius);
    while entry.pieces.len() < rects.len() {
        let Some(p) = create_piece(PIECE_TINT) else { return };
        entry.pieces.push(p);
    }
    for (i, &p) in entry.pieces.iter().enumerate() {
        let h = HWND(p as *mut _);
        unsafe {
            if let Some(r) = rects.get(i) {
                let _ = SetWindowPos(h, Some(HWND_TOPMOST), r.left, r.top, r.right - r.left, r.bottom - r.top, SWP_NOACTIVATE | SWP_SHOWWINDOW);
                let _ = SetWindowPos(h, Some(owner), r.left, r.top, r.right - r.left, r.bottom - r.top, SWP_NOACTIVATE | SWP_SHOWWINDOW);
            } else {
                let _ = SetWindowPos(h, None, 0, 0, 0, 0, SWP_HIDEWINDOW | SWP_NOACTIVATE | SWP_NOMOVE | SWP_NOSIZE | SWP_NOZORDER);
            }
        }
    }
    entry.shown = true;
    entry.placed = Some((rect, radius));
}

fn schedule<R: Runtime>(app: &AppHandle<R>, label: &str) {
    let Some(window) = app.get_webview_window(label) else { return };
    let w = window.clone();
    let label = label.to_string();
    let _ = window.run_on_main_thread(move || apply(&w, &label));
}

/// Activa o desactiva el desenfoque de una ventana (p. ej. estilo oscuro opaco = sin desenfoque).
pub fn set_enabled<R: Runtime>(app: &AppHandle<R>, label: &str, enabled: bool) {
    {
        let mut map = entries().lock().unwrap();
        let entry = map.entry(label.to_string()).or_default();
        if entry.enabled == enabled { return; }
        entry.enabled = enabled;
    }
    schedule(app, label);
}

/// La vista informa de su forma de vidrio (px lógicos relativos a su ventana).
#[tauri::command]
pub fn glass_shape(window: WebviewWindow, x: f64, y: f64, w: f64, h: f64, radius: f64) {
    let label = window.label().to_string();
    let shape = Shape { x, y, w, h, radius };
    {
        let mut map = entries().lock().unwrap();
        let entry = map.entry(label.clone()).or_insert_with(|| Entry { enabled: true, ..Default::default() });
        if entry.shape == Some(shape) && entry.shown { return; }
        entry.shape = Some(shape);
    }
    let w = window.clone();
    let _ = window.run_on_main_thread(move || apply(&w, &label));
}

/// Sigue a las ventanas visibles: si se mueven, se esconden o se muestran, las piezas las acompañan.
/// Sondeo ligero (GetWindowRect/IsWindowVisible no envían mensajes) cada 120 ms.
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
