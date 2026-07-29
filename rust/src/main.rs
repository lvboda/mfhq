mod vision;

use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 3 {
        eprintln!("usage: mfhq <scene> <template>");
        std::process::exit(2);
    }

    let scene = image::open(&args[1]).expect("read scene").to_rgb8();
    let tmpl = image::open(&args[2]).expect("read template").to_rgb8();

    match vision::match_template(&scene, &tmpl) {
        Some(m) => println!("{:.6} {} {}", m.score, m.x, m.y),
        None => {
            eprintln!("template larger than scene");
            std::process::exit(1);
        }
    }
}
