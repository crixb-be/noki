//! Ventana flotante y miniaturas de las ventanas asociadas a un icono del Dock.
use crate::{dock, notch::NotchState, window_utils};
use base64::{engine::general_purpose::STANDARD, Engine};
use std::{
    io::Cursor,
    sync::Mutex,
    time::{Duration, Instant},
};
use tauri::{
    window::Color, AppHandle, Emitter, Manager, WebviewUrl, WebviewWindow, WebviewWindowBuilder,
};
use windows::Win32::{
    Foundation::{HWND, LPARAM, POINT, RECT, WPARAM},
    Graphics::Gdi::{
        CreateCompatibleDC, CreateDIBSection, DeleteDC, DeleteObject, GetMonitorInfoW,
        MonitorFromWindow, SelectObject, BITMAPINFO, BITMAPINFOHEADER, BI_RGB, DIB_RGB_COLORS,
        HGDIOBJ, MONITORINFO, MONITOR_DEFAULTTONEAREST,
    },
    Storage::Xps::{PrintWindow, PRINT_WINDOW_FLAGS},
    UI::WindowsAndMessaging::{
        GetCursorPos, GetWindowPlacement, GetWindowRect, IsIconic, IsWindow, PostMessageW,
        SendMessageTimeoutW, SetWindowPos, HWND_TOPMOST, PW_RENDERFULLCONTENT, SMTO_ABORTIFHUNG,
        SWP_NOACTIVATE, WINDOWPLACEMENT, WM_CLOSE, WM_NULL,
    },
};

#[derive(serde::Deserialize)]
pub struct Anchor {
    left: i32,
    top: i32,
    right: i32,
    bottom: i32,
}

#[derive(Clone, serde::Serialize)]
struct Card {
    hwnd: isize,
    title: String,
    image: Option<String>,
}

#[derive(Clone, serde::Serialize)]
pub struct Preview {
    request: u64,
    key: String,
    name: String,
    cards: Vec<Card>,
}

struct Session {
    view: Preview,
    anchor: RECT,
    panel: RECT,
    opened: Instant,
}

static SESSION: Mutex<Option<Session>> = Mutex::new(None);
static REQUEST: Mutex<u64> = Mutex::new(0);
static CAPTURE_WORKERS: Mutex<usize> = Mutex::new(0);

pub(crate) fn ensure(app: &AppHandle) -> Option<WebviewWindow> {
    if let Some(window) = app.get_webview_window("dockpreview") {
        return Some(window);
    }
    WebviewWindowBuilder::new(
        app,
        "dockpreview",
        WebviewUrl::App("index.html?view=dockpreview".into()),
    )
    .title("")
    .inner_size(320.0, 158.0)
    .decorations(false)
    .transparent(true)
    .background_color(Color(0, 0, 0, 0))
    .always_on_top(true)
    .shadow(false)
    .resizable(false)
    .skip_taskbar(true)
    .focused(false)
    .focusable(false)
    .visible(false)
    .build()
    .ok()
}

#[tauri::command]
pub fn dock_preview_state(window: WebviewWindow) -> Option<Preview> {
    if window.label() != "dockpreview" {
        return None;
    }
    SESSION
        .lock()
        .unwrap()
        .as_ref()
        .map(|session| session.view.clone())
}

fn contains(rect: RECT, point: POINT) -> bool {
    point.x >= rect.left && point.x < rect.right && point.y >= rect.top && point.y < rect.bottom
}

fn active(request: u64) -> bool {
    SESSION
        .lock()
        .unwrap()
        .as_ref()
        .is_some_and(|s| s.view.request == request)
}

