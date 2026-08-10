use std::process::Command;
use windows::core::BOOL;
use windows::Win32::System::Console::{CTRL_BREAK_EVENT, CTRL_C_EVENT, SetConsoleCtrlHandler};
use windows::Win32::System::RemoteDesktop::ProcessIdToSessionId;
use windows::Win32::System::Threading::GetCurrentProcessId;

unsafe extern "system" fn ctrl_handler(event: u32) -> BOOL {
    if event == CTRL_C_EVENT || event == CTRL_BREAK_EVENT {
        crate::ce::close();
        std::process::exit(130);
    }
    BOOL(0)
}

/// 对应 Python 的 atexit + SIGINT/SIGTERM：退出前关闭 CE 进程。
pub fn install_cleanup() {
    unsafe {
        let _ = SetConsoleCtrlHandler(Some(ctrl_handler), true);
    }
}

fn session_id() -> Option<u32> {
    unsafe {
        let mut id = 0u32;
        if ProcessIdToSessionId(GetCurrentProcessId(), &mut id).is_ok() {
            Some(id)
        } else {
            None
        }
    }
}

/// 对应 Python 的 detach_to_console：把当前 RDP 会话切到控制台，
/// 断开远程连接后仍保留可用于截图的交互式桌面。
pub fn detach_to_console() {
    if std::env::var("SESSIONNAME")
        .map(|s| s.eq_ignore_ascii_case("console"))
        .unwrap_or(false)
    {
        println!("already running on console session; skipped tscon");
        return;
    }

    let id = match session_id() {
        Some(id) => id,
        None => {
            println!("failed to get current session id; skipped tscon");
            return;
        }
    };

    let windir = std::env::var("WINDIR").unwrap_or_else(|_| r"C:\Windows".to_string());
    let tscon = format!(r"{windir}\System32\tscon.exe");
    let output = Command::new(&tscon)
        .arg(id.to_string())
        .arg("/dest:console")
        .output();

    match output {
        Ok(out) if out.status.success() => println!("detaching RDP session {id} to console"),
        Ok(out) => {
            let err = String::from_utf8_lossy(if out.stderr.is_empty() { &out.stdout } else { &out.stderr });
            println!("tscon failed for session {id}: {}", err.trim());
            println!("try running mfhq.exe as Administrator");
        }
        Err(e) => {
            println!("failed to run tscon for session {id}: {e}");
        }
    }
}
