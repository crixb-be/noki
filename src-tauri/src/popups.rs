//! Gestor único de ventanas emergentes de Noki (lanzador, buscador y paneles de la barra).
//!
//! Se cierran con Esc (cada vista), con un segundo clic en su botón (toggle), con un **clic fuera**
//! (vigilante global de este módulo) y cuando otra app pasa al primer plano. Antes el cierre dependía
//! solo de `Focused(false)`, y en Windows + WebView2 perder el foco no equivale a «clic fuera»: el
//! bloqueo de primer plano, el foco en la ventana hija de WebView2 y las activaciones que hace tao
//! generaban blurs espurios y el lanzador se cerraba solo a los ~400 ms (o se quedaba sin foco y ya
//! no se cerraba).

use std::{collections::HashMap, sync::{Arc, Mutex, OnceLock}, time::{Duration, Instant}};
use tauri::{AppHandle, Emitter, Manager, WebviewWindow, WindowEvent};
use windows::Win32::{
    Foundation::{POINT, RECT},
    System::Threading::{AttachThreadInput, GetCurrentProcessId, GetCurrentThreadId},
    UI::{
        Input::KeyboardAndMouse::{GetAsyncKeyState, VK_LBUTTON, VK_MBUTTON, VK_RBUTTON},
        WindowsAndMessaging::{
            AllowSetForegroundWindow, BringWindowToTop, GetCursorPos, GetForegroundWindow, GetWindowRect,
            GetWindowThreadProcessId, IsWindowVisible, SetForegroundWindow, ASFW_ANY,
        },
    },
};

/// Tras mostrarse, un popup ignora durante este tiempo los blurs y los clics fuera (los que deja la
/// propia maniobra de apertura). Cuenta desde el `show()` real, no desde el clic.
const OPEN_GRACE: Duration = Duration::from_millis(350);
/// Un blur solo cierra si, pasado este margen, el primer plano sigue siendo de otra app.
const BLUR_SETTLE: Duration = Duration::from_millis(120);

type Callback = Arc<dyn Fn() + Send + Sync>;

#[derive(Default)]
struct PopupState {
    active_popup: Option<String>,
    /// Momento del `show()` real (ver `present`); `None` mientras no se ha mostrado.
    shown_at: Option<Instant>,
    last_blur_hide: Option<(String, Instant)>,
    generation: u64,
}

static STATE: OnceLock<Mutex<PopupState>> = OnceLock::new();
static CALLBACKS: OnceLock<Mutex<HashMap<&'static str, Callback>>> = OnceLock::new();

fn state() -> &'static Mutex<PopupState> { STATE.get_or_init(|| Mutex::new(PopupState::default())) }
fn callbacks() -> &'static Mutex<HashMap<&'static str, Callback>> { CALLBACKS.get_or_init(|| Mutex::new(HashMap::new())) }

fn window_name(id: &str) -> &'static str {
    if id == "launchpad" { "launchpad" }
    else if id == "spotlight" { "spotlight" }
    else { "barpanel" }
}

/// Id con el que se registró la ventana (los paneles comparten «bar_panel»).
fn base_id(id: &str) -> &'static str {
    if id == "launchpad" { "launchpad" } else if id == "spotlight" { "spotlight" } else { "bar_panel" }
}

fn same_window(left: &str, right: &str) -> bool {
    if left == right { return true; }
    left.starts_with("bar_panel:") && right.starts_with("bar_panel:")
}

fn active_for_window(active: Option<&str>, id: &str) -> bool {
    match active {
        Some(value) if id == "bar_panel" => value.starts_with("bar_panel:"),
        Some(value) => value == id,
        None => false,
    }
}

fn in_grace(current: &PopupState) -> bool {
    // Sin `show()` todavía: también en gracia (la ventana aún se está abriendo).
    current.shown_at.is_none_or(|at| at.elapsed() < OPEN_GRACE)
}

fn hide_window(app: &AppHandle, name: &'static str) {
    if let Some(window) = app.get_webview_window(name) {
        let ui_window = window.clone();
        let _ = window.run_on_main_thread(move || { let _ = ui_window.hide(); });
    }
}

fn hide_other_windows(app: &AppHandle, id: &str) {
    for name in ["launchpad", "spotlight", "barpanel"] {
        let keep = (name == "launchpad" && id == "launchpad")
            || (name == "spotlight" && id == "spotlight")
            || (name == "barpanel" && id.starts_with("bar_panel:"));
        if !keep { hide_window(app, name); }
    }
}

/// Abre un popup lógico y oculta los demás inmediatamente.
pub fn open(app: &AppHandle, id: &str) -> Result<(), String> {
    if !matches!(id, "launchpad" | "spotlight") && !id.starts_with("bar_panel:") {
        return Err("Popup desconocido.".into());
    }
    let mut current = state().lock().unwrap();
    let previous = current.active_popup.clone();
    if previous.as_deref() == Some(id) { return Ok(()); }
    current.active_popup = Some(id.to_string());
    current.shown_at = None;
    current.last_blur_hide = None;
    current.generation = current.generation.wrapping_add(1);
    if previous.as_deref().is_none_or(|value| !same_window(value, id)) {
        hide_other_windows(app, id);
    }
    Ok(())
}

