use image::RgbImage;
use std::sync::OnceLock;

macro_rules! template {
    ($name:ident, $file:literal) => {
        pub fn $name() -> &'static RgbImage {
            static CELL: OnceLock<RgbImage> = OnceLock::new();
            CELL.get_or_init(|| {
                image::load_from_memory(include_bytes!(concat!(
                    "../../src/common/photo/",
                    $file
                )))
                .expect(concat!("decode ", $file))
                .to_rgb8()
            })
        }
    };
}

template!(fuwenka, "fuwenka.png");
template!(jinu1, "jinu1.png");
template!(jinu2, "jinu2.png");
template!(jinu3, "jinu3.png");
template!(weijinu, "weijinu.png");
template!(maichongaoyi, "maichongaoyi.png");
template!(ditu1, "ditu1.png");
template!(guanbi, "guanbi.png");

pub fn jinu_by_level(level: u8) -> &'static RgbImage {
    match level {
        2 => jinu2(),
        3 => jinu3(),
        _ => jinu1(),
    }
}
