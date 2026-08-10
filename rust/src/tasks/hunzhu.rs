use std::thread::sleep;
use std::time::Duration;

use crate::assets::{self, Template};
use crate::bot;
use crate::log::log;
use crate::platform::input;

const SELECT_ENEMY_OFFSET: (i32, i32) = (0, -35);
const TURN_LEFT_SECONDS: f64 = 0.25;
const FORWARD_SECONDS: f64 = 1.0;

fn jinu(level: u8) -> &'static Template {
    match level {
        2 => &assets::JINU2,
        3 => &assets::JINU3,
        _ => &assets::JINU1,
    }
}

fn start(round_no: u64, level: u8) {
    log(&format!("第 {round_no} 轮开始[{level} 级激怒]"));
    bot::focus_game();
    bot::click_if_exists(&assets::GUANBI, (0, 0));
    bot::find_and_click(&assets::FUWENKA);
    bot::find_and_click(jinu(level));
    bot::click_if_exists(&assets::WEIJINU, SELECT_ENEMY_OFFSET);
    bot::find_and_click(&assets::MAICHONGAOYI);
    log(&format!("第 {round_no} 轮完成"));
}

pub fn run(level: u8) {
    log(&format!("魂珠脚本启动[{level} 级激怒]"));
    let mut round_count: u64 = 0;
    loop {
        round_count += 1;
        start(round_count, level);
        if bot::wait_appear(&assets::DITU1, bot::DEFAULT_WAIT).is_some() {
            log(&format!("第 {round_count} 轮：检测到地图，走位"));
            input::press_for('a', TURN_LEFT_SECONDS);
            input::press_for('w', FORWARD_SECONDS);
        } else {
            log(&format!("第 {round_count} 轮：等待地图超时，跳过走位"));
        }
        sleep(Duration::from_secs(2));
    }
}
