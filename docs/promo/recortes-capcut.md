# Recortes y encuadres para CapCut (9:16)

Proyecto: **1080 × 1920**, 30 fps. Las capturas de `assets/promo/*.jpg` miden **3840 × 2160** (las coordenadas son píxeles de esa imagen).

## Cómo aplicarlo en CapCut

1. **Lienzo:** relación 9:16.
2. **Capa de fondo (pista inferior):** la misma captura, **Escala 320 %**, Posición (0, 0), **Desenfoque al máximo** y una capa negra al 30 % encima. Así nunca se ven bordes vacíos.
3. **Capa principal (pista superior):** la captura, ajustada al ancho del lienzo (1080 × 607,5 a **Escala 100 %**; si CapCut la rellena en vez de ajustarla, usa *Ajustar*). Pon **Escala**, **Posición X** y **Posición Y** de cada fila como fotogramas clave (◆), uno por fila de la tabla.
4. **Convención de signos:** X positivo = derecha, Y positivo = arriba. *Si el encuadre se mueve al revés en vertical, invierte el signo de Y.*
5. **Interpolación:** *Suave / Ease in-out* entre fotogramas clave.

## Fórmulas (por si quieres ajustar)

Con *w* = ancho visible de la captura en px y (cx, cy) el punto que quieres en pantalla:

- Escala % = 3840 ÷ *w* × 100
- Posición X = −(cx − 1920) × 1080 ÷ *w*
- Posición Y (arriba +) = (cy − 1080) × 1080 ÷ *w* + 180

El punto (cx, cy) queda en x = 540 e **y = 780 px** del lienzo: deja libre la parte inferior (y ≥ 1250) para los subtítulos y respeta la zona superior de TikTok/Reels.

## Zonas útiles (px de la captura 3840×2160)

| Captura | Zona clave |
|---|---|
| Capturas con portátil | Notch: x 930–2910. Alto (y): 192 → 1212 (panel, temas, notas, música), 1272 (calendario, sistema), 1302 (grabando), 1362 (tareas), 1422 (transcripción), 1452 (hábitos) |
| `panel` | Clima: x 978–2010, y 384–1164 · Interruptores y apps: x 2034–2862, y 384–1164 |
| `system` | Tarjeta: x 978–2862, y 483–1227 · Gráficas CPU y red: x 1266–2826, y 678–870 |
| `habits` | Lista de hábitos: x 1014–2814, y 516–1401 |
| `notes` | Lista de notas: x 1014–1446, y 519–1131 · Editor: x 1470–2826, y 519–1074 |
| `planner` | Entrada y lista: x 1014–2850, y 519–1242 |
| `calendar` | Mes: x 978–1920, y 384–1224 · «Hoy»: x 1944–2862 |
| `music-lyrics` | Carátula y título: x 978–1900, y 384–552 · Barra: x 978–1602, y 762–774 · Controles: x 1230–1350, y 1044–1164 · Letra: x 1650–2754, y 492–1032 |
| `recorder-text` | Reproductor: x 1518–2826, y 519–759 · Botones: y 783–858 · Texto: y 972–1329 |
| `closed-recording` | Punto rojo y tiempo: x 1513–1796, y 935–1045 |
| `welcome` | Cara: centro (1920, 375) · Título: y 855–1092 · Chips: y 1410–1797 |
| `themes` | Filas de temas: y 420–672, 714–966, 1008–1260, 1302–1554, 1596–1848 (franja: x 756–3660) |
| `shortcuts` | Ventana: x 420–3420, y 72–2088 · Atajos: x 2628–3108 · Interruptores: x 3144–3276 |

## Plano a plano

### A · 0:00–0:03,2 · `CLIP REAL 1 (tuyo): el notch abriéndose`

**Clip real.** Grábalo con **OBS** (*Captura de pantalla*) o con **Recortes** (`Win + Mayús + S`, modo vídeo). **La Barra de juegos (`Win + G`) no sirve**: solo graba la ventana de una app, no el escritorio.

