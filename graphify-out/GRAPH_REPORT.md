# Graph Report - noki-dock  (2026-11-12)

## Corpus Check
- 109 files · ~103,406 words
- Verdict: corpus is large enough that graph structure adds value.
- Unclassified: 6 file(s) not represented in the graph (top: (none) 3, .icns 1, .ico 1)

## Summary
- 1723 nodes · 3937 edges · 101 communities (63 shown, 38 thin omitted)
- Extraction: 99% EXTRACTED · 1% INFERRED · 0% AMBIGUOUS · INFERRED: 37 edges (avg confidence: 0.71)
- Token cost: 0 input · 0 output

## Graph Freshness
- Built from commit: `186691f2`
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
- settings.rs
- icons.rs
- sysmon.rs
- items
- ExpandedView.jsx
- topbar.rs
- PanelView.jsx
- media.rs
- window_utils.rs
- sync
- TopBar.jsx
- spotlight.rs
- Store
- popups.rs
- fileops.rs
- Settings
- spotlight_search
- AppHandle
- SystemView.jsx
- run
- State
- barPaint.jsx
- audio_and_brightness
- PanelView.jsx
- ambient.js
- set_avatar_image
- FileShelf.jsx
- blocking_with_timeout
- sync
- MediaPlayer.jsx
- Lcd
- contact_links
- AppHandle
- updates.rs
- default.json
- dayKey
- launcher.rs
- W
- manager
- render
- react
- dock.rs
- AppHandle
- ScreenTime.jsx
- noki
- backup_export_blocking
- Mutex
- window_app_user_model_id
- SysHandle
- ProviderCfg

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
- `shortcut_list()` --references--> `Shortcuts`  [INFERRED]
  src-tauri/src/lib.rs → src-tauri/src/settings.rs
- `backup_export()` --references--> `Store`  [INFERRED]
  src-tauri/src/lib.rs → src-tauri/src/store.rs

## Import Cycles
- None detected.

## Communities (101 total, 38 thin omitted)

### Community 0 - "SettingsWindow.jsx"
Cohesion: 0.09
Nodes (57): NokiLogo(), About(), Ai(), Alerts(), Appearance(), Avatar(), CalendarSettings(), ColorPicker() (+49 more)

### Community 1 - "translate.rs"
Cohesion: 0.15
Nodes (9): client(), google(), joins_google_parts_and_detects_language(), ocr_file(), ocr_image(), parse_google(), recognize(), translate_text() (+1 more)

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
Cohesion: 0.14
Nodes (22): ai_cancel(), ai_chat(), AiJobs, base(), build(), ChatMsg, chunk_text(), client() (+14 more)

### Community 6 - "Recorder.jsx"
Cohesion: 0.11
Nodes (33): Detail(), fmtDur(), Live(), Options(), Recorder(), shape(), toBase64(), isEn() (+25 more)

### Community 7 - "calendar.rs"
Cohesion: 0.11
Nodes (28): Acc, calendar_fetch(), CalResult, DAYS_AHEAD, days_in_month(), download(), Dt, Event (+20 more)

### Community 8 - "bundle"
Cohesion: 0.05
Nodes (38): app, security, windows, withGlobalTauri, build, beforeBuildCommand, beforeDevCommand, devUrl (+30 more)

### Community 9 - "notch.rs"
Cohesion: 0.05
Nodes (46): remove(), app_user_model_id(), window_app_user_model_id(), remove_noactivate(), ABE_TOP, ABM_NEW, ABM_QUERYPOS, ABM_REMOVE (+38 more)

### Community 10 - "statuswatch.rs"
Cohesion: 0.09
Nodes (15): PERSONALIZE, alert(), bluetooth_loop(), connected_bluetooth(), MIC_KEY, mic_loop(), mic_users(), open_key() (+7 more)

### Community 11 - "App.jsx"
Cohesion: 0.08
Nodes (42): @tauri-apps/api, isTextField(), Notch(), TABS, useBattery(), Costume(), Eyes(), isNight() (+34 more)

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

