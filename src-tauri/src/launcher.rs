//! Lanzador de aplicaciones: lista los accesos del menú Inicio y abre el que se elija.

use serde::Serialize;
use std::{
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};
use tauri::State;
use windows::{
    core::{w, HSTRING},
    Win32::{
        Foundation::HWND,
        UI::{Shell::ShellExecuteW, WindowsAndMessaging::SW_SHOWNORMAL},
    },
};

#[derive(Serialize, Clone)]
pub struct AppEntry {
    pub name: String,
    pub path: String,
    pub installed: i64,
}

#[derive(Clone, Default)]
pub struct AppCache(pub Arc<Mutex<Option<(Instant, Vec<AppEntry>)>>>);

const EXTENSIONS: [&str; 3] = ["lnk", "exe", "url"];
const SKIP: [&str; 6] = ["uninstall", "desinstal", "readme", "léeme", "leeme", "license"];

fn walk(dir: &Path, depth: u8, out: &mut Vec<AppEntry>) {
    let Ok(entries) = std::fs::read_dir(dir) else { return };
    for e in entries.flatten() {
        let p = e.path();
        if p.is_dir() {
            if depth < 3 {
                walk(&p, depth + 1, out);
            }
        } else if p.extension().and_then(|x| x.to_str()).map(|x| x.eq_ignore_ascii_case("lnk")).unwrap_or(false) {
            let name = p.file_stem().map(|s| s.to_string_lossy().into_owned()).unwrap_or_default();
            let lower = name.to_lowercase();
            if !name.is_empty() && !SKIP.iter().any(|s| lower.contains(s)) {
                let installed = std::fs::metadata(&p).ok().and_then(|m| m.created().ok()).and_then(|d| d.duration_since(std::time::UNIX_EPOCH).ok()).map(|d| d.as_secs() as i64).unwrap_or(0);
                out.push(AppEntry { name, path: p.to_string_lossy().into_owned(), installed });
            }
        }
    }
}

/// Programas del menú Inicio (todos los usuarios y el actual). Se cachea 5 minutos.
#[tauri::command]
pub async fn launcher_apps(cache: State<'_, AppCache>) -> Result<Vec<AppEntry>, String> {
    let cache = cache.inner().clone();
    Ok(tauri::async_runtime::spawn_blocking(move || launcher_list(&cache)).await.map_err(|e| e.to_string())?)
}

pub fn launcher_list(cache: &AppCache) -> Vec<AppEntry> {
    // El recorrido del menú Inicio puede tardar varios segundos. Nunca mantengas el
    // lock de la caché mientras haces I/O: otras búsquedas deben poder leer la lista.
    if let Some(cached) = cache.0.lock().unwrap().as_ref().and_then(|(at, list)| {
        (at.elapsed() < Duration::from_secs(300)).then(|| list.clone())
    }) {
        return cached;
    }
    let mut out = Vec::new();
    for var in ["ProgramData", "APPDATA"] {
        if let Ok(base) = std::env::var(var) {
            walk(&PathBuf::from(base).join(r"Microsoft\Windows\Start Menu\Programs"), 0, &mut out);
        }
    }
    // Los accesos .lnk van primero para que, ante nombres repetidos, gane el acceso clásico.
    out.extend(store_apps());
    out.sort_by_key(|a| a.name.to_lowercase());
    out.dedup_by(|a, b| a.name.eq_ignore_ascii_case(&b.name));
    let mut guard = cache.0.lock().unwrap();
    if let Some((at, list)) = guard.as_ref() {
        if at.elapsed() < Duration::from_secs(300) {
            return list.clone();
        }
    }
    *guard = Some((Instant::now(), out.clone()));
    out
}

const SHELL_PREFIX: &str = r"shell:AppsFolder\";

/// Apps de la Tienda y demás programas que Windows muestra en Inicio pero que no son un .lnk (Calculadora, Configuración…).
fn store_apps() -> Vec<AppEntry> {
    use std::os::windows::process::CommandExt;
    #[derive(serde::Deserialize)]
    struct Raw {
        #[serde(rename = "Name")]
        name: String,
        #[serde(rename = "AppID")]
        app_id: String,
    }
    let out = std::process::Command::new("powershell")
        .args([
            "-NoProfile",
            "-NonInteractive",
            "-Command",
            "[Console]::OutputEncoding=[Text.Encoding]::UTF8; Get-StartApps | Select-Object Name,AppID | ConvertTo-Json -Compress",
        ])
        .creation_flags(0x0800_0000) // CREATE_NO_WINDOW
        .output();
    let Ok(out) = out else { return vec![] };
    let text = String::from_utf8_lossy(&out.stdout);
    let text = text.trim().trim_start_matches('\u{feff}');
    let raws: Vec<Raw> = serde_json::from_str::<Vec<Raw>>(text)
        .or_else(|_| serde_json::from_str::<Raw>(text).map(|r| vec![r]))
        .unwrap_or_default();
    raws.into_iter()
        .filter(|r| !r.name.trim().is_empty() && !r.app_id.is_empty())
        .filter(|r| {
            let l = r.name.to_lowercase();
            !SKIP.iter().any(|s| l.contains(s))
        })
        .map(|r| AppEntry { name: r.name.trim().to_string(), path: format!("{SHELL_PREFIX}{}", r.app_id), installed: 0 })
        .collect()
}

/// Abre un acceso directo o programa. Solo se aceptan rutas existentes de tipos ejecutables conocidos.
fn launch_app_blocking(path: String) -> Result<(), String> {
    let p = Path::new(&path);
    let is_store_app = path.starts_with(SHELL_PREFIX) && path.len() > SHELL_PREFIX.len();
    let ok_ext = p
        .extension()
        .and_then(|x| x.to_str())
        .map(|x| EXTENSIONS.iter().any(|e| e.eq_ignore_ascii_case(x)))
        .unwrap_or(false);
    if !is_store_app && (!ok_ext || !p.exists()) {
        return Err("no se encontró el programa".into());
    }
    let result = unsafe { ShellExecuteW(Some(HWND::default()), w!("open"), &HSTRING::from(path.as_str()), None, None, SW_SHOWNORMAL) };
    // ShellExecute devuelve un valor > 32 si funcionó.
    if (result.0 as isize) > 32 {
        Ok(())
    } else {
        Err("Windows no pudo abrirlo".into())
    }
}

#[tauri::command]
pub async fn launch_app(path: String) -> Result<(), String> {
    tauri::async_runtime::spawn_blocking(move || launch_app_blocking(path))
        .await
        .map_err(|e| e.to_string())?
}
