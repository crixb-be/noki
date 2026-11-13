//! Buscador rápido: aplicaciones del menú Inicio y un índice pequeño de archivos del usuario.

use crate::{dock, launcher::{self, AppCache, AppEntry}, notch, popups, window_utils};
use serde::Serialize;
use std::{path::{Path, PathBuf}, sync::{OnceLock, RwLock}, time::{Duration, SystemTime}};
use tauri::{window::Color, AppHandle, Manager, WebviewWindow, WebviewWindowBuilder};
use window_vibrancy::{clear_acrylic, clear_blur};

const MAX_ENTRIES: usize = 60_000;

#[derive(Clone)]
struct FileEntry { name: String, path: String, modified: i64 }

#[derive(Default)]
struct FileIndex { entries: Vec<FileEntry>, built: Option<SystemTime>, building: bool }

static INDEX: OnceLock<RwLock<FileIndex>> = OnceLock::new();
fn index() -> &'static RwLock<FileIndex> { INDEX.get_or_init(|| RwLock::new(FileIndex::default())) }

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SpotlightResult { pub kind: String, pub name: String, pub path: String, pub modified: i64 }

fn fold(value: &str) -> String {
    value.chars().map(|c| match c {
        'á' | 'à' | 'ä' | 'â' | 'Á' | 'À' | 'Ä' | 'Â' => 'a',
        'é' | 'è' | 'ë' | 'ê' | 'É' | 'È' | 'Ë' | 'Ê' => 'e',
        'í' | 'ì' | 'ï' | 'î' | 'Í' | 'Ì' | 'Ï' | 'Î' => 'i',
        'ó' | 'ò' | 'ö' | 'ô' | 'Ó' | 'Ò' | 'Ö' | 'Ô' => 'o',
        'ú' | 'ù' | 'ü' | 'û' | 'Ú' | 'Ù' | 'Ü' | 'Û' => 'u',
        'ñ' | 'Ñ' => 'n',
        other => other,
    }).collect::<String>().to_lowercase()
}

fn roots() -> Vec<PathBuf> {
    let mut out = Vec::new();
    if let Ok(profile) = std::env::var("USERPROFILE") {
        let root = PathBuf::from(profile);
        for name in ["Desktop", "Documents", "Downloads", "Pictures", "Music", "Videos"] { out.push(root.join(name)); }
        if let Ok(one) = std::env::var("OneDrive") { out.push(PathBuf::from(one)); }
    }
    out.retain(|path| path.exists());
    out
}

fn skip(path: &Path) -> bool {
    path.file_name().and_then(|n| n.to_str()).map(|name| {
        let lower = name.to_lowercase();
        name.starts_with('.') || ["node_modules", ".git", "appdata", "windows", "$recycle.bin", "system volume information"].contains(&lower.as_str())
    }).unwrap_or(false)
}

fn scan(dir: &Path, depth: u8, out: &mut Vec<FileEntry>) {
    if depth > 6 || out.len() >= MAX_ENTRIES || skip(dir) { return; }
    let Ok(entries) = std::fs::read_dir(dir) else { return };
    for item in entries.flatten() {
        if out.len() >= MAX_ENTRIES { break; }
        let path = item.path();
        if skip(&path) { continue; }
        let Ok(meta) = item.metadata() else { continue };
        let modified = meta.modified().ok().and_then(|t| t.duration_since(SystemTime::UNIX_EPOCH).ok()).map(|d| d.as_secs() as i64).unwrap_or(0);
        if meta.is_dir() { scan(&path, depth + 1, out); }
        else if let Some(name) = path.file_name().and_then(|n| n.to_str()) { out.push(FileEntry { name: name.to_string(), path: path.to_string_lossy().into_owned(), modified }); }
    }
}

fn ensure_index() {
    let should_build = {
        let mut state = index().write().unwrap();
        if state.building || state.built.is_some_and(|at| at.elapsed().unwrap_or_default() < Duration::from_secs(300)) {
            false
        } else {
            state.building = true;
            true
        }
    };
    if !should_build { return; }
    let mut entries = Vec::new();
    for root in roots() { scan(&root, 0, &mut entries); if entries.len() >= MAX_ENTRIES { break; } }
    let mut state = index().write().unwrap();
    state.entries = entries;
    state.built = Some(SystemTime::now());
    state.building = false;
}

fn app_result(app: &AppEntry, query: &str) -> Option<SpotlightResult> {
    let q = fold(query);
    let text = fold(&format!("{} {}", app.name, app.path));
    (q.is_empty() || text.contains(&q)).then(|| SpotlightResult { kind: "app".into(), name: app.name.clone(), path: app.path.clone(), modified: app.installed })
}

