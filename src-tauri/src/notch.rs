//! Comportamiento nativo del notch: posición, AppBar, click-through, hover y pantalla completa.
//!
//! Cada pantalla puede tener su propio notch (una ventana por pantalla: `main` y `main-1`, `main-2`…).
//! Cada ventana tiene un tamaño fijo (el del notch expandido más un margen) y nunca se redimensiona al
//! abrir o cerrar. Un único hilo sondea el cursor cada pocos ms y, para cada ventana, decide:
//!  - si el cursor está sobre su notch (la ventana capta clics) o no (los clics atraviesan);
//!  - cuándo abrir o cerrar, y avisa a esa ventana con el evento `notch-open`.
//! El mismo hilo crea y destruye las ventanas extra cuando cambian los ajustes o las pantallas conectadas.

use crate::settings::Settings;
use std::{
    collections::HashMap,
    mem::size_of,
    sync::{
        atomic::{AtomicBool, AtomicI32, Ordering::SeqCst},
        Arc, Mutex,
    },
    time::{Duration, Instant},
};
use tauri::{
    AppHandle, Emitter, Manager, PhysicalPosition, PhysicalSize, WebviewUrl, WebviewWindow, WebviewWindowBuilder,
};
use windows::Win32::{
    Foundation::{HWND, LPARAM, POINT, RECT},
    Graphics::Gdi::{GetDC, GetPixel, ReleaseDC},
    UI::{
        Input::KeyboardAndMouse::{GetAsyncKeyState, VK_LBUTTON, VK_RBUTTON},
        Shell::{
            SHAppBarMessage, SHQueryUserNotificationState, APPBARDATA, QUNS_BUSY, QUNS_PRESENTATION_MODE,
            QUNS_RUNNING_D3D_FULL_SCREEN,
        },
        WindowsAndMessaging::{
            GetCursorPos, GetWindowLongPtrW, IsWindowVisible, SetWindowLongPtrW, SetWindowPos, GWL_EXSTYLE,
            HWND_TOPMOST, SWP_NOACTIVATE, SWP_NOMOVE, SWP_NOSIZE,
            WS_EX_TOOLWINDOW,
        },
    },
};

/// Margen transparente a cada lado del notch para las "orejas" cóncavas (px lógicos).
pub const EAR: i32 = 16;
/// Margen inferior para la sombra (px lógicos).
pub const SHADOW: i32 = 28;

const ABM_NEW: u32 = 0;
const ABM_REMOVE: u32 = 1;
const ABM_QUERYPOS: u32 = 2;
const ABM_SETPOS: u32 = 3;
const ABE_TOP: u32 = 1;

/// Estado de un notch (una ventana).
pub struct Win {
    pub open: AtomicBool,
    /// El frontend está escribiendo en un campo: no cerrar por salir el cursor.
    pub pinned: AtomicBool,
    /// Abierto por atajo: se cierra con Esc, el atajo o un clic fuera.
    pub sticky: AtomicBool,
    /// Tamaño (px lógicos) del aviso visible en el notch cerrado; 0 = ninguno. Define la zona que capta clics.
    pub peek_w: AtomicI32,
    pub peek_h: AtomicI32,
    appbar_hwnd: Mutex<Option<isize>>,
    /// Rectángulo de la pantalla donde está este notch (para elegir cuál abre el atajo).
    monitor: Mutex<Option<Rect>>,
}

impl Win {
    fn new() -> Self {
        Self {
            open: AtomicBool::new(false),
            pinned: AtomicBool::new(false),
            sticky: AtomicBool::new(false),
            peek_w: AtomicI32::new(0),
            peek_h: AtomicI32::new(0),
            appbar_hwnd: Mutex::new(None),
            monitor: Mutex::new(None),
        }
    }
}

pub struct NotchState {
    pub settings: Mutex<Settings>,
    /// Los ajustes cambiaron: recalcular ventanas y layout ya.
    pub dirty: AtomicBool,
    wins: Mutex<HashMap<String, Arc<Win>>>,
}

impl NotchState {
    pub fn new(settings: Settings) -> Self {
        Self { settings: Mutex::new(settings), dirty: AtomicBool::new(true), wins: Mutex::new(HashMap::new()) }
    }

    pub fn win(&self, label: &str) -> Option<Arc<Win>> {
        self.wins.lock().unwrap().get(label).cloned()
    }