#[tauri::command]
pub async fn dock_preview_open(
    window: WebviewWindow,
    key: String,
    name: String,
    anchor: Anchor,
) -> Result<(), String> {
    if window.label() != "dock"
        || !dock::can_show()
        || dock::capture_in_progress()
        || dock::menu_open()
    {
        return Ok(());
    }
    let request = {
        let mut generation = REQUEST.lock().unwrap();
        *generation += 1;
        *generation
    };
    let app = window.app_handle().clone();
    let mut dock_rect = RECT::default();
    let mut monitor = MONITORINFO {
        cbSize: std::mem::size_of::<MONITORINFO>() as u32,
        ..Default::default()
    };
    {
        let Some(dock_hwnd) = dock::hwnd(&window) else {
            return Ok(());
        };
        unsafe {
            GetWindowRect(dock_hwnd, &mut dock_rect).map_err(|e| e.to_string())?;
            if !GetMonitorInfoW(
                MonitorFromWindow(dock_hwnd, MONITOR_DEFAULTTONEAREST),
                &mut monitor,
            )
            .as_bool()
            {
                return Ok(());
            }
        }
    }
    let anchor = RECT {
        left: dock_rect.left + anchor.left,
        top: dock_rect.top + anchor.top,
        right: dock_rect.left + anchor.right,
        bottom: dock_rect.top + anchor.bottom,
    };
    let settings = app.state::<NotchState>().settings.lock().unwrap().clone();
    let query_key = key.clone();
    let cards = tauri::async_runtime::spawn_blocking(move || {
        dock::window_titles_for(&settings, &query_key)
            .into_iter()
            .map(|(hwnd, title)| Card {
                hwnd,
                title,
                image: None,
            })
            .collect::<Vec<_>>()
    })
    .await
    .map_err(|e| e.to_string())?;
    let view = Preview {
        request,
        key,
        name,
        cards,
    };
    let scale = window.scale_factor().unwrap_or(1.0);
    let px = |logical: i32| (logical as f64 * scale).round() as i32;
    let area = monitor.rcMonitor;
    let width = if view.cards.is_empty() {
        px((view.name.chars().count().min(40) as i32 * 8 + 34).clamp(70, 355))
    } else {
        px(16 + 176 * view.cards.len().min(4) as i32)
    }
    .min(area.right - area.left);
    let height = px(if view.cards.is_empty() { 34 } else { 156 });
    let left = ((anchor.left + anchor.right - width) / 2).clamp(area.left, area.right - width);
    let top = (anchor.top - height - px(8)).max(area.top);
    let panel = RECT {
        left,
        top,
        right: left + width,
        bottom: top + height,
    };
    let Some(popup) = ensure(&app) else {
        return Ok(());
    };
    let handles = view.cards.iter().map(|c| c.hwnd).collect::<Vec<_>>();
    let popup_ui = popup.clone();
    popup
        .run_on_main_thread(move || {
            let generation = REQUEST.lock().unwrap();
            if request != *generation || !dock::can_show() || dock::menu_open() {
                return;
            }
            let mut point = POINT::default();
            unsafe {
                let _ = GetCursorPos(&mut point);
            }
            if !contains(anchor, point) {
                return;
            }
            let Some(h) = dock::hwnd(&popup_ui) else {
                return;
            };
            *SESSION.lock().unwrap() = Some(Session {
                view: view.clone(),
                anchor,
                panel,
                opened: Instant::now(),
            });
            unsafe {
                let _ = SetWindowPos(
                    h,
                    Some(HWND_TOPMOST),
                    left,
                    top,
                    width,
                    height,
                    SWP_NOACTIVATE,
                );
            }
            let _ = popup_ui.emit_to("dockpreview", "dock-preview", view);
            let _ = popup_ui.show();
            window_utils::enforce_chrome_free_hwnd(h);
            start_captures(popup_ui, request, handles);
        })
        .map_err(|e| e.to_string())
}

fn start_captures(popup: WebviewWindow, request: u64, handles: Vec<isize>) {
    if handles.is_empty() {
        return;
    }
    {
        let mut workers = CAPTURE_WORKERS.lock().unwrap();
        if *workers >= 2 {
            return;
        }
        *workers += 1;
    }
    tauri::async_runtime::spawn_blocking(move || {
        struct Worker;
        impl Drop for Worker {
            fn drop(&mut self) {
                *CAPTURE_WORKERS.lock().unwrap() -= 1;
            }
        }
        let _worker = Worker;
        for raw in handles {
            if !active(request) {
                break;
            }
            let Some(image) = capture(raw) else {
                continue;
            };
            let mut current = SESSION.lock().unwrap();
            let Some(session) = current
                .as_mut()
                .filter(|session| session.view.request == request)
            else {
                break;
            };
            if let Some(card) = session.view.cards.iter_mut().find(|card| card.hwnd == raw) {
                card.image = Some(image);
                let _ = popup.emit_to("dockpreview", "dock-preview-image", (request, &*card));
            }
        }
    });
}

fn close(app: &AppHandle, expected: Option<u64>) {
    let mut generation = REQUEST.lock().unwrap();
    let mut session = SESSION.lock().unwrap();
    if expected.is_some_and(|expected| session.as_ref().is_none_or(|s| s.view.request != expected))
    {
        return;
    }
    *generation += 1;
    *session = None;
    drop(session);
    let id = *generation;
    drop(generation);
    if let Some(popup) = app.get_webview_window("dockpreview") {
        let w = popup.clone();
        let _ = popup.run_on_main_thread(move || {
            if id == *REQUEST.lock().unwrap() {
                let _ = w.hide();
            }
        });
    }
}

#[tauri::command]
pub fn dock_preview_close(window: WebviewWindow) {
    if matches!(window.label(), "dock" | "dockpreview") {
        close(window.app_handle(), None);
    }
}

