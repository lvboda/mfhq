use std::path::Path;
use windows::Win32::Foundation::CloseHandle;
use windows::Win32::System::Diagnostics::ToolHelp::{
    CreateToolhelp32Snapshot, PROCESSENTRY32W, Process32FirstW, Process32NextW, TH32CS_SNAPPROCESS,
};
use windows::Win32::System::Threading::{
    OpenProcess, PROCESS_QUERY_LIMITED_INFORMATION, PROCESS_TERMINATE, QueryFullProcessImageNameW,
    TerminateProcess,
};
use windows::Win32::UI::Shell::ShellExecuteW;
use windows::Win32::UI::WindowsAndMessaging::SW_SHOWNORMAL;
use windows::core::{HSTRING, PCWSTR};

fn entry_name(entry: &PROCESSENTRY32W) -> String {
    let end = entry
        .szExeFile
        .iter()
        .position(|&c| c == 0)
        .unwrap_or(entry.szExeFile.len());
    String::from_utf16_lossy(&entry.szExeFile[..end])
}

fn each_process<F: FnMut(u32, &str)>(mut f: F) {
    unsafe {
        let snapshot = match CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0) {
            Ok(h) => h,
            Err(_) => return,
        };
        let mut entry = PROCESSENTRY32W {
            dwSize: std::mem::size_of::<PROCESSENTRY32W>() as u32,
            ..Default::default()
        };
        if Process32FirstW(snapshot, &mut entry).is_ok() {
            loop {
                f(entry.th32ProcessID, &entry_name(&entry));
                if Process32NextW(snapshot, &mut entry).is_err() {
                    break;
                }
            }
        }
        let _ = CloseHandle(snapshot);
    }
}

pub fn is_running(names: &[&str]) -> bool {
    let mut found = false;
    each_process(|_, name| {
        if names.iter().any(|n| n.eq_ignore_ascii_case(name)) {
            found = true;
        }
    });
    found
}

pub fn pid_alive(pid: u32) -> bool {
    if pid == 0 {
        return false;
    }
    let mut alive = false;
    each_process(|p, _| {
        if p == pid {
            alive = true;
        }
    });
    alive
}

pub fn exe_path(pid: u32) -> String {
    if pid == 0 {
        return String::new();
    }
    unsafe {
        let handle = match OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid) {
            Ok(h) => h,
            Err(_) => return String::new(),
        };
        let mut buf = [0u16; 1024];
        let mut len = buf.len() as u32;
        let ok = QueryFullProcessImageNameW(
            handle,
            Default::default(),
            windows::core::PWSTR(buf.as_mut_ptr()),
            &mut len,
        )
        .is_ok();
        let _ = CloseHandle(handle);
        if ok {
            String::from_utf16_lossy(&buf[..len as usize])
        } else {
            String::new()
        }
    }
}

pub fn kill_by_name(names: &[&str]) -> bool {
    let mut pids = Vec::new();
    each_process(|pid, name| {
        if names.iter().any(|n| n.eq_ignore_ascii_case(name)) {
            pids.push(pid);
        }
    });
    pids.into_iter().map(kill_pid).any(|ok| ok)
}

pub fn kill_pid(pid: u32) -> bool {
    unsafe {
        match OpenProcess(PROCESS_TERMINATE, false, pid) {
            Ok(handle) => {
                let ok = TerminateProcess(handle, 1).is_ok();
                let _ = CloseHandle(handle);
                ok
            }
            Err(_) => false,
        }
    }
}

/// 对应 Python 的 os.startfile。
pub fn shell_open(path: &Path) -> bool {
    let file = HSTRING::from(path.as_os_str());
    unsafe {
        let result = ShellExecuteW(
            None,
            PCWSTR::null(),
            PCWSTR(file.as_ptr()),
            PCWSTR::null(),
            PCWSTR::null(),
            SW_SHOWNORMAL,
        );
        result.0 as usize > 32
    }
}