    fn ensure_win(&self, label: &str) -> Arc<Win> {
        self.wins.lock().unwrap().entry(label.to_string()).or_insert_with(|| Arc::new(Win::new())).clone()
    }

    fn remove_win(&self, label: &str) {
        self.wins.lock().unwrap().remove(label);
    }

    fn all(&self) -> Vec<(String, Arc<Win>)> {
        self.wins.lock().unwrap().iter().map(|(k, v)| (k.clone(), v.clone())).collect()
    }
}

// ───────────────────────────── geometría ─────────────────────────────

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Rect {
    left: i32,
    top: i32,
    right: i32,
    bottom: i32,
}

impl Rect {
    fn contains(&self, p: POINT) -> bool {
        p.x >= self.left && p.x < self.right && p.y >= self.top && p.y < self.bottom
    }
    fn inflate(&self, d: i32) -> Rect {
        Rect { left: self.left - d, top: self.top - d, right: self.right + d, bottom: self.bottom + d }
    }
    fn width(&self) -> i32 {
        self.right - self.left
    }
    fn height(&self) -> i32 {
        self.bottom - self.top
    }
}

/// Una pantalla (real o simulada), en píxeles físicos y coordenadas de pantalla virtual.
#[derive(Clone, Debug)]
pub struct Mon {
    rect: Rect,
    scale: f64,
    name: String,
    primary: bool,
}

/// Pantallas conectadas. Con la variable de entorno `MINOTCH_FAKE_MONITORS=N` la pantalla principal se
/// divide en N franjas verticales (solo sirve para probar el modo multipantalla en un equipo con un monitor).
pub fn monitors(app: &AppHandle) -> Vec<Mon> {
    let Some(window) = app.get_webview_window("main") else { return vec![] };
    let primary_pos = window.primary_monitor().ok().flatten().map(|m| *m.position());
    let real: Vec<Mon> = window
        .available_monitors()
        .unwrap_or_default()
        .iter()
        .enumerate()
        .map(|(i, m)| {
            let p = m.position();
            let s = m.size();
            Mon {
                rect: Rect { left: p.x, top: p.y, right: p.x + s.width as i32, bottom: p.y + s.height as i32 },
                scale: m.scale_factor(),
                name: m.name().cloned().unwrap_or_else(|| format!("Monitor {}", i + 1)),
                primary: primary_pos.map(|pp| pp == *p).unwrap_or(i == 0),
            }
        })
        .collect();

    let fake: usize = std::env::var("MINOTCH_FAKE_MONITORS").ok().and_then(|v| v.parse().ok()).unwrap_or(0);
    if fake >= 2 {
        if let Some(base) = real.iter().find(|m| m.primary).or(real.first()) {
            let w = base.rect.width() / fake as i32;
            return (0..fake as i32)
                .map(|i| Mon {
                    rect: Rect {
                        left: base.rect.left + i * w,
                        top: base.rect.top,
                        right: base.rect.left + (i + 1) * w,
                        bottom: base.rect.bottom,
                    },
                    scale: base.scale,
                    name: format!("Pantalla simulada {}", i + 1),
                    primary: i == 0,
                })
                .collect();
        }
    }
    real
}

pub struct MonitorInfo {
    pub index: usize,
    pub name: String,
    pub width: u32,
    pub height: u32,
    pub primary: bool,
}

pub fn list_monitors(app: &AppHandle) -> Vec<MonitorInfo> {
    monitors(app)
        .iter()
        .enumerate()
        .map(|(index, m)| MonitorInfo {
            index,
            name: m.name.clone(),
            width: m.rect.width() as u32,
            height: m.rect.height() as u32,
            primary: m.primary,
        })
        .collect()
}

/// Qué ventana va en qué pantalla: `main` en la elegida (o la principal) y, con «un notch por pantalla»,
/// `main-1`, `main-2`… en las demás.
fn assignments(s: &Settings, mons: &[Mon]) -> Vec<(String, Mon)> {
    if mons.is_empty() {
        return vec![];
    }
    let primary = mons.iter().position(|m| m.primary).unwrap_or(0);
    let main_idx = if s.general.monitor >= 0 && (s.general.monitor as usize) < mons.len() {
        s.general.monitor as usize
    } else {
        primary
    };
    let mut out = vec![("main".to_string(), mons[main_idx].clone())];
    if s.general.all_monitors {
        let mut k = 1;
        for (i, m) in mons.iter().enumerate() {
            if i != main_idx {
                out.push((format!("main-{k}"), m.clone()));
                k += 1;
            }
        }
    }
    out
}

