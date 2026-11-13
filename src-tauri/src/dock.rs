//! Dock inferior flotante y operaciones nativas de sus elementos.
//!
//! La ventana se mantiene pequeña y transparente; el contenido React comunica el rectángulo real
//! de la píldora para que el sondeo pueda dejar pasar los clics fuera de ella.

use crate::{icons, notch, settings::{DockPinned, Settings}, window_utils};
use serde::Serialize;
use std::{path::Path, sync::{Mutex, OnceLock}, time::{Duration, Instant}};
use tauri::{window::Color, AppHandle, Emitter, Manager, PhysicalPosition, PhysicalSize, WebviewUrl, WebviewWindow, WebviewWindowBuilder};
use windows::{core::{w, PCWSTR}, Win32::{Foundation::{HWND, LPARAM, RECT}, Graphics::{Gdi::{GetMonitorInfoW, MonitorFromWindow, MONITORINFO, MONITOR_DEFAULTTONEAREST}}, UI::{Shell::{SHAppBarMessage, SHEmptyRecycleBinW, SHFileOperationW, SHQueryRecycleBinW, APPBARDATA, SHFILEOPSTRUCTW, FO_DELETE, FOF_ALLOWUNDO, FOF_NOCONFIRMATION, FOF_NOERRORUI, FOF_SILENT}, WindowsAndMessaging::{EnumChildWindows, EnumWindows, FindWindowExW, FindWindowW, GetClassNameW, GetWindowRect, GetAncestor, GetCursorPos, GetWindowLongPtrW, GetWindowTextLengthW, GetWindowTextW, GetWindowThreadProcessId, GetForegroundWindow, IsIconic, IsWindowVisible, SetForegroundWindow, SetWindowPos, ShowWindow, GA_ROOT, GWL_EXSTYLE, HWND_TOPMOST, SW_MINIMIZE, SW_RESTORE, SW_HIDE, SW_SHOW, SW_SHOWNA, SWP_ASYNCWINDOWPOS, SWP_NOACTIVATE, SWP_NOMOVE, SWP_NOSIZE, WS_EX_TOOLWINDOW}}}};

const ABM_NEW: u32 = 0;
const ABM_REMOVE: u32 = 1;
const ABM_QUERYPOS: u32 = 2;
const ABM_SETPOS: u32 = 3;
const ABE_BOTTOM: u32 = 3;
const ABM_GETSTATE: u32 = 4;
const ABM_SETSTATE: u32 = 10;
const ABS_AUTOHIDE: u32 = 1;

#[derive(Default, Clone, Copy)]
struct Hit { x: i32, y: i32, w: i32, h: i32, scale: f64 }

#[derive(Clone, Copy)]
struct AppBarState { hwnd: isize, rect: RECT, monitor: notch::Rect, height: i32, dpi_milli: i32 }

static HIT: OnceLock<Mutex<Hit>> = OnceLock::new();
static APPBAR: OnceLock<Mutex<Option<AppBarState>>> = OnceLock::new();
static SIZE: OnceLock<Mutex<Option<(i32, i32)>>> = OnceLock::new();
static TASKBAR_STATE: OnceLock<Mutex<Option<u32>>> = OnceLock::new();

