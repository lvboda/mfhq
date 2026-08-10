use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::thread::sleep;
use std::time::Duration;

use crate::assets;
use crate::bot;
use crate::ce;
use crate::log::log;
use crate::platform::input;

const MAX_CE_PATCH_ATTEMPTS: u32 = 2;

/// 对应 Python 的 beibao100s：等比赛成绩期间定期开合背包，防止挂机被判定为空闲。
fn bag_guard(stop: Arc<AtomicBool>) {
    let wait = |secs: u64| -> bool {
        for _ in 0..secs * 10 {
            if stop.load(Ordering::Relaxed) {
                return true;
            }
            sleep(Duration::from_millis(100));
        }
        false
    };

    while !stop.load(Ordering::Relaxed) {
        if bot::find(&assets::DITU).is_some() {
            input::press_for('b', 0.5);
            if wait(1) {
                break;
            }
            if bot::find(&assets::ZHUANGBEI).is_some() {
                input::press_for('b', 0.5);
            }
            if wait(100) {
                break;
            }
        } else if wait(10) {
            break;
        }
    }
}

fn start(round_no: u64, ce_patch_attempts: &mut u32) {
    log(&format!("第 {round_no} 轮开始"));
    bot::focus_game();
    bot::wait_appear(&assets::DITU, bot::DEFAULT_WAIT);
    bot::find_and_click(&assets::YINGXIONGGU);
    bot::find_and_click(&assets::JIARUYINGXIONGGU);
    sleep(Duration::from_secs(2));

    if bot::find(&assets::YINGXIONGGU10CI).is_some() {
        bot::find_and_click(&assets::QUEDING);
        if *ce_patch_attempts < MAX_CE_PATCH_ATTEMPTS {
            *ce_patch_attempts += 1;
            log(&format!(
                "第 {round_no} 轮：检测到 10 次提示，第 {ce_patch_attempts} 次启动 CE 修改"
            ));
            ce::double_patch(50417, -1);
            sleep(Duration::from_secs(1));
            bot::focus_game();
        } else {
            log(&format!("第 {round_no} 轮：检测到 10 次提示，CE 修改已达最大尝试次数"));
        }
        sleep(Duration::from_secs(10));
        return;
    }

    if bot::find(&assets::SHI).is_some() {
        log(&format!("第 {round_no} 轮：购买门票"));
        bot::find_and_click(&assets::FOU);
        bot::find_and_click(&assets::DIANJICHAKAN);
        bot::click_within_region(&assets::YINGXIONGGUMENPIAO, &assets::GOUMAI1);
        bot::find_and_click(&assets::MASHANGGOUMAI1);
        bot::find_and_click(&assets::CHA);
        bot::find_and_click(&assets::JIARUYINGXIONGGU);
    }

    sleep(Duration::from_secs(10));
    if bot::find(&assets::FANHUIZHUCHENG).is_none() {
        if bot::find(&assets::ZHUANGBEI).is_some() {
            input::press_for('b', 0.5);
        }
        return;
    }

    // 等比赛成绩，无超时，与 Python 版一致；期间开一个背包守护线程。
    let stop = Arc::new(AtomicBool::new(false));
    let guard = {
        let stop = Arc::clone(&stop);
        thread::spawn(move || bag_guard(stop))
    };
    bot::wait_appear(&assets::SAICHANGCHENGJI, 0.0);
    stop.store(true, Ordering::Relaxed);
    let _ = guard.join();

    bot::find_and_click(&assets::FANHUIZHUCHENG);
    bot::find_and_click(&assets::SHI);
    log(&format!("第 {round_no} 轮完成"));
}

pub fn run() {
    log("英雄谷脚本启动");

    if !bot::game_running() {
        log("魔法哈奇未运行，尝试启动并登录");
        bot::start_game();
        bot::login_game(120.0);
    }

    let mut round_count: u64 = 0;
    let mut ce_patch_attempts: u32 = 0;
    loop {
        round_count += 1;
        start(round_count, &mut ce_patch_attempts);
        sleep(Duration::from_secs(2));
    }
}
