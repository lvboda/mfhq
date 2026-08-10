use image::RgbImage;
use rayon::prelude::*;
use rustfft::{Fft, FftPlanner, num_complex::Complex};

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

fn fft2d(data: &mut [C], scratch: &mut [C], w: usize, h: usize, row: &dyn Fft<f32>, col: &dyn Fft<f32>) {
    row.process(data);
    transpose(data, scratch, w, h);
    col.process(scratch);
    transpose(scratch, data, h, w);
}

/// 窗口和的前缀和表，用于 O(1) 求任意窗口的 ΣI 或 ΣI²。
/// 平方和最大可达 1e11 量级，必须用 f64 累加，f32 的尾数不够。
struct Integral {
    sum: Vec<f64>,
    w: usize,
}

impl Integral {
    /// channel 为 Some(c) 时累加该通道原值，为 None 时累加三通道平方之和。
    fn new(raw: &[u8], sw: usize, sh: usize, channel: Option<usize>) -> Self {
        let w = sw + 1;
        let mut sum = vec![0.0f64; w * (sh + 1)];
        for y in 0..sh {
            let mut row = 0.0f64;
            let base = y * sw * 3;
            for x in 0..sw {
                row += match channel {
                    Some(c) => raw[base + x * 3 + c] as f64,
                    None => (0..3)
                        .map(|c| {
                            let v = raw[base + x * 3 + c] as f64;
                            v * v
                        })
                        .sum(),
                };
                sum[(y + 1) * w + x + 1] = sum[y * w + x + 1] + row;
            }
        }
        Integral { sum, w }
    }

    fn window(&self, x: usize, y: usize, tw: usize, th: usize) -> f64 {
        let a = y * self.w + x;
        let b = y * self.w + x + tw;
        let c = (y + th) * self.w + x;
        let d = (y + th) * self.w + x + tw;
        self.sum[d] - self.sum[b] - self.sum[c] + self.sum[a]
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

    let mut sum = [0.0f64; 3];
    let mut sum_sq = [0.0f64; 3];
    for i in 0..tw * th {
        for c in 0..3 {
            let v = tmpl_raw[i * 3 + c] as f64;
            sum[c] += v;
            sum_sq[c] += v * v;
        }
    }
    let mut tmpl_mean = [0.0f64; 3];
    let mut tmpl_norm = 0.0f64;
    for c in 0..3 {
        tmpl_mean[c] = sum[c] / area;
        tmpl_norm += sum_sq[c] - sum[c] * sum[c] / area;
    }
    if tmpl_norm <= 0.0 {
        return None;
    }

    let fw = next_fast_len(sw);
    let fh = next_fast_len(sh);
    let mut planner = FftPlanner::<f32>::new();
    let (row, col) = (planner.plan_fft_forward(fw), planner.plan_fft_forward(fh));
    let (row_inv, col_inv) = (planner.plan_fft_inverse(fw), planner.plan_fft_inverse(fh));

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
            fft2d(&mut si, &mut scratch, fw, fh, &*row, &*col);
            fft2d(&mut ti, &mut scratch, fw, fh, &*row, &*col);
            for i in 0..si.len() {
                si[i] *= ti[i];
            }
            fft2d(&mut si, &mut scratch, fw, fh, &*row_inv, &*col_inv);

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

    // 三张原值积分图各自算，平方和只需一张——它在通道间是线性相加的。
    let sums: Vec<Integral> = (0..3usize)
        .into_par_iter()
        .map(|c| Integral::new(scene_raw, sw, sh, Some(c)))
        .collect();
    let sq = Integral::new(scene_raw, sw, sh, None);

    let mut best = Match { score: -2.0, x: 0, y: 0 };
    for y in 0..out_h {
        for x in 0..out_w {
            let mut win_norm = sq.window(x, y, tw, th);
            for integral in &sums {
                let s = integral.window(x, y, tw, th);
                win_norm -= s * s / area;
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
