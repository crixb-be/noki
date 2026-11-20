//! Efecto Genie para las ventanas que tienen un icono en el Dock.
use crate::{dock, dock_drop, dock_preview, notch::NotchState, window_utils};
use std::{
    collections::HashMap,
    sync::{
        atomic::{AtomicBool, AtomicU64, Ordering},
        Mutex,
    },
    time::{Duration, Instant},
};
use tauri::{window::Color, AppHandle, Emitter, Manager, WebviewWindow, WebviewWindowBuilder};
use windows::Win32::{
    Foundation::{COLORREF, HWND, RECT},
    Graphics::{
        Dwm::{DwmGetWindowAttribute, DwmSetWindowAttribute, DWMWA_CLOAK, DWMWA_CLOAKED},
        Gdi::{RedrawWindow, RDW_ALLCHILDREN, RDW_FRAME, RDW_INVALIDATE},
    },
    UI::{
        Accessibility::{SetWinEventHook, UnhookWinEvent, HWINEVENTHOOK},
        WindowsAndMessaging::{
            GetLayeredWindowAttributes, GetWindowDisplayAffinity, GetWindowLongPtrW, GetWindowRect,
            GetWindowThreadProcessId, IsIconic, SetLayeredWindowAttributes, SetWindowLongPtrW,
            SetWindowPos, SystemParametersInfoW, ANIMATIONINFO, EVENT_SYSTEM_MINIMIZEEND,
            EVENT_SYSTEM_MINIMIZESTART, GWL_EXSTYLE, HWND_TOPMOST, LAYERED_WINDOW_ATTRIBUTES_FLAGS,
            LWA_ALPHA, SPI_GETANIMATION, SPI_SETANIMATION, SWP_NOACTIVATE,
            SYSTEM_PARAMETERS_INFO_UPDATE_FLAGS, WDA_NONE, WINEVENT_OUTOFCONTEXT,
            WINEVENT_SKIPOWNPROCESS, WS_EX_LAYERED,
        },
    },
};

const LABEL: &str = "dockgenie";
const DURATION_MS: u32 = 250;
const CACHE_PERIOD: Duration = Duration::from_millis(1500);
const MAX_SNAPSHOTS: usize = 24;

static ENABLED: AtomicBool = AtomicBool::new(false);
static GENERATION: AtomicU64 = AtomicU64::new(0);
static NEXT_ID: AtomicU64 = AtomicU64::new(0);
static HOOK: Mutex<Option<isize>> = Mutex::new(None);
static EVENT_APP: Mutex<Option<(AppHandle, u64)>> = Mutex::new(None);
static SNAPSHOTS: std::sync::LazyLock<Mutex<HashMap<isize, Snapshot>>> =
    std::sync::LazyLock::new(|| Mutex::new(HashMap::new()));
static CURRENT: Mutex<Option<Transition>> = Mutex::new(None);
static CREATE_OVERLAY: Mutex<()> = Mutex::new(());
static NATIVE_ANIMATION: Mutex<Option<i32>> = Mutex::new(None);

#[derive(Clone)]
struct Snapshot {
    hwnd: isize,
    pid: u32,
    bounds: RECT,
    image: String,
    key: String,
    captured_at: Instant,
    minimized: bool,
}

struct Transition {
    frame: Animation,
    _cloak: Option<Cloak>,
}

struct Cloak {
    hwnd: isize,
    pid: u32,
    method: Concealment,
}

enum Concealment {
    Dwm,
    Opacity(COLORREF, u8, LAYERED_WINDOW_ATTRIBUTES_FLAGS),
    TemporaryLayer,
}

impl Cloak {
    fn new(raw: isize, pid: u32) -> Option<Self> {
        let hwnd = HWND(raw as *mut _);
        if window_pid(hwnd) != Some(pid) {
            return None;
        }
        let mut state = 0u32;
        unsafe {
            DwmGetWindowAttribute(hwnd, DWMWA_CLOAKED, (&mut state as *mut u32).cast(), 4).ok()?;
        }
        if state != 0 {
            return None;
        }
        let method = if set_cloak(hwnd, true) {
            Concealment::Dwm
        } else {
            Self::make_transparent(hwnd)?
        };
        Some(Self {
            hwnd: raw,
            pid,
            method,
        })
    }

