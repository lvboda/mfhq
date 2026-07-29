use image::RgbImage;

pub struct Match {
    pub score: f64,
    pub x: u32,
    pub y: u32,
}

/// 等价于 OpenCV 的 matchTemplate + TM_CCOEFF_NORMED，逐通道减均值后做归一化互相关。
pub fn match_template(scene: &RgbImage, tmpl: &RgbImage) -> Option<Match> {
    let (sw, sh) = (scene.width() as usize, scene.height() as usize);
    let (tw, th) = (tmpl.width() as usize, tmpl.height() as usize);
    if tw > sw || th > sh || tw == 0 || th == 0 {
        return None;
    }

    let area = (tw * th) as f64;
    let mut tmpl_mean = [0.0f64; 3];
    for px in tmpl.pixels() {
        for c in 0..3 {
            tmpl_mean[c] += px[c] as f64;
        }
    }
    for m in tmpl_mean.iter_mut() {
        *m /= area;
    }

    let mut tmpl_dev = vec![0.0f64; tw * th * 3];
    let mut tmpl_norm = 0.0f64;
    for ty in 0..th {
        for tx in 0..tw {
            let px = tmpl.get_pixel(tx as u32, ty as u32);
            for c in 0..3 {
                let d = px[c] as f64 - tmpl_mean[c];
                tmpl_dev[(ty * tw + tx) * 3 + c] = d;
                tmpl_norm += d * d;
            }
        }
    }
    if tmpl_norm <= 0.0 {
        return None;
    }

    let mut best = Match { score: -2.0, x: 0, y: 0 };
    for y in 0..=(sh - th) {
        for x in 0..=(sw - tw) {
            let mut win_mean = [0.0f64; 3];
            for wy in 0..th {
                for wx in 0..tw {
                    let px = scene.get_pixel((x + wx) as u32, (y + wy) as u32);
                    for c in 0..3 {
                        win_mean[c] += px[c] as f64;
                    }
                }
            }
            for m in win_mean.iter_mut() {
                *m /= area;
            }

            let mut num = 0.0f64;
            let mut win_norm = 0.0f64;
            for wy in 0..th {
                for wx in 0..tw {
                    let px = scene.get_pixel((x + wx) as u32, (y + wy) as u32);
                    for c in 0..3 {
                        let d = px[c] as f64 - win_mean[c];
                        win_norm += d * d;
                        num += d * tmpl_dev[(wy * tw + wx) * 3 + c];
                    }
                }
            }
            if win_norm <= 0.0 {
                continue;
            }

            let score = num / (tmpl_norm * win_norm).sqrt();
            if score > best.score {
                best = Match { score, x: x as u32, y: y as u32 };
            }
        }
    }

    Some(best)
}
