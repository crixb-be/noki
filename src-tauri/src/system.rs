//! Señales del sistema para los avisos del notch: volumen, brillo, cambio de dispositivo de audio,
//! Bloq Mayús e idioma del teclado. Todo se emite como eventos; el frontend decide qué mostrar.

use serde::Serialize;
use std::{
    ffi::c_void,
    sync::{mpsc, Mutex},
    time::{Duration, Instant},
};
use tauri::{AppHandle, Emitter};
use windows::{
    core::{w, PWSTR},
    Win32::{
        Devices::FunctionDiscovery::PKEY_Device_FriendlyName,
        Foundation::{CloseHandle, GENERIC_READ, GENERIC_WRITE, HANDLE},
        Globalization::LCIDToLocaleName,
        Media::Audio::{
            eMultimedia, eRender, Endpoints::IAudioEndpointVolume, IMMDevice, IMMDeviceEnumerator,
            MMDeviceEnumerator,
        },
        Storage::FileSystem::{CreateFileW, FILE_ATTRIBUTE_NORMAL, FILE_SHARE_READ, FILE_SHARE_WRITE, OPEN_EXISTING},
        System::{
            Com::StructuredStorage::PropVariantToStringAlloc,
            Com::{CoCreateInstance, CoInitializeEx, CoTaskMemFree, CLSCTX_ALL, COINIT_MULTITHREADED, STGM_READ},
            IO::DeviceIoControl,
            Power::{GetSystemPowerStatus, SYSTEM_POWER_STATUS},
        },
        UI::{
            Input::KeyboardAndMouse::{GetKeyState, GetKeyboardLayout, VK_CAPITAL},
            WindowsAndMessaging::{GetForegroundWindow, GetWindowThreadProcessId},
        },
    },
};

pub enum SysCmd {
    Volume(f32),
    Mute(bool),
    Brightness(u8),
}

pub struct SysHandle {
    tx: Mutex<mpsc::Sender<SysCmd>>,
}

impl SysHandle {
    pub fn new() -> (Self, mpsc::Receiver<SysCmd>) {
        let (tx, rx) = mpsc::channel();
        (Self { tx: Mutex::new(tx) }, rx)
    }
    pub fn send(&self, cmd: SysCmd) {
        let _ = self.tx.lock().unwrap().send(cmd);
    }
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
struct Hud {
    kind: &'static str,
    value: u8,
    muted: bool,
}

#[derive(Serialize, Clone)]
struct AudioDevice {
    name: String,
    /// "headphones" o "speaker", deducido del nombre del dispositivo.
    kind: &'static str,
}

pub fn start(app: AppHandle, rx: mpsc::Receiver<SysCmd>) {
    let a = app.clone();
    std::thread::spawn(move || audio_and_brightness(a, rx));
    std::thread::spawn(move || keyboard(app));
}

// ───────────────────────────── audio ─────────────────────────────

struct Endpoint {
    id: String,
    vol: IAudioEndpointVolume,
}

fn pwstr_to_string(p: PWSTR) -> String {
    let s = unsafe { p.to_string().unwrap_or_default() };
    unsafe { CoTaskMemFree(Some(p.0 as *const c_void)) };
    s
}

fn endpoint(enumerator: &IMMDeviceEnumerator) -> Option<(IMMDevice, Endpoint)> {
    unsafe {
        let dev = enumerator.GetDefaultAudioEndpoint(eRender, eMultimedia).ok()?;
        let id = pwstr_to_string(dev.GetId().ok()?);
        let vol: IAudioEndpointVolume = dev.Activate(CLSCTX_ALL, None).ok()?;
        Some((dev, Endpoint { id, vol }))
    }
}

fn friendly_name(dev: &IMMDevice) -> String {
    unsafe {
        let Ok(store) = dev.OpenPropertyStore(STGM_READ) else { return String::new() };
        let Ok(pv) = store.GetValue(&PKEY_Device_FriendlyName) else { return String::new() };
        PropVariantToStringAlloc(&pv).map(pwstr_to_string).unwrap_or_default()
    }
}

fn device_kind(name: &str) -> &'static str {
    let n = name.to_lowercase();
    const HINTS: [&str; 9] = ["head", "auricular", "airpods", "buds", "earbud", "casco", "audífonos", "audifonos", "manos libres"];
    if HINTS.iter().any(|h| n.contains(h)) {
        "headphones"
    } else {
        "speaker"
    }
}

