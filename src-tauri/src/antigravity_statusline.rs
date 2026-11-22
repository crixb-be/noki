use crate::ai_usage::Limit;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{
    fs::{self, File},
    io::{self, Read, Write},
    path::{Path, PathBuf},
};

const MAX_INPUT_BYTES: usize = 1024 * 1024;
const MAX_CACHE_BYTES: usize = 64 * 1024;
const NOT_CONFIGURED: &str = "Configura Antigravity CLI en Ajustes → Barra superior para ver la cuota de Gemini.";
const EXPIRED: &str = "La cuota de Antigravity CLI ha caducado. Abre agy para actualizarla.";
const COMMAND_ERROR: &str = "La ruta de Noki no es compatible con el comando de Antigravity CLI.";

#[derive(Deserialize)]
struct StatuslineInput {
    quota: serde_json::Map<String, Value>,
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct Capture {
    captured_at: i64,
    buckets: Vec<Bucket>,
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct Bucket {
    id: String,
    used_percent: f64,
    resets_at: Option<i64>,
}

fn cache_path() -> Option<PathBuf> {
    let local = PathBuf::from(std::env::var_os("LOCALAPPDATA")?);
    local.is_absolute().then(|| local.join("com.noki.app/antigravity-quota.json"))
}

fn read_bounded(reader: impl Read, max_bytes: usize) -> io::Result<Vec<u8>> {
    let mut bytes = Vec::new();
    reader.take(max_bytes as u64 + 1).read_to_end(&mut bytes)?;
    if bytes.len() > max_bytes {
        return Err(io::ErrorKind::InvalidData.into());
    }
    Ok(bytes)
}

fn gemini_id(id: &str) -> bool {
    id.starts_with("gemini") && !id.chars().any(char::is_control)
}

fn number(value: &Value) -> Option<f64> {
    value.as_f64()
        .or_else(|| value.as_str()?.trim().parse().ok())
        .filter(|number| number.is_finite())
}

fn valid_timestamp(seconds: i64) -> bool {
    DateTime::from_timestamp(seconds, 0).is_some()
}

fn parse_input(bytes: &[u8], now: i64) -> Option<Capture> {
    let input: StatuslineInput = serde_json::from_slice(bytes).ok()?;
    let buckets: Vec<_> = input.quota.into_iter().filter_map(|(id, quota)| {
        if !gemini_id(&id) || quota["disabled"].as_bool() == Some(true) {
            return None;
        }
        let fraction = &quota["remaining_fraction"];
        let used_percent = if fraction.is_null() {
            (number(&quota["remaining_amount"])? == 0.0).then_some(100.0)?
        } else {
            (1.0 - number(fraction)?).clamp(0.0, 1.0) * 100.0
        };
        let resets_at = quota["reset_time"].as_str()
            .and_then(|time| DateTime::parse_from_rfc3339(time).ok())
            .map(|time| time.timestamp())
            .or_else(|| {
                let seconds = quota["reset_in_seconds"].as_i64()
                    .or_else(|| quota["reset_in_seconds"].as_str()?.trim().parse().ok())?;
                now.checked_add(seconds)
            })
            .filter(|&time| valid_timestamp(time));
        Some(Bucket { id, used_percent, resets_at })
    }).collect();
    (!buckets.is_empty()).then_some(Capture { captured_at: now, buckets })
}

fn save_capture(path: &Path, capture: &Capture) -> io::Result<()> {
    let dir = path.parent().ok_or(io::ErrorKind::InvalidInput)?;
    fs::create_dir_all(dir)?;
    let mut file = tempfile::NamedTempFile::new_in(dir)?;
    serde_json::to_writer(&mut file, capture)?;
    file.persist(path).map_err(|error| error.error)?;
    Ok(())
}

fn window_minutes(id: &str) -> Option<u64> {
    if id.contains("5h") {
        Some(300)
    } else if id.contains("weekly") || id.contains("7d") {
        Some(10080)
    } else {
        None
    }
}

fn statusline(capture: &Capture) -> String {
    let mut line = String::from("Noki");
    for (minutes, label) in [(300, "Gemini 5h"), (10080, "7d")] {
        if let Some(bucket) = capture.buckets.iter().find(|bucket| window_minutes(&bucket.id) == Some(minutes)) {
            line.push_str(&format!(" · {label} {:.0}%", bucket.used_percent));
        }
    }
    if line == "Noki" {
        if let Some(bucket) = capture.buckets.first() {
            line.push_str(&format!(" · Gemini {:.0}%", bucket.used_percent));
        }
    }
    line
}

fn capture_statusline(input: impl Read, path: Option<&Path>, now: i64) -> String {
    let capture = read_bounded(input, MAX_INPUT_BYTES).ok()
        .and_then(|bytes| parse_input(&bytes, now));
    if let Some(capture) = capture {
        if let Some(path) = path {
            let _ = save_capture(path, &capture);
        }
        statusline(&capture)
    } else {
        String::from("Noki")
    }
}

pub fn run_antigravity_statusline() {
    let line = capture_statusline(io::stdin().lock(), cache_path().as_deref(), Utc::now().timestamp());
    let _ = writeln!(io::stdout().lock(), "{line}");
}

fn read_capture(path: &Path, now: i64) -> Result<(Vec<Limit>, i64), &'static str> {
    let file = File::open(path).map_err(|_| NOT_CONFIGURED)?;
    let bytes = read_bounded(file, MAX_CACHE_BYTES).map_err(|_| NOT_CONFIGURED)?;
    let capture: Capture = serde_json::from_slice(&bytes).map_err(|_| NOT_CONFIGURED)?;
    if !valid_timestamp(capture.captured_at) || capture.captured_at > now || capture.buckets.is_empty() {
        return Err(NOT_CONFIGURED);
    }
    let mut limits = Vec::new();
    for bucket in capture.buckets {
        if !gemini_id(&bucket.id)
            || !bucket.used_percent.is_finite()
            || !(0.0..=100.0).contains(&bucket.used_percent)
            || bucket.resets_at.is_some_and(|reset| !valid_timestamp(reset))
        {
            return Err(NOT_CONFIGURED);
        }
        if bucket.resets_at.unwrap_or(capture.captured_at + 86400) <= now {
            continue;
        }
        let minutes = window_minutes(&bucket.id);
        limits.push(Limit {
            minutes,
            used_percent: bucket.used_percent,
            resets_at: bucket.resets_at,
            label: minutes.is_none().then_some(bucket.id),
        });
    }
    if limits.is_empty() {
        return Err(EXPIRED);
    }
    limits.sort_by(|a, b| {
        a.minutes.unwrap_or(u64::MAX).cmp(&b.minutes.unwrap_or(u64::MAX))
            .then_with(|| b.used_percent.total_cmp(&a.used_percent))
    });
    Ok((limits, capture.captured_at))
}

pub(crate) async fn limits() -> Result<(Vec<Limit>, i64), &'static str> {
    tauri::async_runtime::spawn_blocking(|| {
        let path = cache_path().ok_or(NOT_CONFIGURED)?;
        read_capture(&path, Utc::now().timestamp())
    }).await.map_err(|_| NOT_CONFIGURED)?
}