#[tauri::command]
pub async fn dock_preview_action(
    window: WebviewWindow,
    request: u64,
    hwnd: isize,
    dismiss: bool,
) -> Result<(), String> {
    if window.label() != "dockpreview" {
        return Ok(());
    }
    let key = SESSION
        .lock()
        .unwrap()
        .as_ref()
        .filter(|session| {
            session.view.request == request && session.view.cards.iter().any(|c| c.hwnd == hwnd)
        })
        .map(|session| session.view.key.clone());
    let Some(key) = key else {
        return Ok(());
    };
    let settings = window
        .state::<NotchState>()
        .settings
        .lock()
        .unwrap()
        .clone();
    let app = window.app_handle().clone();
    tauri::async_runtime::spawn_blocking(move || {
        if !dock::windows_for(&settings, &key).contains(&hwnd) {
            return Ok(());
        }
        if dismiss {
            unsafe { PostMessageW(Some(HWND(hwnd as *mut _)), WM_CLOSE, WPARAM(0), LPARAM(0)) }
                .map_err(|e| e.to_string())?;
            let mut state = SESSION.lock().unwrap();
            let Some(session) = state
                .as_mut()
                .filter(|session| session.view.request == request)
            else {
                return Ok(());
            };
            session.view.cards.retain(|card| card.hwnd != hwnd);
            if session.view.cards.is_empty() {
                drop(state);
                close(&app, Some(request));
            } else {
                let _ = window.emit_to("dockpreview", "dock-preview", &session.view);
            }
        } else {
            close(&app, Some(request));
            if !dock::focus_window(hwnd) {
                return Err("Windows no pudo enfocar la ventana.".into());
            }
        }
        Ok(())
    })
    .await
    .map_err(|e| e.to_string())?
}

pub(crate) fn watch(app: AppHandle) {
    std::thread::spawn(move || loop {
        std::thread::sleep(Duration::from_millis(35));
        let snapshot = SESSION
            .lock()
            .unwrap()
            .as_ref()
            .map(|s| (s.view.request, s.anchor, s.panel, s.opened));
        let Some((request, anchor, panel, opened)) = snapshot else {
            continue;
        };
        let mut point = POINT::default();
        unsafe {
            let _ = GetCursorPos(&mut point);
        }
        let bridge = RECT {
            left: anchor.left.min(panel.left),
            top: panel.bottom,
            right: anchor.right.max(panel.right),
            bottom: anchor.top,
        };
        if !dock::can_show()
            || dock::menu_open()
            || dock::capture_in_progress()
            || (opened.elapsed() > Duration::from_millis(120)
                && ![anchor, panel, bridge]
                    .into_iter()
                    .any(|rect| contains(rect, point)))
        {
            close(&app, Some(request));
        }
    });
}

fn capture(raw: isize) -> Option<String> {
    let hwnd = HWND(raw as *mut _);
    let mut rect = RECT::default();
    let mut placement = WINDOWPLACEMENT {
        length: std::mem::size_of::<WINDOWPLACEMENT>() as u32,
        ..Default::default()
    };
    let image = unsafe {
        if !IsWindow(Some(hwnd)).as_bool()
            || SendMessageTimeoutW(
                hwnd,
                WM_NULL,
                WPARAM(0),
                LPARAM(0),
                SMTO_ABORTIFHUNG,
                120,
                None,
            )
            .0 == 0
        {
            return None;
        }
        if IsIconic(hwnd).as_bool() {
            GetWindowPlacement(hwnd, &mut placement).ok()?;
            rect = placement.rcNormalPosition;
        } else {
            GetWindowRect(hwnd, &mut rect).ok()?;
        }
        let width = rect.right - rect.left;
        let height = rect.bottom - rect.top;
        if width <= 0 || height <= 0 || i64::from(width) * i64::from(height) > 32_000_000 {
            return None;
        }
        let info = BITMAPINFO {
            bmiHeader: BITMAPINFOHEADER {
                biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
                biWidth: width,
                biHeight: -height,
                biPlanes: 1,
                biBitCount: 32,
                biCompression: BI_RGB.0,
                ..Default::default()
            },
            ..Default::default()
        };
        let dc = CreateCompatibleDC(None);
        if dc.is_invalid() {
            return None;
        }
        let mut pixels = std::ptr::null_mut();
        let Ok(bitmap) = CreateDIBSection(Some(dc), &info, DIB_RGB_COLORS, &mut pixels, None, 0)
        else {
            let _ = DeleteDC(dc);
            return None;
        };
        let previous = SelectObject(dc, HGDIOBJ(bitmap.0));
        let result = if !previous.is_invalid()
            && !pixels.is_null()
            && PrintWindow(hwnd, dc, PRINT_WINDOW_FLAGS(PW_RENDERFULLCONTENT)).as_bool()
        {
            let data = std::slice::from_raw_parts(
                pixels.cast::<u8>(),
                width as usize * height as usize * 4,
            );
            let rgb = data
                .chunks_exact(4)
                .flat_map(|px| [px[2], px[1], px[0]])
                .collect::<Vec<_>>();
            if rgb.iter().all(|&value| value == 0) {
                None
            } else {
                image::RgbImage::from_raw(width as u32, height as u32, rgb)
            }
        } else {
            None
        };
        if !previous.is_invalid() {
            SelectObject(dc, previous);
        }
        let _ = DeleteObject(HGDIOBJ(bitmap.0));
        let _ = DeleteDC(dc);
        result?
    };
    let scaled = image::DynamicImage::ImageRgb8(image).thumbnail(320, 180);
    let mut png = Cursor::new(Vec::new());
    scaled.write_to(&mut png, image::ImageFormat::Png).ok()?;
    Some(format!(
        "data:image/png;base64,{}",
        STANDARD.encode(png.into_inner())
    ))
}
