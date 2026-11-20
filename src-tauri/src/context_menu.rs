//! Glass menu view shared with Bluetooth/Wi-Fi. Shell objects stay on the UI STA until selection.
use crate::{dock, jump_lists, notch::NotchState};
use serde::{Deserialize, Serialize};
use std::{
    cell::RefCell,
    collections::HashSet,
    os::windows::process::CommandExt,
    process::Command,
    sync::{
        atomic::{AtomicU64, Ordering},
        Mutex, OnceLock,
    },
};
use tauri::{
    window::Color, AppHandle, Emitter, Manager, PhysicalPosition, PhysicalSize, WebviewWindow,
    WebviewWindowBuilder,
};
use windows::{
    core::{w, IUnknown, Interface, HSTRING, PCSTR},
    Win32::{
        Foundation::{HWND, LPARAM, POINT, RECT, WPARAM},
        Graphics::Gdi::{GetMonitorInfoW, MonitorFromPoint, MONITORINFO, MONITOR_DEFAULTTONEAREST},
        UI::{
            Shell::{
                BHID_SFUIObject, IContextMenu, IShellItem, ShellExecuteW, CMF_DEFAULTONLY,
                CMINVOKECOMMANDINFO,
            },
            WindowsAndMessaging::*,
        },
    },
};

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct MenuEntry {
    id: usize,
    label: String,
    enabled: bool,
    separator: bool,
    children: Vec<MenuEntry>,
}

struct Menu {
    entries: RefCell<Vec<MenuEntry>>,
}
impl Menu {
    fn new() -> Result<Self, String> {
        Ok(Self {
            entries: RefCell::new(Vec::new()),
        })
    }
    fn row(&self, id: usize, text: &str, enabled: bool) -> Result<(), String> {
        self.entries.borrow_mut().push(MenuEntry {
            id,
            label: text.into(),
            enabled,
            separator: false,
            children: Vec::new(),
        });
        Ok(())
    }
    fn separator(&self) -> Result<(), String> {
        self.entries.borrow_mut().push(MenuEntry {
            id: 0,
            label: String::new(),
            enabled: false,
            separator: true,
            children: Vec::new(),
        });
        Ok(())
    }
    fn submenu(&self, name: &str, child: Menu) -> Result<(), String> {
        self.entries.borrow_mut().push(MenuEntry {
            id: 0,
            label: name.into(),
            enabled: true,
            separator: false,
            children: child.entries.into_inner(),
        });
        Ok(())
    }
}

// Only used to obtain a Shell item's default verb. This menu is never displayed.
struct ShellMenu(HMENU);
impl ShellMenu {
    fn new() -> Result<Self, String> {
        unsafe { CreatePopupMenu().map(Self).map_err(|e| e.to_string()) }
    }
}
impl Drop for ShellMenu {
    fn drop(&mut self) {
        unsafe {
            let _ = DestroyMenu(self.0);
        }
    }
}

fn shell_open(target: &str, args: &str) -> Result<(), String> {
    let result = unsafe {
        ShellExecuteW(
            None,
            w!("open"),
            &HSTRING::from(target),
            &HSTRING::from(args),
            None,
            SW_SHOWNORMAL,
        )
    };
    (result.0 as isize > 32)
        .then_some(())
        .ok_or_else(|| format!("Windows no pudo abrir {target} ({}).", result.0 as isize))
}

unsafe fn invoke_jump(object: &IUnknown, owner: HWND) -> Result<(), String> {
    let context = object
        .cast::<IContextMenu>()
        .or_else(|_| {
            object
                .cast::<IShellItem>()?
                .BindToHandler::<_, IContextMenu>(None, &BHID_SFUIObject)
        })
        .map_err(|e| e.to_string())?;
    let menu = ShellMenu::new()?;
    context
        .QueryContextMenu(menu.0, 0, 1, 0x7fff, CMF_DEFAULTONLY)
        .ok()
        .map_err(|e| e.to_string())?;
    let id = GetMenuDefaultItem(menu.0, 0, GET_MENU_DEFAULT_ITEM_FLAGS(0));
    if id == u32::MAX || id < 1 {
        return Err("El destino no tiene una acción de apertura.".into());
    }
    let info = CMINVOKECOMMANDINFO {
        cbSize: std::mem::size_of::<CMINVOKECOMMANDINFO>() as u32,
        hwnd: owner,
        lpVerb: PCSTR((id - 1) as usize as *const u8),
        nShow: SW_SHOWNORMAL.0,
        ..Default::default()
    };
    context.InvokeCommand(&info).map_err(|e| e.to_string())
}

