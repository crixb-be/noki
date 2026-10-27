//! Asistente de IA con respuesta en streaming para Gemini, Claude, servicios compatibles con
//! OpenAI y Ollama local. Las llamadas salen desde Rust (no desde el WebView): así no hay
//! problemas de CORS con servidores locales y la clave de API nunca pasa por el frontend.

use crate::{settings::ProviderCfg, NotchState};
use futures_util::StreamExt;
use serde::Deserialize;
use serde_json::{json, Value};
use std::{
    collections::HashMap,
    sync::{
        atomic::{AtomicBool, Ordering::SeqCst},
        Arc, Mutex, OnceLock,
    },
    time::Duration,
};
use tauri::{AppHandle, Emitter, Manager, State};

#[derive(Deserialize, Clone)]
pub struct ChatMsg {
    pub role: String,
    pub content: String,
}

#[derive(Default)]
pub struct AiJobs {
    cancels: Mutex<HashMap<String, Arc<AtomicBool>>>,
}

fn client() -> &'static reqwest::Client {
    static C: OnceLock<reqwest::Client> = OnceLock::new();
    C.get_or_init(|| {
        reqwest::Client::builder()
            .user_agent(concat!("Noki/", env!("CARGO_PKG_VERSION")))
            .connect_timeout(Duration::from_secs(15))
            .build()
            .expect("cliente HTTP")
    })
}

#[derive(Clone, Copy, PartialEq)]
enum Format {
    /// Líneas `data: {...}` (Server-Sent Events).
    Sse,
    /// Una línea JSON por fragmento (Ollama).
    Ndjson,
}

