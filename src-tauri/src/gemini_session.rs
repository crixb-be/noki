//! Consulta la cuota de una sesión de Gemini CLI sin modificar su almacén.
use crate::{
    ai_usage::Limit,
    claude_session::{active_distros, file_login, nonempty, now_ms},
};
use serde_json::{json, Value};
use std::{
    env, fs,
    io::Read,
    path::{Component, Path, PathBuf, Prefix},
};

pub(crate) const NO_SESSION: &str = "Inicia sesión en Gemini CLI para ver la cuota.";
pub(crate) const EXPIRED: &str = "La sesión de Gemini CLI ha caducado.";
pub(crate) const UNSUPPORTED: &str = "Google ya no admite Gemini CLI en cuentas individuales; solo Code Assist Standard y Enterprise ofrecen esta cuota.";
pub(crate) const UNAVAILABLE: &str = "No se pudo consultar la cuota de Gemini.";
const NO_CLI: &str = "Abre Gemini CLI para renovar la sesión.";
pub(crate) const NO_QUOTA: &str = "Gemini no ofrece datos de cuota.";
const MAX_JS_BYTES: u64 = 32 * 1024 * 1024;
const CORE_OAUTH: &str = "@google/gemini-cli-core/dist/src/code_assist/oauth2.js";

#[derive(Clone)]
struct Token {
    access: String,
    expiry_ms: i64,
}

impl Token {
    fn live(&self, now: i64) -> bool {
        !self.access.trim().is_empty() && self.expiry_ms > now.saturating_add(60_000)
    }
}

#[derive(PartialEq)]
struct LoginKey {
    path: PathBuf,
    refresh: String,
}

struct Login {
    key: LoginKey,
    token: Token,
    written: i64,
}

impl Login {
    fn read(path: PathBuf) -> Option<Self> {
        let (document, written) = file_login(&path)?;
        let access = nonempty(&document["access_token"]).unwrap_or_default();
        let refresh = nonempty(&document["refresh_token"]).unwrap_or_default();
        if access.is_empty() && refresh.is_empty() {
            return None;
        }
        Some(Self {
            key: LoginKey { path, refresh: refresh.to_owned() },
            token: Token {
                access: access.to_owned(),
                expiry_ms: document["expiry_date"].as_i64().unwrap_or_default(),
            },
            written,
        })
    }
}

fn wsl_roots() -> impl Iterator<Item = PathBuf> {
    // Usa la lista de distros activas antes de acceder al recurso compartido.
    active_distros().into_iter().map(|distro| PathBuf::from(format!(r"\\wsl$\{distro}")))
}

fn available_path(path: &Path) -> bool {
    if !path.is_absolute() {
        return false;
    }
    // Las rutas configuradas en Windows también pueden apuntar a WSL.
    if let Some(Component::Prefix(prefix)) = path.components().next() {
        if let Prefix::UNC(server, distro) | Prefix::VerbatimUNC(server, distro) = prefix.kind() {
            if server.eq_ignore_ascii_case("wsl$") || server.eq_ignore_ascii_case("wsl.localhost") {
                return active_distros().into_iter().any(|name| distro.eq_ignore_ascii_case(name));
            }
        }
    }
    true
}

fn wsl_homes(root: &Path) -> impl Iterator<Item = PathBuf> {
    fs::read_dir(root.join("home"))
        .into_iter().flatten().flatten()
        .map(|entry| entry.path())
        .chain(std::iter::once(root.join("root")))
}

fn discover() -> Vec<Login> {
    let mut paths = Vec::new();
    let mut logins = Vec::new();
    for home in ["GEMINI_CLI_HOME", "USERPROFILE"].into_iter()
        .filter_map(|key| env::var_os(key).map(PathBuf::from))
        .filter(|path| available_path(path))
        .chain(std::iter::once_with(wsl_roots).flatten().flat_map(|root| wsl_homes(&root)))
    {
        let path = home.join(".gemini/oauth_creds.json");
        if !paths.contains(&path) {
            paths.push(path.clone());
            logins.extend(Login::read(path));
        }
    }
    logins
}

struct OAuthClient {
    id: String,
    secret: String,
}

