//! Iconos reales de los programas (los mismos que muestra el Explorador) como imagen PNG en base64.
//!
//! Windows tiene varias rutas para resolver iconos. Los accesos del menú Inicio suelen ser `.lnk`,
//! las apps empaquetadas viven bajo `shell:AppsFolder` y algunas ventanas solo exponen un icono a
//! través de la extensión de Shell. Se prueban todas antes de generar un icono legible de respaldo.

use base64::{engine::general_purpose::STANDARD, Engine};
use image::ImageEncoder;
use std::{collections::HashMap, sync::{OnceLock, RwLock}};
use windows::{
    core::{HSTRING, Interface, PCWSTR},
    Win32::{
        Foundation::{CloseHandle, HWND, SIZE},
        Graphics::Gdi::{DeleteObject, GetDC, GetDIBits, ReleaseDC, BITMAPINFO, BITMAPINFOHEADER, BI_RGB, DIB_RGB_COLORS, HGDIOBJ},
        Storage::EnhancedStorage::PKEY_AppUserModel_ID,
        System::{
            Com::{CoCreateInstance, CoInitializeEx, COINIT_APARTMENTTHREADED, IPersistFile, CLSCTX_INPROC_SERVER, STGM_READ},
            Com::StructuredStorage::PROPVARIANT,
            Threading::{OpenProcess, QueryFullProcessImageNameW, PROCESS_NAME_FORMAT, PROCESS_QUERY_LIMITED_INFORMATION},
            Variant::VT_LPWSTR,
        },
        UI::{
            Shell::{
                IExtractIconW, IShellItem, IShellItemImageFactory, IShellLinkW, SHCreateItemFromParsingName, SHGetFileInfoW,
                SHFILEINFOW, SHGFI_ICON, SHGFI_LARGEICON, SIIGBF_BIGGERSIZEOK, SIIGBF_ICONONLY, ShellLink,
                PropertiesSystem::{IPropertyStore, SHGetPropertyStoreForWindow},
            },
            WindowsAndMessaging::{DestroyIcon, GetIconInfo, HICON, ICONINFO},
        },
    },
};

const SIZE_PX: i32 = 256;
static CACHE: OnceLock<RwLock<HashMap<String, String>>> = OnceLock::new();

fn cache() -> &'static RwLock<HashMap<String, String>> { CACHE.get_or_init(|| RwLock::new(HashMap::new())) }

fn png_data(bytes: Vec<u8>) -> String { format!("data:image/png;base64,{}", STANDARD.encode(bytes)) }

fn bitmap_png(hbm: windows::Win32::Graphics::Gdi::HBITMAP) -> Result<String, String> {
    if hbm.0.is_null() { return Err("bitmap vacío".into()); }
    unsafe {
        let mut bmi = BITMAPINFO::default();
        bmi.bmiHeader = BITMAPINFOHEADER {
            biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
            biWidth: SIZE_PX,
            biHeight: -SIZE_PX,
            biPlanes: 1,
            biBitCount: 32,
            biCompression: BI_RGB.0,
            ..Default::default()
        };
        let mut px = vec![0u8; (SIZE_PX * SIZE_PX * 4) as usize];
        let hdc = GetDC(None);
        let lines = GetDIBits(hdc, hbm, 0, SIZE_PX as u32, Some(px.as_mut_ptr() as *mut _), &mut bmi, DIB_RGB_COLORS);
        ReleaseDC(None, hdc);
        if lines == 0 { return Err("no se pudo leer el bitmap".into()); }
        for p in px.chunks_exact_mut(4) {
            let (b, g, r, a) = (p[0], p[1], p[2], p[3]);
            let un = |c: u8| if a == 0 || a == 255 { c } else { ((c as u32 * 255 + a as u32 / 2) / a as u32).min(255) as u8 };
            p[0] = un(r); p[1] = un(g); p[2] = un(b); p[3] = a;
        }
        let mut bytes = Vec::new();
        image::codecs::png::PngEncoder::new(&mut bytes)
            .write_image(&px, SIZE_PX as u32, SIZE_PX as u32, image::ExtendedColorType::Rgba8)
            .map_err(|e| e.to_string())?;
        Ok(png_data(bytes))
    }
}