/// Todo en píxeles físicos y coordenadas de pantalla virtual.
#[derive(Clone, Copy, Debug)]
struct Layout {
    scale: f64,
    /// Zona del notch abierto (la que debe captar el ratón), sin contar el resto de la ventana.
    window: Rect,
    /// La ventana nativa: ocupa todo el ancho de la pantalla para poder pintar la franja a los costados del notch.
    frame: Rect,
    pill: Rect,
}

/// Entradas que, si cambian, obligan a recalcular el layout (y renegociar el AppBar).
#[derive(Clone, Copy, PartialEq, Debug)]
struct Key {
    monitor: Rect,
    scale_milli: i32,
    bar_h: i32,
    reserve: bool,
    sizes: [i32; 5],
}

fn px(v: i32, scale: f64) -> i32 {
    (v as f64 * scale).round() as i32
}

fn compute(monitor: Rect, scale: f64, s: &Settings, top: i32) -> Layout {
    let a = &s.appearance;
    let win_w = px(a.expanded_w + 2 * EAR, scale);
    let win_h = px(a.expanded_h + SHADOW, scale);
    let cx = (monitor.left + monitor.right) / 2 + px(a.offset_x, scale);
    let win_left = cx - win_w / 2;
    let centered = |w: i32, h: i32| Rect { left: cx - w / 2, top, right: cx - w / 2 + w, bottom: top + h };
    Layout {
        scale,
        window: Rect { left: win_left, top, right: win_left + win_w, bottom: top + win_h },
        frame: Rect { left: monitor.left, top, right: monitor.right, bottom: top + win_h },
        pill: centered(px(a.collapsed_w, scale), px(a.collapsed_h, scale)),
    }
}

// ───────────────────────────── Win32 ─────────────────────────────

fn hwnd_of(window: &WebviewWindow) -> Option<HWND> {
    window.hwnd().ok()
}

fn cursor() -> POINT {
    let mut p = POINT::default();
    unsafe {
        let _ = GetCursorPos(&mut p);
    }
    p
}

fn button_down() -> bool {
    unsafe { (GetAsyncKeyState(VK_LBUTTON.0 as i32) < 0) || (GetAsyncKeyState(VK_RBUTTON.0 as i32) < 0) }
}

/// ¿Hay una app a pantalla completa (juego, vídeo, F11) o una presentación?
fn fullscreen_app_active() -> bool {
    unsafe {
        match SHQueryUserNotificationState() {
            Ok(st) => st == QUNS_BUSY || st == QUNS_RUNNING_D3D_FULL_SCREEN || st == QUNS_PRESENTATION_MODE,
            Err(_) => false,
        }
    }
}

/// WS_EX_TOOLWINDOW lo oculta de Alt+Tab. Tao reescribe los estilos cuando cambian sus flags
/// (p. ej. click-through), así que se reaplica periódicamente.
fn hide_from_alt_tab(hwnd: HWND) {
    unsafe {
        let ex = GetWindowLongPtrW(hwnd, GWL_EXSTYLE);
        let tool = WS_EX_TOOLWINDOW.0 as isize;
        if ex & tool == 0 {
            SetWindowLongPtrW(hwnd, GWL_EXSTYLE, ex | tool);
        }
    }
}

/// Color medio de la pantalla justo bajo la franja (dos puntos, cerca de cada borde, fuera del notch).
fn sample_tint(frame: Rect, dy: i32) -> Option<(u8, u8, u8)> {
    unsafe {
        let hdc = GetDC(None);
        if hdc.is_invalid() {
            return None;
        }
        let y = frame.top + dy;
        let w = frame.width();
        let mut sum = (0u32, 0u32, 0u32);
        let mut n = 0u32;
        for x in [frame.left + w / 10, frame.left + w * 9 / 10] {
            let c = GetPixel(hdc, x, y).0;
            if c != 0xFFFF_FFFF {
                sum.0 += c & 0xFF;
                sum.1 += (c >> 8) & 0xFF;
                sum.2 += (c >> 16) & 0xFF;
                n += 1;
            }
        }
        ReleaseDC(None, hdc);
        (n > 0).then(|| ((sum.0 / n) as u8, (sum.1 / n) as u8, (sum.2 / n) as u8))
    }
}

