mod vision;

#[cfg(windows)]
mod assets;
#[cfg(windows)]
mod bot;
#[cfg(windows)]
mod log;
#[cfg(windows)]
mod platform;
#[cfg(windows)]
mod tasks;

use std::env;

fn usage() -> ! {
    eprintln!("usage:");
    eprintln!("  mfhq hunzhu [1|2|3]        run the hunzhu loop");
    eprintln!("  mfhq match <scene> <tmpl>  print template match score (portable)");
    std::process::exit(2)
}

/// 解析等级：位置参数与 -l / -level / --level 等价，默认 1。
fn parse_level(args: &[String]) -> u8 {
    let mut i = 0;
    while i < args.len() {
        let a = args[i].as_str();
        let value = match a {
            "-l" | "-level" | "--level" | "--l" | "--le" | "--lev" | "--leve" => {
                i += 1;
                args.get(i).map(|s| s.as_str())
            }
            other if !other.starts_with('-') => Some(other),
            _ => None,
        };
        if let Some(v) = value {
            if let Ok(n) = v.parse::<u8>() {
                if (1..=3).contains(&n) {
                    return n;
                }
            }
            eprintln!("invalid level: {v} (expected 1, 2 or 3)");
            std::process::exit(2);
        }
        i += 1;
    }
    1
}

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    match args.first().map(|s| s.as_str()) {
        Some("match") if args.len() == 3 => {
            let scene = image::open(&args[1]).expect("read scene").to_rgb8();
            let tmpl = image::open(&args[2]).expect("read template").to_rgb8();
            match vision::match_template(&scene, &tmpl) {
                Some(m) => println!("{:.6} {} {}", m.score, m.x, m.y),
                None => std::process::exit(1),
            }
        }
        #[cfg(windows)]
        Some("hunzhu") => tasks::hunzhu::run(parse_level(&args[1..])),
        #[cfg(not(windows))]
        Some("hunzhu") => {
            eprintln!("hunzhu is only available on Windows");
            std::process::exit(1);
        }
        _ => usage(),
    }
}
