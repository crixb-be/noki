# Graph Report - noki-dock  (2026-11-19)

## Corpus Check
- 128 files · ~132,842 words
- Verdict: corpus is large enough that graph structure adds value.
- Unclassified: 6 file(s) not represented in the graph (top: (none) 3, .icns 1, .ico 1)

## Summary
- 2374 nodes · 5651 edges · 137 communities (82 shown, 55 thin omitted)
- Extraction: 99% EXTRACTED · 1% INFERRED · 0% AMBIGUOUS · INFERRED: 55 edges (avg confidence: 0.77)
- Token cost: 0 input · 0 output

## Graph Freshness
- Built from commit: `68110adc`
- Run `git rev-parse HEAD` and compare to check if the graph is stale.
- Run `graphify update .` after code changes (no API cost).

## Community Hubs (Navigation)
- SettingsWindow.jsx
- translate.rs
- dropzone.rs
- quick.rs
- package.json
- clipboard.rs
- Recorder.jsx
- calendar.rs
- bundle
- AppHandle
- statuswatch.rs
- App.jsx
- ai.rs
- lib.rs
- viz.rs
- system.rs
- screentime.rs
- Settings
- icons.rs
- backdrop.rs
- dayKey
- useStore
- topbar.rs
- fileops.rs
- claude_session.rs
- create
- capture_in_progress
- TopBar.jsx
- @tauri-apps/api
- window_utils.rs
- popups.rs
- ai_usage.rs
- Productivity.jsx
- sysmon.rs
- media.rs
- SystemView.jsx
- LPARAM
- NotchState
- launcher.rs
- prepare_rotation
- i18n.js
- Dock
- set_avatar_image
- dock_preview.rs
- shortcut_folder.rs
- RgbaImage
- notch.rs
- PWSTR
- Lcd
- contact_links
- spotlight_search
- jump_lists.rs
- default.json
- AppHandle
- watch_tray_overflow
- barPaint.jsx
- render
- manager
- Mutex
- dock.rs
- settings.rs
- context_menu.rs
- noki
- PanelView.jsx
- .clamp
- backgrounds.rs
- Cmd
- prepare_rotation
- appicon.rs
- Win
- super
- Mutex
- HRESULT
- ExpandedView.jsx
- Launchpad.jsx
- MediaPlayer.jsx
- W
- PCWSTR
- ScreenTime.jsx
- MediaHandle
- ProviderCfg
- path
- program_name
- mark_rounded
- battery_snapshot
- CredBuffer<T>
- dock_animation.rs
- DockGenie.jsx

## God Nodes (most connected - your core abstractions)
1. `react` - 64 edges
2. `Settings` - 60 edges
3. `@tauri-apps/api` - 48 edges
4. `useStore()` - 46 edges
5. `Notch()` - 40 edges
6. `NotchState` - 38 edges
7. `run_loop()` - 28 edges
8. `TopBar()` - 24 edges
9. `dayKey()` - 23 edges
10. `useSettings()` - 23 edges

## Surprising Connections (you probably didn't know these)
- `Control` --references--> `DockItem`  [INFERRED]
  src-tauri/src/context_menu.rs → src-tauri/src/dock.rs
- `dock_items()` --references--> `NotchState`  [INFERRED]
  src-tauri/src/dock.rs → src-tauri/src/notch.rs
- `dock_activate()` --references--> `NotchState`  [INFERRED]
  src-tauri/src/dock.rs → src-tauri/src/notch.rs
- `dock_close_windows()` --references--> `NotchState`  [INFERRED]
  src-tauri/src/dock.rs → src-tauri/src/notch.rs
- `execute()` --calls--> `trash_open()`  [EXTRACTED]
  src-tauri/src/context_menu.rs → src-tauri/src/dock.rs

## Import Cycles
- None detected.

## Communities (137 total, 55 thin omitted)

### Community 0 - "SettingsWindow.jsx"
Cohesion: 0.08
Nodes (63): NokiLogo(), About(), Ai(), Alerts(), Appearance(), Avatar(), BLUR_PX, CalendarSettings() (+55 more)

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

