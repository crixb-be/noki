# Graph Report - noki-dock  (2026-11-13)

## Corpus Check
- 111 files · ~106,407 words
- Verdict: corpus is large enough that graph structure adds value.
- Unclassified: 6 file(s) not represented in the graph (top: (none) 3, .icns 1, .ico 1)

## Summary
- 1785 nodes · 4109 edges · 98 communities (61 shown, 37 thin omitted)
- Extraction: 99% EXTRACTED · 1% INFERRED · 0% AMBIGUOUS · INFERRED: 37 edges (avg confidence: 0.71)
- Token cost: 0 input · 0 output

## Graph Freshness
- Built from commit: `2a171eda`
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
- backdrop.rs
- dayKey
- ExpandedView.jsx
- topbar.rs
- useStore
- sysmon.rs
- create_panel
- sync
- TopBar.jsx
- spotlight.rs
- window_utils.rs
- popups.rs
- fileops.rs
- useGreeting.js
- blocking_with_timeout
- media.rs
- SystemView.jsx
- run
- State
- audio_and_brightness
- MediaHandle
- ambient.js
- set_avatar_image
- Cmd
- duration
- sync
- spotlight_search
- MediaPlayer.jsx
- Lcd
- contact_links
- AppHandle
- Launchpad.jsx
- default.json
- AppHandle
- backgrounds.rs
- AppHandle
- Store
- render
- Mutex
- dock.rs
- SysHandle
- noki
- W
- Mutex
- window_app_user_model_id
- backup_export_blocking

## God Nodes (most connected - your core abstractions)
1. `react` - 56 edges
2. `Settings` - 52 edges
3. `useStore()` - 46 edges
4. `@tauri-apps/api` - 41 edges
5. `Notch()` - 40 edges
6. `NotchState` - 32 edges
7. `dayKey()` - 23 edges
8. `run_loop()` - 23 edges
9. `SettingsWindow()` - 20 edges
10. `Row()` - 19 edges

## Surprising Connections (you probably didn't know these)
- `shortcut_list()` --references--> `Shortcuts`  [INFERRED]
  src-tauri/src/lib.rs → src-tauri/src/settings.rs
- `dock_activate()` --references--> `NotchState`  [INFERRED]
  src-tauri/src/dock.rs → src-tauri/src/notch.rs
- `dock_close_windows()` --references--> `NotchState`  [INFERRED]
  src-tauri/src/dock.rs → src-tauri/src/notch.rs
- `dock_items()` --references--> `NotchState`  [INFERRED]
  src-tauri/src/dock.rs → src-tauri/src/notch.rs
- `command_user_avatar()` --references--> `NotchState`  [INFERRED]
  src-tauri/src/topbar.rs → src-tauri/src/notch.rs

## Import Cycles
- None detected.

## Communities (98 total, 37 thin omitted)

### Community 0 - "SettingsWindow.jsx"
Cohesion: 0.09
Nodes (61): NokiLogo(), About(), Ai(), Alerts(), Appearance(), Avatar(), CalendarSettings(), ColorPicker() (+53 more)

### Community 1 - "translate.rs"
Cohesion: 0.15
Nodes (9): client(), google(), joins_google_parts_and_detects_language(), ocr_file(), ocr_image(), parse_google(), recognize(), translate_text() (+1 more)

### Community 2 - "dropzone.rs"
Cohesion: 0.07
Nodes (7): DragEvent, ensure(), NotchDropTarget, NotchDropTarget_Impl, paths_of(), registered_ptr(), TARGETS

### Community 3 - "quick.rs"
Cohesion: 0.08
Nodes (14): dark_enabled(), endpoint_volume(), KEEP_AWAKE, LAST_RADIO, muted(), quick_set(), quick_state(), QuickState (+6 more)

### Community 4 - "package.json"
Cohesion: 0.06
Nodes (37): author, dependencies, @crabnebula/tauri-plugin-drag@^2.1.0, @huggingface/transformers@^4.3.2, react@^18.3.1, react-dom@^18.3.1, @tauri-apps/api@^2.1.1, @tauri-apps/plugin-dialog@^2.8.3 (+29 more)

### Community 5 - "ai.rs"
Cohesion: 0.12
Nodes (22): ai_cancel(), ai_chat(), AiJobs, base(), build(), ChatMsg, chunk_text(), client() (+14 more)

### Community 6 - "Recorder.jsx"
Cohesion: 0.08
Nodes (46): Detail(), fmtDur(), Live(), Options(), Recorder(), shape(), toBase64(), EN (+38 more)

