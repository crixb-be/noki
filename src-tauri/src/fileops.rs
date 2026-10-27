//! Acciones sobre los archivos del estante: copiar, mover, enviar a la papelera y extraer .zip.

use std::{
    fs,
    path::{Path, PathBuf},
};
use windows::{
    core::PCWSTR,
    Win32::UI::Shell::{SHFileOperationW, FOF_ALLOWUNDO, FOF_NOCONFIRMATION, FOF_NOERRORUI, FOF_SILENT, FO_DELETE, SHFILEOPSTRUCTW},
};

/// `dir/nombre`, o `dir/nombre (2)`, `(3)`… si ya existe: nunca se sobrescribe nada.
fn unique(dir: &Path, name: &str) -> PathBuf {
    let first = dir.join(name);
    if !first.exists() {
        return first;
    }
    let p = Path::new(name);
    let stem = p.file_stem().map(|s| s.to_string_lossy().into_owned()).unwrap_or_else(|| name.to_string());
    let ext = p.extension().map(|e| format!(".{}", e.to_string_lossy())).unwrap_or_default();
    (2..)
        .map(|n| dir.join(format!("{stem} ({n}){ext}")))
        .find(|c| !c.exists())
        .expect("hay un número libre")
}

fn copy_recursive(from: &Path, to: &Path) -> std::io::Result<()> {
    if from.is_dir() {
        fs::create_dir_all(to)?;
        for entry in fs::read_dir(from)? {
            let entry = entry?;
            copy_recursive(&entry.path(), &to.join(entry.file_name()))?;
        }
        Ok(())
    } else {
        fs::copy(from, to).map(|_| ())
    }
}

/// Destino sin colisiones dentro de `dest` (si ya existe, «nombre (2)»).
fn target_for(src: &Path, dest: &Path) -> Result<PathBuf, String> {
    let name = src.file_name().ok_or("Ruta no válida")?.to_string_lossy().into_owned();
    Ok(unique(dest, &name))
}

fn copy_to(paths: &[String], dest: &str) -> Result<Vec<String>, String> {
    let dest = Path::new(dest);
    if !dest.is_dir() {
        return Err("La carpeta de destino no existe.".into());
    }
    let mut out = vec![];
    for p in paths {
        let src = Path::new(p);
        if !src.exists() {
            return Err(format!("{} ya no existe.", src.file_name().unwrap_or_default().to_string_lossy()));
        }
        let to = target_for(src, dest)?;
        copy_recursive(src, &to).map_err(|e| e.to_string())?;
        out.push(to.to_string_lossy().into_owned());
    }
    Ok(out)
}

fn move_to(paths: &[String], dest: &str) -> Result<Vec<String>, String> {
    let dest = Path::new(dest);
    if !dest.is_dir() {
        return Err("La carpeta de destino no existe.".into());
    }
    let mut out = vec![];
    for p in paths {
        let src = Path::new(p);
        if !src.exists() {
            return Err(format!("{} ya no existe.", src.file_name().unwrap_or_default().to_string_lossy()));
        }
        // Mover a la misma carpeta no hace nada.
        if src.parent() == Some(dest) {
            out.push(p.clone());
            continue;
        }
        let to = target_for(src, dest)?;
        if fs::rename(src, &to).is_err() {
            // Entre unidades distintas no se puede renombrar: se copia y se borra el original.
            copy_recursive(src, &to).map_err(|e| e.to_string())?;
            if src.is_dir() { fs::remove_dir_all(src) } else { fs::remove_file(src) }.map_err(|e| e.to_string())?;
        }
        out.push(to.to_string_lossy().into_owned());
    }
    Ok(out)
}

fn trash(paths: &[String]) -> Result<(), String> {
    // Lista de rutas separadas por NUL y terminada con doble NUL.
    let mut from: Vec<u16> = vec![];
    for p in paths {
        from.extend(p.encode_utf16());
        from.push(0);
    }
    from.push(0);
    let mut op = SHFILEOPSTRUCTW {
        wFunc: FO_DELETE,
        pFrom: PCWSTR(from.as_ptr()),
        fFlags: (FOF_ALLOWUNDO | FOF_NOCONFIRMATION | FOF_NOERRORUI | FOF_SILENT).0 as u16,
        ..Default::default()
    };
    let rc = unsafe { SHFileOperationW(&mut op) };
    if rc != 0 || op.fAnyOperationsAborted.as_bool() {
        return Err("Windows no pudo enviar los archivos a la papelera.".into());
    }
    Ok(())
}

