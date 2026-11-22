//! Lee y renueva la sesión de Claude Code en su almacén original.
use serde::Deserialize;
use serde_json::{json, Value};
use std::{
    ffi::{c_void, OsString},
    fs::{self, File},
    io::{self, Read, Write},
    os::windows::process::CommandExt,
    path::{Path, PathBuf},
    process::{Command, Output, Stdio},
    time::{Duration, Instant},
};
use windows::{
    core::{w, HSTRING},
    Win32::Security::Credentials::{
        CredEnumerateW, CredFree, CredReadW, CredWriteW, CREDENTIALW,
        CRED_MAX_CREDENTIAL_BLOB_SIZE, CRED_TYPE_GENERIC,
    },
};

const TOKEN_URL: &str = "https://console.anthropic.com/v1/oauth/token";
const CLIENT_ID: &str = "9d1c250a-e61b-44d9-88ed-5944d1962f5e";
const FILE_NAME: &str = ".credentials.json";
const MAX_FILE_BYTES: u64 = 256 * 1024;

pub(crate) const NO_SESSION: &str = "Inicia sesión en el CLI para ver la cuota.";
pub(crate) const EXPIRED: &str = "La sesión de Claude Code ha caducado.";
pub(crate) const UNAVAILABLE: &str = "No se pudo consultar la cuota.";
const BUSY: &str =
    "Claude Code está usando el almacén de sesión. Vuelve a actualizar en unos segundos.";
const STORE_ERROR: &str = "No se pudo actualizar la sesión en su almacén original.";

#[derive(Clone, PartialEq, Eq)]
enum Store {
    File {
        path: PathBuf,
        wsl_home: Option<(String, String)>,
    },
    WindowsVault {
        target: String,
        config_dir: PathBuf,
    },
}

struct Login {
    store: Store,
    document: Value,
    written_at: i64,
}

impl Login {
    fn session(&self) -> &Value {
        &self.document["claudeAiOauth"]
    }

    fn live_token(&self, now: i64) -> Option<&str> {
        let session = self.session();
        if session["expiresAt"]
            .as_i64()
            .is_some_and(|expiry| expiry <= now)
        {
            return None;
        }
        nonempty(&session["accessToken"])
    }

    fn refresh_token(&self, now: i64) -> Option<&str> {
        let session = self.session();
        if session["refreshTokenExpiresAt"]
            .as_i64()
            .is_some_and(|expiry| expiry > 0 && expiry <= now)
        {
            return None;
        }
        nonempty(&session["refreshToken"])
    }

    fn same_tokens(&self, newer: &Self) -> bool {
        ["accessToken", "refreshToken"]
            .into_iter()
            .all(|field| self.session()[field] == newer.session()[field])
    }
}

pub(crate) fn nonempty(value: &Value) -> Option<&str> {
    value.as_str().filter(|value| !value.trim().is_empty())
}

pub(crate) fn now_ms() -> i64 {
    chrono::Utc::now().timestamp_millis()
}

pub(crate) fn file_login(path: &Path) -> Option<(Value, i64)> {
    let file = File::open(path).ok()?;
    let metadata = file.metadata().ok()?;
    if !metadata.is_file() || metadata.len() > MAX_FILE_BYTES {
        return None;
    }
    let mut bytes = Vec::new();
    file.take(MAX_FILE_BYTES).read_to_end(&mut bytes).ok()?;
    let written = metadata
        .modified()
        .ok()
        .map(chrono::DateTime::<chrono::Utc>::from)
        .map(|date| date.timestamp_millis())
        .unwrap_or_default();
    Some((decode_json(&bytes)?, written))
}

fn decode_json(bytes: &[u8]) -> Option<Value> {
    let text = serde_json::from_slice(bytes.strip_prefix(&[0xef, 0xbb, 0xbf]).unwrap_or(bytes));
    text.ok().or_else(|| {
        let bytes = bytes.strip_prefix(&[0xff, 0xfe]).unwrap_or(bytes);
        let wide = bytes
            .chunks_exact(2)
            .map(|pair| u16::from_le_bytes([pair[0], pair[1]]))
            .collect::<Vec<_>>();
        let text = String::from_utf16(&wide).ok()?;
        serde_json::from_str(text.trim_end_matches('\0')).ok()
    })
}

