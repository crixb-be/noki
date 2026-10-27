//! Claves de API en el Administrador de credenciales de Windows (no en `settings.json`).
//!
//! En memoria, `Settings` guarda la clave real para que `ai.rs` pueda usarla. Hacia el frontend y hacia
//! el disco nunca viaja: el frontend recibe `MASK` si hay una clave guardada, y el archivo de ajustes
//! se escribe siempre sin claves.

use keyring::Entry;

const SERVICE: &str = "Noki";
/// Nombre anterior de la app: las claves guardadas con él se copian al nombre nuevo la primera vez.
const LEGACY_SERVICE: &str = "MiNotch";

/// Lo que ve el frontend en lugar de una clave guardada.
pub const MASK: &str = "••••••••";

/// Proveedores cuya clave se guarda (Ollama no usa clave).
pub const PROVIDERS: [&str; 3] = ["gemini", "claude", "openai"];

fn entry(provider: &str) -> Result<Entry, String> {
    Entry::new(SERVICE, &format!("ai.{provider}")).map_err(|e| e.to_string())
}

pub fn get(provider: &str) -> Option<String> {
    if let Some(k) = entry(provider).ok()?.get_password().ok().filter(|k| !k.is_empty()) {
        return Some(k);
    }
    // Migración: la clave pudo guardarse cuando la app se llamaba «Mi Notch».
    let old = Entry::new(LEGACY_SERVICE, &format!("ai.{provider}")).ok()?;
    let key = old.get_password().ok().filter(|k| !k.is_empty())?;
    let _ = entry(provider).ok()?.set_password(&key);
    Some(key)
}

/// Guarda la clave; una clave vacía borra la credencial.
pub fn set(provider: &str, key: &str) -> Result<(), String> {
    let e = entry(provider)?;
    if key.trim().is_empty() {
        return match e.delete_credential() {
            Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
            Err(err) => Err(err.to_string()),
        };
    }
    e.set_password(key.trim()).map_err(|err| format!("No se pudo guardar la clave en Windows: {err}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Usa una credencial de prueba real del Administrador de credenciales y la borra al terminar.
    #[test]
    fn roundtrip_in_windows_credential_manager() {
        let user = "ai.test-noki-roundtrip";
        let e = Entry::new(SERVICE, user).unwrap();
        let _ = e.delete_credential();
        assert!(e.get_password().is_err());
        e.set_password("sk-prueba-ñ-🙂").unwrap();
        assert_eq!(e.get_password().unwrap(), "sk-prueba-ñ-🙂");
        e.delete_credential().unwrap();
        assert!(matches!(e.get_password(), Err(keyring::Error::NoEntry)));
    }
}
