# Graph Report - noki-dock  (2026-11-13)

## Corpus Check
- 111 files · ~112,809 words
- Verdict: corpus is large enough that graph structure adds value.
- Unclassified: 6 file(s) not represented in the graph (top: (none) 3, .icns 1, .ico 1)

## Summary
- 1852 nodes · 4251 edges · 98 communities (66 shown, 32 thin omitted)
- Extraction: 99% EXTRACTED · 1% INFERRED · 0% AMBIGUOUS · INFERRED: 40 edges (avg confidence: 0.7)
- Token cost: 0 input · 0 output

## Graph Freshness
- Built from commit: `7d5b4693`
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
- MediaPill.jsx
- topbar.rs
- fileops.rs
- sysmon.rs
- create
- sync
- TopBar.jsx
- glass.js
- window_utils.rs
- popups.rs
- FileShelf.jsx
- Productivity.jsx
- blocking_with_timeout
- media.rs
- SystemView.jsx
- items
- NotchState
- main.jsx
- Cmd
- i18n.js
- useStore
- set_avatar_image
- Store
- launcher.rs
- RgbaImage
- sync
- Launchpad.jsx
- .clamp
- Lcd
- contact_links
- spotlight_search
- BarPanel.jsx
- default.json
- AppHandle
- open_tray_overflow
- Mutex
- render
- Mutex
- dock.rs
- AIChat.jsx
- noki
- launcher.rs
- LPARAM
- window_app_user_model_id
- MediaHandle
- useSettings
- ScreenTime.jsx
- ExpandedView.jsx

## God Nodes (most connected - your core abstractions)
1. `react` - 56 edges
2. `Settings` - 52 edges
3. `useStore()` - 46 edges
4. `@tauri-apps/api` - 41 edges
5. `Notch()` - 40 edges
6. `NotchState` - 32 edges
7. `run_loop()` - 27 edges
8. `dayKey()` - 23 edges
9. `TopBar()` - 22 edges
10. `Row()` - 20 edges

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

## Communities (98 total, 32 thin omitted)

### Community 0 - "SettingsWindow.jsx"
Cohesion: 0.09
Nodes (60): NokiLogo(), About(), Ai(), Alerts(), Appearance(), Avatar(), BLUR_PX, CalendarSettings() (+52 more)

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
Cohesion: 0.09
Nodes (26): ai_cancel(), ai_chat(), AiJobs, base(), build(), ChatMsg, chunk_text(), client() (+18 more)

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
Nodes (43): remove(), ABE_TOP, ABM_NEW, ABM_QUERYPOS, ABM_REMOVE, ABM_SETPOS, appbar_remove(), appbar_remove_win() (+35 more)

### Community 10 - "statuswatch.rs"
Cohesion: 0.09
Nodes (15): PERSONALIZE, alert(), bluetooth_loop(), connected_bluetooth(), MIC_KEY, mic_loop(), mic_users(), open_key() (+7 more)

### Community 11 - "App.jsx"
Cohesion: 0.09
Nodes (46): react, @tauri-apps/api, isTextField(), Notch(), TABS, isNight(), STROKE, useLook() (+38 more)

### Community 12 - "clipboard.rs"
Cohesion: 0.10
Nodes (12): CF_UNICODETEXT, clip_clear(), clip_copy(), clip_list(), clip_remove(), Entry, History, MAX_CHARS (+4 more)

### Community 13 - "lib.rs"
Cohesion: 0.06
Nodes (27): ALLOWED_HOSTS, apply_autostart(), apply_shortcut(), backup_import(), backup_import_blocking(), build_tray(), commit_settings(), data_dir() (+19 more)

### Community 14 - "viz.rs"
Cohesion: 0.09
Nodes (13): a_tone_lights_the_matching_band(), bands(), BANDS, Capture, default_device_id(), drain(), EMIT_EVERY, fft() (+5 more)

### Community 15 - "system.rs"
Cohesion: 0.07
Nodes (20): audio_and_brightness(), audio_output_name(), AudioDevice, device_kind(), DisplayBrightness, Endpoint, endpoint(), friendly_name() (+12 more)

### Community 16 - "screentime.rs"
Cohesion: 0.11
Nodes (14): add(), CLEARED, enabled(), flush(), FLUSH_EVERY, foreground_app(), IDLE_LIMIT, idle_secs() (+6 more)