- Graba una **zona vertical 9:16 arriba al centro de la pantalla** (por ejemplo **900 × 1600**) con el notch dentro. Pasa el cursor para que se abra y se cierre. 4–5 s bastan.
- En CapCut: escala para llenar el lienzo (≈ **120 %** si grabaste 900×1600) y **Posición Y ≈ −550** para bajar el notch hacia y ≈ 780 (el hueco de arriba lo cubre la capa de fondo desenfocada).
- Usa los primeros **3,2 s**, con la apertura hacia el segundo 1.

### B · 0:03,2–0:09,2 · `es-welcome.jpg`

| Tiempo | Centro (x, y) | Ancho visible | Escala | Posición X | Posición Y | Zona visible: esquina sup.-izq. → tamaño |
|---|---|---|---|---|---|---|
| 0:03,2 | (1920, 375) | 900 | **427 %** | 0 | -666 | (1470, -275) → 900×1600 |
| 0:06,0 | (1920, 375) | 1400 | **274 %** | 0 | -364 | (1220, -636) → 1400×2489 |
| 0:09,2 | (1920, 1010) | 2700 | **142 %** | 0 | 152 | (570, -940) → 2700×4800 |

### C · 0:09,2–0:11,8 · `es-panel.jpg`

| Tiempo | Centro (x, y) | Ancho visible | Escala | Posición X | Posición Y | Zona visible: esquina sup.-izq. → tamaño |
|---|---|---|---|---|---|---|
| 0:09,2 | (1920, 702) | 2160 | **178 %** | 0 | -9 | (840, -858) → 2160×3840 |
| 0:10,6 | (1494, 774) | 1150 | **334 %** | 400 | -107 | (919, -57) → 1150×2044 |
| 0:11,8 | (2448, 774) | 1100 | **349 %** | -518 | -120 | (1898, -20) → 1100×1956 |

### D · 0:11,8–0:13,0 · `es-system.jpg`

| Tiempo | Centro (x, y) | Ancho visible | Escala | Posición X | Posición Y | Zona visible: esquina sup.-izq. → tamaño |
|---|---|---|---|---|---|---|
| 0:11,8 | (1920, 732) | 2160 | **178 %** | 0 | 6 | (840, -828) → 2160×3840 |
| 0:12,6 | (1920, 855) | 1900 | **202 %** | 0 | 52 | (970, -517) → 1900×3378 |

### E · 0:13,0–0:14,0 · `es-habits.jpg`

| Tiempo | Centro (x, y) | Ancho visible | Escala | Posición X | Posición Y | Zona visible: esquina sup.-izq. → tamaño |
|---|---|---|---|---|---|---|
| 0:13,0 | (1920, 822) | 2160 | **178 %** | 0 | 51 | (840, -738) → 2160×3840 |
| 0:13,8 | (1914, 960) | 1900 | **202 %** | 3 | 112 | (964, -412) → 1900×3378 |

### F · 0:14,0–0:15,4 · `es-notes.jpg`

| Tiempo | Centro (x, y) | Ancho visible | Escala | Posición X | Posición Y | Zona visible: esquina sup.-izq. → tamaño |
|---|---|---|---|---|---|---|
| 0:14,0 | (1920, 702) | 2160 | **178 %** | 0 | -9 | (840, -858) → 2160×3840 |
| 0:15,0 | (1920, 800) | 1900 | **202 %** | 0 | 21 | (970, -572) → 1900×3378 |

### G · 0:15,4–0:16,8 · `es-planner.jpg`

| Tiempo | Centro (x, y) | Ancho visible | Escala | Posición X | Posición Y | Zona visible: esquina sup.-izq. → tamaño |
|---|---|---|---|---|---|---|
| 0:15,4 | (1920, 777) | 2160 | **178 %** | 0 | 28 | (840, -783) → 2160×3840 |
| 0:16,2 | (1932, 880) | 1900 | **202 %** | -7 | 66 | (982, -492) → 1900×3378 |

### H · 0:16,8–0:18,6 · `es-calendar.jpg`

