# Graph Report - noki-dock  (2026-11-12)

## Corpus Check
- 109 files · ~104,128 words
- Verdict: corpus is large enough that graph structure adds value.
- Unclassified: 6 file(s) not represented in the graph (top: (none) 3, .icns 1, .ico 1)

## Summary
- 1734 nodes · 3969 edges · 88 communities (55 shown, 33 thin omitted)
- Extraction: 99% EXTRACTED · 1% INFERRED · 0% AMBIGUOUS · INFERRED: 37 edges (avg confidence: 0.71)
- Token cost: 0 input · 0 output

## Graph Freshness
- Built from commit: `bfb40b3a`
- Run `git rev-parse HEAD` and compare to check if the graph is stale.
- Run `graphify update .` after code changes (no API cost).

## Community Hubs (Navigation)
- SettingsWindow.jsx
- translate.rs
- dropzone.rs
- quick.rs
- package.json
- ai.rs
- Recorder.jsx
- calendar.rs
- bundle
- notch.rs
- statuswatch.rs
- App.jsx
- clipboard.rs
- lib.rs
- viz.rs
- system.rs
- screentime.rs
- Settings
- icons.rs
- sysmon.rs
- dayKey
- ExpandedView.jsx
- topbar.rs
- useStore
- media.rs
- create_panel
- sync
- TopBar.jsx
- spotlight.rs
- Store
- settings.rs
- fileops.rs
- barPaint.jsx
- RwLock
- launcher.rs
- SystemView.jsx
- run
- State
- items
- audio_and_brightness
- MediaHandle
- ambient.js
- set_avatar_image
- ProviderCfg
- sync
- MediaPlayer.jsx
- Lcd
- contact_links
- default.json
- FileShelf.jsx
- W
- render
- dock.rs
- noki
- backup_export_blocking
- Mutex
- window_app_user_model_id
- SysHandle

## God Nodes (most connected - your core abstractions)
1. `react` - 55 edges
2. `Settings` - 52 edges
3. `useStore()` - 46 edges
4. `Notch()` - 40 edges
5. `@tauri-apps/api` - 40 edges
6. `NotchState` - 32 edges
7. `dayKey()` - 23 edges
8. `run_loop()` - 23 edges
9. `SettingsWindow()` - 20 edges
10. `Row()` - 19 edges

## Surprising Connections (you probably didn't know these)
- `dock_items()` --references--> `NotchState`  [INFERRED]
  src-tauri/src/dock.rs → src-tauri/src/notch.rs
- `dock_activate()` --references--> `NotchState`  [INFERRED]
  src-tauri/src/dock.rs → src-tauri/src/notch.rs
- `dock_close_windows()` --references--> `NotchState`  [INFERRED]
  src-tauri/src/dock.rs → src-tauri/src/notch.rs
- `backup_export()` --references--> `Store`  [INFERRED]
  src-tauri/src/lib.rs → src-tauri/src/store.rs
- `store_all()` --references--> `Store`  [INFERRED]
  src-tauri/src/lib.rs → src-tauri/src/store.rs

## Import Cycles
- None detected.

## Communities (88 total, 33 thin omitted)

### Community 0 - "SettingsWindow.jsx"
Cohesion: 0.10
Nodes (56): NokiLogo(), About(), Ai(), Alerts(), Appearance(), Avatar(), CalendarSettings(), ColorPicker() (+48 more)

### Community 1 - "translate.rs"
Cohesion: 0.08
Nodes (27): bg_data(), bg_data_blocking(), bg_import(), bg_import_blocking(), dir(), EXTENSIONS, MAX_BYTES, mime() (+19 more)

### Community 2 - "dropzone.rs"
Cohesion: 0.07
Nodes (7): DragEvent, ensure(), NotchDropTarget, NotchDropTarget_Impl, paths_of(), registered_ptr(), TARGETS

### Community 3 - "quick.rs"
Cohesion: 0.07
Nodes (14): dark_enabled(), endpoint_volume(), KEEP_AWAKE, LAST_RADIO, muted(), quick_set(), quick_state(), QuickState (+6 more)

### Community 4 - "package.json"
Cohesion: 0.06
Nodes (37): author, dependencies, @crabnebula/tauri-plugin-drag@^2.1.0, @huggingface/transformers@^4.3.2, react@^18.3.1, react-dom@^18.3.1, @tauri-apps/api@^2.1.1, @tauri-apps/plugin-dialog@^2.8.3 (+29 more)