/// Cierra el popup indicado. Devuelve `true` si era el popup activo.
pub fn close(app: &AppHandle, id: &str) -> bool {
    let mut current = state().lock().unwrap();
    let was_active = current.active_popup.as_deref().is_some_and(|value| {
        if id == "bar_panel" { value.starts_with("bar_panel:") } else { value == id }
    });
    if was_active {
        current.active_popup = None;
        current.shown_at = None;
        current.generation = current.generation.wrapping_add(1);
    }
    // Se oculta siempre: si el estado se hubiera desincronizado, cerrar debe seguir funcionando.
    hide_window(app, window_name(id));
    was_active
}

/// Alterna un popup. Un toggle del mismo id justo después de un cierre automático se suprime para
/// evitar la carrera cierre -> clic en su botón -> reapertura.
pub fn toggle(app: &AppHandle, id: &str) -> Result<bool, String> {
    if !matches!(id, "launchpad" | "spotlight") && !id.starts_with("bar_panel:") {
        return Err("Popup desconocido.".into());
    }
    // Visibilidad real de la ventana (fuera del candado): si quedó oculta por otra vía,
    // el siguiente clic debe abrirla en vez de «cerrarla» sin efecto visible.
    let shown = app.get_webview_window(window_name(id)).and_then(|w| w.is_visible().ok()).unwrap_or(false);
    let mut current = state().lock().unwrap();
    if shown && current.active_popup.as_deref().is_some_and(|value| value == id) {
        current.active_popup = None;
        current.shown_at = None;
        current.generation = current.generation.wrapping_add(1);
        hide_window(app, window_name(id));
        return Ok(false);
    }
    if current.last_blur_hide.as_ref().is_some_and(|(last_id, at)| last_id == id && at.elapsed() < OPEN_GRACE) {
        return Ok(false);
    }
    let previous = current.active_popup.clone();
    current.active_popup = Some(id.to_string());
    current.shown_at = None;
    current.last_blur_hide = None;
    current.generation = current.generation.wrapping_add(1);
    if previous.as_deref().is_none_or(|value| !same_window(value, id)) {
        hide_other_windows(app, id);
    }
    Ok(true)
}

pub fn close_all(app: &AppHandle) {
    {
        let mut current = state().lock().unwrap();
        current.active_popup = None;
        current.shown_at = None;
        current.generation = current.generation.wrapping_add(1);
    }
    for name in ["launchpad", "spotlight", "barpanel"] { hide_window(app, name); }
}

#[tauri::command]
pub async fn close_all_command(app: AppHandle) { close_all(&app); }

/// Cierre automático (clic fuera u otra app delante): oculta, limpia el estado de la vista y avisa.
/// Solo si `id` sigue siendo el popup activo de la generación `epoch`.
fn auto_close(app: &AppHandle, id: &str, epoch: u64) {
    let closed = {
        let mut current = state().lock().unwrap();
        if !active_for_window(current.active_popup.as_deref(), base_id(id)) || current.generation != epoch { return; }
        let active = current.active_popup.take().unwrap_or_else(|| id.to_string());
        current.last_blur_hide = Some((active.clone(), Instant::now()));
        current.shown_at = None;
        current.generation = current.generation.wrapping_add(1);
        active
    };
    hide_window(app, window_name(&closed));
    if let Some(callback) = callbacks().lock().unwrap().get(base_id(&closed)).cloned() { callback(); }
    let _ = app.emit("popup-closed", base_id(&closed));
    if closed == "launchpad" { let _ = app.emit("launchpad-open", false); }
}

/// Muestra la ventana del popup, le da el foco de forma fiable y marca el inicio de la gracia.
/// El clic en el Dock (ventana no activable de este proceso) o en la barra da a Noki el derecho a
/// pasar al primer plano, así que basta con SetForegroundWindow; AttachThreadInput queda como
/// respaldo. Ya no se inyecta un ALT sintético: llegaba a la app de delante y podía abrir su menú
/// o reactivarla, que era uno de los blurs espurios que cerraban el lanzador.
pub fn present(window: &WebviewWindow, id: &'static str) {
    let ui_window = window.clone();
    let _ = window.run_on_main_thread(move || unsafe {
        let Ok(hwnd) = ui_window.hwnd() else { return };
        // tao debe saber que está visible, o su hide() posterior no la ocultaría.
        let _ = ui_window.show();
        // Después de show(): tao acaba de reescribir los estilos.
        crate::window_utils::enforce_chrome_free_hwnd(hwnd);
        let _ = AllowSetForegroundWindow(ASFW_ANY);
        let _ = BringWindowToTop(hwnd);
        if !SetForegroundWindow(hwnd).as_bool() {
            let fg = GetForegroundWindow();
            let mut fg_pid = 0u32;
            let fg_thread = if fg.0.is_null() { 0 } else { GetWindowThreadProcessId(fg, Some(&mut fg_pid)) };
            let me = GetCurrentThreadId();
            if fg_thread != 0 && fg_thread != me && fg_pid != GetCurrentProcessId() && AttachThreadInput(fg_thread, me, true).as_bool() {
                let _ = BringWindowToTop(hwnd);
                let _ = SetForegroundWindow(hwnd);
                let _ = AttachThreadInput(fg_thread, me, false);
            }
        }
        let mut current = state().lock().unwrap();
        if active_for_window(current.active_popup.as_deref(), id) { current.shown_at = Some(Instant::now()); }
    });
}