struct CredBuffer<T>(*mut T);

impl<T> Drop for CredBuffer<T> {
    fn drop(&mut self) {
        unsafe { CredFree(self.0.cast::<c_void>()) };
    }
}

fn vault_read(target: &str) -> Option<CredBuffer<CREDENTIALW>> {
    let mut raw = std::ptr::null_mut();
    unsafe {
        CredReadW(&HSTRING::from(target), CRED_TYPE_GENERIC, None, &mut raw).ok()?;
    }
    (!raw.is_null()).then(|| CredBuffer(raw))
}

fn vault_login(credential: &CREDENTIALW) -> Option<(Value, i64)> {
    if credential.CredentialBlob.is_null() || credential.CredentialBlobSize == 0 {
        return None;
    }
    let bytes = unsafe {
        std::slice::from_raw_parts(
            credential.CredentialBlob,
            credential.CredentialBlobSize as usize,
        )
    };
    let written = ((i64::from(credential.LastWritten.dwHighDateTime) << 32)
        | i64::from(credential.LastWritten.dwLowDateTime))
        / 10_000
        - 11_644_473_600_000;
    Some((decode_json(bytes)?, written))
}

impl Store {
    fn config_dir(&self) -> Option<&Path> {
        match self {
            Self::File { path, .. } => path.parent(),
            Self::WindowsVault { config_dir, .. } => Some(config_dir),
        }
    }

    fn read(&self) -> Option<Login> {
        let (document, written_at) = match self {
            Self::File { path, .. } => file_login(path)?,
            Self::WindowsVault { target, .. } => {
                let buffer = vault_read(target)?;
                vault_login(unsafe { &*buffer.0 })?
            }
        };
        document["claudeAiOauth"].is_object().then(|| Login {
            store: self.clone(),
            document,
            written_at,
        })
    }

    fn stage_write(&self) -> io::Result<Option<tempfile::NamedTempFile>> {
        let Self::File { path, wsl_home } = self else {
            return Ok(None);
        };
        // No sustituir un enlace simbólico que el usuario usa para compartir el perfil.
        if fs::symlink_metadata(path)?.file_type().is_symlink() {
            return Err(io::ErrorKind::Unsupported.into());
        }
        let tmp = tempfile::Builder::new()
            .prefix(".credentials-")
            .tempfile_in(path.parent().ok_or(io::ErrorKind::NotFound)?)?;
        if let Some((distro, home)) = wsl_home {
            // El archivo temporal de Linux debe ser privado antes de escribir el OAuth.
            let name = tmp
                .path()
                .file_name()
                .and_then(|name| name.to_str())
                .ok_or(io::ErrorKind::InvalidData)?;
            let linux_path = format!("{home}/.claude/{name}");
            let mut command = wsl_command();
            command.args([
                "--distribution",
                distro,
                "--exec",
                "/bin/chmod",
                "600",
                "--",
                &linux_path,
            ]);
            if !bounded_output(command).is_some_and(|output| output.status.success()) {
                return Err(io::ErrorKind::PermissionDenied.into());
            }
        }
        Ok(Some(tmp))
    }

