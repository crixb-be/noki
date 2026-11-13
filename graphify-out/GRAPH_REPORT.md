# Graph Report - noki-dock  (2026-11-13)

## Corpus Check
- 111 files · ~107,563 words
- Verdict: corpus is large enough that graph structure adds value.
- Unclassified: 6 file(s) not represented in the graph (top: (none) 3, .icns 1, .ico 1)

## Summary
- 1787 nodes · 4105 edges · 109 communities (70 shown, 39 thin omitted)
- Extraction: 99% EXTRACTED · 1% INFERRED · 0% AMBIGUOUS · INFERRED: 38 edges (avg confidence: 0.71)
- Token cost: 0 input · 0 output

## Graph Freshness
- Built from commit: `15533405`
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
- backdrop.rs
- FileShelf.jsx
- ExpandedView.jsx
- topbar.rs
- useStore
- sysmon.rs
- create
- sync
- TopBar.jsx
- spotlight.rs
- window_utils.rs
- popups.rs
- fileops.rs
- Productivity.jsx
- blocking_with_timeout
- media.rs
- SystemView.jsx
- run
- NotchState
- barPaint.jsx
- audio_and_brightness
- MediaHandle
- useStore
- set_avatar_image
- Cmd
- launcher.rs
- sync
- appicon.rs
- items
- Lcd
- contact_links
- spotlight_search
- updates.rs
- default.json
- AppHandle
- AppHandle
- create_panel
- render
- Mutex
- dock.rs
- quick_set
- .clamp
- dayKey
- noki
- W
- LPARAM
- Mutex
- window_app_user_model_id
- apply
- RwLock
- ScreenTime.jsx
- mic_loop
- react
- PCWSTR
- collect_networks
- ProviderCfg
- create_piece

## God Nodes (most connected - your core abstractions)
1. `react` - 56 edges
2. `Settings` - 52 edges
3. `useStore()` - 46 edges
4. `@tauri-apps/api` - 41 edges
5. `Notch()` - 40 edges
6. `NotchState` - 32 edges
7. `run_loop()` - 24 edges
8. `sync()` - 23 edges
9. `dayKey()` - 23 edges
10. `Row()` - 20 edges

## Surprising Connections (you probably didn't know these)
- `dock_items()` --references--> `NotchState`  [INFERRED]
  src-tauri/src/dock.rs → src-tauri/src/notch.rs
- `dock_activate()` --references--> `NotchState`  [INFERRED]
  src-tauri/src/dock.rs → src-tauri/src/notch.rs
- `dock_close_windows()` --references--> `NotchState`  [INFERRED]
  src-tauri/src/dock.rs → src-tauri/src/notch.rs
- `command_user_avatar()` --references--> `NotchState`  [INFERRED]
  src-tauri/src/topbar.rs → src-tauri/src/notch.rs
- `set_avatar_image()` --references--> `NotchState`  [INFERRED]
  src-tauri/src/topbar.rs → src-tauri/src/notch.rs

## Import Cycles
- None detected.

## Communities (109 total, 39 thin omitted)

### Community 0 - "SettingsWindow.jsx"
Cohesion: 0.09
Nodes (60): NokiLogo(), About(), Ai(), Alerts(), Appearance(), Avatar(), BLUR_PX, CalendarSettings() (+52 more)

### Community 1 - "translate.rs"
Cohesion: 0.14
Nodes (9): client(), google(), joins_google_parts_and_detects_language(), ocr_file(), ocr_image(), parse_google(), recognize(), translate_text() (+1 more)

### Community 2 - "dropzone.rs"
Cohesion: 0.07
Nodes (7): DragEvent, ensure(), NotchDropTarget, NotchDropTarget_Impl, paths_of(), registered_ptr(), TARGETS

### Community 3 - "quick.rs"
Cohesion: 0.08
Nodes (8): dark_enabled(), KEEP_AWAKE, LAST_RADIO, quick_state(), QuickState, radio_state(), read_dword(), read_radio()

