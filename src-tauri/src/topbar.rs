//! Barra superior y paneles rápidos de Windows.
//!
//! La barra vive en una ventana propia para que el acrílico cubra todo el monitor. El notch conserva su
//! ventana independiente y se convierte en ventana owned de `topbar` mientras la barra está activa.

use crate::{notch, popups, settings::Settings, statuswatch, system, window_utils};
use base64::Engine;
use image::ImageEncoder;
use serde::Serialize;
use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
    sync::{Mutex, OnceLock},
    time::Duration,
};
use tauri::{window::Color, AppHandle, Emitter, Manager, PhysicalPosition, PhysicalSize, WebviewWindow, WebviewWindowBuilder};
use windows::{
    core::{HSTRING, PCWSTR, PWSTR},
    Win32::{
    Foundation::{CloseHandle, HANDLE, HWND, LPARAM, POINT},
        Graphics::Imaging::{CLSID_WICImagingFactory, GUID_WICPixelFormat32bppPBGRA, WICConvertBitmapSource, WICDecodeMetadataCacheOnLoad, IWICImagingFactory},
        NetworkManagement::WiFi::{
            dot11_BSS_type_infrastructure, wlan_connection_mode_profile, WlanCloseHandle, WlanConnect, WlanEnumInterfaces,
            WlanFreeMemory, WlanGetAvailableNetworkList, WlanOpenHandle, WlanScan,
            WLAN_AVAILABLE_NETWORK_CONNECTED, WLAN_AVAILABLE_NETWORK_HAS_PROFILE, WLAN_AVAILABLE_NETWORK_LIST,
            WLAN_CONNECTION_PARAMETERS, WLAN_INTERFACE_INFO_LIST,
        },
        UI::{
            Input::KeyboardAndMouse::{
                SendInput, INPUT, INPUT_KEYBOARD, KEYBDINPUT, KEYEVENTF_EXTENDEDKEY, KEYEVENTF_KEYUP, VK_LWIN,
                VIRTUAL_KEY,
            },
            WindowsAndMessaging::{
                GetForegroundWindow, GetWindowLongPtrW, GetWindowThreadProcessId, SetWindowLongPtrW, SetWindowPos, WindowFromPoint,
                GWL_EXSTYLE, GWLP_HWNDPARENT, HWND_TOPMOST, SWP_ASYNCWINDOWPOS, SWP_NOACTIVATE, SWP_NOMOVE,
                SWP_NOSENDCHANGING, SWP_NOSIZE, WS_EX_TOPMOST, WS_EX_TRANSPARENT,
            },
            Shell::{IsUserAnAdmin, SHAppBarMessage, ShellExecuteW, APPBARDATA},
        },
    },
};
use window_vibrancy::{clear_acrylic, clear_blur};
use windows::Win32::System::Com::{CoCreateInstance, CLSCTX_INPROC_SERVER};
use windows::Win32::System::Threading::{OpenProcess, QueryFullProcessImageNameW, PROCESS_NAME_FORMAT, PROCESS_QUERY_LIMITED_INFORMATION};
use windows::Win32::System::WinRT::{RoInitialize, RO_INIT_MULTITHREADED};

const ABM_NEW: u32 = 0;
const ABM_REMOVE: u32 = 1;
const ABM_QUERYPOS: u32 = 2;
const ABM_SETPOS: u32 = 3;
const ABE_TOP: u32 = 1;
const PANEL_WIDTH: i32 = 330;
const TOPBAR_MARGIN: i32 = 8;
const PANEL_MARGIN: i32 = 0;

#[derive(Default)]
struct PanelState {
    kind: Option<String>,
    anchor_x: i32,
    height: i32,
}

static APPBAR: OnceLock<Mutex<Option<(isize, notch::Rect, i32)>>> = OnceLock::new();
static PANEL: OnceLock<Mutex<PanelState>> = OnceLock::new();
static GLASS: OnceLock<Mutex<Option<(String, String)>>> = OnceLock::new();
static AVATAR_CACHE: OnceLock<Mutex<Option<(String, Option<String>)>>> = OnceLock::new();

fn appbar() -> &'static Mutex<Option<(isize, notch::Rect, i32)>> {
    APPBAR.get_or_init(|| Mutex::new(None))
}

fn panel_state() -> &'static Mutex<PanelState> {
    PANEL.get_or_init(|| Mutex::new(PanelState { height: 320, ..PanelState::default() }))
}

fn glass_state() -> &'static Mutex<Option<(String, String)>> { GLASS.get_or_init(|| Mutex::new(None)) }
fn avatar_cache() -> &'static Mutex<Option<(String, Option<String>)>> { AVATAR_CACHE.get_or_init(|| Mutex::new(None)) }

fn hwnd(window: &WebviewWindow) -> Option<HWND> {
    window.hwnd().ok()
}

fn physical(logical: i32, scale: f64) -> i32 {
    (logical as f64 * scale).round() as i32
}

fn create_topbar(app: &AppHandle) -> Option<WebviewWindow> {
    if let Some(window) = app.get_webview_window("topbar") {
        return Some(window);
    }
    let window = WebviewWindowBuilder::new(app, "topbar", window_utils::app_url("topbar"))
        .title("")
        .inner_size(1200.0, 30.0)
        .decorations(false)
        .transparent(true)
        .background_color(Color(0, 0, 0, 0))
        .always_on_top(true)
        .resizable(false)
        .skip_taskbar(true)
        .shadow(false)
        .focused(false)
        .visible(false)
        .build()
        .ok()?;
    let _ = window.set_background_color(Some(Color(0, 0, 0, 0)));
    let _ = clear_acrylic(&window);
    let _ = clear_blur(&window);
    window_utils::mark_rounded(&window);
    window_utils::schedule_app_recovery(&window, "topbar");
    Some(window)
}

fn create_panel(app: &AppHandle) -> Option<WebviewWindow> {
    if let Some(window) = app.get_webview_window("barpanel") {
        return Some(window);
    }
    let window = WebviewWindowBuilder::new(app, "barpanel", window_utils::app_url("barpanel"))
        .title("")
        .inner_size((PANEL_WIDTH + PANEL_MARGIN * 2) as f64, 320.0 + (PANEL_MARGIN * 2) as f64)
        .decorations(false)
        .transparent(true)
        .background_color(Color(0, 0, 0, 0))
        .always_on_top(true)
        .resizable(false)
        .skip_taskbar(true)
        .shadow(false)
        .focused(false)
        .visible(false)
        .on_navigation(|url| window_utils::route_is_valid(url, "barpanel"))
        .on_page_load(|window, payload| window_utils::repair_page_load(window, payload.event(), payload.url(), "barpanel"))
        .build()
        .ok()?;
    let _ = window.set_background_color(Some(Color(0, 0, 0, 0)));
    let _ = clear_acrylic(&window);
    let _ = clear_blur(&window);
    popups::register_window(app, &window, "bar_panel", || {
        panel_state().lock().unwrap().kind = None;
    });
    window_utils::mark_rounded(&window);
    let style = app.state::<crate::notch::NotchState>().settings.try_lock().map(|s| s.appearance.panel_style.clone()).unwrap_or_else(|_| "light".into());
    window_utils::sync_panel_glass(app, &style);
    window_utils::schedule_app_recovery(&window, "barpanel");
    Some(window)
}

