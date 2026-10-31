# Third-party notices / Avisos de terceros

Noki is licensed under the GNU General Public License v3.0 or later (see [LICENSE](LICENSE)).
It is built on the open-source components below, each under its own license. The full license texts ship with
each package (`node_modules/<name>/LICENSE` and the Rust crate sources); this file lists the direct dependencies.

Noki se distribuye bajo la GNU GPL v3.0 o posterior ([LICENSE](LICENSE)) y está construido sobre los componentes
de código abierto de abajo, cada uno con su propia licencia.

## Rust crates (direct dependencies)

| Crate | Version | License |
|---|---|---|
| [base64](https://github.com/marshallpierce/rust-base64) | 0.23.1 | MIT OR Apache-2.0 |
| [chrono](https://github.com/chronotope/chrono) | 0.4.45 | MIT OR Apache-2.0 |
| [futures-util](https://github.com/rust-lang/futures-rs) | 0.3.34 | MIT OR Apache-2.0 |
| [keyring](https://github.com/hwchen/keyring-rs.git) | 3.6.3 | MIT OR Apache-2.0 |
| [png](https://github.com/image-rs/image-png) | 0.18.1 | MIT OR Apache-2.0 |
| [reqwest](https://github.com/seanmonstar/reqwest) | 0.13.5 | MIT OR Apache-2.0 |
| [serde](https://github.com/serde-rs/serde) | 1.0.229 | MIT OR Apache-2.0 |
| [serde_json](https://github.com/serde-rs/json) | 1.0.154 | MIT OR Apache-2.0 |
| [sysinfo](https://github.com/GuillaumeGomez/sysinfo) | 0.39.6 | MIT |
| [tauri](https://github.com/tauri-apps/tauri) | 2.12.5 | Apache-2.0 OR MIT |
| [tauri-build](https://github.com/tauri-apps/tauri) | 2.7.4 | Apache-2.0 OR MIT |
| [tauri-plugin-autostart](https://github.com/tauri-apps/plugins-workspace) | 2.7.0 | Apache-2.0 OR MIT |
| [tauri-plugin-dialog](https://github.com/tauri-apps/plugins-workspace) | 2.8.3 | Apache-2.0 OR MIT |
| [tauri-plugin-drag](https://crates.io/crates/tauri-plugin-drag) | 2.1.1 | Apache-2.0 OR MIT |
| [tauri-plugin-global-shortcut](https://github.com/tauri-apps/plugins-workspace) | 2.4.1 | Apache-2.0 OR MIT |
| [tauri-plugin-single-instance](https://github.com/tauri-apps/plugins-workspace) | 2.5.4 | Apache-2.0 OR MIT |
| [tauri-plugin-updater](https://github.com/tauri-apps/plugins-workspace) | 2.14.0 | Apache-2.0 OR MIT |
| [tokio](https://github.com/tokio-rs/tokio) | 1.54.1 | MIT |
| [windows](https://github.com/microsoft/windows-rs) | 0.62.2 | MIT OR Apache-2.0 |
| [windows-core](https://github.com/microsoft/windows-rs) | 0.62.2 | MIT OR Apache-2.0 |
| [zip](https://github.com/zip-rs/zip2.git) | 4.6.1 | MIT |

## JavaScript packages (direct dependencies)

| Package | Version | License | Use |
|---|---|---|---|
| [@crabnebula/tauri-plugin-drag](https://www.npmjs.com/package/@crabnebula/tauri-plugin-drag) | 2.1.0 | Apache-2.0 OR MIT (see the tauri-plugin-drag repository) | runtime |
| [@huggingface/transformers](https://www.npmjs.com/package/@huggingface/transformers) | 4.3.2 | Apache-2.0 | runtime (voice transcription) |
| [@tauri-apps/api](https://www.npmjs.com/package/@tauri-apps/api) | 2.12.4 | Apache-2.0 OR MIT | runtime |
| [@tauri-apps/cli](https://www.npmjs.com/package/@tauri-apps/cli) | 2.12.4 | Apache-2.0 OR MIT | build |
| [@tauri-apps/plugin-dialog](https://www.npmjs.com/package/@tauri-apps/plugin-dialog) | 2.8.3 | MIT OR Apache-2.0 | runtime |
| [@vitejs/plugin-react](https://www.npmjs.com/package/@vitejs/plugin-react) | 4.7.0 | MIT | build |
| [autoprefixer](https://www.npmjs.com/package/autoprefixer) | 10.6.1 | MIT | build |
| [postcss](https://www.npmjs.com/package/postcss) | 8.5.31 | MIT | build |
| [onnxruntime-web](https://www.npmjs.com/package/onnxruntime-web) | 1.31.0-dev | MIT | runtime (voice transcription) |
| [react](https://www.npmjs.com/package/react) | 18.3.1 | MIT | runtime |
| [react-dom](https://www.npmjs.com/package/react-dom) | 18.3.1 | MIT | runtime |
| [tailwindcss](https://www.npmjs.com/package/tailwindcss) | 3.4.19 | MIT | build |
| [vite](https://www.npmjs.com/package/vite) | 5.4.21 | MIT | build |

## Icons, data and services / Iconos, datos y servicios

- **Toggle icons (Panel)**: hand-drawn in the style of [Lucide](https://lucide.dev) (ISC License, © Lucide Contributors). / Iconos de los interruptores dibujados al estilo de Lucide (licencia ISC).
- **Weather**: [Open-Meteo](https://open-meteo.com) (CC BY 4.0; free for non-commercial use). / Clima de Open-Meteo.
- **Synced lyrics**: [LRCLIB](https://lrclib.net). / Letras sincronizadas de LRCLIB.
- **Text translation**: the free Google Translate web service (unofficial endpoint, may change or stop working). / Traducción de texto con el servicio web gratuito de Google Translate (punto de acceso no oficial).
- **Image text recognition (OCR)**: the OCR engine built into Windows, run locally. / OCR integrado en Windows, se ejecuta en el equipo.
- **City detection (optional button)**: [ipwho.is](https://ipwho.is). / Detección de ciudad (botón opcional).
- **AI assistant**: your own API key with the provider you choose (Google Gemini, Anthropic Claude, OpenAI-compatible, Ollama). / Asistente de IA con tu propia clave.

"Windows" and "WebView2" are trademarks of Microsoft. Other names are property of their respective owners.

## Speech models (downloaded on demand) / Modelos de voz (se descargan al usarlos)

Voice transcription uses OpenAI Whisper (MIT License) through ONNX conversions published by
[onnx-community](https://huggingface.co/onnx-community) (`whisper-tiny`, `whisper-base`). They are not bundled with Noki:
they are downloaded once from Hugging Face the first time you transcribe, then cached on your PC.

La transcripción de voz usa Whisper de OpenAI (licencia MIT) mediante conversiones ONNX de
[onnx-community](https://huggingface.co/onnx-community). No vienen con Noki: se descargan una vez de Hugging Face la primera
vez que transcribes y quedan guardados en tu PC.
