# Integración de MiDock en Noki — especificación técnica

Fuente de verdad para portar MiDock (C#/WinForms, `D:\CLAUDE CODE\MiDock`) a Noki (Tauri v2 + Rust + React/Vite/Tailwind).
MiDock queda solo como **referencia de comportamiento**: `MiDock\docs\diseno.md`, `MiDock\docs\tarea-punto2.md`,
`MiDock\docs\tarea-punto3.md` y `MiDock\src\*.cs` (sí se puede portar su lógica: es código propio del usuario).
**No** copiar nada de `MiDock\referencia\` (sin licencia). Las capturas de `MiDock\Capturas\` sirven de guía visual.

## Reglas
- Solo Windows 10/11. Comentarios y textos de UI en español (y entradas en inglés en `src/lib/en.js` con la misma clave).
- Mantener el estilo existente de Noki: ver `src-tauri/src/notch.rs` (bucle de sondeo, `ex_style_on_ui`, cuidado con
  los estilos que tao reescribe, nada de `SendMessage` síncrono desde hilos que no son de la UI).
- Bajo consumo: nada de bucles rápidos en reposo. Sin telemetría.
- Todo nuevo ajuste con `#[serde(default)]` para que los `settings.json` antiguos sigan cargando.
- Permisos: añadir las nuevas etiquetas de ventana a `src-tauri/capabilities/default.json`.
- Al terminar cada fase: `cargo check` (en `src-tauri`) y `npm run build` sin errores. Sin warnings nuevos en Rust.

## 1. Ventanas y z-order

| Etiqueta | `?view=` | Descripción |
|---|---|---|
| `topbar` | `topbar` | Barra superior a todo el ancho del monitor del notch `main`, alto `topbar.height` px lógicos (28–32, 30 por defecto). Acrílico nativo. Registra el AppBar `ABE_TOP`. |
| `main` (y `main-N`) | `notch` | El notch. Sin franja propia cuando la barra está activa. **Owned por `topbar`** (`WebviewWindowBuilder::parent` o `SetWindowLongPtrW(GWLP_HWNDPARENT)` en el hilo de UI): Windows siempre la pinta encima de su dueña, así que no hace falta reafirmar TOPMOST en bucle. |
| `barpanel` | `barpanel` | Panel desplegable de la barra (uno a la vez). Acrílico, esquinas redondeadas de Win11 (`DWMWA_WINDOW_CORNER_PREFERENCE = DWMWCP_ROUND`). Se enfoca al abrirse; se cierra con Esc, al perder el foco (`WindowEvent::Focused(false)`) o al volver a pulsar el mismo icono. |
| `dock` | `dock` | Dock inferior flotante. |
| `launchpad` | `launchpad` | Superposición del Launchpad. |
| `settings` | `settings` | Ajustes unificados (ya existe). |

### Reglas de z-order (eliminan la pelea de TopMost)
- **Con la barra activa (`topbar.enabled`)**:
  - Solo `topbar` registra el AppBar (`ABM_NEW`/`ABM_QUERYPOS`/`ABM_SETPOS`, alto exacto de la barra). El notch **no** registra AppBar (llamar a `appbar_remove_win`) y su `top` = `monitor.top` (ocupa el mismo hueco que la barra).
  - La píldora cerrada del notch mide `collapsed_h = topbar.height` de alto (se fuerza en `compute` cuando la barra está activa) y queda en el centro exacto de la barra. Sin huecos: la barra pinta acrílico en todo el ancho, incluida la zona detrás de la píldora; las «orejas» cóncavas se siguen dibujando para unir la píldora con la barra.
  - Al expandirse, el notch crece hacia abajo desde el centro de la barra (la ventana ya tiene el tamaño expandido; solo hay que asegurar que la animación CSS parte del borde inferior de la barra).
  - `raise_topmost` del notch se mantiene, pero solo como autocorrección (si perdió WS_EX_TOPMOST). La barra comprueba cada 2 s con `WindowFromPoint` dos puntos (izquierda y derecha, fuera de la píldora): solo si una ventana de **otro proceso** la tapa, se reafirma `HWND_TOPMOST` con `SWP_NOACTIVATE|SWP_NOMOVE|SWP_NOSIZE|SWP_ASYNCWINDOWPOS` para la barra y, justo después, el notch.
  - La franja oscura de `App.jsx` (el `div` con `barFill`) **no** se pinta con la barra activa. Los ajustes de la franja se reaprovechan como tinte de la barra (ver §2).
  - Pantalla completa (`fullscreen_app_active`, ya existe): ocultar la barra y el dock igual que el notch, y quitar/reponer el AppBar de la barra.
- **Con la barra desactivada**, Noki se comporta exactamente como hoy (franja + AppBar propios del notch).
- Al salir (`quit_app`, actualizar, pánico): `ABM_REMOVE` de la barra y del dock (si reserva espacio). Ampliar `notch::appbar_remove`.

### Acrílico
Usar el crate `window-vibrancy` (versión compatible con tauri 2): `apply_acrylic(&window, Some((r,g,b,a)))`.
Si falla (Windows 10 antiguo), probar `apply_blur`; si también falla, el frontend pinta un fondo translúcido sólido.
Tinte oscuro ≈ `(16,16,16,110)`, claro ≈ `(242,242,242,110)`. Las ventanas con acrílico tienen fondo de documento
transparente y **no** son `transparent` a nivel de píxeles fuera de su forma: por eso la barra y el panel son ventanas
propias del tamaño exacto de lo que muestran. El dock **no** usa acrílico (su ventana tiene margen transparente para la
magnificación): fondo translúcido CSS + borde de 1 px.

## 2. Ajustes nuevos (`settings.rs`, `useSettings.js`)

```rust
pub struct TopBar {            // settings.topbar
    pub enabled: bool,         // false por defecto (no cambiar el comportamiento de quien actualiza)
    pub height: i32,           // 30 (clamp 28..=32)
    pub tint: String,          // "auto" (según tema del sistema) | "dark" | "light"
    pub show_cpu: bool, pub show_ram: bool,        // false
    pub show_wifi: bool, pub show_bluetooth: bool, pub show_sound: bool, pub show_battery: bool,  // true
    pub show_battery_pct: bool,                    // true
    pub show_search: bool, pub show_control_center: bool, pub show_capture: bool,
    pub show_notifications: bool, pub show_avatar: bool, pub show_clock: bool,   // true
    pub avatar_path: String,   // "" = automático (AccountPictures)
}
pub struct DockPinned { pub name: String, pub path: String, pub args: String, pub aumid: String }
pub struct DockFolder { pub name: String, pub path: String }
pub struct Dock {              // settings.dock
    pub enabled: bool,         // false por defecto
    pub icon_size: i32,        // 48 (32..=96)
    pub magnify: bool,         // true
    pub magnify_scale: f64,    // 1.6 (1.0..=2.5)
    pub show_apps: bool,       // true: icono «Aplicaciones» (abre el Launchpad) al principio
    pub show_running: bool,    // true: apps abiertas no ancladas
    pub show_trash: bool,      // true
    pub autohide: bool,        // false
    pub reserve_space: bool,   // false (si true y !autohide: AppBar ABE_BOTTOM)
    pub pinned: Vec<DockPinned>, // por defecto: Explorador, Edge (si existe), Bloc de notas
    pub folders: Vec<DockFolder>, // por defecto: Descargas
}
```
`commit_settings` en `lib.rs` debe marcar `dirty` y avisar a las nuevas ventanas (`settings-changed` ya se emite a todas).

## 3. Backend Rust (módulos nuevos)

- `topbar.rs`: crear/mostrar/ocultar la ventana `topbar` según ajustes, posicionarla en el monitor de `main`, AppBar,
  acrílico, comprobación de z-order cada 2 s y ocultación en pantalla completa. Lo más simple: integrarlo en el
  camino lento del bucle de `notch::run_loop` (ya corre cada 2 s y tiene `fullscreen`) llamando a funciones de `topbar.rs`.
  Comandos:
  - `user_avatar() -> Option<String>`: data URL PNG/JPG. Si `topbar.avatar_path` existe → esa imagen. Si no:
    `%APPDATA%\Microsoft\Windows\AccountPictures\*.accountpicture-ms` (contiene JPEG/PNG embebidos: extraer el
    de mayor tamaño buscando las firmas `FF D8 FF` … `FF D9` o PNG) o, en su defecto, la imagen más grande de
    `C:\Users\Public\AccountPictures\<SID>\` / `%PROGRAMDATA%\Microsoft\User Account Pictures\user.png`. Devuelve `None` si no hay nada.
  - `user_info()`: nombre visible, `EQUIPO\usuario`, nombre del equipo, si es administrador.
  - `open_bar_panel(kind: String, anchor_x: i32)`: `kind` ∈ `wifi|bluetooth|sound|battery|control|user`. Posiciona
    `barpanel` bajo el icono (x en px lógicos de la barra, mantenido dentro de la pantalla), emite `bar-panel` con el
    `kind`, lo muestra y le da el foco. Si ya está abierto con el mismo `kind`, lo cierra.
  - `close_bar_panel()`, `resize_bar_panel(height)` (el panel mide su contenido).
  - `send_keys(combo: String)`: `SendInput` para `win+s`, `win+shift+s`, `win+shift+r`, `win+n`, `win+a` (con
    `KEYEVENTF_EXTENDEDKEY` para LWIN). Captura → `ms-screenclip:`; grabación → Win+Shift+R; notificaciones →
    `ms-actioncenter:` con respaldo Win+N; centro de control de Windows → Win+A.
  - Wi-Fi: `wifi_networks()` (WLAN API: `WlanOpenHandle`, `WlanScan`, `WlanGetAvailableNetworkList`; deduplicar
    por SSID, ordenar por señal, marcar conectada/segura/perfil), `wifi_connect(ssid)` (con perfil →
    `WlanConnect`; sin perfil → abrir `ms-availablenetworks:`). Reutilizar lo que ya haga `system.rs`/`statuswatch.rs`
    para radios Wi-Fi/Bluetooth, volumen y batería; no duplicar.
  - Bluetooth: `bt_devices()` dispositivos conectados (`DeviceInformation::FindAllAsync` con
    `BluetoothDevice::GetDeviceSelectorFromConnectionStatus(Connected)`; añadir la feature `Devices_Bluetooth` si hace falta).
  - Sonido: `audio_output_name()` (nombre del dispositivo por defecto). Volumen/mute ya existen (`set_volume`, `set_mute`).
  - Centro de control: modo oscuro (registro `Personalize` + `WM_SETTINGCHANGE "ImmersiveColorSet"` con
    `SendMessageTimeout`), modo avión (todas las radios WinRT off/on), y abrir `ms-settings:` para lo que no tenga API.
- `dock.rs`:
  - Ventana `dock`: transparente, sin decoraciones, `skip_taskbar`, TOPMOST, **`WS_EX_NOACTIVATE`** (y
    `WS_EX_TOOLWINDOW`, ambos reaplicados en el hilo de UI como hace `ex_style_on_ui`). Centrada abajo en el monitor
    de `main`, a 6 px del borde de la zona de trabajo. Alto = `icon_size*magnify_scale + margen`; ancho = ancho
    máximo de la píldora magnificada. Click-through fuera de la píldora: el frontend informa del rectángulo de la
    píldora con `dock_set_hit(x,y,w,h)` y el bucle (mismo patrón que el notch) activa/desactiva `set_ignore_cursor_events`.
    Autoocultar: se esconde bajando y reaparece con el cursor en el borde inferior.
  - Ventanas abiertas: port de `MiDock\src\Ventanas.cs` (filtro de `EnumWindows`: visible, raíz, sin owner, sin
    `WS_EX_TOOLWINDOW` salvo `WS_EX_APPWINDOW`, no *cloaked*, con título, no de nuestro proceso; identidad por AUMID
    (`SHGetPropertyStoreForWindow` + `PKEY_AppUserModel_ID`) o ruta del exe). Emitir `dock-windows` solo cuando cambia
    la lista (sondeo cada 1 s mientras el dock está visible, o `RegisterShellHookWindow` si es sencillo).
  - Comandos: `dock_items()`, `dock_activate(key)` (sin ventanas → lanzar; una en primer plano → minimizar; minimizada
    u otra delante → restaurar y traer al frente; varias → alternar), `dock_launch(path,args,aumid)` (con aumid →
    `explorer.exe shell:AppsFolder\<aumid>`), `dock_close_windows(key)`, `dock_pin/unpin/reorder`, `trash_state()`
    (`SHQueryRecycleBinW`), `trash_open()` (`shell:RecycleBinFolder`), `trash_empty()` (`SHEmptyRecycleBinW`),
    `trash_drop(paths)` (`SHFileOperationW FO_DELETE | FOF_ALLOWUNDO`). Iconos: reutilizar `icons::app_icon`
    (ampliarlo para AUMID con `shell:AppsFolder\<aumid>` si no lo soporta) e iconos de stock para la papelera.
  - Menú contextual: el frontend dibuja su propio menú dentro de la ventana del dock (más alto de lo necesario solo
    mientras está abierto) o, más simple, con un menú nativo de Tauri (`Menu` + `popup`). Elegir el nativo.
- `launchpad.rs`:
  - Ventana `launchpad`: sin decoraciones, transparente, TOPMOST, cubre la zona de trabajo del monitor de `main`,
    oculta al inicio y reutilizada (no recrearla).
  - `open_launchpad()` / `toggle_launchpad()`: mostrar y **forzar el primer plano**:
    ```
    let fg = GetForegroundWindow();
    let fg_thread = GetWindowThreadProcessId(fg, None);
    let me = GetCurrentThreadId();
    AttachThreadInput(fg_thread, me, true);
    AllowSetForegroundWindow(ASFW_ANY);
    ShowWindow(hwnd, SW_SHOW); BringWindowToTop(hwnd); SetForegroundWindow(hwnd); SetFocus(hwnd);
    AttachThreadInput(fg_thread, me, false);
    ```
    Todo en el hilo de UI (`run_on_main_thread`). La ventana del launchpad **no** lleva `WS_EX_NOACTIVATE`
    (asegurarse de quitarlo si tao lo pusiera). Respaldo si `SetForegroundWindow` devuelve false: pulsación vacía
    de ALT con `SendInput` y reintento.
  - Se cierra con Esc (frontend → `close_launchpad`), al perder el foco (`WindowEvent::Focused(false)`) o con un clic
    en el fondo fuera de la rejilla.
  - Datos: `launcher::launcher_apps` (ya existe, con caché) y `launch_app`. Añadir fecha de instalación si es barato.

## 4. Frontend

- `main.jsx`: enrutar `?view=topbar|barpanel|dock|launchpad` a componentes nuevos en `src/components/topbar/`,
  `src/components/dock/` y `src/components/launchpad/`. Cada vista pone `body` transparente.
- **Iconos SF Symbols**: `src/components/topbar/SymbolIcons.jsx` con SVG propios (no copiar SF Symbols de Apple; dibujos
  originales con el mismo lenguaje: `viewBox 0 0 24 24`, trazo 1.6–1.8, extremos redondeados, `currentColor`, 15 px):
  `WifiIcon(level 0-3, off)`, `BluetoothIcon(off)`, `SpeakerIcon(level 0-3, muted)`, `BatteryIcon(pct, charging)`
  (cuerpo redondeado con relleno proporcional y rayo), `SearchIcon`, `ControlCenterIcon` (dos interruptores
  apilados), `RecordIcon` (círculo + punto), `CameraIcon` (captura), `BellIcon`. Alineación vertical perfecta
  (`display:block`), separación 14 px, resaltado redondeado al pasar el cursor (`rgba(255,255,255,.12)`).
- **TopBar.jsx**: izquierda vacía (o el logo de Noki pequeño que abre los Ajustes); derecha, de izquierda a derecha:
  CPU, RAM (opcionales), Bluetooth, Sonido, Wi-Fi, Batería + `%`, Lupa, Centro de Control, Grabar, Captura, Campana,
  Avatar (20×20, `border-radius:50%`, `object-fit:cover`; si no hay foto, la inicial como respaldo) y fecha/hora con el
  formato «jue. 12 nov. 11:26 a. m.» (`Intl` es-ES con hora de 12 h; en inglés «Thu Nov 12 11:26 AM»). Zona central
  reservada para el notch (ancho `appearance.collapsed_w` + orejas): no se pinta contenido ahí. Texto 13 px, peso 500,
  color blanco o casi negro según el tinte.
- **BarPanel.jsx**: un componente por `kind` siguiendo `MiDock\docs\tarea-punto2.md` §C (Wi-Fi con lista de redes,
  Bluetooth, Sonido con deslizador/silencio/dispositivo, Batería, Usuario con bloquear/cerrar sesión, Centro de Control
  con rejilla de mosaicos y deslizadores de pantalla y sonido). Interruptores verdes macOS (#34C759). Mide su alto y
  llama a `resize_bar_panel`. Esc → `close_bar_panel`.
- **Dock.jsx**: píldora redondeada translúcida, iconos `icon_size`, magnificación coseno/gaussiana con alcance ≈ 3
  iconos y suavizado por frame (`requestAnimationFrame` solo mientras el cursor está encima o la animación no ha
  terminado), punto indicador bajo apps abiertas, separador antes de carpetas y papelera, tooltip con el nombre, clic
  izquierdo/central/derecho como en `MiDock\docs\diseno.md`, arrastrar para reordenar ancladas, soltar archivos sobre la
  papelera. Primer elemento «Aplicaciones» (icono generado: cuadrado redondeado con rejilla 3×3 de colores) → `toggle_launchpad`.
- **Launchpad.jsx**: según `MiDock\docs\tarea-punto3.md` §2 (búsqueda, chips de categorías, rejilla, orden A-Z / fecha,
  carga perezosa de iconos, Enter lanza el primero, Esc borra búsqueda o cierra, clic en el fondo cierra, clic derecho
  Anclar/Quitar del Dock). Fondo oscuro translúcido con desenfoque CSS de su propio contenido; animación de entrada 150 ms.
- **SettingsWindow.jsx**: nuevas secciones «Barra superior» y «Dock» en la barra lateral (mismos componentes `Row`,
  `Toggle`, `Slider`, `Select` que ya usa). En «Barra superior»: activar, alto, tinte, interruptores de cada icono y
  avatar (vista previa circular + «Elegir imagen…» con `tauri-plugin-dialog` + «Usar la de Windows»). En «Dock»:
  todos los campos de `Dock`, lista de ancladas con quitar/reordenar y «Añadir programa…».
  **Ojo:** hay cambios sin confirmar del usuario en `SettingsWindow.jsx`; trabajar encima de ellos, no revertirlos.

## 5. Fases
1. Ajustes + `topbar.rs` + ventana `topbar` + notch owned/centrado + quitar la franja con la barra activa.
2. `TopBar.jsx` + iconos + avatar + `barpanel` y sus comandos.
3. `dock.rs` + `Dock.jsx` + `launchpad.rs` + `Launchpad.jsx`.
4. Ajustes unificados + `cargo check` + `npm run build` + `cargo build --release` (o `npm run tauri build`).
