# Graph Report - noki-dock  (2026-11-12)

## Corpus Check
- 109 files · ~102,938 words
- Verdict: corpus is large enough that graph structure adds value.
- Unclassified: 6 file(s) not represented in the graph (top: (none) 3, .icns 1, .ico 1)

## Summary
- 1715 nodes · 3911 edges · 94 communities (62 shown, 32 thin omitted)
- Extraction: 99% EXTRACTED · 1% INFERRED · 0% AMBIGUOUS · INFERRED: 36 edges (avg confidence: 0.72)
- Token cost: 0 input · 0 output

## Graph Freshness
- Built from commit: `0c752101`
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
- LPARAM
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
- ensure_layout
- run_loop
- Cmd
- AppHandle
- SystemView.jsx
- run
- State
- sysmon.rs
- audio_and_brightness
- PanelView.jsx
- ambient.js
- set_avatar_image
- FileShelf.jsx
- blocking_with_timeout
- CACHE
- sync
- MediaPlayer.jsx
- Lcd
- contact_links
- AppHandle
- MediaHandle
- default.json
- dayKey
- launcher.rs
- W
- HANDLE
- render
- Win
- Mutex
- bitmap_png
- ScreenTime.jsx
- noki
- backup_export_blocking
- Mutex
- window_app_user_model_id
- SysHandle

## God Nodes (most connected - your core abstractions)
1. `react` - 55 edges
2. `Settings` - 51 edges
3. `useStore()` - 46 edges
4. `Notch()` - 40 edges
5. `@tauri-apps/api` - 40 edges
6. `NotchState` - 31 edges
7. `dayKey()` - 23 edges
8. `run_loop()` - 23 edges
9. `SettingsWindow()` - 20 edges
10. `Row()` - 19 edges

## Surprising Connections (you probably didn't know these)
- `shortcut_list()` --references--> `Shortcuts`  [INFERRED]
  src-tauri/src/lib.rs → src-tauri/src/settings.rs
- `backup_export()` --references--> `Store`  [INFERRED]
  src-tauri/src/lib.rs → src-tauri/src/store.rs
- `store_all()` --references--> `Store`  [INFERRED]
  src-tauri/src/lib.rs → src-tauri/src/store.rs
- `store_set()` --references--> `Store`  [INFERRED]
  src-tauri/src/lib.rs → src-tauri/src/store.rs
- `check_z_order()` --references--> `Rect`  [INFERRED]
  src-tauri/src/topbar.rs → src-tauri/src/notch.rs

## Import Cycles
- None detected.

## Communities (94 total, 32 thin omitted)

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
Cohesion: 0.11
Nodes (22): ai_cancel(), ai_chat(), AiJobs, base(), build(), ChatMsg, chunk_text(), client() (+14 more)

### Community 6 - "Recorder.jsx"
Cohesion: 0.09
Nodes (41): Detail(), fmtDur(), Live(), Options(), Recorder(), shape(), toBase64(), aiGreeting() (+33 more)

### Community 7 - "calendar.rs"
Cohesion: 0.11
Nodes (28): Acc, calendar_fetch(), CalResult, DAYS_AHEAD, days_in_month(), download(), Dt, Event (+20 more)

### Community 8 - "bundle"
Cohesion: 0.05
Nodes (38): app, security, windows, withGlobalTauri, build, beforeBuildCommand, beforeDevCommand, devUrl (+30 more)

### Community 9 - "notch.rs"
Cohesion: 0.10
Nodes (16): ABE_TOP, ABM_NEW, ABM_QUERYPOS, ABM_REMOVE, ABM_SETPOS, appbar_remove(), appbar_remove_win(), appbar_set() (+8 more)

### Community 10 - "statuswatch.rs"
Cohesion: 0.09
Nodes (15): PERSONALIZE, alert(), bluetooth_loop(), connected_bluetooth(), MIC_KEY, mic_loop(), mic_users(), open_key() (+7 more)

### Community 11 - "App.jsx"
Cohesion: 0.09
Nodes (40): react, isTextField(), Notch(), TABS, useBattery(), isNight(), STROKE, useLook() (+32 more)