### Community 5 - "clipboard.rs"
Cohesion: 0.10
Nodes (12): CF_UNICODETEXT, clip_clear(), clip_copy(), clip_list(), clip_remove(), Entry, History, MAX_CHARS (+4 more)

### Community 6 - "Recorder.jsx"
Cohesion: 0.11
Nodes (35): Detail(), fmtDur(), Live(), Options(), Recorder(), shape(), toBase64(), isEn() (+27 more)

### Community 7 - "calendar.rs"
Cohesion: 0.11
Nodes (28): Acc, calendar_fetch(), CalResult, DAYS_AHEAD, days_in_month(), download(), Dt, Event (+20 more)

### Community 8 - "bundle"
Cohesion: 0.05
Nodes (38): app, security, windows, withGlobalTauri, build, beforeBuildCommand, beforeDevCommand, devUrl (+30 more)

### Community 9 - "AppHandle"
Cohesion: 0.39
Nodes (7): request_open(), run_shortcut(), create_window(), open_here(), set_open(), start(), toggle()

### Community 10 - "statuswatch.rs"
Cohesion: 0.09
Nodes (10): alert(), bluetooth_loop(), connected_bluetooth(), MIC_KEY, mic_loop(), pretty_app(), probe(), start() (+2 more)

### Community 11 - "App.jsx"
Cohesion: 0.09
Nodes (39): react, @tauri-apps/api, App(), isTextField(), Notch(), TABS, AIChat(), useBattery() (+31 more)

### Community 12 - "ai.rs"
Cohesion: 0.13
Nodes (21): ai_cancel(), ai_chat(), AiJobs, build(), ChatMsg, chunk_text(), client(), drain_lines() (+13 more)

### Community 13 - "lib.rs"
Cohesion: 0.06
Nodes (27): ALLOWED_HOSTS, apply_autostart(), apply_shortcut(), build_tray(), commit_settings(), data_dir(), detect_city(), DetectedCity (+19 more)

### Community 14 - "viz.rs"
Cohesion: 0.09
Nodes (13): a_tone_lights_the_matching_band(), bands(), BANDS, Capture, default_device_id(), drain(), EMIT_EVERY, fft() (+5 more)

### Community 15 - "system.rs"
Cohesion: 0.07
Nodes (20): audio_and_brightness(), audio_output_name(), AudioDevice, device_kind(), DisplayBrightness, Endpoint, endpoint(), friendly_name() (+12 more)

### Community 16 - "screentime.rs"
Cohesion: 0.10
Nodes (14): add(), CLEARED, enabled(), flush(), FLUSH_EVERY, foreground_app(), IDLE_LIMIT, idle_secs() (+6 more)

### Community 17 - "Settings"
Cohesion: 0.10
Nodes (10): Clipboard, General, Health, Hud, Modules, Productivity, Settings, TopBar (+2 more)

### Community 18 - "icons.rs"
Cohesion: 0.05
Nodes (11): app_user_model_id(), CACHE, CACHE_MAX, link_targets(), OUT_PX, process_app_user_model_id(), process_image_path(), SMALL_CONTENT_PX (+3 more)

### Community 19 - "backdrop.rs"
Cohesion: 0.07
Nodes (21): apply(), apply_attributes(), corner_for(), create_piece(), ENTRIES, entries(), Entry, follow_window() (+13 more)

### Community 20 - "dayKey"
Cohesion: 0.11
Nodes (33): CalendarView(), dayLabel(), monthTitle(), addPaths(), baseName(), dragIcon(), FileShelf(), fmtSize() (+25 more)

### Community 21 - "useStore"
Cohesion: 0.12
Nodes (27): AppIcon(), cache, colorOf(), COLORS, failedAt, fetchIcon(), wait(), Notes() (+19 more)