    fn make_transparent(hwnd: HWND) -> Option<Concealment> {
        // Windows puede denegar DWMWA_CLOAK entre procesos. La opacidad deja el HWND restaurado
        // con su foco y posición, y se revierte usando el estilo o atributos que tenía al entrar.
        unsafe {
            let style = GetWindowLongPtrW(hwnd, GWL_EXSTYLE);
            let layered = WS_EX_LAYERED.0 as isize;
            if style & layered != 0 {
                let (mut key, mut alpha, mut flags) =
                    (COLORREF(0), 255, LAYERED_WINDOW_ATTRIBUTES_FLAGS(0));
                // Una ventana con alfa por píxel no expone estos atributos: deja su superficie intacta.
                GetLayeredWindowAttributes(
                    hwnd,
                    Some(&mut key),
                    Some(&mut alpha),
                    Some(&mut flags),
                )
                .ok()?;
                SetLayeredWindowAttributes(hwnd, key, 0, flags | LWA_ALPHA).ok()?;
                Some(Concealment::Opacity(key, alpha, flags))
            } else {
                SetWindowLongPtrW(hwnd, GWL_EXSTYLE, style | layered);
                if GetWindowLongPtrW(hwnd, GWL_EXSTYLE) & layered == 0 {
                    return None;
                }
                if SetLayeredWindowAttributes(hwnd, COLORREF(0), 0, LWA_ALPHA).is_err() {
                    Self::remove_temporary_layer(hwnd);
                    return None;
                }
                Some(Concealment::TemporaryLayer)
            }
        }
    }

    fn remove_temporary_layer(hwnd: HWND) {
        unsafe {
            let style = GetWindowLongPtrW(hwnd, GWL_EXSTYLE);
            SetWindowLongPtrW(hwnd, GWL_EXSTYLE, style & !(WS_EX_LAYERED.0 as isize));
            let _ = RedrawWindow(
                Some(hwnd),
                None,
                None,
                RDW_INVALIDATE | RDW_FRAME | RDW_ALLCHILDREN,
            );
        }
    }
}

impl Drop for Cloak {
    fn drop(&mut self) {
        let hwnd = HWND(self.hwnd as *mut _);
        if window_pid(hwnd) != Some(self.pid) {
            return;
        }
        match self.method {
            Concealment::Dwm => {
                set_cloak(hwnd, false);
            }
            Concealment::Opacity(key, alpha, flags) => unsafe {
                let _ = SetLayeredWindowAttributes(hwnd, key, alpha, flags);
            },
            Concealment::TemporaryLayer => Self::remove_temporary_layer(hwnd),
        }
    }
}

fn set_cloak(hwnd: HWND, cloak: bool) -> bool {
    let value = i32::from(cloak);
    unsafe { DwmSetWindowAttribute(hwnd, DWMWA_CLOAK, (&value as *const i32).cast(), 4).is_ok() }
}

fn window_pid(hwnd: HWND) -> Option<u32> {
    let mut pid = 0;
    unsafe {
        GetWindowThreadProcessId(hwnd, Some(&mut pid));
    }
    (pid != 0).then_some(pid)
}

fn animation_info(minimize: i32) -> ANIMATIONINFO {
    ANIMATIONINFO {
        cbSize: std::mem::size_of::<ANIMATIONINFO>() as u32,
        iMinAnimate: minimize,
    }
}

fn set_native_animation(info: &mut ANIMATIONINFO) -> bool {
    // Sin SPIF_UPDATEINIFILE: el ajuste de Windows solo cambia en esta sesión.
    unsafe {
        SystemParametersInfoW(
            SPI_SETANIMATION,
            info.cbSize,
            Some((info as *mut ANIMATIONINFO).cast()),
            SYSTEM_PARAMETERS_INFO_UPDATE_FLAGS(0),
        )
        .is_ok()
    }
}

fn suspend_native_animation() -> bool {
    let mut previous = NATIVE_ANIMATION.lock().unwrap();
    if previous.is_some() {
        return true;
    }
    let mut info = animation_info(0);
    if unsafe {
        SystemParametersInfoW(
            SPI_GETANIMATION,
            info.cbSize,
            Some((&mut info as *mut ANIMATIONINFO).cast()),
            SYSTEM_PARAMETERS_INFO_UPDATE_FLAGS(0),
        )
        .is_err()
    } {
        return false;
    }
    if info.iMinAnimate == 0 {
        return true;
    }
    if !set_native_animation(&mut animation_info(0)) {
        return false;
    }
    *previous = Some(info.iMinAnimate);
    true
}