### Community 12 - "clipboard.rs"
Cohesion: 0.10
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
Cohesion: 0.12
Nodes (14): add(), CLEARED, enabled(), flush(), FLUSH_EVERY, foreground_app(), IDLE_LIMIT, idle_secs() (+6 more)

### Community 17 - "Settings"
Cohesion: 0.06
Nodes (32): Ai, Appearance, Calendar, CalSource, Clipboard, config_dir(), DEFAULT_UPDATE_URL, Dock (+24 more)

### Community 19 - "fileops.rs"
Cohesion: 0.05
Nodes (36): apply(), ATARDECER, AURORA, DEFAULT, every_icon_decodes(), image(), OCEANO, PASTEL_ATARDECER (+28 more)

### Community 20 - "LPARAM"
Cohesion: 0.15
Nodes (16): child_callback(), dock_activate(), dock_close_windows(), dock_items(), dock_launch(), dock_launch_blocking(), DockItem, enum_callback() (+8 more)

### Community 21 - "ExpandedView.jsx"
Cohesion: 0.16
Nodes (19): AIChat(), Battery(), Clock(), Costume(), Eyes(), Mascot(), Mouth(), CompactBar() (+11 more)

### Community 22 - "topbar.rs"
Cohesion: 0.05
Nodes (13): ABE_TOP, ABM_NEW, ABM_QUERYPOS, ABM_REMOVE, ABM_SETPOS, command_send_keys(), command_user_info(), PANEL_MARGIN (+5 more)

### Community 23 - "PanelView.jsx"
Cohesion: 0.13
Nodes (24): @tauri-apps/api, App(), ago(), Clipboard(), Launcher(), SUBTABS, ToolsView(), LANGS (+16 more)

### Community 24 - "media.rs"
Cohesion: 0.14
Nodes (5): ArtCache, now_filetime(), pretty_source(), read_state(), read_thumbnail()

### Community 25 - "window_utils.rs"
Cohesion: 0.04
Nodes (23): ABE_BOTTOM, ABM_GETSTATE, ABM_NEW, ABM_QUERYPOS, ABM_REMOVE, ABM_SETPOS, ABM_SETSTATE, ABS_AUTOHIDE (+15 more)

### Community 26 - "sync"
Cohesion: 0.15
Nodes (20): appbar(), AppBarState, commit_pin(), create(), dock_pin(), dock_reorder(), dock_resize(), dock_unpin() (+12 more)

### Community 27 - "TopBar.jsx"
Cohesion: 0.06
Nodes (64): AppsIcon(), Dock(), onMouseMove(), resetMagnification(), updateMagnification(), fallbackColor(), FallbackIcon(), ItemIcon() (+56 more)

### Community 28 - "spotlight.rs"
Cohesion: 0.07
Nodes (22): app_result(), close_spotlight(), create(), ensure(), ensure_index(), FileEntry, FileIndex, focus() (+14 more)

### Community 29 - "Store"
Cohesion: 0.23
Nodes (4): backup_import(), backup_import_blocking(), Store, StoreInner

### Community 30 - "popups.rs"
Cohesion: 0.21
Nodes (16): active_for_window(), BLUR_DEBOUNCE, close(), close_all(), close_all_command(), hide_other_windows(), hide_window(), open() (+8 more)

### Community 31 - "ensure_layout"
Cohesion: 0.21
Nodes (13): assignments(), compute(), ensure_layout(), Key, Layout, list_monitors(), Mon, MonitorInfo (+5 more)

### Community 32 - "run_loop"
Cohesion: 0.26
Nodes (11): button_down(), color_distance(), create_window(), cursor(), fullscreen_app_active(), open_here(), run_loop(), set_open() (+3 more)

### Community 33 - "Cmd"
Cohesion: 0.18
Nodes (10): apply(), Cmd, Next, Pause, Play, Prev, Seek, Toggle (+2 more)

### Community 34 - "AppHandle"
Cohesion: 0.26
Nodes (10): close_launchpad(), create(), ensure(), focus_window(), open(), open_launchpad(), position(), remove_noactivate() (+2 more)

