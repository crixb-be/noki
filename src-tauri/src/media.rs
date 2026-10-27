//! Música real: lee y controla la sesión multimedia actual de Windows (GSMTC).
//! Funciona con Spotify, YouTube Music/Netflix en el navegador, VLC, Reproductor de Windows, etc.

use base64::{engine::general_purpose::STANDARD, Engine};
use serde::Serialize;
use std::{
    sync::{mpsc, Mutex},
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};
use tauri::{AppHandle, Emitter, Manager};
use windows::{
    Media::Control::{
        GlobalSystemMediaTransportControlsSession as Session,
        GlobalSystemMediaTransportControlsSessionManager as Manager_,
        GlobalSystemMediaTransportControlsSessionPlaybackStatus as Status,
    },
    Storage::Streams::DataReader,
    Win32::System::WinRT::{RoInitialize, RO_INIT_MULTITHREADED},
};

#[derive(Serialize, Clone, Default, PartialEq, Debug)]
#[serde(rename_all = "camelCase")]
pub struct MediaState {
    pub available: bool,
    pub track_key: String,
    pub title: String,
    pub artist: String,
    pub album: String,
    /// Nombre legible de la app origen (Spotify, Chrome…).
    pub source: String,
    pub playing: bool,
    /// Segundos (ya extrapolados al instante de lectura).
    pub position: f64,
    pub duration: f64,
    pub can_play_pause: bool,
    pub can_next: bool,
    pub can_prev: bool,
    pub can_seek: bool,
    /// Carátula como data URL. En los eventos solo viaja cuando cambia (ver `art_key`).
    pub art: Option<String>,
    /// Igual a `track_key` cuando la carátula corresponde a la pista actual; vacío si no hay.
    pub art_key: String,
}

pub enum Cmd {
    Toggle,
    Play,
    Pause,
    Next,
    Prev,
    Seek(f64),
}

pub struct MediaHandle {
    tx: Mutex<mpsc::Sender<Cmd>>,
    latest: Mutex<MediaState>,
}

impl MediaHandle {
    pub fn new() -> (Self, mpsc::Receiver<Cmd>) {
        let (tx, rx) = mpsc::channel();
        (Self { tx: Mutex::new(tx), latest: Mutex::new(MediaState::default()) }, rx)
    }
    pub fn send(&self, cmd: Cmd) {
        let _ = self.tx.lock().unwrap().send(cmd);
    }
    pub fn latest(&self) -> MediaState {
        self.latest.lock().unwrap().clone()
    }
}

pub fn start(app: AppHandle, rx: mpsc::Receiver<Cmd>) {
    std::thread::spawn(move || run(app, rx));
}

#[derive(Default)]
struct ArtCache {
    key: String,
    art: Option<String>,
    tries: u32,
}

fn run(app: AppHandle, rx: mpsc::Receiver<Cmd>) {
    unsafe {
        let _ = RoInitialize(RO_INIT_MULTITHREADED);
    }
    let mut manager: Option<Manager_> = None;
    let mut cache = ArtCache::default();
    let mut last: Option<MediaState> = None;
    let mut last_emit = Instant::now();

    loop {
        let cmd = match rx.recv_timeout(Duration::from_millis(400)) {
            Ok(c) => Some(c),
            Err(mpsc::RecvTimeoutError::Timeout) => None,
            Err(mpsc::RecvTimeoutError::Disconnected) => break,
        };
        if manager.is_none() {
            manager = Manager_::RequestAsync().and_then(|op| op.join()).ok();
        }
        // Sin sesión activa, WinRT devuelve un resultado nulo (se trata como "nada reproduciéndose").
        let session = manager.as_ref().and_then(|m| m.GetCurrentSession().ok());
        if let (Some(c), Some(s)) = (cmd, session.as_ref()) {
            apply(s, c);
        }

        let mut state = read_state(session.as_ref(), &mut cache);
        let handle = app.state::<MediaHandle>();
        *handle.latest.lock().unwrap() = state.clone();

        // Se emite si cambió algo relevante, si hubo un salto de posición (seek) o cada segundo mientras suena.
        let now = Instant::now();
        let elapsed = now.duration_since(last_emit).as_secs_f64();
        let should = match &last {
            None => true,
            Some(p) => {
                let expected = p.position + if p.playing { elapsed } else { 0.0 };
                let mut a = p.clone();
                let mut b = state.clone();
                a.position = 0.0;
                b.position = 0.0;
                a.art = None;
                b.art = None;
                a != b || (state.position - expected).abs() > 1.5 || (state.playing && elapsed >= 1.0)
            }
        };
        if should {
            let art_changed = last.as_ref().map(|p| p.art_key != state.art_key).unwrap_or(true);
            let full = state.clone();
            if !art_changed {
                state.art = None;
            }
            let _ = app.emit("media-state", &state);
            last = Some(full);
            last_emit = now;
        }
    }
}

fn apply(s: &Session, cmd: Cmd) {
    let _ = match cmd {
        Cmd::Toggle => s.TryTogglePlayPauseAsync().and_then(|o| o.join()),
        Cmd::Play => s.TryPlayAsync().and_then(|o| o.join()),
        Cmd::Pause => s.TryPauseAsync().and_then(|o| o.join()),
        Cmd::Next => s.TrySkipNextAsync().and_then(|o| o.join()),
        Cmd::Prev => s.TrySkipPreviousAsync().and_then(|o| o.join()),
        Cmd::Seek(secs) => s.TryChangePlaybackPositionAsync((secs.max(0.0) * 1e7) as i64).and_then(|o| o.join()),
    };
}