pub fn ensure_panel(app: &AppHandle) { let _ = create_panel(app); }

fn glass_changed(settings: &Settings) -> bool {
    let mode = settings.appearance.bar_mode.clone();
    let tint = settings.topbar.tint.clone();
    glass_state().lock().unwrap().as_ref() != Some(&(mode, tint))
}

/// Barra de vidrio: desenfoque real debajo (ventanas auxiliares de `backdrop`);
/// la ventana de la barra queda 100 % transparente y sin efectos de Windows.
fn apply_glass_mode(window: &WebviewWindow, mode: &str, _tint: &str) {
    // Liquid glass en todos los modos de la barra (también «auto»): siempre desenfoque debajo.
    let _ = mode;
    crate::backdrop::set_enabled(window.app_handle(), "topbar", true);
}

fn set_owner(app: &AppHandle, enabled: bool) {
    let Some(topbar) = app.get_webview_window("topbar") else { return };
    let Some(main) = app.get_webview_window("main") else { return };
    let Some(owner) = hwnd(&topbar) else { return };
    let Some(child) = hwnd(&main) else { return };
    let owner = if enabled { owner.0 as isize } else { 0 };
    let child = child.0 as isize;
    let _ = main.run_on_main_thread(move || unsafe {
        SetWindowLongPtrW(HWND(child as *mut _), GWLP_HWNDPARENT, owner);
    });
}

fn register_appbar(window: &WebviewWindow, monitor: notch::Rect, height: i32) {
    let Some(h) = hwnd(window) else { return };
    let key = (h.0 as isize, monitor, height);
    let mut current = appbar().lock().unwrap();
    if *current == Some(key) { return; }
    let mut data = APPBARDATA {
        cbSize: std::mem::size_of::<APPBARDATA>() as u32,
        hWnd: h,
        uCallbackMessage: 0,
        uEdge: ABE_TOP,
        rc: windows::Win32::Foundation::RECT { left: monitor.left, top: monitor.top, right: monitor.right, bottom: monitor.top + height },
        lParam: LPARAM(0),
    };
    unsafe {
        if !current.is_some_and(|(raw, _, _)| raw == key.0)
            && SHAppBarMessage(ABM_NEW, &mut data) == 0
        {
            return;
        }
        *current = Some(key);
        let _ = SHAppBarMessage(ABM_QUERYPOS, &mut data);
        data.rc.left = monitor.left;
        data.rc.right = monitor.right;
        data.rc.top = monitor.top;
        data.rc.bottom = monitor.top + height;
        let _ = SHAppBarMessage(ABM_SETPOS, &mut data);
    }
}

fn remove_appbar() {
    let mut current = appbar().lock().unwrap();
    let Some((raw, _, _)) = current.take() else { return };
    let mut data = APPBARDATA { cbSize: std::mem::size_of::<APPBARDATA>() as u32, hWnd: HWND(raw as *mut _), ..Default::default() };
    unsafe {
        let _ = SHAppBarMessage(ABM_REMOVE, &mut data);
    }
}

fn transient_foreground_process() -> bool {
    unsafe {
        let foreground = GetForegroundWindow();
        if foreground.0.is_null() { return false; }
        let mut pid = 0u32;
        GetWindowThreadProcessId(foreground, Some(&mut pid));
        let Ok(process) = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid) else { return false };
        let mut length = 32768u32;
        let mut path = vec![0u16; length as usize];
        let result = QueryFullProcessImageNameW(process, PROCESS_NAME_FORMAT(0), PWSTR(path.as_mut_ptr()), &mut length);
        let _ = CloseHandle(process);
        if result.is_err() { return false; }
        path.truncate(length as usize);
        let path = String::from_utf16_lossy(&path);
        matches!(path.rsplit(['\\', '/']).next().unwrap_or_default().to_ascii_lowercase().as_str(), "screenclippinghost.exe" | "snippingtool.exe" | "shellexperiencehost.exe" | "searchhost.exe")
    }
}

fn raise(hwnd: HWND, _force: bool) {
    if crate::dock::capture_overlay_active() || transient_foreground_process() { return; }
    unsafe {
        if GetWindowLongPtrW(hwnd, GWL_EXSTYLE) & WS_EX_TOPMOST.0 as isize != 0 {
            return;
        }
        let _ = SetWindowPos(
            hwnd,
            Some(HWND_TOPMOST),
            0,
            0,
            0,
            0,
            SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE | SWP_NOSENDCHANGING | SWP_ASYNCWINDOWPOS,
        );
    }
}

fn ensure_mouse_style(window: &WebviewWindow) {
    let w = window.clone();
    let _ = window.run_on_main_thread(move || {
        if let Some(h) = hwnd(&w) {
            unsafe {
                let ex = GetWindowLongPtrW(h, GWL_EXSTYLE);
                if ex & WS_EX_TRANSPARENT.0 as isize != 0 {
                    SetWindowLongPtrW(h, GWL_EXSTYLE, ex & !(WS_EX_TRANSPARENT.0 as isize));
                }
            }
        }
    });
}

fn foreign_window_at(point: POINT, own_pid: u32, topbar: HWND) -> bool {
    unsafe {
        let found = WindowFromPoint(point);
        if found.0.is_null() || found == topbar {
            return false;
        }
        let mut pid = 0u32;
        GetWindowThreadProcessId(found, Some(&mut pid as *mut u32));
        pid != 0 && pid != own_pid
    }
}

fn check_z_order(app: &AppHandle, monitor: notch::Rect) {
    if crate::dock::capture_overlay_active() || transient_foreground_process() { return; }
    let Some(topbar) = app.get_webview_window("topbar") else { return };
    let Some(top_hwnd) = hwnd(&topbar) else { return };
    let width = monitor.right - monitor.left;
    let own_pid = unsafe { windows::Win32::System::Threading::GetCurrentProcessId() };
    let points = [
        POINT { x: monitor.left + width / 10, y: monitor.top + 2 },
        POINT { x: monitor.left + width * 9 / 10, y: monitor.top + 2 },
    ];
    if points.iter().any(|p| foreign_window_at(*p, own_pid, top_hwnd)) {
        raise(top_hwnd, true);
        if let Some(main) = app.get_webview_window("main").and_then(|w| hwnd(&w)) {
            raise(main, true);
        }
    }
}

