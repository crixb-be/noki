//! Dock inferior flotante y operaciones nativas de sus elementos.
//!
//! La ventana se mantiene pequeña y transparente; el contenido React comunica el rectángulo real
//! de la píldora para que el sondeo pueda dejar pasar los clics fuera de ella.

use crate::{icons, notch, settings::{DockPinned, Settings}, window_utils};
use serde::Serialize;
use windows::Win32::Graphics::{
    Dwm::{DwmGetWindowAttribute, DWMWA_CLOAKED},
    Gdi::{CreateRectRgn, DeleteObject, GetWindowRgn, GetWindowRgnBox, SetWindowRgn, HGDIOBJ, HRGN, NULLREGION},
};
use std::{collections::HashMap, sync::{Mutex, OnceLock}, time::{Duration, Instant}};
use tauri::{window::Color, AppHandle, Emitter, Manager, PhysicalPosition, PhysicalSize, WebviewUrl, WebviewWindow, WebviewWindowBuilder};
use windows::{core::{w, PCWSTR}, Win32::{Foundation::{HWND, LPARAM, RECT}, Graphics::{Gdi::{GetMonitorInfoW, MonitorFromWindow, MONITORINFO, MONITOR_DEFAULTTONEAREST}}, UI::{Shell::{SHAppBarMessage, SHEmptyRecycleBinW, SHFileOperationW, SHQueryRecycleBinW, APPBARDATA, SHFILEOPSTRUCTW, FO_DELETE, FOF_ALLOWUNDO, FOF_NOCONFIRMATION, FOF_NOERRORUI, FOF_SILENT}, WindowsAndMessaging::{EnumChildWindows, EnumWindows, FindWindowExW, FindWindowW, GetClassNameW, GetWindowRect, GetAncestor, GetCursorPos, GetWindowLongPtrW, GetWindowTextLengthW, GetWindowTextW, GetWindowThreadProcessId, GetForegroundWindow, IsIconic, IsWindowVisible, SetForegroundWindow, SetWindowPos, ShowWindowAsync, GA_ROOT, GWL_EXSTYLE, HWND_TOPMOST, SW_MINIMIZE, SW_RESTORE, SW_HIDE, SW_SHOWNA, SWP_ASYNCWINDOWPOS, SWP_NOACTIVATE, SWP_NOMOVE, SWP_NOSIZE, SWP_NOZORDER, WS_EX_TOOLWINDOW}}}};

const ABM_NEW: u32 = 0;
const ABM_REMOVE: u32 = 1;
const ABM_QUERYPOS: u32 = 2;
const ABM_SETPOS: u32 = 3;
const ABE_BOTTOM: u32 = 3;

#[derive(Default, Clone, Copy)]
struct Hit { x: i32, y: i32, w: i32, h: i32, scale: f64 }

#[derive(Clone, Copy)]
struct AppBarState { hwnd: isize, rect: RECT, monitor: notch::Rect, height: i32, dpi_milli: i32 }

static RUNNING_ORDER: OnceLock<Mutex<Vec<String>>> = OnceLock::new();
const ACTIVATION_GUARD: Duration = Duration::from_millis(350);
static ACTIVATIONS: std::sync::LazyLock<Mutex<HashMap<isize, Instant>>> =
    std::sync::LazyLock::new(|| Mutex::new(HashMap::new()));

pub(crate) fn recently_activated(raw: isize) -> bool {
    ACTIVATIONS.lock().unwrap().get(&raw).is_some_and(|at| at.elapsed() < ACTIVATION_GUARD)
}
static CAN_SHOW: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
static MENU_OPEN: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

pub(crate) fn set_menu_open(open: bool) { MENU_OPEN.store(open, std::sync::atomic::Ordering::SeqCst); }

pub(crate) fn menu_open() -> bool { MENU_OPEN.load(std::sync::atomic::Ordering::SeqCst) }

static HIT: OnceLock<Mutex<Hit>> = OnceLock::new();
static APPBAR: OnceLock<Mutex<Option<AppBarState>>> = OnceLock::new();
static SIZE: OnceLock<Mutex<Option<(i32, i32)>>> = OnceLock::new();
struct HiddenTaskbar {
    hwnd: isize,
    region: Option<isize>,
    hidden: bool,
}

// None desactiva el hook. Una región guardada de valor 0 indica que Explorer no usaba una región.
static HIDDEN_TASKBARS: Mutex<Option<Vec<HiddenTaskbar>>> = Mutex::new(None);

fn hit() -> &'static Mutex<Hit> { HIT.get_or_init(|| Mutex::new(Hit { scale: 1.0, ..Hit::default() })) }
fn appbar() -> &'static Mutex<Option<AppBarState>> { APPBAR.get_or_init(|| Mutex::new(None)) }
fn requested_size() -> &'static Mutex<Option<(i32, i32)>> { SIZE.get_or_init(|| Mutex::new(None)) }

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
    pub is_downloads: bool,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TrashState { pub items: u32, pub bytes: u64 }

fn physical(v: i32, scale: f64) -> i32 { (v as f64 * scale).round() as i32 }

pub(crate) fn hwnd(window: &WebviewWindow) -> Option<HWND> { window.hwnd().ok() }


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

fn clip_taskbar(hwnd: HWND) -> Option<isize> {
    unsafe {
        let original = CreateRectRgn(0, 0, 0, 0);
        if original.is_invalid() { return None; }
        let previous = if GetWindowRgn(hwnd, original).0 == 0 {
            let _ = DeleteObject(HGDIOBJ(original.0));
            0
        } else {
            original.0 as isize
        };
        // Explorer puede marcar la ventana visible antes de que llegue EVENT_OBJECT_SHOW.
        // Una región vacía impide que se dibuje durante ese intervalo sin mover la ventana.
        let empty = CreateRectRgn(0, 0, 0, 0);
        if !empty.is_invalid() && SetWindowRgn(hwnd, Some(empty), false) != 0 {
            return Some(previous); // Windows es dueño de la región instalada.
        }
        if !empty.is_invalid() { let _ = DeleteObject(HGDIOBJ(empty.0)); }
        if previous != 0 { let _ = DeleteObject(HGDIOBJ(previous as *mut _)); }
        None
    }
}