fn unzip(path: &str) -> Result<String, String> {
    let src = Path::new(path);
    let stem = src.file_stem().map(|s| s.to_string_lossy().into_owned()).unwrap_or_else(|| "extraído".into());
    let dir = unique(src.parent().ok_or("Ruta no válida")?, &stem);
    let file = fs::File::open(src).map_err(|e| e.to_string())?;
    let mut archive = zip::ZipArchive::new(file).map_err(|_| "No es un .zip válido.".to_string())?;
    // `extract` rechaza rutas que intenten salirse de la carpeta de destino.
    archive.extract(&dir).map_err(|e| e.to_string())?;
    Ok(dir.to_string_lossy().into_owned())
}

#[tauri::command]
pub async fn shelf_copy(paths: Vec<String>, dest: String) -> Result<Vec<String>, String> {
    tauri::async_runtime::spawn_blocking(move || copy_to(&paths, &dest)).await.map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn shelf_move(paths: Vec<String>, dest: String) -> Result<Vec<String>, String> {
    tauri::async_runtime::spawn_blocking(move || move_to(&paths, &dest)).await.map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn shelf_trash(paths: Vec<String>) -> Result<(), String> {
    tauri::async_runtime::spawn_blocking(move || trash(&paths)).await.map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn shelf_unzip(path: String) -> Result<String, String> {
    tauri::async_runtime::spawn_blocking(move || unzip(&path)).await.map_err(|e| e.to_string())?
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp(name: &str) -> PathBuf {
        let d = std::env::temp_dir().join(format!("noki-test-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&d);
        fs::create_dir_all(&d).unwrap();
        d
    }

    #[test]
    fn unique_names_never_overwrite() {
        let d = temp("unique");
        fs::write(d.join("a.txt"), "1").unwrap();
        assert_eq!(unique(&d, "a.txt"), d.join("a (2).txt"));
        fs::write(d.join("a (2).txt"), "2").unwrap();
        assert_eq!(unique(&d, "a.txt"), d.join("a (3).txt"));
        assert_eq!(unique(&d, "b.txt"), d.join("b.txt"));
    }

    #[test]
    fn copy_and_move_keep_content_and_avoid_collisions() {
        let src = temp("src");
        let dst = temp("dst");
        fs::write(src.join("n.txt"), "hola").unwrap();
        fs::create_dir_all(src.join("carpeta")).unwrap();
        fs::write(src.join("carpeta").join("x.txt"), "x").unwrap();
        fs::write(dst.join("n.txt"), "ya existía").unwrap();

        let copied = copy_to(&[src.join("n.txt").to_string_lossy().into(), src.join("carpeta").to_string_lossy().into()], &dst.to_string_lossy()).unwrap();
        assert_eq!(fs::read_to_string(&copied[0]).unwrap(), "hola");
        assert!(copied[0].ends_with("n (2).txt"));
        assert_eq!(fs::read_to_string(dst.join("n.txt")).unwrap(), "ya existía");
        assert_eq!(fs::read_to_string(dst.join("carpeta").join("x.txt")).unwrap(), "x");
        assert!(src.join("n.txt").exists());

        let moved = move_to(&[src.join("n.txt").to_string_lossy().into()], &dst.to_string_lossy()).unwrap();
        assert!(moved[0].ends_with("n (3).txt"));
        assert!(!src.join("n.txt").exists());
    }

    #[test]
    fn unzip_extracts_and_rejects_non_zip() {
        let d = temp("zip");
        let zpath = d.join("datos.zip");
        {
            let f = fs::File::create(&zpath).unwrap();
            let mut z = zip::ZipWriter::new(f);
            z.start_file("dentro.txt", zip::write::SimpleFileOptions::default()).unwrap();
            std::io::Write::write_all(&mut z, b"contenido").unwrap();
            z.finish().unwrap();
        }
        let out = unzip(&zpath.to_string_lossy()).unwrap();
        assert_eq!(fs::read_to_string(Path::new(&out).join("dentro.txt")).unwrap(), "contenido");
        fs::write(d.join("falso.zip"), "no soy un zip").unwrap();
        assert!(unzip(&d.join("falso.zip").to_string_lossy()).is_err());
    }
}