fn hit() -> &'static Mutex<Hit> { HIT.get_or_init(|| Mutex::new(Hit { scale: 1.0, ..Hit::default() })) }
fn appbar() -> &'static Mutex<Option<AppBarState>> { APPBAR.get_or_init(|| Mutex::new(None)) }
fn requested_size() -> &'static Mutex<Option<(i32, i32)>> { SIZE.get_or_init(|| Mutex::new(None)) }
fn taskbar_state() -> &'static Mutex<Option<u32>> { TASKBAR_STATE.get_or_init(|| Mutex::new(None)) }

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DockItem {
    pub key: String,
    pub name: String,
    pub path: String,
    pub args: String,
    pub aumid: String,
    pub running: bool,
    pub window_count: usize,
    pub kind: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TrashState { pub items: u32, pub bytes: u64 }

fn physical(v: i32, scale: f64) -> i32 { (v as f64 * scale).round() as i32 }

fn hwnd(window: &WebviewWindow) -> Option<HWND> { window.hwnd().ok() }


/// Zona de trabajo real del monitor de `main`, no el rectángulo que incluye la barra de tareas.
pub(crate) fn work_area(app: &AppHandle) -> Option<(notch::Rect, f64)> {
    work_area_with_monitor(app, false)
}

pub(crate) fn work_area_with_monitor(app: &AppHandle, full_monitor: bool) -> Option<(notch::Rect, f64)> {
    let main = app.get_webview_window("main")?;
    let main_hwnd = hwnd(&main)?;
    unsafe {
        let monitor = MonitorFromWindow(main_hwnd, MONITOR_DEFAULTTONEAREST);
        if monitor.0.is_null() { return None; }
        let mut info = MONITORINFO { cbSize: std::mem::size_of::<MONITORINFO>() as u32, ..Default::default() };
        if !GetMonitorInfoW(monitor, &mut info).as_bool() { return None; }
        let scale = notch::monitors(app).into_iter().find(|m| m.rect.left == info.rcMonitor.left && m.rect.top == info.rcMonitor.top).map(|m| m.scale).unwrap_or(1.0);
        let r = if full_monitor { info.rcMonitor } else { info.rcWork };
        Some((notch::Rect { left: r.left, top: r.top, right: r.right, bottom: r.bottom }, scale))
    }
}

/// Ventanas de la barra de tareas de Windows (principal y de otros monitores).
fn taskbars() -> Vec<HWND> {
    let mut out = Vec::new();
    unsafe {
        if let Ok(h) = FindWindowW(w!("Shell_TrayWnd"), PCWSTR::null()) { out.push(h); }
        let mut after: Option<HWND> = None;
        while let Ok(h) = FindWindowExW(None, after, w!("Shell_SecondaryTrayWnd"), PCWSTR::null()) {
            if h.0.is_null() { break; }
            out.push(h);
            after = Some(h);
        }
    }
    out
}

/// Oculta de verdad la barra de tareas cuando la opción está activa. Solo con
/// autoocultar, Windows la volvía a mostrar al acercar el ratón al borde inferior
/// (justo donde está el Dock). Se llama cada 2 s: si Explorer la vuelve a mostrar, se oculta otra vez.
pub fn sync_taskbar(settings: &Settings) {
    // Nunca durante un recorte de pantalla: cambiar el AppBar de la barra de tareas mueve el área
    // de trabajo y la superposición del recorte salta o se cierra.
    if capture_overlay_active() { return; }
    let Some(taskbar) = (unsafe { FindWindowW(w!("Shell_TrayWnd"), PCWSTR::null()) }).ok() else { return };
    let mut data = APPBARDATA { cbSize: std::mem::size_of::<APPBARDATA>() as u32, hWnd: taskbar, ..Default::default() };
    unsafe {
        let mut previous = taskbar_state().lock().unwrap();
        if settings.dock.enabled && settings.dock.hide_taskbar && settings.general.visible {
            if previous.is_none() {
                *previous = Some(SHAppBarMessage(ABM_GETSTATE, &mut data) as u32);
            }
            // Autoocultar libera la zona de trabajo; SW_HIDE evita que reaparezca.
            if SHAppBarMessage(ABM_GETSTATE, &mut data) as u32 & ABS_AUTOHIDE == 0 {
                data.lParam = LPARAM(ABS_AUTOHIDE as isize);
                let _ = SHAppBarMessage(ABM_SETSTATE, &mut data);
            }
            for h in taskbars() { if IsWindowVisible(h).as_bool() { let _ = ShowWindow(h, SW_HIDE); } }
        } else if let Some(state) = previous.take() {
            for h in taskbars() { let _ = ShowWindow(h, SW_SHOWNA); }
            data.lParam = LPARAM(state as isize);
            let _ = SHAppBarMessage(ABM_SETSTATE, &mut data);
        }
    }
}

/// ¿Hay una herramienta de captura de Windows en marcha (Win+Shift+S, Recortes)? Su superposición
/// es una ventana TOPMOST a pantalla completa: no hay que confundirla con un juego o un vídeo ni
/// subir ninguna ventana de Noki por encima de ella.
pub(crate) fn capture_overlay_active() -> bool {
    let active = capture_overlay_now();
    if active { *last_capture().lock().unwrap() = Some(Instant::now()); }
    active
}

fn last_capture() -> &'static Mutex<Option<Instant>> {
    static LAST: OnceLock<Mutex<Option<Instant>>> = OnceLock::new();
    LAST.get_or_init(|| Mutex::new(None))
}

