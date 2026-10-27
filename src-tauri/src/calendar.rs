//! Calendario por iCal (.ics): descarga las direcciones configuradas, lee los eventos
//! (incluidas repeticiones RRULE, excepciones y ocurrencias modificadas) y devuelve los próximos días.
//!
//! Funciona con la «dirección secreta en formato iCal» de Google Calendar, el enlace ICS publicado de
//! Outlook/Microsoft 365 y cualquier otro .ics. Las horas con zona horaria (TZID) se interpretan en la
//! zona horaria de este equipo.

use crate::NotchState;
use chrono::{DateTime, Datelike, Duration, Local, NaiveDate, NaiveDateTime, TimeZone, Utc, Weekday};
use serde::Serialize;
use std::{
    collections::{HashMap, HashSet},
    time::Duration as StdDuration,
};
use tauri::{AppHandle, Manager};

const DAYS_AHEAD: i64 = 62;
const MAX_BYTES: usize = 15 * 1024 * 1024;

#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Event {
    pub id: String,
    pub title: String,
    /// Milisegundos desde 1970.
    pub start: i64,
    pub end: i64,
    pub all_day: bool,
    pub location: String,
    pub calendar: String,
}

#[derive(Serialize, Default)]
pub struct CalResult {
    pub events: Vec<Event>,
    pub errors: Vec<String>,
}

// ───────────────────────────── lectura de propiedades ─────────────────────────────

struct Prop {
    name: String,
    params: HashMap<String, String>,
    value: String,
}

/// Une las líneas plegadas (las que empiezan con espacio o tabulador continúan la anterior).
fn unfold(text: &str) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for raw in text.split('\n') {
        let line = raw.trim_end_matches('\r');
        if (line.starts_with(' ') || line.starts_with('\t')) && !out.is_empty() {
            out.last_mut().unwrap().push_str(&line[1..]);
        } else {
            out.push(line.to_string());
        }
    }
    out
}

fn parse_prop(line: &str) -> Option<Prop> {
    // Los dos puntos que separan nombre y valor no pueden estar dentro de comillas (parámetros).
    let mut in_q = false;
    let mut split = None;
    for (i, c) in line.char_indices() {
        match c {
            '"' => in_q = !in_q,
            ':' if !in_q => {
                split = Some(i);
                break;
            }
            _ => {}
        }
    }
    let i = split?;
    let (left, value) = (&line[..i], &line[i + 1..]);
    let mut parts = left.split(';');
    let name = parts.next()?.to_uppercase();
    let params = parts
        .filter_map(|p| p.split_once('='))
        .map(|(k, v)| (k.to_uppercase(), v.trim_matches('"').to_string()))
        .collect();
    Some(Prop { name, params, value: unescape(value) })
}

fn unescape(s: &str) -> String {
    s.replace("\\n", "\n").replace("\\N", "\n").replace("\\,", ",").replace("\\;", ";").replace("\\\\", "\\")
}

// ───────────────────────────── fechas ─────────────────────────────

#[derive(Clone, Copy, Debug)]
struct Dt {
    /// Hora de pared local; las repeticiones se calculan sobre ella.
    local: NaiveDateTime,
    all_day: bool,
}

fn to_ms(local: NaiveDateTime) -> i64 {
    Local
        .from_local_datetime(&local)
        .earliest()
        .map(|d| d.timestamp_millis())
        .unwrap_or_else(|| local.and_utc().timestamp_millis())
}

fn parse_dt(value: &str, params: &HashMap<String, String>) -> Option<Dt> {
    let v = value.trim();
    let all_day = params.get("VALUE").map(|s| s == "DATE").unwrap_or(false) || v.len() == 8;
    if all_day {
        let d = NaiveDate::parse_from_str(v.get(..8)?, "%Y%m%d").ok()?;
        return Some(Dt { local: d.and_hms_opt(0, 0, 0)?, all_day: true });
    }
    if let Some(utc) = v.strip_suffix('Z') {
        let n = NaiveDateTime::parse_from_str(utc, "%Y%m%dT%H%M%S").ok()?;
        return Some(Dt { local: Utc.from_utc_datetime(&n).with_timezone(&Local).naive_local(), all_day: false });
    }
    let n = NaiveDateTime::parse_from_str(v, "%Y%m%dT%H%M%S").ok()?;
    Some(Dt { local: n, all_day: false })
}

