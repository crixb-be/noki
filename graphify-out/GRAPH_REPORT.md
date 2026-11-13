# Graph Report - noki-dock  (2026-11-12)

## Corpus Check
- 109 files · ~102,415 words
- Verdict: corpus is large enough that graph structure adds value.
- Unclassified: 6 file(s) not represented in the graph (top: (none) 3, .icns 1, .ico 1)

## Summary
- 1707 nodes · 3928 edges · 88 communities (60 shown, 28 thin omitted)
- Extraction: 99% EXTRACTED · 1% INFERRED · 0% AMBIGUOUS · INFERRED: 36 edges (avg confidence: 0.69)
- Token cost: 0 input · 0 output

## Graph Freshness
- Built from commit: `d05eb62f`
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
- fileops.rs
- dock.rs
- ExpandedView.jsx
- topbar.rs
- PanelView.jsx
- media.rs
- window_utils.rs
- sync
- TopBar.jsx
- spotlight.rs
- Store
- backgrounds.rs
- useStore
- super
- ScreenTime.jsx
- updates.rs
- SystemView.jsx
- run
- State
- sysmon.rs
- audio_and_brightness
- ProviderCfg
- ambient.js
- set_avatar_image
- react
- blocking_with_timeout
- Cmd
- sync
- CACHE
- MediaPlayer.jsx
- Lcd
- contact_links
- AppHandle
- default.json
- dayKey
- W
- appicon.rs
- render
- secrets.rs
- Mutex
- items
- LPARAM
- noki
- FileShelf.jsx
- Mutex
- window_app_user_model_id

## God Nodes (most connected - your core abstractions)
1. `react` - 55 edges
2. `Settings` - 51 edges
3. `useStore()` - 46 edges
4. `Notch()` - 41 edges
5. `@tauri-apps/api` - 40 edges
6. `NotchState` - 31 edges
7. `dayKey()` - 23 edges
8. `run_loop()` - 23 edges
9. `SettingsWindow()` - 20 edges
10. `sync()` - 19 edges

## Surprising Connections (you probably didn't know these)
- `dock_items()` --references--> `NotchState`  [INFERRED]
  src-tauri/src/dock.rs → src-tauri/src/notch.rs
- `dock_activate()` --references--> `NotchState`  [INFERRED]
  src-tauri/src/dock.rs → src-tauri/src/notch.rs
- `shortcut_list()` --references--> `Shortcuts`  [INFERRED]
  src-tauri/src/lib.rs → src-tauri/src/settings.rs
- `store_all()` --references--> `Store`  [INFERRED]
  src-tauri/src/lib.rs → src-tauri/src/store.rs
- `store_set()` --references--> `Store`  [INFERRED]
  src-tauri/src/lib.rs → src-tauri/src/store.rs

## Import Cycles
- None detected.

## Communities (88 total, 28 thin omitted)

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
Cohesion: 0.07
Nodes (30): author, @huggingface/transformers@^4.3.2, react-dom@^18.3.1, description, devDependencies, autoprefixer@^10.4.20, postcss@^8.4.49, tailwindcss@^3.4.17 (+22 more)

### Community 5 - "ai.rs"
Cohesion: 0.12
Nodes (21): ai_cancel(), ai_chat(), AiJobs, build(), ChatMsg, chunk_text(), client(), drain_lines() (+13 more)

### Community 6 - "Recorder.jsx"
Cohesion: 0.11
Nodes (34): Detail(), fmtDur(), Live(), Options(), Recorder(), shape(), toBase64(), isEn() (+26 more)

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
Cohesion: 0.11
Nodes (31): isTextField(), Notch(), TABS, useBattery(), useNativeDrop(), EAR, Ear(), QUICK_H (+23 more)

### Community 12 - "clipboard.rs"
Cohesion: 0.10
Nodes (12): CF_UNICODETEXT, clip_clear(), clip_copy(), clip_list(), clip_remove(), Entry, History, MAX_CHARS (+4 more)

### Community 13 - "lib.rs"
Cohesion: 0.05
Nodes (12): ALLOWED_HOSTS, detect_city(), DetectedCity, file_size(), FileInfo, list_monitors(), MonitorDto, set_brightness() (+4 more)

