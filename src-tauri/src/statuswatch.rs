//! Vigilante del estado del sistema para las alertas del notch: Wi-Fi, Bluetooth y micrófono en uso.
//! (La batería y el cargador los vigila el WebView; aquí solo va lo que necesita APIs de Windows.)
//! Emite `sys-alert` ({ kind, icon, text, tone }) con cada cambio y `mic-state` ({ active, app }).

use serde_json::json;
use std::{collections::HashMap, time::Duration};
use tauri::{AppHandle, Emitter};
use windows::{
    core::{w, HSTRING, PCWSTR, PWSTR},
    Devices::Enumeration::{DeviceInformation, DeviceInformationKind},

    Win32::{
        Foundation::{ERROR_SUCCESS, HANDLE},
        NetworkManagement::WiFi::{
            wlan_intf_opcode_current_connection, wlan_interface_state_connected, WlanCloseHandle, WlanEnumInterfaces,
            WlanFreeMemory, WlanOpenHandle, WlanQueryInterface, WLAN_CONNECTION_ATTRIBUTES, WLAN_INTERFACE_INFO_LIST,
        },
        System::{
            Registry::{
                RegCloseKey, RegEnumKeyExW, RegOpenKeyExW, RegQueryValueExW, HKEY, HKEY_CURRENT_USER, KEY_READ, REG_QWORD,
                REG_VALUE_TYPE,
            },
            WinRT::{RoInitialize, RO_INIT_MULTITHREADED},
        },
    },
};

pub fn start(app: AppHandle) {
    let a = app.clone();
    std::thread::spawn(move || wifi_loop(a));
    let a = app.clone();
    std::thread::spawn(move || bluetooth_loop(a));
    std::thread::spawn(move || mic_loop(app));
}

fn alert(app: &AppHandle, kind: &str, icon: &str, text: String, warn: bool) {
    let _ = app.emit("sys-alert", json!({ "kind": kind, "icon": icon, "text": text, "tone": if warn { "warn" } else { "" } }));
}

// ───────────── Wi-Fi ─────────────

/// `Some(Some(ssid))` conectado, `Some(None)` hay adaptador pero sin conexión, `None` no hay Wi-Fi.
fn wifi_status() -> Option<Option<String>> {
    unsafe {
        let mut negotiated = 0u32;
        let mut handle = HANDLE::default();
        if WlanOpenHandle(2, None, &mut negotiated, &mut handle) != ERROR_SUCCESS.0 {
            return None;
        }
        let mut result: Option<Option<String>> = None;
        let mut list: *mut WLAN_INTERFACE_INFO_LIST = std::ptr::null_mut();
        if WlanEnumInterfaces(handle, None, &mut list) == ERROR_SUCCESS.0 && !list.is_null() {
            let n = (*list).dwNumberOfItems as usize;
            let items = std::slice::from_raw_parts((*list).InterfaceInfo.as_ptr(), n);
            for it in items {
                result = Some(None);
                if it.isState != wlan_interface_state_connected {
                    continue;
                }
                let mut size = 0u32;
                let mut data: *mut std::ffi::c_void = std::ptr::null_mut();
                if WlanQueryInterface(handle, &it.InterfaceGuid, wlan_intf_opcode_current_connection, None, &mut size, &mut data, None)
                    == ERROR_SUCCESS.0
                    && !data.is_null()
                {
                    let attrs = &*(data as *const WLAN_CONNECTION_ATTRIBUTES);
                    let ssid = &attrs.wlanAssociationAttributes.dot11Ssid;
                    let len = (ssid.uSSIDLength as usize).min(32);
                    let name = String::from_utf8_lossy(&ssid.ucSSID[..len]).to_string();
                    WlanFreeMemory(data);
                    result = Some(Some(name));
                    break;
                }
            }
            WlanFreeMemory(list as *const _);
        }
        WlanCloseHandle(handle, None);
        result
    }
}

fn wifi_loop(app: AppHandle) {
    let mut prev: Option<Option<String>> = None;
    let mut pending = 0;
    loop {
        std::thread::sleep(Duration::from_millis(2500));
        let Some(now) = wifi_status() else { continue };
        if let Some(p) = &prev {
            if *p != now {
                // El cambio debe mantenerse un ciclo más (evita parpadeos al saltar entre puntos de acceso).
                pending += 1;
                if pending < 2 {
                    continue;
                }
                match (&now, p) {
                    (Some(ssid), _) => alert(&app, "wifi", "📶", format!("Wi-Fi · {ssid}"), false),
                    (None, Some(_)) => alert(&app, "wifi", "📵", "Wi-Fi desconectado".into(), true),
                    _ => {}
                }
            }
        }
        pending = 0;
        prev = Some(now);
    }
}

// ───────────── Bluetooth ─────────────

pub fn connected_bluetooth() -> Option<HashMap<String, String>> {
    let aqs = HSTRING::from(
        "(System.Devices.Aep.ProtocolId:=\"{e0cbf06c-cd8b-4647-bb8a-263b43f0f974}\" OR System.Devices.Aep.ProtocolId:=\"{bb7bb05e-5972-42b5-94fc-76eaa7084d49}\") AND System.Devices.Aep.IsConnected:=System.StructuredQueryType.Boolean#True",
    );
    let props: Option<&windows_collections::IIterable<HSTRING>> = None;
    let list = DeviceInformation::FindAllAsyncWithKindAqsFilterAndAdditionalProperties(&aqs, props, DeviceInformationKind::AssociationEndpoint)
        .ok()?
        .join()
        .ok()?;
    let mut out = HashMap::new();
    for d in list {
        let name = d.Name().map(|n| n.to_string()).unwrap_or_default();
        if name.trim().is_empty() {
            continue;
        }
        out.insert(d.Id().map(|i| i.to_string()).unwrap_or_default(), name);
    }
    Some(out)
}

