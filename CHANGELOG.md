# Changelog

## [1.4.2]

- La consulta directa de la cuota de Gemini CLI queda desactivada por defecto, también al actualizar desde 1.4.1. Google no permite ese acceso desde otras aplicaciones y puede suspender la cuenta. Quien acepte el riesgo puede activarla en Ajustes → Barra superior.
- La cuota de Gemini se puede leer con la statusline oficial de Antigravity CLI (`agy`): copia el comando desde Ajustes → Barra superior y pégalo dentro de `agy`. Sustituye tu statusline actual. Noki no lee sus credenciales y solo guarda el nombre de cada cuota, su porcentaje, su reinicio y la hora de recepción.
- El panel de uso de IA indica de dónde viene la cuota de Gemini y cuándo se recibió.

## [1.4.1]

- El Dock restaura y enfoca las ventanas minimizadas y evita que un rebote del clic vuelva a minimizarlas.
- Nuevo interruptor «Mostrar Descargas» en Ajustes → Dock, activado por defecto.

## [1.4.0]

- Animaciones opcionales del Dock, desactivadas por defecto. Los iconos rebotan al lanzar una app y el efecto Genie conecta sus ventanas con el icono al minimizar y restaurar.

## [1.3.3]

- Vista previa al pasar el cursor sobre un icono del Dock, con miniaturas, títulos y controles para enfocar o cerrar cada ventana.
- Arrastre de archivos sobre el Dock para enfocar apps y carpetas, abrir archivos con una app, moverlos a una carpeta o enviarlos a la Papelera.
- Nivel y estado de carga de la batería sincronizados en tiempo real entre la barra superior y sus paneles.
- Capturas consecutivas con `PRTSC` sin que la barra de tareas nativa reaparezca ni se recalcule el área de trabajo. El Dock y la barra superior ignoran las superposiciones de captura.
- El recuadro de iconos ocultos de Windows se ancla debajo del botón `^` y conserva su posición ante cambios de Explorer y DWM.

## [1.3.1]

- Corregida la identidad de las apps UWP: ApplicationFrameHost resuelve el proceso real y se descartan ventanas vacías, ocultas y procesos de infraestructura del sistema.
- Papelera protegida: conserva su arte metálico y las ventanas del Explorador abiertas en la Papelera no añaden un icono duplicado.
- Aplicaciones ancladas reordenables por drag-and-drop en Ajustes, con agarre y papelera a la derecha; se mantiene la navegación por teclado.

## 1.3.0 — Barra superior unificada, Dock y Launchpad

- Barra superior unificada con buscador rápido, paneles de la barra y avatar; nuevo Dock y Launchpad para acceder a aplicaciones y ventanas desde un solo lugar.
- Menús contextuales del Dock y la barra superior con el mismo gris translúcido, desenfoque, borde y resaltado de los paneles de Wi-Fi y Bluetooth.
- Menús del Dock por encima de sus iconos, con submenús que se adaptan a los bordes de pantalla, desplazamiento para listas largas y navegación por teclado.
- Jump Lists de Windows con tareas y destinos publicados por cada aplicación; acciones para abrir, anclar, desanclar, cerrar ventanas y finalizar tareas.
- Orden estable de las aplicaciones abiertas y gestión de los anclados desde Ajustes: reordenación por arrastre, nombres de aplicación e iconos de Windows.
- Accesos contextuales a terminales, carpetas, ventanas abiertas, escritorio y ajustes; papelera con icono propio en el Dock.

## 1.2.0 — Themes, recorder and shortcuts / Temas, grabadora y atajos

- **Customizable shortcuts**: each one can be turned on/off and rebound — open/close, record voice, do not disturb, quick task, and jump to tab (Alt+1…9). /
  **Atajos personalizables**: cada uno se enciende/apaga y se cambia — abrir/cerrar, grabar voz, no molestar, tarea rápida e ir a una pestaña (Alt+1…9).
- **Recorder**: search, export audio/text, send a transcript to Notes or Tasks, choose microphone and transcription quality; recording keeps going with the notch closed (red dot). /
  **Grabadora**: búsqueda, exportar audio/texto, pasar una transcripción a Notas o Tareas, elegir micrófono y calidad; sigue grabando con el notch cerrado (punto rojo).