### Community 4 - "package.json"
Cohesion: 0.06
Nodes (37): author, dependencies, @crabnebula/tauri-plugin-drag@^2.1.0, @huggingface/transformers@^4.3.2, react@^18.3.1, react-dom@^18.3.1, @tauri-apps/api@^2.1.1, @tauri-apps/plugin-dialog@^2.8.3 (+29 more)

### Community 5 - "ai.rs"
Cohesion: 0.12
Nodes (22): ai_cancel(), ai_chat(), AiJobs, base(), build(), ChatMsg, chunk_text(), client() (+14 more)

### Community 6 - "Recorder.jsx"
Cohesion: 0.08
Nodes (48): Detail(), fmtDur(), Live(), Options(), Recorder(), shape(), toBase64(), EN (+40 more)

### Community 7 - "calendar.rs"
Cohesion: 0.11
Nodes (28): Acc, calendar_fetch(), CalResult, DAYS_AHEAD, days_in_month(), download(), Dt, Event (+20 more)

### Community 8 - "bundle"
Cohesion: 0.05
Nodes (38): app, security, windows, withGlobalTauri, build, beforeBuildCommand, beforeDevCommand, devUrl (+30 more)

### Community 9 - "notch.rs"
Cohesion: 0.06
Nodes (43): remove(), sync(), ABE_TOP, ABM_NEW, ABM_QUERYPOS, ABM_REMOVE, ABM_SETPOS, appbar_remove() (+35 more)

### Community 11 - "App.jsx"
Cohesion: 0.08
Nodes (42): @tauri-apps/api, isTextField(), Notch(), TABS, useBattery(), useNativeDrop(), isNight(), STROKE (+34 more)

### Community 12 - "clipboard.rs"
Cohesion: 0.10
Nodes (12): CF_UNICODETEXT, clip_clear(), clip_copy(), clip_list(), clip_remove(), Entry, History, MAX_CHARS (+4 more)

### Community 13 - "lib.rs"
Cohesion: 0.06
Nodes (12): ALLOWED_HOSTS, detect_city(), DetectedCity, file_size(), FileInfo, list_monitors(), MonitorDto, set_brightness() (+4 more)

### Community 14 - "viz.rs"
Cohesion: 0.09
Nodes (13): a_tone_lights_the_matching_band(), bands(), BANDS, Capture, default_device_id(), drain(), EMIT_EVERY, fft() (+5 more)

### Community 15 - "system.rs"
Cohesion: 0.08
Nodes (12): audio_output_name(), AudioDevice, DisplayBrightness, Endpoint, endpoint(), friendly_name(), Hud, IOCTL_VIDEO_QUERY_DISPLAY_BRIGHTNESS (+4 more)

### Community 16 - "screentime.rs"
Cohesion: 0.12
Nodes (14): add(), CLEARED, enabled(), flush(), FLUSH_EVERY, foreground_app(), IDLE_LIMIT, idle_secs() (+6 more)

### Community 17 - "Settings"
Cohesion: 0.08
Nodes (23): backup_import_blocking(), Calendar, CalSource, Clipboard, DEFAULT_UPDATE_URL, Dock, DockFolder, General (+15 more)

### Community 19 - "backdrop.rs"
Cohesion: 0.08
Nodes (5): ENTRIES, Entry, pieces_for(), Shape, wnd_proc()

### Community 20 - "FileShelf.jsx"
Cohesion: 0.17
Nodes (21): CalendarView(), dayLabel(), monthTitle(), addPaths(), baseName(), dragIcon(), FileShelf(), fmtSize() (+13 more)

### Community 21 - "ExpandedView.jsx"
Cohesion: 0.15
Nodes (22): Battery(), Clock(), Costume(), Eyes(), Mascot(), Mouth(), LyricsPanel(), MediaPlayer() (+14 more)

### Community 22 - "topbar.rs"
Cohesion: 0.06
Nodes (16): ABE_TOP, ABM_NEW, ABM_QUERYPOS, ABM_REMOVE, ABM_SETPOS, battery_info(), BatteryInfo, command_battery_info() (+8 more)

### Community 23 - "useStore"
Cohesion: 0.14
Nodes (17): AppIcon(), cache, colorOf(), COLORS, failedAt, fetchIcon(), wait(), ACTIONS (+9 more)

