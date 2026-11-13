# Graph Report - noki-dock  (2026-11-13)

## Corpus Check
- 111 files · ~111,089 words
- Verdict: corpus is large enough that graph structure adds value.
- Unclassified: 6 file(s) not represented in the graph (top: (none) 3, .icns 1, .ico 1)

## Summary
- 1832 nodes · 4231 edges · 96 communities (66 shown, 30 thin omitted)
- Extraction: 99% EXTRACTED · 1% INFERRED · 0% AMBIGUOUS · INFERRED: 40 edges (avg confidence: 0.7)
- Token cost: 0 input · 0 output

## Graph Freshness
- Built from commit: `90b7a36a`
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
- run_loop
- window_utils.rs
- popups.rs
- FileShelf.jsx
- Productivity.jsx
- blocking_with_timeout
- media.rs
- SystemView.jsx
- run
- NotchState
- barPaint.jsx
- audio_and_brightness
- i18n.js
- useStore
- set_avatar_image
- Store
- launcher.rs
- RgbaImage
- sync
- manager
- SysHandle
- Lcd
- contact_links
- spotlight_search
- backup_export_blocking
- default.json
- AppHandle
- fallback_png
- create_panel
- render
- Mutex
- dock.rs
- endpoint_volume
- noki
- W
- LPARAM
- window_app_user_model_id
- CACHE
- ScreenTime.jsx
- ExpandedView.jsx
- collect_networks

## God Nodes (most connected - your core abstractions)
1. `react` - 56 edges
2. `Settings` - 52 edges
3. `useStore()` - 46 edges
4. `@tauri-apps/api` - 41 edges
5. `Notch()` - 40 edges
6. `NotchState` - 32 edges
7. `run_loop()` - 27 edges
8. `sync()` - 23 edges
9. `dayKey()` - 23 edges
10. `run()` - 21 edges

## Surprising Connections (you probably didn't know these)
- `dock_items()` --references--> `NotchState`  [INFERRED]
  src-tauri/src/dock.rs → src-tauri/src/notch.rs
- `dock_activate()` --references--> `NotchState`  [INFERRED]
  src-tauri/src/dock.rs → src-tauri/src/notch.rs
- `dock_close_windows()` --references--> `NotchState`  [INFERRED]
  src-tauri/src/dock.rs → src-tauri/src/notch.rs
- `shortcut_list()` --references--> `Shortcuts`  [INFERRED]
  src-tauri/src/lib.rs → src-tauri/src/settings.rs
- `store_all()` --references--> `Store`  [INFERRED]
  src-tauri/src/lib.rs → src-tauri/src/store.rs

## Import Cycles
- None detected.

## Communities (96 total, 30 thin omitted)

### Community 0 - "SettingsWindow.jsx"
Cohesion: 0.09
Nodes (59): NokiLogo(), About(), Ai(), Alerts(), Appearance(), Avatar(), BLUR_PX, CalendarSettings() (+51 more)

### Community 1 - "translate.rs"
Cohesion: 0.14
Nodes (9): client(), google(), joins_google_parts_and_detects_language(), ocr_file(), ocr_image(), parse_google(), recognize(), translate_text() (+1 more)

### Community 2 - "dropzone.rs"
Cohesion: 0.07
Nodes (7): DragEvent, ensure(), NotchDropTarget, NotchDropTarget_Impl, paths_of(), registered_ptr(), TARGETS

### Community 3 - "quick.rs"
Cohesion: 0.07
Nodes (12): dark_enabled(), KEEP_AWAKE, LAST_RADIO, PERSONALIZE, quick_set(), quick_state(), QuickState, radio_state() (+4 more)

### Community 4 - "package.json"
Cohesion: 0.07
Nodes (28): author, @huggingface/transformers@^4.3.2, description, devDependencies, autoprefixer@^10.4.20, postcss@^8.4.49, tailwindcss@^3.4.17, @tauri-apps/cli@^2.1.0 (+20 more)

### Community 5 - "ai.rs"
Cohesion: 0.12
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
Cohesion: 0.06
Nodes (18): remove(), ABE_TOP, ABM_NEW, ABM_QUERYPOS, ABM_REMOVE, ABM_SETPOS, appbar_remove(), appbar_remove_win() (+10 more)

### Community 10 - "statuswatch.rs"
Cohesion: 0.09
Nodes (14): alert(), bluetooth_loop(), connected_bluetooth(), MIC_KEY, mic_loop(), mic_users(), open_key(), pretty_app() (+6 more)