fn color_distance(a: (u8, u8, u8), b: (u8, u8, u8)) -> i32 {
    (a.0 as i32 - b.0 as i32).abs() + (a.1 as i32 - b.1 as i32).abs() + (a.2 as i32 - b.2 as i32).abs()
}

fn raise_topmost(hwnd: HWND) {
    unsafe {
        let _ = SetWindowPos(hwnd, Some(HWND_TOPMOST), 0, 0, 0, 0, SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE);
    }
}

/// Registra/actualiza el AppBar superior de esta ventana y devuelve el `top` real tras negociar con la barra de tareas.
fn appbar_set(win: &Win, hwnd: HWND, monitor: Rect, height: i32) -> i32 {
    let mut abd = APPBARDATA {
        cbSize: size_of::<APPBARDATA>() as u32,
        hWnd: hwnd,
        uCallbackMessage: 0,
        uEdge: ABE_TOP,
        rc: RECT { left: monitor.left, top: monitor.top, right: monitor.right, bottom: monitor.top + height },
        lParam: LPARAM(0),
    };
    unsafe {
        let mut reg = win.appbar_hwnd.lock().unwrap();
        if reg.is_none() {
            SHAppBarMessage(ABM_NEW, &mut abd);
            *reg = Some(hwnd.0 as isize);
        }
        SHAppBarMessage(ABM_QUERYPOS, &mut abd);
        abd.rc.bottom = abd.rc.top + height;
        SHAppBarMessage(ABM_SETPOS, &mut abd);
    }
    abd.rc.top
}

fn appbar_remove_win(win: &Win) {
    let mut reg = win.appbar_hwnd.lock().unwrap();
    if let Some(raw) = reg.take() {
        let mut abd =
            APPBARDATA { cbSize: size_of::<APPBARDATA>() as u32, hWnd: HWND(raw as *mut _), ..Default::default() };
        unsafe {
            SHAppBarMessage(ABM_REMOVE, &mut abd);
        }
    }
}

/// Libera el espacio reservado en todas las pantallas (al salir o al actualizar).
pub fn appbar_remove(state: &NotchState) {
    for (_, win) in state.all() {
        appbar_remove_win(&win);
    }
}

// ───────────────────────────── estado abierto/cerrado ─────────────────────────────

pub fn set_open(app: &AppHandle, label: &str, open: bool) {
    let st = app.state::<NotchState>();
    let Some(win) = st.win(label) else { return };
    if win.open.swap(open, SeqCst) != open {
        // Evento global con la etiqueta en el contenido: cada ventana filtra el suyo (los oyentes de JS reciben todos los eventos).
        let _ = app.emit("notch-open", serde_json::json!({ "label": label, "open": open }));
    }
    if !open {
        win.pinned.store(false, SeqCst);
        win.sticky.store(false, SeqCst);
    }
}

/// Atajo global / menú de bandeja: si hay algún notch abierto los cierra; si no, abre (de forma «pegajosa»)
/// el de la pantalla donde está el cursor.
pub fn toggle(app: &AppHandle) {
    let st = app.state::<NotchState>();
    let wins = st.all();
    let open: Vec<&String> = wins.iter().filter(|(_, w)| w.open.load(SeqCst)).map(|(l, _)| l).collect();
    if !open.is_empty() {
        for label in open {
            set_open(app, label, false);
        }
        return;
    }
    open_here(app);
}

/// Abre (de forma «pegajosa») el notch de la pantalla donde está el cursor y devuelve su etiqueta.
pub fn open_here(app: &AppHandle) -> Option<String> {
    let st = app.state::<NotchState>();
    let wins = st.all();
    if !st.settings.lock().unwrap().general.visible {
        return None;
    }
    let cur = cursor();
    let target = wins
        .iter()
        .find(|(_, w)| w.monitor.lock().unwrap().map(|m| m.contains(cur)).unwrap_or(false))
        .map(|(l, _)| l.clone())
        .unwrap_or_else(|| "main".to_string());
    let win = st.win(&target)?;
    win.sticky.store(true, SeqCst);
    set_open(app, &target, true);
    if let Some(w) = app.get_webview_window(&target) {
        let _ = w.set_focus();
    }
    Some(target)
}

