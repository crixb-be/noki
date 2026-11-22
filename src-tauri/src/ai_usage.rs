//! Cuotas de suscripción de las sesiones locales. Solo las métricas llegan a la interfaz.
use crate::{claude_session, gemini_session, notch::NotchState, settings::{Settings, TopBar}};
use futures_util::FutureExt;
use serde::Serialize;
use serde_json::{json, Value};
use std::{
    io::{BufRead, BufReader, Write},
    os::windows::process::CommandExt,
    path::PathBuf,
    process::{Command, Stdio},
    sync::{mpsc, Mutex},
    time::Duration,
};
use tauri::{AppHandle, Emitter, Manager, State, WebviewWindow};
use tokio::sync::Notify;

const INTERVAL: Duration = Duration::from_secs(300);
const TIMEOUT: Duration = Duration::from_secs(15);

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Limit {
    pub(crate) minutes: Option<u64>,
    pub(crate) used_percent: f64,
    pub(crate) resets_at: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) label: Option<String>,
}

#[derive(Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
struct Provider {
    limits: Vec<Limit>,
    updated_at: Option<i64>,
    error: Option<&'static str>,
}

impl Provider {
    fn apply(&mut self, result: Result<Vec<Limit>, &'static str>) {
        match result {
            Ok(limits) => {
                self.limits = limits;
                self.updated_at = Some(chrono::Utc::now().timestamp());
                self.error = None;
            }
            Err(error) => {
                self.error = Some(error);
                if matches!(
                    error,
                    NO_SESSION | NO_LIMITS | claude_session::EXPIRED
                        | gemini_session::NO_SESSION | gemini_session::EXPIRED
                        | gemini_session::UNSUPPORTED | gemini_session::NO_QUOTA
                ) {
                    self.limits.clear();
                    self.updated_at = None;
                }
            }
        }
    }
}

#[derive(Clone, Serialize)]
pub struct Snapshot {
    claude: Provider,
    codex: Provider,
    gemini: Provider,
    refreshing: bool,
}

#[derive(Clone, Copy)]
enum Kind {
    Claude,
    Codex,
    Gemini,
}

impl Kind {
    const ALL: [Self; 3] = [Self::Claude, Self::Codex, Self::Gemini];

    fn enabled(self, topbar: &TopBar) -> bool {
        topbar.enabled
            && topbar.show_ai_usage
            && match self {
                Self::Claude => topbar.show_claude_quota,
                Self::Codex => topbar.show_codex_quota,
                Self::Gemini => topbar.show_gemini_quota,
            }
    }
}

#[derive(Default)]
struct Slot {
    data: Provider,
    refreshing: bool,
    generation: u64,
}

#[derive(Default)]
struct Providers([Slot; 3]);

impl Providers {
    fn snapshot(&self, topbar: &TopBar) -> Snapshot {
        let [claude, codex, gemini] = Kind::ALL.map(|kind| {
            if kind.enabled(topbar) {
                self.0[kind as usize].data.clone()
            } else {
                Provider::default()
            }
        });
        Snapshot {
            claude,
            codex,
            gemini,
            refreshing: Kind::ALL.into_iter().any(|kind| {
                kind.enabled(topbar) && self.0[kind as usize].refreshing
            }),
        }
    }

    fn settings_changed(&mut self, app: &AppHandle, prev: &TopBar, new: &TopBar) {
        let mut changed = false;
        let mut wake = [false; 3];
        for kind in Kind::ALL {
            let enabled = kind.enabled(new);
            if enabled == kind.enabled(prev) {
                continue;
            }
            changed = true;
            if enabled {
                wake[kind as usize] = true;
            } else {
                let slot = &mut self.0[kind as usize];
                slot.data = Provider::default();
                slot.generation += 1;
            }
        }
        if changed {
            let _ = app.emit("ai-usage-updated", self.snapshot(new));
            let state = app.state::<UsageState>();
            for (wake, notify) in wake.into_iter().zip(&state.wake) {
                if wake {
                    notify.notify_one();
                }
            }
        }
    }
}

