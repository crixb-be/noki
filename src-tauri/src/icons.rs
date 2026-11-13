//! Iconos reales de los programas (los mismos que muestra el Explorador) como imagen PNG en base64.
//!
//! Windows tiene varias rutas para resolver iconos. Los accesos del menú Inicio suelen ser `.lnk`,
//! las apps empaquetadas viven bajo `shell:AppsFolder` y algunas ventanas solo exponen un icono a
//! través de la extensión de Shell. Se prueban todas antes de generar un icono legible de respaldo.
//!
//! Nitidez: cada mapa de bits se lee con su tamaño real (antes se suponía siempre 256×256 y los de
//! 32/48 px salían recortados o mal leídos), se distingue el alfa premultiplicado del directo (antes
//! se «despremultiplicaba» también el alfa directo de los iconos y los bordes salían con halos y
//! dientes), se usa la lista de imágenes JUMBO (256 px) del sistema y, si la app solo trae un icono
//! pequeño, se amplía con Lanczos. El resultado se entrega a 128 px: a 125 % el Dock y el lanzador lo
//! muestran a 60-100 px físicos, así Chromium ya no reduce 4:1 de golpe (aliasing).

use base64::{engine::general_purpose::STANDARD, Engine};
use image::{imageops::FilterType, ImageEncoder, RgbaImage};
use std::{collections::HashMap, sync::{OnceLock, RwLock}};
use windows::{
    core::{HSTRING, Interface, PCWSTR},
    Win32::{
        Foundation::{CloseHandle, HWND, SIZE},
        Graphics::Gdi::{DeleteObject, GetDC, GetDIBits, GetObjectW, ReleaseDC, BITMAP, BITMAPINFO, BITMAPINFOHEADER, BI_RGB, DIB_RGB_COLORS, HBITMAP, HGDIOBJ},
        Storage::EnhancedStorage::PKEY_AppUserModel_ID,
        System::{
            Com::{CoCreateInstance, CoInitializeEx, COINIT_APARTMENTTHREADED, IPersistFile, CLSCTX_INPROC_SERVER, STGM_READ},
            Com::StructuredStorage::PROPVARIANT,
            Threading::{OpenProcess, QueryFullProcessImageNameW, PROCESS_NAME_FORMAT, PROCESS_QUERY_LIMITED_INFORMATION},
            Variant::VT_LPWSTR,
        },
        UI::{
            Controls::{IImageList, ILD_TRANSPARENT},
            Shell::{
                IExtractIconW, IShellItem, IShellItemImageFactory, IShellLinkW, SHCreateItemFromParsingName, SHGetFileInfoW,
                SHGetImageList, SHFILEINFOW, SHGFI_SYSICONINDEX, SHIL_EXTRALARGE, SHIL_JUMBO, SIIGBF_BIGGERSIZEOK, SIIGBF_ICONONLY, ShellLink,
                PropertiesSystem::{IPropertyStore, SHGetPropertyStoreForWindow},
            },
            WindowsAndMessaging::{DestroyIcon, GetIconInfo, HICON, ICONINFO},
        },
    },
};

/// Tamaño pedido a Windows (el mayor que existe) y tamaño entregado a la interfaz.
const SOURCE_PX: i32 = 256;
const OUT_PX: u32 = 128;
/// Un icono cuyo contenido visible mide menos que esto en la lista JUMBO es uno pequeño (32/48 px)
/// que Windows ha colocado en un lienzo de 256: se toma el de 48 px y se amplía con suavizado.
const SMALL_CONTENT_PX: u32 = 96;
/// Límite de la caché de iconos (PNG en base64): antes crecía sin fin.
const CACHE_MAX: usize = 400;
static CACHE: OnceLock<RwLock<HashMap<String, String>>> = OnceLock::new();

fn cache() -> &'static RwLock<HashMap<String, String>> { CACHE.get_or_init(|| RwLock::new(HashMap::new())) }

fn png_data(bytes: Vec<u8>) -> String { format!("data:image/png;base64,{}", STANDARD.encode(bytes)) }