### Community 5 - "ai.rs"
Cohesion: 0.13
Nodes (22): ai_cancel(), ai_chat(), AiJobs, base(), build(), ChatMsg, chunk_text(), client() (+14 more)

### Community 6 - "Recorder.jsx"
Cohesion: 0.11
Nodes (32): Detail(), fmtDur(), Live(), Options(), Recorder(), shape(), toBase64(), BARS (+24 more)

### Community 7 - "calendar.rs"
Cohesion: 0.11
Nodes (28): Acc, calendar_fetch(), CalResult, DAYS_AHEAD, days_in_month(), download(), Dt, Event (+20 more)

### Community 8 - "bundle"
Cohesion: 0.05
Nodes (38): app, security, windows, withGlobalTauri, build, beforeBuildCommand, beforeDevCommand, devUrl (+30 more)

### Community 9 - "notch.rs"
Cohesion: 0.05
Nodes (42): remove(), ABE_TOP, ABM_NEW, ABM_QUERYPOS, ABM_REMOVE, ABM_SETPOS, appbar_remove(), appbar_remove_win() (+34 more)

### Community 10 - "statuswatch.rs"
Cohesion: 0.09
Nodes (15): PERSONALIZE, alert(), bluetooth_loop(), connected_bluetooth(), MIC_KEY, mic_loop(), mic_users(), open_key() (+7 more)

### Community 11 - "App.jsx"
Cohesion: 0.08
Nodes (45): react, @tauri-apps/api, App(), isTextField(), Notch(), TABS, AIChat(), useBattery() (+37 more)

### Community 12 - "clipboard.rs"
Cohesion: 0.09
Nodes (12): CF_UNICODETEXT, clip_clear(), clip_copy(), clip_list(), clip_remove(), Entry, History, MAX_CHARS (+4 more)

### Community 13 - "lib.rs"
Cohesion: 0.06
Nodes (8): ALLOWED_HOSTS, detect_city(), DetectedCity, file_size(), FileInfo, list_monitors(), MonitorDto, TrayDnd

### Community 14 - "viz.rs"
Cohesion: 0.09
Nodes (13): a_tone_lights_the_matching_band(), bands(), BANDS, Capture, default_device_id(), drain(), EMIT_EVERY, fft() (+5 more)

### Community 15 - "system.rs"
Cohesion: 0.07
Nodes (11): audio_output_name(), AudioDevice, DisplayBrightness, Endpoint, endpoint(), friendly_name(), Hud, IOCTL_VIDEO_QUERY_DISPLAY_BRIGHTNESS (+3 more)

### Community 16 - "screentime.rs"
Cohesion: 0.11
Nodes (14): add(), CLEARED, enabled(), flush(), FLUSH_EVERY, foreground_app(), IDLE_LIMIT, idle_secs() (+6 more)

### Community 17 - "Settings"
Cohesion: 0.10
Nodes (10): Clipboard, General, Health, Hud, Modules, Productivity, Settings, TopBar (+2 more)

### Community 19 - "sysmon.rs"
Cohesion: 0.19
Nodes (15): DiskInfo, end_task(), end_task_blocking(), is_protected(), ProcInfo, PROTECTED, protects_system_and_self(), Stats (+7 more)

### Community 20 - "dayKey"
Cohesion: 0.18
Nodes (21): blank(), Editor(), HabitsView(), ICONS, PRESETS, AppList(), Bars(), colorOf() (+13 more)

### Community 21 - "ExpandedView.jsx"
Cohesion: 0.15
Nodes (22): Battery(), CalendarView(), dayLabel(), monthTitle(), Clock(), Costume(), Eyes(), isNight() (+14 more)

### Community 22 - "topbar.rs"
Cohesion: 0.04
Nodes (44): ABE_TOP, ABM_NEW, ABM_QUERYPOS, ABM_REMOVE, ABM_SETPOS, APPBAR_HWND, appbar_hwnd(), AVATAR_CACHE (+36 more)

### Community 23 - "useStore"
Cohesion: 0.10
Nodes (29): AppIcon(), cache, colorOf(), COLORS, failedAt, fetchIcon(), wait(), ACTIONS (+21 more)