### Community 22 - "topbar.rs"
Cohesion: 0.05
Nodes (29): ABE_TOP, ABM_NEW, ABM_QUERYPOS, ABM_REMOVE, ABM_SETPOS, blocking_with_timeout(), bt_devices(), collect_networks() (+21 more)

### Community 23 - "fileops.rs"
Cohesion: 0.15
Nodes (15): copy_and_move_keep_content_and_avoid_collisions(), copy_recursive(), copy_to(), move_to(), shelf_copy(), shelf_move(), shelf_trash(), shelf_unzip() (+7 more)

### Community 24 - "claude_session.rs"
Cohesion: 0.09
Nodes (17): active_distros(), bounded_output(), BUSY, CLIENT_ID, EXPIRED, FILE_NAME, MAX_FILE_BYTES, NO_SESSION (+9 more)

### Community 25 - "create"
Cohesion: 0.22
Nodes (9): close_launchpad(), create(), ensure(), open(), open_launchpad(), position(), sync(), toggle_launchpad() (+1 more)

### Community 26 - "capture_in_progress"
Cohesion: 0.18
Nodes (17): dock_genie_finished(), EVENT_APP, overlay(), appbar(), AppBarState, commit_pin(), create(), dock_pin() (+9 more)

### Community 27 - "TopBar.jsx"
Cohesion: 0.16
Nodes (31): AiUsagePanel(), duration(), ProviderIcon(), providers, timestamp(), useAiUsage(), action(), BarPanel() (+23 more)

### Community 28 - "@tauri-apps/api"
Cohesion: 0.11
Nodes (28): clamp(), ContextMenu(), close(), keyboard(), select(), DockPreview(), FolderIcon(), ShortcutFolder() (+20 more)

### Community 29 - "window_utils.rs"
Cohesion: 0.08
Nodes (18): ACCENT_DISABLED, ACCENT_ENABLE_ACRYLICBLURBEHIND, AccentPolicy, app_url(), CompositionData, corrected_app_url(), GetModuleHandleW(), GetProcAddress() (+10 more)

### Community 30 - "popups.rs"
Cohesion: 0.06
Nodes (52): accepts(), activate_packaged(), animation_anchor(), dock_drop_targets(), Drag, DRAG, drop_files(), enter() (+44 more)

### Community 31 - "ai_usage.rs"
Cohesion: 0.11
Nodes (18): ai_usage_snapshot(), claude_limits(), codex_exe(), codex_limits(), enabled(), INTERVAL, Limit, limits_or_error() (+10 more)

### Community 32 - "Productivity.jsx"
Cohesion: 0.12
Nodes (29): Ctl(), ZEN_EXTRA_H, ZenPill(), FocusView(), Productivity(), SUBTABS, TASK_CATEGORIES, TasksView() (+21 more)

### Community 33 - "sysmon.rs"
Cohesion: 0.17
Nodes (15): DiskInfo, end_task(), end_task_blocking(), is_protected(), ProcInfo, PROTECTED, protects_system_and_self(), Stats (+7 more)

### Community 34 - "media.rs"
Cohesion: 0.13
Nodes (6): apply(), ArtCache, now_filetime(), pretty_source(), read_state(), read_thumbnail()

### Community 35 - "SystemView.jsx"
Cohesion: 0.29
Nodes (12): clean(), gb(), mb(), Performance(), Processes(), rate(), Ring(), Spark() (+4 more)

### Community 36 - "LPARAM"
Cohesion: 0.17
Nodes (23): activation_does_not_reorder_running_apps(), animation_windows(), dock_activate(), dock_close_windows(), dock_items(), DockItem, drop_app(), enumerate_windows() (+15 more)

### Community 37 - "NotchState"
Cohesion: 0.13
Nodes (18): backup_export(), backup_export_blocking(), backup_import(), backup_import_blocking(), broadcast_notify(), get_settings(), notch_state(), set_brightness() (+10 more)

### Community 38 - "launcher.rs"
Cohesion: 0.13
Nodes (18): AppCache, AppEntry, EXTENSIONS, launch_app(), launch_app_blocking(), launcher_apps(), launcher_list(), SHELL_PREFIX (+10 more)