### Community 17 - "Settings"
Cohesion: 0.08
Nodes (23): Calendar, CalSource, Clipboard, config_dir(), DEFAULT_UPDATE_URL, Dock, DockFolder, General (+15 more)

### Community 18 - "icons.rs"
Cohesion: 0.04
Nodes (4): CACHE_MAX, OUT_PX, SMALL_CONTENT_PX, SOURCE_PX

### Community 19 - "backdrop.rs"
Cohesion: 0.06
Nodes (21): apply(), apply_attributes(), corner_for(), create_piece(), ENTRIES, entries(), Entry, follow_window() (+13 more)

### Community 20 - "FileShelf.jsx"
Cohesion: 0.15
Nodes (25): CalendarView(), dayLabel(), monthTitle(), DAY, fmtMin(), FocusStats(), hourLabel(), lastDays() (+17 more)

### Community 21 - "MediaPill.jsx"
Cohesion: 0.53
Nodes (5): Ctl(), Eq(), Icon(), MediaPill(), PILL_EXTRA_H

### Community 22 - "topbar.rs"
Cohesion: 0.05
Nodes (15): ABE_TOP, ABM_NEW, ABM_QUERYPOS, ABM_REMOVE, ABM_SETPOS, battery_info(), BatteryInfo, command_battery_info() (+7 more)

### Community 23 - "fileops.rs"
Cohesion: 0.15
Nodes (15): copy_and_move_keep_content_and_avoid_collisions(), copy_recursive(), copy_to(), move_to(), shelf_copy(), shelf_move(), shelf_trash(), shelf_unzip() (+7 more)

### Community 24 - "sysmon.rs"
Cohesion: 0.17
Nodes (15): DiskInfo, end_task(), end_task_blocking(), is_protected(), ProcInfo, PROTECTED, protects_system_and_self(), Stats (+7 more)

### Community 25 - "create"
Cohesion: 0.21
Nodes (8): create(), ensure(), open(), open_launchpad(), position(), sync(), toggle_launchpad(), frontend_ready()

### Community 26 - "sync"
Cohesion: 0.20
Nodes (15): appbar(), AppBarState, commit_pin(), create(), dock_pin(), dock_reorder(), dock_resize(), dock_unpin() (+7 more)

### Community 27 - "TopBar.jsx"
Cohesion: 0.30
Nodes (17): barInk(), luminance(), base, BatteryIcon(), BellIcon(), BluetoothIcon(), CameraIcon(), ChevronIcon() (+9 more)

### Community 28 - "glass.js"
Cohesion: 0.20
Nodes (15): fallbackColor(), Result(), ResultGroup(), Spotlight(), afterPaint(), GLASS_RADIUS, glassRadius(), glassVars() (+7 more)

### Community 29 - "window_utils.rs"
Cohesion: 0.07
Nodes (22): ACCENT_DISABLED, ACCENT_ENABLE_ACRYLICBLURBEHIND, AccentPolicy, app_url(), CompositionData, corrected_app_url(), enforce_chrome_free_hwnd(), GetModuleHandleW() (+14 more)

### Community 30 - "popups.rs"
Cohesion: 0.12
Nodes (28): close_launchpad(), active_for_window(), auto_close(), base_id(), BLUR_SETTLE, Callback, callbacks(), CALLBACKS (+20 more)

### Community 31 - "FileShelf.jsx"
Cohesion: 0.39
Nodes (8): addPaths(), baseName(), dragIcon(), FileShelf(), fmtSize(), iconFor(), ICONS, useNativeDrop()

### Community 32 - "Productivity.jsx"
Cohesion: 0.11
Nodes (31): Ctl(), ZEN_EXTRA_H, ZenPill(), Notes(), title(), FocusView(), Productivity(), SUBTABS (+23 more)

### Community 33 - "blocking_with_timeout"
Cohesion: 0.11
Nodes (17): blocking_with_timeout(), bt_devices(), collect_networks(), command_audio_output_name(), command_bt_devices(), command_lock_user(), command_logoff_user(), command_open_system_uri() (+9 more)

### Community 34 - "media.rs"
Cohesion: 0.16
Nodes (5): ArtCache, now_filetime(), pretty_source(), read_state(), read_thumbnail()

### Community 35 - "SystemView.jsx"
Cohesion: 0.29
Nodes (12): clean(), gb(), mb(), Performance(), Processes(), rate(), Ring(), Spark() (+4 more)