### Community 24 - "media.rs"
Cohesion: 0.07
Nodes (31): apply(), ArtCache, Cmd, Next, Pause, Play, Prev, Seek (+23 more)

### Community 25 - "create_panel"
Cohesion: 0.13
Nodes (21): create(), sync(), close_spotlight(), create(), ensure(), focus(), position(), spotlight_resize() (+13 more)

### Community 26 - "sync"
Cohesion: 0.13
Nodes (26): appbar(), AppBarState, child_callback(), commit_pin(), create(), dock_pin(), dock_reorder(), dock_resize() (+18 more)

### Community 27 - "TopBar.jsx"
Cohesion: 0.06
Nodes (67): AppsIcon(), Dock(), apply(), measureRest(), onMouseMove(), resetMagnification(), start(), tick() (+59 more)

### Community 28 - "spotlight.rs"
Cohesion: 0.07
Nodes (23): close_launchpad(), ensure(), focus_window(), open(), open_launchpad(), position(), remove_noactivate(), toggle_launchpad() (+15 more)

### Community 29 - "Store"
Cohesion: 0.23
Nodes (4): backup_import(), backup_import_blocking(), Store, StoreInner

### Community 30 - "settings.rs"
Cohesion: 0.11
Nodes (18): shortcut_list(), Appearance, Calendar, CalSource, config_dir(), DEFAULT_UPDATE_URL, Dock, DockFolder (+10 more)

### Community 31 - "fileops.rs"
Cohesion: 0.05
Nodes (36): apply(), ATARDECER, AURORA, DEFAULT, every_icon_decodes(), image(), OCEANO, PASTEL_ATARDECER (+28 more)

### Community 32 - "barPaint.jsx"
Cohesion: 0.28
Nodes (12): StripDecor(), BarPaint(), useBarPaint(), cache, fillFor(), rgba(), useBgImage(), inRange() (+4 more)

### Community 33 - "RwLock"
Cohesion: 0.33
Nodes (4): app_icon(), cache(), CACHE, warm_paths()

### Community 34 - "launcher.rs"
Cohesion: 0.21
Nodes (11): AppCache, AppEntry, EXTENSIONS, launch_app(), launch_app_blocking(), launcher_apps(), launcher_list(), SHELL_PREFIX (+3 more)

### Community 35 - "SystemView.jsx"
Cohesion: 0.29
Nodes (12): clean(), gb(), mb(), Performance(), Processes(), rate(), Ring(), Spark() (+4 more)

### Community 36 - "run"
Cohesion: 0.29
Nodes (13): apply_autostart(), apply_shortcut(), build_tray(), commit_settings(), data_dir(), open_settings(), open_settings_async(), reset_settings() (+5 more)

### Community 37 - "State"
Cohesion: 0.32
Nodes (11): backup_export(), get_settings(), notch_state(), request_open(), set_greeting(), set_media_pill(), set_peek(), set_pinned() (+3 more)

### Community 38 - "items"
Cohesion: 0.20
Nodes (15): dock_activate(), dock_close_windows(), dock_items(), dock_launch(), dock_launch_blocking(), DockItem, file_name(), identity() (+7 more)

### Community 39 - "audio_and_brightness"
Cohesion: 0.22
Nodes (9): audio_and_brightness(), device_kind(), keyboard(), keyboard_language(), start(), SysCmd, Brightness, Mute (+1 more)

### Community 40 - "MediaHandle"
Cohesion: 0.32
Nodes (5): media_control(), media_seek(), media_state(), MediaHandle, MediaState

### Community 41 - "ambient.js"
Cohesion: 0.09
Nodes (38): DAY, fmtMin(), FocusStats(), hourLabel(), lastDays(), Ctl(), ZEN_EXTRA_H, ZenPill() (+30 more)

### Community 42 - "set_avatar_image"
Cohesion: 0.33
Nodes (11): data_url(), encode_avatar_png(), extract_embedded_images(), image_data(), is_jpeg(), is_png(), read_image(), save_avatar_png() (+3 more)

### Community 43 - "ProviderCfg"
Cohesion: 0.47
Nodes (4): Ai, LegacyModels, provider(), ProviderCfg

### Community 46 - "sync"
Cohesion: 0.16
Nodes (24): apply_glass(), apply_glass_mode(), check_z_order(), close_bar_panel_impl(), command_bar_panel_state(), command_close_bar_panel(), command_open_bar_panel(), command_resize_bar_panel() (+16 more)