/// P1D, PT1H30M, P1W… en milisegundos.
fn parse_duration(v: &str) -> Option<i64> {
    let v = v.trim();
    let (neg, v) = match v.strip_prefix('-') {
        Some(r) => (true, r),
        None => (false, v.strip_prefix('+').unwrap_or(v)),
    };
    let v = v.strip_prefix('P')?;
    let mut total = 0i64;
    let mut num = String::new();
    let mut in_time = false;
    for c in v.chars() {
        match c {
            'T' => in_time = true,
            '0'..='9' => num.push(c),
            _ => {
                let n: i64 = num.parse().ok()?;
                num.clear();
                total += n * match (c, in_time) {
                    ('W', _) => 7 * 86400,
                    ('D', _) => 86400,
                    ('H', true) => 3600,
                    ('M', true) => 60,
                    ('S', true) => 1,
                    _ => return None,
                };
            }
        }
    }
    Some(if neg { -total } else { total } * 1000)
}

// ───────────────────────────── eventos y repeticiones ─────────────────────────────

#[derive(Default)]
struct Raw {
    uid: String,
    summary: String,
    location: String,
    start: Option<Dt>,
    end: Option<Dt>,
    duration_ms: Option<i64>,
    rrule: Option<String>,
    exdates: HashSet<i64>,
    recurrence_id: Option<i64>,
    cancelled: bool,
}

struct Rule {
    freq: String,
    interval: i64,
    count: Option<u32>,
    until: Option<NaiveDateTime>,
    byday: Vec<(Option<i32>, Weekday)>,
    bymonthday: Vec<i32>,
}

fn weekday(code: &str) -> Option<Weekday> {
    Some(match code {
        "MO" => Weekday::Mon,
        "TU" => Weekday::Tue,
        "WE" => Weekday::Wed,
        "TH" => Weekday::Thu,
        "FR" => Weekday::Fri,
        "SA" => Weekday::Sat,
        "SU" => Weekday::Sun,
        _ => return None,
    })
}

fn parse_rule(s: &str) -> Option<Rule> {
    let mut r = Rule { freq: String::new(), interval: 1, count: None, until: None, byday: vec![], bymonthday: vec![] };
    for kv in s.split(';') {
        let (k, v) = kv.split_once('=')?;
        match k.to_uppercase().as_str() {
            "FREQ" => r.freq = v.to_uppercase(),
            "INTERVAL" => r.interval = v.parse::<i64>().ok()?.max(1),
            "COUNT" => r.count = v.parse().ok(),
            "UNTIL" => {
                let dt = parse_dt(v, &HashMap::new())?;
                // UNTIL con fecha sola incluye ese día completo.
                r.until = Some(if dt.all_day { dt.local + Duration::hours(23) + Duration::minutes(59) } else { dt.local });
            }
            "BYDAY" => {
                for d in v.split(',') {
                    let d = d.trim();
                    if d.len() < 2 {
                        continue;
                    }
                    let (num, code) = d.split_at(d.len() - 2);
                    r.byday.push((num.parse::<i32>().ok(), weekday(&code.to_uppercase())?));
                }
            }
            "BYMONTHDAY" => r.bymonthday = v.split(',').filter_map(|x| x.parse().ok()).collect(),
            _ => {}
        }
    }
    matches!(r.freq.as_str(), "DAILY" | "WEEKLY" | "MONTHLY" | "YEARLY").then_some(r)
}

fn nth_weekday(year: i32, month: u32, wd: Weekday, n: i32) -> Option<NaiveDate> {
    if n > 0 {
        let first = NaiveDate::from_ymd_opt(year, month, 1)?;
        let offset = (wd.num_days_from_monday() + 7 - first.weekday().num_days_from_monday()) % 7;
        NaiveDate::from_ymd_opt(year, month, 1 + offset + 7 * (n as u32 - 1))
    } else {
        let (ny, nm) = if month == 12 { (year + 1, 1) } else { (year, month + 1) };
        let last = NaiveDate::from_ymd_opt(ny, nm, 1)? - Duration::days(1);
        let offset = (last.weekday().num_days_from_monday() + 7 - wd.num_days_from_monday()) % 7;
        NaiveDate::from_ymd_opt(year, month, last.day() - offset - 7 * ((-n) as u32 - 1))
    }
}