### Community 35 - "SystemView.jsx"
Cohesion: 0.29
Nodes (12): clean(), gb(), mb(), Performance(), Processes(), rate(), Ring(), Spark() (+4 more)

### Community 36 - "run"
Cohesion: 0.24
Nodes (15): apply_autostart(), apply_shortcut(), build_tray(), commit_settings(), data_dir(), open_settings(), open_settings_async(), request_open() (+7 more)

### Community 37 - "State"
Cohesion: 0.29
Nodes (10): backup_export(), get_settings(), notch_state(), set_greeting(), set_media_pill(), set_peek(), set_pinned(), set_zen_pill() (+2 more)

### Community 38 - "sysmon.rs"
Cohesion: 0.19
Nodes (15): DiskInfo, end_task(), end_task_blocking(), is_protected(), ProcInfo, PROTECTED, protects_system_and_self(), Stats (+7 more)

### Community 39 - "audio_and_brightness"
Cohesion: 0.22
Nodes (9): audio_and_brightness(), device_kind(), keyboard(), keyboard_language(), start(), SysCmd, Brightness, Mute (+1 more)

### Community 40 - "PanelView.jsx"
Cohesion: 0.14
Nodes (16): AppIcon(), cache, colorOf(), COLORS, failedAt, fetchIcon(), wait(), ACTIONS (+8 more)

### Community 41 - "ambient.js"
Cohesion: 0.11
Nodes (31): Ctl(), ZEN_EXTRA_H, ZenPill(), Notes(), title(), FocusView(), Productivity(), SUBTABS (+23 more)

### Community 42 - "set_avatar_image"
Cohesion: 0.23
Nodes (14): avatar_cache(), command_user_avatar(), data_url(), encode_avatar_png(), extract_embedded_images(), image_data(), is_jpeg(), is_png() (+6 more)

### Community 43 - "FileShelf.jsx"
Cohesion: 0.39
Nodes (8): addPaths(), baseName(), dragIcon(), FileShelf(), fmtSize(), iconFor(), ICONS, useNativeDrop()

### Community 44 - "blocking_with_timeout"
Cohesion: 0.12
Nodes (15): battery_info(), BatteryInfo, blocking_with_timeout(), bt_devices(), command_audio_output_name(), command_battery_info(), command_bt_devices(), command_lock_user() (+7 more)

### Community 46 - "sync"
Cohesion: 0.24
Nodes (15): apply_glass(), apply_glass_mode(), check_z_order(), create_topbar(), ensure_mouse_style(), foreign_window_at(), glass_changed(), glass_state() (+7 more)

### Community 48 - "MediaPlayer.jsx"
Cohesion: 0.29
Nodes (11): LyricsPanel(), MediaPlayer(), useArtColor(), activeLine(), cache, clean(), find(), lrclib() (+3 more)

### Community 53 - "contact_links"
Cohesion: 0.33
Nodes (5): blank_issues_enabled, contact_links, about, name, url

### Community 54 - "AppHandle"
Cohesion: 0.33
Nodes (11): close_bar_panel_impl(), command_bar_panel_state(), command_close_bar_panel(), command_open_bar_panel(), command_resize_bar_panel(), create_panel(), ensure_panel(), open_bar_panel_impl() (+3 more)

### Community 55 - "MediaHandle"
Cohesion: 0.32
Nodes (5): media_control(), media_seek(), media_state(), MediaHandle, MediaState

### Community 56 - "default.json"
Cohesion: 0.33
Nodes (5): description, identifier, permissions, $schema, windows

### Community 57 - "dayKey"
Cohesion: 0.15
Nodes (25): CalendarView(), dayLabel(), monthTitle(), DAY, fmtMin(), FocusStats(), hourLabel(), lastDays() (+17 more)

### Community 58 - "launcher.rs"
Cohesion: 0.21
Nodes (11): AppCache, AppEntry, EXTENSIONS, launch_app(), launch_app_blocking(), launcher_apps(), launcher_list(), SHELL_PREFIX (+3 more)

### Community 59 - "W"
Cohesion: 0.33
Nodes (3): add_to_zip(), zip_files(), zip_files_blocking()