fn hide_taskbar(hwnd: HWND, bars: &mut Vec<HiddenTaskbar>) {
    let raw = hwnd.0 as isize;
    let bar = match bars.iter().position(|bar| bar.hwnd == raw) {
        Some(index) => &mut bars[index],
        None => {
            bars.push(HiddenTaskbar { hwnd: raw, region: clip_taskbar(hwnd), hidden: false });
            bars.last_mut().unwrap()
        }
    };
    unsafe {
        if IsWindowVisible(hwnd).as_bool() && ShowWindowAsync(hwnd, SW_HIDE).as_bool() {
            bar.hidden = true;
        }
    }
}

fn window_class(hwnd: HWND, buffer: &mut [u16]) -> String {
    let len = unsafe { GetClassNameW(hwnd, buffer) };
    String::from_utf16_lossy(&buffer[..len.max(0) as usize])
}

fn is_taskbar_class(class: &str) -> bool {
    matches!(class, "Shell_TrayWnd" | "Shell_SecondaryTrayWnd")
}

unsafe extern "system" fn taskbar_win_event(
    _hook: windows::Win32::UI::Accessibility::HWINEVENTHOOK, _event: u32, hwnd: HWND,
    object: i32, child: i32, _thread: u32, _time: u32,
) {
    if hwnd.0.is_null() || object != 0 || child != 0 { return; }
    let mut state = HIDDEN_TASKBARS.lock().unwrap();
    let Some(hidden) = state.as_mut() else { return };
    let class = window_class(hwnd, &mut [0; 128]);
    if is_taskbar_class(&class) {
        hide_taskbar(hwnd, hidden);
    } else if class == "Windows.UI.Core.CoreWindow" {
        let mut pid = 0;
        GetWindowThreadProcessId(hwnd, Some(&mut pid));
        if icons::process_image_path(pid).is_some_and(|path| file_name(&path) == "shellexperiencehost.exe") {
            for bar in taskbars() { hide_taskbar(bar, hidden); }
        }
    }
}

fn start_taskbar_watch() {
    use windows::Win32::UI::{
        Accessibility::{SetWinEventHook, UnhookWinEvent},
        WindowsAndMessaging::{DispatchMessageW, GetMessageW, EVENT_OBJECT_SHOW, EVENT_SYSTEM_FOREGROUND, MSG, WINEVENT_OUTOFCONTEXT, WINEVENT_SKIPOWNPROCESS},
    };
    // Los WinEvents se entregan en el hilo que instaló el hook.
    std::thread::spawn(|| unsafe {
        let hooks = [EVENT_OBJECT_SHOW, EVENT_SYSTEM_FOREGROUND].map(|event| {
            SetWinEventHook(event, event, None, Some(taskbar_win_event), 0, 0, WINEVENT_OUTOFCONTEXT | WINEVENT_SKIPOWNPROCESS)
        });
        if hooks.iter().any(|hook| hook.is_invalid()) {
            eprintln!("[dock] No se pudieron instalar todos los hooks de la barra de tareas");
        }
        {
            let mut state = HIDDEN_TASKBARS.lock().unwrap();
            if let Some(hidden) = state.as_mut() {
                for bar in taskbars() { hide_taskbar(bar, hidden); }
            }
        }
        let mut message = MSG::default();
        while GetMessageW(&mut message, None, 0, 0).0 > 0 {
            DispatchMessageW(&message);
        }
        for hook in hooks.into_iter().filter(|hook| !hook.is_invalid()) {
            let _ = UnhookWinEvent(hook);
        }
    });
}

fn sync_taskbar(settings: &Settings) {
    if settings.dock.enabled && settings.dock.hide_taskbar && settings.general.visible {
        let mut state = HIDDEN_TASKBARS.lock().unwrap();
        if state.is_some() { return; }
        let hidden = state.insert(Vec::new());
        for bar in taskbars() { hide_taskbar(bar, hidden); }
    } else {
        restore_taskbar();
    }
}

/// ¿Está activa la interfaz de recorte (Win+Shift+S, Recortes)? Subir Noki por encima
/// de la selección puede interrumpirla.
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

const CAPTURE_CLASSES: [&str; 3] = [
    "Microsoft-Windows-SnipperToolbar",
    "Microsoft-Windows-SnipperCaptureForm",
    "ScreenClippingWindow",
];

fn capture_window(hwnd: HWND) -> bool {
    if hwnd.0.is_null() { return false; }
    if CAPTURE_CLASSES.contains(&window_class(hwnd, &mut [0; 128]).as_str()) { return true; }
    let mut pid = 0;
    unsafe { GetWindowThreadProcessId(hwnd, Some(&mut pid)); }
    let process = file_name(&icons::process_image_path(pid).unwrap_or_default());
    matches!(process.as_str(), "screenclippinghost.exe" | "snippingtool.exe" | "gamebar.exe" | "gamebarftserver.exe")
}

fn snipping_window_name(hwnd: HWND, class: &str) -> bool {
    let matches_name = |name: &str| {
        let name = name.to_ascii_lowercase();
        name.contains("snipping") || name.contains("screenclipping")
    };
    matches_name(class) || matches_name(&title(hwnd))
}

fn ignored_overlay_window(hwnd: HWND) -> bool {
    use windows::Win32::UI::WindowsAndMessaging::{WS_EX_LAYERED, WS_EX_TRANSPARENT};
    let mask = (WS_EX_LAYERED | WS_EX_TRANSPARENT).0 as isize;
    if unsafe { GetWindowLongPtrW(hwnd, GWL_EXSTYLE) } & mask == mask || capture_window(hwnd) { return true; }
    let class = window_class(hwnd, &mut [0; 128]);
    class.to_ascii_lowercase().contains("microsoft.ui.content.desktopchildsitebridge") || snipping_window_name(hwnd, &class)
}

fn layered_window_translucent(hwnd: HWND) -> bool {
    use windows::Win32::UI::WindowsAndMessaging::{GetLayeredWindowAttributes, LWA_ALPHA, LWA_COLORKEY};
    let mut alpha = 255u8;
    let mut flags = Default::default();
    let has_attributes = unsafe { GetLayeredWindowAttributes(hwnd, None, Some(&mut alpha), Some(&mut flags)).is_ok() };
    has_attributes && (flags.contains(LWA_COLORKEY) || (flags.contains(LWA_ALPHA) && alpha < 255))
}