pub(crate) fn restore_system() {
    if let Ok(mut current) = CURRENT.try_lock() {
        current.take();
    }
    if let Ok(mut previous) = NATIVE_ANIMATION.try_lock() {
        if let Some(original) = *previous {
            if set_native_animation(&mut animation_info(original)) {
                *previous = None;
            }
        }
    }
}

#[derive(Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Animation {
    id: u64,
    image: String,
    source: [i32; 4],
    target: [i32; 4],
    width: i32,
    height: i32,
    restoring: bool,
    duration: u32,
}

pub(crate) fn enabled() -> bool {
    ENABLED.load(Ordering::SeqCst)
}

pub(crate) fn in_flight() -> bool {
    enabled() && CURRENT.lock().unwrap().is_some()
}

fn active(generation: u64) -> bool {
    enabled() && GENERATION.load(Ordering::SeqCst) == generation
}

pub(crate) fn sync(app: &AppHandle, enable: bool) {
    let enable = enable
        && app
            .state::<NotchState>()
            .settings
            .lock()
            .unwrap()
            .dock
            .animate_apps;
    if ENABLED.swap(enable, Ordering::SeqCst) == enable {
        return;
    }
    let generation = GENERATION.fetch_add(1, Ordering::SeqCst) + 1;
    let handle = app.clone();
    let _ = app.run_on_main_thread(move || {
        if GENERATION.load(Ordering::SeqCst) != generation || enabled() != enable {
            return;
        }
        EVENT_APP.lock().unwrap().take();
        if let Some(hook) = HOOK.lock().unwrap().take() {
            unsafe {
                let _ = UnhookWinEvent(HWINEVENTHOOK(hook as *mut _));
            }
        }
        SNAPSHOTS.lock().unwrap().clear();
        restore_system();
        if !enable {
            if let Some(window) = handle.get_webview_window(LABEL) {
                let _ = window.hide();
                let _ = window.destroy();
            }
            return;
        }
        tauri::async_runtime::spawn_blocking(move || {
            // WebView2 puede reentrar al bucle de Windows durante la creación. No se construye
            // dentro del callback que registra o retira los hooks de ese mismo bucle.
            let _creating = CREATE_OVERLAY.lock().unwrap();
            if !active(generation) {
                return;
            }
            let Some(window) = overlay(&handle) else {
                return;
            };
            if !active(generation) {
                if !enabled() {
                    let _ = window.destroy();
                }
                return;
            }
            let _ = window.hide();
            let registration = handle.clone();
            let _ = handle.run_on_main_thread(move || install_hooks(registration, generation));
        });
    });
}

fn install_hooks(handle: AppHandle, generation: u64) {
    if !active(generation) {
        return;
    }
    let flags = WINEVENT_OUTOFCONTEXT | WINEVENT_SKIPOWNPROCESS;
    let hook = unsafe {
        SetWinEventHook(
            EVENT_SYSTEM_MINIMIZESTART,
            EVENT_SYSTEM_MINIMIZEEND,
            None,
            Some(on_event),
            0,
            0,
            flags,
        )
    };
    if hook.is_invalid() {
        if let Some(window) = handle.get_webview_window(LABEL) {
            let _ = window.destroy();
        }
        return;
    }
    if !suspend_native_animation() {
        unsafe {
            let _ = UnhookWinEvent(hook);
        }
        if let Some(window) = handle.get_webview_window(LABEL) {
            let _ = window.destroy();
        }
        return;
    }
    *HOOK.lock().unwrap() = Some(hook.0 as isize);
    *EVENT_APP.lock().unwrap() = Some((handle.clone(), generation));
    std::thread::spawn(move || refresh_cache(handle, generation));
}

