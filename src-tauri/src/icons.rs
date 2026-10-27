//! Iconos reales de los programas (los mismos que muestra el Explorador) como imagen PNG en base64.

use base64::{engine::general_purpose::STANDARD, Engine};
use std::{collections::HashMap, sync::Mutex};
use windows::{
    core::HSTRING,
    Win32::{
        Foundation::SIZE,
        Graphics::Gdi::{
            DeleteObject, GetDC, GetDIBits, ReleaseDC, BITMAPINFO, BITMAPINFOHEADER, BI_RGB, DIB_RGB_COLORS, HGDIOBJ,
        },
        System::Com::{CoInitializeEx, COINIT_APARTMENTTHREADED},
        UI::Shell::{IShellItemImageFactory, SHCreateItemFromParsingName, SIIGBF_BIGGERSIZEOK, SIIGBF_ICONONLY},
    },
};

const SIZE_PX: i32 = 64;

static CACHE: Mutex<Option<HashMap<String, String>>> = Mutex::new(None);

fn render(path: &str) -> Result<String, String> {
    unsafe {
        let _ = CoInitializeEx(None, COINIT_APARTMENTTHREADED);
        let normalized = path.replace('/', "\\");
        let factory: IShellItemImageFactory =
            SHCreateItemFromParsingName(&HSTRING::from(normalized.as_str()), None).map_err(|e| e.to_string())?;
        let hbm = factory
            .GetImage(SIZE { cx: SIZE_PX, cy: SIZE_PX }, SIIGBF_ICONONLY | SIIGBF_BIGGERSIZEOK)
            .map_err(|e| e.to_string())?;

        let mut bmi = BITMAPINFO::default();
        bmi.bmiHeader = BITMAPINFOHEADER {
            biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
            biWidth: SIZE_PX,
            biHeight: -SIZE_PX, // de arriba abajo
            biPlanes: 1,
            biBitCount: 32,
            biCompression: BI_RGB.0,
            ..Default::default()
        };
        let mut px = vec![0u8; (SIZE_PX * SIZE_PX * 4) as usize];
        let hdc = GetDC(None);
        let lines = GetDIBits(hdc, hbm, 0, SIZE_PX as u32, Some(px.as_mut_ptr() as *mut _), &mut bmi, DIB_RGB_COLORS);
        ReleaseDC(None, hdc);
        let _ = DeleteObject(HGDIOBJ(hbm.0));
        if lines == 0 {
            return Err("no se pudo leer el icono".into());
        }

        // BGRA con alfa premultiplicado → RGBA normal.
        for p in px.chunks_exact_mut(4) {
            let (b, g, r, a) = (p[0], p[1], p[2], p[3]);
            let un = |c: u8| if a == 0 || a == 255 { c } else { ((c as u32 * 255 + a as u32 / 2) / a as u32).min(255) as u8 };
            p[0] = un(r);
            p[1] = un(g);
            p[2] = un(b);
        }

        let mut png_bytes = Vec::new();
        {
            let mut enc = png::Encoder::new(&mut png_bytes, SIZE_PX as u32, SIZE_PX as u32);
            enc.set_color(png::ColorType::Rgba);
            enc.set_depth(png::BitDepth::Eight);
            let mut w = enc.write_header().map_err(|e| e.to_string())?;
            w.write_image_data(&px).map_err(|e| e.to_string())?;
        }
        Ok(format!("data:image/png;base64,{}", STANDARD.encode(png_bytes)))
    }
}

/// Icono del programa o acceso directo `path` (o `shell:AppsFolder\…` de las apps de la Tienda).
#[tauri::command]
pub async fn app_icon(path: String) -> Result<String, String> {
    if let Some(hit) = CACHE.lock().unwrap().as_ref().and_then(|m| m.get(&path)) {
        return Ok(hit.clone());
    }
    let key = path.clone();
    let url = tauri::async_runtime::spawn_blocking(move || render(&path)).await.map_err(|e| e.to_string())??;
    CACHE.lock().unwrap().get_or_insert_with(HashMap::new).insert(key, url.clone());
    Ok(url)
}