fn kill_windows(handles: &[isize]) -> Result<(), String> {
    // Resolve each HWND again at click time. Never taskkill by image name: that could kill unrelated
    // apps hosted by ApplicationFrameHost. For UWP use the visible app child rather than its host.
    let mut pids = HashSet::new();
    for handle in handles {
        let hwnd = HWND(*handle as *mut _);
        let mut pid = 0;
        unsafe {
            GetWindowThreadProcessId(hwnd, Some(&mut pid));
        }
        if crate::icons::process_image_path(pid)
            .unwrap_or_default()
            .to_lowercase()
            .ends_with("applicationframehost.exe")
        {
            unsafe extern "system" fn child(hwnd: HWND, data: LPARAM) -> windows::core::BOOL {
                let pids = &mut *(data.0 as *mut HashSet<u32>);
                let mut pid = 0;
                GetWindowThreadProcessId(hwnd, Some(&mut pid));
                if let Some(path) = crate::icons::process_image_path(pid) {
                    if !path.to_lowercase().ends_with("applicationframehost.exe") {
                        pids.insert(pid);
                    }
                }
                true.into()
            }
            unsafe {
                let _ = EnumChildWindows(
                    Some(hwnd),
                    Some(child),
                    LPARAM(&mut pids as *mut _ as isize),
                );
            }
        } else if pid != 0 {
            pids.insert(pid);
        }
    }
    for pid in pids {
        if pid == std::process::id() {
            continue;
        }
        let status = Command::new("taskkill.exe")
            .args(["/F", "/PID", &pid.to_string()])
            .creation_flags(0x08000000)
            .status()
            .map_err(|e| e.to_string())?;
        if !status.success() {
            return Err(format!("No se pudo finalizar el proceso {pid}."));
        }
    }
    Ok(())
}

enum Action {
    Jump(IUnknown),
    Command(Control),
}
impl From<Control> for Action {
    fn from(action: Control) -> Self {
        Self::Command(action)
    }
}
enum Control {
    Launch(dock::DockItem),
    Pin(dock::DockItem),
    Unpin(String),
    Kill(String),
    Close(String, isize),
    Open(String, String),
    Desktop,
    Settings(String),
    Focus(isize),
    Launchpad,
    TrashOpen,
    TrashEmpty,
}

fn add(
    menu: &Menu,
    actions: &mut Vec<Action>,
    label: &str,
    action: impl Into<Action>,
    enabled: bool,
) -> Result<(), String> {
    actions.push(action.into());
    menu.row(actions.len(), label, enabled)
}