/// ¿Se está lanzando o acaba de usarse una captura? Atajo pulsado ahora mismo (Win+Shift+S o
/// Impr Pant, que en Windows 11 abre el recorte), superposición visible, o vista hace menos de 1,5 s
/// (el foco salta antes de que la superposición exista y vuelve un instante después de cerrarla).
pub(crate) fn capture_in_progress() -> bool {
    use windows::Win32::UI::Input::KeyboardAndMouse::{GetAsyncKeyState, VK_LWIN, VK_RWIN, VK_SHIFT, VK_SNAPSHOT};
    let down = |k: u16| unsafe { GetAsyncKeyState(k as i32) < 0 };
    let hotkey = down(VK_SNAPSHOT.0) || ((down(VK_LWIN.0) || down(VK_RWIN.0)) && down(VK_SHIFT.0) && down(b'S' as u16));
    if hotkey { *last_capture().lock().unwrap() = Some(Instant::now()); return true; }
    capture_overlay_active() || last_capture().lock().unwrap().is_some_and(|at| at.elapsed() < Duration::from_millis(1500))
}

fn capture_overlay_now() -> bool {
    const CAPTURE_CLASSES: [&str; 2] = ["Microsoft-Windows-SnipperToolbar", "Microsoft-Windows-SnipperCaptureForm"];
    unsafe {
        for class in CAPTURE_CLASSES {
            let wide: Vec<u16> = class.encode_utf16().chain(std::iter::once(0)).collect();
            if let Ok(h) = FindWindowW(PCWSTR(wide.as_ptr()), PCWSTR::null()) {
                if !h.0.is_null() && IsWindowVisible(h).as_bool() { return true; }
            }
        }
        let fg = GetForegroundWindow();
        if fg.0.is_null() { return false; }
        let mut buf = [0u16; 128];
        let n = GetClassNameW(fg, &mut buf);
        let class = String::from_utf16_lossy(&buf[..n.max(0) as usize]);
        if CAPTURE_CLASSES.contains(&class.as_str()) { return true; }
        let mut pid = 0u32;
        GetWindowThreadProcessId(fg, Some(&mut pid));
        let process = file_name(&icons::process_image_path(pid).unwrap_or_default());
        // Incluye las ventanas emergentes XAML (Xaml_WindowedPopupClass) de esos mismos procesos.
        matches!(process.as_str(), "screenclippinghost.exe" | "snippingtool.exe")
    }
}

pub fn restore_taskbar() {
    let Some(taskbar) = (unsafe { FindWindowW(w!("Shell_TrayWnd"), PCWSTR::null()) }).ok() else { return };
    let Some(state) = taskbar_state().lock().unwrap().take() else { return };
    unsafe { for h in taskbars() { let _ = ShowWindow(h, SW_SHOWNA); } }
    let mut data = APPBARDATA { cbSize: std::mem::size_of::<APPBARDATA>() as u32, hWnd: taskbar, lParam: LPARAM(state as isize), ..Default::default() };
    unsafe { let _ = SHAppBarMessage(ABM_SETSTATE, &mut data); }
}
fn create(app: &AppHandle) -> Option<WebviewWindow> {
    if let Some(w) = app.get_webview_window("dock") { return Some(w); }
    let window = WebviewWindowBuilder::new(app, "dock", WebviewUrl::App("index.html?view=dock".into()))
        .title("").inner_size(1200.0, 84.0).decorations(false).transparent(true).background_color(Color(0, 0, 0, 0))
        .always_on_top(true).resizable(false).skip_taskbar(true).shadow(false).focused(false).visible(false)
        // No activable: tao convierte esto en WS_EX_NOACTIVATE dentro de sus propios flags, así que
        // sobrevive a cada show/hide/set_ignore_cursor_events (antes se ponía a mano y tao lo borraba).
        .focusable(false)
        .build().ok()?;
    let _ = window.set_background_color(Some(Color(0, 0, 0, 0)));
    Some(window)
}

fn remove_appbar() {
    let Some(state) = appbar().lock().unwrap().take() else { return };
    let mut data = APPBARDATA { cbSize: std::mem::size_of::<APPBARDATA>() as u32, hWnd: HWND(state.hwnd as *mut _), ..Default::default() };
    unsafe { let _ = SHAppBarMessage(ABM_REMOVE, &mut data); }
}

