//! Visualizador de audio real: captura lo que suena en el equipo (WASAPI loopback del dispositivo de
//! salida predeterminado), calcula el espectro con una FFT y emite 12 bandas (0–1) como `audio-levels`.
//!
//! Solo captura mientras el frontend lo pide (música sonando); si no, el hilo duerme y no consume CPU.
//! No se guarda ni se envía audio: únicamente los niveles por banda.

use std::{
    collections::HashSet,
    f32::consts::PI,
    sync::{
        atomic::{AtomicBool, Ordering::SeqCst},
        Mutex,
    },
    time::{Duration, Instant},
};
use tauri::{AppHandle, Emitter, Manager, State};
use windows::Win32::{
    Media::Audio::{
        eConsole, eRender, IAudioCaptureClient, IAudioClient, IMMDeviceEnumerator, MMDeviceEnumerator,
        AUDCLNT_SHAREMODE_SHARED, AUDCLNT_STREAMFLAGS_LOOPBACK,
    },
    System::Com::{CoCreateInstance, CoInitializeEx, CoTaskMemFree, CLSCTX_ALL, COINIT_MULTITHREADED},
};

const N: usize = 1024;
pub const BANDS: usize = 12;
const EMIT_EVERY: Duration = Duration::from_millis(40);

/// Cada ventana (notch) pide el visualizador por separado; se captura mientras alguna lo necesite.
#[derive(Default)]
pub struct Viz {
    requested_by: Mutex<HashSet<String>>,
    enabled: AtomicBool,
}

#[tauri::command]
pub fn set_visualizer(window: tauri::Window, viz: State<Viz>, enabled: bool) {
    let mut set = viz.requested_by.lock().unwrap();
    if enabled {
        set.insert(window.label().to_string());
    } else {
        set.remove(window.label());
    }
    viz.enabled.store(!set.is_empty(), SeqCst);
}

struct Capture {
    client: IAudioClient,
    cap: IAudioCaptureClient,
    rate: f32,
    channels: usize,
    float: bool,
    bits: u16,
    device_id: String,
}

impl Drop for Capture {
    fn drop(&mut self) {
        unsafe {
            let _ = self.client.Stop();
        }
    }
}

fn default_device_id(en: &IMMDeviceEnumerator) -> Option<String> {
    unsafe {
        let dev = en.GetDefaultAudioEndpoint(eRender, eConsole).ok()?;
        let id = dev.GetId().ok()?;
        let s = id.to_string().ok();
        CoTaskMemFree(Some(id.0 as *const _));
        s
    }
}

fn open(en: &IMMDeviceEnumerator) -> Result<Capture, String> {
    unsafe {
        let dev = en.GetDefaultAudioEndpoint(eRender, eConsole).map_err(|e| e.to_string())?;
        let id = dev.GetId().map_err(|e| e.to_string())?;
        let device_id = id.to_string().unwrap_or_default();
        CoTaskMemFree(Some(id.0 as *const _));

        let client: IAudioClient = dev.Activate(CLSCTX_ALL, None).map_err(|e| e.to_string())?;
        let fmt_ptr = client.GetMixFormat().map_err(|e| e.to_string())?;
        let fmt = *fmt_ptr; // copia (la estructura está empaquetada: no se toman referencias a sus campos)
        let (tag, channels, rate, bits) = (fmt.wFormatTag, fmt.nChannels as usize, fmt.nSamplesPerSec as f32, fmt.wBitsPerSample);
        // 3 = IEEE float; 0xFFFE = extensible (el formato de mezcla de Windows suele ser float de 32 bits).
        let float = (tag == 3 || tag == 0xFFFE) && bits == 32;
        let init = client.Initialize(AUDCLNT_SHAREMODE_SHARED, AUDCLNT_STREAMFLAGS_LOOPBACK, 2_000_000, 0, fmt_ptr, None);
        CoTaskMemFree(Some(fmt_ptr as *const _));
        init.map_err(|e| e.to_string())?;
        if !float && bits != 16 {
            return Err(format!("formato de audio no compatible ({bits} bits)"));
        }
        let cap: IAudioCaptureClient = client.GetService().map_err(|e| e.to_string())?;
        client.Start().map_err(|e| e.to_string())?;
        Ok(Capture { client, cap, rate, channels: channels.max(1), float, bits, device_id })
    }
}