fn application_menu(app: &AppHandle, key: &str) -> Result<(Menu, Vec<Action>), String> {
    if key == "apps" || key == "trash" {
        let menu = Menu::new()?;
        let mut actions = Vec::new();
        if key == "apps" {
            add(
                &menu,
                &mut actions,
                "Abrir Launchpad",
                Control::Launchpad,
                true,
            )?;
        } else {
            add(
                &menu,
                &mut actions,
                "Abrir papelera",
                Control::TrashOpen,
                true,
            )?;
            add(
                &menu,
                &mut actions,
                "Vaciar papelera",
                Control::TrashEmpty,
                true,
            )?;
        }
        return Ok((menu, actions));
    }
    let settings = app.state::<NotchState>().settings.lock().unwrap().clone();
    let item = dock::menu_item(&settings, key).ok_or("La aplicación ya no está en el Dock.")?;
    if item.kind == "folder" {
        let menu = Menu::new()?;
        let mut actions = Vec::new();
        add(
            &menu,
            &mut actions,
            "Abrir carpeta",
            Control::Open(item.path, String::new()),
            true,
        )?;
        return Ok((menu, actions));
    }
    let handles = dock::windows_for(&settings, key);
    let menu = Menu::new()?;
    let mut actions = Vec::new();
    unsafe {
        let id = jump_lists::app_id(&item.aumid, &item.path, &handles);
        for group in jump_lists::read(&id) {
            // Categories are submenus so long real lists remain accessible on any screen size.
            let child = Menu::new()?;
            for jump in group.items {
                add(
                    &child,
                    &mut actions,
                    &jump.name,
                    Action::Jump(jump.object),
                    true,
                )?;
            }
            menu.submenu(&group.name, child)?;
        }
    }
    if !actions.is_empty() {
        menu.separator()?;
    }
    add(
        &menu,
        &mut actions,
        "Abrir nueva instancia",
        Control::Launch(item.clone()),
        true,
    )?;
    if item.kind == "pinned" {
        add(
            &menu,
            &mut actions,
            "Desanclar del dock",
            Control::Unpin(key.into()),
            true,
        )?;
    } else {
        add(
            &menu,
            &mut actions,
            "Anclar al dock",
            Control::Pin(item),
            true,
        )?;
    }
    add(
        &menu,
        &mut actions,
        "Finalizar tarea",
        Control::Kill(key.into()),
        !handles.is_empty(),
    )?;
    let foreground = unsafe { GetForegroundWindow().0 as isize };
    let target = handles
        .iter()
        .copied()
        .find(|h| *h == foreground)
        .or_else(|| handles.first().copied())
        .unwrap_or(0);
    add(
        &menu,
        &mut actions,
        "Cerrar ventana",
        Control::Close(key.into(), target),
        !handles.is_empty(),
    )?;
    Ok((menu, actions))
}

fn background_menu(_app: &AppHandle, section: &str) -> Result<(Menu, Vec<Action>), String> {
    let menu = Menu::new()?;
    let mut actions = Vec::new();
    let terminal = Menu::new()?;
    for (label, exe, args) in [
        ("PowerShell", "powershell.exe", ""),
        ("CMD", "cmd.exe", ""),
        ("WSL MADRE", "wsl.exe", "-d MADRE"),
    ] {
        add(
            &terminal,
            &mut actions,
            label,
            Control::Open(exe.into(), args.into()),
            true,
        )?;
    }
    menu.submenu("Terminal", terminal)?;
    let go = Menu::new()?;
    for (label, target) in [
        ("Escritorio", "shell:Desktop"),
        ("Descargas", "shell:Downloads"),
        ("Documentos", "shell:Personal"),
        ("Explorador", "explorer.exe"),
    ] {
        add(
            &go,
            &mut actions,
            label,
            Control::Open(target.into(), String::new()),
            true,
        )?;
    }
    menu.submenu("Ir a", go)?;
    let apps = Menu::new()?;
    add(
        &apps,
        &mut actions,
        "Todas las aplicaciones",
        Control::Launchpad,
        true,
    )?;
    apps.separator()?;
    for (hwnd, title) in dock::open_window_entries() {
        add(&apps, &mut actions, &title, Control::Focus(hwnd), true)?;
    }
    menu.submenu("Apps / Ventanas abiertas", apps)?;
    menu.separator()?;
    add(
        &menu,
        &mut actions,
        "Mostrar escritorio",
        Control::Desktop,
        true,
    )?;
    add(
        &menu,
        &mut actions,
        "Configuración de Windows",
        Control::Open("ms-settings:".into(), String::new()),
        true,
    )?;
    add(
        &menu,
        &mut actions,
        "Administrador de tareas",
        Control::Open("taskmgr.exe".into(), String::new()),
        true,
    )?;
    menu.separator()?;
    add(
        &menu,
        &mut actions,
        if section == "dock" {
            "Ajustes del Dock"
        } else {
            "Ajustes de la Barra"
        },
        Control::Settings(section.into()),
        true,
    )?;
    Ok((menu, actions))
}

