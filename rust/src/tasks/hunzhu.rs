use std::thread::sleep;
use std::time::Duration;

use crate::assets;
use crate::bot;
use crate::log::log;
use crate::platform::{input, window};

const SELECT_ENEMY_OFFSET: (i32, i32) = (0, -35);
const MAP_WAIT_SECONDS: f64 = 40.0;
const TURN_LEFT_SECONDS: f64 = 0.25;
const FORWARD_SECONDS: f64 = 1.0;

fn start(round_no: u64, level: u8) {
    log(&format!("第 {round_no} 轮开始[{level} 级激怒]"));
    window::focus_game(&bot::GAME_WINDOW_KEYS);
    bot::click_if_exists(assets::guanbi(), (0, 0));
    bot::find_and_click(assets::fuwenka(), "fuwenka.png");
    bot::find_and_click(assets::jinu_by_level(level), "jinu.png");
    bot::click_if_exists(assets::weijinu(), SELECT_ENEMY_OFFSET);
    bot::find_and_click(assets::maichongaoyi(), "maichongaoyi.png");
    log(&format!("第 {round_no} 轮完成"));
}

pub fn run(level: u8) {
    log(&format!("魂珠脚本启动[{level} 级激怒]"));
    let mut round_count: u64 = 0;
    loop {
        round_count += 1;
        start(round_count, level);
        if bot::wait_appear(assets::ditu1(), MAP_WAIT_SECONDS).is_some() {
            log(&format!("第 {round_count} 轮：检测到地图，走位"));
            input::press_for('a', TURN_LEFT_SECONDS);
            input::press_for('w', FORWARD_SECONDS);
        } else {
            log(&format!("第 {round_count} 轮：等待地图超时，跳过走位"));
        }
        sleep(Duration::from_secs(2));
    }
}
