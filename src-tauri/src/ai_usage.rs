//! Cuotas de suscripción de las sesiones locales. Solo las métricas llegan a la interfaz.
use crate::{claude_session, notch::NotchState};
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
struct Limit {
    minutes: Option<u64>,
    used_percent: f64,
    resets_at: Option<i64>,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct Provider {
    limits: Vec<Limit>,
    updated_at: Option<i64>,
    error: Option<&'static str>,
}

impl Provider {
    const fn empty() -> Self {
        Self {
            limits: Vec::new(),
            updated_at: None,
            error: None,
        }
    }

    fn apply(&mut self, result: Result<Vec<Limit>, &'static str>) {
        match result {
            Ok(limits) => {
                self.limits = limits;
                self.updated_at = Some(chrono::Utc::now().timestamp());
                self.error = None;
            }
            Err(error) => {
                self.error = Some(error);
                if matches!(error, NO_SESSION | NO_LIMITS | claude_session::EXPIRED) {
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
    refreshing: bool,
}

impl Default for Snapshot {
    fn default() -> Self {
        Self {
            claude: Provider::empty(),
            codex: Provider::empty(),
            refreshing: false,
        }
    }
}

#[derive(Default)]
pub struct UsageState {
    snapshot: Mutex<Snapshot>,
    wake: Notify,
}

use claude_session::{NO_SESSION, UNAVAILABLE};
const NO_LIMITS: &str = "La cuenta no ofrece datos de cuota.";

fn enabled(app: &AppHandle) -> bool {
    let settings = app.state::<NotchState>();
    let settings = settings.settings.lock().unwrap();
    settings.topbar.enabled && settings.topbar.show_ai_usage
}

pub fn settings_changed(app: &AppHandle) {
    let state = app.state::<UsageState>();
    if !enabled(app) {
        *state.snapshot.lock().unwrap() = Snapshot::default();
    }
    state.wake.notify_one();
}

pub fn start(app: AppHandle) {
    tauri::async_runtime::spawn(async move {
        let state = app.state::<UsageState>();
        let client = reqwest::Client::builder()
            .timeout(TIMEOUT)
            .redirect(reqwest::redirect::Policy::none())
            .build();
        loop {
            if !enabled(&app) {
                state.wake.notified().await;
                continue;
            }
            let started = tokio::time::Instant::now();
            {
                let mut snapshot = state.snapshot.lock().unwrap();
                snapshot.refreshing = true;
                let _ = app.emit("ai-usage-updated", &*snapshot);
            }
            let codex = tauri::async_runtime::spawn_blocking(codex_limits);
            let claude = match &client {
                Ok(client) => claude_limits(client).await,
                Err(_) => Err(UNAVAILABLE),
            };
            let codex = codex.await.unwrap_or(Err(UNAVAILABLE));
            if enabled(&app) {
                let mut snapshot = state.snapshot.lock().unwrap();
                snapshot.claude.apply(claude);
                snapshot.codex.apply(codex);
                snapshot.refreshing = false;
                let _ = app.emit("ai-usage-updated", &*snapshot);
            }
            let _ = tokio::time::timeout_at(started + INTERVAL, state.wake.notified()).await;
        }
    });
}

#[tauri::command]
pub fn ai_usage_snapshot(state: State<UsageState>) -> Snapshot {
    state.snapshot.lock().unwrap().clone()
}

#[tauri::command]
pub fn refresh_ai_usage(window: WebviewWindow, state: State<UsageState>) {
    if window.label() == "barpanel"
        && enabled(window.app_handle())
        && !state.snapshot.lock().unwrap().refreshing
    {
        state.wake.notify_one();
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
                used_percent: window["usedPercent"].as_f64()?.clamp(0.0, 100.0),
                resets_at: window["resetsAt"].as_i64(),
            })
        })
        .collect()
}
