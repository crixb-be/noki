//! Rendimiento del equipo: CPU, memoria, discos, red y los programas que más recursos consumen.
//! Solo se consulta mientras la pestaña «Sistema» está abierta.

use serde::Serialize;
use std::{collections::HashMap, sync::Mutex, time::Instant};
use sysinfo::{Disks, Networks, ProcessesToUpdate, System};
use tauri::State;

pub struct SysMon {
    sys: System,
    disks: Disks,
    nets: Networks,
    last_net: Option<Instant>,
}

impl SysMon {
    pub fn new() -> Self {
        let mut sys = System::new();
        sys.refresh_cpu_usage();
        sys.refresh_memory();
        Self { sys, disks: Disks::new_with_refreshed_list(), nets: Networks::new_with_refreshed_list(), last_net: None }
    }
}

pub type SysMonState = Mutex<SysMon>;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DiskInfo {
    name: String,
    used: u64,
    total: u64,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Stats {
    /// Uso total de la CPU en %.
    cpu: f32,
    mem_used: u64,
    mem_total: u64,
    disks: Vec<DiskInfo>,
    /// Bytes por segundo.
    net_down: f64,
    net_up: f64,
    /// Segundos desde que se encendió el equipo.
    uptime: u64,
}

#[tauri::command]
pub fn system_stats(mon: State<SysMonState>) -> Stats {
    let mut m = mon.lock().unwrap();
    m.sys.refresh_cpu_usage();
    m.sys.refresh_memory();
    m.disks.refresh(true);

    let now = Instant::now();
    m.nets.refresh(true);
    let secs = m.last_net.map(|t| now.duration_since(t).as_secs_f64()).unwrap_or(0.0).max(0.001);
    let (rx, tx) = m.nets.iter().fold((0u64, 0u64), |(r, t), (_, n)| (r + n.received(), t + n.transmitted()));
    let first = m.last_net.is_none();
    m.last_net = Some(now);

    let mut seen = std::collections::HashSet::new();
    let disks = m
        .disks
        .iter()
        .filter(|d| d.total_space() > 0 && seen.insert(d.mount_point().to_string_lossy().to_string()))
        .map(|d| {
            let mount = d.mount_point().to_string_lossy().trim_end_matches('\\').to_string();
            DiskInfo { name: mount, used: d.total_space() - d.available_space(), total: d.total_space() }
        })
        .collect();

    Stats {
        cpu: m.sys.global_cpu_usage(),
        mem_used: m.sys.used_memory(),
        mem_total: m.sys.total_memory(),
        disks,
        net_down: if first { 0.0 } else { rx as f64 / secs },
        net_up: if first { 0.0 } else { tx as f64 / secs },
        uptime: System::uptime(),
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProcInfo {
    /// Nombre del ejecutable (por ejemplo «chrome.exe»); agrupa todos los procesos del programa.
    name: String,
    count: usize,
    /// Memoria total en bytes.
    mem: u64,
    /// CPU en % del equipo completo.
    cpu: f32,
}

/// Programas que más consumen, agrupados por nombre. `by`: "mem" (memoria) o "cpu".
#[tauri::command]
pub fn top_processes(mon: State<SysMonState>, by: String) -> Vec<ProcInfo> {
    let mut m = mon.lock().unwrap();
    m.sys.refresh_processes(ProcessesToUpdate::All, true);
    let cores = m.sys.cpus().len().max(1) as f32;
    let mut groups: HashMap<String, ProcInfo> = HashMap::new();
    for p in m.sys.processes().values() {
        let name = p.name().to_string_lossy().to_string();
        let g = groups.entry(name.clone()).or_insert(ProcInfo { name, count: 0, mem: 0, cpu: 0.0 });
        g.count += 1;
        g.mem += p.memory();
        g.cpu += p.cpu_usage() / cores;
    }
    let mut list: Vec<ProcInfo> = groups.into_values().filter(|g| g.mem > 0).collect();
    if by == "cpu" {
        list.sort_by(|a, b| b.cpu.partial_cmp(&a.cpu).unwrap_or(std::cmp::Ordering::Equal));
    } else {
        list.sort_by(|a, b| b.mem.cmp(&a.mem));
    }
    list.truncate(8);
    list
}

/// Procesos que nunca se cierran desde aquí (el sistema, el Explorador y Noki mismo).
const PROTECTED: [&str; 14] = [
    "system", "system idle process", "registry", "smss.exe", "csrss.exe", "wininit.exe", "winlogon.exe", "services.exe", "lsass.exe",
    "svchost.exe", "dwm.exe", "explorer.exe", "fontdrvhost.exe", "noki.exe",
];

pub fn is_protected(name: &str) -> bool {
    PROTECTED.iter().any(|p| p.eq_ignore_ascii_case(name))
}

/// Cierra todos los procesos de un programa. Devuelve cuántos cerró.
#[tauri::command]
pub fn end_task(mon: State<SysMonState>, name: String) -> Result<usize, String> {
    if is_protected(&name) {
        return Err("Este proceso es del sistema y no se puede cerrar desde aquí.".into());
    }
    let mut m = mon.lock().unwrap();
    m.sys.refresh_processes(ProcessesToUpdate::All, true);
    let killed = m
        .sys
        .processes()
        .values()
        .filter(|p| p.name().to_string_lossy().eq_ignore_ascii_case(&name))
        .filter(|p| p.kill())
        .count();
    if killed == 0 {
        Err("No se pudo cerrar (puede que necesite permisos de administrador).".into())
    } else {
        Ok(killed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn protects_system_and_self() {
        assert!(is_protected("Explorer.EXE"));
        assert!(is_protected("noki.exe"));
        assert!(!is_protected("chrome.exe"));
    }

    #[test]
    fn stats_are_sane() {
        let mon = Mutex::new(SysMon::new());
        let mut m = mon.lock().unwrap();
        m.sys.refresh_memory();
        assert!(m.sys.total_memory() > 0);
        assert!(m.sys.used_memory() <= m.sys.total_memory());
    }
}
