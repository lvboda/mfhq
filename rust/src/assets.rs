use image::RgbImage;
use std::sync::OnceLock;

/// 模板自带文件名，避免调用处再手写一遍字符串——两者分开写过一次就已经漂移过
/// （日志打 jinu.png 而实际搜的是 jinu2.png）。
pub struct Template {
    pub name: &'static str,
    bytes: &'static [u8],
    cell: OnceLock<RgbImage>,
}

impl Template {
    pub fn image(&self) -> &RgbImage {
        self.cell.get_or_init(|| {
            image::load_from_memory(self.bytes)
                .unwrap_or_else(|e| panic!("decode {}: {e}", self.name))
                .to_rgb8()
        })
    }
}

macro_rules! templates {
    ($($ident:ident => $file:literal),* $(,)?) => {
        $(
            pub static $ident: Template = Template {
                name: $file,
                bytes: include_bytes!(concat!("../photo/", $file)),
                cell: OnceLock::new(),
            };
        )*
        /// 全部模板，`diag` 据此遍历，新增模板无需再改别处。
        pub const ALL: &[&Template] = &[$(&$ident),*];
    };
}

templates! {
    FUWENKA => "fuwenka.png",
    JINU1 => "jinu1.png",
    JINU2 => "jinu2.png",
    JINU3 => "jinu3.png",
    WEIJINU => "weijinu.png",
    MAICHONGAOYI => "maichongaoyi.png",
    DITU1 => "ditu1.png",
    GUANBI => "guanbi.png",
    DITU => "ditu.jpg",
    YINGXIONGGU => "yingxionggu.jpg",
    JIARUYINGXIONGGU => "jiaruyingxionggu.jpg",
    YINGXIONGGU10CI => "yingxionggu10ci.png",
    QUEDING => "queding.jpg",
    SHI => "shi.jpg",
    FOU => "fou.jpg",
    DIANJICHAKAN => "dianjichakan.jpg",
    YINGXIONGGUMENPIAO => "yingxionggumenpiao.jpg",
    GOUMAI1 => "goumai1.jpg",
    MASHANGGOUMAI1 => "mashanggoumai1.jpg",
    CHA => "cha.jpg",
    FANHUIZHUCHENG => "fanhuizhucheng.jpg",
    ZHUANGBEI => "zhuangbei.jpg",
    SAICHANGCHENGJI => "saichangchengji.jpg",
    JINRUYOUXI => "jinruyouxi.jpg",
    JINRUYOUXI2 => "jinruyouxi2.jpg",
    DENGLU => "denglu.jpg",
    PAOPAO => "paopao.jpg",
}