### Community 14 - "viz.rs"
Cohesion: 0.09
Nodes (13): a_tone_lights_the_matching_band(), bands(), BANDS, Capture, default_device_id(), drain(), EMIT_EVERY, fft() (+5 more)

### Community 15 - "system.rs"
Cohesion: 0.07
Nodes (11): audio_output_name(), AudioDevice, DisplayBrightness, Endpoint, endpoint(), friendly_name(), Hud, IOCTL_VIDEO_QUERY_DISPLAY_BRIGHTNESS (+3 more)

### Community 16 - "screentime.rs"
Cohesion: 0.12
Nodes (14): add(), CLEARED, enabled(), flush(), FLUSH_EVERY, foreground_app(), IDLE_LIMIT, idle_secs() (+6 more)

### Community 17 - "Settings"
Cohesion: 0.06
Nodes (29): backup_import_blocking(), Appearance, Calendar, CalSource, Clipboard, config_dir(), DEFAULT_UPDATE_URL, Dock (+21 more)

### Community 19 - "fileops.rs"
Cohesion: 0.15
Nodes (15): copy_and_move_keep_content_and_avoid_collisions(), copy_recursive(), copy_to(), move_to(), shelf_copy(), shelf_move(), shelf_trash(), shelf_unzip() (+7 more)

### Community 20 - "dock.rs"
Cohesion: 0.07
Nodes (12): ABE_BOTTOM, ABM_GETSTATE, ABM_NEW, ABM_QUERYPOS, ABM_REMOVE, ABM_SETPOS, ABM_SETSTATE, ABS_AUTOHIDE (+4 more)

### Community 21 - "ExpandedView.jsx"
Cohesion: 0.15
Nodes (22): Battery(), CalendarView(), dayLabel(), monthTitle(), Clock(), Costume(), Eyes(), isNight() (+14 more)

### Community 22 - "topbar.rs"
Cohesion: 0.05
Nodes (16): ABE_TOP, ABM_NEW, ABM_QUERYPOS, ABM_REMOVE, ABM_SETPOS, battery_info(), BatteryInfo, command_battery_info() (+8 more)

### Community 23 - "PanelView.jsx"
Cohesion: 0.14
Nodes (17): AppIcon(), cache, colorOf(), COLORS, failedAt, fetchIcon(), wait(), ACTIONS (+9 more)

### Community 24 - "media.rs"
Cohesion: 0.14
Nodes (8): apply(), ArtCache, now_filetime(), pretty_source(), read_state(), read_thumbnail(), run(), start()

### Community 25 - "window_utils.rs"
Cohesion: 0.07
Nodes (6): corrected_app_url(), mark_frontend_ready(), READY_VIEWS, ready_views(), repair_page_load(), route_is_valid()

### Community 26 - "sync"
Cohesion: 0.15
Nodes (20): appbar(), AppBarState, commit_pin(), create(), dock_pin(), dock_reorder(), dock_resize(), dock_unpin() (+12 more)

### Community 27 - "TopBar.jsx"
Cohesion: 0.08
Nodes (55): App(), AppsIcon(), Dock(), onMouseMove(), resetMagnification(), updateMagnification(), fallbackColor(), FallbackIcon() (+47 more)

### Community 28 - "spotlight.rs"
Cohesion: 0.05
Nodes (47): close_launchpad(), create(), ensure(), focus_window(), open(), open_launchpad(), position(), remove_noactivate() (+39 more)

### Community 30 - "backgrounds.rs"
Cohesion: 0.22
Nodes (11): bg_data(), bg_data_blocking(), bg_import(), bg_import_blocking(), dir(), EXTENSIONS, MAX_BYTES, mime() (+3 more)

### Community 31 - "useStore"
Cohesion: 0.14
Nodes (21): Notes(), title(), ago(), Clipboard(), Launcher(), SUBTABS, ToolsView(), LANGS (+13 more)

### Community 32 - "super"
Cohesion: 0.31
Nodes (4): copies_nested_folders(), copy_dir(), migrate_data(), OLD_DIR

### Community 33 - "ScreenTime.jsx"
Cohesion: 0.38
Nodes (9): AppList(), Bars(), colorOf(), COLORS, fmt(), lastDays(), mergeApps(), ScreenTime() (+1 more)