fn quoted_constant(source: &str, name: &str) -> Option<String> {
    let identifier = |c: char| c.is_ascii_alphanumeric() || matches!(c, '_' | '$');
    source.match_indices(name).find_map(|(offset, _)| {
        if source[..offset].ends_with(identifier) {
            return None;
        }
        let value = source[offset + name.len()..].trim_start().strip_prefix('=')?.trim_start();
        let quote = value.chars().next().filter(|c| matches!(c, '"' | '\''))?;
        let (value, _) = value[1..].split_once(quote)?;
        if value.is_empty() || value.bytes().any(|c| c.is_ascii_whitespace() || c == b'\\') {
            return None;
        }
        Some(value.to_owned())
    })
}

fn client_in_file(path: &Path) -> Option<OAuthClient> {
    let file = fs::File::open(path).ok()?;
    let metadata = file.metadata().ok()?;
    if !metadata.is_file() || metadata.len() > MAX_JS_BYTES {
        return None;
    }
    let mut source = String::new();
    file.take(MAX_JS_BYTES + 1).read_to_string(&mut source).ok()?;
    if source.len() as u64 > MAX_JS_BYTES {
        return None;
    }
    Some(OAuthClient {
        id: quoted_constant(&source, "OAUTH_CLIENT_ID")?,
        secret: quoted_constant(&source, "OAUTH_CLIENT_SECRET")?,
    })
}

fn client_in_modules(modules: &Path) -> Option<OAuthClient> {
    let cli = modules.join("@google/gemini-cli");
    client_in_file(&cli.join("node_modules").join(CORE_OAUTH))
        .or_else(|| client_in_file(&modules.join(CORE_OAUTH)))
        .or_else(|| {
            fs::read_dir(cli.join("bundle")).ok()?.flatten()
                .map(|entry| entry.path())
                .filter(|path| path.extension().is_some_and(|ext| ext == "js"))
                .find_map(|path| client_in_file(&path))
        })
}

fn find_client() -> Option<OAuthClient> {
    let paths = env::var_os("PATH").unwrap_or_default();
    let windows = env::split_paths(&paths)
        .filter(|dir| available_path(dir) && dir.join("gemini.cmd").is_file())
        .map(|dir| dir.join("node_modules"))
        .chain(env::var_os("APPDATA").map(PathBuf::from).filter(|dir| available_path(dir))
            .map(|dir| dir.join("npm/node_modules")))
        .find_map(|dir| client_in_modules(&dir));
    windows.or_else(|| {
        wsl_roots().find_map(|root| {
            ["usr/lib/node_modules", "usr/local/lib/node_modules"].into_iter()
                .find_map(|dir| client_in_modules(&root.join(dir)))
                .or_else(|| wsl_homes(&root).find_map(|home| client_in_modules(&home.join(".npm-global/lib/node_modules"))))
                .or_else(|| wsl_homes(&root).find_map(|home| {
                    fs::read_dir(home.join(".nvm/versions/node")).ok()?.flatten()
                        .find_map(|node| client_in_modules(&node.path().join("lib/node_modules")))
                }))
        })
    })
}

#[derive(Default)]
pub(crate) struct Session {
    client: Option<OAuthClient>,
    cached: Option<(LoginKey, Token)>,
}

impl Session {
    fn live_token<'a>(&'a self, login: &'a Login, now: i64) -> Option<&'a Token> {
        if let Some((_, cached)) = self.cached.as_ref()
            .filter(|(key, token)| key == &login.key && token.live(now))
        {
            if !login.token.live(now) || cached.expiry_ms > login.token.expiry_ms {
                return Some(cached);
            }
        }
        login.token.live(now).then_some(&login.token)
    }