### Community 7 - "calendar.rs"
Cohesion: 0.11
Nodes (28): Acc, calendar_fetch(), CalResult, DAYS_AHEAD, days_in_month(), download(), Dt, Event (+20 more)

### Community 8 - "bundle"
Cohesion: 0.05
Nodes (38): app, security, windows, withGlobalTauri, build, beforeBuildCommand, beforeDevCommand, devUrl (+30 more)

### Community 9 - "notch.rs"
Cohesion: 0.05
Nodes (44): remove(), app_user_model_id(), window_app_user_model_id(), ABE_TOP, ABM_NEW, ABM_QUERYPOS, ABM_REMOVE, ABM_SETPOS (+36 more)

### Community 10 - "statuswatch.rs"
Cohesion: 0.09
Nodes (15): PERSONALIZE, alert(), bluetooth_loop(), connected_bluetooth(), MIC_KEY, mic_loop(), mic_users(), open_key() (+7 more)

### Community 11 - "App.jsx"
Cohesion: 0.09
Nodes (35): react, App(), isTextField(), Notch(), TABS, EAR, Ear(), QUICK_H (+27 more)

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
Cohesion: 0.08
Nodes (11): audio_output_name(), AudioDevice, DisplayBrightness, Endpoint, endpoint(), friendly_name(), Hud, IOCTL_VIDEO_QUERY_DISPLAY_BRIGHTNESS (+3 more)

### Community 16 - "screentime.rs"
Cohesion: 0.12
Nodes (14): add(), CLEARED, enabled(), flush(), FLUSH_EVERY, foreground_app(), IDLE_LIMIT, idle_secs() (+6 more)

### Community 17 - "settings.rs"
Cohesion: 0.06
Nodes (31): Ai, Appearance, Calendar, CalSource, Clipboard, config_dir(), DEFAULT_UPDATE_URL, Dock (+23 more)

### Community 19 - "backdrop.rs"
Cohesion: 0.07
Nodes (15): apply(), create_piece(), ENTRIES, entries(), Entry, GetModuleHandleW(), glass_shape(), PIECE_TINT (+7 more)

### Community 20 - "dayKey"
Cohesion: 0.14
Nodes (28): CalendarView(), dayLabel(), monthTitle(), addPaths(), baseName(), dragIcon(), FileShelf(), fmtSize() (+20 more)

### Community 21 - "ExpandedView.jsx"
Cohesion: 0.15
Nodes (21): AIChat(), Battery(), useBattery(), Clock(), CompactBar(), Indicator(), ExpandedView(), Welcome() (+13 more)

### Community 22 - "topbar.rs"
Cohesion: 0.06
Nodes (16): ABE_TOP, ABM_NEW, ABM_QUERYPOS, ABM_REMOVE, ABM_SETPOS, battery_info(), BatteryInfo, command_battery_info() (+8 more)

### Community 23 - "useStore"
Cohesion: 0.09
Nodes (31): AppIcon(), cache, colorOf(), COLORS, failedAt, fetchIcon(), wait(), ACTIONS (+23 more)

### Community 24 - "sysmon.rs"
Cohesion: 0.09
Nodes (26): AppCache, AppEntry, EXTENSIONS, launch_app(), launch_app_blocking(), launcher_apps(), launcher_list(), SHELL_PREFIX (+18 more)

### Community 25 - "create_panel"
Cohesion: 0.22
Nodes (13): create(), sync(), create(), create_panel(), create_topbar(), app_url(), corrected_app_url(), frontend_ready() (+5 more)

### Community 26 - "sync"
Cohesion: 0.13
Nodes (25): appbar(), AppBarState, child_callback(), commit_pin(), create(), dock_pin(), dock_reorder(), dock_resize() (+17 more)

### Community 27 - "TopBar.jsx"
Cohesion: 0.11
Nodes (43): @tauri-apps/api, AppsIcon(), Dock(), apply(), measureRest(), onMouseMove(), resetMagnification(), start() (+35 more)

### Community 29 - "window_utils.rs"
Cohesion: 0.08
Nodes (17): ACCENT_DISABLED, ACCENT_ENABLE_ACRYLICBLURBEHIND, AccentPolicy, CompositionData, enforce_chrome_free_hwnd(), GetModuleHandleW(), GetProcAddress(), is_popup() (+9 more)