### Community 11 - "App.jsx"
Cohesion: 0.10
Nodes (34): isTextField(), Notch(), TABS, isNight(), STROKE, useLook(), useMascotMood(), EAR (+26 more)

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
Cohesion: 0.11
Nodes (14): add(), CLEARED, enabled(), flush(), FLUSH_EVERY, foreground_app(), IDLE_LIMIT, idle_secs() (+6 more)

### Community 17 - "Settings"
Cohesion: 0.06
Nodes (33): Ai, Appearance, Calendar, CalSource, Clipboard, config_dir(), DEFAULT_UPDATE_URL, Dock (+25 more)

### Community 18 - "icons.rs"
Cohesion: 0.04
Nodes (4): CACHE_MAX, OUT_PX, SMALL_CONTENT_PX, SOURCE_PX

### Community 19 - "backdrop.rs"
Cohesion: 0.07
Nodes (19): apply(), create_piece(), ENTRIES, entries(), Entry, follow_window(), GetModuleHandleW(), glass_ready() (+11 more)

### Community 20 - "FileShelf.jsx"
Cohesion: 0.14
Nodes (27): CalendarView(), dayLabel(), monthTitle(), DAY, fmtMin(), FocusStats(), hourLabel(), lastDays() (+19 more)

### Community 21 - "ExpandedView.jsx"
Cohesion: 0.18
Nodes (17): LyricsPanel(), MediaPlayer(), useArtColor(), Ctl(), Eq(), Icon(), MediaPill(), PILL_EXTRA_H (+9 more)

### Community 22 - "topbar.rs"
Cohesion: 0.05
Nodes (14): ABE_TOP, ABM_NEW, ABM_QUERYPOS, ABM_REMOVE, ABM_SETPOS, battery_info(), BatteryInfo, command_battery_info() (+6 more)

### Community 23 - "useStore"
Cohesion: 0.14
Nodes (17): AppIcon(), cache, colorOf(), COLORS, failedAt, fetchIcon(), wait(), ACTIONS (+9 more)

### Community 24 - "sysmon.rs"
Cohesion: 0.08
Nodes (26): AppCache, AppEntry, EXTENSIONS, launch_app(), launch_app_blocking(), launcher_apps(), launcher_list(), SHELL_PREFIX (+18 more)

### Community 25 - "create"
Cohesion: 0.22
Nodes (9): close_launchpad(), create(), ensure(), open(), open_launchpad(), position(), sync(), toggle_launchpad() (+1 more)

### Community 26 - "sync"
Cohesion: 0.13
Nodes (23): APPBAR, appbar(), AppBarState, commit_pin(), create(), dock_pin(), dock_reorder(), dock_resize() (+15 more)

### Community 27 - "TopBar.jsx"
Cohesion: 0.07
Nodes (62): AppsIcon(), Dock(), apply(), measureRest(), onMouseMove(), resetMagnification(), start(), tick() (+54 more)

### Community 28 - "run_loop"
Cohesion: 0.12
Nodes (26): request_open(), run_shortcut(), button_down(), color_distance(), compute(), create_window(), cursor(), ensure_layout() (+18 more)

### Community 29 - "window_utils.rs"
Cohesion: 0.09
Nodes (14): ACCENT_DISABLED, ACCENT_ENABLE_ACRYLICBLURBEHIND, AccentPolicy, app_url(), CompositionData, GetModuleHandleW(), GetProcAddress(), mark_frontend_ready() (+6 more)

### Community 30 - "popups.rs"
Cohesion: 0.09
Nodes (26): active_for_window(), auto_close(), base_id(), BLUR_SETTLE, Callback, callbacks(), CALLBACKS, close() (+18 more)

### Community 31 - "FileShelf.jsx"
Cohesion: 0.12
Nodes (18): dependencies, @crabnebula/tauri-plugin-drag@^2.1.0, react@^18.3.1, react-dom@^18.3.1, @tauri-apps/api@^2.1.1, @tauri-apps/plugin-dialog@^2.8.3, @crabnebula/tauri-plugin-drag, react-dom (+10 more)

### Community 32 - "Productivity.jsx"
Cohesion: 0.13
Nodes (26): Ctl(), ZEN_EXTRA_H, ZenPill(), FocusView(), SUBTABS, TASK_CATEGORIES, addFocusTime(), fmtTime() (+18 more)