    async fn renew(&mut self, http: &reqwest::Client, login: &Login) -> Result<Token, &'static str> {
        if login.key.refresh.is_empty() {
            return Err(EXPIRED);
        }
        if self.client.is_none() {
            self.client = tauri::async_runtime::spawn_blocking(find_client)
                .await.map_err(|_| UNAVAILABLE)?;
        }
        let client = self.client.as_ref().ok_or(NO_CLI)?;
        let response = http.post("https://oauth2.googleapis.com/token")
            .form(&[
                ("grant_type", "refresh_token"),
                ("refresh_token", &login.key.refresh),
                ("client_id", &client.id),
                ("client_secret", &client.secret),
            ])
            .send().await.map_err(|_| UNAVAILABLE)?;
        let status = response.status();
        if !status.is_success() && !matches!(status.as_u16(), 400 | 401) {
            return Err(UNAVAILABLE);
        }
        let document: Value = response.json().await.map_err(|_| UNAVAILABLE)?;
        if !status.is_success() {
            return Err(if document["error"] == "invalid_grant" { EXPIRED } else { UNAVAILABLE });
        }
        let access = nonempty(&document["access_token"]).ok_or(UNAVAILABLE)?;
        let seconds = document["expires_in"].as_i64().filter(|&seconds| seconds > 0).ok_or(UNAVAILABLE)?;
        let now = now_ms();
        let token = Token {
            access: access.to_owned(),
            expiry_ms: now.saturating_add(seconds.saturating_mul(1000)),
        };
        if !token.live(now) {
            return Err(EXPIRED);
        }
        self.cached = Some((LoginKey {
            path: login.key.path.clone(),
            refresh: login.key.refresh.clone(),
        }, token.clone()));
        Ok(token)
    }

    fn forget(&mut self, login: &Login) {
        if self.cached.as_ref().is_some_and(|(key, _)| key == &login.key) {
            self.cached = None;
        }
    }

    async fn query_login(&mut self, http: &reqwest::Client, login: &Login) -> Result<Vec<Limit>, &'static str> {
        let token = match self.live_token(login, now_ms()) {
            Some(token) => token.clone(),
            None => self.renew(http, login).await?,
        };
        let result = query(http, &token.access).await;
        if !matches!(result, Err(EXPIRED)) {
            return result;
        }
        self.forget(login);
        let token = self.renew(http, login).await?;
        let result = query(http, &token.access).await;
        if matches!(result, Err(EXPIRED)) {
            self.forget(login);
        }
        result
    }

    pub(crate) async fn limits(&mut self, http: &reqwest::Client) -> Result<Vec<Limit>, &'static str> {
        let mut logins = tauri::async_runtime::spawn_blocking(discover).await.map_err(|_| UNAVAILABLE)?;
        if self.cached.as_ref().is_some_and(|(key, _)| !logins.iter().any(|login| &login.key == key)) {
            self.cached = None;
        }
        if logins.is_empty() {
            return Err(NO_SESSION);
        }
        let now = now_ms();
        logins.sort_by_key(|login| std::cmp::Reverse((self.live_token(login, now).is_some(), login.written)));
        let mut best_error = EXPIRED;
        for login in &logins {
            match self.query_login(http, login).await {
                Err(UNSUPPORTED) => best_error = UNSUPPORTED,
                Err(NO_QUOTA) if best_error != UNSUPPORTED => best_error = NO_QUOTA,
                Err(NO_CLI) if best_error == EXPIRED => best_error = NO_CLI,
                Err(NO_QUOTA | NO_CLI | EXPIRED) => {}
                result => return result,
            }
        }
        Err(best_error)
    }
}

fn unsupported_tier(document: &Value) -> bool {
    document["currentTier"].is_null()
        && document["ineligibleTiers"].as_array().is_some_and(|tiers| {
            tiers.iter().any(|tier| tier["reasonCode"] == "UNSUPPORTED_CLIENT")
        })
}

fn subscription_required(document: &Value) -> bool {
    let error = &document["error"];
    error["reason"] == "SUBSCRIPTION_REQUIRED"
        || error["status"] == "SUBSCRIPTION_REQUIRED"
        || error["message"].as_str().is_some_and(|message| message.contains("SUBSCRIPTION_REQUIRED"))
        || error["details"].as_array().is_some_and(|details| {
            details.iter().any(|detail| detail["reason"] == "SUBSCRIPTION_REQUIRED")
        })
}