/// Lee los paquetes pendientes y añade las muestras (mezcla a mono) al buffer circular.
fn drain(c: &Capture, ring: &mut Vec<f32>) -> Result<bool, String> {
    let mut got = false;
    unsafe {
        loop {
            let size = c.cap.GetNextPacketSize().map_err(|e| e.to_string())?;
            if size == 0 {
                break;
            }
            let mut data: *mut u8 = std::ptr::null_mut();
            let mut frames = 0u32;
            let mut flags = 0u32;
            c.cap.GetBuffer(&mut data, &mut frames, &mut flags, None, None).map_err(|e| e.to_string())?;
            let silent = flags & 0x2 != 0; // AUDCLNT_BUFFERFLAGS_SILENT
            let n = frames as usize;
            for f in 0..n {
                let mut sum = 0.0f32;
                if !silent && !data.is_null() {
                    for ch in 0..c.channels {
                        let i = f * c.channels + ch;
                        sum += if c.float {
                            (data as *const f32).add(i).read_unaligned()
                        } else {
                            (data as *const i16).add(i).read_unaligned() as f32 / 32768.0
                        };
                    }
                }
                ring.push(sum / c.channels as f32);
            }
            got |= !silent && n > 0;
            c.cap.ReleaseBuffer(frames).map_err(|e| e.to_string())?;
        }
    }
    let _ = c.bits;
    if ring.len() > N * 4 {
        ring.drain(..ring.len() - N);
    }
    Ok(got)
}

/// FFT radix-2 en el sitio (N potencia de 2).
fn fft(re: &mut [f32], im: &mut [f32]) {
    let n = re.len();
    let mut j = 0;
    for i in 1..n {
        let mut bit = n >> 1;
        while j & bit != 0 {
            j ^= bit;
            bit >>= 1;
        }
        j ^= bit;
        if i < j {
            re.swap(i, j);
            im.swap(i, j);
        }
    }
    let mut len = 2;
    while len <= n {
        let ang = -2.0 * PI / len as f32;
        let (wr, wi) = (ang.cos(), ang.sin());
        for start in (0..n).step_by(len) {
            let (mut cr, mut ci) = (1.0f32, 0.0f32);
            for k in 0..len / 2 {
                let (a, b) = (start + k, start + k + len / 2);
                let tr = re[b] * cr - im[b] * ci;
                let ti = re[b] * ci + im[b] * cr;
                re[b] = re[a] - tr;
                im[b] = im[a] - ti;
                re[a] += tr;
                im[a] += ti;
                let nr = cr * wr - ci * wi;
                ci = cr * wi + ci * wr;
                cr = nr;
            }
        }
        len <<= 1;
    }
}

/// Bandas de frecuencia espaciadas en escala logarítmica entre 50 Hz y 14 kHz.
fn bands(samples: &[f32], rate: f32) -> [f32; BANDS] {
    let mut re = [0.0f32; N];
    let mut im = [0.0f32; N];
    for (i, s) in samples.iter().enumerate().take(N) {
        let hann = 0.5 - 0.5 * (2.0 * PI * i as f32 / (N as f32 - 1.0)).cos();
        re[i] = s * hann;
    }
    fft(&mut re, &mut im);
    let hz_per_bin = rate / N as f32;
    let (lo, hi) = (50.0f32, 14000.0f32.min(rate / 2.0 - 1.0));
    let mut out = [0.0f32; BANDS];
    for (b, slot) in out.iter_mut().enumerate() {
        let f0 = lo * (hi / lo).powf(b as f32 / BANDS as f32);
        let f1 = lo * (hi / lo).powf((b + 1) as f32 / BANDS as f32);
        let k0 = ((f0 / hz_per_bin) as usize).clamp(1, N / 2 - 1);
        let k1 = ((f1 / hz_per_bin).ceil() as usize).clamp(k0 + 1, N / 2);
        let peak = (k0..k1).map(|k| (re[k] * re[k] + im[k] * im[k]).sqrt()).fold(0.0f32, f32::max);
        // Una senoidal a fondo de escala da ≈ N/4 con ventana Hann.
        let amp = peak / (N as f32 / 4.0);
        // Se compensa la caída natural del espectro musical hacia los agudos.
        let db = 20.0 * (amp + 1e-6).log10() + b as f32 * 1.6;
        *slot = ((db + 62.0) / 62.0).clamp(0.0, 1.0);
    }
    out
}

