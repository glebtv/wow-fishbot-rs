//! Color-based fishing-lure (bobber) detector.
//!
//! The lure is a small object with blue + red feathers on a tan/brown cork
//! float, floating on the water above the bottom-center character. It is
//! found by classifying pixels by color (HSV), grouping the blue/red pixels
//! into connected clusters, then filtering/ranking clusters by:
//!   * presence of BOTH blue and red (the two feathers),
//!   * blue/red balance (armor is usually blue-dominated),
//!   * small, compact size,
//!   * position in the upper "water" region (above the character),
//!   * a dark tan cork float nearby (boosts the score).
//!
//! This is far more discriminative for the lure than grayscale NCC template
//! matching, which tends to lock onto the character's armor and UI text.

use std::collections::VecDeque;

use image::RgbImage;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Class {
    None,
    Blue,
    Red,
    Tan,
}

fn hue_deg(r: f32, g: f32, b: f32) -> f32 {
    let mx = r.max(g).max(b);
    let mn = r.min(g).min(b);
    let d = mx - mn;
    if d == 0.0 {
        return 0.0;
    }
    if mx == r {
        let mut h = ((g - b) / d) % 6.0;
        if h < 0.0 {
            h += 6.0;
        }
        h * 60.0
    } else if mx == g {
        (b - r) / d * 60.0 + 120.0
    } else {
        (r - g) / d * 60.0 + 240.0
    }
}

fn classify(r: u8, g: u8, b: u8) -> Class {
    let rf = r as f32 / 255.0;
    let gf = g as f32 / 255.0;
    let bf = b as f32 / 255.0;
    let mx = rf.max(gf).max(bf);
    let mn = rf.min(gf).min(bf);
    let d = mx - mn;
    if mx < 0.10 {
        return Class::None;
    }
    let s = d / mx;
    let v = mx;
    let h = hue_deg(rf, gf, bf);
    // blue feathers
    if (200.0..=275.0).contains(&h) && s > 0.25 && v > 0.15 {
        return Class::Blue;
    }
    // red feathers
    if (h < 20.0 || h >= 340.0) && s > 0.30 && v > 0.20 {
        return Class::Red;
    }
    // tan / brown cork float (dark, muted — excludes bright gold armor)
    if (14.0..=48.0).contains(&h) && s > 0.12 && s < 0.62 && v > 0.18 && v < 0.50 {
        return Class::Tan;
    }
    Class::None
}

/// Tunables for [`find_lure`]. The defaults are validated against the
/// screenshots in `test/`.
#[derive(Clone, Copy, Debug)]
pub struct LureOptions {
    /// Minimum number of blue/red pixels in a cluster.
    pub min_n: usize,
    /// Maximum number of blue/red pixels in a cluster (bigger = character/UI).
    pub max_n: usize,
    /// Maximum cluster bounding-box width/height in pixels.
    pub max_dim: u32,
    /// Minimum blue feather pixels.
    pub min_blue: usize,
    /// Minimum red feather pixels.
    pub min_red: usize,
    /// Cork float pixels that give a full score boost.
    pub tan_full: usize,
    /// Minimum blue/red balance (min/max) to count as "both feathers".
    pub min_balance: f32,
    /// Fraction of screen height below which the lure is not searched
    /// (the character occupies the bottom-center).
    pub water_frac: f32,
}

impl Default for LureOptions {
    fn default() -> Self {
        Self {
            min_n: 25,
            max_n: 900,
            max_dim: 80,
            min_blue: 8,
            min_red: 8,
            tan_full: 300,
            min_balance: 0.30,
            water_frac: 0.45,
        }
    }
}

/// A detected lure candidate. `x`,`y` is the centroid of the blue/red feathers.
#[derive(Clone, Copy, Debug)]
pub struct LureMatch {
    pub x: u32,
    pub y: u32,
    pub score: f32,
    pub blue: usize,
    pub red: usize,
    pub tan: usize,
    pub balance: f32,
}

