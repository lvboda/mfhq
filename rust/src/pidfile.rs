use std::fs;
use std::path::{Path, PathBuf};

use crate::log::log;
use crate::platform::process;

pub fn runtime_dir() -> PathBuf {
    std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|d| d.to_path_buf()))
        .unwrap_or_else(|| PathBuf::from("."))
}

fn path(name: &str) -> PathBuf {
    runtime_dir().join(name)
}

/// 记录 pid 与可执行文件路径。路径用于终止前校验，避免 pid 复用后误杀无关进程。
pub fn write(name: &str, pid: u32, exe: &Path) {
    let text = format!("{pid}\n{}", exe.display());
    if let Err(e) = fs::write(path(name), text) {
        log(&format!("write {name} failed: {e}"));
    }
}

pub fn read(name: &str) -> (u32, String) {
    let text = match fs::read_to_string(path(name)) {
        Ok(t) => t,
        Err(_) => return (0, String::new()),
    };
    let mut lines = text.lines();
    let pid = lines.next().and_then(|l| l.trim().parse().ok()).unwrap_or(0);
    (pid, lines.next().unwrap_or("").trim().to_string())
}

pub fn clear(name: &str) {
    let _ = fs::remove_file(path(name));
}

/// 只有记录的路径与进程实际路径一致时才终止。记录为空则拒绝终止。
pub fn kill_if_owned(name: &str, label: &str) {
    let (pid, expected) = read(name);
    if !process::pid_alive(pid) {
        clear(name);
        return;
    }
    if expected.is_empty() {
        log(&format!("skip killing {label} pid={pid}: missing recorded exe path"));
        return;
    }
    if !process::exe_path(pid).eq_ignore_ascii_case(&expected) {
        log(&format!("skip killing {label} pid={pid}: exe path mismatch"));
        clear(name);
        return;
    }
    if process::kill_pid(pid) {
        log(&format!("killed {label} pid={pid}"));
    }
    clear(name);
}