### Community 24 - "sysmon.rs"
Cohesion: 0.18
Nodes (15): DiskInfo, end_task(), end_task_blocking(), is_protected(), ProcInfo, PROTECTED, protects_system_and_self(), Stats (+7 more)

### Community 25 - "create"
Cohesion: 0.20
Nodes (9): close_launchpad(), ensure(), focus_window(), open(), open_launchpad(), position(), remove_noactivate(), sync() (+1 more)

### Community 26 - "sync"
Cohesion: 0.18
Nodes (14): appbar(), AppBarState, commit_pin(), create(), dock_pin(), dock_reorder(), dock_resize(), dock_unpin() (+6 more)

### Community 27 - "TopBar.jsx"
Cohesion: 0.07
Nodes (61): App(), AppsIcon(), Dock(), apply(), measureRest(), onMouseMove(), resetMagnification(), start() (+53 more)

### Community 29 - "window_utils.rs"
Cohesion: 0.09
Nodes (15): ACCENT_DISABLED, ACCENT_ENABLE_ACRYLICBLURBEHIND, AccentPolicy, CompositionData, frontend_ready(), GetModuleHandleW(), GetProcAddress(), mark_frontend_ready() (+7 more)

### Community 30 - "popups.rs"
Cohesion: 0.16
Nodes (16): active_for_window(), BLUR_DEBOUNCE, close(), close_all(), close_all_command(), hide_other_windows(), hide_window(), open() (+8 more)

### Community 31 - "fileops.rs"
Cohesion: 0.15
Nodes (15): copy_and_move_keep_content_and_avoid_collisions(), copy_recursive(), copy_to(), move_to(), shelf_copy(), shelf_move(), shelf_trash(), shelf_unzip() (+7 more)

### Community 32 - "Productivity.jsx"
Cohesion: 0.12
Nodes (29): Ctl(), ZEN_EXTRA_H, ZenPill(), FocusView(), Productivity(), SUBTABS, TASK_CATEGORIES, TasksView() (+21 more)

### Community 33 - "blocking_with_timeout"
Cohesion: 0.14
Nodes (13): blocking_with_timeout(), bt_devices(), command_audio_output_name(), command_bt_devices(), command_lock_user(), command_logoff_user(), command_open_system_uri(), command_open_task_manager() (+5 more)

### Community 34 - "media.rs"
Cohesion: 0.14
Nodes (5): ArtCache, now_filetime(), pretty_source(), read_state(), read_thumbnail()

### Community 35 - "SystemView.jsx"
Cohesion: 0.29
Nodes (12): clean(), gb(), mb(), Performance(), Processes(), rate(), Ring(), Spark() (+4 more)

### Community 36 - "run"
Cohesion: 0.19
Nodes (18): apply_autostart(), apply_shortcut(), build_tray(), commit_settings(), data_dir(), open_settings(), open_settings_async(), reset_settings() (+10 more)

### Community 37 - "NotchState"
Cohesion: 0.19
Nodes (15): backup_export(), backup_export_blocking(), backup_import(), broadcast_notify(), get_settings(), notch_state(), request_open(), set_greeting() (+7 more)

### Community 38 - "barPaint.jsx"
Cohesion: 0.28
Nodes (12): StripDecor(), BarPaint(), useBarPaint(), cache, fillFor(), rgba(), useBgImage(), inRange() (+4 more)

### Community 39 - "audio_and_brightness"
Cohesion: 0.22
Nodes (9): audio_and_brightness(), device_kind(), keyboard(), keyboard_language(), start(), SysCmd, Brightness, Mute (+1 more)

### Community 40 - "MediaHandle"
Cohesion: 0.32
Nodes (5): media_control(), media_seek(), media_state(), MediaHandle, MediaState

### Community 41 - "useStore"
Cohesion: 0.17
Nodes (19): Notes(), title(), ago(), Clipboard(), Launcher(), SUBTABS, ToolsView(), LANGS (+11 more)

### Community 42 - "set_avatar_image"
Cohesion: 0.23
Nodes (14): avatar_cache(), command_user_avatar(), data_url(), encode_avatar_png(), extract_embedded_images(), image_data(), is_jpeg(), is_png() (+6 more)