// ───────────────────────────── ventanas ─────────────────────────────

fn create_window(app: &AppHandle, label: &str) -> bool {
    WebviewWindowBuilder::new(app, label, WebviewUrl::App("index.html".into()))
        .title("Noki")
        .inner_size(572.0, 348.0)
        .decorations(false)
        .transparent(true)
        .always_on_top(true)
        .resizable(false)
        .maximizable(false)
        .minimizable(false)
        .skip_taskbar(true)
        .shadow(false)
        .focused(false)
        .visible(false)
        .build()
        .is_ok()
}

/// Crea las ventanas que faltan y destruye las extra que ya no corresponden a ninguna pantalla.
fn sync_windows(app: &AppHandle, st: &NotchState, assign: &[(String, Mon)], loops: &mut HashMap<String, WinLoop>) {
    for (label, _) in assign {
        if app.get_webview_window(label).is_none() {
            create_window(app, label);
        }
        st.ensure_win(label);
    }
    for (label, window) in app.webview_windows() {
        if label.starts_with("main-") && !assign.iter().any(|(l, _)| *l == label) {
            if let Some(win) = st.win(&label) {
                appbar_remove_win(&win);
            }
            st.remove_win(&label);
            loops.remove(&label);
            let _ = window.destroy();
        }
    }
}

// ───────────────────────────── hilo principal ─────────────────────────────

/// Calcula el layout deseado, negocia el AppBar si hace falta y mueve/redimensiona la ventana.
fn ensure_layout(
    win: &Win,
    window: &WebviewWindow,
    s: &Settings,
    mon: &Mon,
    applied: &mut Option<(Key, Layout)>,
    force: bool,
) {
    let a = &s.appearance;
    let bar_h = px(a.collapsed_h, mon.scale);
    let reserve = s.general.reserve_space && s.general.visible;
    let key = Key {
        monitor: mon.rect,
        scale_milli: (mon.scale * 1000.0) as i32,
        bar_h,
        reserve,
        sizes: [a.collapsed_w, a.collapsed_h, a.expanded_w, a.expanded_h, a.offset_x],
    };

    if force || applied.as_ref().map(|(k, _)| *k != key).unwrap_or(true) {
        let top = match (reserve, hwnd_of(window)) {
            (true, Some(h)) => appbar_set(win, h, mon.rect, bar_h),
            _ => {
                appbar_remove_win(win);
                mon.rect.top
            }
        };
        *applied = Some((key, compute(mon.rect, mon.scale, s, top)));
    }
    *win.monitor.lock().unwrap() = Some(mon.rect);

    let want = applied.as_ref().unwrap().1.frame;
    let pos = window.outer_position().ok();
    let size = window.outer_size().ok();
    let pos_ok = pos.map(|p| (p.x - want.left).abs() <= 1 && (p.y - want.top).abs() <= 1).unwrap_or(false);
    let size_ok = size
        .map(|z| (z.width as i32 - want.width()).abs() <= 1 && (z.height as i32 - want.height()).abs() <= 1)
        .unwrap_or(false);
    if !pos_ok || !size_ok {
        // Primero se mueve (para caer en la pantalla correcta y su DPI) y luego se dimensiona.
        let _ = window.set_position(PhysicalPosition::new(want.left, want.top));
        let _ = window.set_size(PhysicalSize::new(want.width() as u32, want.height() as u32));
    }
}

pub fn start(app: AppHandle) {
    std::thread::spawn(move || run_loop(app));
}

/// Lo que el hilo recuerda de cada notch entre vueltas.
struct WinLoop {
    applied: Option<(Key, Layout)>,
    ignoring: Option<bool>,
    enter_since: Option<Instant>,
    leave_since: Option<Instant>,
    last_top: Instant,
    last_show: Instant,
    /// Franja automática: cuándo se muestreó y se avisó por última vez, y el último color enviado.
    last_tint: Instant,
    last_tint_emit: Instant,
    tint: Option<(u8, u8, u8)>,
    /// Personaje: última mirada enviada (cuantizada) y cuándo.
    last_look: Option<(i32, i32)>,
    last_look_at: Instant,
}