fn overlay(app: &AppHandle) -> Option<WebviewWindow> {
    if let Some(window) = app.get_webview_window(LABEL) {
        return Some(window);
    }
    let window = WebviewWindowBuilder::new(app, LABEL, window_utils::app_url(LABEL))
        .title("")
        .inner_size(64.0, 64.0)
        .decorations(false)
        .transparent(true)
        .background_color(Color(0, 0, 0, 0))
        .always_on_top(true)
        .resizable(false)
        .skip_taskbar(true)
        .shadow(false)
        .focused(false)
        .focusable(false)
        .visible(false)
        .on_navigation(|url| window_utils::route_is_valid(url, LABEL))
        .on_page_load(|window, payload| {
            window_utils::repair_page_load(window, payload.event(), payload.url(), LABEL)
        })
        .build()
        .ok()?;
    let _ = window.set_ignore_cursor_events(true);
    Some(window)
}

unsafe extern "system" fn on_event(
    _hook: HWINEVENTHOOK,
    kind: u32,
    hwnd: HWND,
    object: i32,
    child: i32,
    _thread: u32,
    _time: u32,
) {
    if hwnd.0.is_null() || object != 0 || child != 0 || !dock::can_show() {
        return;
    }
    let restoring = match kind {
        EVENT_SYSTEM_MINIMIZESTART => false,
        EVENT_SYSTEM_MINIMIZEEND => true,
        _ => return,
    };
    let Some((app, generation)) = EVENT_APP.lock().unwrap().clone() else {
        return;
    };
    if !active(generation) {
        return;
    }
    let raw = hwnd.0 as isize;
    if !restoring && dock::recently_activated(raw) {
        return;
    }
    let snapshot = {
        let mut cache = SNAPSHOTS.lock().unwrap();
        let Some(snapshot) = cache.get_mut(&raw) else {
            return;
        };
        if window_pid(hwnd) != Some(snapshot.pid) || snapshot.minimized != restoring {
            return;
        }
        snapshot.minimized = !restoring;
        if !restoring && snapshot.captured_at.elapsed() > CACHE_PERIOD * 3 {
            return;
        }
        snapshot.clone()
    };
    let Some(target) = dock_drop::animation_anchor(&app, &snapshot.key) else {
        return;
    };
    present(&app, generation, snapshot, target, restoring);
}

fn visible_rect(hwnd: HWND) -> Option<RECT> {
    let mut rect = RECT::default();
    unsafe {
        if IsIconic(hwnd).as_bool() || GetWindowRect(hwnd, &mut rect).is_err() {
            return None;
        }
    }
    (rect.right > rect.left && rect.bottom > rect.top).then_some(rect)
}

fn refresh_cache(app: AppHandle, generation: u64) {
    while active(generation) {
        let next = Instant::now() + CACHE_PERIOD;
        if dock::can_show() && !dock::capture_in_progress() {
            collect_snapshots(&app, generation);
        }
        std::thread::sleep(next.saturating_duration_since(Instant::now()));
    }
}

fn collect_snapshots(app: &AppHandle, generation: u64) {
    let settings = app.state::<NotchState>().settings.lock().unwrap().clone();
    let windows = dock::animation_windows(&settings);
    {
        let mut cache = SNAPSHOTS.lock().unwrap();
        if !active(generation) {
            return;
        }
        cache.retain(|raw, snapshot| {
            window_pid(HWND(*raw as *mut _)) == Some(snapshot.pid)
                && (snapshot.minimized || snapshot.captured_at.elapsed() < CACHE_PERIOD * 6)
        });
    }
    for (raw, key) in windows.into_iter().take(MAX_SNAPSHOTS) {
        if !active(generation) {
            return;
        }
        let hwnd = HWND(raw as *mut _);
        if unsafe { IsIconic(hwnd).as_bool() } {
            if let Some(snapshot) = SNAPSHOTS.lock().unwrap().get_mut(&raw) {
                snapshot.minimized = true;
            }
            continue;
        }
        let Some(bounds) = visible_rect(hwnd) else {
            continue;
        };
        let Some(pid) = window_pid(hwnd) else {
            continue;
        };
        let mut affinity = 0;
        if unsafe { GetWindowDisplayAffinity(hwnd, &mut affinity).is_ok() }
            && affinity != WDA_NONE.0
        {
            SNAPSHOTS.lock().unwrap().remove(&raw);
            continue;
        }
        let Some(image) = dock_preview::capture_window(raw, 800, 540) else {
            continue;
        };
        if !active(generation) || unsafe { IsIconic(hwnd).as_bool() } {
            continue;
        }
        let mut cache = SNAPSHOTS.lock().unwrap();
        if !active(generation) || cache.get(&raw).is_some_and(|item| item.minimized) {
            continue;
        }
        if cache.len() >= MAX_SNAPSHOTS && !cache.contains_key(&raw) {
            if let Some(oldest) = cache
                .iter()
                .min_by_key(|(_, item)| item.captured_at)
                .map(|(&raw, _)| raw)
            {
                cache.remove(&oldest);
            }
        }
        cache.insert(
            raw,
            Snapshot {
                hwnd: raw,
                pid,
                bounds,
                image,
                key,
                captured_at: Instant::now(),
                minimized: false,
            },
        );
    }
}