async fn execute(app: AppHandle, owner: isize, action: Control) -> Result<(), String> {
    match action {
        Control::Open(path, args) => {
            tauri::async_runtime::spawn_blocking(move || shell_open(&path, &args))
                .await
                .map_err(|e| e.to_string())?
        }
        Control::Launch(item) => {
            let animate = crate::dock_animation::enabled();
            if animate {
                let _ = app.emit_to("dock", "dock-app-launching", (&item.key, item.window_count));
            }
            let result = dock::dock_launch(item.path, item.args, item.aumid).await;
            if animate && result.is_err() {
                let _ = app.emit_to("dock", "dock-app-launch-failed", &item.key);
            }
            result
        }
        Control::Pin(item) => {
            dock::dock_pin(app, item.path, item.name, item.args, item.aumid).await
        }
        Control::Unpin(key) => dock::dock_unpin(app, key).await,
        Control::Close(key, target) => {
            let settings = app.state::<NotchState>().settings.lock().unwrap().clone();
            if dock::windows_for(&settings, &key).contains(&target) {
                unsafe {
                    PostMessageW(Some(HWND(target as *mut _)), WM_CLOSE, WPARAM(0), LPARAM(0))
                        .map_err(|e| e.to_string())?;
                }
            }
            Ok(())
        }
        Control::Kill(key) => {
            let settings = app.state::<NotchState>().settings.lock().unwrap().clone();
            tauri::async_runtime::spawn_blocking(move || {
                kill_windows(&dock::windows_for(&settings, &key))
            })
            .await
            .map_err(|e| e.to_string())?
        }
        Control::Focus(raw) => {
            unsafe {
                let hwnd = HWND(raw as *mut _);
                if IsWindow(Some(hwnd)).as_bool() {
                    let _ = ShowWindowAsync(hwnd, SW_RESTORE);
                    let _ = SetForegroundWindow(hwnd);
                }
            }
            Ok(())
        }
        Control::Desktop => {
            // Drop menu focus before sending Win+D.
            unsafe {
                let _ = PostMessageW(Some(HWND(owner as *mut _)), WM_NULL, WPARAM(0), LPARAM(0));
            }
            crate::topbar::send_keys("desktop".into())
        }
        Control::Settings(section) => crate::show_settings(&app, Some(section)),
        Control::Launchpad => crate::launchpad::toggle_launchpad(app).await,
        Control::TrashOpen => dock::trash_open().await,
        Control::TrashEmpty => dock::trash_empty().await,
    }
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MenuPayload {
    token: u64,
    entries: Vec<MenuEntry>,
    x: f64,
    y: f64,
    above: bool,
}
struct Session {
    token: u64,
    actions: Vec<Action>,
    owner: isize,
}
thread_local! { static SESSION: RefCell<Option<Session>> = const { RefCell::new(None) }; }
static PAYLOAD: OnceLock<Mutex<Option<MenuPayload>>> = OnceLock::new();
static REGIONS: OnceLock<Mutex<Vec<RECT>>> = OnceLock::new();
static GENERATION: AtomicU64 = AtomicU64::new(1);
fn payload() -> &'static Mutex<Option<MenuPayload>> {
    PAYLOAD.get_or_init(|| Mutex::new(None))
}

pub(crate) fn contains_point(point: POINT) -> bool {
    REGIONS
        .get_or_init(|| Mutex::new(Vec::new()))
        .lock()
        .unwrap()
        .iter()
        .any(|r| point.x >= r.left && point.x < r.right && point.y >= r.top && point.y < r.bottom)
}

/// Called by the common popup manager too; COM references must be dropped on their owning STA.
pub(crate) fn clear_session(app: &AppHandle) {
    let token = payload().lock().unwrap().take().map(|p| p.token);
    REGIONS
        .get_or_init(|| Mutex::new(Vec::new()))
        .lock()
        .unwrap()
        .clear();
    dock::set_menu_open(false);
    let _ = app.emit("context-menu-visibility", false);
    let _ = app.run_on_main_thread(move || {
        if let Some(token) = token {
            release_session(token);
        }
    });
}

fn release_session(token: u64) {
    SESSION.with(|slot| {
        let matches = slot.borrow().as_ref().is_some_and(|s| s.token == token);
        if matches {
            slot.borrow_mut().take();
        }
    });
}

