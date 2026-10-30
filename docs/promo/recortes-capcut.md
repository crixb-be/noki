# Recortes y encuadres para CapCut (9:16)

Proyecto: **1080 × 1920**, 30 fps. Las capturas de `assets/promo/*.jpg` miden **3840 × 2160** (coordenadas en píxeles de esa imagen).

## Cómo aplicarlo en CapCut

1. **Lienzo:** relación 9:16.
2. **Capa de fondo (pista inferior):** la misma captura, **Escala 320 %**, Posición (0, 0), **Desenfoque al máximo** y una capa negra al 30 % encima. Así nunca se ven bordes vacíos.
3. **Capa principal (pista superior):** la captura ajustada al ancho del lienzo (1080 × 607,5 a **Escala 100 %**; si CapCut la rellena en vez de ajustarla, usa *Ajustar*). Pon **Escala**, **Posición X** y **Posición Y** de cada fila como fotogramas clave (◆).
4. **Signos:** X positivo = derecha, Y positivo = arriba. *Si el encuadre se mueve al revés en vertical, invierte el signo de Y.*
5. **Interpolación:** *Suave / Ease in-out* entre fotogramas clave.

## Fórmulas

Con *w* = ancho visible en px y (cx, cy) el punto que quieres en pantalla:

- Escala % = 3840 ÷ *w* × 100
- Posición X = −(cx − 1920) × 1080 ÷ *w*
- Posición Y (arriba +) = (cy − 1080) × 1080 ÷ *w* + 320

El punto (cx, cy) queda en x = 540 e **y = 640 px** del lienzo (33 % desde arriba): el contenido va en la mitad superior y deja libre la inferior para los subtítulos.

## Zonas útiles (px de la captura 3840×2160)

| Captura | Zona clave |
|---|---|
| Capturas con portátil | Notch: x 930–2910. Alto (y): 192 → 1212 (panel, temas, notas, música, archivos, traductor, IA, Pomodoro), 1272 (calendario, sistema), 1302 (grabando), 1362 (tareas), 1422 (transcripción), 1452 (hábitos) |
| `panel` | Clima: x 978–2010, y 384–1164 · Interruptores y apps: x 2034–2862, y 384–1164 |
| `system` | Tarjeta: x 978–2862, y 483–1227 · Gráficas: x 1266–2826, y 678–870 |
| `habits` | Lista: x 1014–2814, y 516–1401 |
| `notes` | Lista: x 1014–1446, y 519–1131 · Editor: x 1470–2826, y 519–1074 |
| `planner` | Entrada y lista: x 1014–2850, y 519–1242 |
| `calendar` | Mes: x 978–1920, y 384–1224 · «Hoy»: x 1944–2862 |
| `music-lyrics` | Carátula y título: x 978–1900, y 384–552 · Controles: x 1230–1350, y 1044–1164 · Letra: x 1650–2754, y 492–1032 |
| `recorder-text` | Reproductor: x 1518–2826, y 519–759 · Botones: y 783–858 · Texto: y 972–1329 |
| `closed-recording` | Punto rojo y tiempo: x 1513–1796, y 935–1045 |
| `welcome` | Cara: centro (1920, 375) · Título: y 855–1092 · Chips: y 1410–1797 |
| `themes` | Filas: y 420–672, 714–966, 1008–1260, 1302–1554, 1596–1848 (franja: x 756–3660) |
| `shortcuts` | Ventana: x 420–3420, y 72–2088 · Atajos: x 2628–3108 · Interruptores: x 3144–3276 |
| `files`, `translator`, `ai`, `pomodoro` | Contenido del notch: x 1014–2826, y 384–1164 (centro ≈ 1920, 820) |

## Plano a plano

### A · 0:00–0:03,0 · `CLIP REAL 1 (tuyo): el notch abriéndose`

**Clip real.** Grábalo con **OBS** (*Captura de pantalla*) o con **Recortes** (`Win + Mayús + S`, modo vídeo). **La Barra de juegos (`Win + G`) no sirve**: solo graba la ventana de una app, no el escritorio.

- Graba una **zona vertical 9:16 arriba al centro de la pantalla** (por ejemplo **900 × 1600**) con el notch dentro. Pasa el cursor para que se abra y se cierre. 4–5 s bastan.
- En CapCut: escala para llenar el lienzo (≈ **120 %** si grabaste 900×1600) y **Posición Y ≈ −435** para bajar el notch hacia y ≈ 640 (el hueco de arriba lo cubre la capa de fondo desenfocada).
- Usa los primeros **3 s**, con la apertura hacia el segundo 1.