- **Personalization**: your own theme dates, export/import your appearance, and a first-run welcome. / **Personalización**: fechas propias para temas, exportar/importar tu apariencia y una bienvenida la primera vez.
- **Lighter app**: removed ~27 MB of unused transcription engine files. / **App más ligera**: se quitaron ~27 MB de archivos sin uso del motor de transcripción.
- **Seasonal themes**: Halloween, Christmas, Birthday and Love, chosen separately for Noki's face, the center notch and the side strip, with decorations (pumpkins, reindeer, hearts…); automatic by date or chosen in Settings → Appearance. /
  **Temas de temporada**: Halloween, Navidad, Cumpleaños y Amor, a elegir por separado para la cara de Noki, el notch central y la franja de los costados, con adornos (calabazas, renos, corazones…); automáticos por fecha o a elegir en Ajustes → Apariencia.
- **Reorder tabs** in Settings → Modules. / **Reordena las pestañas** en Ajustes → Módulos y dentro de Agenda (botón ⇄).
- **Recorder** in Planner: record your voice, keep the audio on your PC and, optionally, transcribe it locally and for free (Whisper; the ~40 MB model downloads once, then it works offline). /
  **Grabadora** en Agenda: graba tu voz, guarda los audios en tu equipo y, si quieres, transcríbelos en local y gratis (Whisper; el modelo de ~40 MB se descarga una vez y luego funciona sin internet).

## 1.1.0 — Character and new logo / Personaje y logo nuevo

- **New logo** (Ciruela Océano) and a **Noki icon picker** in Settings → Appearance (tray and Settings icon). /
  **Logo nuevo** (Ciruela Océano) y un **selector del icono de Noki** en Ajustes → Apariencia (bandeja y Ajustes).
- **Character in the notch**: a little face that blinks, looks at your cursor, cheers when you finish a task, a habit or a Pomodoro, dances to the music, gets surprised if you click it (dizzy if you click it a lot), opens its mouth when you drop a file, and gets sleepy at night or on low battery. It can be turned off in Settings → Appearance. /
  **Personaje en el notch**: una carita que parpadea, mira tu cursor, se alegra al terminar una tarea, un hábito o un Pomodoro, baila con la música, se sorprende si la pinchas (y se marea si la pinchas mucho), abre la boca al soltar un archivo y se adormece de noche o con poca batería. Se apaga en Ajustes → Apariencia.

## 1.0.2 — App icons / Iconos de apps

- App icons now retry when Windows fails to provide them the first time, instead of staying as a letter until restart. /
  Los iconos de las apps se reintentan si Windows no los da a la primera, en vez de quedarse con la inicial hasta reiniciar.

## 1.0.1 — Translation fixes / Arreglos de traducción

- Fixed leftover Spanish text in the English interface (detected language, pending tasks, clear completed). /
  Corregido texto en español que quedaba en la interfaz en inglés (idioma detectado, tareas pendientes, limpiar completadas).
- Framed screenshots and AI-assistance note in the README. / Capturas con marco y nota de ayuda de IA en el README.

## 1.0.0 — First public release / Primera versión pública

- **Renamed to Noki** (formerly “Mi Notch”), with a new logo. Settings, data and saved API keys migrate automatically. /
  **Ahora se llama Noki** (antes «Mi Notch»), con logo nuevo. Los ajustes, datos y claves guardadas se migran solos.
- Habits with goals, streaks and reminders; System monitor (CPU, RAM, disks, network, top programs); optional Screen time (off by default). /
  Hábitos con meta, rachas y recordatorios; monitor del Sistema (CPU, RAM, discos, red y programas que más consumen); Tiempo de pantalla opcional (apagado por defecto).
- Panel with weather (hourly and daily), quick toggles and favorite apps with real icons. /
  Panel con clima (por horas y días), interruptores rápidos y apps favoritas con iconos reales.
- Music player with synced lyrics and audio visualizer. / Reproductor con letras sincronizadas y visualizador de audio.
- Planner: tasks with date, time and category, notes and Pomodoro. / Agenda: tareas con fecha, hora y categoría, notas y Pomodoro.
- File shelf: drag in and out, compress, extract, copy, move, recycle. / Estante de archivos: arrastrar dentro y fuera, comprimir, extraer, copiar, mover y papelera.
- Calendar with iCal support and meeting alerts. / Calendario con iCal y avisos de reuniones.
- Fast AI chat that keeps answering in the background. / Chat de IA rápido que sigue respondiendo en segundo plano.
- Translator for text and images (no API key; images use Windows OCR). / Traductor de texto e imágenes (sin clave; las imágenes usan el OCR de Windows).
- Appearance: notch color/image, automatic color, side strip, choose what the closed notch shows. /
  Apariencia: color/imagen del notch, color automático, franja lateral y qué se ve con el notch cerrado.
- English and Spanish interface. / Interfaz en español e inglés.
- Developer section in Settings. / Apartado «Desarrollador» en Ajustes.
- Released under GPL-3.0-or-later. / Publicado bajo GPL-3.0 o posterior.