### Community 39 - "prepare_rotation"
Cohesion: 0.21
Nodes (11): access_token(), CredBuffer, decode_json(), discover(), file_login(), Login, nonempty(), now_ms() (+3 more)

### Community 40 - "i18n.js"
Cohesion: 0.25
Nodes (14): EN, PATTERNS, applyAttrs(), applyText(), ATTRS, attrState, setLanguage(), skip() (+6 more)

### Community 41 - "Dock"
Cohesion: 0.18
Nodes (19): AppsIcon(), Dock(), activate(), apply(), beginLaunch(), closePreview(), finishLaunch(), measureRest() (+11 more)

### Community 42 - "set_avatar_image"
Cohesion: 0.23
Nodes (14): avatar_cache(), command_user_avatar(), data_url(), encode_avatar_png(), extract_embedded_images(), image_data(), is_jpeg(), is_png() (+6 more)

### Community 43 - "dock_preview.rs"
Cohesion: 0.06
Nodes (22): can_show(), focus_window(), menu_open(), active(), Anchor, capture(), capture_window(), CAPTURE_WORKERS (+14 more)

### Community 44 - "shortcut_folder.rs"
Cohesion: 0.28
Nodes (11): close(), close_shortcut_folder(), configured_path(), Contents, Entry, is_open(), LABEL, open_shortcut_entry() (+3 more)

### Community 45 - "RgbaImage"
Cohesion: 0.20
Nodes (16): bitmap_rgba(), content_box(), custom_folder_png(), file_icon_png(), finish(), finish_at(), folder_custom_icon(), generic_application_icon() (+8 more)

### Community 47 - "notch.rs"
Cohesion: 0.07
Nodes (32): ABE_TOP, ABM_NEW, ABM_QUERYPOS, ABM_REMOVE, ABM_SETPOS, appbar_set(), assignments(), button_down() (+24 more)

### Community 53 - "contact_links"
Cohesion: 0.33
Nodes (5): blank_issues_enabled, contact_links, about, name, url

### Community 54 - "spotlight_search"
Cohesion: 0.19
Nodes (20): app_result(), close_spotlight(), create(), ensure(), ensure_index(), FileEntry, FileIndex, fold() (+12 more)

### Community 55 - "jump_lists.rs"
Cohesion: 0.11
Nodes (8): collection(), indirect(), item_name(), JumpGroup, JumpItem, read(), reads_real_shell_lists_without_mutating_them(), take_string()

### Community 56 - "default.json"
Cohesion: 0.33
Nodes (5): description, identifier, permissions, $schema, windows

### Community 57 - "AppHandle"
Cohesion: 0.16
Nodes (24): apply_glass_mode(), check_z_order(), close_bar_panel_impl(), command_bar_panel_state(), command_close_bar_panel(), command_open_bar_panel(), command_resize_bar_panel(), command_tray_overflow() (+16 more)

### Community 59 - "watch_tray_overflow"
Cohesion: 0.20
Nodes (9): command_send_keys(), dismiss_tray_overflow(), key_input(), pump_tray(), send_keys(), tray_is_open(), tray_win_event(), TrayPlacement (+1 more)

### Community 60 - "barPaint.jsx"
Cohesion: 0.23
Nodes (15): StripDecor(), barInk(), BarPaint(), GLASS_BASE, luminance(), useBarPaint(), cache, fillFor() (+7 more)

### Community 61 - "render"
Cohesion: 0.33
Nodes (9): app_icon(), cache(), fallback_png(), glyph(), is_shell_host(), png_data(), remember(), render() (+1 more)

### Community 63 - "Mutex"
Cohesion: 0.19
Nodes (11): APPBAR, appbar(), AVATAR_CACHE, GLASS, glass_changed(), glass_state(), PANEL, PanelState (+3 more)