### Community 33 - "blocking_with_timeout"
Cohesion: 0.12
Nodes (15): blocking_with_timeout(), bt_devices(), command_audio_output_name(), command_bt_devices(), command_lock_user(), command_logoff_user(), command_open_system_uri(), command_open_task_manager() (+7 more)

### Community 34 - "media.rs"
Cohesion: 0.09
Nodes (20): media_control(), media_seek(), media_state(), apply(), ArtCache, Cmd, Next, Pause (+12 more)

### Community 35 - "SystemView.jsx"
Cohesion: 0.29
Nodes (12): clean(), gb(), mb(), Performance(), Processes(), rate(), Ring(), Spark() (+4 more)

### Community 36 - "run"
Cohesion: 0.24
Nodes (15): apply_autostart(), apply_shortcut(), build_tray(), commit_settings(), data_dir(), open_settings(), open_settings_async(), quit_app() (+7 more)

### Community 37 - "NotchState"
Cohesion: 0.32
Nodes (10): backup_export(), get_settings(), notch_state(), set_greeting(), set_media_pill(), set_peek(), set_pinned(), set_zen_pill() (+2 more)

### Community 38 - "barPaint.jsx"
Cohesion: 0.22
Nodes (15): StripDecor(), barInk(), BarPaint(), GLASS_BASE, luminance(), useBarPaint(), cache, fillFor() (+7 more)

### Community 39 - "audio_and_brightness"
Cohesion: 0.22
Nodes (9): audio_and_brightness(), device_kind(), keyboard(), keyboard_language(), start(), SysCmd, Brightness, Mute (+1 more)

### Community 40 - "i18n.js"
Cohesion: 0.27
Nodes (13): EN, PATTERNS, applyAttrs(), applyText(), ATTRS, attrState, setLanguage(), skip() (+5 more)

### Community 41 - "useStore"
Cohesion: 0.13
Nodes (26): react, @tauri-apps/api, Notes(), title(), ago(), Clipboard(), Launcher(), SUBTABS (+18 more)

### Community 42 - "set_avatar_image"
Cohesion: 0.23
Nodes (14): avatar_cache(), command_user_avatar(), data_url(), encode_avatar_png(), extract_embedded_images(), image_data(), is_jpeg(), is_png() (+6 more)

### Community 43 - "Store"
Cohesion: 0.31
Nodes (4): backup_import(), backup_import_blocking(), Store, StoreInner

### Community 44 - "launcher.rs"
Cohesion: 0.06
Nodes (37): bg_data(), bg_data_blocking(), bg_import(), bg_import_blocking(), dir(), EXTENSIONS, MAX_BYTES, mime() (+29 more)

### Community 45 - "RgbaImage"
Cohesion: 0.30
Nodes (9): bitmap_rgba(), content_box(), file_icon_png(), finish(), hicon_rgba(), image_list_icon(), read_bitmap(), resize_clean() (+1 more)

### Community 46 - "sync"
Cohesion: 0.22
Nodes (15): appbar_hwnd(), apply_glass_mode(), check_z_order(), create_topbar(), ensure_mouse_style(), foreign_window_at(), glass_changed(), glass_state() (+7 more)

### Community 47 - "manager"
Cohesion: 0.13
Nodes (17): apply(), ATARDECER, AURORA, DEFAULT, every_icon_decodes(), image(), OCEANO, PASTEL_ATARDECER (+9 more)

### Community 48 - "SysHandle"
Cohesion: 0.33
Nodes (4): set_brightness(), set_mute(), set_volume(), SysHandle

### Community 53 - "contact_links"
Cohesion: 0.33
Nodes (5): blank_issues_enabled, contact_links, about, name, url

### Community 54 - "spotlight_search"
Cohesion: 0.19
Nodes (20): app_result(), close_spotlight(), create(), ensure(), ensure_index(), FileEntry, FileIndex, fold() (+12 more)

### Community 55 - "backup_export_blocking"
Cohesion: 0.50
Nodes (3): backup_export_blocking(), broadcast_notify(), store_all()

### Community 56 - "default.json"
Cohesion: 0.33
Nodes (5): description, identifier, permissions, $schema, windows

### Community 57 - "AppHandle"
Cohesion: 0.23
Nodes (13): assignments(), close_bar_panel_impl(), command_bar_panel_state(), command_close_bar_panel(), command_open_bar_panel(), command_resize_bar_panel(), ensure_panel(), open_bar_panel_impl() (+5 more)