fn hicon_png(icon: HICON) -> Option<String> {
    if icon.0.is_null() { return None; }
    unsafe {
        let mut info = ICONINFO::default();
        let result = if GetIconInfo(icon, &mut info).is_ok() {
            let bitmap = if !info.hbmColor.0.is_null() { info.hbmColor } else { info.hbmMask };
            bitmap_png(bitmap).ok()
        } else { None };
        if !info.hbmColor.0.is_null() { let _ = DeleteObject(HGDIOBJ(info.hbmColor.0)); }
        if !info.hbmMask.0.is_null() { let _ = DeleteObject(HGDIOBJ(info.hbmMask.0)); }
        let _ = DestroyIcon(icon);
        result
    }
}

fn shell_item_png(path: &str) -> Option<String> {
    unsafe {
        let item: IShellItem = SHCreateItemFromParsingName(&HSTRING::from(path), None).ok()?;
        let factory: IShellItemImageFactory = item.cast().ok()?;
        let hbm = factory.GetImage(SIZE { cx: SIZE_PX, cy: SIZE_PX }, SIIGBF_ICONONLY | SIIGBF_BIGGERSIZEOK).ok()?;
        let result = bitmap_png(hbm).ok();
        let _ = DeleteObject(HGDIOBJ(hbm.0));
        result
    }
}

fn shgetfileinfo_png(path: &str) -> Option<String> {
    let wide: Vec<u16> = path.encode_utf16().chain(std::iter::once(0)).collect();
    unsafe {
        let mut info = SHFILEINFOW::default();
        let flags = SHGFI_ICON | SHGFI_LARGEICON;
        if SHGetFileInfoW(PCWSTR(wide.as_ptr()), Default::default(), Some(&mut info), std::mem::size_of::<SHFILEINFOW>() as u32, flags) == 0 { return None; }
        hicon_png(info.hIcon)
    }
}

fn shell_extension_png(path: &str) -> Option<String> {
    unsafe {
        let item: IShellItem = SHCreateItemFromParsingName(&HSTRING::from(path), None).ok()?;
        let extractor: IExtractIconW = item.cast().ok()?;
        let mut file = [0u16; 1024];
        let mut index = 0i32;
        let mut flags = 0u32;
        extractor.GetIconLocation(0, &mut file, &mut index, &mut flags).ok()?;
        let mut large = HICON::default();
        extractor.Extract(PCWSTR(file.as_ptr()), index.max(0) as u32, Some(&mut large), None, SIZE_PX as u32).ok()?;
        hicon_png(large)
    }
}

fn utf16_string(buf: &[u16]) -> Option<String> {
    let end = buf.iter().position(|c| *c == 0).unwrap_or(buf.len());
    (!buf[..end].is_empty()).then(|| String::from_utf16_lossy(&buf[..end]))
}

fn link_targets(path: &str) -> Vec<String> {
    if !path.to_ascii_lowercase().ends_with(".lnk") { return Vec::new(); }
    unsafe {
        let Ok(link) = CoCreateInstance::<_, IShellLinkW>(&ShellLink, None, CLSCTX_INPROC_SERVER) else { return Vec::new() };
        let Ok(persist) = link.cast::<IPersistFile>() else { return Vec::new() };
        let wide: Vec<u16> = path.encode_utf16().chain(std::iter::once(0)).collect();
        if persist.Load(PCWSTR(wide.as_ptr()), STGM_READ).is_err() { return Vec::new(); }
        let mut target = [0u16; 32768];
        let _ = link.GetPath(&mut target, std::ptr::null_mut(), 0);
        let mut icon = [0u16; 32768];
        let mut icon_index = 0i32;
        let _ = link.GetIconLocation(&mut icon, &mut icon_index);
        [utf16_string(&target), utf16_string(&icon)].into_iter().flatten().collect()
    }
}

