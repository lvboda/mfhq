use image::RgbImage;
use rayon::prelude::*;
use rustfft::{Fft, FftPlanner, num_complex::Complex};
use std::sync::Arc;

pub struct Match {
    pub score: f64,
    pub x: u32,
    pub y: u32,
}

type C = Complex<f32>;

fn next_fast_len(mut n: usize) -> usize {
    loop {
        let mut m = n;
        for f in [2usize, 3, 5] {
            while m % f == 0 {
                m /= f;
            }
        }
        if m == 1 {
            return n;
        }
        n += 1;
    }
}

fn transpose(src: &[C], dst: &mut [C], w: usize, h: usize) {
    for y in 0..h {
        for x in 0..w {
            dst[x * h + y] = src[y * w + x];
        }
    }
}

struct Plans {
    row: Arc<dyn Fft<f32>>,
    col: Arc<dyn Fft<f32>>,
    row_inv: Arc<dyn Fft<f32>>,
    col_inv: Arc<dyn Fft<f32>>,
}

fn fft2d(data: &mut [C], scratch: &mut [C], w: usize, h: usize, plans: &Plans, inverse: bool) {
    let (row, col) = if inverse {
        (&plans.row_inv, &plans.col_inv)
    } else {
        (&plans.row, &plans.col)
    };
    row.process(data);
    transpose(data, scratch, w, h);
    col.process(scratch);
    transpose(scratch, data, h, w);
}

/// 每通道的前缀和与平方前缀和，用于 O(1) 求任意窗口的 ΣI 与 ΣI²。
/// 平方和最大可达 1e11 量级，必须用 f64 累加，f32 的尾数不够。
struct Integral {
    sum: Vec<f64>,
    sq: Vec<f64>,
    w: usize,
}

impl Integral {
    fn new(raw: &[u8], sw: usize, sh: usize, channel: usize) -> Self {
        let w = sw + 1;
        let mut sum = vec![0.0f64; w * (sh + 1)];
        let mut sq = vec![0.0f64; w * (sh + 1)];
        for y in 0..sh {
            let mut row = 0.0f64;
            let mut row_sq = 0.0f64;
            let base = y * sw * 3 + channel;
            for x in 0..sw {
                let v = raw[base + x * 3] as f64;
                row += v;
                row_sq += v * v;
                sum[(y + 1) * w + x + 1] = sum[y * w + x + 1] + row;
                sq[(y + 1) * w + x + 1] = sq[y * w + x + 1] + row_sq;
            }
        }
        Integral { sum, sq, w }
    }

    fn window(&self, x: usize, y: usize, tw: usize, th: usize) -> (f64, f64) {
        let a = y * self.w + x;
        let b = y * self.w + x + tw;
        let c = (y + th) * self.w + x;
        let d = (y + th) * self.w + x + tw;
        (
            self.sum[d] - self.sum[b] - self.sum[c] + self.sum[a],
            self.sq[d] - self.sq[b] - self.sq[c] + self.sq[a],
        )
    }
}

/// 等价于 OpenCV 的 matchTemplate + TM_CCOEFF_NORMED。
///
/// 因为 Σ(T-meanT)=0，分子 Σ(I-meanI)(T-meanT) 化简为 Σ I·(T-meanT)，
/// 即场景与零均值模板的互相关，用 FFT 计算；分母的 Σ(I-meanI)² 用积分图 O(1) 求得。
pub fn match_template(scene: &RgbImage, tmpl: &RgbImage) -> Option<Match> {
    let (sw, sh) = (scene.width() as usize, scene.height() as usize);
    let (tw, th) = (tmpl.width() as usize, tmpl.height() as usize);
    if tw > sw || th > sh || tw == 0 || th == 0 {
        return None;
    }

    let scene_raw = scene.as_raw();
    let tmpl_raw = tmpl.as_raw();
    let area = (tw * th) as f64;

    let mut tmpl_mean = [0.0f64; 3];
    for i in 0..tw * th {
        for c in 0..3 {
            tmpl_mean[c] += tmpl_raw[i * 3 + c] as f64;
        }
    }
    for m in tmpl_mean.iter_mut() {
        *m /= area;
    }

    let mut tmpl_norm = 0.0f64;
    for i in 0..tw * th {
        for c in 0..3 {
            let d = tmpl_raw[i * 3 + c] as f64 - tmpl_mean[c];
            tmpl_norm += d * d;
        }
    }
    if tmpl_norm <= 0.0 {
        return None;
    }

    let fw = next_fast_len(sw);
    let fh = next_fast_len(sh);
    let mut planner = FftPlanner::<f32>::new();
    let plans = Plans {
        row: planner.plan_fft_forward(fw),
        col: planner.plan_fft_forward(fh),
        row_inv: planner.plan_fft_inverse(fw),
        col_inv: planner.plan_fft_inverse(fh),
    };

    let out_w = sw - tw + 1;
    let out_h = sh - th + 1;

    // 三个通道互不依赖，并行计算各自的互相关后相加。
    let per_channel: Vec<Vec<f32>> = (0..3usize)
        .into_par_iter()
        .map(|c| {
            let mut si = vec![C::new(0.0, 0.0); fw * fh];
            for y in 0..sh {
                let base = y * sw * 3 + c;
                for x in 0..sw {
                    si[y * fw + x] = C::new(scene_raw[base + x * 3] as f32, 0.0);
                }
            }
            // 卷积等价于与翻转模板的互相关，翻转后补零。
            let mut ti = vec![C::new(0.0, 0.0); fw * fh];
            for y in 0..th {
                for x in 0..tw {
                    let sx = tw - 1 - x;
                    let sy = th - 1 - y;
                    let v = tmpl_raw[(sy * tw + sx) * 3 + c] as f64 - tmpl_mean[c];
                    ti[y * fw + x] = C::new(v as f32, 0.0);
                }
            }

            let mut scratch = vec![C::new(0.0, 0.0); fw * fh];
            fft2d(&mut si, &mut scratch, fw, fh, &plans, false);
            fft2d(&mut ti, &mut scratch, fw, fh, &plans, false);
            for i in 0..si.len() {
                si[i] *= ti[i];
            }
            fft2d(&mut si, &mut scratch, fw, fh, &plans, true);

            let scale = (fw * fh) as f32;
            let mut out = vec![0.0f32; out_w * out_h];
            for y in 0..out_h {
                for x in 0..out_w {
                    out[y * out_w + x] = si[(y + th - 1) * fw + x + tw - 1].re / scale;
                }
            }
            out
        })
        .collect();

    let integrals: Vec<Integral> = (0..3usize)
        .into_par_iter()
        .map(|c| Integral::new(scene_raw, sw, sh, c))
        .collect();

    let mut best = Match { score: -2.0, x: 0, y: 0 };
    for y in 0..out_h {
        for x in 0..out_w {
            let mut win_norm = 0.0f64;
            for integral in &integrals {
                let (s, s2) = integral.window(x, y, tw, th);
                win_norm += s2 - s * s / area;
            }
            if win_norm <= 0.0 {
                continue;
            }
            let num: f64 = per_channel.iter().map(|ch| ch[y * out_w + x] as f64).sum();
            let score = num / (tmpl_norm * win_norm).sqrt();
            if score > best.score {
                best = Match { score, x: x as u32, y: y as u32 };
            }
        }
    }

    Some(best)
}