fn register_appbar(window: &WebviewWindow, monitor: notch::Rect, height: i32, scale: f64) -> Option<notch::Rect> {
    let Some(h) = hwnd(window) else { return None };
    let key = (h.0 as isize, monitor, height, (scale * 1000.0).round() as i32);
    if let Some(state) = appbar().lock().unwrap().as_ref().copied() {
        if (state.hwnd, state.monitor, state.height, state.dpi_milli) == key {
            return Some(notch::Rect { left: state.rect.left, top: state.rect.top, right: state.rect.right, bottom: state.rect.bottom });
        }
    }
    remove_appbar();
    let mut data = APPBARDATA { cbSize: std::mem::size_of::<APPBARDATA>() as u32, hWnd: h, uEdge: ABE_BOTTOM, rc: RECT { left: monitor.left, top: monitor.bottom - height, right: monitor.right, bottom: monitor.bottom }, ..Default::default() };
    unsafe {
        if SHAppBarMessage(ABM_NEW, &mut data) == 0 { return None; }
        let _ = SHAppBarMessage(ABM_QUERYPOS, &mut data);
        data.rc.left = monitor.left;
        data.rc.right = monitor.right;
        data.rc.top = data.rc.bottom - height;
        let _ = SHAppBarMessage(ABM_SETPOS, &mut data);
    }
    let state = AppBarState { hwnd: h.0 as isize, rect: data.rc, monitor, height, dpi_milli: key.3 };
    *appbar().lock().unwrap() = Some(state);
    Some(notch::Rect { left: data.rc.left, top: data.rc.top, right: data.rc.right, bottom: data.rc.bottom })
}

/// Sincroniza visibilidad, tamaño y AppBar del dock en el camino lento del notch.
pub fn sync(app: &AppHandle, settings: &Settings, fullscreen: bool) {
    if !window_utils::frontend_ready("notch") { return; }
    let Some(window) = create(app) else { return };
    let icon_size = settings.dock.icon_size.clamp(32, 96);
    sync_taskbar(settings);
    let enabled = settings.dock.enabled && !fullscreen && settings.general.visible;
    if !enabled {
        remove_appbar();
        let ui_window = window.clone();
        let _ = window.run_on_main_thread(move || { let _ = ui_window.hide(); });
        return;
    }
    let Some((fallback, monitor)) = notch::assignments(settings, &notch::monitors(app)).into_iter().find(|(l, _)| l == "main").map(|(_, m)| (m.rect, m.scale)) else { return };
    let scale = work_area_with_monitor(app, settings.dock.hide_taskbar).map(|(_, scale)| scale).unwrap_or(monitor);
    let reserve = settings.dock.reserve_space && !settings.dock.autohide;
    let granted = if reserve { register_appbar(&window, fallback, physical(icon_size + 26, scale), scale) } else { remove_appbar(); None };
    let (work, _) = if let Some(rect) = granted {
        (rect, scale)
    } else {
        work_area_with_monitor(app, settings.dock.hide_taskbar).unwrap_or((fallback, scale))
    };
    let requested = *requested_size().lock().unwrap();
    let (width, height) = requested.map(|(w, h)| (physical(w, scale), physical(h, scale))).unwrap_or_else(|| (
        physical((settings.dock.pinned.len().max(4) as i32 * icon_size + 60).min((work.width() as f64 / scale) as i32 - 24), scale),
        physical(80, scale),
    ));
    let width = width.clamp(120, work.width().max(120));
    let height = height.max(physical(80, scale));
    let left = work.left + (work.width() - width) / 2;
    let margin = physical(6, scale);
    let mut top = work.bottom - height - margin;
    if top < fallback.top + fallback.height() / 2 {
        eprintln!("[dock] posición defensiva: top={} queda en la mitad superior; recalculando desde rcMonitor", top);
        top = fallback.bottom - height - margin;
    }
    hit().lock().unwrap().scale = scale;
    let w = window.clone();
    let _ = window.run_on_main_thread(move || {
        let _ = w.set_position(PhysicalPosition::new(left, top));
        let _ = w.set_size(PhysicalSize::new(width as u32, height as u32));
        let _ = w.show();
        if let Ok(h) = w.hwnd() {
            // Después de show(): tao acaba de reescribir los estilos si la visibilidad cambió.
            window_utils::enforce_chrome_free_hwnd(h);
            unsafe { let _ = SetWindowPos(h, Some(HWND_TOPMOST), 0, 0, 0, 0, SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE); }
        }
    });
}