| Tiempo | Centro (x, y) | Ancho visible | Escala | Posición X | Posición Y | Zona visible: esquina sup.-izq. → tamaño |
|---|---|---|---|---|---|---|
| 0:16,8 | (1920, 732) | 2160 | **178 %** | 0 | 6 | (840, -828) → 2160×3840 |
| 0:17,8 | (1920, 804) | 2000 | **192 %** | 0 | 31 | (920, -640) → 2000×3556 |

### I · 0:18,6–0:22,4 · `es-music-lyrics.jpg`

| Tiempo | Centro (x, y) | Ancho visible | Escala | Posición X | Posición Y | Zona visible: esquina sup.-izq. → tamaño |
|---|---|---|---|---|---|---|
| 0:18,6 | (1920, 702) | 2160 | **178 %** | 0 | -9 | (840, -858) → 2160×3840 |
| 0:19,8 | (1062, 630) | 1000 | **384 %** | 927 | -306 | (562, -92) → 1000×1778 |
| 0:21,0 | (2200, 780) | 1300 | **295 %** | -233 | -69 | (1550, -159) → 1300×2311 |

### J · 0:22,4–0:24,4 · `CLIP REAL 2 (tuyo): Alt+R y punto rojo`

**Clip real.** Misma técnica y encuadre que el clip A, con el notch **cerrado**. Pulsa `Alt + R`, deja 2 s grabando (se ve el punto rojo con el tiempo) y para. Usa **2 s**.
- Sincroniza el golpe de tecla con el *beep* y la voz («pulsas un atajo»).

### K · 0:24,4–0:25,8 · `es-closed-recording.jpg`

| Tiempo | Centro (x, y) | Ancho visible | Escala | Posición X | Posición Y | Zona visible: esquina sup.-izq. → tamaño |
|---|---|---|---|---|---|---|
| 0:24,4 | (1920, 1080) | 3450 | **111 %** | 0 | 180 | (195, -1412) → 3450×6133 |
| 0:25,0 | (1654, 990) | 900 | **427 %** | 319 | 72 | (1204, 340) → 900×1600 |

### L · 0:25,8–0:29,0 · `es-recorder-text.jpg`

| Tiempo | Centro (x, y) | Ancho visible | Escala | Posición X | Posición Y | Zona visible: esquina sup.-izq. → tamaño |
|---|---|---|---|---|---|---|
| 0:25,8 | (1920, 807) | 2160 | **178 %** | 0 | 44 | (840, -753) → 2160×3840 |
| 0:26,8 | (2172, 1150) | 1500 | **256 %** | -181 | 230 | (1422, 67) → 1500×2667 |
| 0:28,0 | (2250, 820) | 1400 | **274 %** | -255 | -21 | (1550, -191) → 1400×2489 |

### M · 0:29,0–0:31,2 · `es-themes.jpg`

| Tiempo | Centro (x, y) | Ancho visible | Escala | Posición X | Posición Y | Zona visible: esquina sup.-izq. → tamaño |
|---|---|---|---|---|---|---|
| 0:29,0 | (1500, 560) | 2600 | **148 %** | 174 | -36 | (200, -1318) → 2600×4622 |
| 0:30,2 | (2300, 1000) | 2600 | **148 %** | -158 | 147 | (1000, -878) → 2600×4622 |

### N1 · 0:31,2–0:31,85 · `es-theme-halloween.jpg`

| Tiempo | Centro (x, y) | Ancho visible | Escala | Posición X | Posición Y | Zona visible: esquina sup.-izq. → tamaño |
|---|---|---|---|---|---|---|
| inicio | (1920, 702) | 2160 | **178 %** | 0 | -9 | (840, -858) → 2160×3840 |
| fin | (1920, 702) | 1900 | **202 %** | 0 | -35 | (970, -670) → 1900×3378 |

### N2 · 0:31,85–0:32,5 · `es-theme-christmas.jpg`

| Tiempo | Centro (x, y) | Ancho visible | Escala | Posición X | Posición Y | Zona visible: esquina sup.-izq. → tamaño |
|---|---|---|---|---|---|---|
| inicio | (1920, 702) | 2160 | **178 %** | 0 | -9 | (840, -858) → 2160×3840 |
| fin | (1920, 702) | 1900 | **202 %** | 0 | -35 | (970, -670) → 1900×3378 |

