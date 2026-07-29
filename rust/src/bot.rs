use image::RgbImage;
use std::thread::sleep;
use std::time::{Duration, Instant};

use crate::platform::{input, screen};
use crate::vision;

pub const THRESHOLD: f64 = 0.8;
pub const DEFAULT_WAIT: f64 = 40.0;

pub const GAME_PROCESS_NAMES: [&str; 2] = ["paraengineclient.exe", "00000BAC-paraengineclient.exe"];
pub const GAME_WINDOW_KEYS: [&str; 3] = ["魔法哈奇", "哈奇", "paraengine"];

/// 截一次屏并匹配，命中返回匹配区域中心坐标。对应 Python 的 find_img。
pub fn find(tmpl: &RgbImage) -> Option<(i32, i32)> {
    let scene = screen::capture()?;
    let m = vision::match_template(&scene, tmpl)?;
    if m.score <= THRESHOLD {
        return None;
    }
    Some((
        m.x as i32 + tmpl.width() as i32 / 2,
        m.y as i32 + tmpl.height() as i32 / 2,
    ))
}

/// 轮询直到出现或超时。seconds 为 0 表示无限等待，与 Python 版语义一致。
pub fn wait_appear(tmpl: &RgbImage, seconds: f64) -> Option<(i32, i32)> {
    let start = Instant::now();
    loop {
        if let Some(pos) = find(tmpl) {
            return Some(pos);
        }
        if seconds != 0.0 && start.elapsed().as_secs_f64() >= seconds {
            return None;
        }
        sleep(Duration::from_millis(200));
    }
}

/// 轮询直到出现后点击，超时则放弃。对应 Python 的 find_and_click。
pub fn find_and_click(tmpl: &RgbImage, name: &str) -> bool {
    match wait_appear(tmpl, DEFAULT_WAIT) {
        Some((x, y)) => {
            input::click(x, y);
            // 与 Python 版一致：点击后把指针移开，避免悬停影响后续截图。
            input::move_to(200, 200);
            true
        }
        None => {
            crate::log::log(&format!("卡片{name}没有找到"));
            false
        }
    }
}

/// 只匹配一次，命中则按偏移点击。对应 Python 的 find_and_clickIfExist。
pub fn click_if_exists(tmpl: &RgbImage, offset: (i32, i32)) -> bool {
    match find(tmpl) {
        Some((x, y)) => {
            input::click(x + offset.0, y + offset.1);
            sleep(Duration::from_secs(1));
            true
        }
        None => false,
    }
}
