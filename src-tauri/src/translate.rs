//! Traductor sin claves: texto con el servicio web gratuito de Google Translate y fotos con el OCR que ya
//! trae Windows (se ejecuta en el equipo, sin enviar la imagen a ningún sitio).

use base64::{engine::general_purpose::STANDARD, Engine};
use serde::Serialize;
use std::{sync::OnceLock, time::Duration};
use windows::{
    Graphics::Imaging::{BitmapDecoder, BitmapPixelFormat, SoftwareBitmap},
    Media::Ocr::OcrEngine,
    Storage::Streams::{DataWriter, InMemoryRandomAccessStream},
    Win32::System::Com::{CoInitializeEx, COINIT_MULTITHREADED},
};

#[derive(Serialize)]
pub struct Translation {
    pub text: String,
    /// Idioma detectado del original (código de Google, p. ej. "en").
    pub detected: String,
}

fn client() -> &'static reqwest::Client {
    static CLIENT: OnceLock<reqwest::Client> = OnceLock::new();
    CLIENT.get_or_init(|| {
        reqwest::Client::builder()
            .timeout(Duration::from_secs(10))
            .connect_timeout(Duration::from_secs(4))
            .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/124.0 Safari/537.36")
            .build()
            .expect("cliente HTTP")
    })
}

/// Junta los trozos traducidos de la respuesta de Google: `[[["traducido","original",…],…],null,"en",…]`.
fn parse_google(v: &serde_json::Value) -> Option<Translation> {
    let parts = v.get(0)?.as_array()?;
    let text: String = parts.iter().filter_map(|p| p.get(0).and_then(|t| t.as_str())).collect();
    if text.is_empty() {
        return None;
    }
    let detected = v.get(2).and_then(|d| d.as_str()).unwrap_or("auto").to_string();
    Some(Translation { text, detected })
}

async fn google(text: &str, from: &str, to: &str) -> Result<Translation, String> {
    let res = client()
        .post("https://translate.googleapis.com/translate_a/single")
        .query(&[("client", "gtx"), ("sl", from), ("tl", to), ("dt", "t"), ("dj", "0"), ("ie", "UTF-8"), ("oe", "UTF-8")])
        .form(&[("q", text)])
        .send()
        .await
        .map_err(|_| "No se pudo conectar con el traductor. Revisa tu conexión.".to_string())?;
    if !res.status().is_success() {
        return Err(format!("El traductor respondió con error {}.", res.status().as_u16()));
    }
    let v: serde_json::Value = res.json().await.map_err(|_| "Respuesta no válida del traductor.".to_string())?;
    parse_google(&v).ok_or_else(|| "El traductor no devolvió texto.".to_string())
}

/// Traduce `text` a `to` (código de idioma). `from` vacío o "auto" detecta el idioma.
#[tauri::command]
pub async fn translate_text(text: String, to: String, from: Option<String>) -> Result<Translation, String> {
    let text = text.trim().to_string();
    if text.is_empty() {
        return Err("Escribe o pega algo para traducir.".into());
    }
    let from = from.filter(|f| !f.is_empty()).unwrap_or_else(|| "auto".into());
    // Un reintento por si la primera conexión falla.
    match google(&text, &from, &to).await {
        Ok(t) => Ok(t),
        Err(_) => google(&text, &from, &to).await,
    }
}

// ───────────── OCR de Windows ─────────────

fn recognize(bytes: &[u8]) -> Result<String, String> {
    unsafe {
        let _ = CoInitializeEx(None, COINIT_MULTITHREADED);
    }
    let e = |x: windows::core::Error| x.to_string();
    let stream = InMemoryRandomAccessStream::new().map_err(e)?;
    let writer = DataWriter::CreateDataWriter(&stream).map_err(e)?;
    writer.WriteBytes(bytes).map_err(e)?;
    writer.StoreAsync().map_err(e)?.join().map_err(e)?;
    writer.DetachStream().map_err(e)?;
    stream.Seek(0).map_err(e)?;
    let decoder = BitmapDecoder::CreateAsync(&stream)
        .and_then(|op| op.join())
        .map_err(|_| "No se pudo leer la imagen (usa PNG, JPG o BMP).".to_string())?;
    let bitmap: SoftwareBitmap = decoder.GetSoftwareBitmapAsync().map_err(e)?.join().map_err(e)?;
    // El OCR de Windows pide Bgra8 (o Gray8).
    let bitmap = if bitmap.BitmapPixelFormat().map_err(e)? == BitmapPixelFormat::Bgra8 {
        bitmap
    } else {
        SoftwareBitmap::Convert(&bitmap, BitmapPixelFormat::Bgra8).map_err(e)?
    };
    let max = OcrEngine::MaxImageDimension().unwrap_or(10000) as i32;
    if bitmap.PixelWidth().map_err(e)? > max || bitmap.PixelHeight().map_err(e)? > max {
        return Err("La imagen es demasiado grande para leerla.".into());
    }

    let read = |engine: &OcrEngine| -> String {
        engine
            .RecognizeAsync(&bitmap)
            .and_then(|op| op.join())
            .and_then(|r| r.Text())
            .map(|t| t.to_string())
            .unwrap_or_default()
    };

    let mut best = String::new();
    if let Ok(engine) = OcrEngine::TryCreateFromUserProfileLanguages() {
        best = read(&engine);
    }
    // Si con tus idiomas no sale nada, se prueba con el resto de idiomas de OCR instalados y se queda el que más texto lea.
    if best.trim().is_empty() {
        if let Ok(langs) = OcrEngine::AvailableRecognizerLanguages() {
            for lang in langs {
                if let Ok(engine) = OcrEngine::TryCreateFromLanguage(&lang) {
                    let t = read(&engine);
                    if t.trim().len() > best.trim().len() {
                        best = t;
                    }
                }
            }
        }
    }
    let text = best.trim().to_string();
    if text.is_empty() {
        Err("No se encontró texto en la imagen.".into())
    } else {
        Ok(text)
    }
}

/// Lee el texto de una imagen en base64 (pegada desde el portapapeles).
#[tauri::command]
pub async fn ocr_image(data: String) -> Result<String, String> {
    let raw = data.rsplit(',').next().unwrap_or(&data).to_string(); // admite «data:image/png;base64,…»
    let bytes = STANDARD.decode(raw.trim()).map_err(|_| "Imagen no válida.".to_string())?;
    tauri::async_runtime::spawn_blocking(move || recognize(&bytes)).await.map_err(|e| e.to_string())?
}

/// Lee el texto de una imagen guardada en disco.
#[tauri::command]
pub async fn ocr_file(path: String) -> Result<String, String> {
    let bytes = std::fs::read(&path).map_err(|_| "No se pudo abrir el archivo.".to_string())?;
    tauri::async_runtime::spawn_blocking(move || recognize(&bytes)).await.map_err(|e| e.to_string())?
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn joins_google_parts_and_detects_language() {
        let v = serde_json::json!([[["Hola ", "Hello ", null, null, 1], ["mundo", "world", null, null, 1]], null, "en"]);
        let t = parse_google(&v).unwrap();
        assert_eq!(t.text, "Hola mundo");
        assert_eq!(t.detected, "en");
        assert!(parse_google(&serde_json::json!([[], null, "en"])).is_none());
    }
}