### Community 17 - "settings.rs"
Cohesion: 0.07
Nodes (18): Appearance, Calendar, CalSource, Clipboard, DEFAULT_UPDATE_URL, Dock, DockFolder, General (+10 more)

### Community 19 - "sysmon.rs"
Cohesion: 0.07
Nodes (32): apply(), ATARDECER, AURORA, DEFAULT, every_icon_decodes(), image(), OCEANO, PASTEL_ATARDECER (+24 more)

### Community 20 - "items"
Cohesion: 0.19
Nodes (16): dock_activate(), dock_close_windows(), dock_items(), dock_launch(), dock_launch_blocking(), DockItem, file_name(), identity() (+8 more)

### Community 21 - "ExpandedView.jsx"
Cohesion: 0.19
Nodes (16): Battery(), Clock(), CompactBar(), Indicator(), ExpandedView(), Welcome(), Ctl(), Eq() (+8 more)

### Community 22 - "topbar.rs"
Cohesion: 0.05
Nodes (16): ABE_TOP, ABM_NEW, ABM_QUERYPOS, ABM_REMOVE, ABM_SETPOS, battery_info(), BatteryInfo, command_battery_info() (+8 more)

### Community 23 - "PanelView.jsx"
Cohesion: 0.11
Nodes (27): App(), AppIcon(), cache, colorOf(), COLORS, failedAt, fetchIcon(), wait() (+19 more)

### Community 24 - "media.rs"
Cohesion: 0.06
Nodes (31): bg_data(), bg_data_blocking(), bg_import(), bg_import_blocking(), dir(), EXTENSIONS, MAX_BYTES, mime() (+23 more)

### Community 25 - "window_utils.rs"
Cohesion: 0.07
Nodes (11): app_url(), corrected_app_url(), enforce_chrome_free(), enforce_chrome_free_hwnd(), frontend_ready(), mark_frontend_ready(), READY_VIEWS, ready_views() (+3 more)

### Community 26 - "sync"
Cohesion: 0.15
Nodes (18): APPBAR, appbar(), AppBarState, child_callback(), create(), dock_resize(), enum_callback(), hwnd() (+10 more)

### Community 27 - "TopBar.jsx"
Cohesion: 0.06
Nodes (64): AppsIcon(), Dock(), onMouseMove(), resetMagnification(), updateMagnification(), fallbackColor(), FallbackIcon(), ItemIcon() (+56 more)

### Community 29 - "Store"
Cohesion: 0.31
Nodes (4): backup_import(), backup_import_blocking(), Store, StoreInner

### Community 30 - "popups.rs"
Cohesion: 0.20
Nodes (16): active_for_window(), BLUR_DEBOUNCE, close(), close_all(), close_all_command(), hide_other_windows(), hide_window(), open() (+8 more)

### Community 31 - "fileops.rs"
Cohesion: 0.15
Nodes (15): copy_and_move_keep_content_and_avoid_collisions(), copy_recursive(), copy_to(), move_to(), shelf_copy(), shelf_move(), shelf_trash(), shelf_unzip() (+7 more)

### Community 32 - "Settings"
Cohesion: 0.16
Nodes (9): config_dir(), Hud, load(), path(), save(), Settings, Shortcuts, valid_md() (+1 more)

### Community 33 - "spotlight_search"
Cohesion: 0.18
Nodes (13): app_result(), ensure_index(), FileEntry, FileIndex, fold(), INDEX, index(), roots() (+5 more)

### Community 34 - "AppHandle"
Cohesion: 0.29
Nodes (9): close_launchpad(), create(), ensure(), focus_window(), open(), open_launchpad(), position(), sync() (+1 more)

### Community 35 - "SystemView.jsx"
Cohesion: 0.30
Nodes (11): clean(), gb(), mb(), Performance(), Processes(), rate(), Ring(), Spark() (+3 more)

### Community 36 - "run"
Cohesion: 0.21
Nodes (17): restore_taskbar(), apply_autostart(), apply_shortcut(), build_tray(), commit_settings(), data_dir(), open_settings(), open_settings_async() (+9 more)