### Community 34 - "updates.rs"
Cohesion: 0.30
Nodes (7): check(), endpoint(), Pending, start_auto(), update_check(), update_install(), UpdateInfo

### Community 35 - "SystemView.jsx"
Cohesion: 0.29
Nodes (12): clean(), gb(), mb(), Performance(), Processes(), rate(), Ring(), Spark() (+4 more)

### Community 36 - "run"
Cohesion: 0.23
Nodes (16): apply_autostart(), apply_shortcut(), build_tray(), commit_settings(), data_dir(), open_settings(), open_settings_async(), quit_app() (+8 more)

### Community 37 - "State"
Cohesion: 0.20
Nodes (15): backup_export(), backup_export_blocking(), backup_import(), broadcast_notify(), get_settings(), notch_state(), request_open(), set_greeting() (+7 more)

### Community 38 - "sysmon.rs"
Cohesion: 0.09
Nodes (26): AppCache, AppEntry, EXTENSIONS, launch_app(), launch_app_blocking(), launcher_apps(), launcher_list(), SHELL_PREFIX (+18 more)

### Community 39 - "audio_and_brightness"
Cohesion: 0.22
Nodes (9): audio_and_brightness(), device_kind(), keyboard(), keyboard_language(), start(), SysCmd, Brightness, Mute (+1 more)

### Community 40 - "ProviderCfg"
Cohesion: 0.38
Nodes (5): base(), Ai, LegacyModels, provider(), ProviderCfg

### Community 41 - "ambient.js"
Cohesion: 0.12
Nodes (29): Ctl(), ZEN_EXTRA_H, ZenPill(), FocusView(), SUBTABS, TASK_CATEGORIES, TasksView(), addFocusTime() (+21 more)

### Community 42 - "set_avatar_image"
Cohesion: 0.23
Nodes (14): avatar_cache(), command_user_avatar(), data_url(), encode_avatar_png(), extract_embedded_images(), image_data(), is_jpeg(), is_png() (+6 more)

### Community 43 - "react"
Cohesion: 0.15
Nodes (21): react, @tauri-apps/api, AIChat(), fallbackColor(), Result(), ResultGroup(), Spotlight(), PROVIDER_NAMES (+13 more)

### Community 44 - "blocking_with_timeout"
Cohesion: 0.12
Nodes (16): blocking_with_timeout(), bt_devices(), command_audio_output_name(), command_bt_devices(), command_lock_user(), command_logoff_user(), command_open_system_uri(), command_open_task_manager() (+8 more)

### Community 45 - "Cmd"
Cohesion: 0.15
Nodes (12): media_control(), media_seek(), media_state(), Cmd, Next, Pause, Play, Prev (+4 more)

### Community 46 - "sync"
Cohesion: 0.23
Nodes (16): appbar_hwnd(), apply_glass(), apply_glass_mode(), check_z_order(), create_topbar(), ensure_mouse_style(), foreign_window_at(), glass_changed() (+8 more)

### Community 48 - "MediaPlayer.jsx"
Cohesion: 0.18
Nodes (17): LyricsPanel(), MediaPlayer(), useArtColor(), Ctl(), Eq(), Icon(), MediaPill(), PILL_EXTRA_H (+9 more)

### Community 53 - "contact_links"
Cohesion: 0.33
Nodes (5): blank_issues_enabled, contact_links, about, name, url

### Community 54 - "AppHandle"
Cohesion: 0.16
Nodes (18): close_bar_panel_impl(), command_bar_panel_state(), command_close_bar_panel(), command_open_bar_panel(), command_resize_bar_panel(), create_panel(), ensure_panel(), open_bar_panel_impl() (+10 more)

### Community 56 - "default.json"
Cohesion: 0.33
Nodes (5): description, identifier, permissions, $schema, windows

### Community 57 - "dayKey"
Cohesion: 0.22
Nodes (17): DAY, fmtMin(), FocusStats(), hourLabel(), lastDays(), blank(), Editor(), HabitsView() (+9 more)

### Community 59 - "W"
Cohesion: 0.33
Nodes (3): add_to_zip(), zip_files(), zip_files_blocking()