    fn write(
        &self,
        document: &Value,
        staged: Option<tempfile::NamedTempFile>,
    ) -> Result<(), &'static str> {
        match self {
            Self::File { path, .. } => {
                let mut tmp = staged.ok_or(STORE_ERROR)?;
                serde_json::to_writer(&mut tmp, document).map_err(|_| STORE_ERROR)?;
                tmp.flush().map_err(|_| STORE_ERROR)?;
                tmp.as_file().sync_all().map_err(|_| STORE_ERROR)?;
                tmp.persist(path).map_err(|_| STORE_ERROR)?;
                Ok(())
            }
            Self::WindowsVault { target, .. } => {
                let buffer = vault_read(target).ok_or(STORE_ERROR)?;
                let mut credential = unsafe { *buffer.0 };
                let original = unsafe {
                    std::slice::from_raw_parts(
                        credential.CredentialBlob,
                        credential.CredentialBlobSize as usize,
                    )
                };
                let text = serde_json::to_string(document).map_err(|_| STORE_ERROR)?;
                let mut bytes =
                    if original.starts_with(&[0xff, 0xfe]) || original.get(1) == Some(&0) {
                        let mut bytes = Vec::new();
                        if original.starts_with(&[0xff, 0xfe]) {
                            bytes.extend([0xff, 0xfe]);
                        }
                        bytes.extend(text.encode_utf16().flat_map(u16::to_le_bytes));
                        bytes
                    } else {
                        text.into_bytes()
                    };
                if bytes.len() > CRED_MAX_CREDENTIAL_BLOB_SIZE as usize {
                    return Err(STORE_ERROR);
                }
                credential.CredentialBlob = bytes.as_mut_ptr();
                credential.CredentialBlobSize = bytes.len() as u32;
                unsafe { CredWriteW(&credential, 0).map_err(|_| STORE_ERROR) }
            }
        }
    }
}

fn wsl_command() -> Command {
    let system = std::env::var_os("SystemRoot")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(r"C:\Windows"));
    Command::new(system.join("System32/wsl.exe"))
}

fn bounded_output(mut command: Command) -> Option<Output> {
    let mut child = command
        .creation_flags(0x08000000)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .ok()?;
    let deadline = Instant::now() + Duration::from_secs(3);
    loop {
        if matches!(child.try_wait(), Ok(Some(_))) {
            return child.wait_with_output().ok();
        }
        if Instant::now() >= deadline {
            let _ = child.kill();
            let _ = child.wait();
            return None;
        }
        std::thread::sleep(Duration::from_millis(25));
    }
}

pub(crate) fn active_distros() -> Vec<String> {
    let mut command = wsl_command();
    command.args(["--list", "--running", "--quiet"]);
    let Some(output) = bounded_output(command).filter(|output| output.status.success()) else {
        return Vec::new();
    };
    let names = if output.stdout.contains(&0) || output.stdout.starts_with(&[0xff, 0xfe]) {
        let units = output
            .stdout
            .chunks_exact(2)
            .map(|pair| u16::from_le_bytes([pair[0], pair[1]]))
            .collect::<Vec<_>>();
        String::from_utf16_lossy(&units)
    } else {
        String::from_utf8_lossy(&output.stdout).into_owned()
    };
    names
        .lines()
        .map(|name| name.trim_matches(|c: char| c == '\u{feff}' || c.is_whitespace()))
        .filter(|name| safe_component(name))
        .map(str::to_owned)
        .collect()
}

fn safe_component(name: &str) -> bool {
    !name.is_empty() && !matches!(name, "." | "..") && !name.contains(['/', '\\', ':', '\0'])
}

fn wsl_stores() -> Vec<Store> {
    let mut stores = Vec::new();
    for distro in active_distros() {
        for share in [r"\\wsl.localhost", r"\\wsl$"] {
            let home = PathBuf::from(format!(r"{share}\{distro}\home"));
            let Ok(users) = fs::read_dir(home) else {
                continue;
            };
            for user in users.flatten() {
                let name = user.file_name();
                let Some(name) = name.to_str().filter(|name| safe_component(name)) else {
                    continue;
                };
                let path = user.path().join(".claude").join(FILE_NAME);
                if path.is_file() {
                    stores.push(Store::File {
                        path,
                        wsl_home: Some((distro.clone(), format!("/home/{name}"))),
                    });
                }
            }
            break;
        }
    }
    stores
}