fn present(app: &AppHandle, generation: u64, snapshot: Snapshot, target: RECT, restoring: bool) {
    let hwnd = HWND(snapshot.hwnd as *mut _);
    let source = visible_rect(hwnd).unwrap_or(snapshot.bounds);
    let left = source.left.min(target.left);
    let top = source.top.min(target.top);
    let width = source.right.max(target.right) - left;
    let height = source.bottom.max(target.bottom) - top;
    if width <= 0 || height <= 0 || i64::from(width) * i64::from(height) > 24_000_000 {
        return;
    }
    let relative = |bounds: RECT| {
        [
            bounds.left - left,
            bounds.top - top,
            bounds.right - left,
            bounds.bottom - top,
        ]
    };
    let id = NEXT_ID.fetch_add(1, Ordering::SeqCst) + 1;
    let (raw, pid) = (snapshot.hwnd, snapshot.pid);
    let animation = Animation {
        id,
        image: snapshot.image,
        source: relative(source),
        target: relative(target),
        width,
        height,
        restoring,
        duration: DURATION_MS,
    };
    let handle = app.clone();
    let _ = app.run_on_main_thread(move || {
        if !active(generation) || !dock::can_show() || (!restoring && dock::recently_activated(raw)) {
            return;
        }
        let Some(window) = handle.get_webview_window(LABEL) else {
            return;
        };
        let Some(hwnd) = dock::hwnd(&window) else {
            return;
        };
        if unsafe {
            SetWindowPos(
                hwnd,
                Some(HWND_TOPMOST),
                left,
                top,
                width,
                height,
                SWP_NOACTIVATE,
            )
        }
        .is_err()
        {
            return;
        }
        // Retira la transición anterior antes de cloaking otro HWND, incluso en clics rápidos.
        let previous = CURRENT.lock().unwrap().take();
        if previous.is_some() {
            let _ = window.hide();
        }
        drop(previous);
        let cloak = if restoring {
            let Some(cloak) = Cloak::new(raw, pid) else {
                return;
            };
            Some(cloak)
        } else {
            None
        };
        if window.show().is_err() {
            return;
        }
        *CURRENT.lock().unwrap() = Some(Transition {
            frame: animation.clone(),
            _cloak: cloak,
        });
        let _ = window.set_ignore_cursor_events(true);
        window_utils::enforce_chrome_free_hwnd(hwnd);
        let _ = window.emit_to(LABEL, "dock-genie", animation);
        tauri::async_runtime::spawn(async move {
            tokio::time::sleep(Duration::from_millis(u64::from(DURATION_MS) + 110)).await;
            finish(&handle, id);
        });
    });
}

fn finish(app: &AppHandle, id: u64) {
    let transition = {
        let mut current = CURRENT.lock().unwrap();
        if !current
            .as_ref()
            .is_some_and(|transition| transition.frame.id == id)
        {
            return;
        }
        current.take()
    };
    // El watchdog también libera la ventana real si WebView2 no llega a completar el frame.
    drop(transition);
    let handle = app.clone();
    let _ = app.run_on_main_thread(move || {
        if CURRENT.lock().unwrap().is_none() {
            if let Some(window) = handle.get_webview_window(LABEL) {
                let _ = window.hide();
            }
        }
    });
}

#[tauri::command]
pub fn dock_genie_state(window: WebviewWindow) -> Option<Animation> {
    if window.label() != LABEL || !enabled() {
        return None;
    }
    CURRENT
        .lock()
        .unwrap()
        .as_ref()
        .map(|transition| transition.frame.clone())
}

#[tauri::command]
pub fn dock_genie_finished(window: WebviewWindow, id: u64) {
    if window.label() == LABEL {
        finish(window.app_handle(), id);
    }
}