fn title(hwnd: HWND) -> String {
    unsafe {
        let n = GetWindowTextLengthW(hwnd);
        if n <= 0 { return String::new(); }
        let mut buf = vec![0u16; n as usize + 1];
        let len = GetWindowTextW(hwnd, &mut buf);
        String::from_utf16_lossy(&buf[..len as usize]).trim().to_string()
    }
}

unsafe extern "system" fn child_callback(hwnd: HWND, lparam: LPARAM) -> windows::core::BOOL {
    if IsWindowVisible(hwnd).as_bool() {
        let result = &mut *(lparam.0 as *mut Option<HWND>);
        *result = Some(hwnd);
        return false.into();
    }
    true.into()
}

unsafe extern "system" fn enum_callback(hwnd: HWND, lparam: LPARAM) -> windows::core::BOOL {
    if !IsWindowVisible(hwnd).as_bool() || GetAncestor(hwnd, GA_ROOT) != hwnd { return true.into(); }
    let ex = GetWindowLongPtrW(hwnd, GWL_EXSTYLE);
    if ex & WS_EX_TOOLWINDOW.0 as isize != 0 || title(hwnd).is_empty() { return true.into(); }
    let mut pid = 0u32;
    GetWindowThreadProcessId(hwnd, Some(&mut pid));
    let own = windows::Win32::System::Threading::GetCurrentProcessId();
    if pid == own { return true.into(); }
    let process_path = icons::process_image_path(pid).unwrap_or_default();
    let process_name = process_path.rsplit(['\\', '/']).next().unwrap_or_default().to_ascii_lowercase();
    let icon_hwnd = if process_name == "applicationframehost.exe" {
        let mut child = None;
        let _ = EnumChildWindows(Some(hwnd), Some(child_callback), LPARAM(&mut child as *mut _ as isize));
        child.unwrap_or(hwnd)
    } else { hwnd };
    let aumid = icons::window_app_user_model_id(icon_hwnd, pid).unwrap_or_default();
    let list = &mut *(lparam.0 as *mut Vec<(String, isize, String, String)>);
    list.push((title(hwnd), hwnd.0 as isize, process_path, aumid));
    true.into()
}

/// Ventana abierta de otra app: título, HWND, ruta del ejecutable y AUMID.
struct Running { title: String, hwnd: isize, path: String, aumid: String }

fn running_windows() -> Vec<Running> {
    let mut list: Vec<(String, isize, String, String)> = Vec::new();
    unsafe { let _ = EnumWindows(Some(enum_callback), LPARAM(&mut list as *mut _ as isize)); }
    list.into_iter().map(|(title, hwnd, path, aumid)| Running { title, hwnd, path, aumid }).collect()
}

fn file_name(path: &str) -> String { path.rsplit(['\\', '/']).next().unwrap_or(path).to_ascii_lowercase() }

/// Identidad estable de una app: AUMID si lo hay; si no, la ruta del ejecutable.
fn identity(path: &str, aumid: &str) -> String {
    if !aumid.trim().is_empty() { format!("aumid:{}", aumid.to_ascii_lowercase()) } else { format!("exe:{}", icons::resolve_program(path).to_ascii_lowercase()) }
}

/// ¿Esta ventana pertenece al elemento anclado? Por AUMID, ruta o nombre del ejecutable.
fn pin_matches(pin: &DockPinned, w: &Running) -> bool {
    if !pin.aumid.is_empty() { return w.aumid.eq_ignore_ascii_case(&pin.aumid); }
    let pinned = icons::resolve_program(&pin.path);
    !pinned.is_empty() && (w.path.eq_ignore_ascii_case(&pinned) || (!w.path.is_empty() && file_name(&w.path) == file_name(&pinned)))
}

fn pin_key(pin: &DockPinned) -> String { format!("pin:{}", pin.name) }