/// 100 ns desde 1601 (FILETIME), igual que `DateTime.UniversalTime` de WinRT.
fn now_filetime() -> i64 {
    let d = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default();
    116_444_736_000_000_000 + (d.as_nanos() / 100) as i64
}

fn read_state(session: Option<&Session>, cache: &mut ArtCache) -> MediaState {
    let Some(s) = session else {
        *cache = ArtCache::default();
        return MediaState::default();
    };
    let mut st = MediaState { available: true, ..Default::default() };

    if let Ok(id) = s.SourceAppUserModelId() {
        st.source = pretty_source(&id.to_string());
    }

    if let Ok(info) = s.GetPlaybackInfo() {
        st.playing = info.PlaybackStatus().map(|p| p == Status::Playing).unwrap_or(false);
        if let Ok(c) = info.Controls() {
            st.can_play_pause = c.IsPlayPauseToggleEnabled().unwrap_or(false)
                || c.IsPlayEnabled().unwrap_or(false)
                || c.IsPauseEnabled().unwrap_or(false);
            st.can_next = c.IsNextEnabled().unwrap_or(false);
            st.can_prev = c.IsPreviousEnabled().unwrap_or(false);
            st.can_seek = c.IsPlaybackPositionEnabled().unwrap_or(false);
        }
    }

    if let Ok(t) = s.GetTimelineProperties() {
        let ticks = |r: windows::core::Result<windows::Foundation::TimeSpan>| r.map(|v| v.Duration).unwrap_or(0);
        let start = ticks(t.StartTime());
        let end = ticks(t.EndTime());
        st.duration = ((end - start).max(0)) as f64 / 1e7;
        let mut pos = (ticks(t.Position()) - start).max(0) as f64 / 1e7;
        if st.playing {
            if let Ok(updated) = t.LastUpdatedTime() {
                let since = (now_filetime() - updated.UniversalTime).max(0) as f64 / 1e7;
                // Se descartan desfases absurdos (apps que nunca actualizan la marca de tiempo).
                if since < 6.0 * 3600.0 {
                    pos += since;
                }
            }
        }
        st.position = if st.duration > 0.0 { pos.min(st.duration) } else { pos };
    }

    if let Ok(props) = s.TryGetMediaPropertiesAsync().and_then(|o| o.join()) {
        st.title = props.Title().map(|v| v.to_string()).unwrap_or_default();
        st.artist = props.Artist().map(|v| v.to_string()).unwrap_or_default();
        st.album = props.AlbumTitle().map(|v| v.to_string()).unwrap_or_default();
        st.track_key = format!("{}|{}|{}|{}", st.source, st.title, st.artist, st.album);

        if cache.key != st.track_key {
            *cache = ArtCache { key: st.track_key.clone(), art: None, tries: 0 };
        }
        // Algunas apps publican la carátula unos instantes después del título: se reintenta unas veces.
        if cache.art.is_none() && cache.tries < 6 {
            cache.tries += 1;
            cache.art = read_thumbnail(&props);
        }
        if cache.art.is_some() {
            st.art = cache.art.clone();
            st.art_key = st.track_key.clone();
        }
    }
    st
}

fn read_thumbnail(
    props: &windows::Media::Control::GlobalSystemMediaTransportControlsSessionMediaProperties,
) -> Option<String> {
    let reference = props.Thumbnail().ok()?;
    let stream = reference.OpenReadAsync().ok()?.join().ok()?;
    let size = stream.Size().ok()? as u32;
    if size == 0 || size > 8 * 1024 * 1024 {
        return None;
    }
    let reader = DataReader::CreateDataReader(&stream).ok()?;
    reader.LoadAsync(size).ok()?.join().ok()?;
    let mut buf = vec![0u8; size as usize];
    reader.ReadBytes(&mut buf).ok()?;
    let mime = stream
        .ContentType()
        .map(|c| c.to_string())
        .ok()
        .filter(|c| c.starts_with("image/"))
        .unwrap_or_else(|| if buf.starts_with(&[0x89, b'P', b'N', b'G']) { "image/png".into() } else { "image/jpeg".into() });
    Some(format!("data:{mime};base64,{}", STANDARD.encode(buf)))
}

/// "Spotify.exe" → "Spotify"; "Microsoft.ZuneMusic_8wekyb3d8bbwe!Microsoft.ZuneMusic" → "ZuneMusic".
fn pretty_source(id: &str) -> String {
    let base = id.split('!').next().unwrap_or(id);
    let base = base.split('_').next().unwrap_or(base);
    let base = base.trim_end_matches(".exe");
    let name = base.rsplit('.').next().unwrap_or(base);
    match name.to_lowercase().as_str() {
        "zunemusic" => "Reproductor multimedia".into(),
        "zunevideo" => "Películas y TV".into(),
        "msedge" => "Edge".into(),
        "chrome" => "Chrome".into(),
        _ => name.to_string(),
    }
}