/// Sincroniza la barra con el camino lento del sondeo del notch (aprox. cada 2 s).
pub fn sync(app: &AppHandle, settings: &Settings, fullscreen: bool) {
    if !window_utils::frontend_ready("notch") { return; }
    let enabled = settings.topbar.enabled;
    let window = if enabled { create_topbar(app) } else { app.get_webview_window("topbar") };
    let Some(window) = window else { return };
    if !enabled {
        remove_appbar();
        set_owner(app, false);
        let ui_window = window.clone();
        let _ = window.run_on_main_thread(move || { let _ = ui_window.hide(); });
        if let Some(panel) = app.get_webview_window("barpanel") {
            let ui_panel = panel.clone();
            let _ = panel.run_on_main_thread(move || { let _ = ui_panel.hide(); });
        }
        panel_state().lock().unwrap().kind = None;
        return;
    }
    let _ = create_panel(app);
    let Some((_, monitor)) = notch::assignments(settings, &notch::monitors(app)).into_iter().find(|(l, _)| l == "main") else { return };
    let height = physical(settings.topbar.height, monitor.scale);
    let ui_window = window.clone();
    let _ = window.run_on_main_thread(move || {
        let _ = ui_window.set_position(PhysicalPosition::new(monitor.rect.left, monitor.rect.top));
        let _ = ui_window.set_size(PhysicalSize::new(monitor.rect.width() as u32, height as u32));
    });
    set_owner(app, true);
    register_appbar(&window, monitor.rect, height);
    if fullscreen {
        let ui_window = window.clone();
        let _ = window.run_on_main_thread(move || { let _ = ui_window.hide(); });
        return;
    }
    let ui_window = window.clone();
    let _ = window.run_on_main_thread(move || {
        let _ = ui_window.show();
        // Fuera de Alt+Tab aunque tao haya reescrito los estilos al mostrarla.
        if let Ok(hwnd) = ui_window.hwnd() { window_utils::enforce_chrome_free_hwnd(hwnd); }
    });
    ensure_mouse_style(&window);
    if let Some(h) = hwnd(&window) {
        raise(h, false);
        if glass_changed(settings) {
            let mode = settings.appearance.bar_mode.clone();
            let tint = settings.topbar.tint.clone();
            let ui_window = window.clone();
            let ui_mode = mode.clone();
            let ui_tint = tint.clone();
            let _ = window.run_on_main_thread(move || apply_glass_mode(&ui_window, &ui_mode, &ui_tint));
            *glass_state().lock().unwrap() = Some((mode, tint));
        }
    }
    check_z_order(app, monitor.rect);
}

pub fn remove() {
    remove_appbar();
}

/// Devuelve el PNG/JPG de la cuenta como data URL, usando el ajuste explícito o las imágenes de Windows.
pub fn user_avatar(settings: &Settings) -> Option<String> {
    let key = if settings.topbar.avatar_path.trim().is_empty() { "<windows>".to_string() } else { settings.topbar.avatar_path.clone() };
    if let Some((cached_key, value)) = avatar_cache().lock().unwrap().as_ref() {
        if cached_key == &key {
            return value.clone();
        }
    }
    let value = user_avatar_uncached(settings);
    *avatar_cache().lock().unwrap() = Some((key, value.clone()));
    value
}

fn user_avatar_uncached(settings: &Settings) -> Option<String> {
    let explicit = PathBuf::from(settings.topbar.avatar_path.trim());
    if !settings.topbar.avatar_path.trim().is_empty() {
        return image_data(&explicit);
    }
    let mut candidates = Vec::new();
    if let Some(appdata) = std::env::var_os("APPDATA") {
        let dir = PathBuf::from(appdata).join("Microsoft\\Windows\\AccountPictures");
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                if entry.path().extension().and_then(|e| e.to_str()).map(|e| e.eq_ignore_ascii_case("accountpicture-ms")).unwrap_or(false) {
                    candidates.extend(extract_embedded_images(&entry.path()));
                }
            }
        }
    }
    if let Ok(public) = std::env::var("PUBLIC") {
        let root = PathBuf::from(public).join("AccountPictures");
        if let Ok(dirs) = fs::read_dir(root) {
            for dir in dirs.flatten() {
                if let Ok(entries) = fs::read_dir(dir.path()) {
                    for entry in entries.flatten() {
                        if let Some(data) = read_image(&entry.path()) {
                            candidates.push(data);
                        }
                    }
                }
            }
        }
    }
    if let Ok(program_data) = std::env::var("PROGRAMDATA") {
        let path = PathBuf::from(program_data).join("Microsoft\\User Account Pictures\\user.png");
        if let Some(data) = read_image(&path) {
            candidates.push(data);
        }
    }
    candidates.into_iter().max_by_key(|data| data.len()).map(|data| data_url(&data))
}

fn image_data(path: &Path) -> Option<String> {
    read_image(path).map(|data| data_url(&data))
}

fn read_image(path: &Path) -> Option<Vec<u8>> {
    let data = fs::read(path).ok()?;
    if is_png(&data) || is_jpeg(&data) {
        return Some(data);
    }
    let (width, height, rgba) = wic_decode(&data).ok()?;
    encode_avatar_png(width, height, rgba).ok()
}

fn data_url(data: &[u8]) -> String {
    let mime = if is_png(data) { "image/png" } else { "image/jpeg" };
    format!("data:{mime};base64,{}", base64::engine::general_purpose::STANDARD.encode(data))
}

fn is_png(data: &[u8]) -> bool {
    data.starts_with(b"\x89PNG\r\n\x1a\n")
}

fn is_jpeg(data: &[u8]) -> bool {
    data.starts_with(&[0xff, 0xd8, 0xff])
}