fn read_volume(ep: &Endpoint) -> Option<(u8, bool)> {
    unsafe {
        let v = ep.vol.GetMasterVolumeLevelScalar().ok()?;
        let m = ep.vol.GetMute().ok()?.as_bool();
        Some(((v * 100.0).round().clamp(0.0, 100.0) as u8, m))
    }
}

// ───────────────────────────── brillo (panel interno) ─────────────────────────────

const IOCTL_VIDEO_QUERY_DISPLAY_BRIGHTNESS: u32 = 0x0023_0498;
const IOCTL_VIDEO_SET_DISPLAY_BRIGHTNESS: u32 = 0x0023_049C;

#[repr(C)]
#[derive(Default, Clone, Copy)]
struct DisplayBrightness {
    policy: u8,
    ac: u8,
    dc: u8,
}

struct Lcd(HANDLE);

impl Lcd {
    fn open() -> Option<Lcd> {
        unsafe {
            CreateFileW(
                w!("\\\\.\\LCD"),
                GENERIC_READ.0 | GENERIC_WRITE.0,
                FILE_SHARE_READ | FILE_SHARE_WRITE,
                None,
                OPEN_EXISTING,
                FILE_ATTRIBUTE_NORMAL,
                None,
            )
            .ok()
            .map(Lcd)
        }
    }

    fn on_ac() -> bool {
        let mut s = SYSTEM_POWER_STATUS::default();
        unsafe { GetSystemPowerStatus(&mut s).is_ok() && s.ACLineStatus == 1 }
    }

    fn get(&self) -> Option<u8> {
        let mut b = DisplayBrightness::default();
        let mut returned = 0u32;
        unsafe {
            DeviceIoControl(
                self.0,
                IOCTL_VIDEO_QUERY_DISPLAY_BRIGHTNESS,
                None,
                0,
                Some(&mut b as *mut _ as *mut c_void),
                std::mem::size_of::<DisplayBrightness>() as u32,
                Some(&mut returned),
                None,
            )
            .ok()?;
        }
        Some(if Self::on_ac() { b.ac } else { b.dc })
    }

    fn set(&self, level: u8) {
        let b = DisplayBrightness { policy: 3, ac: level, dc: level };
        let mut returned = 0u32;
        unsafe {
            let _ = DeviceIoControl(
                self.0,
                IOCTL_VIDEO_SET_DISPLAY_BRIGHTNESS,
                Some(&b as *const _ as *const c_void),
                std::mem::size_of::<DisplayBrightness>() as u32,
                None,
                0,
                Some(&mut returned),
                None,
            );
        }
    }
}

impl Drop for Lcd {
    fn drop(&mut self) {
        unsafe {
            let _ = CloseHandle(self.0);
        }
    }
}