### Community 59 - "fallback_png"
Cohesion: 0.67
Nodes (3): fallback_png(), glyph(), png_data()

### Community 60 - "create_panel"
Cohesion: 0.20
Nodes (9): create_panel(), corrected_app_url(), enforce_chrome_free_hwnd(), mark_rounded(), repair_page_load(), route_is_valid(), schedule_app_recovery(), set_acrylic() (+1 more)

### Community 61 - "render"
Cohesion: 0.43
Nodes (7): app_icon(), cache(), remember(), render(), shell_extension_png(), shell_item_png(), warm_paths()

### Community 63 - "Mutex"
Cohesion: 0.53
Nodes (4): APPBAR_HWND, AVATAR_CACHE, GLASS, PANEL

### Community 66 - "dock.rs"
Cohesion: 0.06
Nodes (27): ABE_BOTTOM, ABM_GETSTATE, ABM_NEW, ABM_QUERYPOS, ABM_REMOVE, ABM_SETPOS, ABM_SETSTATE, ABS_AUTOHIDE (+19 more)

### Community 67 - "endpoint_volume"
Cohesion: 0.60
Nodes (3): endpoint_volume(), muted(), set_muted()

### Community 76 - "W"
Cohesion: 0.33
Nodes (3): add_to_zip(), zip_files(), zip_files_blocking()

### Community 78 - "LPARAM"
Cohesion: 0.24
Nodes (11): capture_overlay_active(), child_callback(), enum_callback(), file_name(), hwnd(), restore_taskbar(), sync_taskbar(), taskbar_state() (+3 more)

### Community 83 - "window_app_user_model_id"
Cohesion: 0.40
Nodes (5): app_user_model_id(), link_targets(), process_app_user_model_id(), utf16_string(), window_app_user_model_id()

### Community 90 - "ScreenTime.jsx"
Cohesion: 0.38
Nodes (9): AppList(), Bars(), colorOf(), COLORS, fmt(), lastDays(), mergeApps(), ScreenTime() (+1 more)

### Community 92 - "ExpandedView.jsx"
Cohesion: 0.21
Nodes (15): AIChat(), Battery(), useBattery(), Clock(), Costume(), Eyes(), Mascot(), Mouth() (+7 more)

### Community 96 - "collect_networks"
Cohesion: 0.33
Nodes (5): collect_networks(), command_wifi_networks(), ssid_text(), wifi_networks(), WifiNetwork

## Knowledge Gaps
- **12 isolated node(s):** `noki`, `Brightness`, `Mute`, `Volume`, `Next` (+7 more)
  These have ≤1 connection - possible missing edges. (Counts symbols only; 538 node(s) total have ≤1 connection when file, concept and rationale nodes are included.)
- **30 thin communities (<3 nodes) omitted from report** — run `graphify query` to explore isolated nodes.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `Settings` connect `Settings` to `dock.rs`, `run`, `NotchState`, `notch.rs`, `set_avatar_image`, `Store`, `lib.rs`, `LPARAM`, `sync`, `topbar.rs`, `backup_export_blocking`, `create`, `sync`, `run_loop`, `AppHandle`?**
  _High betweenness centrality (0.031) - this node is a cross-community bridge._
- **What connects `noki`, `Brightness`, `Mute` to the rest of the system?**
  _12 weakly-connected nodes found - possible missing edges._
- **Should `SettingsWindow.jsx` be split into smaller, more focused modules?**
  _Cohesion score 0.09148598625066102 - nodes in this community are weakly interconnected._
- **Why does `react` connect `useStore` to `SettingsWindow.jsx`, `Productivity.jsx`, `SystemView.jsx`, `package.json`, `barPaint.jsx`, `Recorder.jsx`, `i18n.js`, `App.jsx`, `FileShelf.jsx`, `ExpandedView.jsx`, `useStore`, `ScreenTime.jsx`, `TopBar.jsx`, `ExpandedView.jsx`, `FileShelf.jsx`?**
  _High betweenness centrality (0.022) - this node is a cross-community bridge._
- **Should `translate.rs` be split into smaller, more focused modules?**
  _Cohesion score 0.1380952380952381 - nodes in this community are weakly interconnected._
- **Should `dropzone.rs` be split into smaller, more focused modules?**
  _Cohesion score 0.06866002214839424 - nodes in this community are weakly interconnected._
- **Should `quick.rs` be split into smaller, more focused modules?**
  _Cohesion score 0.07254623044096728 - nodes in this community are weakly interconnected._