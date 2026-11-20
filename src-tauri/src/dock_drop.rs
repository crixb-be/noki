//! Recibe archivos de OLE y sigue el cursor durante la espera sobre un icono del Dock.
use crate::{dock, notch::NotchState, settings::Settings};
use std::{
    path::Path,
    sync::Mutex,
    time::{Duration, Instant},
};
use tauri::{AppHandle, DragDropEvent, Emitter, Manager, WebviewWindow, WindowEvent};
use windows::Win32::{
    Foundation::{HWND, POINT, RECT},
    UI::{
        Input::KeyboardAndMouse::{GetAsyncKeyState, VK_LBUTTON},
        WindowsAndMessaging::{GetWindowRect, IsWindowVisible},
    },
};

#[derive(serde::Deserialize)]
pub struct IconBounds {
    key: String,
    left: i32,
    top: i32,
    right: i32,
    bottom: i32,
    icon: Option<[i32; 4]>,
}

static ICONS: Mutex<Vec<IconBounds>> = Mutex::new(Vec::new());

pub(crate) fn animation_anchor(app: &AppHandle, key: &str) -> Option<RECT> {
    let [left, top, right, bottom] = ICONS.lock().unwrap().iter().find(|icon| icon.key == key)?.icon?;
    let dock = app.get_webview_window("dock")?;
    let hwnd = dock::hwnd(&dock)?;
    let mut origin = POINT::default();
    if !unsafe { windows::Win32::Graphics::Gdi::ClientToScreen(hwnd, &mut origin).as_bool() } {
        return None;
    }
    Some(RECT { left: origin.x + left, top: origin.y + top, right: origin.x + right, bottom: origin.y + bottom })
}

#[tauri::command]
pub fn dock_drop_targets(window: WebviewWindow, icons: Vec<IconBounds>) {
    if window.label() == "dock" {
        *ICONS.lock().unwrap() = icons;
    }
}

fn target_at(window: HWND, at: POINT) -> Option<String> {
    if !dock::can_show() {
        return None;
    }
    let mut r = RECT::default();
    unsafe {
        if !IsWindowVisible(window).as_bool() || GetWindowRect(window, &mut r).is_err() {
            return None;
        }
    }
    let (x, y) = (at.x - r.left, at.y - r.top);
    ICONS
        .lock()
        .unwrap()
        .iter()
        .find(|icon| x >= icon.left && x < icon.right && y >= icon.top && y < icon.bottom)
        .map(|icon| icon.key.clone())
}

#[derive(Clone, Copy, PartialEq)]
pub(crate) enum Source {
    Ole,
    Tauri,
}

struct Drag {
    source: Source,
    key: Option<String>,
    since: Instant,
    activated: bool,
}

static DRAG: Mutex<Option<Drag>> = Mutex::new(None);

pub(crate) fn left_button_down() -> bool {
    unsafe { GetAsyncKeyState(VK_LBUTTON.0 as i32) < 0 }
}

pub(crate) fn enter(app: &AppHandle, window: HWND, at: POINT, source: Source) -> bool {
    let target = target_at(window, at);
    let mut drag = DRAG.lock().unwrap();
    // Nuestro destino OLE reemplaza al de WebView2 cuando ambos están disponibles.
    if matches!(source, Source::Tauri) && drag.as_ref().is_some_and(|d| d.source == Source::Ole) {
        return false;
    }
    if drag.as_ref().is_none_or(|state| state.source != source) {
        *drag = Some(Drag {
            source,
            key: None,
            since: Instant::now(),
            activated: false,
        });
    }
    drop(drag);
    update(app, window, at);
    target.is_some()
}

pub(crate) fn accepts(window: HWND, at: POINT) -> bool {
    target_at(window, at).is_some()
}

pub(crate) fn leave(app: &AppHandle, source: Source) -> bool {
    let mut drag = DRAG.lock().unwrap();
    if drag.as_ref().is_none_or(|d| d.source != source) {
        return false;
    }
    *drag = None;
    drop(drag);
    let _ = app.emit_to("dock", "dock-file-hover", Option::<String>::None);
    true
}