### B · 0:03,0–0:10,6 · `es-welcome.jpg`

| Tiempo | Centro (x, y) | Ancho visible | Escala | Posición X | Posición Y | Zona visible: esquina sup.-izq. → tamaño |
|---|---|---|---|---|---|---|
| 0:03,0 | (1920, 375) | 900 | **427 %** | 0 | -526 | (1470, -158) → 900×1600 |
| 0:06,5 | (1920, 375) | 1400 | **274 %** | 0 | -224 | (1220, -455) → 1400×2489 |
| 0:10,6 | (1920, 1010) | 2700 | **142 %** | 0 | 292 | (570, -590) → 2700×4800 |

### C · 0:10,6–0:12,4 · `es-panel.jpg`

| Tiempo | Centro (x, y) | Ancho visible | Escala | Posición X | Posición Y | Zona visible: esquina sup.-izq. → tamaño |
|---|---|---|---|---|---|---|
| 0:10,6 | (1920, 702) | 2160 | **178 %** | 0 | 131 | (840, -578) → 2160×3840 |
| 0:11,5 | (1494, 774) | 1150 | **334 %** | 400 | 33 | (919, 93) → 1150×2044 |
| 0:12,4 | (2448, 774) | 1100 | **349 %** | -518 | 20 | (1898, 122) → 1100×1956 |

### D · 0:12,4–0:14,0 · `es-system.jpg`

| Tiempo | Centro (x, y) | Ancho visible | Escala | Posición X | Posición Y | Zona visible: esquina sup.-izq. → tamaño |
|---|---|---|---|---|---|---|
| 0:12,4 | (1920, 732) | 2160 | **178 %** | 0 | 146 | (840, -548) → 2160×3840 |
| 0:13,2 | (1920, 855) | 1900 | **202 %** | 0 | 192 | (970, -271) → 1900×3378 |

### E · 0:14,0–0:15,6 · `es-habits.jpg`

| Tiempo | Centro (x, y) | Ancho visible | Escala | Posición X | Posición Y | Zona visible: esquina sup.-izq. → tamaño |
|---|---|---|---|---|---|---|
| 0:14,0 | (1920, 822) | 2160 | **178 %** | 0 | 191 | (840, -458) → 2160×3840 |
| 0:14,8 | (1914, 960) | 1900 | **202 %** | 3 | 252 | (964, -166) → 1900×3378 |

### F · 0:15,6–0:16,9 · `es-notes.jpg`

| Tiempo | Centro (x, y) | Ancho visible | Escala | Posición X | Posición Y | Zona visible: esquina sup.-izq. → tamaño |
|---|---|---|---|---|---|---|
| 0:15,6 | (1920, 702) | 2160 | **178 %** | 0 | 131 | (840, -578) → 2160×3840 |
| 0:16,3 | (1920, 800) | 1900 | **202 %** | 0 | 161 | (970, -326) → 1900×3378 |

### G · 0:16,9–0:18,1 · `es-planner.jpg`

| Tiempo | Centro (x, y) | Ancho visible | Escala | Posición X | Posición Y | Zona visible: esquina sup.-izq. → tamaño |
|---|---|---|---|---|---|---|
| 0:16,9 | (1920, 777) | 2160 | **178 %** | 0 | 168 | (840, -503) → 2160×3840 |
| 0:17,5 | (1932, 880) | 1900 | **202 %** | -7 | 206 | (982, -246) → 1900×3378 |

### H · 0:18,1–0:19,4 · `es-calendar.jpg`

| Tiempo | Centro (x, y) | Ancho visible | Escala | Posición X | Posición Y | Zona visible: esquina sup.-izq. → tamaño |
|---|---|---|---|---|---|---|
| 0:18,1 | (1920, 732) | 2160 | **178 %** | 0 | 146 | (840, -548) → 2160×3840 |
| 0:18,8 | (1920, 804) | 2000 | **192 %** | 0 | 171 | (920, -381) → 2000×3556 |

### I · 0:19,4–0:23,0 · `es-music-lyrics.jpg`

| Tiempo | Centro (x, y) | Ancho visible | Escala | Posición X | Posición Y | Zona visible: esquina sup.-izq. → tamaño |
|---|---|---|---|---|---|---|
| 0:19,4 | (1920, 702) | 2160 | **178 %** | 0 | 131 | (840, -578) → 2160×3840 |
| 0:20,6 | (1062, 630) | 1000 | **384 %** | 927 | -166 | (562, 37) → 1000×1778 |
| 0:21,8 | (2200, 780) | 1300 | **295 %** | -233 | 71 | (1550, 10) → 1300×2311 |