fn days_in_month(year: i32, month: u32) -> u32 {
    let (ny, nm) = if month == 12 { (year + 1, 1) } else { (year, month + 1) };
    (NaiveDate::from_ymd_opt(ny, nm, 1).unwrap() - NaiveDate::from_ymd_opt(year, month, 1).unwrap()).num_days() as u32
}

struct Acc<'a> {
    start: NaiveDateTime,
    limit: NaiveDateTime,
    rule: &'a Rule,
    out: Vec<NaiveDateTime>,
    yielded: u32,
    done: bool,
}

impl Acc<'_> {
    /// Añade una ocurrencia; devuelve `false` si ya no hay que seguir generando.
    fn push(&mut self, dt: NaiveDateTime) -> bool {
        if dt < self.start {
            return true;
        }
        if dt > self.limit || self.rule.until.map(|u| dt > u).unwrap_or(false) {
            self.done = true;
            return false;
        }
        self.out.push(dt);
        self.yielded += 1;
        if self.rule.count.map(|c| self.yielded >= c).unwrap_or(false) {
            self.done = true;
            return false;
        }
        true
    }
}

/// Ocurrencias (hora local de pared, ascendentes) desde `start` hasta `limit`, respetando COUNT/UNTIL.
fn occurrences(start: NaiveDateTime, rule: &Rule, limit: NaiveDateTime) -> Vec<NaiveDateTime> {
    let time = start.time();
    let mut acc = Acc { start, limit, rule, out: Vec::new(), yielded: 0, done: false };

    for k in 0..10_000i64 {
        if acc.done {
            break;
        }
        match rule.freq.as_str() {
            "DAILY" => {
                acc.push(start + Duration::days(k * rule.interval));
            }
            "WEEKLY" => {
                let monday = start.date() - Duration::days(start.weekday().num_days_from_monday() as i64);
                let week = monday + Duration::days(7 * rule.interval * k);
                let mut days: Vec<u32> = if rule.byday.is_empty() {
                    vec![start.weekday().num_days_from_monday()]
                } else {
                    rule.byday.iter().map(|(_, w)| w.num_days_from_monday()).collect()
                };
                days.sort_unstable();
                days.dedup();
                for d in days {
                    if !acc.push((week + Duration::days(d as i64)).and_time(time)) {
                        break;
                    }
                }
            }
            "MONTHLY" => {
                let total = start.year() as i64 * 12 + (start.month() as i64 - 1) + k * rule.interval;
                let (y, m) = ((total / 12) as i32, (total % 12) as u32 + 1);
                let mut dates: Vec<NaiveDate> = Vec::new();
                if !rule.byday.is_empty() {
                    for (ord, wd) in &rule.byday {
                        match ord {
                            Some(n) if *n != 0 => dates.extend(nth_weekday(y, m, *wd, *n)),
                            _ => {
                                for n in 1..=5 {
                                    dates.extend(nth_weekday(y, m, *wd, n));
                                }
                            }
                        }
                    }
                } else {
                    let dim = days_in_month(y, m) as i32;
                    let days = if rule.bymonthday.is_empty() { vec![start.day() as i32] } else { rule.bymonthday.clone() };
                    for d in days {
                        let day = if d < 0 { dim + d + 1 } else { d };
                        if day >= 1 && day <= dim {
                            dates.extend(NaiveDate::from_ymd_opt(y, m, day as u32));
                        }
                    }
                }
                dates.sort_unstable();
                dates.dedup();
                for d in dates {
                    if !acc.push(d.and_time(time)) {
                        break;
                    }
                }
            }
            _ => {
                // YEARLY: mismo día y mes cada `interval` años (el 29 de febrero se salta los años no bisiestos).
                let y = start.year() + (k * rule.interval) as i32;
                if let Some(d) = NaiveDate::from_ymd_opt(y, start.month(), start.day()) {
                    acc.push(d.and_time(time));
                }
            }
        }
    }
    acc.out
}