/// Lee un mapa de bits con su tamaño real como BGRA de 32 bits, de arriba abajo.
fn read_bitmap(hbm: HBITMAP) -> Option<(u32, u32, Vec<u8>)> {
    if hbm.0.is_null() { return None; }
    unsafe {
        let mut bm = BITMAP::default();
        if GetObjectW(HGDIOBJ(hbm.0), std::mem::size_of::<BITMAP>() as i32, Some(&mut bm as *mut _ as *mut _)) == 0 { return None; }
        let (w, h) = (bm.bmWidth, bm.bmHeight.abs());
        if w <= 0 || h <= 0 || w > 1024 || h > 1024 { return None; }
        let mut bmi = BITMAPINFO::default();
        bmi.bmiHeader = BITMAPINFOHEADER {
            biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
            biWidth: w,
            biHeight: -h,
            biPlanes: 1,
            biBitCount: 32,
            biCompression: BI_RGB.0,
            ..Default::default()
        };
        let mut px = vec![0u8; (w * h * 4) as usize];
        let hdc = GetDC(None);
        let lines = GetDIBits(hdc, hbm, 0, h as u32, Some(px.as_mut_ptr() as *mut _), &mut bmi, DIB_RGB_COLORS);
        ReleaseDC(None, hdc);
        (lines > 0).then_some((w as u32, h as u32, px))
    }
}

/// BGRA con canal alfa → RGBA con alfa directo. Solo se «despremultiplica» si los datos lo están
/// (en un píxel premultiplicado ningún canal supera al alfa). `None` si no hay canal alfa.
fn straight_rgba(w: u32, h: u32, bgra: &[u8]) -> Option<RgbaImage> {
    if !bgra.chunks_exact(4).any(|p| p[3] != 0) { return None; }
    let premultiplied = bgra.chunks_exact(4).all(|p| p[0] <= p[3] && p[1] <= p[3] && p[2] <= p[3]);
    let mut out = Vec::with_capacity(bgra.len());
    for p in bgra.chunks_exact(4) {
        let (b, g, r, a) = (p[0], p[1], p[2], p[3]);
        let un = |c: u8| if !premultiplied || a == 0 || a == 255 { c } else { ((c as u32 * 255 + a as u32 / 2) / a as u32).min(255) as u8 };
        out.extend_from_slice(&[un(r), un(g), un(b), a]);
    }
    RgbaImage::from_raw(w, h, out)
}

/// Mapa de bits de la Shell (IShellItemImageFactory) → RGBA.
fn bitmap_rgba(hbm: HBITMAP) -> Option<RgbaImage> {
    let (w, h, bgra) = read_bitmap(hbm)?;
    straight_rgba(w, h, &bgra)
}

/// Icono → RGBA. Los iconos sin canal alfa (24 bits) toman la transparencia de su máscara.
fn hicon_rgba(icon: HICON) -> Option<RgbaImage> {
    if icon.0.is_null() { return None; }
    unsafe {
        let mut info = ICONINFO::default();
        let result = if GetIconInfo(icon, &mut info).is_ok() && !info.hbmColor.0.is_null() {
            read_bitmap(info.hbmColor).and_then(|(w, h, bgra)| {
                straight_rgba(w, h, &bgra).or_else(|| {
                    // Sin alfa: la máscara AND dice qué es transparente (blanco = transparente).
                    let (mw, mh, mask) = read_bitmap(info.hbmMask)?;
                    if mw != w || mh < h { return None; }
                    let mut out = Vec::with_capacity(bgra.len());
                    for (p, m) in bgra.chunks_exact(4).zip(mask.chunks_exact(4)) {
                        out.extend_from_slice(&[p[2], p[1], p[0], if m[0] > 127 { 0 } else { 255 }]);
                    }
                    RgbaImage::from_raw(w, h, out)
                })
            })
        } else { None };
        if !info.hbmColor.0.is_null() { let _ = DeleteObject(HGDIOBJ(info.hbmColor.0)); }
        if !info.hbmMask.0.is_null() { let _ = DeleteObject(HGDIOBJ(info.hbmMask.0)); }
        let _ = DestroyIcon(icon);
        result
    }
}

/// Caja del contenido visible (alfa > 16): (x, y, ancho, alto).
fn content_box(img: &RgbaImage) -> Option<(u32, u32, u32, u32)> {
    let (mut x0, mut y0, mut x1, mut y1) = (u32::MAX, u32::MAX, 0u32, 0u32);
    for (x, y, p) in img.enumerate_pixels() {
        if p[3] > 16 { x0 = x0.min(x); y0 = y0.min(y); x1 = x1.max(x); y1 = y1.max(y); }
    }
    (x0 <= x1 && y0 <= y1).then(|| (x0, y0, x1 - x0 + 1, y1 - y0 + 1))
}