### Community 37 - "State"
Cohesion: 0.36
Nodes (10): backup_export(), get_settings(), notch_state(), set_greeting(), set_media_pill(), set_peek(), set_pinned(), set_zen_pill() (+2 more)

### Community 38 - "barPaint.jsx"
Cohesion: 0.30
Nodes (11): StripDecor(), BarPaint(), useBarPaint(), cache, fillFor(), rgba(), useBgImage(), inRange() (+3 more)

### Community 39 - "audio_and_brightness"
Cohesion: 0.22
Nodes (9): audio_and_brightness(), device_kind(), keyboard(), keyboard_language(), start(), SysCmd, Brightness, Mute (+1 more)

### Community 40 - "PanelView.jsx"
Cohesion: 0.24
Nodes (8): ACTIONS, DEFAULT_APPS, deg(), ICONS, TOGGLES, WeatherCard(), TABLE, weatherInfo()

### Community 41 - "ambient.js"
Cohesion: 0.12
Nodes (28): Ctl(), ZEN_EXTRA_H, ZenPill(), FocusView(), SUBTABS, TASK_CATEGORIES, TasksView(), addFocusTime() (+20 more)

### Community 42 - "set_avatar_image"
Cohesion: 0.24
Nodes (14): avatar_cache(), command_user_avatar(), data_url(), encode_avatar_png(), extract_embedded_images(), image_data(), is_jpeg(), is_png() (+6 more)

### Community 43 - "FileShelf.jsx"
Cohesion: 0.39
Nodes (8): addPaths(), baseName(), dragIcon(), FileShelf(), fmtSize(), iconFor(), ICONS, useNativeDrop()

### Community 44 - "blocking_with_timeout"
Cohesion: 0.12
Nodes (17): blocking_with_timeout(), bt_devices(), collect_networks(), command_audio_output_name(), command_bt_devices(), command_lock_user(), command_logoff_user(), command_open_system_uri() (+9 more)

### Community 46 - "sync"
Cohesion: 0.29
Nodes (13): apply_glass(), apply_glass_mode(), check_z_order(), create_topbar(), ensure_mouse_style(), glass_changed(), glass_state(), hwnd() (+5 more)

### Community 48 - "MediaPlayer.jsx"
Cohesion: 0.29
Nodes (11): LyricsPanel(), MediaPlayer(), useArtColor(), activeLine(), cache, clean(), find(), lrclib() (+3 more)

### Community 53 - "contact_links"
Cohesion: 0.33
Nodes (5): blank_issues_enabled, contact_links, about, name, url

### Community 54 - "AppHandle"
Cohesion: 0.20
Nodes (17): commit_pin(), dock_pin(), dock_reorder(), dock_unpin(), close_bar_panel_impl(), command_bar_panel_state(), command_close_bar_panel(), command_open_bar_panel() (+9 more)

### Community 55 - "updates.rs"
Cohesion: 0.37
Nodes (7): check(), endpoint(), Pending, start_auto(), update_check(), update_install(), UpdateInfo

### Community 56 - "default.json"
Cohesion: 0.33
Nodes (5): description, identifier, permissions, $schema, windows

### Community 57 - "dayKey"
Cohesion: 0.15
Nodes (25): CalendarView(), dayLabel(), monthTitle(), DAY, fmtMin(), FocusStats(), hourLabel(), lastDays() (+17 more)

### Community 58 - "launcher.rs"
Cohesion: 0.17
Nodes (11): AppCache, AppEntry, EXTENSIONS, launch_app(), launch_app_blocking(), launcher_apps(), launcher_list(), SHELL_PREFIX (+3 more)

### Community 59 - "W"
Cohesion: 0.33
Nodes (3): add_to_zip(), zip_files(), zip_files_blocking()

### Community 60 - "manager"
Cohesion: 0.31
Nodes (4): copies_nested_folders(), copy_dir(), migrate_data(), OLD_DIR

### Community 61 - "render"
Cohesion: 0.19
Nodes (12): app_icon(), bitmap_png(), cache(), fallback_png(), glyph(), hicon_png(), png_data(), render() (+4 more)