fn bluetooth_loop(app: AppHandle) {
    unsafe {
        let _ = RoInitialize(RO_INIT_MULTITHREADED);
    }
    let mut prev: Option<HashMap<String, String>> = None;
    loop {
        std::thread::sleep(Duration::from_millis(4000));
        let Some(now) = connected_bluetooth() else { continue };
        if let Some(p) = &prev {
            for (id, name) in &now {
                if !p.contains_key(id) {
                    alert(&app, "bluetooth", "🔵", format!("{name} conectado"), false);
                }
            }
            for (id, name) in p {
                if !now.contains_key(id) {
                    alert(&app, "bluetooth", "⚪", format!("{name} desconectado"), false);
                }
            }
        }
        prev = Some(now);
    }
}

// ───────────── Micrófono ─────────────

const MIC_KEY: &str = "Software\\Microsoft\\Windows\\CurrentVersion\\CapabilityAccessManager\\ConsentStore\\microphone";

unsafe fn open_key(parent: HKEY, path: &str) -> Option<HKEY> {
    let wide: Vec<u16> = path.encode_utf16().chain(std::iter::once(0)).collect();
    let mut k = HKEY::default();
    (RegOpenKeyExW(parent, PCWSTR(wide.as_ptr()), None, KEY_READ, &mut k) == ERROR_SUCCESS).then_some(k)
}

unsafe fn qword(k: HKEY, name: PCWSTR) -> Option<u64> {
    let mut ty = REG_VALUE_TYPE::default();
    let mut v = 0u64;
    let mut size = 8u32;
    let r = RegQueryValueExW(k, name, None, Some(&mut ty), Some(&mut v as *mut u64 as *mut u8), Some(&mut size));
    (r == ERROR_SUCCESS && ty == REG_QWORD).then_some(v)
}

/// Aplicaciones que usan el micrófono ahora: Windows deja `LastUsedTimeStop = 0` mientras la captura sigue abierta.
unsafe fn scan(parent: HKEY, out: &mut Vec<String>, skip_nonpackaged: bool) {
    let mut i = 0;
    loop {
        let mut buf = [0u16; 512];
        let mut len = buf.len() as u32;
        if RegEnumKeyExW(parent, i, Some(PWSTR(buf.as_mut_ptr())), &mut len, None, None, None, None) != ERROR_SUCCESS {
            break;
        }
        i += 1;
        let name = String::from_utf16_lossy(&buf[..len as usize]);
        if skip_nonpackaged && name == "NonPackaged" {
            continue;
        }
        if let Some(k) = open_key(parent, &name) {
            if let (Some(start), Some(stop)) = (qword(k, w!("LastUsedTimeStart")), qword(k, w!("LastUsedTimeStop"))) {
                if start > 0 && stop == 0 {
                    out.push(name);
                }
            }
            let _ = RegCloseKey(k);
        }
    }
}

fn mic_users() -> Vec<String> {
    let mut out = Vec::new();
    unsafe {
        let Some(root) = open_key(HKEY_CURRENT_USER, MIC_KEY) else { return out };
        scan(root, &mut out, true);
        if let Some(np) = open_key(root, "NonPackaged") {
            scan(np, &mut out, false);
            let _ = RegCloseKey(np);
        }
        let _ = RegCloseKey(root);
    }
    out
}

/// "C:#Program Files#Zoom#bin#Zoom.exe" → "Zoom"; "Microsoft.Teams_8wekyb3d8bbwe" → "Teams".
fn pretty_app(raw: &str) -> String {
    let base = raw.rsplit('#').next().unwrap_or(raw);
    let base = base.split('_').next().unwrap_or(base).trim_end_matches(".exe");
    base.rsplit('.').next().unwrap_or(base).to_string()
}

fn mic_loop(app: AppHandle) {
    let mut prev: Option<bool> = None;
    let mut ticks = 0u32;
    loop {
        std::thread::sleep(Duration::from_millis(1500));
        let users = mic_users();
        let active = !users.is_empty();
        let name = users.first().map(|u| pretty_app(u)).unwrap_or_default();
        ticks += 1;
        // Se reenvía cada ~12 s por si alguna ventana (otra pantalla) se abrió después del cambio.
        if prev != Some(active) || ticks % 8 == 0 {
            let _ = app.emit("mic-state", json!({ "active": active, "app": name }));
        }
        if prev == Some(false) && active {
            let text = if name.is_empty() { "Micrófono en uso".to_string() } else { format!("Micrófono en uso · {name}") };
            alert(&app, "microphone", "🎙️", text, true);
        }
        prev = Some(active);
    }
}

#[cfg(test)]
mod probe {
    use super::*;
    #[test]
    fn probe() {
        unsafe { let _ = RoInitialize(RO_INIT_MULTITHREADED); }
        println!("WIFI={:?}", wifi_status());
        println!("BT={:?}", connected_bluetooth());
        println!("MIC={:?}", mic_users());
    }
}