struct Request {
    url: String,
    headers: Vec<(&'static str, String)>,
    body: Value,
    format: Format,
}

fn base(cfg: &ProviderCfg, default: &str) -> String {
    let b = if cfg.base_url.trim().is_empty() { default } else { cfg.base_url.trim() };
    b.trim_end_matches('/').to_string()
}

fn build(provider: &str, cfg: &ProviderCfg, system: &str, messages: &[ChatMsg]) -> Result<Request, String> {
    let needs_key = provider != "ollama";
    if needs_key && cfg.api_key.trim().is_empty() && !(provider == "openai" && !cfg.base_url.trim().is_empty()) {
        return Err("Falta la API key. Agrégala en Ajustes → IA.".into());
    }
    if cfg.model.trim().is_empty() {
        return Err("Falta el nombre del modelo. Agrégalo en Ajustes → IA.".into());
    }
    let model = cfg.model.trim();
    let sys = system.trim();

    Ok(match provider {
        "claude" => {
            let mut body = json!({
                "model": model,
                "max_tokens": 4096,
                "stream": true,
                "messages": messages.iter().map(|m| json!({"role": m.role, "content": m.content})).collect::<Vec<_>>(),
            });
            if !sys.is_empty() {
                body["system"] = json!(sys);
            }
            Request {
                url: format!("{}/v1/messages", base(cfg, "https://api.anthropic.com")),
                headers: vec![("x-api-key", cfg.api_key.trim().into()), ("anthropic-version", "2023-06-01".into())],
                body,
                format: Format::Sse,
            }
        }
        "openai" => {
            let mut msgs = vec![];
            if !sys.is_empty() {
                msgs.push(json!({"role": "system", "content": sys}));
            }
            msgs.extend(messages.iter().map(|m| json!({"role": m.role, "content": m.content})));
            let mut headers = vec![];
            if !cfg.api_key.trim().is_empty() {
                headers.push(("authorization", format!("Bearer {}", cfg.api_key.trim())));
            }
            Request {
                url: format!("{}/chat/completions", base(cfg, "https://api.openai.com/v1")),
                headers,
                body: json!({"model": model, "stream": true, "messages": msgs}),
                format: Format::Sse,
            }
        }
        "ollama" => {
            let mut msgs = vec![];
            if !sys.is_empty() {
                msgs.push(json!({"role": "system", "content": sys}));
            }
            msgs.extend(messages.iter().map(|m| json!({"role": m.role, "content": m.content})));
            Request {
                url: format!("{}/api/chat", base(cfg, "http://localhost:11434")),
                headers: vec![],
                body: json!({"model": model, "stream": true, "messages": msgs}),
                format: Format::Ndjson,
            }
        }
        _ => {
            let contents: Vec<Value> = messages
                .iter()
                .map(|m| {
                    json!({
                        "role": if m.role == "assistant" { "model" } else { "user" },
                        "parts": [{"text": m.content}],
                    })
                })
                .collect();
            let mut body = json!({ "contents": contents });
            if !sys.is_empty() {
                body["systemInstruction"] = json!({"parts": [{"text": sys}]});
            }
            // Chat rápido: respuestas cortas y sin «pensar» antes de contestar (lo que más tarda en los modelos Gemini).
            let mut gen = json!({ "maxOutputTokens": 1024 });
            if let Some(t) = fast_thinking(model) {
                gen["thinkingConfig"] = t;
            }
            body["generationConfig"] = gen;
            Request {
                url: format!(
                    "{}/v1beta/models/{}:streamGenerateContent?alt=sse",
                    base(cfg, "https://generativelanguage.googleapis.com"),
                    model
                ),
                headers: vec![("x-goog-api-key", cfg.api_key.trim().into())],
                body,
                format: Format::Sse,
            }
        }
    })
}

/// Modelos que ya rechazaron la configuración de razonamiento: no se vuelve a enviar.
static NO_THINKING_CFG: Mutex<Vec<String>> = Mutex::new(Vec::new());

/// Configuración para que Gemini responda sin razonar antes. Los 2.5 aceptan `thinkingBudget: 0` (salvo Pro);
/// los de la serie 3 usan `thinkingLevel`. Para otros modelos no se envía nada.
fn fast_thinking(model: &str) -> Option<Value> {
    if NO_THINKING_CFG.lock().unwrap().iter().any(|m| m == model) {
        return None;
    }
    let m = model.to_lowercase();
    let pro = m.contains("pro");
    if m.contains("2.5") {
        (!pro).then(|| json!({ "thinkingBudget": 0 }))
    } else if m.contains("gemini-3") {
        Some(json!({ "thinkingLevel": if pro { "low" } else { "minimal" } }))
    } else {
        None
    }
}

/// Texto de un fragmento ya decodificado, según el proveedor. `Err` si el servidor mandó un error en el flujo.
fn chunk_text(provider: &str, v: &Value) -> Result<String, String> {
    if let Some(e) = v.get("error") {
        let msg = e.get("message").and_then(Value::as_str).or_else(|| e.as_str()).unwrap_or("error del proveedor");
        return Err(msg.to_string());
    }
    let text = match provider {
        "claude" => v
            .pointer("/delta/text")
            .and_then(Value::as_str)
            .filter(|_| v.get("type").and_then(Value::as_str) == Some("content_block_delta")),
        "openai" => v.pointer("/choices/0/delta/content").and_then(Value::as_str),
        "ollama" => v.pointer("/message/content").and_then(Value::as_str),
        _ => None,
    };
    if let Some(t) = text {
        return Ok(t.to_string());
    }
    if provider == "gemini" || provider.is_empty() {
        let parts = v.pointer("/candidates/0/content/parts").and_then(Value::as_array);
        return Ok(parts
            .map(|p| p.iter().filter_map(|x| x.get("text").and_then(Value::as_str)).collect::<String>())
            .unwrap_or_default());
    }
    Ok(String::new())
}

/// Mensaje legible a partir del cuerpo de una respuesta de error HTTP.
fn http_error(status: reqwest::StatusCode, body: &str) -> String {
    let detail = serde_json::from_str::<Value>(body).ok().and_then(|v| {
        v.pointer("/error/message")
            .or_else(|| v.pointer("/error"))
            .and_then(Value::as_str)
            .map(str::to_string)
    });
    match (status.as_u16(), detail) {
        (401 | 403, d) => format!("Clave rechazada ({}){}", status.as_u16(), d.map(|d| format!(": {d}")).unwrap_or_default()),
        (404, d) => format!("No encontrado (404): revisa el modelo o la dirección{}", d.map(|d| format!(". {d}")).unwrap_or_default()),
        (429, _) => "Demasiadas solicitudes o cuota agotada (429). Espera un momento.".into(),
        (_, Some(d)) => format!("Error {}: {d}", status.as_u16()),
        _ => format!("Error {}", status.as_u16()),
    }
}

/// Envía la conversación al proveedor activo y emite `ai-chunk {id, text}` por cada fragmento.
/// Termina con `Ok(())` al completar o al cancelar; con `Err` si falla.
#[tauri::command]
pub async fn ai_chat(
    app: AppHandle,
    jobs: State<'_, AiJobs>,
    id: String,
    messages: Vec<ChatMsg>,
    system: Option<String>,
) -> Result<(), String> {
    let (provider, cfg, default_system) = {
        let st = app.state::<NotchState>();
        let s = st.settings.lock().unwrap();
        (s.ai.provider.clone(), s.ai.active().clone(), s.ai.system_prompt.clone())
    };
    let system = system.unwrap_or(default_system);
    let req = build(&provider, &cfg, &system, &messages)?;

    let cancel = Arc::new(AtomicBool::new(false));
    jobs.cancels.lock().unwrap().insert(id.clone(), cancel.clone());
    let mut result = stream(&app, &id, &provider, req, &cancel).await;
    // Si el modelo no admite la configuración de razonamiento (error 400), se recuerda y se reintenta sin ella.
    if let Err(e) = &result {
        let model = cfg.model.trim().to_string();
        if (provider == "gemini" || provider.is_empty()) && e.starts_with("Error 400") && fast_thinking(&model).is_some() {
            NO_THINKING_CFG.lock().unwrap().push(model);
            let retry = build(&provider, &cfg, &system, &messages)?;
            result = stream(&app, &id, &provider, retry, &cancel).await;
        }
    }
    jobs.cancels.lock().unwrap().remove(&id);
    result
}

async fn stream(app: &AppHandle, id: &str, provider: &str, req: Request, cancel: &AtomicBool) -> Result<(), String> {
    let mut rb = client().post(&req.url).timeout(Duration::from_secs(300)).json(&req.body);
    for (k, v) in &req.headers {
        rb = rb.header(*k, v);
    }
    let resp = rb.send().await.map_err(|e| {
        if e.is_connect() {
            "No se pudo conectar con el servidor de IA. ¿Hay internet (o Ollama abierto)?".to_string()
        } else {
            format!("Error de red: {e}")
        }
    })?;
    let status = resp.status();
    if !status.is_success() {
        let text = resp.text().await.unwrap_or_default();
        return Err(http_error(status, &text));
    }

    let mut bytes = resp.bytes_stream();
    let mut buf: Vec<u8> = Vec::new();
    let mut emit = |text: String| {
        let _ = app.emit("ai-chunk", json!({ "id": id, "text": text }));
    };
    while let Some(next) = bytes.next().await {
        if cancel.load(SeqCst) {
            return Ok(());
        }
        buf.extend_from_slice(&next.map_err(|e| format!("Conexión interrumpida: {e}"))?);
        drain_lines(provider, req.format, &mut buf, &mut emit)?;
    }
    // Si el servidor no terminó la última línea con salto de línea, se procesa igualmente.
    if !buf.is_empty() {
        buf.push(b'\n');
        drain_lines(provider, req.format, &mut buf, &mut emit)?;
    }
    Ok(())
}

/// Procesa solo líneas completas del buffer: un fragmento puede cortar a mitad de un carácter UTF-8.
fn drain_lines(provider: &str, format: Format, buf: &mut Vec<u8>, emit: &mut dyn FnMut(String)) -> Result<(), String> {
    while let Some(pos) = buf.iter().position(|b| *b == b'\n') {
        let line_bytes: Vec<u8> = buf.drain(..=pos).collect();
        let line = String::from_utf8_lossy(&line_bytes);
        let line = line.trim();
        let payload = match format {
            Format::Sse => match line.strip_prefix("data:") {
                Some(d) => d.trim(),
                None => continue,
            },
            Format::Ndjson => line,
        };
        if payload.is_empty() || payload == "[DONE]" {
            continue;
        }
        let Ok(v) = serde_json::from_str::<Value>(payload) else { continue };
        let text = chunk_text(provider, &v)?;
        if !text.is_empty() {
            emit(text);
        }
    }
    Ok(())
}

#[tauri::command]
pub fn ai_cancel(jobs: State<'_, AiJobs>, id: String) {
    if let Some(c) = jobs.cancels.lock().unwrap().get(&id) {
        c.store(true, SeqCst);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Alimenta `drain_lines` con trozos de bytes (posiblemente cortados a mitad de carácter) y junta el texto.
    fn run(provider: &str, format: Format, chunks: &[&[u8]]) -> Result<String, String> {
        let mut buf = Vec::new();
        let mut out = String::new();
        for c in chunks {
            buf.extend_from_slice(c);
            drain_lines(provider, format, &mut buf, &mut |t| out.push_str(&t))?;
        }
        Ok(out)
    }

    #[test]
    fn openai_sse_with_utf8_split_across_chunks() {
        let a = "data: {\"choices\":[{\"delta\":{\"content\":\"¡Hola ñandú \"}}]}

".as_bytes();
        let b = "data: {\"choices\":[{\"delta\":{\"content\":\"🙂\"}}]}

data: [DONE]

".as_bytes();
        // Se corta a propósito dentro de «ñ» y dentro del emoji.
        let cut_a = a.iter().position(|x| *x == 0xC3).unwrap() + 1;
        let cut_b = b.iter().position(|x| *x == 0xF0).unwrap() + 2;
        let r = run("openai", Format::Sse, &[&a[..cut_a], &a[cut_a..], &b[..cut_b], &b[cut_b..]]).unwrap();
        assert_eq!(r, "¡Hola ñandú 🙂");
    }

    #[test]
    fn ollama_ndjson_and_last_line_without_newline() {
        let r = run(
            "ollama",
            Format::Ndjson,
            &[b"{\"message\":{\"content\":\"Hola\"},\"done\":false}
{\"message\":{\"content\":\" mundo\"},", b"\"done\":false}
"],
        )
        .unwrap();
        assert_eq!(r, "Hola mundo");
        // Última línea sin salto de línea: stream() lo añade antes de procesarla.
        let mut buf = b"{\"message\":{\"content\":\"fin\"},\"done\":true}".to_vec();
        buf.push(b'\n');
        let mut out = String::new();
        drain_lines("ollama", Format::Ndjson, &mut buf, &mut |t| out.push_str(&t)).unwrap();
        assert_eq!(out, "fin");
    }

    #[test]
    fn claude_sse_only_text_deltas() {
        let body = "event: message_start
data: {\"type\":\"message_start\",\"message\":{}}

                    event: content_block_delta
data: {\"type\":\"content_block_delta\",\"index\":0,\"delta\":{\"type\":\"text_delta\",\"text\":\"Buen\"}}

                    event: content_block_delta
data: {\"type\":\"content_block_delta\",\"index\":0,\"delta\":{\"type\":\"text_delta\",\"text\":\"os días\"}}

                    event: message_stop
data: {\"type\":\"message_stop\"}

";
        assert_eq!(run("claude", Format::Sse, &[body.as_bytes()]).unwrap(), "Buenos días");
    }

    #[test]
    fn gemini_sse_joins_parts() {
        let body = "data: {\"candidates\":[{\"content\":{\"parts\":[{\"text\":\"Hola\"}],\"role\":\"model\"}}]}

                    data: {\"candidates\":[{\"content\":{\"parts\":[{\"text\":\" a \"},{\"text\":\"todos\"}]},\"finishReason\":\"STOP\"}]}

";
        assert_eq!(run("gemini", Format::Sse, &[body.as_bytes()]).unwrap(), "Hola a todos");
    }

    #[test]
    fn errors_inside_stream_and_http_errors_are_reported() {
        let err = run("claude", Format::Sse, &[b"data: {\"type\":\"error\",\"error\":{\"message\":\"Overloaded\"}}
"]).unwrap_err();
        assert_eq!(err, "Overloaded");
        let msg = http_error(reqwest::StatusCode::UNAUTHORIZED, "{\"error\":{\"message\":\"bad key\"}}");
        assert!(msg.contains("Clave rechazada") && msg.contains("bad key"));
        assert!(http_error(reqwest::StatusCode::TOO_MANY_REQUESTS, "").contains("429"));
    }

    #[test]
    fn request_building_per_provider() {
        let cfg = ProviderCfg { api_key: "k".into(), model: "m".into(), base_url: String::new() };
        let msgs = vec![ChatMsg { role: "user".into(), content: "hola".into() }, ChatMsg { role: "assistant".into(), content: "qué tal".into() }];
        let g = build("gemini", &cfg, "sé breve", &msgs).unwrap();
        assert!(g.url.ends_with("/v1beta/models/m:streamGenerateContent?alt=sse"));
        assert_eq!(g.body["contents"][1]["role"], "model");
        assert_eq!(g.body["systemInstruction"]["parts"][0]["text"], "sé breve");
        let c = build("claude", &cfg, "x", &msgs).unwrap();
        assert!(c.url.ends_with("/v1/messages") && c.body["system"] == "x" && c.body["stream"] == true);
        let o = build("openai", &cfg, "x", &msgs).unwrap();
        assert_eq!(o.body["messages"][0]["role"], "system");
        assert!(o.url.starts_with("https://api.openai.com/v1/"));
        let l = build("ollama", &ProviderCfg { model: "llama3.2".into(), ..Default::default() }, "", &msgs).unwrap();
        assert_eq!(l.url, "http://localhost:11434/api/chat");
        assert!(build("gemini", &ProviderCfg { model: "m".into(), ..Default::default() }, "", &msgs).is_err());
    }
}
