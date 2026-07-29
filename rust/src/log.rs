use std::fs::OpenOptions;
use std::io::Write;
use windows::Win32::System::SystemInformation::GetLocalTime;

pub fn log(message: &str) {
    let t = unsafe { GetLocalTime() };
    let line = format!(
        "[{:04}-{:02}-{:02} {:02}:{:02}:{:02}] {}",
        t.wYear, t.wMonth, t.wDay, t.wHour, t.wMinute, t.wSecond, message
    );
    println!("{line}");

    let path = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|d| d.join("yingxionggu.log")));
    if let Some(path) = path {
        if let Ok(mut f) = OpenOptions::new().create(true).append(true).open(path) {
            let _ = writeln!(f, "{line}");
        }
    }
}