### Community 66 - "dock.rs"
Cohesion: 0.05
Nodes (46): ABE_BOTTOM, ABM_NEW, ABM_QUERYPOS, ABM_REMOVE, ABM_SETPOS, ENABLED, CAN_SHOW, CAPTURE_CLASSES (+38 more)

### Community 67 - "settings.rs"
Cohesion: 0.18
Nodes (14): Calendar, CalSource, config_dir(), DEFAULT_UPDATE_URL, Dock, DockFolder, DockPinned, load() (+6 more)

### Community 69 - "context_menu.rs"
Cohesion: 0.05
Nodes (52): Action, Command, Jump, add(), application_menu(), background_menu(), clear_session(), contains_point() (+44 more)

### Community 76 - "PanelView.jsx"
Cohesion: 0.24
Nodes (8): ACTIONS, DEFAULT_APPS, deg(), ICONS, TOGGLES, WeatherCard(), TABLE, weatherInfo()

### Community 79 - ".clamp"
Cohesion: 0.16
Nodes (9): Appearance, Glass, GlassSurface, is_plain_control_shortcut(), keeps_shortcuts_with_alt(), migrates_windows_text_shortcuts_to_safe_defaults(), Shortcuts, ThemeEvent (+1 more)

### Community 82 - "backgrounds.rs"
Cohesion: 0.24
Nodes (11): bg_data(), bg_data_blocking(), bg_import(), bg_import_blocking(), dir(), EXTENSIONS, MAX_BYTES, mime() (+3 more)

### Community 83 - "Cmd"
Cohesion: 0.21
Nodes (9): Cmd, Next, Pause, Play, Prev, Seek, Toggle, run() (+1 more)

### Community 84 - "prepare_rotation"
Cohesion: 0.21
Nodes (10): Lock, prepare_rotation(), Prepared, Current, NeedsRefresh, provider_lock_prevents_a_second_rotation_and_is_released(), Rotation, Store (+2 more)

### Community 85 - "appicon.rs"
Cohesion: 0.19
Nodes (10): apply(), ATARDECER, AURORA, DEFAULT, every_icon_decodes(), image(), OCEANO, PASTEL_ATARDECER (+2 more)

### Community 87 - "Win"
Cohesion: 0.26
Nodes (6): remove(), appbar_remove(), appbar_remove_win(), NotchState, sync_windows(), Win

### Community 88 - "super"
Cohesion: 0.24
Nodes (7): entry(), get(), LEGACY_SERVICE, MASK, PROVIDERS, SERVICE, set()

### Community 90 - "Mutex"
Cohesion: 0.23
Nodes (10): CREATE_OVERLAY, HOOK, NATIVE_ANIMATION, APPBAR, dock_set_hit(), Hit, HIT, hit() (+2 more)

### Community 91 - "HRESULT"
Cohesion: 0.20
Nodes (3): Category, IAutomaticList, ICustomList

### Community 92 - "ExpandedView.jsx"
Cohesion: 0.15
Nodes (21): Battery(), Clock(), Costume(), Eyes(), isNight(), Mascot(), Mouth(), STROKE (+13 more)

### Community 93 - "Launchpad.jsx"
Cohesion: 0.23
Nodes (12): CATEGORIES, fold(), iconCache, iconQueue, iconWaiters, launch(), LaunchItem, Launchpad() (+4 more)

### Community 94 - "MediaPlayer.jsx"
Cohesion: 0.29
Nodes (11): LyricsPanel(), MediaPlayer(), useArtColor(), activeLine(), cache, clean(), find(), lrclib() (+3 more)

### Community 99 - "PCWSTR"
Cohesion: 0.43
Nodes (5): PERSONALIZE, mic_users(), open_key(), qword(), scan()

### Community 100 - "ScreenTime.jsx"
Cohesion: 0.38
Nodes (9): AppList(), Bars(), colorOf(), COLORS, fmt(), lastDays(), mergeApps(), ScreenTime() (+1 more)

### Community 102 - "MediaHandle"
Cohesion: 0.32
Nodes (5): media_control(), media_seek(), media_state(), MediaHandle, MediaState

