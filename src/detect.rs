use image::{ImageBuffer, Luma};
use rayon::prelude::*;
use rustfft::{num_complex::Complex, FftPlanner};

pub type GrayImage = ImageBuffer<Luma<u8>, Vec<u8>>;

pub struct NccResult {
    pub x: u32, // top-left x of the match
    pub y: u32, // top-left y of the match
    pub corr: f64,
}

fn integral_image(pixels: &[u64], width: u32, height: u32) -> Vec<u64> {
    let mut integral = vec![0u64; ((width + 1) * (height + 1)) as usize];
    for y in 0..height {
        let mut row_sum = 0u64;
        for x in 0..width {
            row_sum += pixels[(y * width + x) as usize];
            integral[((y + 1) * (width + 1) + x + 1) as usize] =
                integral[(y * (width + 1) + x + 1) as usize] + row_sum;
        }
    }
    integral
}

fn sum_region(integral: &[u64], x: u32, y: u32, w: u32, h: u32, width: u32) -> u64 {
    let stride = width + 1;
    integral[((y + h) * stride + x + w) as usize]
        - integral[(y * stride + x + w) as usize]
        - integral[((y + h) * stride + x) as usize]
        + integral[(y * stride + x) as usize]
}

/// FFT-based normalized cross-correlation (J.P. Lewis 1995).
/// Returns all positions with corr > threshold, sorted descending by corr.
pub fn ncc_matches(image: &GrayImage, template: &GrayImage, threshold: f64) -> Vec<NccResult> {
    let (iw, ih) = image.dimensions();
    let (tw, th) = template.dimensions();
    if iw < tw || ih < th || tw == 0 || th == 0 {
        return Vec::new();
    }

    let n = (tw * th) as f64;

    // zero-mean template + sum of squared deviations
    let tvec: Vec<f64> = template.pixels().map(|p| p[0] as f64).collect();
    let mean_t = tvec.iter().sum::<f64>() / n;
    let t_zero: Vec<f32> = tvec.iter().map(|v| (*v - mean_t) as f32).collect();
    let ssd_t: f64 = t_zero.iter().map(|v| (*v as f64) * (*v as f64)).sum();
    if ssd_t <= 0.0 {
        return Vec::new();
    }

    let ps = iw
        .next_power_of_two()
        .max(ih.next_power_of_two());
    let ps2 = (ps * ps) as usize;

    let mut img_padded = vec![Complex::new(0.0f32, 0.0f32); ps2];
    for y in 0..ih {
        for x in 0..iw {
            img_padded[(y * ps + x) as usize] =
                Complex::new(image.get_pixel(x, y)[0] as f32, 0.0);
        }
    }

    let mut tpl_padded = vec![Complex::new(0.0f32, 0.0f32); ps2];
    for y in 0..th {
        for x in 0..tw {
            tpl_padded[(y * ps + x) as usize] = Complex::new(t_zero[(y * tw + x) as usize], 0.0);
        }
    }

    let mut planner = FftPlanner::<f32>::new();
    let fft = planner.plan_fft_forward(ps2);
    let ifft = planner.plan_fft_inverse(ps2);

    fft.process(&mut img_padded);
    fft.process(&mut tpl_padded);

    let mut result = vec![Complex::new(0.0f32, 0.0f32); ps2];
    for i in 0..ps2 {
        result[i] = img_padded[i] * tpl_padded[i].conj();
    }
    ifft.process(&mut result);
    let norm = ps2 as f64;

    let flat: Vec<u64> = image.pixels().map(|p| p[0] as u64).collect();
    let integral = integral_image(&flat, iw, ih);
    let sq: Vec<u64> = flat.iter().map(|v| v * v).collect();
    let sq_integral = integral_image(&sq, iw, ih);

    let found: Vec<NccResult> = (0..=(ih - th))
        .into_par_iter()
        .map(|y| {
            (0..=(iw - tw))
                .map(|x| {
                    let sum_i = sum_region(&integral, x, y, tw, th, iw);
                    let sum_sq_i = sum_region(&sq_integral, x, y, tw, th, iw);
                    let ssd_i = sum_sq_i as f64 - (sum_i as f64).powi(2) / n;
                    if ssd_i <= 0.0 {
                        return None;
                    }
                    let denom = (ssd_i * ssd_t).sqrt();
                    if denom <= 0.0 {
                        return None;
                    }
                    let num = result[(y * ps + x) as usize].re as f64 / norm;
                    let corr = num / denom;
                    (corr > threshold).then_some(NccResult { x, y, corr })
                })
                .filter_map(|r| r)
                .collect::<Vec<NccResult>>()
        })
        .flatten()
        .collect();

    let mut found = found;
    found.sort_by(|a, b| b.corr.partial_cmp(&a.corr).unwrap_or(std::cmp::Ordering::Equal));
    found
}

/// NCC over multiple template scales (to cope with resolution / UI-scale differences).
/// Returns (best results list already filtered by threshold, scale of the best match, resized template dims).
pub fn ncc_multiscale(
    image: &GrayImage,
    template: &GrayImage,
    threshold: f64,
    scales: &[f32],
) -> (Vec<NccResult>, f32, (u32, u32)) {
    use image::imageops;
    use image::imageops::FilterType;

    let (tw, th) = template.dimensions();
    let mut best: Vec<NccResult> = Vec::new();
    let mut best_corr = f64::MIN;
    let mut best_scale = 1.0f32;
    let mut best_dims = (tw, th);

    for &s in scales {
        let new_w = ((tw as f32) * s).round().max(8.0) as u32;
        let new_h = ((th as f32) * s).round().max(8.0) as u32;
        let scaled = imageops::resize(template, new_w, new_h, FilterType::Lanczos3);
        let results = ncc_matches(image, &scaled, threshold);
        if let Some(top) = results.first() {
            if top.corr > best_corr {
                best_corr = top.corr;
                best_scale = s;
                best_dims = (new_w, new_h);
                best = results;
            }
        }
    }

    (best, best_scale, best_dims)
}
