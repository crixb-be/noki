# Recortes y encuadres para CapCut (9:16)

Proyecto: **1080 × 1920**, 30 fps. Todas las capturas de `assets/promo/*.jpg` miden **3840 × 2160** (las coordenadas están en píxeles de esa imagen).

## Cómo aplicarlo en CapCut

1. **Lienzo:** relación 9:16.
2. **Capa de fondo (pista inferior):** la misma captura, **Escala 320 %**, Posición (0, 0), **Desenfoque al máximo** y una capa negra al 30 % encima. Así nunca se ven bordes vacíos.
3. **Capa principal (pista superior):** la captura, con la imagen ajustada al ancho del lienzo (1080 × 607,5 a **Escala 100 %**; si CapCut la rellena en vez de ajustarla, usa *Ajustar*). Después pon **Escala**, **Posición X** y **Posición Y** de cada fila como fotogramas clave (◆), uno por fila de la tabla.
4. **Convención de signos:** X positivo = derecha, Y positivo = arriba. *Si el encuadre se mueve al revés en vertical, invierte el signo de Y.*
5. **Interpolación:** *Suave / Ease in-out* entre fotogramas clave. Así el zoom se siente fluido sin que se note el truco.

## Fórmulas (por si quieres ajustar)

Con *w* = ancho visible de la captura en px, y (cx, cy) el punto que quieres en pantalla:

- Escala % = 3840 ÷ *w* × 100
- Posición X = −(cx − 1920) × 1080 ÷ *w*
- Posición Y (arriba +) = (cy − 1080) × 1080 ÷ *w* + 180

El punto (cx, cy) queda en el lienzo en x = 540 e **y = 780 px** (arriba) a propósito: deja libre la parte inferior (y ≥ 1250) para los subtítulos y respeta la zona superior de TikTok/Reels.

## Zonas útiles de las capturas (px de la captura 3840×2160)

| Captura | Zona clave |
|---|---|
| Capturas con portátil (`panel`, `planner`, `calendar`, `recorder*`, `welcome-app`, `theme-*`) | Notch: x 930–2910. Alto: y 192–1212 (panel y temas), 1362 (planner), 1272 (calendario), 1302 (grabando), 1422 (transcripción) |
| `panel` | Clima: x 978–2010, y 384–1164 · Interruptores y apps: x 2034–2862, y 384–1164 |
| `planner` | Entrada de tarea y lista: x 1014–2850, y 519–1242 |
| `calendar` | Mes: x 978–1920, y 384–1224 · Panel «Hoy»: x 1944–2862 |
| `recorder-text` | Reproductor: x 1518–2826, y 519–759 · Botones: y 783–858 · Texto: y 972–1329 |
| `closed-recording` | Punto rojo y tiempo: x 1513–1796, y 935–1045 |
| `welcome` | Cara: centro (1920, 375) · Título: y 855–1092 · Chips: y 1410–1797 |
| `themes` | Filas de temas: y 420–672, 714–966, 1008–1260, 1302–1554, 1596–1848 (imagen de la franja: x 756–3660) |
| `shortcuts` | Ventana: x 420–3420, y 72–2088 · Atajos: x 2628–3108 · Interruptores: x 3144–3276 |

## Plano a plano

### A · 0:00–0:03,4 · `CLIP REAL 1 (tuyo): el notch abriéndose`

**Clip real.** Grábalo con **OBS** (*Captura de pantalla*) o con **Recortes** (`Win + Mayús + S`, modo vídeo). **La Barra de juegos (`Win + G`) no sirve**: solo graba la ventana de una app, no el escritorio.

- Graba una **zona vertical 9:16 arriba al centro de la pantalla** (por ejemplo **900 × 1600**) con el notch dentro. Pasa el cursor para que se abra y se cierre. 4–5 s bastan.
- En CapCut: escala para llenar el lienzo (≈ **120 %** si grabaste 900×1600) y **Posición Y ≈ −550** para bajar el notch hacia y ≈ 780 (el hueco de arriba lo cubre la capa de fondo desenfocada).
- Usa los primeros **3,4 s**, con la apertura hacia el segundo 1.

### B · 0:03,4–0:09,6 · `es-welcome.jpg`