/// Registra el cierre automático para una ventana ya creada. El callback solo limpia el estado
/// específico de la vista.
pub fn register_window<F>(app: &AppHandle, window: &WebviewWindow, id: &'static str, on_blur_close: F)
where
    F: Fn() + Send + Sync + 'static,
{
    callbacks().lock().unwrap().insert(id, Arc::new(on_blur_close));
    let app = app.clone();
    window.on_window_event(move |event| {
        if !matches!(event, WindowEvent::Focused(false)) { return; }
        let app = app.clone();
        std::thread::spawn(move || {
            let (epoch, wait) = {
                let current = state().lock().unwrap();
                if !active_for_window(current.active_popup.as_deref(), id) { return; }
                // Un blur durante la gracia ya no se descarta: se vuelve a mirar cuando termina.
                let left = current.shown_at.map(|at| OPEN_GRACE.saturating_sub(at.elapsed())).unwrap_or(OPEN_GRACE);
                (current.generation, left.max(BLUR_SETTLE))
            };
            std::thread::sleep(wait);
            // Recorte de pantalla (Win+Shift+S, Impr Pant, Recortes): su superposición se lleva el
            // foco, pero el popup debe seguir visible para poder capturarlo.
            if crate::dock::capture_in_progress() { return; }
            // Solo cierra si el primer plano es de otra app (no nula: durante un cambio de foco
            // GetForegroundWindow devuelve NULL un instante y antes eso también cerraba).
            let foreign = unsafe {
                let foreground = GetForegroundWindow();
                let mut pid = 0u32;
                if !foreground.0.is_null() { GetWindowThreadProcessId(foreground, Some(&mut pid)); }
                pid != 0 && pid != GetCurrentProcessId()
            };
            if foreign { auto_close(&app, id, epoch); }
        });
    });
}

/// Rectángulo de una ventana de Noki si está visible.
fn visible_rect(app: &AppHandle, label: &str) -> Option<RECT> {
    let window = app.get_webview_window(label)?;
    let hwnd = window.hwnd().ok()?;
    let mut rect = RECT::default();
    unsafe { (IsWindowVisible(hwnd).as_bool() && GetWindowRect(hwnd, &mut rect).is_ok()).then_some(rect) }
}

fn contains(rect: &RECT, p: POINT) -> bool { p.x >= rect.left && p.x < rect.right && p.y >= rect.top && p.y < rect.bottom }

/// Vigilante de «clic fuera»: con un popup abierto, un clic (izquierdo, derecho o central) fuera de
/// él lo cierra. Los clics en el Dock y en la barra superior no cuentan: sus botones ya alternan el
/// popup (toggle) y abrir otro lo sustituye. Sondeo de 25 ms solo mientras hay un popup abierto.
pub fn start(app: AppHandle) {
    std::thread::spawn(move || {
        let mut was_down = false;
        loop {
            std::thread::sleep(Duration::from_millis(25));
            let active = {
                let current = state().lock().unwrap();
                current.active_popup.clone().filter(|_| !in_grace(&current)).map(|id| (id, current.generation))
            };
            let down = unsafe { [VK_LBUTTON, VK_RBUTTON, VK_MBUTTON].iter().any(|k| GetAsyncKeyState(k.0 as i32) < 0) };
            let Some((id, epoch)) = active else {
                // Un botón que ya estaba pulsado al abrirse no cuenta como clic nuevo.
                was_down = down;
                std::thread::sleep(Duration::from_millis(35));
                continue;
            };
            let pressed = down && !was_down;
            was_down = down;
            if !pressed { continue; }
            // Seleccionar la zona del recorte también es un clic «fuera»: no cierra nada.
            if crate::dock::capture_in_progress() { continue; }
            let mut cursor = POINT::default();
            if unsafe { GetCursorPos(&mut cursor) }.is_err() { continue; }
            let inside = ["launchpad", "spotlight", "barpanel", "dock", "topbar"]
                .iter()
                .filter(|label| **label == window_name(&id) || matches!(**label, "dock" | "topbar"))
                .filter_map(|label| visible_rect(&app, label))
                .any(|rect| contains(&rect, cursor));
            if !inside { auto_close(&app, &id, epoch); }
        }
    });
}