/// Redimensiona con alfa premultiplicado (si no, el color de los píxeles transparentes, casi siempre
/// negro, se filtra al borde y deja un halo oscuro).
fn resize_clean(img: &RgbaImage, w: u32, h: u32) -> RgbaImage {
    let mut pre = img.clone();
    for p in pre.pixels_mut() {
        let a = p[3] as u32;
        for c in 0..3 { p[c] = ((p[c] as u32 * a + 127) / 255) as u8; }
    }
    let mut out = image::imageops::resize(&pre, w, h, FilterType::Lanczos3);
    for p in out.pixels_mut() {
        let a = p[3] as u32;
        for c in 0..3 { p[c] = if a == 0 { 0 } else { ((p[c] as u32 * 255 + a / 2) / a).min(255) as u8 }; }
    }
    out
}

/// Ajusta el icono a un lienzo cuadrado de OUT_PX. Si ocupa poco de su lienzo (un icono de 32/48 px
/// dentro de uno de 256), se recorta a su contenido antes de ampliarlo.
fn finish(img: RgbaImage) -> Option<String> {
    let (bx, by, bw, bh) = content_box(&img)?;
    let (w, h) = img.dimensions();
    let img = if bw.max(bh) * 2 <= w.max(h) {
        let side = bw.max(bh);
        let cx = (bx + bw / 2).saturating_sub(side / 2).min(w.saturating_sub(side));
        let cy = (by + bh / 2).saturating_sub(side / 2).min(h.saturating_sub(side));
        image::imageops::crop_imm(&img, cx, cy, side.min(w), side.min(h)).to_image()
    } else { img };
    let (w, h) = img.dimensions();
    let scale = OUT_PX as f64 / w.max(h) as f64;
    let (nw, nh) = (((w as f64 * scale).round() as u32).max(1), ((h as f64 * scale).round() as u32).max(1));
    let resized = if (nw, nh) == (w, h) { img } else { resize_clean(&img, nw, nh) };
    let mut canvas = RgbaImage::new(OUT_PX, OUT_PX);
    image::imageops::overlay(&mut canvas, &resized, ((OUT_PX - nw) / 2) as i64, ((OUT_PX - nh) / 2) as i64);
    let mut bytes = Vec::new();
    image::codecs::png::PngEncoder::new(&mut bytes)
        .write_image(canvas.as_raw(), OUT_PX, OUT_PX, image::ExtendedColorType::Rgba8)
        .ok()?;
    Some(png_data(bytes))
}

/// Icono de la lista de imágenes del sistema (la que usa el Explorador) en el tamaño indicado.
fn image_list_icon(path: &str, list: u32) -> Option<RgbaImage> {
    let wide: Vec<u16> = path.encode_utf16().chain(std::iter::once(0)).collect();
    unsafe {
        let mut info = SHFILEINFOW::default();
        if SHGetFileInfoW(PCWSTR(wide.as_ptr()), Default::default(), Some(&mut info), std::mem::size_of::<SHFILEINFOW>() as u32, SHGFI_SYSICONINDEX) == 0 { return None; }
        let images: IImageList = SHGetImageList(list as i32).ok()?;
        let icon = images.GetIcon(info.iIcon, ILD_TRANSPARENT.0).ok()?;
        hicon_rgba(icon)
    }
}

/// Mejor icono de un archivo: JUMBO (256) si es grande de verdad; si solo existe uno pequeño, el
/// de 48 px ampliado con suavizado (mejor que el de 256 que Windows rellena o estira).
fn file_icon_png(path: &str) -> Option<String> {
    if let Some(jumbo) = image_list_icon(path, SHIL_JUMBO) {
        if content_box(&jumbo).is_some_and(|(_, _, w, h)| w.max(h) >= SMALL_CONTENT_PX) { return finish(jumbo); }
        if let Some(large) = image_list_icon(path, SHIL_EXTRALARGE) { return finish(large); }
        return finish(jumbo);
    }
    image_list_icon(path, SHIL_EXTRALARGE).and_then(finish)
}

