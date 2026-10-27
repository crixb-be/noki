//! Historial del portapapeles (solo texto, solo en memoria).
//!
//! Un hilo comprueba el número de secuencia del portapapeles; si cambió y la función está activada,
//! lee el texto y lo añade al historial. Se ignoran los contenidos que las apps marcan como sensibles
//! (gestores de contraseñas usan el formato «ExcludeClipboardContentFromMonitorProcessing»).

use crate::NotchState;
use serde::Serialize;
use std::{
    sync::{
        atomic::{AtomicU32, AtomicU64, Ordering::SeqCst},
        Mutex,
    },
    time::{Duration, SystemTime, UNIX_EPOCH},
};
use tauri::{AppHandle, Emitter, Manager, State};
use windows::{
    core::w,
    Win32::{
        Foundation::{GlobalFree, HANDLE, HGLOBAL},
        System::{
            DataExchange::{
                CloseClipboard, EmptyClipboard, GetClipboardData, GetClipboardSequenceNumber, IsClipboardFormatAvailable,
                OpenClipboard, RegisterClipboardFormatW, SetClipboardData,
            },
            Memory::{GlobalAlloc, GlobalLock, GlobalSize, GlobalUnlock, GMEM_MOVEABLE},
        },
    },
};

const CF_UNICODETEXT: u32 = 13;
const MAX_CHARS: usize = 20_000;

#[derive(Serialize, Clone, Debug)]
pub struct Entry {
    pub id: u64,
    pub text: String,
    /// Milisegundos desde 1970.
    pub at: i64,
}

#[derive(Default)]
pub struct History {
    items: Mutex<Vec<Entry>>,
    /// Secuencia del portapapeles que generó la propia app (no se vuelve a registrar).
    own_seq: AtomicU32,
    /// Identificadores pequeños y secuenciales (seguros como número en JavaScript).
    next_id: AtomicU64,
}

fn now_ms() -> i64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_millis() as i64).unwrap_or(0)
}

/// Abre el portapapeles con reintentos (otra app puede tenerlo ocupado un instante).
fn open() -> bool {
    for _ in 0..8 {
        if unsafe { OpenClipboard(None).is_ok() } {
            return true;
        }
        std::thread::sleep(Duration::from_millis(15));
    }
    false
}

fn read_text() -> Option<String> {
    if !open() {
        return None;
    }
    let out = unsafe {
        let sensitive = IsClipboardFormatAvailable(RegisterClipboardFormatW(w!("ExcludeClipboardContentFromMonitorProcessing")))
            .is_ok();
        if sensitive || IsClipboardFormatAvailable(CF_UNICODETEXT).is_err() {
            None
        } else {
            GetClipboardData(CF_UNICODETEXT).ok().and_then(|h| {
                let g = HGLOBAL(h.0);
                let ptr = GlobalLock(g) as *const u16;
                if ptr.is_null() {
                    return None;
                }
                let max = GlobalSize(g) / 2;
                let mut len = 0;
                while len < max && *ptr.add(len) != 0 {
                    len += 1;
                }
                let s = String::from_utf16_lossy(std::slice::from_raw_parts(ptr, len));
                let _ = GlobalUnlock(g);
                Some(s)
            })
        }
    };
    unsafe {
        let _ = CloseClipboard();
    }
    out
}

fn write_text(text: &str) -> Result<(), String> {
    if !open() {
        return Err("el portapapeles está ocupado".into());
    }
    let result = unsafe {
        let _ = EmptyClipboard();
        let wide: Vec<u16> = text.encode_utf16().chain(std::iter::once(0)).collect();
        let bytes = wide.len() * 2;
        match GlobalAlloc(GMEM_MOVEABLE, bytes) {
            Ok(h) => {
                let ptr = GlobalLock(h) as *mut u16;
                if ptr.is_null() {
                    let _ = GlobalFree(Some(h));
                    Err("no se pudo reservar memoria".to_string())
                } else {
                    std::ptr::copy_nonoverlapping(wide.as_ptr(), ptr, wide.len());
                    let _ = GlobalUnlock(h);
                    // Si SetClipboardData funciona, el sistema es dueño de la memoria.
                    SetClipboardData(CF_UNICODETEXT, Some(HANDLE(h.0))).map(|_| ()).map_err(|e| {
                        let _ = GlobalFree(Some(h));
                        e.to_string()
                    })
                }
            }
            Err(e) => Err(e.to_string()),
        }
    };
    unsafe {
        let _ = CloseClipboard();
    }
    result
}

pub fn start(app: AppHandle) {
    std::thread::spawn(move || {
        let mut seen = unsafe { GetClipboardSequenceNumber() };
        let mut was_enabled = false;
        loop {
            std::thread::sleep(Duration::from_millis(350));
            let (enabled, max) = {
                let st = app.state::<NotchState>();
                let s = st.settings.lock().unwrap();
                (s.clipboard.enabled, s.clipboard.max as usize)
            };
            let hist = app.state::<History>();

            if !enabled {
                // Al desactivar, el historial se borra de la memoria.
                if was_enabled {
                    hist.items.lock().unwrap().clear();
                    let _ = app.emit("clip-changed", Vec::<Entry>::new());
                }
                was_enabled = false;
                seen = unsafe { GetClipboardSequenceNumber() };
                continue;
            }
            was_enabled = true;

            let seq = unsafe { GetClipboardSequenceNumber() };
            if seq == seen {
                continue;
            }
            seen = seq;
            if seq == hist.own_seq.load(SeqCst) {
                continue;
            }
            let Some(mut text) = read_text() else { continue };
            if text.trim().is_empty() {
                continue;
            }
            if text.chars().count() > MAX_CHARS {
                text = text.chars().take(MAX_CHARS).collect();
            }
            let snapshot = {
                let mut items = hist.items.lock().unwrap();
                items.retain(|e| e.text != text); // un texto repetido sube al principio
                items.insert(0, Entry { id: hist.next_id.fetch_add(1, SeqCst), text, at: now_ms() });
                items.truncate(max);
                items.clone()
            };
            let _ = app.emit("clip-changed", snapshot);
        }
    });
}

#[tauri::command]
pub fn clip_list(h: State<History>) -> Vec<Entry> {
    h.items.lock().unwrap().clone()
}

/// Vuelve a poner un texto en el portapapeles.
#[tauri::command]
pub fn clip_copy(app: AppHandle, h: State<History>, text: String) -> Result<(), String> {
    write_text(&text)?;
    h.own_seq.store(unsafe { GetClipboardSequenceNumber() }, SeqCst);
    let snapshot = {
        let mut items = h.items.lock().unwrap();
        if items.iter().any(|e| e.text == text) {
            items.retain(|e| e.text != text);
            items.insert(0, Entry { id: h.next_id.fetch_add(1, SeqCst), text, at: now_ms() });
        }
        items.clone()
    };
    let _ = app.emit("clip-changed", snapshot);
    Ok(())
}

#[tauri::command]
pub fn clip_remove(app: AppHandle, h: State<History>, id: u64) {
    let snapshot = {
        let mut items = h.items.lock().unwrap();
        items.retain(|e| e.id != id);
        items.clone()
    };
    let _ = app.emit("clip-changed", snapshot);
}

#[tauri::command]
pub fn clip_clear(app: AppHandle, h: State<History>) {
    h.items.lock().unwrap().clear();
    let _ = app.emit("clip-changed", Vec::<Entry>::new());
}
