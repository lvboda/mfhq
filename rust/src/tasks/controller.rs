use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::thread::sleep;
use std::time::Duration;
use windows::Win32::System::SystemInformation::GetLocalTime;

use crate::ce;
use crate::config;
use crate::log::log;
use crate::platform::process;

fn app_exe() -> PathBuf {
    std::env::var(config::APP_EXE_ENV)
        .map(PathBuf::from)
        .unwrap_or_else(|_| std::env::current_exe().unwrap_or_else(|_| PathBuf::from(config::APP_EXE_NAME)))
}

fn pid_file() -> PathBuf {
    ce::runtime_dir().join(config::SCRIPT_PID_FILE)
}

fn read_pid() -> u32 {
    fs::read_to_string(pid_file())
        .ok()
        .and_then(|t| t.trim().parse().ok())
        .unwrap_or(0)
}

fn write_pid(pid: u32) {
    if let Err(e) = fs::write(pid_file(), pid.to_string()) {
        log(&format!("write pid failed: {e}"));
    }
}

fn clear_pid() {
    let _ = fs::remove_file(pid_file());
}

fn is_active_time() -> bool {
    let t = unsafe { GetLocalTime() };
    config::ACTIVE_WEEKDAYS.contains(&t.wDayOfWeek)
        && t.wHour >= config::START_HOUR
        && t.wHour < config::STOP_HOUR
}

fn start_script() {
    let pid = read_pid();
    if process::pid_alive(pid) {
        return;
    }
    if pid != 0 {
        ce::close();
    }
    clear_pid();

    let exe = app_exe();
    if !exe.exists() {
        log(&format!("app exe not found: {}", exe.display()));
        return;
    }
    match Command::new(&exe).arg(config::SCHEDULED_SCRIPT).spawn() {
        Ok(child) => {
            write_pid(child.id());
            log(&format!("started: {} {} pid={}", exe.display(), config::SCHEDULED_SCRIPT, child.id()));
        }
        Err(e) => log(&format!("failed to start script: {e}")),
    }
}

fn stop_script() {
    let pid = read_pid();
    if !process::pid_alive(pid) {
        ce::close();
        clear_pid();
        return;
    }
    process::kill_pid(pid);
    ce::close();
    clear_pid();
    log(&format!("stopped pid={pid}"));
}

pub fn run() {
    log("controller started");
    loop {
        if is_active_time() {
            start_script();
        } else {
            stop_script();
        }
        sleep(Duration::from_secs(config::CHECK_INTERVAL_SECONDS));
    }
}
