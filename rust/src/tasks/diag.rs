use crate::assets;
use crate::bot;
use crate::platform::screen;
use crate::vision;

/// 截一张屏保存到磁盘，并逐一打印各模板的匹配分数，用于定位识别失败的原因。
pub fn run(out: &str) {
    let (w, h) = screen::size();
    println!("GetSystemMetrics 报告的屏幕尺寸: {w} x {h}");

    let scene = match screen::capture() {
        Some(s) => s,
        None => {
            println!("截屏失败");
            return;
        }
    };
    println!("实际截到的位图尺寸: {} x {}", scene.width(), scene.height());

    if let Err(e) = scene.save(out) {
        println!("保存截图失败: {e}");
    } else {
        println!("截图已保存: {out}");
    }

    let threshold = bot::threshold();
    println!("当前匹配阈值: {threshold:.2}（可用 MFHQ_THRESHOLD 覆盖）");
    println!();
    println!("{:<26} {:<10} {:<10} {}", "模板", "尺寸", "最高分", "位置");
    println!("{}", "-".repeat(60));

    for tmpl in assets::ALL {
        let img = tmpl.image();
        match vision::match_template(&scene, img) {
            Some(m) => println!(
                "{:<26} {:<10} {:<10.4} ({}, {}){}",
                tmpl.name,
                format!("{}x{}", img.width(), img.height()),
                m.score,
                m.x,
                m.y,
                if m.score > threshold { "  命中" } else { "" }
            ),
            None => println!("{:<26} 模板比屏幕还大", tmpl.name),
        }
    }
}