async fn api(http: &reqwest::Client, token: &str, method: &str, body: Value) -> Result<Value, &'static str> {
    let response = http.post(format!("https://cloudcode-pa.googleapis.com/v1internal:{method}"))
        .bearer_auth(token).json(&body).send().await.map_err(|_| UNAVAILABLE)?;
    let status = response.status();
    match status.as_u16() {
        401 => Err(EXPIRED),
        403 => {
            let error: Value = response.json().await.map_err(|_| UNAVAILABLE)?;
            Err(if subscription_required(&error) { UNSUPPORTED } else { UNAVAILABLE })
        }
        _ if status.is_success() => response.json().await.map_err(|_| UNAVAILABLE),
        _ => Err(UNAVAILABLE),
    }
}

async fn query(http: &reqwest::Client, token: &str) -> Result<Vec<Limit>, &'static str> {
    let project = ["GOOGLE_CLOUD_PROJECT", "GOOGLE_CLOUD_PROJECT_ID"].into_iter()
        .filter_map(|key| env::var(key).ok())
        .find(|value| !value.trim().is_empty());
    let mut body = json!({"metadata": {
        "ideType": "IDE_UNSPECIFIED",
        "platform": "PLATFORM_UNSPECIFIED",
        "pluginType": "GEMINI"
    }});
    if let Some(project) = &project {
        body["cloudaicompanionProject"] = json!(project);
        body["metadata"]["duetProject"] = json!(project);
    }
    let account = api(http, token, "loadCodeAssist", body).await?;
    if unsupported_tier(&account) {
        return Err(UNSUPPORTED);
    }
    let companion = &account["cloudaicompanionProject"];
    let project = nonempty(companion).or_else(|| nonempty(&companion["id"]))
        .or(project.as_deref()).ok_or(NO_QUOTA)?;
    let quota = api(http, token, "retrieveUserQuota", json!({"project": project})).await?;
    let limits = parse_limits(&quota);
    if limits.is_empty() { Err(NO_QUOTA) } else { Ok(limits) }
}

fn model_group(model: &str) -> (&str, u8) {
    let lower = model.to_ascii_lowercase();
    let has_family = |family: &str| {
        lower.match_indices(family).any(|(index, _)| {
            let suffix = &lower[index + family.len()..];
            suffix.is_empty() || suffix.starts_with('-')
        })
    };
    if lower.starts_with("gemini-") {
        if has_family("-flash-lite") {
            return ("Gemini Flash Lite", 2);
        }
        if has_family("-flash") {
            return ("Gemini Flash", 1);
        }
        if has_family("-pro") {
            return ("Gemini Pro", 0);
        }
    }
    (model, 3)
}