fn parse_events(text: &str, calendar: &str, now: DateTime<Local>) -> Vec<Event> {
    let lines = unfold(text);
    let mut raws: Vec<Raw> = Vec::new();
    let mut cur: Option<Raw> = None;
    let mut depth = 0; // componentes anidados dentro del VEVENT (p. ej. VALARM)

    for line in &lines {
        let up = line.to_uppercase();
        if up == "BEGIN:VEVENT" {
            cur = Some(Raw::default());
            depth = 0;
            continue;
        }
        if up == "END:VEVENT" {
            if let Some(r) = cur.take() {
                raws.push(r);
            }
            continue;
        }
        let Some(raw) = cur.as_mut() else { continue };
        if up.starts_with("BEGIN:") {
            depth += 1;
            continue;
        }
        if up.starts_with("END:") {
            depth -= 1;
            continue;
        }
        if depth > 0 {
            continue;
        }
        let Some(p) = parse_prop(line) else { continue };
        match p.name.as_str() {
            "UID" => raw.uid = p.value,
            "SUMMARY" => raw.summary = p.value,
            "LOCATION" => raw.location = p.value,
            "STATUS" => raw.cancelled = p.value.eq_ignore_ascii_case("CANCELLED"),
            "DTSTART" => raw.start = parse_dt(&p.value, &p.params),
            "DTEND" => raw.end = parse_dt(&p.value, &p.params),
            "DURATION" => raw.duration_ms = parse_duration(&p.value),
            "RRULE" => raw.rrule = Some(p.value),
            "EXDATE" => {
                for v in p.value.split(',') {
                    if let Some(d) = parse_dt(v, &p.params) {
                        raw.exdates.insert(to_ms(d.local));
                    }
                }
            }
            "RECURRENCE-ID" => raw.recurrence_id = parse_dt(&p.value, &p.params).map(|d| to_ms(d.local)),
            _ => {}
        }
    }

    let win_start = now - Duration::days(1);
    let win_end = now + Duration::days(DAYS_AHEAD);
    let (ws, we) = (win_start.timestamp_millis(), win_end.timestamp_millis());
    let win_end_local = win_end.naive_local();

    // Instancias reemplazadas: (uid, inicio original) → no se generan desde la regla.
    let overridden: HashSet<(String, i64)> = raws
        .iter()
        .filter_map(|r| r.recurrence_id.map(|id| (r.uid.clone(), id)))
        .collect();

    let mut events = Vec::new();
    for r in &raws {
        if r.cancelled {
            continue;
        }
        let Some(start) = r.start else { continue };
        let start_ms = to_ms(start.local);
        let dur_ms = match (r.end, r.duration_ms) {
            (Some(e), _) => (to_ms(e.local) - start_ms).max(0),
            (None, Some(d)) => d,
            (None, None) => if start.all_day { 86_400_000 } else { 0 },
        };
        let title = if r.summary.trim().is_empty() { "(Sin título)".to_string() } else { r.summary.trim().to_string() };

        let starts: Vec<i64> = match r.rrule.as_deref().and_then(parse_rule) {
            Some(rule) if r.recurrence_id.is_none() => occurrences(start.local, &rule, win_end_local)
                .into_iter()
                .map(to_ms)
                .filter(|ms| !r.exdates.contains(ms) && !overridden.contains(&(r.uid.clone(), *ms)))
                .collect(),
            _ => vec![start_ms],
        };

        for s in starts {
            let e = s + dur_ms;
            // Se conservan los eventos que se solapan con la ventana; los de duración 0 cuentan por su inicio.
            if s <= we && e.max(s + 1) > ws {
                events.push(Event {
                    id: format!("{}@{}", r.uid, s),
                    title: title.clone(),
                    start: s,
                    end: e,
                    all_day: start.all_day,
                    location: r.location.trim().to_string(),
                    calendar: calendar.to_string(),
                });
            }
        }
    }
    events
}

// ───────────────────────────── descarga ─────────────────────────────

async fn download(url: &str) -> Result<String, String> {
    let url = match url.strip_prefix("webcal://") {
        Some(rest) => format!("https://{rest}"),
        None => url.to_string(),
    };
    if !url.starts_with("https://") && !url.starts_with("http://") {
        return Err("la dirección debe empezar por https:// o webcal://".into());
    }
    let resp = reqwest::Client::builder()
        .user_agent(concat!("Noki/", env!("CARGO_PKG_VERSION")))
        .timeout(StdDuration::from_secs(25))
        .build()
        .map_err(|e| e.to_string())?
        .get(&url)
        .send()
        .await
        .map_err(|e| if e.is_connect() || e.is_timeout() { "no se pudo conectar".to_string() } else { e.to_string() })?;
    if !resp.status().is_success() {
        return Err(format!("el servidor respondió {}", resp.status().as_u16()));
    }
    let bytes = resp.bytes().await.map_err(|e| e.to_string())?;
    if bytes.len() > MAX_BYTES {
        return Err("el archivo es demasiado grande".into());
    }
    let text = String::from_utf8_lossy(&bytes).into_owned();
    if !text.contains("BEGIN:VCALENDAR") {
        return Err("no parece un calendario iCal (.ics)".into());
    }
    Ok(text)
}