### J · 0:23,0–0:25,0 · `CLIP REAL 2 (tuyo): Alt+R y punto rojo`

**Clip real.** Misma técnica y encuadre que el clip A, con el notch **cerrado**. Pulsa `Alt + R`, deja 2 s grabando (se ve el punto rojo con el tiempo) y para. Usa **2 s**.
- Sincroniza el golpe de tecla con el *beep* y la voz («pulsas un atajo»).

### K · 0:25,0–0:26,2 · `es-closed-recording.jpg`

| Tiempo | Centro (x, y) | Ancho visible | Escala | Posición X | Posición Y | Zona visible: esquina sup.-izq. → tamaño |
|---|---|---|---|---|---|---|
| 0:25,0 | (1920, 1080) | 3450 | **111 %** | 0 | 320 | (195, -964) → 3450×6133 |
| 0:25,6 | (1654, 990) | 900 | **427 %** | 319 | 212 | (1204, 457) → 900×1600 |

### L · 0:26,2–0:28,0 · `es-recorder-text.jpg`

| Tiempo | Centro (x, y) | Ancho visible | Escala | Posición X | Posición Y | Zona visible: esquina sup.-izq. → tamaño |
|---|---|---|---|---|---|---|
| 0:26,2 | (1920, 807) | 2160 | **178 %** | 0 | 184 | (840, -473) → 2160×3840 |
| 0:27,0 | (2172, 1150) | 1500 | **256 %** | -181 | 370 | (1422, 261) → 1500×2667 |
| 0:27,5 | (2250, 820) | 1400 | **274 %** | -255 | 119 | (1550, -10) → 1400×2489 |

### M · 0:28,0–0:30,2 · `es-files.jpg`

| Tiempo | Centro (x, y) | Ancho visible | Escala | Posición X | Posición Y | Zona visible: esquina sup.-izq. → tamaño |
|---|---|---|---|---|---|---|
| 0:28,0 | (1920, 702) | 2160 | **178 %** | 0 | 131 | (840, -578) → 2160×3840 |
| 0:29,0 | (1920, 820) | 1900 | **202 %** | 0 | 172 | (970, -306) → 1900×3378 |

### N · 0:30,2–0:31,6 · `es-translator.jpg`

| Tiempo | Centro (x, y) | Ancho visible | Escala | Posición X | Posición Y | Zona visible: esquina sup.-izq. → tamaño |
|---|---|---|---|---|---|---|
| 0:30,2 | (1920, 702) | 2160 | **178 %** | 0 | 131 | (840, -578) → 2160×3840 |
| 0:30,9 | (1920, 820) | 1900 | **202 %** | 0 | 172 | (970, -306) → 1900×3378 |

### O · 0:31,6–0:32,8 · `es-ai.jpg`

| Tiempo | Centro (x, y) | Ancho visible | Escala | Posición X | Posición Y | Zona visible: esquina sup.-izq. → tamaño |
|---|---|---|---|---|---|---|
| 0:31,6 | (1920, 702) | 2160 | **178 %** | 0 | 131 | (840, -578) → 2160×3840 |
| 0:32,2 | (1920, 820) | 1900 | **202 %** | 0 | 172 | (970, -306) → 1900×3378 |

### P · 0:32,8–0:34,0 · `es-pomodoro.jpg`

| Tiempo | Centro (x, y) | Ancho visible | Escala | Posición X | Posición Y | Zona visible: esquina sup.-izq. → tamaño |
|---|---|---|---|---|---|---|
| 0:32,8 | (1920, 702) | 2160 | **178 %** | 0 | 131 | (840, -578) → 2160×3840 |
| 0:33,4 | (1920, 820) | 1900 | **202 %** | 0 | 172 | (970, -306) → 1900×3378 |

### Q · 0:34,0–0:36,2 · `es-themes.jpg`

| Tiempo | Centro (x, y) | Ancho visible | Escala | Posición X | Posición Y | Zona visible: esquina sup.-izq. → tamaño |
|---|---|---|---|---|---|---|
| 0:34,0 | (1500, 560) | 2600 | **148 %** | 174 | 104 | (200, -981) → 2600×4622 |
| 0:35,1 | (2300, 1000) | 2600 | **148 %** | -158 | 287 | (1000, -541) → 2600×4622 |

### R · 0:36,2–0:37,3 · `es-look-image.jpg`