fn parse_limits(quota: &Value) -> Vec<Limit> {
    let mut groups: Vec<(u8, Limit)> = Vec::new();
    for bucket in quota["buckets"].as_array().into_iter().flatten() {
        let Some(model) = nonempty(&bucket["modelId"]) else { continue };
        let used = match bucket.get("remainingFraction") {
            Some(value) => match value.as_f64() {
                Some(fraction) => (1.0 - fraction.clamp(0.0, 1.0)) * 100.0,
                None => continue,
            },
            None if bucket["remainingAmount"] == "0" => 100.0,
            None => continue,
        };
        let (label, rank) = model_group(model);
        let limit = Limit {
            label: Some(label.to_owned()),
            minutes: None,
            used_percent: used,
            resets_at: bucket["resetTime"].as_str()
                .and_then(|date| chrono::DateTime::parse_from_rfc3339(date).ok())
                .map(|date| date.timestamp()),
        };
        if let Some((_, existing)) = groups.iter_mut().find(|(_, item)| item.label.as_deref() == Some(label)) {
            if existing.used_percent < used {
                *existing = limit;
            }
        } else {
            groups.push((rank, limit));
        }
    }
    groups.sort_by(|(a_rank, a), (b_rank, b)| {
        b.used_percent.total_cmp(&a.used_percent).then_with(|| a_rank.cmp(b_rank))
    });
    groups.into_iter().map(|(_, limit)| limit).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn groups_quota_and_keeps_the_most_restrictive_reset() {
        let limits = parse_limits(&json!({"buckets": [
            {"modelId":"gemini-2.5-flash-lite","remainingFraction":0.0},
            {"modelId":"gemini-2.5-flash","remainingAmount":"0"},
            {"modelId":"gemini-2.5-pro","remainingFraction":0.5,"resetTime":"2000-01-01T00:00:00Z"},
            {"modelId":"gemini-3-pro-preview","remainingFraction":-0.1,"resetTime":"2000-01-02T00:00:00Z"},
            {"modelId":"other-exhausted","remainingAmount":"0"},
            {"modelId":"other-model","remainingFraction":1.2,"resetTime":"inválida"},
            {"modelId":"unknown-amount","remainingAmount":"123"},
            {"modelId":"missing"},
            {"modelId":"malformed","remainingFraction":"no","remainingAmount":"0"},
            {"remainingFraction":0.0}
        ]}));
        let actual: Vec<_> = limits.iter().map(|limit| (limit.label.as_deref(), limit.used_percent)).collect();
        assert_eq!(actual, [
            (Some("Gemini Pro"), 100.0),
            (Some("Gemini Flash"), 100.0),
            (Some("Gemini Flash Lite"), 100.0),
            (Some("other-exhausted"), 100.0),
            (Some("other-model"), 0.0),
        ]);
        assert_eq!(limits[0].resets_at, Some(946_771_200));
        assert_eq!(limits[4].resets_at, None);
        assert!(limits.iter().all(|limit| limit.minutes.is_none()));
        assert_eq!(parse_limits(&json!({"buckets": [
            {"modelId":"gemini-2.5-pro","remainingFraction":0.75}
        ]}))[0].used_percent, 25.0);
    }

    #[test]
    fn detects_unsupported_accounts() {
        let mut account = json!({"ineligibleTiers":[{"reasonCode":"UNSUPPORTED_CLIENT"}]});
        assert!(unsupported_tier(&account));
        account["currentTier"] = json!({"id":"standard"});
        assert!(!unsupported_tier(&account));
        assert!(!unsupported_tier(&json!({"ineligibleTiers":[{"reasonCode":"OTHER"}]})));
        assert!(subscription_required(&json!({"error":{"status":"PERMISSION_DENIED",
            "details":[{"reason":"SUBSCRIPTION_REQUIRED"}]}})));
        assert!(!subscription_required(&json!({"error":{"status":"PERMISSION_DENIED"}})));
    }

    #[test]
    fn extracts_synthetic_constants_in_nested_sibling_and_bundle_layouts() {
        let modules = tempfile::tempdir().unwrap();
        let cli = modules.path().join("@google/gemini-cli");
        for (path, source) in [
            (cli.join("node_modules").join(CORE_OAUTH), "const OAUTH_CLIENT_ID = 'example-id'; const OAUTH_CLIENT_SECRET = \"example-secret\";"),
            (modules.path().join(CORE_OAUTH), "const OAUTH_CLIENT_ID = 'example-id'; const OAUTH_CLIENT_SECRET = \"example-secret\";"),
            (cli.join("bundle/chunk.js"), "var NOT_OAUTH_CLIENT_ID = 'skip'; var OAUTH_CLIENT_ID=\"example-id\", OAUTH_CLIENT_SECRET='example-secret';"),
        ] {
            fs::create_dir_all(path.parent().unwrap()).unwrap();
            fs::write(&path, source).unwrap();
            let client = client_in_modules(modules.path()).unwrap();
            assert!(client.id == "example-id" && client.secret == "example-secret");
            fs::remove_file(path).unwrap();
        }
        assert!(client_in_modules(modules.path()).is_none());
    }

    #[test]
    fn expires_tokens_within_a_minute_and_keys_cache_by_login() {
        let now = 1000;
        let token = |expiry_ms| Token { access: "example-access".into(), expiry_ms };
        assert!(!token(now + 59_999).live(now));
        assert!(!token(now + 60_000).live(now));
        assert!(token(now + 60_001).live(now));

        let key = |path: &str, refresh: &str| LoginKey { path: path.into(), refresh: refresh.into() };
        let session = Session { client: None, cached: Some((key("a", "a"), token(now + 60_001))) };
        let mut login = Login { key: key("a", "a"), token: token(now), written: now };
        assert!(session.live_token(&login, now).is_some());
        login.key = key("a", "b");
        assert!(session.live_token(&login, now).is_none());
        login.key = key("b", "a");
        assert!(session.live_token(&login, now).is_none());
    }
}