#[derive(Default)]
pub struct UsageState {
    snapshot: Mutex<Providers>,
    wake: [Notify; 3],
}

use claude_session::{NO_SESSION, UNAVAILABLE};
const NO_LIMITS: &str = "La cuenta no ofrece datos de cuota.";

pub fn replace_settings(app: &AppHandle, new: &Settings) {
    let usage = app.state::<UsageState>();
    let notch = app.state::<NotchState>();
    // Publica los ajustes y la transición juntos, con el mismo orden de bloqueo que las consultas.
    let mut providers = usage.snapshot.lock().unwrap();
    let prev = std::mem::replace(&mut *notch.settings.lock().unwrap(), new.clone());
    providers.settings_changed(app, &prev.topbar, &new.topbar);
}

pub fn start(app: AppHandle) {
    for kind in Kind::ALL {
        tauri::async_runtime::spawn(poll(app.clone(), kind));
    }
}

async fn poll(app: AppHandle, kind: Kind) {
    let usage = app.state::<UsageState>();
    let notch = app.state::<NotchState>();
    let index = kind as usize;
    let wake = &usage.wake[index];
    let mut client = None;
    let mut gemini = gemini_session::Session::default();
    loop {
        let generation = {
            let mut providers = usage.snapshot.lock().unwrap();
            let settings = notch.settings.lock().unwrap();
            if kind.enabled(&settings.topbar) {
                // Las solicitudes recibidas antes de iniciar se atienden en esta misma consulta.
                let _ = wake.notified().now_or_never();
                let slot = &mut providers.0[index];
                slot.refreshing = true;
                let generation = slot.generation;
                let snapshot = providers.snapshot(&settings.topbar);
                drop(settings);
                let _ = app.emit("ai-usage-updated", snapshot);
                Some(generation)
            } else {
                None
            }
        };
        let Some(generation) = generation else {
            wake.notified().await;
            continue;
        };
        let started = tokio::time::Instant::now();
        let result = match kind {
            Kind::Codex => tauri::async_runtime::spawn_blocking(codex_limits)
                .await.unwrap_or(Err(UNAVAILABLE)),
            Kind::Claude | Kind::Gemini => {
                let client = client.get_or_insert_with(|| reqwest::Client::builder()
                    .timeout(TIMEOUT)
                    .redirect(reqwest::redirect::Policy::none())
                    .build());
                match (kind, client.as_ref()) {
                    (Kind::Claude, Ok(client)) => claude_limits(client).await,
                    (Kind::Gemini, Ok(client)) => gemini.limits(client).await,
                    (Kind::Gemini, Err(_)) => Err(gemini_session::UNAVAILABLE),
                    _ => Err(UNAVAILABLE),
                }
            }
        };
        let enabled = {
            let mut providers = usage.snapshot.lock().unwrap();
            let settings = notch.settings.lock().unwrap();
            let enabled = kind.enabled(&settings.topbar);
            let slot = &mut providers.0[index];
            slot.refreshing = false;
            if enabled {
                if slot.generation == generation {
                    slot.data.apply(result);
                }
                let snapshot = providers.snapshot(&settings.topbar);
                drop(settings);
                let _ = app.emit("ai-usage-updated", snapshot);
            }
            enabled
        };
        if enabled {
            let _ = tokio::time::timeout_at(started + INTERVAL, wake.notified()).await;
        }
    }
}

#[tauri::command]
pub fn ai_usage_snapshot(state: State<UsageState>, notch: State<NotchState>) -> Snapshot {
    let providers = state.snapshot.lock().unwrap();
    let settings = notch.settings.lock().unwrap();
    providers.snapshot(&settings.topbar)
}

