use windows::Win32::Foundation::CloseHandle;
use windows::Win32::System::Diagnostics::ToolHelp::{
    CreateToolhelp32Snapshot, PROCESSENTRY32W, Process32FirstW, Process32NextW, TH32CS_SNAPPROCESS,
};
use windows::Win32::System::Threading::{OpenProcess, PROCESS_TERMINATE, TerminateProcess};

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

pub fn kill(names: &[&str]) -> bool {
    let mut pids = Vec::new();
    each_process(|pid, name| {
        if names.iter().any(|n| n.eq_ignore_ascii_case(name)) {
            pids.push(pid);
        }
    });

    let mut killed = false;
    for pid in pids {
        unsafe {
            if let Ok(handle) = OpenProcess(PROCESS_TERMINATE, false, pid) {
                if TerminateProcess(handle, 1).is_ok() {
                    killed = true;
                }
                let _ = CloseHandle(handle);
            }
        }
    }
    killed
}