| Tiempo | Centro (x, y) | Ancho visible | Escala | Posición X | Posición Y | Zona visible: esquina sup.-izq. → tamaño |
|---|---|---|---|---|---|---|
| inicio | (1920, 702) | 2160 | **178 %** | 0 | 131 | (840, -578) → 2160×3840 |
| fin | (1920, 702) | 1900 | **202 %** | 0 | 105 | (970, -424) → 1900×3378 |

### S · 0:37,3–0:38,3 · `es-look-color.jpg`

| Tiempo | Centro (x, y) | Ancho visible | Escala | Posición X | Posición Y | Zona visible: esquina sup.-izq. → tamaño |
|---|---|---|---|---|---|---|
| inicio | (1920, 702) | 2160 | **178 %** | 0 | 131 | (840, -578) → 2160×3840 |
| fin | (1920, 702) | 1900 | **202 %** | 0 | 105 | (970, -424) → 1900×3378 |

### T · 0:38,3–0:40,4 · `es-shortcuts.jpg`

| Tiempo | Centro (x, y) | Ancho visible | Escala | Posición X | Posición Y | Zona visible: esquina sup.-izq. → tamaño |
|---|---|---|---|---|---|---|
| 0:38,3 | (1920, 1080) | 3100 | **124 %** | 0 | 320 | (370, -757) → 3100×5511 |
| 0:39,2 | (2300, 900) | 2300 | **167 %** | -178 | 235 | (1150, -463) → 2300×4089 |
| 0:39,9 | (2952, 800) | 900 | **427 %** | -1238 | -16 | (2502, 267) → 900×1600 |

### U · 0:40,4–0:43,4 · `es-welcome.jpg`

| Tiempo | Centro (x, y) | Ancho visible | Escala | Posición X | Posición Y | Zona visible: esquina sup.-izq. → tamaño |
|---|---|---|---|---|---|---|
| 0:40,4 | (1920, 400) | 900 | **427 %** | 0 | -496 | (1470, -133) → 900×1600 |
| 0:42,0 | (1920, 1000) | 2800 | **137 %** | 0 | 289 | (520, -659) → 2800×4978 |

> Un valor **negativo** en «Zona visible» significa que esa parte queda fuera de la captura: la cubre la capa de fondo desenfocada.

## Línea guía (mismo eje en todo el vídeo)

| Elemento | Eje vertical en el lienzo 1080 × 1920 | Posición Y en CapCut (centro = 0, arriba +) |
|---|---|---|
| **Punto de interés de cada captura** (cara, notch, tarjeta que enseñas) | **y = 640 px** (33 %) | **+320** |
| **Subtítulos** (centro del texto) | **y = 1320 px** (69 %) | **−360** |
| Zona a evitar arriba (interfaz de TikTok/Reels) | 0–150 px | — |
| Zona a evitar abajo (descripción, botones) | 1540–1920 px | — |

Con esto el contenido nunca llega más abajo de ≈ y 960 y los subtítulos quedan ≥ 300 px por debajo, así no se pegan a la captura en ningún plano.

## Subtítulos

- Importa `subtitulos-es.srt` (*Texto → Subtítulos locales → Importar*). En el estilo del texto pon **Posición X 0** y **Posición Y −360**, y activa **«Aplicar a todos»** para que todos los subtítulos usen el mismo eje. El `.srt` no puede guardar la posición, por eso se fija en CapCut una sola vez.
- 1 línea (máximo 2), **fuente negrita grande**, trazo negro, alineado al centro.
- Color de acento (`#34D6B6` o `#8B5CF6`): **notch**, **Noki**, **gratis**, **atajo**, **transcribe**, **tuyo**.
- **Los tiempos del SRT son una estimación** (≈ 2,8 palabras/s). Con la voz de ElevenLabs ya hecha, usa *Subtítulos automáticos* de CapCut sobre ese audio, o estira los bloques.

## Si ya tienes el vídeo montado con el eje anterior (y = 780)

1. **Capturas:** selecciona todos los clips de captura y **sube 140 px**: suma **+140** a la *Posición Y* de **cada fotograma clave** (si lo haces con la imagen sin fotogramas clave, súmalo a la posición fija). Los clips reales A y J: súmales también +140.
2. **Subtítulos:** pon una sola vez **Posición Y = −360** y aplica a todos.
3. Comprueba el primer plano (`welcome`, «¿Y si tu PC tuviera…»): con el eje nuevo el subtítulo queda 300 px bajo la carita.