### Community 48 - "MediaPlayer.jsx"
Cohesion: 0.17
Nodes (18): LyricsPanel(), MediaPlayer(), useArtColor(), Ctl(), Eq(), Icon(), MediaPill(), PILL_EXTRA_H (+10 more)

### Community 53 - "contact_links"
Cohesion: 0.33
Nodes (5): blank_issues_enabled, contact_links, about, name, url

### Community 56 - "default.json"
Cohesion: 0.33
Nodes (5): description, identifier, permissions, $schema, windows

### Community 57 - "FileShelf.jsx"
Cohesion: 0.46
Nodes (7): addPaths(), baseName(), dragIcon(), FileShelf(), fmtSize(), iconFor(), ICONS

### Community 59 - "W"
Cohesion: 0.33
Nodes (3): add_to_zip(), zip_files(), zip_files_blocking()

### Community 61 - "render"
Cohesion: 0.21
Nodes (10): bitmap_png(), fallback_png(), glyph(), hicon_png(), png_data(), render(), resolve_program(), shell_extension_png() (+2 more)

### Community 66 - "dock.rs"
Cohesion: 0.04
Nodes (16): ABE_BOTTOM, ABM_GETSTATE, ABM_NEW, ABM_QUERYPOS, ABM_REMOVE, ABM_SETPOS, ABM_SETSTATE, ABS_AUTOHIDE (+8 more)

### Community 76 - "backup_export_blocking"
Cohesion: 0.50
Nodes (3): backup_export_blocking(), broadcast_notify(), store_all()

### Community 79 - "Mutex"
Cohesion: 0.33
Nodes (7): APPBAR, dock_set_hit(), Hit, HIT, hit(), SIZE, TASKBAR_STATE

### Community 83 - "window_app_user_model_id"
Cohesion: 0.33
Nodes (6): app_user_model_id(), link_targets(), process_app_user_model_id(), process_image_path(), utf16_string(), window_app_user_model_id()

### Community 87 - "SysHandle"
Cohesion: 0.29
Nodes (4): set_brightness(), set_mute(), set_volume(), SysHandle

## Knowledge Gaps
- **12 isolated node(s):** `Next`, `Pause`, `Play`, `Prev`, `Seek` (+7 more)
  These have ≤1 connection - possible missing edges. (Counts symbols only; 501 node(s) total have ≤1 connection when file, concept and rationale nodes are included.)
- **33 thin communities (<3 nodes) omitted from report** — run `graphify query` to explore isolated nodes.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `react` connect `App.jsx` to `SettingsWindow.jsx`, `barPaint.jsx`, `SystemView.jsx`, `package.json`, `Recorder.jsx`, `ambient.js`, `MediaPlayer.jsx`, `dayKey`, `ExpandedView.jsx`, `useStore`, `FileShelf.jsx`, `TopBar.jsx`?**
  _High betweenness centrality (0.022) - this node is a cross-community bridge._
- **What connects `Next`, `Pause`, `Play` to the rest of the system?**
  _12 weakly-connected nodes found - possible missing edges._
- **Should `SettingsWindow.jsx` be split into smaller, more focused modules?**
  _Cohesion score 0.09643483343074226 - nodes in this community are weakly interconnected._
- **Why does `Settings` connect `Settings` to `dock.rs`, `run`, `State`, `items`, `notch.rs`, `set_avatar_image`, `ProviderCfg`, `backup_export_blocking`, `lib.rs`, `sync`, `topbar.rs`, `create_panel`, `sync`, `spotlight.rs`, `Store`, `settings.rs`?**
  _High betweenness centrality (0.020) - this node is a cross-community bridge._
- **Should `translate.rs` be split into smaller, more focused modules?**
  _Cohesion score 0.07535460992907801 - nodes in this community are weakly interconnected._
- **Why does `NotchState` connect `State` to `translate.rs`, `ai.rs`, `items`, `calendar.rs`, `notch.rs`, `set_avatar_image`, `clipboard.rs`, `lib.rs`, `screentime.rs`, `Settings`, `topbar.rs`, `Store`?**
  _High betweenness centrality (0.013) - this node is a cross-community bridge._
- **Should `dropzone.rs` be split into smaller, more focused modules?**
  _Cohesion score 0.06866002214839424 - nodes in this community are weakly interconnected._