### Community 30 - "popups.rs"
Cohesion: 0.16
Nodes (16): active_for_window(), BLUR_DEBOUNCE, close(), close_all(), close_all_command(), hide_other_windows(), hide_window(), open() (+8 more)

### Community 31 - "fileops.rs"
Cohesion: 0.05
Nodes (36): apply(), ATARDECER, AURORA, DEFAULT, every_icon_decodes(), image(), OCEANO, PASTEL_ATARDECER (+28 more)

### Community 32 - "useGreeting.js"
Cohesion: 0.17
Nodes (18): Costume(), Eyes(), isNight(), Mascot(), Mouth(), STROKE, useLook(), useMascotMood() (+10 more)

### Community 33 - "blocking_with_timeout"
Cohesion: 0.12
Nodes (17): blocking_with_timeout(), bt_devices(), collect_networks(), command_audio_output_name(), command_bt_devices(), command_lock_user(), command_logoff_user(), command_open_system_uri() (+9 more)

### Community 34 - "media.rs"
Cohesion: 0.14
Nodes (8): apply(), ArtCache, now_filetime(), pretty_source(), read_state(), read_thumbnail(), run(), start()

### Community 35 - "SystemView.jsx"
Cohesion: 0.29
Nodes (12): clean(), gb(), mb(), Performance(), Processes(), rate(), Ring(), Spark() (+4 more)

### Community 36 - "run"
Cohesion: 0.21
Nodes (17): apply_autostart(), apply_shortcut(), build_tray(), commit_settings(), data_dir(), open_settings(), open_settings_async(), quit_app() (+9 more)

### Community 37 - "State"
Cohesion: 0.29
Nodes (10): backup_export(), get_settings(), notch_state(), set_greeting(), set_media_pill(), set_peek(), set_pinned(), set_zen_pill() (+2 more)

### Community 39 - "audio_and_brightness"
Cohesion: 0.22
Nodes (9): audio_and_brightness(), device_kind(), keyboard(), keyboard_language(), start(), SysCmd, Brightness, Mute (+1 more)

### Community 40 - "MediaHandle"
Cohesion: 0.32
Nodes (5): media_control(), media_seek(), media_state(), MediaHandle, MediaState

### Community 41 - "ambient.js"
Cohesion: 0.10
Nodes (37): DAY, fmtMin(), FocusStats(), hourLabel(), lastDays(), Ctl(), ZEN_EXTRA_H, ZenPill() (+29 more)

### Community 42 - "set_avatar_image"
Cohesion: 0.23
Nodes (14): avatar_cache(), command_user_avatar(), data_url(), encode_avatar_png(), extract_embedded_images(), image_data(), is_jpeg(), is_png() (+6 more)

### Community 43 - "Cmd"
Cohesion: 0.22
Nodes (7): Cmd, Next, Pause, Play, Prev, Seek, Toggle

### Community 44 - "duration"
Cohesion: 0.37
Nodes (7): check(), endpoint(), Pending, start_auto(), update_check(), update_install(), UpdateInfo

### Community 46 - "sync"
Cohesion: 0.26
Nodes (14): appbar_hwnd(), apply_glass_mode(), check_z_order(), ensure_mouse_style(), foreign_window_at(), glass_changed(), glass_state(), hwnd() (+6 more)

### Community 47 - "spotlight_search"
Cohesion: 0.18
Nodes (13): app_result(), ensure_index(), FileEntry, FileIndex, fold(), INDEX, index(), roots() (+5 more)

### Community 48 - "MediaPlayer.jsx"
Cohesion: 0.18
Nodes (17): LyricsPanel(), MediaPlayer(), useArtColor(), Ctl(), Eq(), Icon(), MediaPill(), PILL_EXTRA_H (+9 more)

### Community 53 - "contact_links"
Cohesion: 0.33
Nodes (5): blank_issues_enabled, contact_links, about, name, url

### Community 54 - "AppHandle"
Cohesion: 0.26
Nodes (12): close_bar_panel_impl(), command_bar_panel_state(), command_close_bar_panel(), command_open_bar_panel(), command_resize_bar_panel(), ensure_panel(), open_bar_panel_impl(), panel_state() (+4 more)

### Community 55 - "Launchpad.jsx"
Cohesion: 0.23
Nodes (12): CATEGORIES, fold(), iconCache, iconQueue, iconWaiters, launch(), LaunchItem, Launchpad() (+4 more)

### Community 56 - "default.json"
Cohesion: 0.33
Nodes (5): description, identifier, permissions, $schema, windows