/// Runs the color detector and returns candidates sorted by score (best first).
pub fn find_lure_candidates(img: &RgbImage, opts: &LureOptions) -> Vec<LureMatch> {
    let (w, h) = img.dimensions();
    let water_y_max = (h as f32 * opts.water_frac) as f32;

    let mut mask: Vec<Class> = Vec::with_capacity((w * h) as usize);
    for p in img.pixels() {
        mask.push(classify(p[0], p[1], p[2]));
    }

    let mut visited = vec![false; (w * h) as usize];
    let mut cands: Vec<LureMatch> = Vec::new();
    let offsets: [(i32, i32); 8] = [
        (-1, -1),
        (0, -1),
        (1, -1),
        (-1, 0),
        (1, 0),
        (-1, 1),
        (0, 1),
        (1, 1),
    ];

    for y in 0..h {
        for x in 0..w {
            let i = (y * w + x) as usize;
            let c = mask[i];
            if c != Class::Blue && c != Class::Red {
                continue;
            }
            if visited[i] {
                continue;
            }
            let mut n = 0usize;
            let mut blue = 0usize;
            let mut red = 0usize;
            let mut sx = 0.0f32;
            let mut sy = 0.0f32;
            let mut minx = x;
            let mut miny = y;
            let mut maxx = x;
            let mut maxy = y;
            let mut queue: VecDeque<(u32, u32)> = VecDeque::new();
            queue.push_back((x, y));
            visited[i] = true;
            while let Some((cx, cy)) = queue.pop_front() {
                let ci = (cy * w + cx) as usize;
                let cc = mask[ci];
                if cc != Class::Blue && cc != Class::Red {
                    continue;
                }
                n += 1;
                if cc == Class::Blue {
                    blue += 1;
                } else {
                    red += 1;
                }
                sx += cx as f32;
                sy += cy as f32;
                minx = minx.min(cx);
                miny = miny.min(cy);
                maxx = maxx.max(cx);
                maxy = maxy.max(cy);
                for (odx, ody) in offsets {
                    let nx = cx as i32 + odx;
                    let ny = cy as i32 + ody;
                    if nx < 0 || ny < 0 || nx >= w as i32 || ny >= h as i32 {
                        continue;
                    }
                    let ni = (ny * w as i32 + nx) as usize;
                    if !visited[ni] && (mask[ni] == Class::Blue || mask[ni] == Class::Red) {
                        visited[ni] = true;
                        queue.push_back((nx as u32, ny as u32));
                    }
                }
            }
            // size + compactness filter
            let bw = maxx - minx + 1;
            let bh = maxy - miny + 1;
            if n < opts.min_n || n > opts.max_n || bw > opts.max_dim || bh > opts.max_dim {
                continue;
            }
            if blue < opts.min_blue || red < opts.min_red {
                continue;
            }
            // position prior: lure floats on the water, above the bottom-center character
            let cyf = sy / n as f32;
            if cyf > water_y_max {
                continue;
            }
            // balance: the lure shows BOTH feathers in similar amounts
            let bal = (blue.min(red)) as f32 / (blue.max(red)) as f32;
            if bal < opts.min_balance {
                continue;
            }
            // count tan (cork float) in an expanded window around the cluster
            let pad = 16u32;
            let wx0 = minx.saturating_sub(pad);
            let wy0 = miny.saturating_sub(pad);
            let wx1 = (maxx + pad + 1).min(w);
            let wy1 = (maxy + pad + 1).min(h);
            let mut tan = 0usize;
            for yy in wy0..wy1 {
                for xx in wx0..wx1 {
                    if mask[(yy * w + xx) as usize] == Class::Tan {
                        tan += 1;
                    }
                }
            }
            let score =
                (blue + red) as f32 * bal * (1.0 + (tan.min(opts.tan_full)) as f32 / opts.tan_full as f32 * 0.5);
            cands.push(LureMatch {
                x: (sx / n as f32).round() as u32,
                y: cyf.round() as u32,
                score,
                blue,
                red,
                tan,
                balance: bal,
            });
        }
    }

    cands.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
    cands
}

/// Returns the single best lure candidate, or `None` if nothing matched.
pub fn find_lure(img: &RgbImage, opts: &LureOptions) -> Option<LureMatch> {
    find_lure_candidates(img, opts).into_iter().next()
}