### Community 36 - "items"
Cohesion: 0.18
Nodes (16): capture_overlay_now(), dock_activate(), dock_close_windows(), dock_items(), dock_launch(), dock_launch_blocking(), DockItem, file_name() (+8 more)

### Community 37 - "NotchState"
Cohesion: 0.16
Nodes (17): backup_export(), backup_export_blocking(), broadcast_notify(), get_settings(), notch_state(), set_brightness(), set_greeting(), set_media_pill() (+9 more)

### Community 38 - "main.jsx"
Cohesion: 0.21
Nodes (11): AppsIcon(), Dock(), apply(), measureRest(), onMouseMove(), resetMagnification(), start(), tick() (+3 more)

### Community 39 - "Cmd"
Cohesion: 0.18
Nodes (10): apply(), Cmd, Next, Pause, Play, Prev, Seek, Toggle (+2 more)

### Community 40 - "i18n.js"
Cohesion: 0.25
Nodes (14): EN, PATTERNS, applyAttrs(), applyText(), ATTRS, attrState, setLanguage(), skip() (+6 more)

### Community 41 - "useStore"
Cohesion: 0.09
Nodes (34): App(), AppIcon(), cache, colorOf(), COLORS, failedAt, fetchIcon(), wait() (+26 more)

### Community 42 - "set_avatar_image"
Cohesion: 0.23
Nodes (14): avatar_cache(), command_user_avatar(), data_url(), encode_avatar_png(), extract_embedded_images(), image_data(), is_jpeg(), is_png() (+6 more)

### Community 44 - "launcher.rs"
Cohesion: 0.05
Nodes (39): apply(), ATARDECER, AURORA, DEFAULT, every_icon_decodes(), image(), OCEANO, PASTEL_ATARDECER (+31 more)

### Community 45 - "RgbaImage"
Cohesion: 0.27
Nodes (10): bitmap_rgba(), content_box(), file_icon_png(), finish(), hicon_rgba(), image_list_icon(), read_bitmap(), resize_clean() (+2 more)

### Community 46 - "sync"
Cohesion: 0.22
Nodes (15): apply_glass_mode(), check_z_order(), create_topbar(), ensure_mouse_style(), foreign_window_at(), glass_changed(), glass_state(), hwnd() (+7 more)

### Community 47 - "Launchpad.jsx"
Cohesion: 0.22
Nodes (13): CATEGORIES, fold(), iconCache, iconQueue, iconWaiters, launch(), LaunchItem, Launchpad() (+5 more)

### Community 48 - ".clamp"
Cohesion: 0.19
Nodes (6): Appearance, Glass, GlassSurface, Shortcuts, ThemeEvent, valid_md()

### Community 53 - "contact_links"
Cohesion: 0.33
Nodes (5): blank_issues_enabled, contact_links, about, name, url

### Community 54 - "spotlight_search"
Cohesion: 0.21
Nodes (18): app_result(), create(), ensure(), ensure_index(), FileEntry, FileIndex, fold(), INDEX (+10 more)

### Community 55 - "BarPanel.jsx"
Cohesion: 0.32
Nodes (10): action(), BarPanel(), CcCircle(), CcPill(), ControlCenter(), PanelLabel(), PanelLink(), Switch() (+2 more)

### Community 56 - "default.json"
Cohesion: 0.33
Nodes (5): description, identifier, permissions, $schema, windows

### Community 57 - "AppHandle"
Cohesion: 0.33
Nodes (11): close_bar_panel_impl(), command_bar_panel_state(), command_close_bar_panel(), command_open_bar_panel(), command_resize_bar_panel(), create_panel(), ensure_panel(), open_bar_panel_impl() (+3 more)

### Community 59 - "open_tray_overflow"
Cohesion: 0.22
Nodes (7): command_send_keys(), command_tray_overflow(), key_input(), open_tray_overflow(), physical(), send_keys(), tray_closed_at()

### Community 60 - "Mutex"
Cohesion: 0.33
Nodes (7): APPBAR, dock_set_hit(), Hit, HIT, hit(), SIZE, TASKBAR_STATE

### Community 61 - "render"
Cohesion: 0.19
Nodes (11): app_icon(), cache(), CACHE, fallback_png(), glyph(), png_data(), remember(), render() (+3 more)

