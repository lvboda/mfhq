mod vision;

#[cfg(windows)]
mod assets;
#[cfg(windows)]
mod ce;
#[cfg(windows)]
mod config;
#[cfg(windows)]
mod bot;
#[cfg(windows)]
mod log;
#[cfg(windows)]
mod platform;
#[cfg(windows)]
mod tasks;

use std::env;

const SCRIPTS: [&str; 3] = ["controller", "hunzhu", "yingxionggu"];

fn usage() -> ! {
    eprintln!("usage:");
    eprintln!("  mfhq yingxionggu           run the yingxionggu loop");
    eprintln!("  mfhq hunzhu [1|2|3]        run the hunzhu loop");
    eprintln!("  mfhq controller            run the scheduler");
    eprintln!("  mfhq match <scene> <tmpl>  print template match score (portable)");
    eprintln!();
    eprintln!("  -c, --console              detach the RDP session to console before running");
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

/// 对应 Python 的 prompt_args：无参数时进入交互式选择。
#[cfg(windows)]
fn prompt() -> Vec<String> {
    use std::io::Write;
    let choices = SCRIPTS.join(", ");
    loop {
        eprint!("script ({choices})> ");
        let _ = std::io::stderr().flush();

        let mut line = String::new();
        if std::io::stdin().read_line(&mut line).is_err() || line.is_empty() {
            std::process::exit(1);
        }
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        if matches!(line, "help" | "-h" | "--help" | "?") {
            usage();
        }
        let parts: Vec<String> = line.split_whitespace().map(|s| s.to_string()).collect();
        if SCRIPTS.contains(&parts[0].as_str()) {
            return parts;
        }
        eprintln!("unknown script: {}", parts[0]);
    }
}

fn main() {
    let mut args: Vec<String> = env::args().skip(1).collect();

    let console = args.iter().any(|a| a == "-c" || a == "--console");
    args.retain(|a| a != "-c" && a != "--console");

    #[cfg(windows)]
    if args.is_empty() {
        args = prompt();
    }
    #[cfg(not(windows))]
    if args.is_empty() {
        usage();
    }

    #[cfg(windows)]
    if console {
        platform::console::detach_to_console();
    }
    #[cfg(not(windows))]
    let _ = console;

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
        Some("hunzhu") => {
            platform::console::install_cleanup();
            tasks::hunzhu::run(parse_level(&args[1..]));
        }
        #[cfg(windows)]
        Some("yingxionggu") => {
            platform::console::install_cleanup();
            tasks::yingxionggu::run();
        }
        #[cfg(windows)]
        Some("controller") => {
            platform::console::install_cleanup();
            tasks::controller::run();
        }
        _ => usage(),
    }
}
