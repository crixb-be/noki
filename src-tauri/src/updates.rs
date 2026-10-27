//! Actualizador automático (plugin oficial de Tauri).
//!
//! La app descarga `latest.json` de la dirección configurada en Ajustes, compara versiones, descarga el
//! instalador, **verifica su firma con la clave pública incluida en la app** (tauri.conf.json) y solo entonces
//! lo ejecuta. Un servidor falso o un archivo alterado no pueden instalar nada sin la clave privada.

use crate::{notch, NotchState};
use serde::Serialize;
use std::{
    sync::Mutex,
    time::Duration,
};
use tauri::{AppHandle, Emitter, Manager, State};
use tauri_plugin_updater::{Update, UpdaterExt};

#[derive(Default)]
pub struct Pending(Mutex<Option<Update>>);

#[derive(Serialize, Clone)]
pub struct UpdateInfo {
    pub version: String,
    pub current: String,
    pub notes: Option<String>,
}

fn endpoint(app: &AppHandle) -> Result<tauri::Url, String> {
    let raw = app.state::<NotchState>().settings.lock().unwrap().updates.url.clone();
    if raw.is_empty() {
        return Err("Aún no hay una dirección de actualizaciones. Configúrala en Ajustes → Acerca de.".into());
    }
    raw.parse::<tauri::Url>().map_err(|_| "La dirección de actualizaciones no es válida.".to_string())
}

async fn check(app: &AppHandle, pending: &Pending) -> Result<Option<UpdateInfo>, String> {
    let url = endpoint(app)?;
    let updater = app
        .updater_builder()
        .endpoints(vec![url])
        .map_err(|e| e.to_string())?
        .timeout(Duration::from_secs(20))
        .build()
        .map_err(|e| e.to_string())?;
    let found = updater.check().await.map_err(|e| match e {
        tauri_plugin_updater::Error::Reqwest(_) => "No se pudo contactar con el servidor de actualizaciones.".to_string(),
        other => format!("No se pudo comprobar: {other}"),
    })?;
    let info = found.as_ref().map(|u| UpdateInfo { version: u.version.clone(), current: u.current_version.clone(), notes: u.body.clone() });
    *pending.0.lock().unwrap() = found;
    Ok(info)
}

/// Busca una versión nueva. `None` = ya estás al día.
#[tauri::command]
pub async fn update_check(app: AppHandle, pending: State<'_, Pending>) -> Result<Option<UpdateInfo>, String> {
    check(&app, &pending).await
}

/// Descarga la versión encontrada (con progreso), verifica la firma e instala; la app se cierra y se reabre sola.
#[tauri::command]
pub async fn update_install(app: AppHandle, pending: State<'_, Pending>) -> Result<(), String> {
    let update = pending.0.lock().unwrap().take().ok_or("No hay ninguna actualización pendiente: busca de nuevo.")?;
    let mut downloaded: u64 = 0;
    let handle = app.clone();
    let bytes = update
        .download(
            move |chunk, total| {
                downloaded += chunk as u64;
                let _ = handle.emit("update-progress", serde_json::json!({ "downloaded": downloaded, "total": total }));
            },
            || {},
        )
        .await
        .map_err(|e| format!("No se pudo descargar o la firma no es válida: {e}"))?;

    // Libera el espacio reservado en el escritorio antes de que el instalador cierre la app.
    notch::appbar_remove(&app.state::<NotchState>());
    update.install(bytes).map_err(|e| format!("No se pudo instalar: {e}"))?;
    Ok(())
}

/// Comprobación automática: a los 45 s de arrancar y luego cada 6 horas; avisa una vez por versión.
pub fn start_auto(app: AppHandle) {
    tauri::async_runtime::spawn(async move {
        tokio::time::sleep(Duration::from_secs(45)).await;
        let mut notified = String::new();
        loop {
            let (auto, has_url) = {
                let st = app.state::<NotchState>();
                let s = st.settings.lock().unwrap();
                (s.updates.auto, !s.updates.url.is_empty())
            };
            if auto && has_url {
                let pending = app.state::<Pending>();
                if let Ok(Some(info)) = check(&app, &pending).await {
                    if info.version != notified {
                        notified = info.version.clone();
                        let _ = app.emit("update-available", info);
                    }
                }
            }
            tokio::time::sleep(Duration::from_secs(6 * 3600)).await;
        }
    });
}