| Tiempo | Centro en la captura (x, y) | Ancho visible | Escala | Posición X | Posición Y | Zona visible: esquina sup.-izq. → tamaño |
|---|---|---|---|---|---|---|
| 0:03,4 | (1920, 375) | 900 | **427 %** | 0 | -666 | (1470, -275) → 900×1600 |
| 0:06,5 | (1920, 375) | 1400 | **274 %** | 0 | -364 | (1220, -636) → 1400×2489 |
| 0:09,6 | (1920, 1010) | 2700 | **142 %** | 0 | 152 | (570, -940) → 2700×4800 |

### C · 0:09,6–0:14,0 · `es-panel.jpg`

| Tiempo | Centro en la captura (x, y) | Ancho visible | Escala | Posición X | Posición Y | Zona visible: esquina sup.-izq. → tamaño |
|---|---|---|---|---|---|---|
| 0:09,6 | (1920, 702) | 2160 | **178 %** | 0 | -9 | (840, -858) → 2160×3840 |
| 0:11,4 | (1494, 774) | 1150 | **334 %** | 400 | -107 | (919, -57) → 1150×2044 |
| 0:12,8 | (2448, 774) | 1100 | **349 %** | -518 | -120 | (1898, -20) → 1100×1956 |

### D · 0:14,0–0:16,3 · `es-planner.jpg`

| Tiempo | Centro en la captura (x, y) | Ancho visible | Escala | Posición X | Posición Y | Zona visible: esquina sup.-izq. → tamaño |
|---|---|---|---|---|---|---|
| 0:14,0 | (1920, 777) | 2160 | **178 %** | 0 | 28 | (840, -783) → 2160×3840 |
| 0:15,0 | (1932, 880) | 1900 | **202 %** | -7 | 66 | (982, -492) → 1900×3378 |

### E · 0:16,3–0:18,6 · `es-calendar.jpg`

| Tiempo | Centro en la captura (x, y) | Ancho visible | Escala | Posición X | Posición Y | Zona visible: esquina sup.-izq. → tamaño |
|---|---|---|---|---|---|---|
| 0:16,3 | (1920, 732) | 2160 | **178 %** | 0 | 6 | (840, -828) → 2160×3840 |
| 0:17,3 | (1920, 804) | 2000 | **192 %** | 0 | 31 | (920, -640) → 2000×3556 |

### F · 0:18,6–0:21,0 · `CLIP REAL 2 (tuyo): Alt+R y punto rojo`

**Clip real.** Misma técnica y mismo encuadre que el clip A, con el notch **cerrado**. Pulsa `Alt + R`, deja 2 s grabando (se ve el punto rojo con el tiempo) y para. Usa **2,4 s**.
- Sugerencia: sincroniza el golpe de tecla con el texto «ALT + R» y el «beep».

### G · 0:21,0–0:22,4 · `es-closed-recording.jpg`

| Tiempo | Centro en la captura (x, y) | Ancho visible | Escala | Posición X | Posición Y | Zona visible: esquina sup.-izq. → tamaño |
|---|---|---|---|---|---|---|
| 0:21,0 | (1920, 1080) | 3450 | **111 %** | 0 | 180 | (195, -1412) → 3450×6133 |
| 0:22,0 | (1654, 990) | 900 | **427 %** | 319 | 72 | (1204, 340) → 900×1600 |

### H · 0:22,4–0:28,2 · `es-recorder-text.jpg`

| Tiempo | Centro en la captura (x, y) | Ancho visible | Escala | Posición X | Posición Y | Zona visible: esquina sup.-izq. → tamaño |
|---|---|---|---|---|---|---|
| 0:22,4 | (1920, 807) | 2160 | **178 %** | 0 | 44 | (840, -753) → 2160×3840 |
| 0:24,0 | (2172, 1150) | 1500 | **256 %** | -181 | 230 | (1422, 67) → 1500×2667 |
| 0:25,6 | (2250, 820) | 1400 | **274 %** | -255 | -21 | (1550, -191) → 1400×2489 |

### I · 0:28,2–0:31,2 · `es-themes.jpg`

