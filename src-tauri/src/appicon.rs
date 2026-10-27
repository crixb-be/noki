//! Icono de Noki elegible en Ajustes → Apariencia. Cambia el de la bandeja y el de la ventana de Ajustes al instante;
//! el del programa instalado (acceso directo, menú Inicio, instalador) lo fija Windows al instalar y no cambia.

use tauri::{image::Image, AppHandle, Manager};

pub const DEFAULT: &str = "oceano";

const OCEANO: &[u8] = include_bytes!("../icons/variants/oceano.png");
const ATARDECER: &[u8] = include_bytes!("../icons/variants/atardecer.png");
const AURORA: &[u8] = include_bytes!("../icons/variants/aurora.png");
const PASTEL_ATARDECER: &[u8] = include_bytes!("../icons/variants/pastel-atardecer.png");
const PASTEL_AURORA: &[u8] = include_bytes!("../icons/variants/pastel-aurora.png");

fn bytes(id: &str) -> Option<&'static [u8]> {
    Some(match id {
        "oceano" => OCEANO,
        "atardecer" => ATARDECER,
        "aurora" => AURORA,
        "pastel-atardecer" => PASTEL_ATARDECER,
        "pastel-aurora" => PASTEL_AURORA,
        _ => return None,
    })
}

pub fn valid(id: &str) -> bool {
    bytes(id).is_some()
}

pub fn image(id: &str) -> Option<Image<'static>> {
    Image::from_bytes(bytes(id)?).ok()
}

/// Aplica el icono a la bandeja y, si está abierta, a la ventana de Ajustes.
pub fn apply(app: &AppHandle, id: &str) {
    let Some(img) = image(id) else { return };
    if let Some(tray) = app.tray_by_id("main") {
        let _ = tray.set_icon(Some(img.clone()));
    }
    if let Some(w) = app.get_webview_window("settings") {
        let _ = w.set_icon(img);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_icon_decodes() {
        for id in ["oceano", "atardecer", "aurora", "pastel-atardecer", "pastel-aurora"] {
            assert!(valid(id), "{id}");
            assert!(image(id).is_some(), "{id} no se puede leer");
        }
        assert!(!valid("otro"));
        assert!(valid(DEFAULT));
    }
}