fn discover() -> Vec<Login> {
    let home = std::env::var_os("USERPROFILE").map(|home| PathBuf::from(home).join(".claude"));
    let config = std::env::var_os("CLAUDE_CONFIG_DIR")
        .map(PathBuf::from)
        .filter(|dir| dir.is_absolute());
    let mut stores = Vec::new();
    for dir in [config.as_ref(), home.as_ref()].into_iter().flatten() {
        let store = Store::File {
            path: dir.join(FILE_NAME),
            wsl_home: None,
        };
        if !stores.contains(&store) {
            stores.push(store);
        }
    }
    if let Some(config_dir) = config.or(home) {
        let mut raw = std::ptr::null_mut();
        let mut count = 0;
        unsafe {
            if CredEnumerateW(w!("Claude Code-credentials*"), None, &mut count, &mut raw).is_ok()
                && !raw.is_null()
            {
                let buffer = CredBuffer(raw);
                for credential in std::slice::from_raw_parts(buffer.0, count as usize) {
                    let credential = &**credential;
                    if credential.Type != CRED_TYPE_GENERIC {
                        continue;
                    }
                    if let Ok(target) = credential.TargetName.to_string() {
                        stores.push(Store::WindowsVault {
                            target,
                            config_dir: config_dir.clone(),
                        });
                    }
                }
            }
        }
    }
    stores.extend(wsl_stores());
    let mut logins = stores.iter().filter_map(Store::read).collect::<Vec<_>>();
    let now = now_ms();
    logins.sort_by_key(|login| {
        std::cmp::Reverse((
            login.live_token(now).is_some(),
            login.written_at,
            login.session()["expiresAt"].as_i64(),
        ))
    });
    logins
}

struct Lock(PathBuf);

impl Lock {
    fn acquire(path: PathBuf) -> Result<Self, &'static str> {
        fs::create_dir(&path).map_err(|error| {
            if error.kind() == io::ErrorKind::AlreadyExists {
                BUSY
            } else {
                STORE_ERROR
            }
        })?;
        Ok(Self(path))
    }
}

impl Drop for Lock {
    fn drop(&mut self) {
        let _ = fs::remove_dir(&self.0);
    }
}

#[derive(Deserialize)]
struct TokenResponse {
    access_token: String,
    refresh_token: Option<String>,
    expires_in: i64,
    refresh_token_expires_in: Option<i64>,
}

fn write_rotation(document: &mut Value, response: TokenResponse, now: i64) -> Option<String> {
    if response.access_token.trim().is_empty() || response.expires_in <= 0 {
        return None;
    }
    let session = document["claudeAiOauth"].as_object_mut()?;
    let access = response.access_token;
    session.insert("accessToken".into(), json!(&access));
    session.insert(
        "expiresAt".into(),
        json!(now.saturating_add(response.expires_in.saturating_mul(1000))),
    );
    if let Some(refresh) = response
        .refresh_token
        .filter(|token| !token.trim().is_empty())
    {
        session.insert("refreshToken".into(), json!(refresh));
    }
    if let Some(seconds) = response
        .refresh_token_expires_in
        .filter(|seconds| *seconds > 0)
    {
        session.insert(
            "refreshTokenExpiresAt".into(),
            json!(now.saturating_add(seconds.saturating_mul(1000))),
        );
    }
    Some(access)
}

struct Rotation {
    login: Login,
    staged: Option<tempfile::NamedTempFile>,
    _locks: [Lock; 3],
}

enum Prepared {
    Current(String),
    NeedsRefresh(Rotation),
}