#[tauri::command]
pub fn refresh_ai_usage(window: WebviewWindow, state: State<UsageState>) {
    if window.label() != "barpanel" {
        return;
    }
    let notch = window.state::<NotchState>();
    let providers = state.snapshot.lock().unwrap();
    let settings = notch.settings.lock().unwrap();
    for kind in Kind::ALL {
        let index = kind as usize;
        if kind.enabled(&settings.topbar) && !providers.0[index].refreshing {
            state.wake[index].notify_one();
        }
    }
}

fn limits_or_error(limits: Vec<Limit>) -> Result<Vec<Limit>, &'static str> {
    if limits.is_empty() {
        Err(NO_LIMITS)
    } else {
        Ok(limits)
    }
}

async fn claude_limits(client: &reqwest::Client) -> Result<Vec<Limit>, &'static str> {
    let token = claude_session::access_token(client).await?;
    let response = client
        .get("https://api.anthropic.com/api/oauth/usage")
        .bearer_auth(token)
        .header("anthropic-beta", "oauth-2025-04-20")
        .header("accept", "application/json")
        .send()
        .await
        .map_err(|_| UNAVAILABLE)?;
    if matches!(response.status().as_u16(), 401 | 403) {
        return Err(NO_SESSION);
    }
    let value: Value = response
        .error_for_status()
        .map_err(|_| UNAVAILABLE)?
        .json()
        .await
        .map_err(|_| UNAVAILABLE)?;
    limits_or_error(parse_claude(&value))
}

fn parse_claude(value: &Value) -> Vec<Limit> {
    [("five_hour", 300), ("seven_day", 10080)]
        .into_iter()
        .filter_map(|(key, minutes)| {
            let window = &value[key];
            Some(Limit {
                minutes: Some(minutes),
                label: None,
                used_percent: window["utilization"].as_f64()?.clamp(0.0, 100.0),
                resets_at: window["resets_at"]
                    .as_str()
                    .and_then(|date| chrono::DateTime::parse_from_rfc3339(date).ok())
                    .map(|date| date.timestamp()),
            })
        })
        .collect()
}

fn codex_exe() -> Option<PathBuf> {
    let paths = std::env::var_os("PATH").unwrap_or_default();
    for dir in std::env::split_paths(&paths).filter(|dir| dir.is_absolute()) {
        let executable = dir.join("codex.exe");
        if executable.is_file() {
            return Some(executable);
        }
        // La instalación global de npm usa un .cmd; su paquete incluye el binario nativo.
        if dir.join("codex.cmd").is_file() {
            let package = dir.join("node_modules/@openai/codex");
            let arch = std::env::consts::ARCH;
            let package_arch = if arch == "aarch64" { "arm64" } else { "x64" };
            let vendor = format!("vendor/{arch}-pc-windows-msvc/bin/codex.exe");
            let platform = format!("@openai/codex-win32-{package_arch}/{vendor}");
            for candidate in [
                package.join("node_modules").join(&platform),
                dir.join("node_modules").join(&platform),
                package.join(&vendor),
            ] {
                if candidate.is_file() {
                    return Some(candidate);
                }
            }
        }
    }
    None
}