### N3 · 0:32,5–0:33,15 · `es-theme-birthday.jpg`

| Tiempo | Centro (x, y) | Ancho visible | Escala | Posición X | Posición Y | Zona visible: esquina sup.-izq. → tamaño |
|---|---|---|---|---|---|---|
| inicio | (1920, 702) | 2160 | **178 %** | 0 | -9 | (840, -858) → 2160×3840 |
| fin | (1920, 702) | 1900 | **202 %** | 0 | -35 | (970, -670) → 1900×3378 |

### N4 · 0:33,15–0:33,8 · `es-theme-love.jpg`

| Tiempo | Centro (x, y) | Ancho visible | Escala | Posición X | Posición Y | Zona visible: esquina sup.-izq. → tamaño |
|---|---|---|---|---|---|---|
| inicio | (1920, 702) | 2160 | **178 %** | 0 | -9 | (840, -858) → 2160×3840 |
| fin | (1920, 702) | 1900 | **202 %** | 0 | -35 | (970, -670) → 1900×3378 |

### O · 0:33,8–0:34,9 · `es-look-color.jpg`

| Tiempo | Centro (x, y) | Ancho visible | Escala | Posición X | Posición Y | Zona visible: esquina sup.-izq. → tamaño |
|---|---|---|---|---|---|---|
| inicio | (1920, 702) | 2160 | **178 %** | 0 | -9 | (840, -858) → 2160×3840 |
| fin | (1920, 702) | 1900 | **202 %** | 0 | -35 | (970, -670) → 1900×3378 |

### P · 0:34,9–0:36,0 · `es-look-image.jpg`

| Tiempo | Centro (x, y) | Ancho visible | Escala | Posición X | Posición Y | Zona visible: esquina sup.-izq. → tamaño |
|---|---|---|---|---|---|---|
| inicio | (1920, 702) | 2160 | **178 %** | 0 | -9 | (840, -858) → 2160×3840 |
| fin | (1920, 702) | 1900 | **202 %** | 0 | -35 | (970, -670) → 1900×3378 |

### Q · 0:36,0–0:38,4 · `es-shortcuts.jpg`

| Tiempo | Centro (x, y) | Ancho visible | Escala | Posición X | Posición Y | Zona visible: esquina sup.-izq. → tamaño |
|---|---|---|---|---|---|---|
| 0:36,0 | (1920, 1080) | 3100 | **124 %** | 0 | 180 | (370, -1159) → 3100×5511 |
| 0:37,0 | (2300, 900) | 2300 | **167 %** | -178 | 95 | (1150, -761) → 2300×4089 |
| 0:37,8 | (2952, 800) | 900 | **427 %** | -1238 | -156 | (2502, 150) → 900×1600 |

### R · 0:38,4–0:44,0 · `es-welcome.jpg`

| Tiempo | Centro (x, y) | Ancho visible | Escala | Posición X | Posición Y | Zona visible: esquina sup.-izq. → tamaño |
|---|---|---|---|---|---|---|
| 0:38,4 | (1920, 400) | 900 | **427 %** | 0 | -636 | (1470, -250) → 900×1600 |
| 0:40,4 | (1920, 1000) | 2800 | **137 %** | 0 | 149 | (520, -1022) → 2800×4978 |

> Un valor **negativo** en «Zona visible» significa que esa parte queda fuera de la captura: la cubre la capa de fondo desenfocada.

## Capa de subtítulos

- Importa `subtitulos-es.srt` (*Texto → Subtítulos locales → Importar*). 1 línea, **fuente negrita grande**, trazo negro, **centro-bajo (y ≈ 1350–1450 px)**.
- Color de acento (`#34D6B6` o `#8B5CF6`) para: **notch**, **Noki**, **gratis**, **atajo**, **transcribe**, **tuyo**.
- Evita los ≈ 250 px inferiores (botones de la app de redes).
- **Los tiempos del SRT son una estimación** (≈ 2,8 palabras/s). Cuando tengas la voz de ElevenLabs, usa *Subtítulos automáticos* de CapCut sobre ese audio o estira los bloques para que coincidan.
