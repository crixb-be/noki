//! Tiempo de pantalla (opcional): cada pocos segundos anota qué aplicación tiene el foco (si no estás ausente)
//! y lo acumula por día, por aplicación y por hora en el almacén de datos (`screentime`). Todo queda en este
//! equipo; la pestaña solo lo lee. Está apagado por defecto (Ajustes → Módulos) y se conservan 30 días.

use crate::{notch::NotchState, store::Store};
use serde_json::{json, Map, Value};
use std::{
    path::Path,
    sync::atomic::{AtomicBool, Ordering::SeqCst},
    time::Duration,
};
use tauri::{AppHandle, Emitter, Manager};
use windows::{
    core::PWSTR,
    Win32::{
        Foundation::CloseHandle,
        System::{
            SystemInformation::GetTickCount,
            Threading::{OpenProcess, QueryFullProcessImageNameW, PROCESS_NAME_WIN32, PROCESS_QUERY_LIMITED_INFORMATION},
        },
        UI::{
            Input::KeyboardAndMouse::{GetLastInputInfo, LASTINPUTINFO},
            WindowsAndMessaging::{GetForegroundWindow, GetWindowThreadProcessId},
        },
    },
};

const TICK: u64 = 5;
/// Sin tocar teclado ni ratón durante este tiempo no se cuenta (te has ido).
const IDLE_LIMIT: u64 = 120;
const KEEP_DAYS: usize = 30;
const FLUSH_EVERY: u64 = 30;

fn idle_secs() -> u64 {
    let mut info = LASTINPUTINFO { cbSize: std::mem::size_of::<LASTINPUTINFO>() as u32, dwTime: 0 };
    unsafe {
        if GetLastInputInfo(&mut info).as_bool() {
            (GetTickCount().wrapping_sub(info.dwTime) / 1000) as u64
        } else {
            0
        }
    }
}

/// Nombre legible de las aplicaciones más comunes; el resto se muestra con el nombre del ejecutable.
fn pretty(stem: &str) -> String {
    let known = match stem.to_lowercase().as_str() {
        "chrome" => "Chrome",
        "msedge" => "Edge",
        "firefox" => "Firefox",
        "brave" => "Brave",
        "opera" => "Opera",
        "code" => "VS Code",
        "windowsterminal" => "Terminal",
        "explorer" => "Explorador",
        "winword" => "Word",
        "excel" => "Excel",
        "powerpnt" => "PowerPoint",
        "onenote" => "OneNote",
        "outlook" => "Outlook",
        "teams" | "ms-teams" => "Teams",
        "whatsapp" | "whatsapp.root" => "WhatsApp",
        "applicationframehost" => "Aplicación de Windows",
        "snippingtool" => "Recortes",
        "notepad" => "Bloc de notas",
        "claude" => "Claude",
        "vlc" => "VLC",
        _ => "",
    };
    if !known.is_empty() {
        return known.to_string();
    }
    let mut c = stem.chars();
    match c.next() {
        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
        None => "Desconocida".into(),
    }
}

/// Aplicación con el foco, salvo Noki y la pantalla de bloqueo.
fn foreground_app() -> Option<String> {
    unsafe {
        let hwnd = GetForegroundWindow();
        if hwnd.0.is_null() {
            return None;
        }
        let mut pid = 0u32;
        GetWindowThreadProcessId(hwnd, Some(&mut pid));
        if pid == 0 {
            return None;
        }
        let proc = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid).ok()?;
        let mut buf = [0u16; 520];
        let mut len = buf.len() as u32;
        let res = QueryFullProcessImageNameW(proc, PROCESS_NAME_WIN32, PWSTR(buf.as_mut_ptr()), &mut len);
        let _ = CloseHandle(proc);
        res.ok()?;
        let path = String::from_utf16_lossy(&buf[..len as usize]);
        let stem = Path::new(&path).file_stem()?.to_string_lossy().into_owned();
        match stem.to_lowercase().as_str() {
            "noki" | "lockapp" | "logonui" | "searchhost" | "startmenuexperiencehost" | "shellexperiencehost" => None,
            _ => Some(pretty(&stem)),
        }
    }
}

fn add(map: &mut Map<String, Value>, key: &str, secs: u64) {
    let cur = map.get(key).and_then(Value::as_u64).unwrap_or(0);
    map.insert(key.to_string(), json!(cur + secs));
}

/// Suma `secs` al día actual dentro del documento `{ days: { "AAAA-MM-DD": { total, apps, hours } } }`.
fn record(doc: &mut Value, app: &str, secs: u64) {
    let now = chrono::Local::now();
    let day = now.format("%Y-%m-%d").to_string();
    let hour = chrono::Timelike::hour(&now) as usize;
    if !doc.is_object() {
        *doc = json!({});
    }
    let days = doc.as_object_mut().unwrap().entry("days").or_insert_with(|| json!({}));
    let Some(days) = days.as_object_mut() else { return };
    let today = days.entry(day).or_insert_with(|| json!({ "total": 0, "apps": {}, "hours": vec![0u64; 24] }));
    let Some(today) = today.as_object_mut() else { return };
    add(today, "total", secs);
    if let Some(apps) = today.entry("apps").or_insert_with(|| json!({})).as_object_mut() {
        add(apps, app, secs);
    }
    if let Some(hours) = today.entry("hours").or_insert_with(|| json!(vec![0u64; 24])).as_array_mut() {
        if hours.len() == 24 {
            let cur = hours[hour].as_u64().unwrap_or(0);
            hours[hour] = json!(cur + secs);
        }
    }
    // Solo los últimos días.
    if days.len() > KEEP_DAYS {
        let mut keys: Vec<String> = days.keys().cloned().collect();
        keys.sort();
        for k in keys.iter().take(days.len() - KEEP_DAYS) {
            days.remove(k);
        }
    }
}

fn flush(app: &AppHandle, doc: &Value) {
    let store = app.state::<Store>();
    if store.set("screentime".into(), doc.clone()).is_ok() {
        let _ = app.emit("store-changed", json!({ "key": "screentime", "value": doc, "source": "rust" }));
    }
}

fn enabled(app: &AppHandle) -> bool {
    app.state::<NotchState>().settings.lock().unwrap().modules.screentime
}

/// El frontend pidió borrar el historial: el hilo descarta lo que llevaba acumulado en memoria.
static CLEARED: AtomicBool = AtomicBool::new(false);

pub fn start(app: AppHandle) {
    std::thread::spawn(move || {
        // El almacén ya está cargado cuando arranca este hilo.
        let mut doc = app.state::<Store>().all().remove("screentime").unwrap_or_else(|| json!({}));
        let mut dirty = false;
        let mut since_flush = 0u64;
        loop {
            std::thread::sleep(Duration::from_secs(TICK));
            since_flush += TICK;
            if CLEARED.swap(false, SeqCst) {
                doc = json!({ "days": {} });
                dirty = false;
            }
            if enabled(&app) && idle_secs() < IDLE_LIMIT {
                if let Some(name) = foreground_app() {
                    record(&mut doc, &name, TICK);
                    dirty = true;
                }
            }
            if dirty && since_flush >= FLUSH_EVERY {
                flush(&app, &doc);
                dirty = false;
                since_flush = 0;
            }
        }
    });
}

/// Borra todo el historial.
#[tauri::command]
pub fn screentime_clear(app: AppHandle) {
    CLEARED.store(true, SeqCst);
    let empty = json!({ "days": {} });
    let _ = app.state::<Store>().set("screentime".into(), empty.clone());
    let _ = app.emit("store-changed", json!({ "key": "screentime", "value": empty, "source": "rust" }));
}
