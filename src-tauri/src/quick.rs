//! Interruptores rápidos del Panel: Wi-Fi, Bluetooth, mantener activo, modo oscuro, micrófono, sonido,
//! captura de pantalla y bloqueo. Cada uno se lee y se cambia con una llamada del sistema; si un equipo
//! no tiene la función (por ejemplo, sin radio Bluetooth) el estado llega como `null`.

use serde::Serialize;
use std::{
    sync::{
        atomic::{AtomicBool, Ordering::SeqCst},
        Mutex,
    },
    time::Duration,
};
use windows::{
    core::{w, PCWSTR},
    Devices::Radios::{Radio, RadioAccessStatus, RadioKind, RadioState},
    Win32::{
        Foundation::{LPARAM, WPARAM},
        Media::Audio::{eCapture, eMultimedia, eRender, EDataFlow, Endpoints::IAudioEndpointVolume, IMMDeviceEnumerator, MMDeviceEnumerator},
        System::{
            Com::{CoCreateInstance, CoInitializeEx, CLSCTX_ALL, COINIT_MULTITHREADED},
            Power::{SetThreadExecutionState, ES_CONTINUOUS, ES_DISPLAY_REQUIRED, ES_SYSTEM_REQUIRED},
            Registry::{RegGetValueW, RegSetKeyValueW, HKEY_CURRENT_USER, REG_DWORD, RRF_RT_REG_DWORD},
            Shutdown::LockWorkStation,
        },
        UI::WindowsAndMessaging::{SendMessageTimeoutW, HWND_BROADCAST, SMTO_ABORTIFHUNG, WM_SETTINGCHANGE},
    },
};

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct QuickState {
    pub wifi: Option<bool>,
    pub bluetooth: Option<bool>,
    pub keep_awake: bool,
    pub dark: bool,
    pub mic_muted: Option<bool>,
    pub muted: Option<bool>,
}

// ───────────── radios (Wi-Fi y Bluetooth) ─────────────

/// Último estado leído de cada radio (Wi-Fi, Bluetooth). Windows a veces falla al listar los radios justo
/// después de otros cambios del sistema (por ejemplo, el modo oscuro); en ese caso se conserva el último
/// estado conocido en vez de dar el interruptor por «no disponible».
static LAST_RADIO: Mutex<[Option<bool>; 2]> = Mutex::new([None, None]);

/// `Ok(None)`: el equipo no tiene ese adaptador. `Err`: no se pudo consultar.
fn read_radio(kind: RadioKind) -> Result<Option<bool>, ()> {
    let radios = Radio::GetRadiosAsync().map_err(|_| ())?.join().map_err(|_| ())?;
    let mut found = None;
    for r in radios {
        if r.Kind().map_err(|_| ())? == kind {
            // Si hay varios adaptadores, basta con que uno esté encendido.
            found = Some(found.unwrap_or(false) || r.State().map_err(|_| ())? == RadioState::On);
        }
    }
    Ok(found)
}

fn radio_state(kind: RadioKind) -> Option<bool> {
    let slot = if kind == RadioKind::WiFi { 0 } else { 1 };
    for attempt in 0..3 {
        match read_radio(kind) {
            Ok(v) => {
                LAST_RADIO.lock().unwrap()[slot] = v;
                return v;
            }
            Err(()) if attempt < 2 => std::thread::sleep(Duration::from_millis(60)),
            Err(()) => {}
        }
    }
    LAST_RADIO.lock().unwrap()[slot]
}

fn set_radio(kind: RadioKind, on: bool) -> Result<(), String> {
    let access = Radio::RequestAccessAsync().and_then(|op| op.join()).map_err(|e| e.to_string())?;
    if access != RadioAccessStatus::Allowed {
        return Err("Windows no permite cambiar este interruptor desde la app.".into());
    }
    let radios = Radio::GetRadiosAsync().and_then(|op| op.join()).map_err(|e| e.to_string())?;
    let target = if on { RadioState::On } else { RadioState::Off };
    let mut any = false;
    for r in radios {
        if r.Kind().map_err(|e| e.to_string())? == kind {
            any = true;
            let res = r.SetStateAsync(target).and_then(|op| op.join()).map_err(|e| e.to_string())?;
            if res != RadioAccessStatus::Allowed {
                return Err("Windows rechazó el cambio.".into());
            }
        }
    }
    if any { Ok(()) } else { Err("Este equipo no tiene ese adaptador.".into()) }
}

// ───────────── mantener activo ─────────────

static KEEP_AWAKE: AtomicBool = AtomicBool::new(false);

