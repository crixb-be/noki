<div align="center">

<img src="assets/noki-logo.svg" alt="Noki" width="120" />

# Noki

**A friendly notch for your Windows desktop.**
Music, tasks, files, calendar, weather, quick toggles, a fast AI chat and a translator — all inside a small, customizable notch at the top of your screen.

[English](README.md) · [Español](README.es.md)

[![License: GPL v3](https://img.shields.io/badge/license-GPL--3.0--or--later-blue.svg)](LICENSE)
![Platform](https://img.shields.io/badge/platform-Windows%2010%20%7C%2011-0078D4)
[![Ko-fi](https://img.shields.io/badge/support-Ko--fi-ff5e5b)](https://ko-fi.com/cristhianbillybustamante)

</div>

---

## What is Noki?

Noki sits at the top of your screen like the notch of a Mac. Hover over it (or press `Alt+N`) and it opens into a small
dashboard. Other windows work *below* it, like with a taskbar, and it hides by itself when you go full screen.

It is free, open source and runs fully on your PC. No account, no telemetry.

## Features

| | |
|---|---|
| 🏠 **Panel** | One-tap toggles (Wi-Fi, Bluetooth, keep awake, dark mode, microphone, sound, Do not disturb, screenshot, lock), current weather with hourly and daily forecast, and your favorite apps with their real icons |
| 🎵 **Music** | Shows and controls whatever is playing (Spotify, YouTube Music, VLC, browser…), album art, seek bar, synced lyrics and an audio visualizer that reacts to your real sound |
| ✅ **Planner** | Tasks with date, time, category and reminders (type “call 15m” or “meeting 14:30”), quick notes and a Pomodoro timer |
| 💧 **Habits** | Daily habits with a goal, streaks and reminders — drink water, take a pill, walk, read… Turn each habit (or the whole module) on or off |
| 🖥️ **System** | CPU, RAM, disks and network in real time, and the programs using the most memory or CPU (with a button to close them) |
| ⏱️ **Screen time** | *Optional, off by default.* How long you use each app today and over 7 days. Stays on your PC; ignores time when you are away |
| 📁 **Files** | Drop files on the notch to keep them on a shelf; drag them back out, compress, extract, copy, move or send them to the recycle bin |
| 📅 **Calendar** | Month view with your tasks and events (Google, Outlook or any iCal link) and alerts before meetings |
| ✨ **AI chat** | A quick chat for fast answers (Gemini, Claude, OpenAI-compatible or local Ollama) that keeps answering in the background. Bring your own API key |
| 🧰 **Tools** | App launcher, clipboard history (opt-in) and a **translator** for text and images — no API key needed |
| 🎨 **Your design** | Notch color or image, automatic color that follows the app below it, a strip beside the notch, choose what shows when closed (time, date, tasks, temperature, battery…) |
| 🌐 **Languages** | English and Spanish |
| 🔔 **Alerts** | Volume, brightness, charger, low battery, internet, audio device, Caps Lock, keyboard language, health reminders |
| 🖥️ **Multi-monitor** | Optional one notch per screen |
| 🔄 **Updates** | Signed updates you approve yourself |

## Install

1. Download `Noki_<version>_x64-setup.exe` from the [Releases](../../releases) page.
2. Run it. It installs for your user only (no administrator rights). If Microsoft Edge WebView2 is missing, the installer downloads it.
3. Open **Noki** from the Start menu. Its icon appears in the tray: left click = Settings, right click = menu.

> **Windows SmartScreen:** the installer is not yet signed with a paid code-signing certificate, so Windows may show
> “Windows protected your PC”. Choose **More info → Run anyway**. You can always build it yourself from source (below).

Your settings live in `%APPDATA%\com.noki.app`. API keys are stored in the **Windows Credential Manager**, never in files or backups.

### Coming from “Mi Notch”?
Noki is the new name of Mi Notch. On first launch it copies your settings, data and saved keys over automatically.
You can uninstall the old version afterwards.

## Privacy

Everything stays on your PC. Noki only goes online when you use something that needs it:

| Feature | Service | When |
|---|---|---|
| Weather | [Open-Meteo](https://open-meteo.com) | when a city is set |
| Text translation | Google Translate web service (unofficial endpoint) | while you translate |
| Image text (OCR) | Windows built-in OCR, **local** | nothing leaves your PC |
| Lyrics | [LRCLIB](https://lrclib.net) | when you open lyrics |
| Detect my city | [ipwho.is](https://ipwho.is) | only when you press the button |
| AI chat | the provider you choose, with *your* key | when you send a message |
| Calendars | the iCal links you add | every 15 minutes |
| Updates | the address you configure (e.g. GitHub Releases) | on start / every 6 h, if enabled |

## Build from source

Requirements: Windows 10/11, [Node.js 20+](https://nodejs.org), [Rust (stable)](https://rustup.rs) and the
Visual Studio Build Tools with the “Desktop development with C++” workload.

```bash
npm install
npm run tauri dev      # development mode with hot reload
npm run tauri build    # builds the NSIS installer (unsigned updater artifacts)
```

To produce a signed installer for auto-updates, create your own updater key
(`npx tauri signer generate -w %USERPROFILE%\.tauri\noki.key`), put its public key in
`src-tauri/tauri.conf.json` and run `scripts\compilar.ps1`. **Never commit your private key.**

Tests: `cd src-tauri && cargo test --lib`.

### How it works
- A fixed-size transparent window per monitor; a Rust thread reads the cursor every 25 ms to decide click-through,
  hover open/close and full-screen hiding.
- With “Reserve space” it registers as a Windows AppBar, so maximized windows stay below the notch.
- Frontend: React 18 + Tailwind, talking to Rust through Tauri commands and events.

| Path | What it does |
|---|---|
| `src-tauri/src/notch.rs` | Position, AppBar, click-through, hover, full screen |
| `src-tauri/src/dropzone.rs` | Native drop target for dragging files onto the notch |
| `src-tauri/src/quick.rs` | Panel toggles (radios, dark mode, audio, keep awake) |
| `src-tauri/src/settings.rs` | Typed settings, validation, atomic saving |
| `src/App.jsx` | The notch (shape, tabs, animation) |
| `src/components/SettingsWindow.jsx` | Settings window |
| `src/lib/en.js` | English translations |

## Contributing

Bug reports and ideas are very welcome — see [CONTRIBUTING.md](CONTRIBUTING.md).

## Credits

- Made by **[Cristhian Bustamante](https://www.instagram.com/cristhian.billy/)** with the help of AI (Claude Code).
- Inspired by the notch of Mac laptops and by desktop “notch” apps — that idea encouraged me to make my own version.
- Built on open-source software; see [THIRD-PARTY-NOTICES.md](THIRD-PARTY-NOTICES.md).

## Support

Noki is free. If you like it, you can [buy me a coffee on Ko-fi](https://ko-fi.com/cristhianbillybustamante) ☕ —
or just follow along: [Instagram](https://www.instagram.com/cristhian.billy/) ·
[Facebook](https://www.facebook.com/cristhian.bustamanteespinoza/) · [X](https://x.com/Cristhian_0702)

## License

Copyright © 2026 Cristhian Bustamante.

Noki is free software: you can redistribute it and/or modify it under the terms of the
[GNU General Public License v3.0 or later](LICENSE). It comes with **no warranty**.
