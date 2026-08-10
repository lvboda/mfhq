use std::thread::sleep;
use std::time::{Duration, Instant};

use crate::assets::{self, Template};
use crate::config;
use crate::log::log;
use crate::platform::{input, process, screen, window};
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

/// 在给定画面里找模板，命中返回中心坐标。
fn locate(scene: &image::RgbImage, tmpl: &Template) -> Option<(i32, i32)> {
    let img = tmpl.image();
    let m = vision::match_template(scene, img)?;
    if m.score <= threshold() {
        return None;
    }
    Some((
        m.x as i32 + img.width() as i32 / 2,
        m.y as i32 + img.height() as i32 / 2,
    ))
}

/// 截一次屏并匹配。对应 Python 的 find_img。
pub fn find(tmpl: &Template) -> Option<(i32, i32)> {
    match screen::capture() {
        Some(scene) => locate(&scene, tmpl),
        None => {
            // 截屏失败与"没找到"是两回事，不记一笔的话无限等待会静默卡死。
            log("截屏失败");
            None
        }
    }
}

/// 轮询直到出现或超时。seconds 为 0 表示无限等待，与 Python 版语义一致。
pub fn wait_appear(tmpl: &Template, seconds: f64) -> Option<(i32, i32)> {
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
pub fn find_and_click(tmpl: &Template) {
    match wait_appear(tmpl, DEFAULT_WAIT) {
        Some((x, y)) => {
            input::click(x, y);
            // 与 Python 版一致：点击后把指针移开，避免悬停影响后续截图。
            input::move_to(200, 200);
        }
        None => log(&format!("卡片{}没有找到", tmpl.name)),
    }
}

/// 只匹配一次，命中则按偏移点击。对应 Python 的 find_and_clickIfExist。
pub fn click_if_exists(tmpl: &Template, offset: (i32, i32)) {
    if let Some((x, y)) = find(tmpl) {
        input::click(x + offset.0, y + offset.1);
        sleep(Duration::from_secs(1));
    }
}

/// 两级匹配：先在全屏找 base，再在 base 命中的区域内找 target。
/// 对应 Python 的 find_img_within_region。
fn find_within_region(base: &Template, target: &Template) -> Option<(i32, i32)> {
    let scene = screen::capture()?;
    let base_img = base.image();
    let b = vision::match_template(&scene, base_img)?;
    if b.score <= threshold() {
        return None;
    }
    let region =
        image::imageops::crop_imm(&scene, b.x, b.y, base_img.width(), base_img.height()).to_image();
    let (tx, ty) = locate(&region, target)?;
    Some((b.x as i32 + tx, b.y as i32 + ty))
}

/// 对应 Python 的 find_and_click_within_region：一直轮询直到命中，无超时。
pub fn click_within_region(base: &Template, target: &Template) {
    loop {
        if let Some((x, y)) = find_within_region(base, target) {
            input::click(x, y);
            sleep(Duration::from_secs(1));
            return;
        }
        sleep(Duration::from_millis(300));
    }
}

pub fn focus_game() {
    window::focus_game(&config::GAME_WINDOW_KEYS);
}

pub fn game_running() -> bool {
    process::is_running(&config::GAME_PROCESS_NAMES)
}

/// 对应 Python 的 start_mofa_haqi：拉起客户端并等待进程出现，最多 60 秒。
pub fn start_game() {
    if game_running() {
        return;
    }
    let path = std::env::var(config::GAME_PATH_ENV)
        .unwrap_or_else(|_| config::GAME_DEFAULT_PATH.to_string());
    if !process::shell_open(std::path::Path::new(&path)) {
        log(&format!("启动魔法哈奇失败: {path}"));
        return;
    }
    for _ in 0..60 {
        if game_running() {
            return;
        }
        sleep(Duration::from_secs(1));
    }
    log("等待魔法哈奇进程超时");
}

/// 对应 Python 的 login_mofa_haqi：点过登录流程后轮询地图，最多 timeout 秒。
pub fn login_game(timeout: f64) {
    if find(&assets::DITU).is_some() {
        return;
    }

    find_and_click(&assets::JINRUYOUXI);
    sleep(Duration::from_secs(2));
    focus_game();
    find_and_click(&assets::DENGLU);
    find_and_click(&assets::JINRUYOUXI2);
    find_and_click(&assets::PAOPAO);

    let start = Instant::now();
    while start.elapsed().as_secs_f64() < timeout {
        click_if_exists(&assets::CHA, (0, 0));
        if find(&assets::DITU).is_some() {
            return;
        }
        sleep(Duration::from_secs(1));
    }
    log("登录魔法哈奇超时");
}