fn prepare_rotation(store: Store) -> Result<Prepared, &'static str> {
    let config_dir = store.config_dir().ok_or(STORE_ERROR)?;
    let refresh_lock = Lock::acquire(config_dir.join(".oauth_refresh.lock"))?;
    let mut legacy = OsString::from(config_dir.as_os_str());
    legacy.push(".lock");
    let legacy_lock = Lock::acquire(PathBuf::from(legacy))?;
    let write_lock = Lock::acquire(config_dir.join(".storage-write.lock"))?;
    let login = store.read().ok_or(EXPIRED)?;
    if let Some(token) = login.live_token(now_ms()) {
        return Ok(Prepared::Current(token.to_owned()));
    }
    login.refresh_token(now_ms()).ok_or(EXPIRED)?;
    let staged = store.stage_write().map_err(|_| STORE_ERROR)?;
    Ok(Prepared::NeedsRefresh(Rotation {
        login,
        staged,
        _locks: [write_lock, legacy_lock, refresh_lock],
    }))
}

async fn renew(client: &reqwest::Client, store: Store) -> Result<String, &'static str> {
    let rotation = match tauri::async_runtime::spawn_blocking(move || prepare_rotation(store))
        .await
        .map_err(|_| UNAVAILABLE)??
    {
        Prepared::Current(token) => return Ok(token),
        Prepared::NeedsRefresh(rotation) => rotation,
    };
    let refresh = rotation.login.refresh_token(now_ms()).ok_or(EXPIRED)?;
    let response = client
        .post(TOKEN_URL)
        .json(&json!({"grant_type":"refresh_token","refresh_token":refresh,"client_id":CLIENT_ID}))
        .send()
        .await
        .map_err(|_| UNAVAILABLE)?;
    if matches!(response.status().as_u16(), 400 | 401 | 403) {
        return Err(EXPIRED);
    }
    let response = response
        .error_for_status()
        .map_err(|_| UNAVAILABLE)?
        .json::<TokenResponse>()
        .await
        .map_err(|_| UNAVAILABLE)?;
    tauri::async_runtime::spawn_blocking(move || {
        let mut current = rotation.login.store.read().ok_or(STORE_ERROR)?;
        if !rotation.login.same_tokens(&current) {
            return current
                .live_token(now_ms())
                .map(str::to_owned)
                .ok_or(STORE_ERROR);
        }
        let access =
            write_rotation(&mut current.document, response, now_ms()).ok_or(UNAVAILABLE)?;
        rotation
            .login
            .store
            .write(&current.document, rotation.staged)?;
        Ok(access)
    })
    .await
    .map_err(|_| UNAVAILABLE)?
}

pub(crate) async fn access_token(client: &reqwest::Client) -> Result<String, &'static str> {
    let logins = tauri::async_runtime::spawn_blocking(discover)
        .await
        .map_err(|_| UNAVAILABLE)?;
    if logins.is_empty() {
        return Err(NO_SESSION);
    }
    for login in logins {
        if let Some(token) = login.live_token(now_ms()) {
            return Ok(token.to_owned());
        }
        if login.refresh_token(now_ms()).is_some() {
            match renew(client, login.store).await {
                // Puede haber otro perfil renovable aunque Anthropic haya revocado éste.
                Err(EXPIRED) => continue,
                result => return result,
            }
        }
    }
    Err(EXPIRED)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rotation_replaces_both_tokens_preserving_cli_metadata() {
        let mut login = json!({"claudeAiOauth":{"accessToken":"old","refreshToken":"old-refresh","expiresAt":1,"scopes":["user:profile"]},"other":true});
        let response = serde_json::from_value(
            json!({"access_token":"new","refresh_token":"new-refresh","expires_in":7200}),
        )
        .unwrap();
        assert_eq!(
            write_rotation(&mut login, response, 1000).as_deref(),
            Some("new")
        );
        assert_eq!(
            login,
            json!({"claudeAiOauth":{"accessToken":"new","refreshToken":"new-refresh","expiresAt":7201000,"scopes":["user:profile"]},"other":true})
        );
    }

    #[test]
    fn provider_lock_prevents_a_second_rotation_and_is_released() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join(".oauth_refresh.lock");
        let lock = Lock::acquire(path.clone()).unwrap();
        assert!(matches!(Lock::acquire(path.clone()), Err(BUSY)));
        drop(lock);
        assert!(Lock::acquire(path).is_ok());
    }
}