fn ensure_window(app: &AppHandle) -> Result<WebviewWindow, String> {
    if let Some(window) = app.get_webview_window("contextmenu") {
        return Ok(window);
    }
    let window = WebviewWindowBuilder::new(
        app,
        "contextmenu",
        crate::window_utils::app_url("contextmenu"),
    )
    .title("")
    .inner_size(900.0, 600.0)
    .decorations(false)
    .transparent(true)
    .background_color(Color(0, 0, 0, 0))
    .always_on_top(true)
    .resizable(false)
    .skip_taskbar(true)
    .shadow(false)
    .focused(false)
    .visible(false)
    .disable_drag_drop_handler()
    .build()
    .map_err(|e| e.to_string())?;
    crate::window_utils::mark_rounded(&window);
    let handle = app.clone();
    crate::popups::register_window(app, &window, "contextmenu", move || clear_session(&handle));
    Ok(window)
}

/// The entire menu canvas, including every submenu, ends before the Dock's top edge.
fn menu_area(work: RECT, source: RECT, above: bool, gap: i32) -> RECT {
    RECT {
        left: work.left,
        top: work.top,
        right: work.right,
        bottom: if above {
            work.bottom.min(source.top - gap).max(work.top + 1)
        } else {
            work.bottom
        },
    }
}

async fn popup(source: WebviewWindow, key: Option<String>, section: String) -> Result<(), String> {
    let app = source.app_handle().clone();
    let menu_window = ensure_window(&app)?;
    let handle = app.clone();
    source
        .clone()
        .run_on_main_thread(move || {
            let result = (|| -> Result<(), String> {
                crate::popups::close(&app, "contextmenu");
                let owner = source.hwnd().map_err(|e| e.to_string())?;
                let hwnd = menu_window.hwnd().map_err(|e| e.to_string())?;
                let mut cursor = POINT::default();
                let mut source_rect = RECT::default();
                unsafe {
                    GetCursorPos(&mut cursor).map_err(|e| e.to_string())?;
                    GetWindowRect(owner, &mut source_rect).map_err(|e| e.to_string())?;
                }
                let monitor = unsafe { MonitorFromPoint(cursor, MONITOR_DEFAULTTONEAREST) };
                let mut info = MONITORINFO {
                    cbSize: std::mem::size_of::<MONITORINFO>() as u32,
                    ..Default::default()
                };
                if !unsafe { GetMonitorInfoW(monitor, &mut info).as_bool() } {
                    return Err("No se pudo consultar el monitor del menú.".into());
                }
                let above = source.label() == "dock";
                let scale = source.scale_factor().unwrap_or(1.0);
                let area = menu_area(
                    info.rcWork,
                    source_rect,
                    above,
                    (8.0 * scale).round() as i32,
                );
                let (menu, actions) = if let Some(key) = key {
                    application_menu(&app, &key)?
                } else {
                    background_menu(&app, &section)?
                };
                let token = GENERATION.fetch_add(1, Ordering::SeqCst);
                let data = MenuPayload {
                    token,
                    entries: menu.entries.into_inner(),
                    x: ((cursor.x - area.left) as f64 / scale).max(0.0),
                    y: if above {
                        (area.bottom - area.top) as f64 / scale
                    } else {
                        ((source_rect.bottom - area.top) as f64 / scale).max(0.0) + 8.0
                    },
                    above,
                };
                let _ = menu_window.hide();
                menu_window
                    .set_position(PhysicalPosition::new(area.left, area.top))
                    .map_err(|e| e.to_string())?;
                menu_window
                    .set_size(PhysicalSize::new(
                        (area.right - area.left).max(1) as u32,
                        (area.bottom - area.top).max(1) as u32,
                    ))
                    .map_err(|e| e.to_string())?;
                unsafe {
                    SetWindowPos(
                        hwnd,
                        Some(HWND_TOPMOST),
                        area.left,
                        area.top,
                        area.right - area.left,
                        area.bottom - area.top,
                        SWP_NOACTIVATE,
                    )
                    .map_err(|e| e.to_string())?;
                }
                SESSION.with(|slot| {
                    *slot.borrow_mut() = Some(Session {
                        token,
                        actions,
                        owner: owner.0 as isize,
                    })
                });
                *payload().lock().unwrap() = Some(data.clone());
                REGIONS
                    .get_or_init(|| Mutex::new(Vec::new()))
                    .lock()
                    .unwrap()
                    .clear();
                crate::popups::open(&app, "contextmenu")?;
                dock::set_menu_open(true);
                let _ = app.emit("context-menu-visibility", true);
                menu_window
                    .emit("context-menu-open", &data)
                    .map_err(|e| e.to_string())?;
                Ok(())
            })();
            if let Err(error) = result {
                clear_session(&handle);
                report_error(&handle, &error);
            }
        })
        .map_err(|e| e.to_string())
}

