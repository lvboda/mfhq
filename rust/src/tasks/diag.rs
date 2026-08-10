use crate::assets;
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

    println!("当前匹配阈值: {:.2}（可用 MFHQ_THRESHOLD 覆盖）", crate::bot::threshold());

    let templates: [(&str, &image::RgbImage); 8] = [
        ("fuwenka", assets::fuwenka()),
        ("jinu1", assets::jinu1()),
        ("jinu2", assets::jinu2()),
        ("jinu3", assets::jinu3()),
        ("weijinu", assets::weijinu()),
        ("maichongaoyi", assets::maichongaoyi()),
        ("ditu1", assets::ditu1()),
        ("guanbi", assets::guanbi()),
    ];

    println!();
    println!("{:<16} {:<10} {:<10} {}", "模板", "尺寸", "最高分", "位置");
    println!("{}", "-".repeat(52));
    for (name, tmpl) in templates {
        match vision::match_template(&scene, tmpl) {
            Some(m) => println!(
                "{:<16} {:<10} {:<10.4} ({}, {}){}",
                name,
                format!("{}x{}", tmpl.width(), tmpl.height()),
                m.score,
                m.x,
                m.y,
                if m.score > crate::bot::threshold() { "  命中" } else { "" }
            ),
            None => println!("{name:<16} 模板比屏幕还大"),
        }
    }
}