fn app_user_model_id(hwnd: HWND) -> Option<String> {
    unsafe {
        let store: IPropertyStore = SHGetPropertyStoreForWindow(hwnd).ok()?;
        let value: PROPVARIANT = store.GetValue(&PKEY_AppUserModel_ID).ok()?;
        let raw = &value.Anonymous.Anonymous;
        if raw.vt != VT_LPWSTR { return None; }
        raw.Anonymous.pwszVal.to_string().ok().filter(|s| !s.trim().is_empty())
    }
}

fn process_app_user_model_id(pid: u32) -> Option<String> {
    unsafe {
        let process = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid).ok()?;
        let mut length = 32768u32;
        let mut id = vec![0u16; length as usize];
        let result = windows::Win32::Storage::Packaging::Appx::GetApplicationUserModelId(process, &mut length, Some(windows::core::PWSTR(id.as_mut_ptr())));
        let _ = CloseHandle(process);
        (result.0 == 0).then_some(())?;
        id.truncate(length as usize);
        utf16_string(&id)
    }
}

/// Devuelve el AUMID de una ventana, incluido el de apps UWP hospedadas por ApplicationFrameHost.
pub fn window_app_user_model_id(hwnd: HWND, pid: u32) -> Option<String> { app_user_model_id(hwnd).or_else(|| process_app_user_model_id(pid)) }

pub fn process_image_path(pid: u32) -> Option<String> {
    unsafe {
        let process = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid).ok()?;
        let mut length = 32768u32;
        let mut path = vec![0u16; length as usize];
        let result = QueryFullProcessImageNameW(process, PROCESS_NAME_FORMAT(0), windows::core::PWSTR(path.as_mut_ptr()), &mut length);
        let _ = CloseHandle(process);
        result.ok()?;
        path.truncate(length as usize);
        utf16_string(&path)
    }
}

fn glyph(ch: char) -> [u8; 7] {
    match ch.to_ascii_uppercase() {
        'A' => [0b01110, 0b10001, 0b10001, 0b11111, 0b10001, 0b10001, 0b10001],
        'B' => [0b11110, 0b10001, 0b10001, 0b11110, 0b10001, 0b10001, 0b11110],
        'C' => [0b01111, 0b10000, 0b10000, 0b10000, 0b10000, 0b10000, 0b01111],
        'D' => [0b11110, 0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b11110],
        'E' => [0b11111, 0b10000, 0b10000, 0b11110, 0b10000, 0b10000, 0b11111],
        'F' => [0b11111, 0b10000, 0b10000, 0b11110, 0b10000, 0b10000, 0b10000],
        'G' => [0b01111, 0b10000, 0b10000, 0b10111, 0b10001, 0b10001, 0b01111],
        'H' => [0b10001, 0b10001, 0b10001, 0b11111, 0b10001, 0b10001, 0b10001],
        'I' => [0b11111, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100, 0b11111],
        'J' => [0b00111, 0b00010, 0b00010, 0b00010, 0b00010, 0b10010, 0b01100],
        'K' => [0b10001, 0b10010, 0b10100, 0b11000, 0b10100, 0b10010, 0b10001],
        'L' => [0b10000, 0b10000, 0b10000, 0b10000, 0b10000, 0b10000, 0b11111],
        'M' => [0b10001, 0b11011, 0b10101, 0b10101, 0b10001, 0b10001, 0b10001],
        'N' => [0b10001, 0b11001, 0b10101, 0b10011, 0b10001, 0b10001, 0b10001],
        'O' => [0b01110, 0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b01110],
        'P' => [0b11110, 0b10001, 0b10001, 0b11110, 0b10000, 0b10000, 0b10000],
        'Q' => [0b01110, 0b10001, 0b10001, 0b10001, 0b10101, 0b10010, 0b01101],
        'R' => [0b11110, 0b10001, 0b10001, 0b11110, 0b10100, 0b10010, 0b10001],
        'S' => [0b01111, 0b10000, 0b10000, 0b01110, 0b00001, 0b00001, 0b11110],
        'T' => [0b11111, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100],
        'U' => [0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b01110],
        'V' => [0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b01010, 0b00100],
        'W' => [0b10001, 0b10001, 0b10001, 0b10101, 0b10101, 0b11011, 0b10001],
        'X' => [0b10001, 0b10001, 0b01010, 0b00100, 0b01010, 0b10001, 0b10001],
        'Y' => [0b10001, 0b10001, 0b01010, 0b00100, 0b00100, 0b00100, 0b00100],
        'Z' => [0b11111, 0b00001, 0b00010, 0b00100, 0b01000, 0b10000, 0b11111],
        _ => [0b01110, 0b10001, 0b00001, 0b00010, 0b00100, 0b00000, 0b00100],
    }
}

