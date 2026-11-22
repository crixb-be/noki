#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    if std::env::args_os().nth(1).as_deref() == Some(std::ffi::OsStr::new("--antigravity-statusline")) {
        noki_lib::run_antigravity_statusline();
        return;
    }
    noki_lib::run()
}