### Community 103 - "ProviderCfg"
Cohesion: 0.38
Nodes (5): base(), Ai, LegacyModels, provider(), ProviderCfg

### Community 112 - "path"
Cohesion: 0.31
Nodes (4): copies_nested_folders(), copy_dir(), migrate_data(), OLD_DIR

### Community 114 - "program_name"
Cohesion: 0.43
Nodes (7): executable_name(), fallback_names_use_program_files_not_window_titles(), filename_label(), program_name(), reads_windows_executable_version_metadata(), resolve_program(), shell_display_name()

### Community 115 - "mark_rounded"
Cohesion: 0.33
Nodes (4): enforce_chrome_free_hwnd(), mark_rounded(), set_acrylic(), sync_panel_glass()

### Community 120 - "battery_snapshot"
Cohesion: 0.70
Nodes (5): battery_info(), battery_snapshot(), BatteryInfo, command_battery_info(), start_battery_watch()

### Community 123 - "dock_animation.rs"
Cohesion: 0.06
Nodes (34): active(), Animation, animation_info(), CACHE_PERIOD, Cloak, collect_snapshots(), Concealment, Dwm (+26 more)

### Community 124 - "DockGenie.jsx"
Cohesion: 0.43
Nodes (7): clamp(), DockGenie(), play(), tick(), draw(), ease(), mix()

## Knowledge Gaps
- **24 isolated node(s):** `CredBuffer<T>`, `Dwm`, `TemporaryLayer`, `Brightness`, `Mute` (+19 more)
  These have ≤1 connection - possible missing edges. (Counts symbols only; 645 node(s) total have ≤1 connection when file, concept and rationale nodes are included.)
- **55 thin communities (<3 nodes) omitted from report** — run `graphify query` to explore isolated nodes.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `Settings` connect `Settings` to `dock.rs`, `settings.rs`, `LPARAM`, `NotchState`, `ProviderCfg`, `set_avatar_image`, `lib.rs`, `notch.rs`, `.clamp`, `topbar.rs`, `Win`, `create`, `capture_in_progress`, `AppHandle`, `popups.rs`, `Mutex`?**
  _High betweenness centrality (0.038) - this node is a cross-community bridge._
- **What connects `CredBuffer<T>`, `Dwm`, `TemporaryLayer` to the rest of the system?**
  _24 weakly-connected nodes found - possible missing edges._
- **Should `SettingsWindow.jsx` be split into smaller, more focused modules?**
  _Cohesion score 0.07800511508951406 - nodes in this community are weakly interconnected._
- **Why does `NotchState` connect `Win` to `LPARAM`, `clipboard.rs`, `context_menu.rs`, `calendar.rs`, `NotchState`, `launcher.rs`, `set_avatar_image`, `dock_preview.rs`, `ai.rs`, `lib.rs`, `shortcut_folder.rs`, `notch.rs`, `screentime.rs`, `Settings`, `dock_animation.rs`, `popups.rs`, `ai_usage.rs`?**
  _High betweenness centrality (0.032) - this node is a cross-community bridge._
- **Should `translate.rs` be split into smaller, more focused modules?**
  _Cohesion score 0.14736842105263157 - nodes in this community are weakly interconnected._
- **Why does `react` connect `App.jsx` to `SettingsWindow.jsx`, `Productivity.jsx`, `SystemView.jsx`, `package.json`, `ScreenTime.jsx`, `Recorder.jsx`, `i18n.js`, `Dock`, `PanelView.jsx`, `barPaint.jsx`, `ExpandedView.jsx`, `DockGenie.jsx`, `dayKey`, `useStore`, `TopBar.jsx`, `@tauri-apps/api`, `Launchpad.jsx`, `MediaPlayer.jsx`?**
  _High betweenness centrality (0.020) - this node is a cross-community bridge._
- **Should `dropzone.rs` be split into smaller, more focused modules?**
  _Cohesion score 0.07198228128460686 - nodes in this community are weakly interconnected._