### Community 57 - "AppHandle"
Cohesion: 0.22
Nodes (8): close_launchpad(), ensure(), focus_window(), open(), open_launchpad(), position(), remove_noactivate(), toggle_launchpad()

### Community 58 - "backgrounds.rs"
Cohesion: 0.24
Nodes (11): bg_data(), bg_data_blocking(), bg_import(), bg_import_blocking(), dir(), EXTENSIONS, MAX_BYTES, mime() (+3 more)

### Community 59 - "AppHandle"
Cohesion: 0.32
Nodes (6): close_spotlight(), ensure(), focus(), position(), spotlight_resize(), toggle_spotlight()

### Community 60 - "Store"
Cohesion: 0.31
Nodes (4): backup_import(), backup_import_blocking(), Store, StoreInner

### Community 61 - "render"
Cohesion: 0.17
Nodes (13): app_icon(), bitmap_png(), cache(), fallback_png(), glyph(), hicon_png(), png_data(), render() (+5 more)

### Community 63 - "Mutex"
Cohesion: 0.53
Nodes (4): APPBAR_HWND, AVATAR_CACHE, GLASS, PANEL

### Community 66 - "dock.rs"
Cohesion: 0.06
Nodes (27): ABE_BOTTOM, ABM_GETSTATE, ABM_NEW, ABM_QUERYPOS, ABM_REMOVE, ABM_SETPOS, ABM_SETSTATE, ABS_AUTOHIDE (+19 more)

### Community 70 - "SysHandle"
Cohesion: 0.29
Nodes (4): set_brightness(), set_mute(), set_volume(), SysHandle

### Community 76 - "W"
Cohesion: 0.33
Nodes (3): add_to_zip(), zip_files(), zip_files_blocking()

### Community 79 - "Mutex"
Cohesion: 0.33
Nodes (7): APPBAR, dock_set_hit(), Hit, HIT, hit(), SIZE, TASKBAR_STATE

### Community 83 - "window_app_user_model_id"
Cohesion: 0.50
Nodes (4): link_targets(), process_app_user_model_id(), process_image_path(), utf16_string()

### Community 85 - "backup_export_blocking"
Cohesion: 0.50
Nodes (3): backup_export_blocking(), broadcast_notify(), store_all()

## Knowledge Gaps
- **12 isolated node(s):** `Brightness`, `Mute`, `Volume`, `Next`, `Pause` (+7 more)
  These have ≤1 connection - possible missing edges. (Counts symbols only; 511 node(s) total have ≤1 connection when file, concept and rationale nodes are included.)
- **37 thin communities (<3 nodes) omitted from report** — run `graphify query` to explore isolated nodes.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `Settings` connect `settings.rs` to `dock.rs`, `run`, `State`, `notch.rs`, `set_avatar_image`, `lib.rs`, `Store`, `sync`, `backup_export_blocking`, `topbar.rs`, `AppHandle`, `sync`, `spotlight.rs`, `create_panel`?**
  _High betweenness centrality (0.031) - this node is a cross-community bridge._
- **What connects `Brightness`, `Mute`, `Volume` to the rest of the system?**
  _12 weakly-connected nodes found - possible missing edges._
- **Should `SettingsWindow.jsx` be split into smaller, more focused modules?**
  _Cohesion score 0.08509615384615385 - nodes in this community are weakly interconnected._
- **Why does `react` connect `App.jsx` to `useGreeting.js`, `SettingsWindow.jsx`, `SystemView.jsx`, `package.json`, `Recorder.jsx`, `ambient.js`, `MediaPlayer.jsx`, `dayKey`, `ExpandedView.jsx`, `Launchpad.jsx`, `useStore`, `TopBar.jsx`?**
  _High betweenness centrality (0.022) - this node is a cross-community bridge._
- **Should `translate.rs` be split into smaller, more focused modules?**
  _Cohesion score 0.14736842105263157 - nodes in this community are weakly interconnected._
- **Why does `NotchState` connect `State` to `dock.rs`, `ai.rs`, `calendar.rs`, `notch.rs`, `set_avatar_image`, `clipboard.rs`, `lib.rs`, `duration`, `screentime.rs`, `settings.rs`, `Store`?**
  _High betweenness centrality (0.013) - this node is a cross-community bridge._
- **Should `dropzone.rs` be split into smaller, more focused modules?**
  _Cohesion score 0.07084785133565621 - nodes in this community are weakly interconnected._