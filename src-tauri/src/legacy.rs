//! Migración desde el nombre anterior de la app («Mi Notch», carpeta `com.minotch.app`): la primera vez que
//! arranca Noki sin datos propios, copia los ajustes, los datos y los fondos de la carpeta antigua.

use std::{fs, path::Path};
use tauri::{AppHandle, Manager};

const OLD_DIR: &str = "com.minotch.app";

fn copy_dir(from: &Path, to: &Path) -> std::io::Result<()> {
    fs::create_dir_all(to)?;
    for entry in fs::read_dir(from)? {
        let entry = entry?;
        let target = to.join(entry.file_name());
        if entry.path().is_dir() {
            copy_dir(&entry.path(), &target)?;
        } else {
            fs::copy(entry.path(), target)?;
        }
    }
    Ok(())
}

/// Copia (no mueve) los datos antiguos si Noki aún no tiene los suyos. Los originales se conservan.
pub fn migrate_data(app: &AppHandle) {
    let Ok(new_dir) = app.path().app_config_dir() else { return };
    let Some(parent) = new_dir.parent() else { return };
    let old = parent.join(OLD_DIR);
    if !old.is_dir() || new_dir.join("settings.json").exists() || new_dir.join("data.json").exists() {
        return;
    }
    let _ = fs::create_dir_all(&new_dir);
    for file in ["settings.json", "data.json"] {
        if old.join(file).is_file() {
            let _ = fs::copy(old.join(file), new_dir.join(file));
        }
    }
    if old.join("backgrounds").is_dir() {
        let _ = copy_dir(&old.join("backgrounds"), &new_dir.join("backgrounds"));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn copies_nested_folders() {
        let base = std::env::temp_dir().join(format!("noki-legacy-test-{}", std::process::id()));
        let _ = fs::remove_dir_all(&base);
        fs::create_dir_all(base.join("a/b")).unwrap();
        fs::write(base.join("a/b/x.txt"), "hola").unwrap();
        copy_dir(&base.join("a"), &base.join("copia")).unwrap();
        assert_eq!(fs::read_to_string(base.join("copia/b/x.txt")).unwrap(), "hola");
        let _ = fs::remove_dir_all(&base);
    }
}
