use image::RgbImage;
use std::thread::sleep;
use std::time::{Duration, Instant};

use crate::platform::{input, screen};
use crate::vision;

pub const DEFAULT_THRESHOLD: f64 = 0.7;
pub const DEFAULT_WAIT: f64 = 40.0;

/// 匹配阈值，可用 MFHQ_THRESHOLD 覆盖，便于在目标机器上直接调参而无需重新编译。
pub fn threshold() -> f64 {
    static CELL: std::sync::OnceLock<f64> = std::sync::OnceLock::new();
    *CELL.get_or_init(|| {
        std::env::var("MFHQ_THRESHOLD")
            .ok()
            .and_then(|v| v.parse().ok())
            .filter(|v: &f64| *v > 0.0 && *v < 1.0)
            .unwrap_or(DEFAULT_THRESHOLD)
    })
}

pub use crate::config::{GAME_PROCESS_NAMES, GAME_WINDOW_KEYS};

/// 截一次屏并匹配，命中返回匹配区域中心坐标。对应 Python 的 find_img。
pub fn find(tmpl: &RgbImage) -> Option<(i32, i32)> {
    let scene = screen::capture()?;
    let m = vision::match_template(&scene, tmpl)?;
    if m.score <= threshold() {
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

/// 两级匹配：先在全屏找 base，再在 base 命中的区域内找 target。
/// 对应 Python 的 find_img_within_region。
pub fn find_within_region(base: &RgbImage, target: &RgbImage) -> Option<(i32, i32)> {
    let scene = screen::capture()?;
    let b = vision::match_template(&scene, base)?;
    if b.score <= threshold() {
        return None;
    }
    let region = image::imageops::crop_imm(&scene, b.x, b.y, base.width(), base.height()).to_image();
    let t = vision::match_template(&region, target)?;
    if t.score <= threshold() {
        return None;
    }
    Some((
        b.x as i32 + t.x as i32 + target.width() as i32 / 2,
        b.y as i32 + t.y as i32 + target.height() as i32 / 2,
    ))
}

/// 对应 Python 的 find_and_click_within_region：一直轮询直到命中，无超时。
pub fn click_within_region(base: &RgbImage, target: &RgbImage) {
    loop {
        if let Some((x, y)) = find_within_region(base, target) {
            input::click(x, y);
            sleep(Duration::from_secs(1));
            return;
        }
        sleep(Duration::from_millis(300));
    }
}

pub fn game_running() -> bool {
    crate::platform::process::is_running(&GAME_PROCESS_NAMES)
}

/// 对应 Python 的 start_mofa_haqi：拉起客户端并等待进程出现，最多 60 秒。
pub fn start_game() -> bool {
    if game_running() {
        return true;
    }
    let path = std::env::var(crate::config::GAME_PATH_ENV)
        .unwrap_or_else(|_| crate::config::GAME_DEFAULT_PATH.to_string());
    if !crate::platform::process::shell_open(std::path::Path::new(&path)) {
        crate::log::log(&format!("启动魔法哈奇失败: {path}"));
        return false;
    }
    for _ in 0..60 {
        if game_running() {
            return true;
        }
        sleep(Duration::from_secs(1));
    }
    false
}

/// 对应 Python 的 login_mofa_haqi：点过登录流程后轮询地图，最多 timeout 秒。
pub fn login_game(timeout: f64) -> bool {
    if find(crate::assets::ditu()).is_some() {
        return true;
    }

    find_and_click(crate::assets::jinruyouxi(), "jinruyouxi.jpg");
    sleep(Duration::from_secs(2));
    crate::platform::window::focus_game(&GAME_WINDOW_KEYS);
    find_and_click(crate::assets::denglu(), "denglu.jpg");
    find_and_click(crate::assets::jinruyouxi2(), "jinruyouxi2.jpg");
    find_and_click(crate::assets::paopao(), "paopao.jpg");

    let start = Instant::now();
    while start.elapsed().as_secs_f64() < timeout {
        click_if_exists(crate::assets::cha(), (0, 0));
        if find(crate::assets::ditu()).is_some() {
            return true;
        }
        sleep(Duration::from_secs(1));
    }
    crate::log::log("登录魔法哈奇超时");
    false
}
