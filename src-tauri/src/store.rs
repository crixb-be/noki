//! Almacén clave-valor persistente (`data.json` en la carpeta de datos de la app) para tareas,
//! notas, historial de agua, estado del Pomodoro y archivos del estante.
//! Vive en Rust y no en localStorage para que sea fácil de respaldar y sobreviva a limpiezas del WebView.

use serde_json::{Map, Value};
use std::{fs, path::PathBuf, sync::Mutex};
use tauri::{AppHandle, Manager};

pub struct Store {
    path: Option<PathBuf>,
    data: Mutex<Map<String, Value>>,
}

impl Store {
    pub fn load(app: &AppHandle) -> Self {
        let path = app.path().app_config_dir().ok().map(|d| d.join("data.json"));
        let data = path
            .as_ref()
            .and_then(|p| fs::read_to_string(p).ok())
            .and_then(|t| serde_json::from_str::<Map<String, Value>>(t.trim_start_matches('\u{feff}')).ok())
            .unwrap_or_default();
        Self { path, data: Mutex::new(data) }
    }

    pub fn all(&self) -> Map<String, Value> {
        self.data.lock().unwrap().clone()
    }

    pub fn set(&self, key: String, value: Value) -> Result<(), String> {
        let mut data = self.data.lock().unwrap();
        data.insert(key, value);
        self.flush(&data)
    }

    /// Sustituye todos los datos (restaurar una copia de seguridad).
    pub fn replace(&self, new: Map<String, Value>) -> Result<(), String> {
        let mut data = self.data.lock().unwrap();
        *data = new;
        self.flush(&data)
    }

    fn flush(&self, data: &Map<String, Value>) -> Result<(), String> {
        let Some(p) = &self.path else { return Err("sin carpeta de datos".into()) };
        if let Some(dir) = p.parent() {
            fs::create_dir_all(dir).map_err(|e| e.to_string())?;
        }
        let json = serde_json::to_string(data).map_err(|e| e.to_string())?;
        let tmp = p.with_extension("json.tmp");
        fs::write(&tmp, json).map_err(|e| e.to_string())?;
        fs::rename(&tmp, p).map_err(|e| e.to_string())
    }
}