### Community 43 - "Cmd"
Cohesion: 0.18
Nodes (10): apply(), Cmd, Next, Pause, Play, Prev, Seek, Toggle (+2 more)

### Community 44 - "launcher.rs"
Cohesion: 0.05
Nodes (35): bg_data(), bg_data_blocking(), bg_import(), bg_import_blocking(), dir(), EXTENSIONS, MAX_BYTES, mime() (+27 more)

### Community 46 - "sync"
Cohesion: 0.27
Nodes (12): apply_glass_mode(), check_z_order(), ensure_mouse_style(), foreign_window_at(), glass_changed(), glass_state(), hwnd(), raise() (+4 more)

### Community 47 - "appicon.rs"
Cohesion: 0.19
Nodes (10): apply(), ATARDECER, AURORA, DEFAULT, every_icon_decodes(), image(), OCEANO, PASTEL_ATARDECER (+2 more)

### Community 48 - "items"
Cohesion: 0.24
Nodes (13): dock_activate(), dock_close_windows(), dock_items(), DockItem, file_name(), identity(), items(), pin_key() (+5 more)

### Community 53 - "contact_links"
Cohesion: 0.33
Nodes (5): blank_issues_enabled, contact_links, about, name, url

### Community 54 - "spotlight_search"
Cohesion: 0.19
Nodes (13): app_result(), ensure_index(), FileEntry, FileIndex, fold(), INDEX, index(), roots() (+5 more)

### Community 55 - "updates.rs"
Cohesion: 0.37
Nodes (7): check(), endpoint(), Pending, start_auto(), update_check(), update_install(), UpdateInfo

### Community 56 - "default.json"
Cohesion: 0.33
Nodes (5): description, identifier, permissions, $schema, windows

### Community 57 - "AppHandle"
Cohesion: 0.26
Nodes (12): close_bar_panel_impl(), command_bar_panel_state(), command_close_bar_panel(), command_open_bar_panel(), command_resize_bar_panel(), open_bar_panel_impl(), panel_state(), PanelState (+4 more)

### Community 59 - "AppHandle"
Cohesion: 0.38
Nodes (5): close_spotlight(), ensure(), focus(), position(), toggle_spotlight()

### Community 60 - "create_panel"
Cohesion: 0.33
Nodes (11): create(), create(), create_panel(), create_topbar(), app_url(), corrected_app_url(), mark_rounded(), repair_page_load() (+3 more)

### Community 61 - "render"
Cohesion: 0.21
Nodes (10): bitmap_png(), fallback_png(), glyph(), hicon_png(), png_data(), render(), resolve_program(), shell_extension_png() (+2 more)

### Community 63 - "Mutex"
Cohesion: 0.27
Nodes (8): quit_app(), APPBAR_HWND, appbar_hwnd(), AVATAR_CACHE, GLASS, PANEL, remove(), remove_appbar()

### Community 66 - "dock.rs"
Cohesion: 0.06
Nodes (14): ABE_BOTTOM, ABM_GETSTATE, ABM_NEW, ABM_QUERYPOS, ABM_REMOVE, ABM_SETPOS, ABM_SETSTATE, ABS_AUTOHIDE (+6 more)

### Community 67 - "quick_set"
Cohesion: 0.24
Nodes (8): endpoint_volume(), muted(), quick_set(), set_dark(), set_muted(), set_radio(), command_toggle_airplane(), command_toggle_dark()

### Community 69 - ".clamp"
Cohesion: 0.19
Nodes (6): Appearance, Glass, GlassSurface, Shortcuts, ThemeEvent, valid_md()

### Community 70 - "dayKey"
Cohesion: 0.32
Nodes (11): blank(), Editor(), HabitsView(), ICONS, PRESETS, countOn(), EMPTY_HABITS, slotsToday() (+3 more)

### Community 78 - "LPARAM"
Cohesion: 0.27
Nodes (10): child_callback(), enum_callback(), hwnd(), physical(), restore_taskbar(), start(), sync_taskbar(), taskbar_state() (+2 more)