### Community 60 - "HANDLE"
Cohesion: 0.33
Nodes (5): collect_networks(), command_wifi_networks(), ssid_text(), wifi_networks(), WifiNetwork

### Community 61 - "render"
Cohesion: 0.32
Nodes (7): app_icon(), cache(), hicon_png(), render(), shell_extension_png(), shgetfileinfo_png(), warm_paths()

### Community 66 - "Mutex"
Cohesion: 0.31
Nodes (8): APPBAR, dock_set_hit(), Hit, HIT, hit(), SIZE, TASKBAR_STATE, WINDOWS

### Community 69 - "bitmap_png"
Cohesion: 0.33
Nodes (5): bitmap_png(), fallback_png(), glyph(), png_data(), shell_item_png()

### Community 70 - "ScreenTime.jsx"
Cohesion: 0.38
Nodes (9): AppList(), Bars(), colorOf(), COLORS, fmt(), lastDays(), mergeApps(), ScreenTime() (+1 more)

### Community 76 - "backup_export_blocking"
Cohesion: 0.50
Nodes (3): backup_export_blocking(), broadcast_notify(), store_all()

### Community 79 - "Mutex"
Cohesion: 0.24
Nodes (9): quit_app(), APPBAR_HWND, appbar_hwnd(), AVATAR_CACHE, GLASS, PANEL, PanelState, remove() (+1 more)

### Community 83 - "window_app_user_model_id"
Cohesion: 0.33
Nodes (6): app_user_model_id(), link_targets(), process_app_user_model_id(), process_image_path(), utf16_string(), window_app_user_model_id()

### Community 87 - "SysHandle"
Cohesion: 0.29
Nodes (4): set_brightness(), set_mute(), set_volume(), SysHandle

## Knowledge Gaps
- **12 isolated node(s):** `Next`, `Pause`, `Play`, `Prev`, `Seek` (+7 more)
  These have ≤1 connection - possible missing edges. (Counts symbols only; 489 node(s) total have ≤1 connection when file, concept and rationale nodes are included.)
- **32 thin communities (<3 nodes) omitted from report** — run `graphify query` to explore isolated nodes.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `Settings` connect `Settings` to `AppHandle`, `run`, `State`, `notch.rs`, `set_avatar_image`, `backup_export_blocking`, `lib.rs`, `sync`, `LPARAM`, `topbar.rs`, `window_utils.rs`, `sync`, `spotlight.rs`, `Store`, `ensure_layout`?**
  _High betweenness centrality (0.031) - this node is a cross-community bridge._
- **What connects `Next`, `Pause`, `Play` to the rest of the system?**
  _12 weakly-connected nodes found - possible missing edges._
- **Should `SettingsWindow.jsx` be split into smaller, more focused modules?**
  _Cohesion score 0.09643483343074226 - nodes in this community are weakly interconnected._
- **Why does `react` connect `App.jsx` to `SettingsWindow.jsx`, `SystemView.jsx`, `package.json`, `Recorder.jsx`, `ScreenTime.jsx`, `PanelView.jsx`, `ambient.js`, `FileShelf.jsx`, `MediaPlayer.jsx`, `ExpandedView.jsx`, `PanelView.jsx`, `dayKey`, `TopBar.jsx`?**
  _High betweenness centrality (0.028) - this node is a cross-community bridge._
- **Should `translate.rs` be split into smaller, more focused modules?**
  _Cohesion score 0.07535460992907801 - nodes in this community are weakly interconnected._
- **Why does `@tauri-apps/api` connect `PanelView.jsx` to `SettingsWindow.jsx`, `SystemView.jsx`, `package.json`, `Recorder.jsx`, `ScreenTime.jsx`, `PanelView.jsx`, `ambient.js`, `App.jsx`, `FileShelf.jsx`, `ExpandedView.jsx`, `dayKey`, `TopBar.jsx`?**
  _High betweenness centrality (0.017) - this node is a cross-community bridge._
- **Should `dropzone.rs` be split into smaller, more focused modules?**
  _Cohesion score 0.07084785133565621 - nodes in this community are weakly interconnected._