fn update(app: &AppHandle, window: HWND, at: POINT) {
    if DRAG.lock().unwrap().is_none() {
        return;
    }
    let key = target_at(window, at);
    let mut drag = DRAG.lock().unwrap();
    let Some(state) = drag.as_mut() else {
        return;
    };
    if state.key != key {
        state.key = key.clone();
        state.since = Instant::now();
        state.activated = false;
        let _ = app.emit_to("dock", "dock-file-hover", &key);
    }
    let Some(key) = key else {
        return;
    };
    if state.activated
        || key == "trash"
        || !left_button_down()
        || state.since.elapsed() < Duration::from_millis(450)
    {
        return;
    }
    state.activated = true;
    drop(drag);
    let app = app.clone();
    tauri::async_runtime::spawn_blocking(move || {
        if !left_button_down()
            || !DRAG
                .lock()
                .unwrap()
                .as_ref()
                .is_some_and(|drag| drag.key.as_ref() == Some(&key) && drag.activated)
        {
            return;
        }
        let settings = app.state::<NotchState>().settings.lock().unwrap().clone();
        if let Some(window) = dock::windows_for(&settings, &key).first() {
            dock::focus_window(*window);
        } else if let Some(path) =
            folder_path(&settings, &key).filter(|path| Path::new(path).is_dir())
        {
            let _ =
                dock::dock_launch_blocking("explorer.exe".into(), quoted_path(path), String::new());
        }
    });
}

pub(crate) fn poll(app: &AppHandle, window: HWND, at: POINT) {
    update(app, window, at);
}

pub(crate) fn drop_files(
    app: &AppHandle,
    window: HWND,
    at: POINT,
    paths: Vec<String>,
    source: Source,
) -> bool {
    if !leave(app, source) {
        return false;
    }
    let Some(key) = target_at(window, at) else {
        return false;
    };
    if paths.is_empty() {
        return false;
    }
    let app = app.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let settings = app.state::<NotchState>().settings.lock().unwrap().clone();
        let result = perform_drop(&settings, &key, paths);
        let _ = app.emit_to("dock", "dock-file-result", result.err());
    });
    true
}

fn folder_path<'a>(settings: &'a Settings, key: &str) -> Option<&'a str> {
    let path = key.strip_prefix("folder:")?;
    settings
        .dock
        .folders
        .iter()
        .find(|folder| folder.path == path)
        .map(|folder| folder.path.as_str())
}

fn quoted_path(path: &str) -> String {
    let trailing = path.len() - path.trim_end_matches('\\').len();
    format!("\"{path}{}\"", "\\".repeat(trailing))
}

fn perform_drop(settings: &Settings, key: &str, paths: Vec<String>) -> Result<(), String> {
    if paths
        .iter()
        .any(|p| p.contains(['\0', '"']) || !Path::new(p).is_absolute() || !Path::new(p).exists())
    {
        return Err("Uno de los archivos arrastrados ya no está disponible.".into());
    }
    if key == "trash" && settings.dock.show_trash {
        return shell_transfer(&paths, None);
    }
    if let Some(folder) = folder_path(settings, key) {
        let destination = Path::new(folder)
            .canonicalize()
            .map_err(|e| e.to_string())?;
        for path in &paths {
            let origin = Path::new(path).canonicalize().map_err(|e| e.to_string())?;
            if destination.starts_with(origin) {
                return Err("No se puede mover una carpeta dentro de sí misma.".into());
            }
        }
        return shell_transfer(&paths, Some(folder));
    }
    let item = dock::drop_app(settings, key).ok_or("La aplicación ya no está disponible.")?;
    if item.aumid.contains('!')
        || item.path.to_ascii_lowercase().starts_with("shell:")
        || item.path.is_empty()
    {
        return activate_packaged(&item.aumid, &paths);
    }
    let args = paths
        .iter()
        .map(|p| quoted_path(p))
        .collect::<Vec<_>>()
        .join(" ");
    let args = [item.args.trim(), &args]
        .into_iter()
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join(" ");
    dock::dock_launch_blocking(item.path, args, String::new())
}