### Community 63 - "react"
Cohesion: 0.39
Nodes (6): react, AIChat(), PROVIDER_NAMES, useAi(), useAiChat(), aiGreeting()

### Community 66 - "dock.rs"
Cohesion: 0.06
Nodes (22): ABE_BOTTOM, ABM_GETSTATE, ABM_NEW, ABM_QUERYPOS, ABM_REMOVE, ABM_SETPOS, ABM_SETSTATE, ABS_AUTOHIDE (+14 more)

### Community 69 - "AppHandle"
Cohesion: 0.36
Nodes (7): close_spotlight(), create(), ensure(), focus(), position(), spotlight_resize(), toggle_spotlight()

### Community 70 - "ScreenTime.jsx"
Cohesion: 0.38
Nodes (9): AppList(), Bars(), colorOf(), COLORS, fmt(), lastDays(), mergeApps(), ScreenTime() (+1 more)

### Community 76 - "backup_export_blocking"
Cohesion: 0.50
Nodes (3): backup_export_blocking(), broadcast_notify(), store_all()

### Community 79 - "Mutex"
Cohesion: 0.31
Nodes (7): APPBAR_HWND, appbar_hwnd(), AVATAR_CACHE, GLASS, PANEL, remove(), remove_appbar()

### Community 83 - "window_app_user_model_id"
Cohesion: 0.50
Nodes (4): link_targets(), process_app_user_model_id(), process_image_path(), utf16_string()

### Community 87 - "SysHandle"
Cohesion: 0.29
Nodes (4): set_brightness(), set_mute(), set_volume(), SysHandle

### Community 94 - "ProviderCfg"
Cohesion: 0.47
Nodes (4): Ai, LegacyModels, provider(), ProviderCfg

## Knowledge Gaps
- **12 isolated node(s):** `Next`, `Pause`, `Play`, `Prev`, `Seek` (+7 more)
  These have ≤1 connection - possible missing edges. (Counts symbols only; 498 node(s) total have ≤1 connection when file, concept and rationale nodes are included.)
- **38 thin communities (<3 nodes) omitted from report** — run `graphify query` to explore isolated nodes.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `react` connect `react` to `SettingsWindow.jsx`, `SystemView.jsx`, `package.json`, `barPaint.jsx`, `Recorder.jsx`, `PanelView.jsx`, `ambient.js`, `ScreenTime.jsx`, `App.jsx`, `FileShelf.jsx`, `MediaPlayer.jsx`, `ExpandedView.jsx`, `PanelView.jsx`, `dayKey`, `TopBar.jsx`?**
  _High betweenness centrality (0.029) - this node is a cross-community bridge._
- **What connects `Next`, `Pause`, `Play` to the rest of the system?**
  _12 weakly-connected nodes found - possible missing edges._
- **Should `SettingsWindow.jsx` be split into smaller, more focused modules?**
  _Cohesion score 0.09378531073446328 - nodes in this community are weakly interconnected._
- **Why does `Settings` connect `Settings` to `dock.rs`, `AppHandle`, `run`, `State`, `notch.rs`, `set_avatar_image`, `backup_export_blocking`, `lib.rs`, `sync`, `settings.rs`, `items`, `AppHandle`, `topbar.rs`, `sync`, `spotlight.rs`, `Store`, `ProviderCfg`?**
  _High betweenness centrality (0.028) - this node is a cross-community bridge._
- **Should `translate.rs` be split into smaller, more focused modules?**
  _Cohesion score 0.14736842105263157 - nodes in this community are weakly interconnected._
- **Why does `NotchState` connect `State` to `Settings`, `ai.rs`, `calendar.rs`, `notch.rs`, `set_avatar_image`, `clipboard.rs`, `lib.rs`, `screentime.rs`, `items`, `updates.rs`, `Store`?**
  _High betweenness centrality (0.021) - this node is a cross-community bridge._
- **Should `dropzone.rs` be split into smaller, more focused modules?**
  _Cohesion score 0.06866002214839424 - nodes in this community are weakly interconnected._