/// El estado de ejecución vale por hilo, así que uno propio lo mantiene mientras el interruptor esté activo.
pub fn start_keep_awake_thread() {
    std::thread::spawn(|| {
        let mut applied = false;
        loop {
            let want = KEEP_AWAKE.load(SeqCst);
            if want != applied || want {
                unsafe {
                    SetThreadExecutionState(if want {
                        ES_CONTINUOUS | ES_SYSTEM_REQUIRED | ES_DISPLAY_REQUIRED
                    } else {
                        ES_CONTINUOUS
                    });
                }
                applied = want;
            }
            std::thread::sleep(Duration::from_secs(if want { 20 } else { 1 }));
        }
    });
}

// ───────────── modo oscuro ─────────────

const PERSONALIZE: PCWSTR = w!("Software\\Microsoft\\Windows\\CurrentVersion\\Themes\\Personalize");

fn read_dword(name: PCWSTR) -> Option<u32> {
    let mut v = 0u32;
    let mut size = 4u32;
    let rc = unsafe {
        RegGetValueW(HKEY_CURRENT_USER, PERSONALIZE, name, RRF_RT_REG_DWORD, None, Some(&mut v as *mut u32 as *mut _), Some(&mut size))
    };
    rc.is_ok().then_some(v)
}

fn dark_enabled() -> bool {
    read_dword(w!("AppsUseLightTheme")) == Some(0)
}

fn set_dark(on: bool) {
    let v: u32 = if on { 0 } else { 1 };
    for name in [w!("AppsUseLightTheme"), w!("SystemUsesLightTheme")] {
        unsafe {
            let _ = RegSetKeyValueW(HKEY_CURRENT_USER, PERSONALIZE, name, REG_DWORD.0, Some(&v as *const u32 as *const _), 4);
        }
    }
    unsafe {
        let mut _res = 0usize;
        let _ = SendMessageTimeoutW(
            HWND_BROADCAST,
            WM_SETTINGCHANGE,
            WPARAM(0),
            LPARAM(w!("ImmersiveColorSet").as_ptr() as isize),
            SMTO_ABORTIFHUNG,
            200,
            Some(&mut _res),
        );
    }
}

// ───────────── audio (sonido y micrófono) ─────────────

fn endpoint_volume(flow: EDataFlow) -> Option<IAudioEndpointVolume> {
    unsafe {
        let _ = CoInitializeEx(None, COINIT_MULTITHREADED);
        let enumerator: IMMDeviceEnumerator = CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL).ok()?;
        let dev = enumerator.GetDefaultAudioEndpoint(flow, eMultimedia).ok()?;
        dev.Activate(CLSCTX_ALL, None).ok()
    }
}

fn muted(flow: EDataFlow) -> Option<bool> {
    let ep = endpoint_volume(flow)?;
    unsafe { ep.GetMute().ok().map(|m| m.as_bool()) }
}

fn set_muted(flow: EDataFlow, on: bool) -> Result<(), String> {
    let ep = endpoint_volume(flow).ok_or("No hay un dispositivo de audio disponible.")?;
    unsafe { ep.SetMute(on, std::ptr::null()).map_err(|e| e.to_string()) }
}

// ───────────── comandos ─────────────

#[tauri::command]
pub fn quick_state() -> QuickState {
    QuickState {
        wifi: radio_state(RadioKind::WiFi),
        bluetooth: radio_state(RadioKind::Bluetooth),
        keep_awake: KEEP_AWAKE.load(SeqCst),
        dark: dark_enabled(),
        mic_muted: muted(eCapture),
        muted: muted(eRender),
    }
}

/// `kind`: wifi, bluetooth, awake, dark, mic, sound.
#[tauri::command]
pub fn quick_set(kind: String, on: bool) -> Result<(), String> {
    match kind.as_str() {
        "wifi" => set_radio(RadioKind::WiFi, on),
        "bluetooth" => set_radio(RadioKind::Bluetooth, on),
        "awake" => {
            KEEP_AWAKE.store(on, SeqCst);
            Ok(())
        }
        "dark" => {
            set_dark(on);
            Ok(())
        }
        "mic" => set_muted(eCapture, on),
        "sound" => set_muted(eRender, on),
        _ => Err("Interruptor desconocido.".into()),
    }
}

/// `kind`: screenshot (recortes de Windows) o lock (bloquear el equipo).
#[tauri::command]
pub fn quick_action(kind: String) -> Result<(), String> {
    match kind.as_str() {
        "screenshot" => std::process::Command::new("explorer.exe")
            .arg("ms-screenclip:")
            .spawn()
            .map(|_| ())
            .map_err(|e| e.to_string()),
        "lock" => unsafe { LockWorkStation().map_err(|e| e.to_string()) },
        _ => Err("Acción desconocida.".into()),
    }
}