| Tiempo | Centro en la captura (x, y) | Ancho visible | Escala | Posición X | Posición Y | Zona visible: esquina sup.-izq. → tamaño |
|---|---|---|---|---|---|---|
| 0:28,2 | (1500, 560) | 2600 | **148 %** | 174 | -36 | (200, -1318) → 2600×4622 |
| 0:29,7 | (2300, 1000) | 2600 | **148 %** | -158 | 147 | (1000, -878) → 2600×4622 |

### J1 · 0:31,2–0:32,05 · `es-theme-halloween.jpg`

| Tiempo | Centro en la captura (x, y) | Ancho visible | Escala | Posición X | Posición Y | Zona visible: esquina sup.-izq. → tamaño |
|---|---|---|---|---|---|---|
| inicio | (1920, 702) | 2160 | **178 %** | 0 | -9 | (840, -858) → 2160×3840 |
| fin | (1920, 702) | 1900 | **202 %** | 0 | -35 | (970, -670) → 1900×3378 |

### J2 · 0:32,05–0:32,9 · `es-theme-christmas.jpg`

| Tiempo | Centro en la captura (x, y) | Ancho visible | Escala | Posición X | Posición Y | Zona visible: esquina sup.-izq. → tamaño |
|---|---|---|---|---|---|---|
| inicio | (1920, 702) | 2160 | **178 %** | 0 | -9 | (840, -858) → 2160×3840 |
| fin | (1920, 702) | 1900 | **202 %** | 0 | -35 | (970, -670) → 1900×3378 |

### J3 · 0:32,9–0:33,75 · `es-theme-birthday.jpg`

| Tiempo | Centro en la captura (x, y) | Ancho visible | Escala | Posición X | Posición Y | Zona visible: esquina sup.-izq. → tamaño |
|---|---|---|---|---|---|---|
| inicio | (1920, 702) | 2160 | **178 %** | 0 | -9 | (840, -858) → 2160×3840 |
| fin | (1920, 702) | 1900 | **202 %** | 0 | -35 | (970, -670) → 1900×3378 |

### J4 · 0:33,75–0:34,6 · `es-theme-love.jpg`

| Tiempo | Centro en la captura (x, y) | Ancho visible | Escala | Posición X | Posición Y | Zona visible: esquina sup.-izq. → tamaño |
|---|---|---|---|---|---|---|
| inicio | (1920, 702) | 2160 | **178 %** | 0 | -9 | (840, -858) → 2160×3840 |
| fin | (1920, 702) | 1900 | **202 %** | 0 | -35 | (970, -670) → 1900×3378 |

### K · 0:34,6–0:38,6 · `es-shortcuts.jpg`

| Tiempo | Centro en la captura (x, y) | Ancho visible | Escala | Posición X | Posición Y | Zona visible: esquina sup.-izq. → tamaño |
|---|---|---|---|---|---|---|
| 0:34,6 | (1920, 1080) | 3100 | **124 %** | 0 | 180 | (370, -1159) → 3100×5511 |
| 0:36,0 | (2300, 900) | 2300 | **167 %** | -178 | 95 | (1150, -761) → 2300×4089 |
| 0:37,6 | (2952, 800) | 900 | **427 %** | -1238 | -156 | (2502, 150) → 900×1600 |

### L · 0:38,6–0:42,0 · `es-welcome.jpg`

| Tiempo | Centro en la captura (x, y) | Ancho visible | Escala | Posición X | Posición Y | Zona visible: esquina sup.-izq. → tamaño |
|---|---|---|---|---|---|---|
| 0:38,6 | (1920, 400) | 900 | **427 %** | 0 | -636 | (1470, -250) → 900×1600 |
| 0:40,6 | (1920, 1000) | 2800 | **137 %** | 0 | 149 | (520, -1022) → 2800×4978 |

> Un valor **negativo** en «Zona visible» significa que esa parte queda fuera de la captura: la cubre la capa de fondo desenfocada.

## Capa de subtítulos

- Importa `subtitulos-es.srt` (*Texto → Subtítulos locales → Importar*). 1 línea, **fuente negrita grande**, trazo negro, **centro-bajo (y ≈ 1350–1450 px)**.
- Pon en color de acento (`#34D6B6` o `#8B5CF6`) estas palabras: **notch**, **Noki**, **gratis**, **Alt + R**, **transcribe**, **tuyo**, **atajos**.
- Los subtítulos no deben pisar los botones de la interfaz de la app de redes (zona inferior ≈ 250 px).
