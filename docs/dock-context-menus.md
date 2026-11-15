# Dock y menús de Windows

Los anclados se renderizan en el orden de `settings.dock.pinned`. Las apps abiertas sin anclar conservan su posición durante la sesión y las nuevas se agregan antes de carpetas y papelera. Activar una ventana no cambia ese orden. Los cambios de orden desde Ajustes siguen siendo explícitos.

El clic derecho en una app abre un menú de vidrio en una ventana WebView de Noki con las categorías disponibles de sus Jump Lists y, debajo, abrir nueva instancia, anclar/desanclar, finalizar tarea y cerrar ventana. El menú y sus submenús comparten la clase `noki-panel-surface`, las variables de `useGlass(settings, "panels")`, el borde de 1 px y el hover de Wi-Fi/Bluetooth. Las listas largas tienen desplazamiento y navegación por teclado. No se inventan tareas de incógnito: se muestran los accesos que publica la aplicación.

`jump_lists.rs` consulta el Shell en modo de lectura, que accede a `%APPDATA%\Microsoft\Windows\Recent\AutomaticDestinations` y `CustomDestinations`. Resuelve el AppUserModelID desde la configuración, ventana o acceso directo registrado. Los destinos conservan sus objetos COM en el hilo STA de la interfaz hasta seleccionar o cerrar el menú; la apertura invoca su comando nativo y conserva argumentos, perfil y directorio de trabajo. No se llama a `BeginList` ni se reescriben los archivos de otra app.

Las interfaces privadas de listas automáticas y categorías se solicitan por su IID de Windows 10/11. Si Windows no las ofrece, se usa `IApplicationDocumentLists` para Recientes/Frecuentes. Las categorías vacías, las apps sin Jump List y los historiales deshabilitados por Windows no generan entradas. Se consultan hasta 30 destinos por categoría automática.

El clic derecho en el fondo del dock o en la barra superior ofrece Terminal (PowerShell, CMD, `wsl.exe -d MADRE`), carpetas conocidas de Windows, aplicaciones y cada ventana abierta, Win+D, configuración de Windows, administrador de tareas y ajustes de la superficie correspondiente.

La ventana contextual usa `HWND_TOPMOST`. Desde el Dock, su borde inferior termina antes del borde superior de la ventana del Dock; todos los submenús quedan dentro de ese mismo espacio. Se cierran con Escape, clic fuera o al cambiar de aplicación.

Cerrar ventana envía `WM_CLOSE` a la ventana elegida y permite los avisos de guardar. Finalizar tarea ejecuta `taskkill.exe /F /PID` sobre los procesos de esa app; para apps hospedadas se seleccionan sus procesos hijos en vez de terminar ApplicationFrameHost. Los errores se muestran en un diálogo nativo.

## Verificación

En Windows, desde `src-tauri`:

```powershell
cargo test --lib --offline
```

El ejecutable autónomo se genera desde la raíz con `npm run tauri -- build --no-bundle -- --offline` (Tauri incluye el frontend y activa `custom-protocol`). `npm run build` comprueba solo el frontend. Las pruebas cubren orden estable, cierre y reapertura de apps, resolución y lectura real de Jump Lists, y disponibilidad del comando nativo de sus entradas sin ejecutarlo.

Comprobación visual: abrir ambos menús de fondo, navegar sus submenús con teclado, abrir el menú de una app que publique tareas y alternar ventanas para confirmar que no se mueven los anclados. Para verificar Finalizar tarea, usar una app de prueba sin documentos pendientes.

Referencias de ABI: [Open-Shell](https://github.com/Open-Shell/Open-Shell-Menu/blob/master/Src/StartMenu/StartMenuDLL/JumpLists.cpp), [System Informer](https://github.com/winsiderss/systeminformer/blob/master/phlib/include/appresolverp.h) y [documentación de GetList](https://learn.microsoft.com/windows/win32/api/shobjidl_core/nf-shobjidl_core-iapplicationdocumentlists-getlist).