pub fn start(app: AppHandle) {
    std::thread::spawn(move || {
        unsafe {
            let _ = CoInitializeEx(None, COINIT_MULTITHREADED);
        }
        let enumerator: Option<IMMDeviceEnumerator> = unsafe { CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL).ok() };
        let Some(enumerator) = enumerator else { return };
        let viz = app.state::<Viz>();

        let mut capture: Option<Capture> = None;
        let mut ring: Vec<f32> = Vec::with_capacity(N * 4);
        let mut last_emit = Instant::now();
        let mut last_audio = Instant::now();
        let mut last_check = Instant::now();
        let mut retry_at = Instant::now();
        let mut zeros_sent = true;

        loop {
            if !viz.enabled.load(SeqCst) {
                if capture.take().is_some() {
                    ring.clear();
                }
                std::thread::sleep(Duration::from_millis(150));
                continue;
            }

            // El dispositivo de salida pudo cambiar (auriculares): se reabre la captura.
            if let Some(c) = &capture {
                if last_check.elapsed() > Duration::from_secs(1) {
                    last_check = Instant::now();
                    if default_device_id(&enumerator).map(|id| id != c.device_id).unwrap_or(true) {
                        capture = None;
                    }
                }
            }
            if capture.is_none() {
                if Instant::now() < retry_at {
                    std::thread::sleep(Duration::from_millis(200));
                    continue;
                }
                match open(&enumerator) {
                    Ok(c) => {
                        capture = Some(c);
                        ring.clear();
                    }
                    Err(_) => {
                        retry_at = Instant::now() + Duration::from_secs(2);
                        continue;
                    }
                }
            }

            let c = capture.as_ref().unwrap();
            match drain(c, &mut ring) {
                Ok(got) => {
                    if got {
                        last_audio = Instant::now();
                    }
                }
                Err(_) => {
                    capture = None;
                    retry_at = Instant::now() + Duration::from_secs(1);
                    continue;
                }
            }

            if last_emit.elapsed() >= EMIT_EVERY {
                last_emit = Instant::now();
                if last_audio.elapsed() < Duration::from_millis(350) && ring.len() >= N {
                    let levels = bands(&ring[ring.len() - N..], c.rate);
                    let rounded: Vec<f32> = levels.iter().map(|v| (v * 100.0).round() / 100.0).collect();
                    let _ = app.emit("audio-levels", rounded);
                    zeros_sent = false;
                } else if !zeros_sent {
                    // Silencio: una sola vez se manda ceros para que las barras bajen.
                    let _ = app.emit("audio-levels", vec![0.0f32; BANDS]);
                    zeros_sent = true;
                }
            }
            std::thread::sleep(Duration::from_millis(12));
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_tone_lights_the_matching_band() {
        let rate = 48000.0f32;
        let samples: Vec<f32> = (0..N).map(|i| 0.5 * (2.0 * PI * 440.0 * i as f32 / rate).sin()).collect();
        let b = bands(&samples, rate);
        let (best, level) = b.iter().enumerate().fold((0, 0.0f32), |a, (i, v)| if *v > a.1 { (i, *v) } else { a });
        // 440 Hz cae en la 4.ª banda (≈ 50·(14000/50)^(k/12): 4.ª entre 220 y 330 Hz o 5.ª 330–500 Hz).
        assert!((3..=5).contains(&best), "banda {best}: {b:?}");
        assert!(level > 0.6, "nivel {level}");
        // Y el silencio no enciende nada.
        let silent = bands(&vec![0.0f32; N], rate);
        assert!(silent.iter().all(|v| *v < 0.01), "{silent:?}");
    }
}