pub fn items(settings: &Settings) -> Vec<DockItem> {
    let running = running_windows();
    let mut used = vec![false; running.len()];
    let mut out = Vec::new();
    for p in &settings.dock.pinned {
        let mut count = 0;
        for (i, w) in running.iter().enumerate() { if pin_matches(p, w) { used[i] = true; count += 1; } }
        out.push(DockItem { key: pin_key(p), name: p.name.clone(), path: icons::resolve_program(&p.path), args: p.args.clone(), aumid: p.aumid.clone(), running: count > 0, window_count: count, kind: "pinned".into() });
    }
    if settings.dock.show_running {
        // Una entrada por app (no por ventana), en el orden en que aparecen.
        let mut groups: Vec<(String, usize)> = Vec::new();
        for (i, w) in running.iter().enumerate() {
            if used[i] { continue; }
            let id = identity(&w.path, &w.aumid);
            if let Some(group) = groups.iter_mut().find(|(g, _)| *g == id) { group.1 += 1; continue; }
            groups.push((id, 1));
            let icon_path = if w.aumid.is_empty() { w.path.clone() } else { format!("shell:AppsFolder\\{}", w.aumid) };
            out.push(DockItem { key: String::new(), name: w.title.clone(), path: icon_path, args: String::new(), aumid: w.aumid.clone(), running: true, window_count: 1, kind: "running".into() });
        }
        let first_running = out.len() - groups.len();
        for (item, (id, count)) in out[first_running..].iter_mut().zip(groups) { item.key = format!("app:{id}"); item.window_count = count; }
    }
    for folder in &settings.dock.folders {
        out.push(DockItem { key: format!("folder:{}", folder.path), name: folder.name.clone(), path: folder.path.clone(), args: String::new(), aumid: String::new(), running: false, window_count: 0, kind: "folder".into() });
    }
    out
}

/// Ventanas abiertas que corresponden a una clave del Dock (`pin:` o `app:`).
fn windows_for(settings: &Settings, key: &str) -> Vec<isize> {
    let running = running_windows();
    if let Some(pin) = settings.dock.pinned.iter().find(|p| pin_key(p) == key) {
        return running.iter().filter(|w| pin_matches(pin, w)).map(|w| w.hwnd).collect();
    }
    if let Some(id) = key.strip_prefix("app:") {
        return running.iter().filter(|w| identity(&w.path, &w.aumid) == id).map(|w| w.hwnd).collect();
    }
    Vec::new()
}
pub fn remove() { remove_appbar(); }

#[tauri::command]
pub async fn dock_items(state: tauri::State<'_, notch::NotchState>) -> Result<Vec<DockItem>, String> {
    let settings = state.settings.lock().unwrap().clone();
    match tokio::time::timeout(Duration::from_secs(2), tauri::async_runtime::spawn_blocking(move || items(&settings))).await {
        Ok(Ok(value)) => Ok(value),
        _ => Ok(Vec::new()),
    }
}

#[tauri::command]
pub fn dock_set_hit(x: i32, y: i32, w: i32, h: i32) { let scale = hit().lock().unwrap().scale; *hit().lock().unwrap() = Hit { x, y, w, h, scale }; }

#[tauri::command]
pub fn dock_resize(app: AppHandle, width: i32, height: i32) {
    *requested_size().lock().unwrap() = Some((width.clamp(120, 3000), height.clamp(60, 400)));
    let settings = app.state::<notch::NotchState>().settings.lock().unwrap().clone();
    sync(&app, &settings, false);
}

#[tauri::command]
pub async fn dock_launch(path: String, args: String, aumid: String) -> Result<(), String> {
    tauri::async_runtime::spawn_blocking(move || dock_launch_blocking(path, args, aumid)).await.map_err(|e| e.to_string())?
}