### Community 63 - "Mutex"
Cohesion: 0.27
Nodes (8): APPBAR_HWND, appbar_hwnd(), AVATAR_CACHE, GLASS, PANEL, PanelState, remove(), remove_appbar()

### Community 66 - "dock.rs"
Cohesion: 0.06
Nodes (12): ABE_BOTTOM, ABM_GETSTATE, ABM_NEW, ABM_QUERYPOS, ABM_REMOVE, ABM_SETPOS, ABM_SETSTATE, ABS_AUTOHIDE (+4 more)

### Community 67 - "AIChat.jsx"
Cohesion: 0.39
Nodes (5): AIChat(), PROVIDER_NAMES, useAi(), useAiChat(), aiGreeting()

### Community 76 - "launcher.rs"
Cohesion: 0.14
Nodes (12): AppCache, AppEntry, EXTENSIONS, launch_app(), launch_app_blocking(), launcher_apps(), launcher_list(), SHELL_PREFIX (+4 more)

### Community 78 - "LPARAM"
Cohesion: 0.19
Nodes (15): capture_in_progress(), capture_overlay_active(), child_callback(), enum_callback(), hwnd(), last_capture(), physical(), restore_taskbar() (+7 more)

### Community 83 - "window_app_user_model_id"
Cohesion: 0.33
Nodes (6): app_user_model_id(), link_targets(), process_app_user_model_id(), process_image_path(), utf16_string(), window_app_user_model_id()

### Community 84 - "MediaHandle"
Cohesion: 0.32
Nodes (5): media_control(), media_seek(), media_state(), MediaHandle, MediaState

### Community 85 - "useSettings"
Cohesion: 0.53
Nodes (5): DEFAULTS, isObj(), merge(), useSettings(), update()

### Community 90 - "ScreenTime.jsx"
Cohesion: 0.38
Nodes (9): AppList(), Bars(), colorOf(), COLORS, fmt(), lastDays(), mergeApps(), ScreenTime() (+1 more)

### Community 92 - "ExpandedView.jsx"
Cohesion: 0.13
Nodes (26): Battery(), useBattery(), Clock(), Costume(), Eyes(), Mascot(), Mouth(), LyricsPanel() (+18 more)

## Knowledge Gaps
- **12 isolated node(s):** `Next`, `Pause`, `Play`, `Prev`, `Seek` (+7 more)
  These have ≤1 connection - possible missing edges. (Counts symbols only; 541 node(s) total have ≤1 connection when file, concept and rationale nodes are included.)
- **32 thin communities (<3 nodes) omitted from report** — run `graphify query` to explore isolated nodes.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `Settings` connect `Settings` to `dock.rs`, `items`, `NotchState`, `ai.rs`, `notch.rs`, `set_avatar_image`, `lib.rs`, `LPARAM`, `sync`, `.clamp`, `topbar.rs`, `create`, `sync`?**
  _High betweenness centrality (0.027) - this node is a cross-community bridge._
- **What connects `Next`, `Pause`, `Play` to the rest of the system?**
  _12 weakly-connected nodes found - possible missing edges._
- **Should `SettingsWindow.jsx` be split into smaller, more focused modules?**
  _Cohesion score 0.0890937019969278 - nodes in this community are weakly interconnected._
- **Why does `react` connect `App.jsx` to `SettingsWindow.jsx`, `Productivity.jsx`, `AIChat.jsx`, `package.json`, `SystemView.jsx`, `main.jsx`, `Recorder.jsx`, `i18n.js`, `useStore`, `glass.js`, `Launchpad.jsx`, `FileShelf.jsx`, `useSettings`, `BarPanel.jsx`, `ScreenTime.jsx`, `TopBar.jsx`, `ExpandedView.jsx`, `FileShelf.jsx`?**
  _High betweenness centrality (0.023) - this node is a cross-community bridge._
- **Should `translate.rs` be split into smaller, more focused modules?**
  _Cohesion score 0.14736842105263157 - nodes in this community are weakly interconnected._
- **Why does `NotchState` connect `NotchState` to `items`, `ai.rs`, `calendar.rs`, `notch.rs`, `set_avatar_image`, `clipboard.rs`, `lib.rs`, `launcher.rs`, `screentime.rs`, `Settings`?**
  _High betweenness centrality (0.012) - this node is a cross-community bridge._
- **Should `dropzone.rs` be split into smaller, more focused modules?**
  _Cohesion score 0.06866002214839424 - nodes in this community are weakly interconnected._