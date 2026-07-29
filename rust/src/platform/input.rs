use std::thread::sleep;
use std::time::{Duration, Instant};
use windows::Win32::UI::Input::KeyboardAndMouse::{
    INPUT, INPUT_0, INPUT_KEYBOARD, INPUT_MOUSE, KEYBD_EVENT_FLAGS, KEYBDINPUT, KEYEVENTF_KEYUP,
    MOUSEEVENTF_ABSOLUTE, MOUSEEVENTF_LEFTDOWN, MOUSEEVENTF_LEFTUP, MOUSEEVENTF_MOVE, MOUSEINPUT,
    SendInput, VIRTUAL_KEY, VkKeyScanW,
};

use super::screen;

fn send(input: INPUT) {
    unsafe {
        SendInput(&[input], std::mem::size_of::<INPUT>() as i32);
    }
}

fn to_absolute(x: i32, y: i32) -> (i32, i32) {
    let (w, h) = screen::size();
    (x * 65535 / w.max(1), y * 65535 / h.max(1))
}

pub fn move_to(x: i32, y: i32) {
    let (ax, ay) = to_absolute(x, y);
    send(INPUT {
        r#type: INPUT_MOUSE,
        Anonymous: INPUT_0 {
            mi: MOUSEINPUT {
                dx: ax,
                dy: ay,
                mouseData: 0,
                dwFlags: MOUSEEVENTF_MOVE | MOUSEEVENTF_ABSOLUTE,
                time: 0,
                dwExtraInfo: 0,
            },
        },
    });
}

pub fn click(x: i32, y: i32) {
    move_to(x, y);
    sleep(Duration::from_millis(20));
    for flag in [MOUSEEVENTF_LEFTDOWN, MOUSEEVENTF_LEFTUP] {
        send(INPUT {
            r#type: INPUT_MOUSE,
            Anonymous: INPUT_0 {
                mi: MOUSEINPUT {
                    dx: 0,
                    dy: 0,
                    mouseData: 0,
                    dwFlags: flag,
                    time: 0,
                    dwExtraInfo: 0,
                },
            },
        });
        sleep(Duration::from_millis(10));
    }
}

fn key_input(vk: VIRTUAL_KEY, up: bool) -> INPUT {
    INPUT {
        r#type: INPUT_KEYBOARD,
        Anonymous: INPUT_0 {
            ki: KEYBDINPUT {
                wVk: vk,
                wScan: 0,
                dwFlags: if up { KEYEVENTF_KEYUP } else { KEYBD_EVENT_FLAGS(0) },
                time: 0,
                dwExtraInfo: 0,
            },
        },
    }
}

fn vk_of(ch: char) -> VIRTUAL_KEY {
    unsafe { VIRTUAL_KEY((VkKeyScanW(ch as u16) & 0xff) as u16) }
}

/// 对应 Python 的 press_with_correction：按住指定时长后释放。
/// Python 版用忙等待，这里用 sleep —— 时长语义相同，但不占满一个核心。
pub fn press_for(ch: char, seconds: f64) {
    let vk = vk_of(ch);
    send(key_input(vk, false));
    let start = Instant::now();
    let target = Duration::from_secs_f64(seconds);
    while start.elapsed() < target {
        sleep(Duration::from_millis(1));
    }
    send(key_input(vk, true));
}