pub(crate) fn eligible_foreground() -> Option<HWND> {
    use windows::Win32::{System::Threading::GetCurrentProcessId, UI::WindowsAndMessaging::{IsZoomed, GWL_STYLE, WS_CAPTION, WS_EX_LAYERED, WS_EX_TRANSPARENT}};
    unsafe {
        let foreground = GetForegroundWindow();
        if foreground.0.is_null() { return None; }
        let hwnd = GetAncestor(foreground, GA_ROOT);
        if hwnd.0.is_null() || !IsWindowVisible(hwnd).as_bool() || IsIconic(hwnd).as_bool() { return None; }
        let mut pid = 0;
        GetWindowThreadProcessId(hwnd, Some(&mut pid));
        if pid == GetCurrentProcessId() { return None; }
        let class = window_class(hwnd, &mut [0; 128]);
        if matches!(class.as_str(), "Progman" | "WorkerW" | "Shell_TrayWnd" | "Shell_SecondaryTrayWnd") || ignored_overlay_window(hwnd) { return None; }
        let ex = GetWindowLongPtrW(hwnd, GWL_EXSTYLE) as u32;
        if ex & WS_EX_TRANSPARENT.0 != 0 || (ex & WS_EX_LAYERED.0 != 0 && layered_window_translucent(hwnd)) { return None; }
        // Una ventana maximizada con barra de título puede cubrir el monitor si se oculta la barra de tareas.
        if IsZoomed(hwnd).as_bool() && (GetWindowLongPtrW(hwnd, GWL_STYLE) as u32 & WS_CAPTION.0) == WS_CAPTION.0 { return None; }
        Some(hwnd)
    }
}

pub(crate) fn fullscreen_foreground() -> bool {
    let Some(hwnd) = eligible_foreground() else { return false };
    unsafe {
        let mut rect = RECT::default();
        if GetWindowRect(hwnd, &mut rect).is_err() { return false; }
        let monitor = MonitorFromWindow(hwnd, MONITOR_DEFAULTTONEAREST);
        let mut info = MONITORINFO { cbSize: std::mem::size_of::<MONITORINFO>() as u32, ..Default::default() };
        if monitor.0.is_null() || !GetMonitorInfoW(monitor, &mut info).as_bool() { return false; }
        let area = info.rcMonitor;
        rect.left <= area.left && rect.top <= area.top && rect.right >= area.right && rect.bottom >= area.bottom
    }
}

pub(crate) fn capture_foreground_excluded() -> bool {
    use windows::Win32::UI::WindowsAndMessaging::{GWL_STYLE, WS_CAPTION, WS_EX_LAYERED, WS_EX_NOACTIVATE, WS_EX_TRANSPARENT, WS_THICKFRAME};
    unsafe {
        let hwnd = GetForegroundWindow();
        if hwnd.0.is_null() { return false; }
        if ignored_overlay_window(hwnd) { return true; }
        let style = GetWindowLongPtrW(hwnd, GWL_STYLE) as u32;
        let ex = GetWindowLongPtrW(hwnd, GWL_EXSTYLE) as u32;
        if style & (WS_CAPTION | WS_THICKFRAME).0 != 0 { return false; }
        if ex & WS_EX_TRANSPARENT.0 != 0 { return true; }
        if ex & WS_EX_LAYERED.0 == 0 { return false; }
        ex & (WS_EX_NOACTIVATE | WS_EX_TOOLWINDOW).0 != 0 || layered_window_translucent(hwnd)
    }
}

pub(crate) fn screen_recording_active() -> bool {
    use windows::Win32::{System::Threading::GetCurrentProcessId, UI::WindowsAndMessaging::{WS_EX_LAYERED, WS_EX_TOPMOST, WS_EX_TRANSPARENT}};
    unsafe extern "system" fn recording_frame(hwnd: HWND, lparam: LPARAM) -> windows::core::BOOL {
        let mask = (WS_EX_LAYERED | WS_EX_TRANSPARENT | WS_EX_TOPMOST).0 as isize;
        if !IsWindowVisible(hwnd).as_bool() || GetWindowLongPtrW(hwnd, GWL_EXSTYLE) & mask != mask { return true.into(); }
        // Leer el título de una ventana propia puede esperar al hilo de UI.
        let mut pid = 0;
        GetWindowThreadProcessId(hwnd, Some(&mut pid));
        if pid == GetCurrentProcessId() { return true.into(); }
        if !capture_window(hwnd) && !snipping_window_name(hwnd, &window_class(hwnd, &mut [0; 128])) { return true.into(); }
        *(lparam.0 as *mut bool) = true;
        false.into()
    }

    // El marco rojo sigue visible aunque Recortes pierda el foco. Se comparte el sondeo entre los bucles.
    const TTL: Duration = Duration::from_millis(250);
    static CACHE: OnceLock<Mutex<(Instant, bool)>> = OnceLock::new();
    let mut cached = CACHE.get_or_init(|| Mutex::new((Instant::now() - TTL, false))).lock().unwrap();
    if cached.0.elapsed() >= TTL {
        let mut active = false;
        unsafe { let _ = EnumWindows(Some(recording_frame), LPARAM(&mut active as *mut bool as isize)); }
        *cached = (Instant::now(), active);
    }
    cached.1
}

fn capture_overlay_now() -> bool {
    unsafe {
        for class in CAPTURE_CLASSES {
            let wide: Vec<u16> = class.encode_utf16().chain(std::iter::once(0)).collect();
            let mut after = None;
            while let Ok(h) = FindWindowExW(None, after, PCWSTR(wide.as_ptr()), PCWSTR::null()) {
                if h.0.is_null() { break; }
                if IsWindowVisible(h).as_bool() { return true; }
                after = Some(h);
            }
        }
        if screen_recording_active() { return false; }
        capture_window(GetForegroundWindow())
    }
}