fn audio_and_brightness(app: AppHandle, rx: mpsc::Receiver<SysCmd>) {
    unsafe {
        let _ = CoInitializeEx(None, COINIT_MULTITHREADED);
    }
    let enumerator: Option<IMMDeviceEnumerator> =
        unsafe { CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL).ok() };
    let lcd = Lcd::open();

    let mut current: Option<Endpoint> = None;
    let mut last_vol: Option<(u8, bool)> = None;
    let mut known_device = false;
    let mut last_br: Option<u8> = lcd.as_ref().and_then(|l| l.get());
    let mut last_br_check = Instant::now();

    loop {
        match rx.recv_timeout(Duration::from_millis(100)) {
            Ok(SysCmd::Volume(v)) => {
                if let Some(ep) = &current {
                    unsafe {
                        let _ = ep.vol.SetMasterVolumeLevelScalar((v / 100.0).clamp(0.0, 1.0), std::ptr::null());
                        if v > 0.0 {
                            let _ = ep.vol.SetMute(false, std::ptr::null());
                        }
                    }
                }
            }
            Ok(SysCmd::Mute(m)) => {
                if let Some(ep) = &current {
                    unsafe {
                        let _ = ep.vol.SetMute(m, std::ptr::null());
                    }
                }
            }
            Ok(SysCmd::Brightness(b)) => {
                if let Some(l) = &lcd {
                    l.set(b.min(100));
                }
            }
            Err(mpsc::RecvTimeoutError::Disconnected) => break,
            Err(mpsc::RecvTimeoutError::Timeout) => {}
        }

        // Dispositivo de salida predeterminado (auriculares conectados, cambio de altavoces…).
        if let Some(en) = &enumerator {
            if let Some((dev, ep)) = endpoint(en) {
                if current.as_ref().map(|c| c.id != ep.id).unwrap_or(true) {
                    if known_device {
                        let name = friendly_name(&dev);
                        let kind = device_kind(&name);
                        let _ = app.emit("audio-device", AudioDevice { name, kind });
                    }
                    known_device = true;
                    last_vol = read_volume(&ep); // el cambio de dispositivo no cuenta como cambio de volumen
                    current = Some(ep);
                }
            } else {
                current = None;
            }
        }

        if let Some(ep) = &current {
            let now = read_volume(ep);
            if now != last_vol {
                if let (Some((value, muted)), Some(_)) = (now, last_vol) {
                    let _ = app.emit("hud", Hud { kind: "volume", value, muted });
                }
                last_vol = now;
            }
        }

        if let Some(l) = &lcd {
            if last_br_check.elapsed() >= Duration::from_millis(250) {
                last_br_check = Instant::now();
                let now = l.get();
                if now != last_br {
                    if let (Some(value), Some(_)) = (now, last_br) {
                        let _ = app.emit("hud", Hud { kind: "brightness", value, muted: false });
                    }
                    last_br = now;
                }
            }
        }
    }
}

// ───────────────────────────── teclado ─────────────────────────────

fn keyboard_language() -> Option<String> {
    unsafe {
        let fg = GetForegroundWindow();
        if fg.0.is_null() {
            return None;
        }
        let tid = GetWindowThreadProcessId(fg, None);
        let hkl = GetKeyboardLayout(tid);
        let langid = (hkl.0 as usize & 0xFFFF) as u32;
        let mut buf = [0u16; 85];
        let n = LCIDToLocaleName(langid, Some(&mut buf), 0);
        if n <= 1 {
            return None;
        }
        Some(String::from_utf16_lossy(&buf[..(n - 1) as usize]))
    }
}

fn keyboard(app: AppHandle) {
    let mut caps: Option<bool> = None;
    let mut lang: Option<String> = None;
    let mut pending: Option<(String, u32)> = None;

    loop {
        std::thread::sleep(Duration::from_millis(100));

        let on = unsafe { GetKeyState(VK_CAPITAL.0 as i32) } & 1 != 0;
        if caps != Some(on) {
            if caps.is_some() {
                let _ = app.emit("caps-lock", on);
            }
            caps = Some(on);
        }

        // El idioma solo se anuncia si se mantiene igual en dos lecturas seguidas (evita falsos
        // cambios al pasar el foco por ventanas del sistema).
        if let Some(now) = keyboard_language() {
            if lang.as_deref() != Some(&now) {
                match &mut pending {
                    Some((p, n)) if *p == now => {
                        *n += 1;
                        if *n >= 2 {
                            if lang.is_some() {
                                let _ = app.emit("keyboard-language", &now);
                            }
                            lang = Some(now);
                            pending = None;
                        }
                    }
                    _ => pending = Some((now, 1)),
                }
            } else {
                pending = None;
            }
        }
    }
}
