use image::RgbImage;
use rustfft::{FftPlanner, num_complex::Complex};

pub struct Match {
    pub score: f64,
    pub x: u32,
    pub y: u32,
}

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

fn transpose(src: &[Complex<f64>], dst: &mut [Complex<f64>], w: usize, h: usize) {
    for y in 0..h {
        for x in 0..w {
            dst[x * h + y] = src[y * w + x];
        }
    }
}

fn fft2d(data: &mut Vec<Complex<f64>>, w: usize, h: usize, planner: &mut FftPlanner<f64>, inverse: bool) {
    let row_fft = if inverse {
        planner.plan_fft_inverse(w)
    } else {
        planner.plan_fft_forward(w)
    };
    row_fft.process(data);

    let mut t = vec![Complex::new(0.0, 0.0); w * h];
    transpose(data, &mut t, w, h);

    let col_fft = if inverse {
        planner.plan_fft_inverse(h)
    } else {
        planner.plan_fft_forward(h)
    };
    col_fft.process(&mut t);

    transpose(&t, data, h, w);
}

/// 每通道的前缀和与平方前缀和，用于 O(1) 求任意窗口的 ΣI 与 ΣI²。
struct Integral {
    sum: Vec<f64>,
    sq: Vec<f64>,
    w: usize,
}

impl Integral {
    fn new(scene: &RgbImage, channel: usize) -> Self {
        let (sw, sh) = (scene.width() as usize, scene.height() as usize);
        let w = sw + 1;
        let mut sum = vec![0.0f64; w * (sh + 1)];
        let mut sq = vec![0.0f64; w * (sh + 1)];
        for y in 0..sh {
            let mut row = 0.0f64;
            let mut row_sq = 0.0f64;
            for x in 0..sw {
                let v = scene.get_pixel(x as u32, y as u32)[channel] as f64;
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

    let mut tmpl_norm = 0.0f64;
    for px in tmpl.pixels() {
        for c in 0..3 {
            let d = px[c] as f64 - tmpl_mean[c];
            tmpl_norm += d * d;
        }
    }
    if tmpl_norm <= 0.0 {
        return None;
    }

    let fw = next_fast_len(sw);
    let fh = next_fast_len(sh);
    let mut planner = FftPlanner::<f64>::new();

    let out_w = sw - tw + 1;
    let out_h = sh - th + 1;
    let mut num = vec![0.0f64; out_w * out_h];

    for c in 0..3 {
        let mut si = vec![Complex::new(0.0, 0.0); fw * fh];
        for y in 0..sh {
            for x in 0..sw {
                si[y * fw + x] = Complex::new(scene.get_pixel(x as u32, y as u32)[c] as f64, 0.0);
            }
        }
        // 卷积等价于与翻转模板的互相关，翻转后补零。
        let mut ti = vec![Complex::new(0.0, 0.0); fw * fh];
        for y in 0..th {
            for x in 0..tw {
                let v = tmpl.get_pixel((tw - 1 - x) as u32, (th - 1 - y) as u32)[c] as f64 - tmpl_mean[c];
                ti[y * fw + x] = Complex::new(v, 0.0);
            }
        }

        fft2d(&mut si, fw, fh, &mut planner, false);
        fft2d(&mut ti, fw, fh, &mut planner, false);
        for i in 0..si.len() {
            si[i] *= ti[i];
        }
        fft2d(&mut si, fw, fh, &mut planner, true);

        let scale = (fw * fh) as f64;
        for y in 0..out_h {
            for x in 0..out_w {
                num[y * out_w + x] += si[(y + th - 1) * fw + x + tw - 1].re / scale;
            }
        }
    }

    let integrals: Vec<Integral> = (0..3).map(|c| Integral::new(scene, c)).collect();

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
            let score = num[y * out_w + x] / (tmpl_norm * win_norm).sqrt();
            if score > best.score {
                best = Match { score, x: x as u32, y: y as u32 };
            }
        }
    }

    Some(best)
}