/// Descarga todos los calendarios configurados y devuelve los eventos de los próximos días.
#[tauri::command]
pub async fn calendar_fetch(app: AppHandle) -> CalResult {
    let sources = app.state::<NotchState>().settings.lock().unwrap().calendar.sources.clone();
    let mut result = CalResult::default();
    let now = Local::now();
    for (i, s) in sources.iter().enumerate() {
        if s.url.is_empty() {
            continue;
        }
        let name = if s.name.is_empty() { format!("Calendario {}", i + 1) } else { s.name.clone() };
        match download(&s.url).await {
            Ok(text) => result.events.extend(parse_events(&text, &name, now)),
            Err(e) => result.errors.push(format!("{name}: {e}")),
        }
    }
    result.events.sort_by_key(|e| (e.start, e.title.clone()));
    result.events.dedup_by(|a, b| a.id == b.id && a.calendar == b.calendar);
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> String {
        // Lunes 2026-10-19 09:00 local, semanal (L y X), con una excepción y una instancia movida.
        [
            "BEGIN:VCALENDAR",
            "BEGIN:VEVENT",
            "UID:weekly@test",
            "SUMMARY:Reunión semanal",
            "DTSTART;TZID=America/Bogota:20261019T090000",
            "DTEND;TZID=America/Bogota:20261019T100000",
            "RRULE:FREQ=WEEKLY;BYDAY=MO,WE",
            "EXDATE;TZID=America/Bogota:20261021T090000",
            "END:VEVENT",
            "BEGIN:VEVENT",
            "UID:weekly@test",
            "RECURRENCE-ID;TZID=America/Bogota:20261026T090000",
            "SUMMARY:Reunión semanal (movida)",
            "DTSTART;TZID=America/Bogota:20261026T150000",
            "DTEND;TZID=America/Bogota:20261026T160000",
            "END:VEVENT",
            "BEGIN:VEVENT",
            "UID:allday@test",
            "SUMMARY:Feriado",
            "DTSTART;VALUE=DATE:20261027",
            "DTEND;VALUE=DATE:20261028",
            "END:VEVENT",
            "END:VCALENDAR",
        ]
        .join("\r\n")
    }

    #[test]
    fn expands_recurrence_with_exdate_and_override() {
        let now = Local.with_ymd_and_hms(2026, 10, 24, 12, 0, 0).unwrap();
        let ev = parse_events(&sample(), "Prueba", now);
        let starts: Vec<String> = ev
            .iter()
            .map(|e| format!("{} {}", e.title, Local.timestamp_millis_opt(e.start).unwrap().format("%m-%d %H:%M")))
            .collect();
        println!("{starts:#?}");
        // 21/10 está excluido; 26/10 se reemplaza por la instancia movida; 28/10 y 02/11 siguen la regla.
        assert!(!starts.iter().any(|s| s.ends_with("10-21 09:00")));
        assert!(starts.contains(&"Reunión semanal (movida) 10-26 15:00".to_string()));
        assert!(!starts.contains(&"Reunión semanal 10-26 09:00".to_string()));
        assert!(starts.contains(&"Reunión semanal 10-28 09:00".to_string()));
        assert!(starts.contains(&"Reunión semanal 11-02 09:00".to_string()));
        assert!(ev.iter().any(|e| e.all_day && e.title == "Feriado"));
    }

    #[test]
    fn parses_durations_and_monthly_rules() {
        assert_eq!(parse_duration("PT1H30M"), Some(5_400_000));
        assert_eq!(parse_duration("P1D"), Some(86_400_000));
        let rule = parse_rule("FREQ=MONTHLY;BYDAY=2TU;COUNT=3").unwrap();
        let start = NaiveDate::from_ymd_opt(2026, 10, 13).unwrap().and_hms_opt(8, 0, 0).unwrap();
        let limit = NaiveDate::from_ymd_opt(2027, 12, 31).unwrap().and_hms_opt(0, 0, 0).unwrap();
        let occ = occurrences(start, &rule, limit);
        let days: Vec<String> = occ.iter().map(|d| d.format("%Y-%m-%d").to_string()).collect();
        assert_eq!(days, vec!["2026-10-13", "2026-11-10", "2026-12-08"]);
    }
}