fn dock_launch_blocking(path: String, args: String, aumid: String) -> Result<(), String> {
    if !aumid.trim().is_empty() { return std::process::Command::new("explorer.exe").arg(format!("shell:AppsFolder\\{aumid}")).spawn().map(|_| ()).map_err(|e| e.to_string()); }
    let p = Path::new(&path);
    if !p.exists() && !path.eq_ignore_ascii_case("explorer.exe") && !path.eq_ignore_ascii_case("notepad.exe") { return Err("El programa ya no existe.".into()); }
    let mut cmd = std::process::Command::new(&path);
    for arg in args.split_whitespace() { cmd.arg(arg); }
    cmd.spawn().map(|_| ()).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn dock_activate(key: String, state: tauri::State<'_, notch::NotchState>) -> Result<(), String> {
    let settings = state.settings.lock().unwrap().clone();
    tauri::async_runtime::spawn_blocking(move || {
        let windows = windows_for(&settings, &key);
        if windows.is_empty() {
            // Sin ventanas: lanzar si es un elemento anclado.
            if let Some(pin) = settings.dock.pinned.iter().find(|p| pin_key(p) == key) { return dock_launch_blocking(pin.path.clone(), pin.args.clone(), pin.aumid.clone()); }
            return Ok(());
        }
        unsafe {
            let fg = GetForegroundWindow().0 as isize;
            let target = match windows.iter().position(|h| *h == fg) {
                // Una sola ventana y ya está delante: minimizar.
                Some(_) if windows.len() == 1 => { let _ = ShowWindow(HWND(fg as *mut _), SW_MINIMIZE); return Ok(()); }
                // Varias: alternar a la siguiente.
                Some(i) => windows[(i + 1) % windows.len()],
                None => windows[0],
            };
            let h = HWND(target as *mut _);
            let _ = ShowWindow(h, if IsIconic(h).as_bool() { SW_RESTORE } else { SW_SHOW });
            let _ = SetForegroundWindow(h);
        }
        Ok(())
    }).await.map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn dock_close_windows(key: String, state: tauri::State<'_, notch::NotchState>) -> Result<(), String> {
    let settings = state.settings.lock().unwrap().clone();
    for raw in windows_for(&settings, &key) {
        unsafe { let _ = windows::Win32::UI::WindowsAndMessaging::PostMessageW(Some(HWND(raw as *mut _)), windows::Win32::UI::WindowsAndMessaging::WM_CLOSE, windows::Win32::Foundation::WPARAM(0), windows::Win32::Foundation::LPARAM(0)); }
    }
    Ok(())
}
async fn commit_pin(app: &AppHandle, mut s: Settings) -> Result<(), String> { crate::commit_settings(app, { s.clamp(); s }).await.map(|_| ()) }

#[tauri::command]
pub async fn dock_pin(app: AppHandle, path: String, name: String, args: String, aumid: String) -> Result<(), String> {
    let state = app.state::<notch::NotchState>(); let mut s = state.settings.lock().unwrap().clone();
    if !s.dock.pinned.iter().any(|p| p.path == path && p.aumid == aumid) { s.dock.pinned.push(DockPinned { name, path, args, aumid }); }
    commit_pin(&app, s).await
}

#[tauri::command]
pub async fn dock_unpin(app: AppHandle, key: String) -> Result<(), String> {
    let state = app.state::<notch::NotchState>(); let mut s = state.settings.lock().unwrap().clone();
    s.dock.pinned.retain(|p| format!("pin:{}", p.name) != key && p.path != key);
    commit_pin(&app, s).await
}

#[tauri::command]
pub async fn dock_reorder(app: AppHandle, from: usize, to: usize) -> Result<(), String> {
    let state = app.state::<notch::NotchState>(); let mut s = state.settings.lock().unwrap().clone();
    if from < s.dock.pinned.len() && to < s.dock.pinned.len() { let item = s.dock.pinned.remove(from); s.dock.pinned.insert(to, item); }
    commit_pin(&app, s).await
}

#[tauri::command]
pub async fn trash_state() -> TrashState {
    match tokio::time::timeout(Duration::from_secs(1), tauri::async_runtime::spawn_blocking(|| {
        let mut q = windows::Win32::UI::Shell::SHQUERYRBINFO { cbSize: std::mem::size_of::<windows::Win32::UI::Shell::SHQUERYRBINFO>() as u32, ..Default::default() };
        unsafe { if SHQueryRecycleBinW(PCWSTR::null(), &mut q).is_ok() { return TrashState { items: q.i64NumItems as u32, bytes: q.i64Size as u64 }; } }
        TrashState { items: 0, bytes: 0 }
    })).await {
        Ok(Ok(value)) => value,
        _ => TrashState { items: 0, bytes: 0 },
    }
}

#[tauri::command]
pub async fn trash_open() -> Result<(), String> { tauri::async_runtime::spawn_blocking(|| std::process::Command::new("explorer.exe").arg("shell:RecycleBinFolder").spawn().map(|_| ()).map_err(|e| e.to_string())).await.map_err(|e| e.to_string())? }

#[tauri::command]
pub async fn trash_empty() -> Result<(), String> { tauri::async_runtime::spawn_blocking(|| unsafe { SHEmptyRecycleBinW(None, PCWSTR::null(), windows::Win32::UI::Shell::SHERB_NOCONFIRMATION | windows::Win32::UI::Shell::SHERB_NOPROGRESSUI | windows::Win32::UI::Shell::SHERB_NOSOUND).map_err(|e| e.to_string()) }).await.map_err(|e| e.to_string())? }

#[tauri::command]
pub async fn trash_drop(paths: Vec<String>) -> Result<(), String> {
    tauri::async_runtime::spawn_blocking(move || trash_drop_blocking(paths)).await.map_err(|e| e.to_string())?
}

fn trash_drop_blocking(paths: Vec<String>) -> Result<(), String> {
    if paths.is_empty() { return Ok(()); }
    let mut from = Vec::<u16>::new();
    for path in paths { from.extend(path.encode_utf16()); from.push(0); }
    from.push(0);
    let mut op = SHFILEOPSTRUCTW { wFunc: FO_DELETE, pFrom: PCWSTR(from.as_ptr()), fFlags: FOF_ALLOWUNDO.0 as u16 | FOF_NOCONFIRMATION.0 as u16 | FOF_NOERRORUI.0 as u16 | FOF_SILENT.0 as u16, ..Default::default() };
    let code = unsafe { SHFileOperationW(&mut op) };
    if code == 0 { Ok(()) } else { Err(format!("No se pudieron enviar los elementos a la papelera ({}).", code)) }
}

/// Sondeo económico: solo ajusta click-through y auto-ocultar cuando el dock está visible.
pub fn start(app: AppHandle) {
    std::thread::spawn(move || {
        let mut last: Option<(bool, bool)> = None;
        let mut last_scan = Instant::now() - Duration::from_secs(1);
        let mut last_check = Instant::now();
        let mut last_signature: Option<Vec<String>> = None;
        loop {
            let settings = app.state::<notch::NotchState>().settings.lock().unwrap().clone();
            let dock_settings = settings.dock.clone();
            let interval = if dock_settings.autohide { 120 } else { 250 };
            std::thread::sleep(Duration::from_millis(interval));
            let Some(window) = app.get_webview_window("dock") else { continue };
            if !dock_settings.enabled { continue; }
            // Win32 directo: nada de esperar al hilo de UI (antes un recv() sin límite
            // podía dejar este bucle bloqueado y el Dock sin responder).
            let Some(h) = hwnd(&window) else { continue };
            let visible = unsafe { IsWindowVisible(h).as_bool() };
            if visible && last_scan.elapsed() >= Duration::from_secs(1) {
                let current = items(&settings);
                let signature = current.iter().map(|item| format!("{}:{}:{}", item.key, item.window_count, item.running)).collect::<Vec<_>>();
                if last_signature.as_ref() != Some(&signature) {
                    let _ = app.emit("dock-windows", &current);
                    last_signature = Some(signature);
                }
                last_scan = Instant::now();
            }
            let mut rect = RECT::default();
            let has_rect = unsafe { GetWindowRect(h, &mut rect).is_ok() };
            let hit_area = *hit().lock().unwrap();
            let mut p = windows::Win32::Foundation::POINT::default();
            unsafe { let _ = GetCursorPos(&mut p); }
            let inside = has_rect && hit_area.w > 0 && {
                let x = ((p.x - rect.left) as f64 / hit_area.scale) as i32;
                let y = ((p.y - rect.top) as f64 / hit_area.scale) as i32;
                x >= hit_area.x && x < hit_area.x + hit_area.w && y >= hit_area.y && y < hit_area.y + hit_area.h
            };
            let near_bottom = work_area(&app).map(|(r, scale)| p.y >= r.bottom - physical(12, scale) && p.x >= r.left && p.x < r.right).unwrap_or(false);
            let should_show = !dock_settings.autohide || inside || near_bottom;
            // Cada 2 s se reaplica todo aunque nada haya cambiado: si Windows o tao
            // alteraron los estilos (clics ignorados) o el orden Z, se corrige solo.
            let periodic = last_check.elapsed() >= Duration::from_secs(2);
            if periodic {
                last_check = Instant::now();
                sync_taskbar(&settings);
                if visible && !capture_overlay_active() {
                    unsafe { let _ = SetWindowPos(h, Some(HWND_TOPMOST), 0, 0, 0, 0, SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE | SWP_ASYNCWINDOWPOS); }
                }
            }
            if !periodic && last == Some((should_show, inside)) { continue; }
            last = Some((should_show, inside));
            let w = window.clone();
            let _ = window.run_on_main_thread(move || {
                if should_show { let _ = w.show(); } else { let _ = w.hide(); }
                let _ = w.set_ignore_cursor_events(!inside);
                // tao reescribe GWL_EXSTYLE entero al cambiar el click-through: se repone justo después.
                if let Some(h) = hwnd(&w) { window_utils::enforce_chrome_free_hwnd(h); }
            });
        }
    });
}