fn extract_embedded_images(path: &Path) -> Vec<Vec<u8>> {
    let Ok(data) = fs::read(path) else { return Vec::new() };
    let mut out = Vec::new();
    let mut i = 0usize;
    while i + 8 < data.len() {
        if is_png(&data[i..]) {
            if let Some(end) = data[i..].windows(8).position(|w| w == [0, 0, 0, 0x49, 0x45, 0x4e, 0x44, 0xae]) {
                out.push(data[i..i + end + 8].to_vec());
                i += end + 8;
                continue;
            }
        }
        if is_jpeg(&data[i..]) {
            if let Some(end) = data[i + 3..].windows(2).position(|w| w == [0xff, 0xd9]) {
                out.push(data[i..i + end + 5].to_vec());
                i += end + 5;
                continue;
            }
        }
        i += 1;
    }
    out
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UserInfo {
    pub display_name: String,
    pub account_name: String,
    pub computer_name: String,
    pub administrator: bool,
}

pub fn user_info() -> UserInfo {
    let user = std::env::var("USERNAME").unwrap_or_else(|_| "Usuario".into());
    let domain = std::env::var("USERDOMAIN").unwrap_or_else(|_| "EQUIPO".into());
    let computer = std::env::var("COMPUTERNAME").unwrap_or_else(|_| domain.clone());
    UserInfo {
        display_name: user.clone(),
        account_name: format!("{domain}\\{user}"),
        computer_name: computer,
        administrator: unsafe { IsUserAnAdmin().as_bool() },
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WifiNetwork {
    pub ssid: String,
    pub signal: u32,
    pub connected: bool,
    pub secure: bool,
    pub profile: bool,
}

/// Fusiona en `out` las redes que Windows conoce ahora mismo para una interfaz.
unsafe fn collect_networks(client: HANDLE, guid: &windows::core::GUID, out: &mut HashMap<String, WifiNetwork>) {
    let mut list: *mut WLAN_AVAILABLE_NETWORK_LIST = std::ptr::null_mut();
    if WlanGetAvailableNetworkList(client, guid, 0, None, &mut list) != 0 || list.is_null() { return; }
    let count = (*list).dwNumberOfItems as usize;
    for network in std::slice::from_raw_parts((*list).Network.as_ptr(), count) {
        let ssid = ssid_text(&network.dot11Ssid.ucSSID, network.dot11Ssid.uSSIDLength);
        if ssid.is_empty() { continue; }
        let item = out.entry(ssid.clone()).or_insert(WifiNetwork { ssid, signal: 0, connected: false, secure: false, profile: false });
        item.signal = item.signal.max(network.wlanSignalQuality);
        item.connected |= network.dwFlags & WLAN_AVAILABLE_NETWORK_CONNECTED != 0;
        item.secure |= network.bSecurityEnabled.as_bool();
        item.profile |= network.dwFlags & WLAN_AVAILABLE_NETWORK_HAS_PROFILE != 0;
    }
    WlanFreeMemory(list as *const _);
}

pub fn wifi_networks() -> Vec<WifiNetwork> {
    let mut out: HashMap<String, WifiNetwork> = HashMap::new();
    unsafe {
        let mut negotiated = 0u32;
        let mut client = HANDLE::default();
        if WlanOpenHandle(2, None, &mut negotiated, &mut client) != 0 {
            return Vec::new();
        }
        let mut interfaces: *mut WLAN_INTERFACE_INFO_LIST = std::ptr::null_mut();
        if WlanEnumInterfaces(client, None, &mut interfaces) == 0 && !interfaces.is_null() {
            let count = (*interfaces).dwNumberOfItems as usize;
            let guids: Vec<_> = std::slice::from_raw_parts((*interfaces).InterfaceInfo.as_ptr(), count).iter().map(|i| i.InterfaceGuid).collect();
            WlanFreeMemory(interfaces as *const _);
            // 1) Resultados en caché (lo que Windows ya vio). 2) Nuevo escaneo.
            // 3) Lecturas durante ~3 s: mientras escanea, Windows vacía la lista y
            //    leerla una sola vez justo después de WlanScan devolvía 0-1 redes.
            for guid in &guids { collect_networks(client, guid, &mut out); }
            for guid in &guids { let _ = WlanScan(client, guid, None, None, None); }
            for _ in 0..6 {
                std::thread::sleep(Duration::from_millis(500));
                for guid in &guids { collect_networks(client, guid, &mut out); }
            }
        }
        WlanCloseHandle(client, None);
    }
    let mut values: Vec<_> = out.into_values().collect();
    values.sort_by(|a, b| b.connected.cmp(&a.connected).then_with(|| b.signal.cmp(&a.signal)).then_with(|| a.ssid.cmp(&b.ssid)));
    values
}
fn ssid_text(bytes: &[u8; 32], len: u32) -> String {
    String::from_utf8_lossy(&bytes[..(len as usize).min(32)]).trim().to_string()
}

pub fn wifi_connect(ssid: String) -> Result<(), String> {
    unsafe {
        let mut negotiated = 0u32;
        let mut client = HANDLE::default();
        if WlanOpenHandle(2, None, &mut negotiated, &mut client) != 0 { return Err("No se pudo abrir WLAN.".into()); }
        let mut interfaces: *mut WLAN_INTERFACE_INFO_LIST = std::ptr::null_mut();
        if WlanEnumInterfaces(client, None, &mut interfaces) != 0 || interfaces.is_null() {
            WlanCloseHandle(client, None);
            return Err("No hay adaptadores Wi-Fi disponibles.".into());
        }
        let mut profile = None;
        let count = (*interfaces).dwNumberOfItems as usize;
        'outer: for interface in std::slice::from_raw_parts((*interfaces).InterfaceInfo.as_ptr(), count) {
            let mut list: *mut WLAN_AVAILABLE_NETWORK_LIST = std::ptr::null_mut();
            if WlanGetAvailableNetworkList(client, &interface.InterfaceGuid, 0, None, &mut list) != 0 || list.is_null() { continue; }
            let n = (*list).dwNumberOfItems as usize;
            for network in std::slice::from_raw_parts((*list).Network.as_ptr(), n) {
                if ssid_text(&network.dot11Ssid.ucSSID, network.dot11Ssid.uSSIDLength) == ssid && network.dwFlags & WLAN_AVAILABLE_NETWORK_HAS_PROFILE != 0 {
                    profile = Some((interface.InterfaceGuid, utf16_text(&network.strProfileName)));
                    break 'outer;
                }
            }
            WlanFreeMemory(list as *const _);
        }
        WlanFreeMemory(interfaces as *const _);
        if let Some((guid, profile)) = profile {
            let wide: Vec<u16> = profile.encode_utf16().chain(std::iter::once(0)).collect();
            let params = WLAN_CONNECTION_PARAMETERS {
                wlanConnectionMode: wlan_connection_mode_profile,
                strProfile: PCWSTR(wide.as_ptr()),
                pDot11Ssid: std::ptr::null_mut(),
                pDesiredBssidList: std::ptr::null_mut(),
                dot11BssType: dot11_BSS_type_infrastructure,
                dwFlags: 0,
            };
            let code = WlanConnect(client, &guid, &params, None);
            WlanCloseHandle(client, None);
            if code == 0 { return Ok(()); }
        } else {
            WlanCloseHandle(client, None);
        }
    }
    open_uri("ms-availablenetworks:")
}

fn utf16_text(value: &[u16]) -> String {
    let end = value.iter().position(|c| *c == 0).unwrap_or(value.len());
    String::from_utf16_lossy(&value[..end])
}

pub fn bt_devices() -> Vec<String> {
    statuswatch::connected_bluetooth().map(|v| v.into_values().collect()).unwrap_or_default()
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BatteryInfo {
    pub level: u8,
    pub charging: bool,
    pub ac: bool,
}

pub fn battery_info() -> Option<BatteryInfo> {
    let mut status = windows::Win32::System::Power::SYSTEM_POWER_STATUS::default();
    unsafe { windows::Win32::System::Power::GetSystemPowerStatus(&mut status).ok()?; }
    if status.BatteryLifePercent == 255 { return None; }
    Some(BatteryInfo { level: status.BatteryLifePercent, charging: status.ACLineStatus == 1 && status.BatteryFlag != 255 && status.BatteryFlag & 8 != 0, ac: status.ACLineStatus == 1 })
}

// One Windows snapshot for every WebView; polling also detects charger changes.
fn battery_snapshot() -> &'static Mutex<Option<BatteryInfo>> {
    static SNAPSHOT: OnceLock<Mutex<Option<BatteryInfo>>> = OnceLock::new();
    SNAPSHOT.get_or_init(|| Mutex::new(battery_info()))
}

pub fn start_battery_watch(app: AppHandle) {
    std::thread::spawn(move || loop {
        let current = battery_info();
        *battery_snapshot().lock().unwrap() = current.clone();
        let _ = app.emit("battery-info", &current);
        std::thread::sleep(Duration::from_secs(2));
    });
}

pub fn send_keys(combo: String) -> Result<(), String> {
    let combo = combo.to_lowercase();
    let keys: Vec<VIRTUAL_KEY> = match combo.as_str() {
        "win+d" | "desktop" => vec![VIRTUAL_KEY(0x44)],
        "win+s" | "search" => vec![VIRTUAL_KEY(0x53)],
        "win+shift+s" | "capture" => vec![VIRTUAL_KEY(0x10), VIRTUAL_KEY(0x53)],
        "win+shift+r" | "record" => vec![VIRTUAL_KEY(0x10), VIRTUAL_KEY(0x52)],
        "win+n" | "notifications" => vec![VIRTUAL_KEY(0x4e)],
        "win+a" | "control" => vec![VIRTUAL_KEY(0x41)],
        _ => return Err("Combinación no admitida.".into()),
    };
    let mut inputs = Vec::with_capacity(keys.len() * 2 + 2);
    inputs.push(key_input(VK_LWIN, KEYEVENTF_EXTENDEDKEY));
    for key in &keys { inputs.push(key_input(*key, Default::default())); }
    for key in keys.iter().rev() { inputs.push(key_input(*key, KEYEVENTF_KEYUP)); }
    inputs.push(key_input(VK_LWIN, KEYEVENTF_EXTENDEDKEY | KEYEVENTF_KEYUP));
    let sent = unsafe { SendInput(&inputs, std::mem::size_of::<INPUT>() as i32) };
    (sent == inputs.len() as u32).then_some(()).ok_or_else(|| "Windows no aceptó la combinación.".into())
}

fn key_input(key: VIRTUAL_KEY, flags: windows::Win32::UI::Input::KeyboardAndMouse::KEYBD_EVENT_FLAGS) -> INPUT {
    INPUT { r#type: INPUT_KEYBOARD, Anonymous: windows::Win32::UI::Input::KeyboardAndMouse::INPUT_0 { ki: KEYBDINPUT { wVk: key, wScan: 0, dwFlags: flags, time: 0, dwExtraInfo: 0 } } }
}

fn open_uri(uri: &str) -> Result<(), String> {
    let result = unsafe { ShellExecuteW(None, PCWSTR::null(), &HSTRING::from(uri), None, None, windows::Win32::UI::WindowsAndMessaging::SW_SHOWNORMAL) };
    (result.0 as isize > 32).then_some(()).ok_or_else(|| "Windows no pudo abrir este destino.".into())
}

pub fn audio_output_name() -> String {
    system::audio_output_name()
}

fn open_bar_panel_impl(app: &AppHandle, kind: String, anchor_x: i32) -> Result<(), String> {
    if !["wifi", "bluetooth", "sound", "battery", "control", "user"].contains(&kind.as_str()) {
        return Err("Panel desconocido.".into());
    }
    let popup_id = format!("bar_panel:{kind}");
    if !popups::toggle(app, &popup_id)? { return Ok(()); }
    let Some(panel) = create_panel(app) else { return Err("No se pudo crear el panel.".into()) };
    {
        let mut state = panel_state().lock().unwrap();
        state.kind = Some(kind.clone());
        state.anchor_x = anchor_x;
        // Se conserva el último alto medido: si el contenido no cambia, el panel no vuelve a informar.
        if state.height <= 0 { state.height = 320; }
    }
    position_panel(app, &panel);
    let _ = panel.emit("bar-panel", &kind);
    popups::present(&panel, "bar_panel");
    Ok(())
}

fn position_panel(app: &AppHandle, panel: &WebviewWindow) {
    if app.get_webview_window("topbar").is_none() { return; }
    let settings = app.state::<crate::notch::NotchState>().settings.lock().unwrap().clone();
    let Some((_, monitor)) = notch::assignments(&settings, &notch::monitors(app)).into_iter().find(|(l, _)| l == "main") else { return };
    let (anchor_x, panel_height) = {
        let state = panel_state().lock().unwrap();
        (state.anchor_x, state.height)
    };
    let scale = monitor.scale;
    let width = physical(PANEL_WIDTH + PANEL_MARGIN * 2, scale);
    let height = physical(panel_height + PANEL_MARGIN * 2, scale);
    let anchor = monitor.rect.left + physical(anchor_x, scale);
    let mut left = anchor - width / 2;
    left = left.clamp(monitor.rect.left + physical(TOPBAR_MARGIN, scale), monitor.rect.right - width - physical(TOPBAR_MARGIN, scale));
    let bar_height = physical(settings.topbar.height, scale);
    let top = monitor.rect.top + bar_height - physical(PANEL_MARGIN, scale);
    let ui_panel = panel.clone();
    let _ = panel.run_on_main_thread(move || {
        let _ = ui_panel.set_position(PhysicalPosition::new(left, top));
        let _ = ui_panel.set_size(PhysicalSize::new(width as u32, height as u32));
    });
}

fn close_bar_panel_impl(app: &AppHandle) {
    popups::close(app, "bar_panel");
    let mut state = panel_state().lock().unwrap();
    state.kind = None;
}

fn resize_bar_panel_impl(app: &AppHandle, height: i32) {
    panel_state().lock().unwrap().height = height.clamp(60, 700);
    if let Some(panel) = app.get_webview_window("barpanel") { position_panel(app, &panel); }
}

async fn blocking_with_timeout<T, F>(duration: Duration, work: F) -> Option<T>
where
    T: Send + 'static,
    F: FnOnce() -> T + Send + 'static,
{
    match tokio::time::timeout(duration, tauri::async_runtime::spawn_blocking(work)).await {
        Ok(Ok(value)) => Some(value),
        _ => None,
    }
}

#[tauri::command]
pub async fn command_user_avatar(state: tauri::State<'_, notch::NotchState>) -> Result<Option<String>, String> {
    let settings = state.settings.lock().unwrap().clone();
    Ok(blocking_with_timeout(Duration::from_secs(4), move || user_avatar(&settings)).await.flatten())
}

fn wic_decode(bytes: &[u8]) -> Result<(u32, u32, Vec<u8>), String> {
    unsafe {
        let _ = windows::Win32::System::Com::CoInitializeEx(None, windows::Win32::System::Com::COINIT_MULTITHREADED);
        let factory: IWICImagingFactory = CoCreateInstance(&CLSID_WICImagingFactory, None, CLSCTX_INPROC_SERVER).map_err(|e| e.to_string())?;
        let stream = factory.CreateStream().map_err(|e| e.to_string())?;
        stream.InitializeFromMemory(bytes).map_err(|e| e.to_string())?;
        let decoder = factory.CreateDecoderFromStream(&stream, std::ptr::null(), WICDecodeMetadataCacheOnLoad).map_err(|e| e.to_string())?;
        let frame = decoder.GetFrame(0).map_err(|e| e.to_string())?;
        let mut width = 0u32;
        let mut height = 0u32;
        frame.GetSize(&mut width, &mut height).map_err(|e| e.to_string())?;
        if width == 0 || height == 0 || width.saturating_mul(height) > 16_777_216 { return Err("La imagen tiene demasiados píxeles.".into()); }
        let mut bgra = vec![0u8; (width * height * 4) as usize];
        let source = if frame.GetPixelFormat().map_err(|e| e.to_string())? != GUID_WICPixelFormat32bppPBGRA {
            WICConvertBitmapSource(&GUID_WICPixelFormat32bppPBGRA, &frame).map_err(|e| e.to_string())?
        } else {
            frame.clone().into()
        };
        source.CopyPixels(std::ptr::null(), width * 4, &mut bgra).map_err(|e| e.to_string())?;
        for px in bgra.chunks_exact_mut(4) {
            let (b, g, r, a) = (px[0], px[1], px[2], px[3]);
            px[0] = r; px[1] = g; px[2] = b; px[3] = a;
        }
        Ok((width, height, bgra))
    }
}

fn encode_avatar_png(width: u32, height: u32, rgba: Vec<u8>) -> Result<Vec<u8>, String> {
    let source = image::RgbaImage::from_raw(width, height, rgba).ok_or("La imagen no tiene un formato válido.")?;
    let resized = image::imageops::resize(&source, 256, 256, image::imageops::FilterType::Lanczos3);
    let mut bytes = Vec::new();
    image::codecs::png::PngEncoder::new(&mut bytes)
        .write_image(&resized, 256, 256, image::ExtendedColorType::Rgba8)
        .map_err(|e| e.to_string())?;
    Ok(bytes)
}

fn save_avatar_png(source: &Path, destination: &Path) -> Result<(), String> {
    let metadata = fs::metadata(source).map_err(|e| format!("No se pudo leer la imagen: {e}"))?;
    if metadata.len() > 15 * 1024 * 1024 {
        return Err("La imagen no puede superar 15 MB.".into());
    }
    let source_bytes = fs::read(source).map_err(|e| e.to_string())?;
    let (width, height, rgba) = wic_decode(&source_bytes).map_err(|e| format!("Formato de imagen no válido: {e}"))?;
    if let Some(parent) = destination.parent() { fs::create_dir_all(parent).map_err(|e| e.to_string())?; }
    fs::write(destination, encode_avatar_png(width, height, rgba)?).map_err(|e| e.to_string())
}

/// Normaliza una imagen elegida a PNG local para que ninguna ventana tenga que abrir una ruta del usuario.
#[tauri::command]
pub async fn set_avatar_image(app: AppHandle, state: tauri::State<'_, notch::NotchState>, path: String) -> Result<String, String> {
    let current = state.settings.lock().unwrap().clone();
    let data_dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    let result = blocking_with_timeout(Duration::from_secs(8), move || {
        let mut next = current.clone();
        if path.trim().is_empty() {
            next.topbar.avatar_path.clear();
            let url = user_avatar_uncached(&next);
            return Ok::<(Settings, String), String>((next, url.unwrap_or_default()));
        }
        let destination = data_dir.join("avatar.png");
        save_avatar_png(Path::new(path.trim()), &destination)?;
        next.topbar.avatar_path = destination.to_string_lossy().into_owned();
        let url = data_url(&fs::read(&destination).map_err(|e| e.to_string())?);
        Ok((next, url))
    }).await.ok_or_else(|| "La imagen tardó demasiado en procesarse.".to_string())??;
    *avatar_cache().lock().unwrap() = None;
    let (next, url) = result;
    let _ = crate::commit_settings(&app, next).await?;
    let _ = app.emit("avatar-changed", &url);
    Ok(url)
}

#[tauri::command]
pub async fn command_user_info() -> UserInfo {
    blocking_with_timeout(Duration::from_secs(2), user_info).await.unwrap_or_else(|| UserInfo {
        display_name: "Usuario".into(), account_name: String::new(), computer_name: String::new(), administrator: false,
    })
}

#[tauri::command]
pub async fn command_wifi_networks() -> Vec<WifiNetwork> {
    blocking_with_timeout(Duration::from_secs(6), wifi_networks).await.unwrap_or_default()
}

#[tauri::command]
pub async fn command_wifi_connect(ssid: String) -> Result<(), String> {
    blocking_with_timeout(Duration::from_secs(4), move || wifi_connect(ssid)).await.unwrap_or_else(|| Err("La conexión Wi-Fi tardó demasiado.".into()))
}

#[tauri::command]
pub async fn command_bt_devices() -> Vec<String> {
    blocking_with_timeout(Duration::from_secs(3), || unsafe { let _ = RoInitialize(RO_INIT_MULTITHREADED); bt_devices() }).await.unwrap_or_default()
}

#[tauri::command]
pub async fn command_audio_output_name() -> String {
    blocking_with_timeout(Duration::from_secs(2), audio_output_name).await.unwrap_or_else(|| "Salida predeterminada".into())
}

#[tauri::command]
pub fn command_send_keys(combo: String) -> Result<(), String> { send_keys(combo) }

const TRAY_OVERFLOW_CLASSES: [&str; 2] = [
    "NotifyIconOverflowWindow",
    "TopLevelWindowForOverflowXamlIsland",
];

fn tray_is_open(hwnd: HWND) -> bool {
    use windows::Win32::{
        Graphics::Dwm::{DwmGetWindowAttribute, DWMWA_CLOAKED},
        UI::WindowsAndMessaging::IsWindowVisible,
    };
    unsafe {
        if !IsWindowVisible(hwnd).as_bool() { return false; }
        // Explorer puede mantener visible el HWND de XAML aunque DWM ya haya ocultado su contenido.
        let mut cloaked = 0u32;
        let status = DwmGetWindowAttribute(
            hwnd, DWMWA_CLOAKED, (&mut cloaked as *mut u32).cast(),
            std::mem::size_of_val(&cloaked) as u32,
        );
        status.is_err() || cloaked == 0
    }
}

#[derive(Clone, Copy)]
struct TrayPlacement {
    center_x: i32,
    top: i32,
    min_x: i32,
    max_x: i32,
}

impl TrayPlacement {
    fn from_button(window: &WebviewWindow, x: i32, bottom: i32) -> Result<Self, String> {
        use windows::Win32::Graphics::Gdi::{ClientToScreen, GetMonitorInfoW, MonitorFromPoint, MONITORINFO, MONITOR_DEFAULTTONEAREST};
        let hwnd = window.hwnd().map_err(|e| e.to_string())?;
        let scale = window.scale_factor().map_err(|e| e.to_string())?;
        let mut button = POINT { x, y: bottom };
        let mut monitor = MONITORINFO { cbSize: std::mem::size_of::<MONITORINFO>() as u32, ..Default::default() };
        unsafe {
            ClientToScreen(hwnd, &mut button).ok().map_err(|e| e.to_string())?;
            let handle = MonitorFromPoint(button, MONITOR_DEFAULTTONEAREST);
            GetMonitorInfoW(handle, &mut monitor).ok().map_err(|e| e.to_string())?;
        }
        let margin = physical(TOPBAR_MARGIN, scale);
        Ok(Self {
            center_x: button.x,
            top: button.y + physical(4, scale),
            min_x: monitor.rcMonitor.left + margin,
            max_x: monitor.rcMonitor.right - margin,
        })
    }

    fn apply(self, hwnd: HWND) {
        use windows::Win32::UI::WindowsAndMessaging::{GetWindowRect, SWP_NOZORDER};
        let mut rect = windows::Win32::Foundation::RECT::default();
        unsafe {
            if GetWindowRect(hwnd, &mut rect).is_err() { return; }
            let width = rect.right - rect.left;
            let left = (self.center_x - width / 2).clamp(self.min_x, (self.max_x - width).max(self.min_x));
            if width > 0 && (rect.left != left || rect.top != self.top) {
                let _ = SetWindowPos(
                    hwnd, None, left, self.top, 0, 0,
                    SWP_NOSIZE | SWP_NOZORDER | SWP_NOACTIVATE | SWP_NOSENDCHANGING,
                );
            }
        }
    }

    fn apply_to_panels(self) -> bool {
        // Incluye ventanas ocultas para prepararlas antes de Enter y resuelve un HWND nuevo si
        // Explorer crea o sustituye el host XAML durante la apertura.
        use windows::Win32::UI::WindowsAndMessaging::FindWindowW;
        let mut visible = false;
        for class in TRAY_OVERFLOW_CLASSES {
            if let Ok(hwnd) = unsafe { FindWindowW(&HSTRING::from(class), PCWSTR::null()) } {
                self.apply(hwnd);
                visible |= tray_is_open(hwnd);
            }
        }
        visible
    }
}

thread_local! {
    // Los eventos OUTOFCONTEXT se entregan en el hilo que instaló el hook.
    static TRAY_PLACEMENT: std::cell::Cell<Option<TrayPlacement>> = const { std::cell::Cell::new(None) };
}
static TRAY_SESSION: Mutex<()> = Mutex::new(());
static TRAY_CLOSED_AT: Mutex<Option<std::time::Instant>> = Mutex::new(None);

unsafe extern "system" fn tray_win_event(
    _hook: windows::Win32::UI::Accessibility::HWINEVENTHOOK, _event: u32, hwnd: HWND,
    object: i32, child: i32, _thread: u32, _time: u32,
) {
    use windows::Win32::UI::WindowsAndMessaging::GetClassNameW;
    if hwnd.0.is_null() || object != 0 || child != 0 { return; }
    let mut class = [0u16; 64];
    let len = GetClassNameW(hwnd, &mut class);
    if len <= 0 || !TRAY_OVERFLOW_CLASSES.iter().any(|name| name.encode_utf16().eq(class[..len as usize].iter().copied())) {
        return;
    }
    TRAY_PLACEMENT.with(|target| {
        if let Some(placement) = target.get() { placement.apply(hwnd); }
    });
}

/// Mantiene activa la cola que recibe los WinEvents, con una comprobación de respaldo al vencer
/// el plazo. En la apertura el plazo es corto porque XAML reajusta varias veces tamaño y posición.
fn pump_tray(placement: TrayPlacement, duration: Duration, interval_ms: u32) -> bool {
    use windows::Win32::UI::WindowsAndMessaging::{
        DispatchMessageW, MsgWaitForMultipleObjectsEx, PeekMessageW, MSG, MWMO_INPUTAVAILABLE, PM_REMOVE, QS_ALLINPUT,
    };
    let until = std::time::Instant::now() + duration;
    let mut visible = placement.apply_to_panels();
    loop {
        let remaining = until.saturating_duration_since(std::time::Instant::now());
        if remaining.is_zero() { return visible; }
        unsafe {
            let _ = MsgWaitForMultipleObjectsEx(None, interval_ms.min(remaining.as_millis().max(1) as u32), QS_ALLINPUT, MWMO_INPUTAVAILABLE);
            let mut msg = MSG::default();
            while PeekMessageW(&mut msg, None, 0, 0, PM_REMOVE).as_bool() { DispatchMessageW(&msg); }
        }
        visible = placement.apply_to_panels();
    }
}

fn dismiss_tray_overflow() -> bool {
    use windows::Win32::UI::{
        Input::KeyboardAndMouse::VK_ESCAPE,
        WindowsAndMessaging::{FindWindowW, PostMessageW, WM_CLOSE},
    };
    use windows::Win32::Foundation::WPARAM;
    let mut found = false;
    unsafe {
        for class in TRAY_OVERFLOW_CLASSES {
            let Ok(hwnd) = FindWindowW(&HSTRING::from(class), PCWSTR::null()) else { continue; };
            if !tray_is_open(hwnd) { continue; }
            found = true;
            if GetForegroundWindow() == hwnd {
                let escape = [key_input(VK_ESCAPE, Default::default()), key_input(VK_ESCAPE, KEYEVENTF_KEYUP)];
                SendInput(&escape, std::mem::size_of::<INPUT>() as i32);
            } else {
                let _ = PostMessageW(Some(hwnd), WM_CLOSE, WPARAM(0), LPARAM(0));
            }
        }
    }
    found
}

fn watch_tray_overflow(placement: TrayPlacement, opened: tokio::sync::oneshot::Sender<Result<(), String>>) {
    use windows::Win32::UI::{
        Accessibility::{SetWinEventHook, UnhookWinEvent, HWINEVENTHOOK},
        WindowsAndMessaging::{
            EVENT_OBJECT_CLOAKED, EVENT_OBJECT_LOCATIONCHANGE, EVENT_OBJECT_SHOW, EVENT_OBJECT_UNCLOAKED,
            WINEVENT_OUTOFCONTEXT, WINEVENT_SKIPOWNPROCESS,
        },
    };
    // La barra de Noki no toma el foco: un segundo clic puede dejar el recuadro abierto.
    if dismiss_tray_overflow() { let _ = opened.send(Ok(())); return; }
    let Ok(_session) = TRAY_SESSION.try_lock() else { let _ = opened.send(Ok(())); return; };
    if TRAY_CLOSED_AT.lock().unwrap().is_some_and(|at| at.elapsed() < Duration::from_millis(300)) {
        let _ = opened.send(Ok(()));
        return;
    }

    struct Hooks([HWINEVENTHOOK; 4]);
    impl Drop for Hooks {
        fn drop(&mut self) {
            for hook in &self.0 {
                if !hook.is_invalid() { unsafe { let _ = UnhookWinEvent(*hook); } }
            }
            TRAY_PLACEMENT.with(|target| target.set(None));
        }
    }
    TRAY_PLACEMENT.with(|target| target.set(Some(placement)));
    let _hooks = Hooks([
        EVENT_OBJECT_SHOW, EVENT_OBJECT_LOCATIONCHANGE, EVENT_OBJECT_CLOAKED, EVENT_OBJECT_UNCLOAKED,
    ].map(|event| unsafe {
        // No depende del PID o HWND del panel antes de abrir. El callback filtra las dos clases.
        SetWinEventHook(event, event, None, Some(tray_win_event), 0, 0, WINEVENT_OUTOFCONTEXT | WINEVENT_SKIPOWNPROCESS)
    }));
    let mut shown = placement.apply_to_panels();

    let focus_tray = [
        key_input(VK_LWIN, KEYEVENTF_EXTENDEDKEY),
        key_input(VIRTUAL_KEY(0x42), Default::default()),
        key_input(VIRTUAL_KEY(0x42), KEYEVENTF_KEYUP),
        key_input(VK_LWIN, KEYEVENTF_EXTENDEDKEY | KEYEVENTF_KEYUP),
    ];
    let enter = [key_input(VIRTUAL_KEY(0x0D), Default::default()), key_input(VIRTUAL_KEY(0x0D), KEYEVENTF_KEYUP)];
    for timeout in [700, 1500] {
        if shown { break; }
        unsafe { SendInput(&focus_tray, std::mem::size_of::<INPUT>() as i32); }
        pump_tray(placement, Duration::from_millis(220), 2);
        unsafe { SendInput(&enter, std::mem::size_of::<INPUT>() as i32); }
        let deadline = std::time::Instant::now() + Duration::from_millis(timeout);
        while std::time::Instant::now() < deadline {
            if pump_tray(placement, Duration::from_millis(20), 2) {
                shown = true;
                break;
            }
        }
    }
    if !shown {
        let _ = opened.send(Err("Windows no abrió el panel de iconos ocultos.".into()));
        return;
    }

    let _ = opened.send(Ok(()));
    // XAML puede volver a anclar el panel después de la animación inicial. Conserva los hooks
    // hasta que hayan desaparecido ambos hosts y tolera una sustitución breve entre ellos.
    let mut hidden_since = None;
    loop {
        if pump_tray(placement, Duration::from_millis(25), 25) {
            hidden_since = None;
        } else if hidden_since.get_or_insert_with(std::time::Instant::now).elapsed() >= Duration::from_millis(90) {
            *TRAY_CLOSED_AT.lock().unwrap() = hidden_since;
            break;
        }
    }
}

#[tauri::command]
pub async fn command_tray_overflow(window: WebviewWindow, anchor_x: i32, anchor_bottom: i32) -> Result<(), String> {
    if window.label() != "topbar" { return Err("El anclaje debe pertenecer a la barra superior.".into()); }
    let placement = TrayPlacement::from_button(&window, anchor_x, anchor_bottom)?;
    close_bar_panel_impl(window.app_handle());
    let (opened, ready) = tokio::sync::oneshot::channel();
    std::thread::spawn(move || watch_tray_overflow(placement, opened));
    ready.await.map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn command_open_bar_panel(app: AppHandle, kind: String, anchor_x: i32) -> Result<(), String> { open_bar_panel_impl(&app, kind, anchor_x) }

#[tauri::command]
pub async fn command_close_bar_panel(app: AppHandle) { close_bar_panel_impl(&app); }

#[tauri::command]
pub async fn command_resize_bar_panel(app: AppHandle, height: i32) { resize_bar_panel_impl(&app, height); }

#[tauri::command]
pub fn command_bar_panel_state() -> Option<String> { panel_state().lock().unwrap().kind.clone() }

#[tauri::command]
pub async fn command_battery_info() -> Option<BatteryInfo> {
    battery_snapshot().lock().unwrap().clone()
}

#[tauri::command]
pub async fn command_open_system_uri(uri: String) -> Result<(), String> {
    let allowed = uri.starts_with("ms-settings") || uri.starts_with("ms-screenclip:") || uri.starts_with("ms-actioncenter:") || uri == "ms-availablenetworks:";
    if !allowed { return Err("Destino de Windows no permitido.".into()); }
    blocking_with_timeout(Duration::from_secs(2), move || open_uri(&uri)).await.unwrap_or_else(|| Err("Windows tardó demasiado en abrir este destino.".into()))
}

#[tauri::command]
pub async fn command_lock_user() -> Result<(), String> {
    blocking_with_timeout(Duration::from_secs(2), || std::process::Command::new("rundll32.exe").args(["user32.dll,LockWorkStation"]).spawn().map(|_| ()).map_err(|e| e.to_string())).await.unwrap_or_else(|| Err("Windows tardó demasiado en bloquear la sesión.".into()))
}

#[tauri::command]
pub async fn command_logoff_user() -> Result<(), String> {
    blocking_with_timeout(Duration::from_secs(2), || std::process::Command::new("shutdown.exe").args(["/l"]).spawn().map(|_| ()).map_err(|e| e.to_string())).await.unwrap_or_else(|| Err("Windows tardó demasiado en cerrar la sesión.".into()))
}

#[tauri::command]
pub async fn command_toggle_dark(dark: bool) -> Result<(), String> {
    tokio::time::timeout(Duration::from_secs(2), crate::quick::quick_set("dark".into(), dark)).await.map_err(|_| "No se pudo cambiar el modo oscuro.".to_string())??;
    Ok(())
}

#[tauri::command]
pub async fn command_toggle_airplane(on: bool) -> Result<(), String> {
    tokio::time::timeout(Duration::from_secs(3), async move {
        crate::quick::quick_set("wifi".into(), !on).await?;
        crate::quick::quick_set("bluetooth".into(), !on).await
    }).await.map_err(|_| "No se pudo cambiar el modo avión.".to_string())??;
    Ok(())
}

#[tauri::command]
pub async fn command_open_task_manager() -> Result<(), String> {
    blocking_with_timeout(Duration::from_secs(2), || std::process::Command::new("taskmgr.exe").spawn().map(|_| ()).map_err(|e| e.to_string())).await.unwrap_or_else(|| Err("Windows tardó demasiado en abrir el Administrador de tareas.".into()))
}