#[derive(Deserialize)]
pub struct MenuRegion {
    x: f64,
    y: f64,
    w: f64,
    h: f64,
}

#[tauri::command]
pub fn context_menu_state(window: WebviewWindow) -> Result<Option<MenuPayload>, String> {
    if window.label() != "contextmenu" {
        return Err("Ventana incorrecta.".into());
    }
    Ok(payload().lock().unwrap().clone())
}

#[tauri::command]
pub fn context_menu_present(
    window: WebviewWindow,
    token: u64,
    regions: Vec<MenuRegion>,
) -> Result<(), String> {
    if window.label() != "contextmenu"
        || payload()
            .lock()
            .unwrap()
            .as_ref()
            .is_none_or(|p| p.token != token)
    {
        return Ok(());
    }
    let scale = window.scale_factor().map_err(|e| e.to_string())?;
    let pos = window.outer_position().map_err(|e| e.to_string())?;
    let rects = regions
        .into_iter()
        .filter(|r| [r.x, r.y, r.w, r.h].iter().all(|v| v.is_finite()) && r.w > 0.0 && r.h > 0.0)
        .map(|r| RECT {
            left: pos.x + (r.x * scale).round() as i32,
            top: pos.y + (r.y * scale).round() as i32,
            right: pos.x + ((r.x + r.w) * scale).round() as i32,
            bottom: pos.y + ((r.y + r.h) * scale).round() as i32,
        })
        .collect::<Vec<_>>();
    if rects.is_empty() {
        return Ok(());
    }
    *REGIONS
        .get_or_init(|| Mutex::new(Vec::new()))
        .lock()
        .unwrap() = rects;
    let ui = window.clone();
    window
        .run_on_main_thread(move || {
            if payload()
                .lock()
                .unwrap()
                .as_ref()
                .is_none_or(|p| p.token != token)
            {
                return;
            }
            if !ui.is_visible().unwrap_or(false) {
                crate::popups::present(&ui, "contextmenu");
                let _ = ui.emit("context-menu-focus", token);
            }
            if let Ok(hwnd) = ui.hwnd() {
                unsafe {
                    let _ = SetWindowPos(
                        hwnd,
                        Some(HWND_TOPMOST),
                        0,
                        0,
                        0,
                        0,
                        SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE,
                    );
                }
            }
        })
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn context_menu_close(app: AppHandle, token: u64) {
    let current = payload()
        .lock()
        .unwrap()
        .as_ref()
        .is_some_and(|p| p.token == token);
    if current {
        crate::popups::close(&app, "contextmenu");
    }
}

#[tauri::command]
pub fn context_menu_execute(window: WebviewWindow, token: u64, id: usize) -> Result<(), String> {
    if window.label() != "contextmenu" {
        return Err("Ventana incorrecta.".into());
    }
    let app = window.app_handle().clone();
    window
        .run_on_main_thread(move || {
            let selected = SESSION.with(|slot| {
                let mut slot = slot.borrow_mut();
                if slot
                    .as_ref()
                    .is_none_or(|s| s.token != token || id == 0 || id > s.actions.len())
                {
                    return None;
                }
                let mut session = slot.take().unwrap();
                Some((session.actions.remove(id - 1), session.owner))
            });
            let Some((action, owner)) = selected else {
                return;
            };
            crate::popups::close(&app, "contextmenu");
            if let Action::Jump(object) = action {
                if let Err(error) = unsafe { invoke_jump(&object, HWND(owner as *mut _)) } {
                    report_error(&app, &error);
                }
            } else if let Action::Command(action) = action {
                tauri::async_runtime::spawn(async move {
                    if let Err(error) = execute(app.clone(), owner, action).await {
                        report_error(&app, &error);
                    }
                });
            }
        })
        .map_err(|e| e.to_string())
}

fn report_error(app: &AppHandle, error: &str) {
    eprintln!("[context-menu] {error}");
    let _ = app.emit("context-menu-error", error);
    let message = HSTRING::from(error);
    let _ = app.run_on_main_thread(move || unsafe {
        MessageBoxW(None, &message, w!("Noki"), MB_OK | MB_ICONERROR);
    });
}

#[tauri::command]
pub async fn dock_context_menu(window: WebviewWindow, key: String) -> Result<(), String> {
    popup(window, Some(key), "dock".into()).await
}

#[tauri::command]
pub async fn desktop_context_menu(window: WebviewWindow, section: String) -> Result<(), String> {
    if section != "dock" && section != "topbar" {
        return Err("Sección de ajustes desconocida.".into());
    }
    popup(window, None, section).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use windows::Win32::System::Com::{CoInitializeEx, CoUninitialize, COINIT_APARTMENTTHREADED};
    #[test]
    fn real_jump_tasks_have_native_default_commands() {
        std::thread::spawn(|| unsafe {
            CoInitializeEx(None, COINIT_APARTMENTTHREADED).ok().unwrap();
            {
                for app in ["Chrome", "MSEdge"] {
                    for group in jump_lists::read(app) {
                        for item in group.items {
                            let context = item
                                .object
                                .cast::<IContextMenu>()
                                .or_else(|_| {
                                    item.object
                                        .cast::<IShellItem>()?
                                        .BindToHandler::<_, IContextMenu>(None, &BHID_SFUIObject)
                                })
                                .unwrap_or_else(|e| panic!("{} / {}: {e}", group.name, item.name));
                            let menu = ShellMenu::new().unwrap();
                            context
                                .QueryContextMenu(menu.0, 0, 1, 0x7fff, CMF_DEFAULTONLY)
                                .ok()
                                .unwrap();
                            assert_ne!(
                                GetMenuDefaultItem(menu.0, 0, GET_MENU_DEFAULT_ITEM_FLAGS(0)),
                                u32::MAX,
                                "{}",
                                item.name
                            );
                        }
                    }
                }
            }
            CoUninitialize();
        })
        .join()
        .unwrap();
    }
}

#[cfg(test)]
mod placement_tests {
    use super::*;
    #[test]
    fn dock_canvas_never_extends_behind_dock_even_on_negative_monitors() {
        for (left, top, scale) in [(0, 0, 1.0), (-1920, 0, 1.25), (1920, -1080, 1.5)] {
            let work = RECT {
                left,
                top: top + 38,
                right: left + 1920,
                bottom: top + 1080,
            };
            let dock = RECT {
                left: left + 500,
                top: top + 990,
                right: left + 1400,
                bottom: top + 1080,
            };
            let area = menu_area(work, dock, true, (8.0_f64 * scale).round() as i32);
            assert!(area.bottom < dock.top);
            assert!(area.top >= work.top && area.left == work.left && area.right == work.right);
        }
    }
    #[test]
    fn queued_close_does_not_release_a_newer_shell_session() {
        SESSION.with(|slot| {
            *slot.borrow_mut() = Some(Session {
                token: 2,
                actions: Vec::new(),
                owner: 0,
            })
        });
        release_session(1);
        SESSION.with(|slot| assert_eq!(slot.borrow().as_ref().unwrap().token, 2));
        release_session(2);
        SESSION.with(|slot| assert!(slot.borrow().is_none()));
    }
    #[test]
    fn menu_tree_keeps_ids_and_disabled_items() {
        let root = Menu::new().unwrap();
        let child = Menu::new().unwrap();
        child.row(7, "PowerShell", true).unwrap();
        root.submenu("Terminal", child).unwrap();
        root.separator().unwrap();
        root.row(8, "Cerrar ventana", false).unwrap();
        let rows = root.entries.into_inner();
        assert_eq!(rows[0].children[0].id, 7);
        assert!(rows[1].separator);
        assert!(!rows[2].enabled);
    }
}