fn escape_cmd(text: &str) -> String {
    let mut escaped = String::with_capacity(text.len());
    for ch in text.chars() {
        if matches!(ch, '&' | '(' | ')' | '^' | '|' | '<' | '>') {
            escaped.push('^');
        }
        escaped.push(ch);
    }
    escaped
}

fn statusline_command(path: &str) -> Result<String, &'static str> {
    let path = path.strip_prefix(r"\\?\").unwrap_or(path);
    if path.contains(['%', '"', '\r', '\n'])
        || !matches!(path.as_bytes(), [b'A'..=b'Z' | b'a'..=b'z', b':', b'\\', ..])
    {
        return Err(COMMAND_ERROR);
    }
    let path = Path::new(path);
    let dir = path.parent().and_then(Path::to_str).ok_or(COMMAND_ERROR)?;
    let file = path.file_name().and_then(|name| name.to_str()).ok_or(COMMAND_ERROR)?;
    Ok(format!("/statusline cd /d {}&&.\\{} --antigravity-statusline", escape_cmd(dir), escape_cmd(file)))
}

#[tauri::command]
pub fn antigravity_statusline_command() -> Result<String, &'static str> {
    let exe = std::env::current_exe().map_err(|_| COMMAND_ERROR)?;
    statusline_command(exe.to_str().ok_or(COMMAND_ERROR)?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    const NOW: i64 = 1_783_000_000;

    #[test]
    fn hook_saves_only_valid_gemini_quota_fields() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("noki/antigravity-quota.json");
        let input = json!({
            "email": "synthetic@example.invalid",
            "cwd": "synthetic-home",
            "transcript_path": "synthetic-transcript",
            "conversation_title": "synthetic-title",
            "quota": {
                "gemini-5h": {"remaining_fraction": "0.75", "reset_in_seconds": 120},
                "gemini-weekly": {"remaining_fraction": 0.9375, "reset_time": "2026-07-06T07:50:32Z", "reset_in_seconds": 30},
                "gemini-empty": {"remaining_amount": "0"},
                "gemini-lower": {"remaining_fraction": -0.25},
                "gemini-upper": {"remaining_fraction": 1.25},
                "gemini-positive": {"remaining_amount": 12},
                "gemini-disabled": {"remaining_fraction": 0, "disabled": true},
                "gemini-invalid": {"remaining_fraction": "NaN", "remaining_amount": 0},
                "3p-5h": {"remaining_fraction": 0.5}
            }
        }).to_string();
        assert_eq!(capture_statusline(input.as_bytes(), Some(&path), NOW), "Noki · Gemini 5h 25% · 7d 6%");
        let saved = fs::read(&path).unwrap();
        let weekly_reset = DateTime::parse_from_rfc3339("2026-07-06T07:50:32Z").unwrap().timestamp();
        assert_eq!(serde_json::from_slice::<Value>(&saved).unwrap(), json!({
            "capturedAt": NOW,
            "buckets": [
                {"id": "gemini-5h", "usedPercent": 25.0, "resetsAt": NOW + 120},
                {"id": "gemini-empty", "usedPercent": 100.0, "resetsAt": null},
                {"id": "gemini-lower", "usedPercent": 100.0, "resetsAt": null},
                {"id": "gemini-upper", "usedPercent": 0.0, "resetsAt": null},
                {"id": "gemini-weekly", "usedPercent": 6.25, "resetsAt": weekly_reset}
            ]
        }));
        for invalid in [r#"{"quota":{}}"#, "{", ""] {
            assert_eq!(capture_statusline(invalid.as_bytes(), Some(&path), NOW + 1), "Noki");
            assert_eq!(fs::read(&path).unwrap(), saved);
        }
        let oversized = vec![b' '; MAX_INPUT_BYTES + 1];
        assert_eq!(capture_statusline(&oversized[..], Some(&path), NOW + 1), "Noki");
        assert_eq!(fs::read(&path).unwrap(), saved);
    }

    #[test]
    fn reader_orders_windows_and_drops_expired_buckets() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("antigravity-quota.json");
        let input = br#"{"quota":{
            "gemini-5h":{"remaining_fraction":0.75,"reset_in_seconds":30},
            "gemini-7d":{"remaining_fraction":0.5, "reset_in_seconds":86401},
            "gemini-extra-a":{"remaining_fraction":0.875},
            "gemini-extra-b":{"remaining_fraction":0.125}
        }}"#;
        capture_statusline(&input[..], Some(&path), NOW);
        let (limits, captured_at) = read_capture(&path, NOW).ok().unwrap();
        assert_eq!(captured_at, NOW);
        assert_eq!(limits.iter().map(|limit| limit.minutes).collect::<Vec<_>>(), [Some(300), Some(10080), None, None]);
        assert_eq!(limits.iter().map(|limit| limit.label.as_deref()).collect::<Vec<_>>(), [None, None, Some("gemini-extra-b"), Some("gemini-extra-a")]);
        let (limits, _) = read_capture(&path, NOW + 30).ok().unwrap();
        assert_eq!(limits.len(), 3);
        let (limits, _) = read_capture(&path, NOW + 86400).ok().unwrap();
        assert_eq!(limits.len(), 1);
        assert_eq!(limits[0].minutes, Some(10080));
        assert_eq!(read_capture(&path, NOW + 86401).err(), Some(EXPIRED));
    }

    #[test]
    fn reader_rejects_missing_oversized_and_invalid_cache() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("antigravity-quota.json");
        assert_eq!(read_capture(&path, NOW).err(), Some(NOT_CONFIGURED));
        fs::write(&path, vec![b' '; MAX_CACHE_BYTES + 1]).unwrap();
        assert_eq!(read_capture(&path, NOW).err(), Some(NOT_CONFIGURED));
        for (captured_at, percent, reset, id) in [
            (NOW + 1, 50.0, NOW + 120, "gemini-5h"),
            (NOW, 101.0, NOW + 120, "gemini-5h"),
            (NOW, 50.0, i64::MAX, "gemini-5h"),
            (NOW, 50.0, NOW + 120, "3p-5h"),
        ] {
            fs::write(&path, json!({
                "capturedAt": captured_at,
                "buckets": [{"id": id, "usedPercent": percent, "resetsAt": reset}]
            }).to_string()).unwrap();
            assert_eq!(read_capture(&path, NOW).err(), Some(NOT_CONFIGURED));
        }
    }

    #[test]
    fn command_preserves_spaces_and_escapes_cmd_metacharacters() {
        assert_eq!(
            statusline_command(r"\\?\C:\Aplicación &()^|<>\Noki &()^|<>.exe").unwrap(),
            r"/statusline cd /d C:\Aplicación ^&^(^)^^^|^<^>&&.\Noki ^&^(^)^^^|^<^>.exe --antigravity-statusline"
        );
        assert_eq!(
            statusline_command(r"D:\noki.exe").unwrap(),
            r"/statusline cd /d D:\&&.\noki.exe --antigravity-statusline"
        );
        for path in [
            r"\\server\folder\noki.exe",
            r"\\?\UNC\server\folder\noki.exe",
            r"C:\100%\noki.exe",
            r#"C:\folder\no"ki.exe"#,
            "C:\\bad\r\\noki.exe",
            "C:\\bad\n\\noki.exe",
        ] {
            assert_eq!(statusline_command(path), Err(COMMAND_ERROR));
        }
    }
}