impl WinLoop {
    fn new() -> Self {
        let long_ago = Instant::now() - Duration::from_secs(10);
        Self {
            applied: None,
            ignoring: None,
            enter_since: None,
            leave_since: None,
            last_top: long_ago,
            last_show: long_ago,
            last_tint: long_ago,
            last_tint_emit: long_ago,
            tint: None,
            last_look: None,
            last_look_at: long_ago,
        }
    }
}

fn run_loop(app: AppHandle) {
    let mut loops: HashMap<String, WinLoop> = HashMap::new();
    let mut assign: Vec<(String, Mon)> = vec![];
    let mut fullscreen = false;
    let long_ago = Instant::now() - Duration::from_secs(10);
    let mut last_slow = long_ago;
    let mut last_fs = long_ago;

    loop {
        std::thread::sleep(Duration::from_millis(25));
        let st = app.state::<NotchState>();
        let s = st.settings.lock().unwrap().clone();
        let now = Instant::now();
        let dirty = st.dirty.swap(false, SeqCst);

        // Camino lento (cada segundo o al cambiar ajustes): qué pantallas hay, qué ventanas existen y dónde van.
        let slow = dirty || assign.is_empty() || now - last_slow >= Duration::from_secs(1);
        if slow {
            last_slow = now;
            let mons = monitors(&app);
            if mons.is_empty() {
                continue;
            }
            assign = assignments(&s, &mons);
            sync_windows(&app, &st, &assign, &mut loops);
        }

        if now - last_fs >= Duration::from_millis(400) {
            last_fs = now;
            fullscreen = s.general.hide_on_fullscreen && fullscreen_app_active();
        }
        let want_visible = s.general.visible && !fullscreen;

        let cur = cursor();
        let btn = button_down();

        for (label, mon) in &assign {
            let Some(window) = app.get_webview_window(label) else { continue };
            let Some(win) = st.win(label) else { continue };
            let Some(hwnd) = hwnd_of(&window) else { continue };
            let lp = loops.entry(label.clone()).or_insert_with(WinLoop::new);

            if slow || lp.applied.is_none() {
                ensure_layout(&win, &window, &s, mon, &mut lp.applied, dirty);
                crate::dropzone::ensure(&app, &window);
            }
            let Some((_, layout)) = lp.applied else { continue };

            // Se comprueba el estado real (no uno recordado): tao/WebView2 pueden mostrar u ocultar
            // la ventana por su cuenta durante el arranque y el notch debe autocorregirse.
            if unsafe { IsWindowVisible(hwnd).as_bool() } != want_visible && now - lp.last_show >= Duration::from_millis(250) {
                lp.last_show = now;
                // Se muestra/oculta con tao y no con SetWindowPos: tao recuerda su propio estado de
                // visibilidad y lo reaplica cada vez que reescribe los estilos (click-through), lo que
                // ocultaba la ventana en pleno uso (p. ej. al salir el cursor hacia un selector nativo).
                if want_visible {
                    let _ = window.show();
                } else {
                    let _ = window.hide();
                }
                if want_visible {
                    raise_topmost(hwnd);
                }
            }
            if want_visible && now - lp.last_top >= Duration::from_secs(1) {
                lp.last_top = now;
                hide_from_alt_tab(hwnd);
                raise_topmost(hwnd);
            }

            let open = win.open.load(SeqCst);
            if !want_visible {
                if open {
                    set_open(&app, label, false);
                }
                if lp.ignoring != Some(true) {
                    let _ = window.set_ignore_cursor_events(true);
                    hide_from_alt_tab(hwnd);
                    lp.ignoring = Some(true);
                }
                continue;
            }

            // El personaje del notch mira hacia el cursor: se avisa de su posición relativa (como mucho cada 60 ms y solo si cambia).
            if s.appearance.mascot && now - lp.last_look_at >= Duration::from_millis(60) {
                lp.last_look_at = now;
                let cx = (layout.pill.left + layout.pill.right) / 2;
                let cy = layout.pill.top + layout.pill.height() / 2;
                let dx = (cur.x - cx) as f64 / layout.scale;
                let dy = (cur.y - cy) as f64 / layout.scale;
                let near = dx.abs() < 700.0 && dy > -60.0 && dy < 500.0;
                let q = if near {
                    (((dx / 260.0).clamp(-1.0, 1.0) * 20.0).round() as i32, ((dy / 180.0).clamp(-1.0, 1.0) * 20.0).round() as i32)
                } else {
                    (99, 99)
                };
                if lp.last_look != Some(q) {
                    lp.last_look = Some(q);
                    let _ = app.emit(
                        "mascot-cursor",
                        serde_json::json!({ "label": label, "near": near, "x": q.0 as f64 / 20.0, "y": q.1 as f64 / 20.0 }),
                    );
                }
            }

            // Franja en modo «automático»: toma el color de lo que hay justo debajo de ella (el programa abierto).
            if s.appearance.bar_mode == "auto" || s.appearance.notch_auto {
                if now - lp.last_tint >= Duration::from_millis(350) {
                    lp.last_tint = now;
                    if let Some(c) = sample_tint(layout.frame, px(s.appearance.collapsed_h, layout.scale) + px(4, layout.scale)) {
                        let changed = lp.tint.map_or(true, |p| color_distance(p, c) > 6);
                        // Se repite cada 3 s aunque no cambie, por si la ventana aún no escuchaba.
                        if changed || now - lp.last_tint_emit >= Duration::from_secs(3) {
                            lp.tint = Some(c);
                            lp.last_tint_emit = now;
                            let _ = app.emit(
                                "bar-tint",
                                serde_json::json!({ "label": label, "color": format!("#{:02x}{:02x}{:02x}", c.0, c.1, c.2) }),
                            );
                        }
                    }
                }
            } else {
                lp.tint = None;
            }

            let open_zone = layout.window.inflate(px(10, layout.scale));
            let inside_open = open_zone.contains(cur);
            let on_pill = layout.pill.contains(cur);
            let on_edge = {
                let reach = px(80, layout.scale);
                let strip = Rect {
                    left: layout.pill.left - reach,
                    right: layout.pill.right + reach,
                    top: layout.pill.top,
                    bottom: layout.pill.top + px(3, layout.scale),
                };
                strip.contains(cur)
            };

            // Click-through: la ventana solo capta el ratón sobre el notch visible.
            let peek_w = win.peek_w.load(SeqCst);
            let closed_hit = if peek_w > 0 {
                let w = px(peek_w, layout.scale);
                let h = px(win.peek_h.load(SeqCst).max(s.appearance.collapsed_h), layout.scale);
                let left = (layout.pill.left + layout.pill.right) / 2 - w / 2;
                Rect { left, top: layout.pill.top, right: left + w, bottom: layout.pill.top + h }
            } else {
                layout.pill
            };
            // Con el notch ensanchado para mostrar más datos (sin aviso encima), todo su ancho abre al pasar el cursor.
            let on_pill = on_pill || (peek_w > 0 && win.peek_h.load(SeqCst) == 0 && closed_hit.contains(cur));
            let hit = if open { inside_open } else { closed_hit.contains(cur) };
            if lp.ignoring != Some(!hit) {
                let _ = window.set_ignore_cursor_events(!hit);
                hide_from_alt_tab(hwnd);
                lp.ignoring = Some(!hit);
            }

            if open {
                lp.enter_since = None;
                if inside_open {
                    lp.leave_since = None;
                } else {
                    lp.leave_since.get_or_insert(now);
                    // Un campo de formulario puede abrir un selector nativo fuera de la ventana
                    // (por ejemplo, el calendario de Windows). Mientras el frontend está fijado,
                    // ese clic no debe cerrar el notch ni desmontar el contenido. El cierre usa
                    // siempre el retraso configurado para evitar carreras con los clics del WebView.
                    if !win.pinned.load(SeqCst)
                        && !win.sticky.load(SeqCst)
                        // Con un botón pulsado (p. ej. arrastrando un archivo de una carpeta al notch) no se cierra solo.
                        && !btn
                        && now - lp.leave_since.unwrap() >= Duration::from_millis(s.general.close_delay as u64)
                    {
                        set_open(&app, label, false);
                    }
                }
            } else {
                lp.leave_since = None;
                let triggered = on_pill || (s.general.trigger == "edge" && on_edge);
                if triggered {
                    let since = *lp.enter_since.get_or_insert(now);
                    // Con un botón pulsado (arrastrando un archivo) se abre al instante.
                    let delay = if btn { 0 } else { s.general.hover_delay as u64 };
                    if now - since >= Duration::from_millis(delay) {
                        set_open(&app, label, true);
                        lp.enter_since = None;
                    }
                } else {
                    lp.enter_since = None;
                }
            }
        }
    }
}