#[tauri::command(async)]
pub async fn spotlight_search(app: AppHandle, query: String) -> Result<Vec<SpotlightResult>, String> {
    let q = fold(&query);
    let cache = app.state::<AppCache>().inner().clone();
    tauri::async_runtime::spawn_blocking(move || {
        ensure_index();
        let apps = launcher::launcher_list(&cache);
        let mut results = apps.iter().filter_map(|entry| app_result(entry, &q)).take(5).collect::<Vec<_>>();
        let files = index().read().unwrap();
        let mut matching = files.entries.iter().filter_map(|file| {
            let name = fold(&file.name);
            let path = fold(&file.path);
            let score = if q.is_empty() { 1 } else if name.starts_with(&q) { 100 } else if name.split_whitespace().any(|word| word.starts_with(&q)) { 80 } else if name.contains(&q) { 50 } else if path.contains(&q) { 20 } else { return None };
            Some((score, file))
        }).collect::<Vec<_>>();
        matching.sort_by(|a, b| b.0.cmp(&a.0).then_with(|| b.1.modified.cmp(&a.1.modified)));
        results.extend(matching.into_iter().take(12).map(|(_, file)| SpotlightResult { kind: "file".into(), name: file.name.clone(), path: file.path.clone(), modified: file.modified }));
        results
    }).await.map_err(|e| e.to_string())
}

fn create(app: &AppHandle) -> Option<WebviewWindow> {
    if let Some(window) = app.get_webview_window("spotlight") { return Some(window); }
    let window = WebviewWindowBuilder::new(app, "spotlight", crate::window_utils::app_url("spotlight"))
        .title("").inner_size(632.0, 332.0).decorations(false).transparent(true).background_color(Color(0, 0, 0, 0)).always_on_top(true)
        .resizable(false).skip_taskbar(true).shadow(false).focused(false).visible(false)
        .on_navigation(|url| crate::window_utils::route_is_valid(url, "spotlight"))
        .on_page_load(|window, payload| crate::window_utils::repair_page_load(window, payload.event(), payload.url(), "spotlight"))
        .build().ok()?;
    let _ = window.set_background_color(Some(Color(0, 0, 0, 0)));
    let _ = clear_acrylic(&window);
    let _ = clear_blur(&window);
    popups::register_window(app, &window, "spotlight", || {});
    window_utils::mark_rounded(&window);
    let style = app.state::<crate::notch::NotchState>().settings.try_lock().map(|s| s.appearance.panel_style.clone()).unwrap_or_else(|_| "light".into());
    window_utils::sync_panel_glass(app, &style);
    window_utils::schedule_app_recovery(&window, "spotlight");
    Some(window)
}

fn position(app: &AppHandle, window: &WebviewWindow) {
    let settings = app.state::<notch::NotchState>().settings.lock().unwrap().clone();
    let Some((_, mon)) = notch::assignments(&settings, &notch::monitors(app)).into_iter().find(|(label, _)| label == "main") else { return };
    let (work, scale) = dock::work_area(app).unwrap_or((mon.rect, mon.scale));
    let width = (632.0 * scale).round() as i32;
    let height = (332.0 * scale).round() as i32;
    let top = work.top + (work.height() as f64 * 0.22) as i32;
    let w = window.clone();
    let _ = window.run_on_main_thread(move || {
        let _ = w.set_position(tauri::PhysicalPosition::new(work.left + (work.width() - width) / 2, top));
        let _ = w.set_size(tauri::PhysicalSize::new(width as u32, height as u32));
    });
}

pub fn ensure(app: &AppHandle) { let _ = create(app); }

pub fn warm_cache(app: &AppHandle) {
    let cache = app.state::<AppCache>().inner().clone();
    tauri::async_runtime::spawn_blocking(move || {
        ensure_index();
        let _ = launcher::launcher_list(&cache);
    });
}

#[tauri::command]
pub async fn toggle_spotlight(app: AppHandle) -> Result<(), String> {
    let window = create(&app).ok_or("No se pudo crear el buscador.")?;
    if !popups::toggle(&app, "spotlight")? { return Ok(()); }
    position(&app, &window);
    popups::present(&window, "spotlight");
    Ok(())
}

#[tauri::command]
pub async fn close_spotlight(app: AppHandle) {
    popups::close(&app, "spotlight");
}

#[tauri::command]
pub fn spotlight_resize(app: AppHandle, height: i32) {
    if let Some(window) = app.get_webview_window("spotlight") {
        let scale = dock::work_area(&app).map(|(_, scale)| scale).unwrap_or(1.0);
        let width = (632.0 * scale).round() as u32;
        let next_height = (height.clamp(180, 460) + 32) as f64 * scale;
        let w = window.clone();
        let _ = window.run_on_main_thread(move || { let _ = w.set_size(tauri::PhysicalSize::new(width, next_height.round() as u32)); });
    }
}