fn fallback_png(name: &str) -> String {
    let mut hash = 2166136261u32;
    for byte in name.bytes() { hash = (hash ^ byte as u32).wrapping_mul(16777619); }
    let color = [70 + ((hash >> 16) & 0x7f) as u8, 70 + ((hash >> 8) & 0x7f) as u8, 70 + (hash & 0x7f) as u8, 255];
    let mut image = image::RgbaImage::new(SIZE_PX as u32, SIZE_PX as u32);
    for y in 0..SIZE_PX {
        for x in 0..SIZE_PX {
            let dx = if x < 13 { x - 13 } else if x >= 51 { x - 50 } else { 0 };
            let dy = if y < 13 { y - 13 } else if y >= 51 { y - 50 } else { 0 };
            let inside = (x >= 9 && x < 55 && y >= 9 && y < 55) || dx * dx + dy * dy <= 13 * 13;
            image.put_pixel(x as u32, y as u32, if inside { image::Rgba(color) } else { image::Rgba([0, 0, 0, 0]) });
        }
    }
    let rows = glyph(name.chars().find(|c| c.is_alphanumeric()).unwrap_or('?'));
    for (row, bits) in rows.into_iter().enumerate() {
        for col in 0..5 {
            if bits & (1 << (4 - col)) != 0 {
                for dy in 0..3 { for dx in 0..3 { image.put_pixel((26 + col * 3 + dx) as u32, (22 + row as i32 * 3 + dy) as u32, image::Rgba([255, 255, 255, 245])); } }
            }
        }
    }
    let mut bytes = Vec::new();
    let _ = image::codecs::png::PngEncoder::new(&mut bytes).write_image(image.as_raw(), SIZE_PX as u32, SIZE_PX as u32, image::ExtendedColorType::Rgba8);
    png_data(bytes)
}

fn render(path: &str) -> String {
    unsafe { let _ = CoInitializeEx(None, COINIT_APARTMENTTHREADED); }
    let normalized = path.replace('/', "\\");
    let mut candidates = vec![normalized.clone()];
    candidates.extend(link_targets(&normalized));
    for candidate in candidates {
        if let Some(url) = shell_item_png(&candidate).or_else(|| shgetfileinfo_png(&candidate)).or_else(|| shell_extension_png(&candidate)) { return url; }
    }
    fallback_png(normalized.rsplit(['\\', '/']).next().unwrap_or(&normalized))
}

/// Precalienta iconos sin bloquear el hilo de ventanas. La caché se comparte
/// con las peticiones posteriores de Launchpad, Spotlight y Dock.
pub fn warm_paths(paths: Vec<String>) {
    for path in paths {
        if cache().read().unwrap().contains_key(&path) { continue; }
        let url = render(&path);
        cache().write().unwrap().insert(path, url);
    }
}

/// Icono del programa, acceso directo o `shell:AppsFolder\\…` de una app de la Tienda.
#[tauri::command]
pub async fn app_icon(path: String) -> Result<String, String> {
    if let Some(hit) = cache().read().unwrap().get(&path) { return Ok(hit.clone()); }
    let key = path.clone();
    let url = tauri::async_runtime::spawn_blocking(move || render(&path)).await.map_err(|e| e.to_string())?;
    cache().write().unwrap().insert(key, url.clone());
    Ok(url)
}
