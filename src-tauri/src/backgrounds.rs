//! Imágenes de fondo del notch y de la franja de sus costados. Se copian a la carpeta de datos de la app
//! (`backgrounds`) para que sigan funcionando si el original se mueve o se borra; en los ajustes solo se
//! guarda el nombre del archivo.

use base64::{engine::general_purpose::STANDARD, Engine};
use std::{fs, path::PathBuf};
use tauri::{AppHandle, Manager};

const EXTENSIONS: [&str; 6] = ["png", "jpg", "jpeg", "webp", "gif", "bmp"];
const MAX_BYTES: u64 = 10 * 1024 * 1024;

/// `#rrggbb`.
pub fn valid_color(c: &str) -> bool {
    c.len() == 7 && c.starts_with('#') && c[1..].chars().all(|x| x.is_ascii_hexdigit())
}

/// Nombre de archivo simple (sin rutas) con una extensión de imagen conocida; vacío también vale (= ninguna).
pub fn valid_name(n: &str) -> bool {
    if n.is_empty() {
        return true;
    }
    n.len() <= 80
        && n.chars().all(|c| c.is_ascii_alphanumeric() || c == '.' || c == '-' || c == '_')
        && !n.starts_with('.')
        && n.rsplit_once('.').is_some_and(|(_, e)| EXTENSIONS.contains(&e.to_ascii_lowercase().as_str()))
}

fn dir(app: &AppHandle) -> Result<PathBuf, String> {
    let d = app.path().app_config_dir().map_err(|e| e.to_string())?.join("backgrounds");
    fs::create_dir_all(&d).map_err(|e| e.to_string())?;
    Ok(d)
}

fn mime(name: &str) -> &'static str {
    match name.rsplit('.').next().unwrap_or("").to_ascii_lowercase().as_str() {
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "webp" => "image/webp",
        "gif" => "image/gif",
        _ => "image/bmp",
    }
}

/// Copia la imagen elegida a la carpeta de fondos y devuelve el nombre con el que se guardó.
#[tauri::command]
pub fn bg_import(app: AppHandle, path: String) -> Result<String, String> {
    let src = PathBuf::from(&path);
    let ext = src.extension().and_then(|e| e.to_str()).unwrap_or("").to_ascii_lowercase();
    if !EXTENSIONS.contains(&ext.as_str()) {
        return Err("Elige una imagen PNG, JPG, WEBP, GIF o BMP.".into());
    }
    let size = fs::metadata(&src).map_err(|_| "No se pudo abrir la imagen.".to_string())?.len();
    if size > MAX_BYTES {
        return Err("La imagen pesa más de 10 MB; elige una más ligera.".into());
    }
    let stem: String = src
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("fondo")
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '_' })
        .take(40)
        .collect();
    let stamp = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_millis()).unwrap_or(0);
    let name = format!("{stem}-{stamp}.{ext}");
    fs::copy(&src, dir(&app)?.join(&name)).map_err(|e| e.to_string())?;
    Ok(name)
}

/// La imagen guardada como «data URL» lista para usar en CSS; vacío si ya no existe.
#[tauri::command]
pub fn bg_data(app: AppHandle, name: String) -> Result<String, String> {
    if name.is_empty() || !valid_name(&name) {
        return Ok(String::new());
    }
    match fs::read(dir(&app)?.join(&name)) {
        Ok(bytes) => Ok(format!("data:{};base64,{}", mime(&name), STANDARD.encode(bytes))),
        Err(_) => Ok(String::new()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validates_colors_and_names() {
        assert!(valid_color("#0a1B2c"));
        assert!(!valid_color("0a1b2c"));
        assert!(!valid_color("#12345"));
        assert!(!valid_color("#12345g"));
        assert!(valid_name(""));
        assert!(valid_name("playa-1700000000000.jpg"));
        assert!(!valid_name("..\\secreto.png"));
        assert!(!valid_name("a/b.png"));
        assert!(!valid_name("virus.exe"));
        assert!(!valid_name(".oculto.png"));
    }
}