### Community 79 - "Mutex"
Cohesion: 0.33
Nodes (7): APPBAR, dock_set_hit(), Hit, HIT, hit(), SIZE, TASKBAR_STATE

### Community 83 - "window_app_user_model_id"
Cohesion: 0.33
Nodes (6): app_user_model_id(), link_targets(), process_app_user_model_id(), process_image_path(), utf16_string(), window_app_user_model_id()

### Community 85 - "apply"
Cohesion: 0.38
Nodes (7): apply(), entries(), glass_shape(), piece_tint(), schedule(), set_enabled(), start()

### Community 89 - "RwLock"
Cohesion: 0.33
Nodes (4): app_icon(), cache(), CACHE, warm_paths()

### Community 90 - "ScreenTime.jsx"
Cohesion: 0.38
Nodes (9): AppList(), Bars(), colorOf(), COLORS, fmt(), lastDays(), mergeApps(), ScreenTime() (+1 more)

### Community 91 - "mic_loop"
Cohesion: 0.36
Nodes (9): alert(), bluetooth_loop(), connected_bluetooth(), mic_loop(), pretty_app(), probe(), start(), wifi_loop() (+1 more)

### Community 92 - "react"
Cohesion: 0.39
Nodes (6): react, AIChat(), PROVIDER_NAMES, useAi(), useAiChat(), aiGreeting()

### Community 95 - "PCWSTR"
Cohesion: 0.43
Nodes (5): PERSONALIZE, mic_users(), open_key(), qword(), scan()

### Community 96 - "collect_networks"
Cohesion: 0.40
Nodes (4): collect_networks(), command_wifi_networks(), wifi_networks(), WifiNetwork

### Community 97 - "ProviderCfg"
Cohesion: 0.47
Nodes (4): Ai, LegacyModels, provider(), ProviderCfg

### Community 98 - "create_piece"
Cohesion: 1.00
Nodes (3): create_piece(), GetModuleHandleW(), register_class()

## Knowledge Gaps
- **12 isolated node(s):** `Volume`, `Mute`, `Brightness`, `Next`, `Pause` (+7 more)
  These have ≤1 connection - possible missing edges. (Counts symbols only; 515 node(s) total have ≤1 connection when file, concept and rationale nodes are included.)
- **39 thin communities (<3 nodes) omitted from report** — run `graphify query` to explore isolated nodes.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `Settings` connect `Settings` to `ProviderCfg`, `dock.rs`, `run`, `NotchState`, `.clamp`, `notch.rs`, `set_avatar_image`, `lib.rs`, `LPARAM`, `sync`, `items`, `topbar.rs`, `create`, `sync`, `spotlight.rs`?**
  _High betweenness centrality (0.034) - this node is a cross-community bridge._
- **What connects `Volume`, `Mute`, `Brightness` to the rest of the system?**
  _12 weakly-connected nodes found - possible missing edges._
- **Should `SettingsWindow.jsx` be split into smaller, more focused modules?**
  _Cohesion score 0.08960573476702509 - nodes in this community are weakly interconnected._
- **Why does `react` connect `react` to `SettingsWindow.jsx`, `Productivity.jsx`, `SystemView.jsx`, `package.json`, `dayKey`, `barPaint.jsx`, `Recorder.jsx`, `useStore`, `App.jsx`, `FileShelf.jsx`, `ExpandedView.jsx`, `useStore`, `ScreenTime.jsx`, `TopBar.jsx`?**
  _High betweenness centrality (0.018) - this node is a cross-community bridge._
- **Should `translate.rs` be split into smaller, more focused modules?**
  _Cohesion score 0.1380952380952381 - nodes in this community are weakly interconnected._
- **Why does `NotchState` connect `NotchState` to `ai.rs`, `calendar.rs`, `notch.rs`, `set_avatar_image`, `clipboard.rs`, `lib.rs`, `items`, `Settings`, `screentime.rs`, `updates.rs`?**
  _High betweenness centrality (0.017) - this node is a cross-community bridge._
- **Should `dropzone.rs` be split into smaller, more focused modules?**
  _Cohesion score 0.07084785133565621 - nodes in this community are weakly interconnected._