### Community 60 - "appicon.rs"
Cohesion: 0.19
Nodes (10): apply(), ATARDECER, AURORA, DEFAULT, every_icon_decodes(), image(), OCEANO, PASTEL_ATARDECER (+2 more)

### Community 61 - "render"
Cohesion: 0.19
Nodes (12): app_icon(), bitmap_png(), cache(), fallback_png(), glyph(), hicon_png(), png_data(), render() (+4 more)

### Community 63 - "secrets.rs"
Cohesion: 0.24
Nodes (7): entry(), get(), LEGACY_SERVICE, MASK, PROVIDERS, SERVICE, set()

### Community 66 - "Mutex"
Cohesion: 0.31
Nodes (8): APPBAR, dock_set_hit(), Hit, HIT, hit(), SIZE, TASKBAR_STATE, WINDOWS

### Community 69 - "items"
Cohesion: 0.24
Nodes (9): dock_activate(), dock_close_windows(), dock_items(), dock_launch(), dock_launch_blocking(), DockItem, items(), running_windows() (+1 more)

### Community 70 - "LPARAM"
Cohesion: 0.33
Nodes (7): child_callback(), enum_callback(), hwnd(), restore_taskbar(), sync_taskbar(), taskbar_state(), title()

### Community 76 - "FileShelf.jsx"
Cohesion: 0.19
Nodes (15): dependencies, @crabnebula/tauri-plugin-drag@^2.1.0, react@^18.3.1, @tauri-apps/api@^2.1.1, @tauri-apps/plugin-dialog@^2.8.3, @crabnebula/tauri-plugin-drag, @tauri-apps/plugin-dialog, addPaths() (+7 more)

### Community 79 - "Mutex"
Cohesion: 0.53
Nodes (4): APPBAR_HWND, AVATAR_CACHE, GLASS, PANEL

### Community 83 - "window_app_user_model_id"
Cohesion: 0.33
Nodes (6): app_user_model_id(), link_targets(), process_app_user_model_id(), process_image_path(), utf16_string(), window_app_user_model_id()

## Knowledge Gaps
- **12 isolated node(s):** `Brightness`, `Mute`, `Volume`, `Next`, `Pause` (+7 more)
  These have ≤1 connection - possible missing edges. (Counts symbols only; 483 node(s) total have ≤1 connection when file, concept and rationale nodes are included.)
- **28 thin communities (<3 nodes) omitted from report** — run `graphify query` to explore isolated nodes.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `react` connect `react` to `SettingsWindow.jsx`, `ScreenTime.jsx`, `SystemView.jsx`, `package.json`, `Recorder.jsx`, `ambient.js`, `App.jsx`, `FileShelf.jsx`, `MediaPlayer.jsx`, `ExpandedView.jsx`, `PanelView.jsx`, `dayKey`, `TopBar.jsx`, `useStore`?**
  _High betweenness centrality (0.026) - this node is a cross-community bridge._
- **What connects `Brightness`, `Mute`, `Volume` to the rest of the system?**
  _12 weakly-connected nodes found - possible missing edges._
- **Should `SettingsWindow.jsx` be split into smaller, more focused modules?**
  _Cohesion score 0.09378531073446328 - nodes in this community are weakly interconnected._
- **Why does `Settings` connect `Settings` to `run`, `items`, `LPARAM`, `State`, `ProviderCfg`, `notch.rs`, `set_avatar_image`, `lib.rs`, `sync`, `dock.rs`, `topbar.rs`, `sync`, `spotlight.rs`?**
  _High betweenness centrality (0.023) - this node is a cross-community bridge._
- **Should `translate.rs` be split into smaller, more focused modules?**
  _Cohesion score 0.14736842105263157 - nodes in this community are weakly interconnected._
- **Why does `NotchState` connect `State` to `updates.rs`, `items`, `ai.rs`, `calendar.rs`, `notch.rs`, `set_avatar_image`, `clipboard.rs`, `lib.rs`, `screentime.rs`, `Settings`?**
  _High betweenness centrality (0.014) - this node is a cross-community bridge._
- **Should `dropzone.rs` be split into smaller, more focused modules?**
  _Cohesion score 0.07084785133565621 - nodes in this community are weakly interconnected._