fn shell_item_png(path: &str) -> Option<String> {
    unsafe {
        let item: IShellItem = SHCreateItemFromParsingName(&HSTRING::from(path), None).ok()?;
        let factory: IShellItemImageFactory = item.cast().ok()?;
        let hbm = factory.GetImage(SIZE { cx: SOURCE_PX, cy: SOURCE_PX }, SIIGBF_ICONONLY | SIIGBF_BIGGERSIZEOK).ok()?;
        let result = bitmap_rgba(hbm).and_then(finish);
        let _ = DeleteObject(HGDIOBJ(hbm.0));
        result
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
        extractor.Extract(PCWSTR(file.as_ptr()), index.max(0) as u32, Some(&mut large), None, SOURCE_PX as u32).ok()?;
        hicon_rgba(large).and_then(finish)
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
    // El dibujo mide 64 px (antes iba en una esquina de un lienzo de 256 y salía diminuto).
    const FALLBACK_PX: i32 = 64;
    let mut image = image::RgbaImage::new(FALLBACK_PX as u32, FALLBACK_PX as u32);
    for y in 0..FALLBACK_PX {
        for x in 0..FALLBACK_PX {
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
    let _ = image::codecs::png::PngEncoder::new(&mut bytes).write_image(image.as_raw(), FALLBACK_PX as u32, FALLBACK_PX as u32, image::ExtendedColorType::Rgba8);
    png_data(bytes)
}

/// Convierte un nombre suelto como `explorer.exe` o `notepad.exe` en su ruta completa
/// (carpeta de Windows o System32); con eso el icono y la identificación de ventanas funcionan.
pub(crate) fn resolve_program(path: &str) -> String {
    let path = path.trim();
    if path.is_empty() || path.contains(['\\', '/']) || !path.to_ascii_lowercase().ends_with(".exe") { return path.to_string(); }
    let windir = std::env::var("WINDIR").or_else(|_| std::env::var("SystemRoot")).unwrap_or_else(|_| "C:\\Windows".into());
    for dir in [windir.clone(), format!("{windir}\\System32")] {
        let candidate = format!("{dir}\\{path}");
        if std::path::Path::new(&candidate).exists() { return candidate; }
    }
    path.to_string()
}

fn render(path: &str) -> String {
    unsafe { let _ = CoInitializeEx(None, COINIT_APARTMENTTHREADED); }
    let normalized = resolve_program(path).replace('/', "\\");
    let mut candidates = vec![normalized.clone()];
    candidates.extend(link_targets(&normalized));
    // Apps de la Tienda (`shell:AppsFolder\…`): solo la Shell sabe resolverlas. Archivos: primero la
    // lista de imágenes del sistema (tamaño real y nitidez), luego la Shell y su extensión.
    let shell_only = normalized.to_ascii_lowercase().starts_with("shell:");
    for candidate in candidates {
        let found = if shell_only { shell_item_png(&candidate) } else { file_icon_png(&candidate).or_else(|| shell_item_png(&candidate)) };
        if let Some(url) = found.or_else(|| shell_extension_png(&candidate)) { return url; }
    }
    fallback_png(normalized.rsplit(['\\', '/']).next().unwrap_or(&normalized))
}

/// Guarda un icono en la caché sin dejarla crecer sin límite.
fn remember(path: String, url: String) {
    let mut map = cache().write().unwrap();
    if map.len() >= CACHE_MAX && !map.contains_key(&path) { map.clear(); }
    map.insert(path, url);
}

/// Precalienta iconos sin bloquear el hilo de ventanas. La caché se comparte
/// con las peticiones posteriores de Launchpad, Spotlight y Dock.
pub fn warm_paths(paths: Vec<String>) {
    for path in paths {
        if cache().read().unwrap().contains_key(&path) { continue; }
        let url = render(&path);
        remember(path, url);
    }
}

/// Icono del programa, acceso directo o `shell:AppsFolder\\…` de una app de la Tienda.
#[tauri::command]
pub async fn app_icon(path: String) -> Result<String, String> {
    if let Some(hit) = cache().read().unwrap().get(&path) { return Ok(hit.clone()); }
    let key = path.clone();
    let url = tauri::async_runtime::spawn_blocking(move || render(&path)).await.map_err(|e| e.to_string())?;
    remember(key, url.clone());
    Ok(url)
}