pub fn restore_taskbar() {
    let mut state = HIDDEN_TASKBARS.lock().unwrap();
    let Some(bars) = state.take() else { return };
    let current = taskbars();
    for bar in bars {
        let hwnd = HWND(bar.hwnd as *mut _);
        let exists = current.contains(&hwnd);
        if let Some(original) = bar.region {
            let restored = exists && unsafe {
                GetWindowRgnBox(hwnd, &mut RECT::default()) == NULLREGION
                    && SetWindowRgn(hwnd, (original != 0).then_some(HRGN(original as *mut _)), true) != 0
            };
            if !restored && original != 0 {
                unsafe { let _ = DeleteObject(HGDIOBJ(original as *mut _)); }
            }
        }
        if exists && bar.hidden {
            unsafe { let _ = ShowWindowAsync(hwnd, SW_SHOWNA); }
        }
    }
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
    crate::dock_drop::register(app, &window);
    crate::dropzone::ensure(app, &window);
    let _ = crate::dock_preview::ensure(app);
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
    let icon_size = settings.dock.icon_size.clamp(36, 64);
    sync_taskbar(settings);
    let enabled = settings.dock.enabled && settings.general.visible;
    let fullscreen = fullscreen && !capture_foreground_excluded();
    CAN_SHOW.store(enabled && !fullscreen, std::sync::atomic::Ordering::SeqCst);
    crate::dock_animation::sync(app, enabled && settings.dock.animate_apps);
    if !enabled || fullscreen { crate::shortcut_folder::close(app); }
    if !enabled {
        remove_appbar();
        let ui_window = window.clone();
        let _ = window.run_on_main_thread(move || { let _ = ui_window.hide(); });
        return;
    }
    let Some((fallback, monitor)) = notch::assignments(settings, &notch::monitors(app)).into_iter().find(|(l, _)| l == "main").map(|(_, m)| (m.rect, m.scale)) else { return };
    let (physical_monitor, scale) = work_area_with_monitor(app, true).unwrap_or((fallback, monitor));
    let reserve = settings.dock.reserve_space && !settings.dock.autohide;
    let granted = if reserve { register_appbar(&window, physical_monitor, physical(icon_size + 26, scale), scale) } else { remove_appbar(); None };
    if fullscreen {
        let ui_window = window.clone();
        let _ = window.run_on_main_thread(move || { let _ = ui_window.hide(); });
        return;
    }
    // AppBar reserva espacio para otras ventanas. Con la barra de tareas oculta, el Dock se ancla
    // al monitor físico aunque Windows siga reservando espacio al negociar ABM_SETPOS.
    let work = if settings.dock.hide_taskbar {
        physical_monitor
    } else {
        granted.or_else(|| work_area(app).map(|(area, _)| area)).unwrap_or(physical_monitor)
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
    if top < physical_monitor.top + physical_monitor.height() / 2 {
        eprintln!("[dock] posición defensiva: top={} queda en la mitad superior; recalculando desde rcMonitor", top);
        top = physical_monitor.bottom - height - margin;
    }
    hit().lock().unwrap().scale = scale;
    let w = window.clone();
    let _ = window.run_on_main_thread(move || {
        // Posición y tamaño en un solo SetWindowPos: con set_position y luego set_size la ventana
        // pasaba un instante por un estado intermedio (desplazada) y el vidrio daba un salto al
        // crecer hacia arriba (p. ej. al abrir el menú contextual).
        match w.hwnd() {
            Ok(h) => unsafe { let _ = SetWindowPos(h, None, left, top, width, height, SWP_NOZORDER | SWP_NOACTIVATE); },
            Err(_) => { let _ = w.set_position(PhysicalPosition::new(left, top)); let _ = w.set_size(PhysicalSize::new(width as u32, height as u32)); }
        }
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

fn cloaked(hwnd: HWND) -> bool {
    let mut value = 0u32;
    unsafe { DwmGetWindowAttribute(hwnd, DWMWA_CLOAKED, &mut value as *mut _ as *mut _, std::mem::size_of::<u32>() as u32).is_ok() && value != 0 }
}

struct HostedApp { host_pid: u32, child: Option<(HWND, u32, String)> }

unsafe extern "system" fn child_callback(hwnd: HWND, lparam: LPARAM) -> windows::core::BOOL {
    let state = &mut *(lparam.0 as *mut HostedApp);
    let mut class = [0u16; 128];
    let len = GetClassNameW(hwnd, &mut class);
    if String::from_utf16_lossy(&class[..len.max(0) as usize]) != "Windows.UI.Core.CoreWindow"
        || !IsWindowVisible(hwnd).as_bool() || cloaked(hwnd) { return true.into(); }
    let mut pid = 0;
    GetWindowThreadProcessId(hwnd, Some(&mut pid));
    if pid == 0 || pid == state.host_pid { return true.into(); }
    if let Some(path) = icons::process_image_path(pid).filter(|p| !p.is_empty() && !icons::is_shell_host(p)) {
        state.child = Some((hwnd, pid, path));
        return false.into();
    }
    true.into()
}

unsafe extern "system" fn enum_callback(hwnd: HWND, lparam: LPARAM) -> windows::core::BOOL {
    if !IsWindowVisible(hwnd).as_bool() || cloaked(hwnd) || GetAncestor(hwnd, GA_ROOT) != hwnd { return true.into(); }
    let ex = GetWindowLongPtrW(hwnd, GWL_EXSTYLE);
    if ex & WS_EX_TOOLWINDOW.0 as isize != 0 || title(hwnd).is_empty() { return true.into(); }
    let mut pid = 0u32;
    GetWindowThreadProcessId(hwnd, Some(&mut pid));
    if pid == windows::Win32::System::Threading::GetCurrentProcessId() { return true.into(); }
    let Some(mut process_path) = icons::process_image_path(pid).filter(|p| !p.is_empty()) else { return true.into(); };
    let icon_hwnd = if file_name(&process_path) == "applicationframehost.exe" {
        let mut state = HostedApp { host_pid: pid, child: None };
        let _ = EnumChildWindows(Some(hwnd), Some(child_callback), LPARAM(&mut state as *mut _ as isize));
        let Some((child, child_pid, path)) = state.child else { return true.into(); };
        pid = child_pid;
        process_path = path;
        child
    } else { hwnd };
    if icons::is_shell_host(&process_path) { return true.into(); }
    let aumid = icons::window_app_user_model_id(icon_hwnd, pid).unwrap_or_default();
    let list = &mut *(lparam.0 as *mut Vec<Running>);
    list.push(Running { title: title(hwnd), hwnd: hwnd.0 as isize, path: process_path, aumid, recycle_bin: false });
    true.into()
}

/// Keep the frame HWND for activation/WM_CLOSE, but always use the real app for identity/icons.
struct Running { title: String, hwnd: isize, path: String, aumid: String, recycle_bin: bool }

/// Identify Explorer's current namespace using PIDLs, independent of title, language or URL.
fn explorer_windows(path: Option<&str>) -> Vec<isize> {
    use windows::{core::Interface, Win32::{System::{Com::{CoCreateInstance, CoInitializeEx, CoTaskMemFree, CoUninitialize, IServiceProvider, CLSCTX_ALL, COINIT_APARTMENTTHREADED}, Variant::VARIANT}, UI::Shell::{IFolderView, IPersistFolder2, IShellBrowser, IShellWindows, IWebBrowserApp, ShellWindows, SID_STopLevelBrowser, SHGetKnownFolderIDList, FOLDERID_RecycleBinFolder, KF_FLAG_DEFAULT, ILIsEqual, SHParseDisplayName}}};
    struct ComGuard(bool);
    impl Drop for ComGuard { fn drop(&mut self) { if self.0 { unsafe { CoUninitialize(); } } } }
    unsafe {
        let _com = ComGuard(CoInitializeEx(None, COINIT_APARTMENTTHREADED).is_ok());
        let Ok(windows): Result<IShellWindows, _> = CoCreateInstance(&ShellWindows, None, CLSCTX_ALL) else { return Vec::new(); };
        let target = if let Some(path) = path {
            let mut pidl = std::ptr::null_mut();
            if SHParseDisplayName(&windows::core::HSTRING::from(path), None, &mut pidl, 0, None).is_err() {
                return Vec::new();
            }
            pidl
        } else {
            let Ok(pidl) = SHGetKnownFolderIDList(&FOLDERID_RecycleBinFolder, KF_FLAG_DEFAULT.0 as u32, None) else { return Vec::new(); };
            pidl
        };
        let mut result = Vec::new();
        for i in 0..windows.Count().unwrap_or(0) {
            let resolved = (|| -> windows::core::Result<Option<isize>> {
                let dispatch = windows.Item(&VARIANT::from(i))?;
                let browser: IWebBrowserApp = dispatch.cast()?;
                let provider: IServiceProvider = dispatch.cast()?;
                let shell: IShellBrowser = provider.QueryService(&SID_STopLevelBrowser)?;
                let view: IFolderView = shell.QueryActiveShellView()?.cast()?;
                let folder: IPersistFolder2 = view.GetFolder()?;
                let pidl = folder.GetCurFolder()?;
                let matches = ILIsEqual(pidl, target).as_bool();
                CoTaskMemFree(Some(pidl.cast()));
                Ok(if matches { Some(browser.HWND()?.0) } else { None })
            })();
            if let Ok(Some(hwnd)) = resolved { result.push(hwnd); }
        }
        CoTaskMemFree(Some(target.cast()));
        result
    }
}

fn enumerate_windows() -> Vec<Running> {
    let mut list = Vec::new();
    unsafe { let _ = EnumWindows(Some(enum_callback), LPARAM(&mut list as *mut Vec<Running> as isize)); }
    list
}

fn running_windows() -> Vec<Running> {
    let mut list = enumerate_windows();
    if list.iter().any(|w| file_name(&w.path) == "explorer.exe") {
        let bins = explorer_windows(None);
        for w in &mut list { w.recycle_bin = bins.contains(&w.hwnd); }
    }
    list
}

pub(crate) fn open_window_entries() -> Vec<(isize, String)> {
    running_windows().into_iter().map(|w| (w.hwnd, w.title)).collect()
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

fn pin_key(pin: &DockPinned) -> String { format!("pin:{}:{}", identity(&pin.path, &pin.aumid), pin.args) }

pub(crate) fn animation_windows(settings: &Settings) -> Vec<(isize, String)> {
    running_windows().into_iter().filter(|window| !settings.dock.show_trash || !window.recycle_bin).filter_map(|window| {
        let key = settings.dock.pinned.iter().find(|pin| pin_matches(pin, &window)).map(pin_key)
            .or_else(|| settings.dock.show_running.then(|| format!("app:{}", identity(&window.path, &window.aumid))))?;
        Some((window.hwnd, key))
    }).collect()
}

/// Normalize both existing pins and incoming settings without changing paths, arguments or order.
pub(crate) fn normalize_pinned_names(settings: &mut Settings) -> bool {
    let mut changed = false;
    for pin in &mut settings.dock.pinned {
        let name = icons::program_name(&pin.path, &pin.aumid);
        if pin.name != name { pin.name = name; changed = true; }
    }
    changed
}

pub fn items(settings: &Settings) -> Vec<DockItem> {
    let running: Vec<_> = running_windows().into_iter().filter(|w| !settings.dock.show_trash || !w.recycle_bin).collect();
    let mut used = vec![false; running.len()];
    let mut out = Vec::new();
    for p in &settings.dock.pinned {
        if icons::is_shell_host(&p.path) { continue; }
        let mut count = 0;
        for (i, w) in running.iter().enumerate() { if pin_matches(p, w) { used[i] = true; count += 1; } }
        out.push(DockItem { key: pin_key(p), name: p.name.clone(), path: icons::resolve_program(&p.path), args: p.args.clone(), aumid: p.aumid.clone(), running: count > 0, window_count: count, kind: "pinned".into(), is_downloads: false });
    }
    let first = out.len();
    if settings.dock.show_running {
        // Una entrada por app (no por ventana), en el orden en que aparecen.
        let mut groups: Vec<(String, usize)> = Vec::new();
        for (i, w) in running.iter().enumerate() {
            if used[i] { continue; }
            let id = identity(&w.path, &w.aumid);
            if let Some(group) = groups.iter_mut().find(|(g, _)| *g == id) { group.1 += 1; continue; }
            groups.push((id, 1));
            let icon_path = if w.aumid.is_empty() { w.path.clone() } else { format!("shell:AppsFolder\\{}", w.aumid) };
            out.push(DockItem { key: String::new(), name: icons::program_name(&w.path, &w.aumid), path: icon_path, args: String::new(), aumid: w.aumid.clone(), running: true, window_count: 1, kind: "running".into(), is_downloads: false });
        }
        let first_running = out.len() - groups.len();
        for (item, (id, count)) in out[first_running..].iter_mut().zip(groups) { item.key = format!("app:{id}"); item.window_count = count; }
    }
    // EnumWindows follows Z-order. Keep surviving app identities in their first-seen order;
    // only append newly opened apps, without ever sorting the configured pins.
    let mut order = RUNNING_ORDER.get_or_init(|| Mutex::new(Vec::new())).lock().unwrap();
    stable_running_order(&mut order, &mut out[first..]);
    drop(order);
    let downloads = crate::settings::default_downloads_path();
    for folder in &settings.dock.folders {
        let is_downloads = folder.path.eq_ignore_ascii_case(&downloads.to_string_lossy());
        out.push(DockItem { key: format!("folder:{}", folder.path), name: folder.name.clone(), path: folder.path.clone(), args: String::new(), aumid: String::new(), running: false, window_count: 0, kind: "folder".into(), is_downloads });
    }
    out
}

fn stable_running_order(order: &mut Vec<String>, items: &mut [DockItem]) {
    order.retain(|key| items.iter().any(|item| &item.key == key));
    let mut new_keys: Vec<_> = items.iter().filter(|item| !order.contains(&item.key)).map(|item| item.key.clone()).collect();
    // Deterministic order when several apps first appear in the same snapshot.
    new_keys.sort();
    new_keys.dedup();
    order.extend(new_keys);
    items.sort_by_key(|item| order.iter().position(|key| key == &item.key).unwrap_or(usize::MAX));
}

pub(crate) fn menu_item(settings: &Settings, key: &str) -> Option<DockItem> {
    items(settings).into_iter().find(|item| item.key == key)
}

pub(crate) fn drop_app(settings: &Settings, key: &str) -> Option<DockItem> {
    let mut item = menu_item(settings, key)?;
    if item.kind != "pinned" && item.kind != "running" { return None; }
    if item.path.to_ascii_lowercase().starts_with("shell:") && !item.aumid.contains('!') {
        if let Some(window) = running_windows().into_iter().find(|w| w.aumid.eq_ignore_ascii_case(&item.aumid)) {
            item.path = window.path;
        }
    }
    Some(item)
}

/// Ventanas abiertas que corresponden a una clave del Dock (`pin:` o `app:`).
pub(crate) fn windows_for(settings: &Settings, key: &str) -> Vec<isize> {
    window_titles_for(settings, key).into_iter().map(|(hwnd, _)| hwnd).collect()
}

pub(crate) fn window_titles_for(settings: &Settings, key: &str) -> Vec<(isize, String)> {
    if let Some(folder) = settings.dock.folders.iter().find(|folder| key.strip_prefix("folder:") == Some(folder.path.as_str())) {
        return explorer_windows(Some(&folder.path)).into_iter().map(|raw| (raw, title(HWND(raw as *mut _)))).collect();
    }
    if key == "trash" && settings.dock.show_trash {
        return explorer_windows(None).into_iter().map(|raw| (raw, title(HWND(raw as *mut _)))).collect();
    }
    let pin = settings.dock.pinned.iter().find(|p| pin_key(p) == key);
    let app_id = key.strip_prefix("app:");
    if pin.is_none() && app_id.is_none() { return Vec::new(); }
    let windows = enumerate_windows().into_iter()
        .filter(|w| pin.is_some_and(|p| pin_matches(p, w)) || app_id.is_some_and(|id| identity(&w.path, &w.aumid) == id))
        .collect::<Vec<_>>();
    // Solo consulta el espacio de nombres de Explorer cuando el icono representa Explorer.
    // Durante un arrastre, Explorer puede mantener esa consulta COM ocupada.
    let bins = if settings.dock.show_trash && windows.iter().any(|w| file_name(&w.path) == "explorer.exe") {
        explorer_windows(None)
    } else {
        Vec::new()
    };
    windows.into_iter().filter(|w| !bins.contains(&w.hwnd)).map(|w| (w.hwnd, w.title)).collect()
}

pub(crate) fn can_show() -> bool {
    CAN_SHOW.load(std::sync::atomic::Ordering::SeqCst)
}

pub(crate) fn focus_window(raw: isize) -> bool {
    use windows::Win32::{
        System::Threading::{AttachThreadInput, GetCurrentThreadId},
        UI::WindowsAndMessaging::{
            AllowSetForegroundWindow, BringWindowToTop, IsWindow, PeekMessageW, ASFW_ANY,
            MSG, PM_NOREMOVE,
        },
    };
    let h = HWND(raw as *mut _);
    unsafe {
        if !IsWindow(Some(h)).as_bool() { return false; }
        let _ = AllowSetForegroundWindow(ASFW_ANY);
        if IsIconic(h).as_bool() {
            if !ShowWindowAsync(h, SW_RESTORE).as_bool() { return false; }
            // La solicitud se procesa en la cola de la ventana de destino.
            let deadline = Instant::now() + Duration::from_millis(150);
            while IsIconic(h).as_bool() && Instant::now() < deadline {
                std::thread::sleep(Duration::from_millis(5));
            }
        }
        let _ = PeekMessageW(&mut MSG::default(), None, 0, 0, PM_NOREMOVE);
        let thread = GetCurrentThreadId();
        let foreground = GetForegroundWindow();
        let owner = if foreground.0.is_null() { 0 } else { GetWindowThreadProcessId(foreground, None) };
        let attached = owner != 0 && owner != thread && AttachThreadInput(thread, owner, true).as_bool();
        let focused = SetForegroundWindow(h).as_bool();
        let _ = BringWindowToTop(h);
        if attached {
            let _ = AttachThreadInput(thread, owner, false);
        }
        focused || GetForegroundWindow() == h
    }
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
    if capture_in_progress() { return; }
    let state = app.state::<notch::NotchState>();
    let settings = state.settings.lock().unwrap().clone();
    sync(&app, &settings, state.fullscreen.load(std::sync::atomic::Ordering::SeqCst));
}

#[tauri::command]
pub async fn dock_launch(path: String, args: String, aumid: String) -> Result<(), String> {
    tauri::async_runtime::spawn_blocking(move || dock_launch_blocking(path, args, aumid)).await.map_err(|e| e.to_string())?
}

pub(crate) fn dock_launch_blocking(path: String, args: String, aumid: String) -> Result<(), String> {
    // ShellExecute preserves the original Windows command line (quoted profile/document paths)
    // and handles shortcuts. Launch a desktop executable directly even if it has an AUMID.
    let target = if !aumid.trim().is_empty() && (path.is_empty() || path.starts_with("shell:")) {
        format!("shell:AppsFolder\\{aumid}")
    } else { path };
    let result = unsafe {
        windows::Win32::UI::Shell::ShellExecuteW(None, w!("open"), &windows::core::HSTRING::from(&target),
            &windows::core::HSTRING::from(args), None, windows::Win32::UI::WindowsAndMessaging::SW_SHOWNORMAL)
    };
    (result.0 as isize > 32).then_some(()).ok_or_else(|| format!("Windows no pudo abrir {target} ({}).", result.0 as isize))
}

#[tauri::command]
pub async fn dock_activate(app: AppHandle, key: String, state: tauri::State<'_, notch::NotchState>) -> Result<bool, String> {
    let clicked_foreground = unsafe {
        let foreground = GetForegroundWindow();
        (!foreground.0.is_null() && !IsIconic(foreground).as_bool()).then_some(foreground.0 as isize)
    };
    let clicked_after_activation = clicked_foreground.is_some_and(recently_activated);
    let settings = state.settings.lock().unwrap().clone();
    tauri::async_runtime::spawn_blocking(move || {
        let windows = windows_for(&settings, &key);
        if windows.is_empty() {
            // Sin ventanas: lanzar si es un elemento anclado.
            if let Some(pin) = settings.dock.pinned.iter().find(|p| pin_key(p) == key) {
                // ShellExecute puede esperar a que la app entre en reposo. Inicia el rebote cuando
                // se confirma que hace falta lanzar, antes de esperar el retorno de Windows.
                if settings.dock.animate_apps && crate::dock_animation::enabled() {
                    let _ = app.emit_to("dock", "dock-app-launching", (&key, 0));
                }
                return dock_launch_blocking(pin.path.clone(), pin.args.clone(), pin.aumid.clone()).map(|_| true);
            }
            return Ok(false);
        }
        let target = {
            let mut activations = ACTIVATIONS.lock().unwrap();
            activations.retain(|_, at| at.elapsed() < ACTIVATION_GUARD);
            let focused = clicked_foreground.filter(|&raw| unsafe {
                let foreground = HWND(raw as *mut _);
                GetForegroundWindow() == foreground && !IsIconic(foreground).as_bool()
            });
            let index = windows.iter().position(|&raw| Some(raw) == focused);
            let target = match index {
                Some(i) if windows.len() == 1 => {
                    let raw = windows[i];
                    if !clicked_after_activation && !activations.contains_key(&raw) {
                        unsafe { let _ = ShowWindowAsync(HWND(raw as *mut _), SW_MINIMIZE); }
                    }
                    return Ok(false);
                }
                Some(i) => windows[(i + 1) % windows.len()],
                None => windows[0],
            };
            activations.insert(target, Instant::now());
            target
        };
        focus_window(target);
        Ok(false)
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
    // `name` is accepted for IPC compatibility only; callers may pass a dynamic window title.
    let _ = name;
    let name_path = path.clone(); let name_aumid = aumid.clone();
    let name = tauri::async_runtime::spawn_blocking(move || icons::program_name(&name_path, &name_aumid)).await.map_err(|e| e.to_string())?;
    if !s.dock.pinned.iter().any(|p| p.path == path && p.aumid == aumid) { s.dock.pinned.push(DockPinned { name, path, args, aumid }); }
    commit_pin(&app, s).await
}

#[tauri::command]
pub async fn dock_unpin(app: AppHandle, key: String) -> Result<(), String> {
    let state = app.state::<notch::NotchState>(); let mut s = state.settings.lock().unwrap().clone();
    s.dock.pinned.retain(|p| pin_key(p) != key && p.path != key);
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
    if code == 0 && !op.fAnyOperationsAborted.as_bool() { Ok(()) } else { Err(format!("No se pudieron enviar los elementos a la papelera ({}).", code)) }
}

/// Actualiza la lista de aplicaciones abiertas fuera del sondeo de clics.
fn start_maintenance(app: AppHandle) {
    std::thread::spawn(move || {
        let mut last_signature: Option<Vec<String>> = None;
        loop {
            std::thread::sleep(Duration::from_millis(1000));
            let settings = app.state::<notch::NotchState>().settings.lock().unwrap().clone();
            if !settings.dock.enabled { continue; }
            let Some(window) = app.get_webview_window("dock") else { continue };
            let Some(h) = hwnd(&window) else { continue };
            crate::dropzone::ensure(&app, &window);
            if !unsafe { IsWindowVisible(h).as_bool() } { continue; }
            let current = items(&settings);
            let signature = current.iter().map(|item| format!("{}:{}:{}", item.key, item.window_count, item.running)).collect::<Vec<_>>();
            if last_signature.as_ref() != Some(&signature) {
                let _ = app.emit("dock-windows", &current);
                last_signature = Some(signature);
            }
        }
    });
}

/// Sondeo económico: solo ajusta click-through y auto-ocultar cuando el dock está visible.
/// Este bucle no hace ninguna llamada que pueda esperar a otro proceso.
pub fn start(app: AppHandle) {
    start_taskbar_watch();
    sync_taskbar(&app.state::<notch::NotchState>().settings.lock().unwrap());
    start_maintenance(app.clone());
    crate::dock_preview::watch(app.clone());
    std::thread::spawn(move || {
        let mut last: Option<(bool, bool)> = None;
        let mut last_check = Instant::now();
        let mut was_capturing = false;
        loop {
            let dock_settings = app.state::<notch::NotchState>().settings.lock().unwrap().dock.clone();
            let interval = if crate::dock_drop::left_button_down() { 25 } else if dock_settings.autohide { 120 } else { 250 };
            std::thread::sleep(Duration::from_millis(interval));
            let Some(window) = app.get_webview_window("dock") else { continue };
            if !dock_settings.enabled { continue; }
            // Win32 directo: nada de esperar al hilo de UI (antes un recv() sin límite
            // podía dejar este bucle bloqueado y el Dock sin responder).
            let Some(h) = hwnd(&window) else { continue };
            let visible = unsafe { IsWindowVisible(h).as_bool() };
            // Al terminar un recorte (Impr Pant, Win+Shift+S, Recortes) se reaplica todo al momento:
            // visibilidad, click-through, orden Z, área de clic y estado de la vista.
            let capturing = capture_in_progress();
            let recovered = was_capturing && !capturing;
            was_capturing = capturing;
            if capturing || !CAN_SHOW.load(std::sync::atomic::Ordering::SeqCst) {
                last = None;
                continue;
            }
            if recovered {
                last = None;
                last_check = Instant::now() - Duration::from_secs(10);
                let _ = window.emit("dock-recover", ());
            }
            let mut rect = RECT::default();
            let has_rect = unsafe { GetWindowRect(h, &mut rect).is_ok() };
            let hit_area = *hit().lock().unwrap();
            let mut p = windows::Win32::Foundation::POINT::default();
            unsafe { let _ = GetCursorPos(&mut p); }
            crate::dock_drop::poll(&app, h, p);
            let inside = has_rect && hit_area.w > 0 && {
                let x = ((p.x - rect.left) as f64 / hit_area.scale) as i32;
                let y = ((p.y - rect.top) as f64 / hit_area.scale) as i32;
                x >= hit_area.x && x < hit_area.x + hit_area.w && y >= hit_area.y && y < hit_area.y + hit_area.h
            };
            let near_bottom = work_area_with_monitor(&app, dock_settings.hide_taskbar).map(|(r, scale)| p.y >= r.bottom - physical(12, scale) && p.x >= r.left && p.x < r.right).unwrap_or(false);
            let menu_open = menu_open();
            // Cada 2 s se reaplica todo aunque nada haya cambiado: si Windows o tao
            // alteraron los estilos (clics ignorados) o el marco de grabación tapó el Dock, se corrige solo.
            let periodic = last_check.elapsed() >= Duration::from_secs(2);
            if periodic && (!menu_open || screen_recording_active()) {
                last_check = Instant::now();
                if visible {
                    unsafe { let _ = SetWindowPos(h, Some(HWND_TOPMOST), 0, 0, 0, 0, SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE | SWP_ASYNCWINDOWPOS); }
                }
            }
            if menu_open { continue; }
            let should_show = !dock_settings.autohide || inside || near_bottom || crate::shortcut_folder::is_open() || crate::dock_animation::in_flight();
            // El cursor sale del área de clic: la ventana pasa a click-through y WebView2 ya no recibe
            // `mouseleave`, así que se avisa a la vista (cierra el menú contextual y la magnificación).
            if last.is_some_and(|(_, was_inside)| was_inside) && !inside {
                let _ = window.emit("dock-leave", ());
            }
            if !periodic && last == Some((should_show, inside)) { continue; }
            last = Some((should_show, inside));
            let w = window.clone();
            let _ = window.run_on_main_thread(move || {
                if !CAN_SHOW.load(std::sync::atomic::Ordering::SeqCst) { return; }
                if should_show { let _ = w.show(); } else { let _ = w.hide(); }
                let _ = w.set_ignore_cursor_events(!inside);
                // tao reescribe GWL_EXSTYLE entero al cambiar el click-through: se repone justo después.
                if let Some(h) = hwnd(&w) { window_utils::enforce_chrome_free_hwnd(h); }
            });
        }
    });
}

#[cfg(test)]
mod order_tests {
    use super::*;
    fn item(key: &str) -> DockItem { DockItem { key: key.into(), name: key.into(), path: String::new(), args: String::new(), aumid: String::new(), running: true, window_count: 1, kind: "running".into(), is_downloads: false } }
    #[test]
    fn activation_does_not_reorder_running_apps() {
        let mut order = vec!["a".into(), "b".into()];
        let mut items = vec![item("b"), item("c"), item("a")];
        stable_running_order(&mut order, &mut items);
        assert_eq!(items.iter().map(|i| i.key.as_str()).collect::<Vec<_>>(), ["a", "b", "c"]);
    }
    #[test]
    fn reopened_app_appends_after_survivors() {
        let mut order = vec!["a".into(), "b".into()];
        stable_running_order(&mut order, &mut [item("b")]);
        let mut items = vec![item("a"), item("b")];
        stable_running_order(&mut order, &mut items);
        assert_eq!(items.iter().map(|i| i.key.as_str()).collect::<Vec<_>>(), ["b", "a"]);
    }
}

#[cfg(test)]
mod pin_name_tests {
    use super::*;
    #[test]
    fn migration_discards_tab_titles_preserves_launch_data_and_order() {
        let mut settings = Settings::default();
        settings.dock.pinned = vec![
            DockPinned { name: "Corrección y unificación - Google Gemini - Brave".into(), path: r"C:\NokiMissing\brave.exe".into(), args: "--profile-directory=Work".into(), aumid: String::new() },
            DockPinned { name: "Draft.txt - Editor".into(), path: r"C:\NokiMissing\editor.exe".into(), args: String::new(), aumid: String::new() },
        ];
        let before = settings.dock.pinned.clone();
        let key = pin_key(&before[0]);
        assert!(normalize_pinned_names(&mut settings));
        assert_eq!(settings.dock.pinned.iter().map(|pin| pin.name.as_str()).collect::<Vec<_>>(), ["Brave", "Editor"]);
        for (old, new) in before.iter().zip(&settings.dock.pinned) {
            assert_eq!((&old.path, &old.args, &old.aumid), (&new.path, &new.args, &new.aumid));
        }
        assert_eq!(key, pin_key(&settings.dock.pinned[0]));
        assert!(!normalize_pinned_names(&mut settings));
    }
}

#[cfg(test)]
mod window_filter_tests {
    use super::*;

    #[test]
    fn shell_infrastructure_is_never_an_independent_app() {
        for executable in ["ApplicationFrameHost.exe", "SearchHost.exe", "StartMenuExperienceHost.exe", "ShellExperienceHost.exe", "TextInputHost.exe", "LockApp.exe"] {
            assert!(icons::is_shell_host(&format!(r"C:\Windows\System32\{executable}")));
        }
        assert!(!icons::is_shell_host(r"C:\Windows\ImmersiveControlPanel\SystemSettings.exe"));
        assert!(!icons::is_shell_host(r"C:\Windows\explorer.exe"));
    }

    #[test]
    #[ignore = "Requires Settings and the Recycle Bin open on the interactive Windows desktop"]
    fn live_settings_and_recycle_bin_have_no_duplicate_entries() {
        let windows = running_windows();
        assert!(windows.iter().all(|w| !icons::is_shell_host(&w.path)));
        let settings_window = windows.iter().find(|w| file_name(&w.path) == "systemsettings.exe").expect("Open Windows Settings first");
        assert!(windows.iter().any(|w| w.recycle_bin), "Open the Recycle Bin first; its namespace must be recognized");
        let mut settings = Settings::default();
        settings.dock.pinned.clear();
        settings.dock.show_running = true;
        settings.dock.show_trash = true;
        let key = format!("app:{}", identity(&settings_window.path, &settings_window.aumid));
        let unpinned = items(&settings);
        assert_eq!(unpinned.iter().filter(|i| i.key == key).count(), 1);
        let bin_only: Vec<_> = windows.iter().filter(|w| w.recycle_bin).collect();
        for bin in bin_only {
            if !windows.iter().any(|w| !w.recycle_bin && identity(&w.path, &w.aumid) == identity(&bin.path, &bin.aumid)) {
                assert!(!unpinned.iter().any(|i| i.key == format!("app:{}", identity(&bin.path, &bin.aumid))));
            }
        }
        settings.dock.pinned.push(DockPinned { name: "Settings".into(), path: settings_window.path.clone(), args: String::new(), aumid: settings_window.aumid.clone() });
        let pinned = items(&settings);
        assert!(pinned[0].running);
        assert_eq!(pinned.iter().filter(|i| i.aumid == settings_window.aumid && i.path.to_lowercase().contains("systemsettings.exe")).count(), 1);
        assert!(!pinned.iter().any(|i| i.key == key));
        println!("PASS: real Settings app merges with its pin; Recycle Bin namespace belongs to the fixed metallic icon; no infrastructure processes.");
    }
}
