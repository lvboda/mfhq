use std::path::PathBuf;
use std::process::Command;
use std::thread::sleep;
use std::time::Duration;
use windows::Win32::System::SystemInformation::GetLocalTime;

use crate::ce;
use crate::pidfile;
use crate::config;
use crate::log::log;
use crate::platform::process;

fn app_exe() -> PathBuf {
    std::env::var(config::APP_EXE_ENV)
        .map(PathBuf::from)
        .unwrap_or_else(|_| std::env::current_exe().unwrap_or_else(|_| PathBuf::from(config::APP_EXE_NAME)))
}

fn is_active_time() -> bool {
    let t = unsafe { GetLocalTime() };
    config::ACTIVE_WEEKDAYS.contains(&t.wDayOfWeek)
        && t.wHour >= config::START_HOUR
        && t.wHour < config::STOP_HOUR
}

fn start_script() {
    let (pid, _) = pidfile::read(config::SCRIPT_PID_FILE);
    if process::pid_alive(pid) {
        return;
    }
    if pid != 0 {
        ce::close();
    }
    pidfile::clear(config::SCRIPT_PID_FILE);

    let exe = app_exe();
    if !exe.exists() {
        log(&format!("app exe not found: {}", exe.display()));
        return;
    }
    match Command::new(&exe).arg(config::SCHEDULED_SCRIPT).spawn() {
        Ok(child) => {
            pidfile::write(config::SCRIPT_PID_FILE, child.id(), &exe);
            log(&format!("started: {} {} pid={}", exe.display(), config::SCHEDULED_SCRIPT, child.id()));
        }
        Err(e) => log(&format!("failed to start script: {e}")),
    }
}

fn stop_script() {
    let (pid, _) = pidfile::read(config::SCRIPT_PID_FILE);
    if !process::pid_alive(pid) {
        ce::close();
        pidfile::clear(config::SCRIPT_PID_FILE);
        return;
    }
    pidfile::kill_if_owned(config::SCRIPT_PID_FILE, "yingxionggu");
    ce::close();
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