fn codex_limits() -> Result<Vec<Limit>, &'static str> {
    let executable = codex_exe().ok_or("Codex CLI no está instalado o no está en PATH.")?;
    let mut child = Command::new(executable)
        .args(["app-server", "--listen", "stdio://"])
        .current_dir(std::env::temp_dir())
        .creation_flags(0x08000000) // CREATE_NO_WINDOW
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|_| UNAVAILABLE)?;
    let mut input = child.stdin.take().ok_or(UNAVAILABLE)?;
    let output = child.stdout.take().ok_or(UNAVAILABLE)?;
    let (tx, rx) = mpsc::channel();
    let reader = std::thread::spawn(move || {
        for line in BufReader::new(output).lines().map_while(Result::ok) {
            if let Ok(value) = serde_json::from_str::<Value>(&line) {
                if value["id"].as_u64().is_some() && tx.send(value).is_err() {
                    break;
                }
            }
        }
    });
    let deadline = std::time::Instant::now() + TIMEOUT;
    let result = (|| {
        let mut send = |message: Value| writeln!(input, "{message}").map_err(|_| UNAVAILABLE);
        let receive = |id| loop {
            let value = rx
                .recv_timeout(deadline.saturating_duration_since(std::time::Instant::now()))
                .map_err(|_| UNAVAILABLE)?;
            if value["id"].as_u64() == Some(id) {
                break Ok::<Value, &'static str>(value);
            }
        };
        send(
            json!({"id":1, "method":"initialize", "params":{"clientInfo":{"name":"noki", "title":"Noki", "version":env!("CARGO_PKG_VERSION")}}}),
        )?;
        if receive(1)?["result"].is_null() {
            return Err(UNAVAILABLE);
        }
        send(json!({"method":"initialized"}))?;
        send(json!({"id":2, "method":"account/read", "params":{"refreshToken":false}}))?;
        let account = receive(2)?;
        if account["result"].is_null() {
            return Err(UNAVAILABLE);
        }
        match account["result"]["account"]["type"].as_str() {
            Some("apiKey") => return Err(NO_LIMITS),
            None => return Err(NO_SESSION),
            _ => {}
        }
        send(json!({"id":3, "method":"account/rateLimits/read"}))?;
        let response = receive(3)?;
        if response["result"].is_null() {
            return Err(UNAVAILABLE);
        }
        limits_or_error(parse_codex(&response["result"]))
    })();
    drop(input);
    let _ = child.kill();
    let _ = child.wait();
    let _ = reader.join();
    result
}

fn parse_codex(result: &Value) -> Vec<Limit> {
    let preferred = &result["rateLimitsByLimitId"]["codex"];
    let value = if preferred.is_object() {
        preferred
    } else {
        &result["rateLimits"]
    };
    ["primary", "secondary"]
        .into_iter()
        .filter_map(|key| {
            let window = &value[key];
            Some(Limit {
                minutes: window["windowDurationMins"].as_u64(),
                label: None,
                used_percent: window["usedPercent"].as_f64()?.clamp(0.0, 100.0),
                resets_at: window["resetsAt"].as_i64(),
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn snapshot_counts_only_enabled_refreshes_and_omits_absent_labels() {
        let topbar = TopBar {
            enabled: true,
            show_ai_usage: true,
            show_codex_quota: false,
            ..TopBar::default()
        };
        let mut providers = Providers::default();
        providers.0[Kind::Codex as usize].refreshing = true;
        providers.0[Kind::Claude as usize].data.apply(Ok(parse_claude(
            &json!({"five_hour":{"utilization":37.0}}),
        )));
        let snapshot = serde_json::to_value(providers.snapshot(&topbar)).unwrap();
        assert_eq!(snapshot.as_object().unwrap().len(), 4);
        assert_eq!(snapshot["refreshing"], false);
        assert!(snapshot["claude"]["limits"][0].get("label").is_none());
        assert_eq!(snapshot["gemini"]["limits"], json!([]));

        providers.0[Kind::Gemini as usize].refreshing = true;
        assert!(providers.snapshot(&topbar).refreshing);
    }

    #[test]
    fn transient_gemini_errors_keep_limits_and_definitive_errors_clear_them() {
        let limits = vec![Limit {
            minutes: None,
            used_percent: 20.0,
            resets_at: None,
            label: Some("Gemini Flash".into()),
        }];
        let mut provider = Provider::default();
        for error in [
            gemini_session::NO_SESSION,
            gemini_session::EXPIRED,
            gemini_session::UNSUPPORTED,
            gemini_session::NO_QUOTA,
        ] {
            provider.apply(Ok(limits.clone()));
            provider.apply(Err(gemini_session::UNAVAILABLE));
            assert_eq!(provider.limits.len(), 1);
            assert!(provider.updated_at.is_some());
            provider.apply(Err(error));
            assert!(provider.limits.is_empty());
            assert!(provider.updated_at.is_none());
        }
    }
}