fn shell_transfer(paths: &[String], folder: Option<&str>) -> Result<(), String> {
    use windows::{
        core::{HSTRING, PCWSTR},
        Win32::{
            System::Com::{CoCreateInstance, CLSCTX_INPROC_SERVER},
            UI::Shell::{
                FileOperation, IFileOperation, IShellItem, SHCreateItemFromParsingName,
                FOFX_RECYCLEONDELETE, FOF_ALLOWUNDO, FOF_RENAMEONCOLLISION, FOF_WANTNUKEWARNING,
            },
        },
    };
    with_com(|| unsafe {
        let operation: IFileOperation =
            CoCreateInstance(&FileOperation, None, CLSCTX_INPROC_SERVER)?;
        let destination: Option<IShellItem> = folder
            .map(|path| SHCreateItemFromParsingName(&HSTRING::from(path), None))
            .transpose()?;
        operation.SetOperationFlags(if destination.is_some() {
            FOF_ALLOWUNDO | FOF_RENAMEONCOLLISION
        } else {
            FOF_ALLOWUNDO | FOFX_RECYCLEONDELETE | FOF_WANTNUKEWARNING
        })?;
        for path in paths {
            let item: IShellItem = SHCreateItemFromParsingName(&HSTRING::from(path), None)?;
            if let Some(destination) = destination.as_ref() {
                operation.MoveItem(&item, destination, PCWSTR::null(), None)?;
            } else {
                operation.DeleteItem(&item, None)?;
            }
        }
        operation.PerformOperations()?;
        operation.GetAnyOperationsAborted()
    })
    .and_then(|aborted| {
        if aborted.as_bool() {
            Err("Windows canceló parte de la operación.".into())
        } else {
            Ok(())
        }
    })
}

fn with_com<T>(run: impl FnOnce() -> windows::core::Result<T>) -> Result<T, String> {
    use windows::Win32::System::Com::{CoInitializeEx, CoUninitialize, COINIT_APARTMENTTHREADED};
    unsafe { CoInitializeEx(None, COINIT_APARTMENTTHREADED).ok() }.map_err(|e| e.to_string())?;
    let result = run();
    unsafe {
        CoUninitialize();
    }
    result.map_err(|e| e.to_string())
}

fn activate_packaged(aumid: &str, paths: &[String]) -> Result<(), String> {
    use windows::{
        core::{w, HSTRING},
        Win32::{
            System::Com::{CoCreateInstance, CLSCTX_LOCAL_SERVER},
            UI::Shell::{
                ApplicationActivationManager, IApplicationActivationManager, IShellItem,
                IShellItemArray, SHCreateItemFromParsingName, SHCreateShellItemArrayFromShellItem,
            },
        },
    };
    if aumid.is_empty() {
        return Err("Windows no pudo identificar la aplicación de destino.".into());
    }
    with_com(|| unsafe {
        let manager: IApplicationActivationManager =
            CoCreateInstance(&ApplicationActivationManager, None, CLSCTX_LOCAL_SERVER)?;
        let aumid = HSTRING::from(aumid);
        for path in paths {
            let item: IShellItem = SHCreateItemFromParsingName(&HSTRING::from(path), None)?;
            let array: IShellItemArray = SHCreateShellItemArrayFromShellItem(&item)?;
            manager.ActivateForFile(&aumid, &array, w!("open"))?;
        }
        Ok(())
    })
}

pub(crate) fn register(app: &AppHandle, window: &WebviewWindow) {
    let app = app.clone();
    let Ok(raw) = window.hwnd().map(|h| h.0 as isize) else {
        return;
    };
    window.on_window_event(move |event| {
        let WindowEvent::DragDrop(event) = event else {
            return;
        };
        handle_tauri_event(&app, raw, event);
    });
    let app = window.app_handle().clone();
    window.on_webview_event(move |event| {
        let tauri::WebviewEvent::DragDrop(event) = event else {
            return;
        };
        handle_tauri_event(&app, raw, event);
    });
}

fn handle_tauri_event(app: &AppHandle, raw: isize, event: &DragDropEvent) {
    let hwnd = HWND(raw as *mut _);
    let mut r = RECT::default();
    if unsafe { GetWindowRect(hwnd, &mut r) }.is_err() {
        return;
    }
    let screen = |x: f64, y: f64| POINT {
        x: r.left + x.round() as i32,
        y: r.top + y.round() as i32,
    };
    match event {
        DragDropEvent::Enter { paths, position } if !paths.is_empty() => {
            enter(app, hwnd, screen(position.x, position.y), Source::Tauri);
        }
        DragDropEvent::Over { position } => {
            let tauri_drag = DRAG
                .lock()
                .unwrap()
                .as_ref()
                .is_some_and(|drag| drag.source == Source::Tauri);
            if tauri_drag {
                poll(app, hwnd, screen(position.x, position.y));
            }
        }
        DragDropEvent::Drop { paths, position } => {
            drop_files(
                app,
                hwnd,
                screen(position.x, position.y),
                paths
                    .iter()
                    .map(|path| path.to_string_lossy().into_owned())
                    .collect(),
                Source::Tauri,
            );
        }
        DragDropEvent::Leave => {
            leave(app, Source::Tauri);
        }
        _ => {}
    }
}
