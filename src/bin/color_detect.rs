use std::collections::VecDeque;

use image::{ImageBuffer, RgbImage};

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

#[derive(Clone)]
struct Cand {
    n: usize,
    blue: usize,
    red: usize,
    tan: usize,
    bal: f32,
    score: f32,
    cx: f32,
    cy: f32,
    minx: u32,
    miny: u32,
    maxx: u32,
    maxy: u32,
}

fn main() {
    let path = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "./test/WoWScrnShot_100626_004009.jpg".to_string());
    let min_n = std::env::args().nth(2).and_then(|a| a.parse().ok()).unwrap_or(25);
    let max_n = std::env::args().nth(3).and_then(|a| a.parse().ok()).unwrap_or(900);
    let max_dim = std::env::args()
        .nth(4)
        .and_then(|a| a.parse().ok())
        .unwrap_or(80);
    let min_blue = std::env::args()
        .nth(5)
        .and_then(|a| a.parse().ok())
        .unwrap_or(8);
    let min_red = std::env::args()
        .nth(6)
        .and_then(|a| a.parse().ok())
        .unwrap_or(8);
    let min_tan = std::env::args()
        .nth(7)
        .and_then(|a| a.parse().ok())
        .unwrap_or(8);
    let min_balance = std::env::args()
        .nth(8)
        .and_then(|a| a.parse().ok())
        .unwrap_or(0.30);
    let water_frac = std::env::args()
        .nth(9)
        .and_then(|a| a.parse().ok())
        .unwrap_or(0.45);

    let img: RgbImage = image::open(&path).unwrap().to_rgb8();
    let (w, h) = img.dimensions();
    let water_y_max = (h as f32 * water_frac) as u32;
    println!(
        "{} {}x{} (min_n={} max_n={} max_dim={} min_b={} min_r={} min_tan={})",
        path,
        w,
        h,
        min_n,
        max_n,
        max_dim,
        min_blue,
        min_red,
        min_tan
    );

    let mut mask: Vec<Class> = Vec::with_capacity((w * h) as usize);
    for p in img.pixels() {
        mask.push(classify(p[0], p[1], p[2]));
    }

    let mut visited = vec![false; (w * h) as usize];
    let mut cands: Vec<Cand> = Vec::new();
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
            if n < min_n || n > max_n || bw > max_dim || bh > max_dim {
                continue;
            }
            if blue < min_blue || red < min_red {
                continue;
            }
            // position prior: lure floats on the water, above the bottom-center character
            let cyf = sy / n as f32;
            if cyf > water_y_max as f32 {
                continue;
            }
            // balance: the lure shows BOTH feathers in similar amounts; armor is blue-dominated
            let bal = (blue.min(red)) as f32 / (blue.max(red)) as f32;
            if bal < min_balance {
                continue;
            }
            // count tan in an expanded window around the cluster (float is just below/around feathers)
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
            let score = (blue + red) as f32 * bal * (1.0 + (tan.min(300)) as f32 / 300.0 * 0.5);
            cands.push(Cand {
                n,
                blue,
                red,
                tan,
                bal,
                score,
                cx: sx / n as f32,
                cy: sy / n as f32,
                minx,
                miny,
                maxx,
                maxy,
            });
        }
    }

    let with_tan: Vec<&Cand> = cands.iter().filter(|c| c.tan >= min_tan).collect();
    let mut ranked = cands.clone();
    ranked.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));

    println!(
        "lure candidates (passed pos+balance+size)={}  of which with cork (>={})={}",
        ranked.len(),
        min_tan,
        with_tan.len()
    );
    for (i, c) in ranked.iter().take(8).enumerate() {
        println!(
            "  #{}: center=({:.0},{:.0}) score={:.0} blue={} red={} bal={:.2} tan={} n={} bbox=({},{})-({},{})",
            i + 1,
            c.cx,
            c.cy,
            c.score,
            c.blue,
            c.red,
            c.bal,
            c.tan,
            c.n,
            c.minx,
            c.miny,
            c.maxx,
            c.maxy
        );
        let pad = 18u32;
        let sx0 = c.cx.round() as u32 - pad;
        let sy0 = c.cy.round() as u32 - pad;
        let sw = 2 * pad + 16;
        let sh = 2 * pad + 16;
        let sx0 = sx0.min(w.saturating_sub(sw));
        let sy0 = sy0.min(h.saturating_sub(sh));
        let crop: RgbImage =
            ImageBuffer::from_fn(sw, sh, |x, y| *img.get_pixel(sx0 + x, sy0 + y));
        let fname = format!("test/cand{}_at_{}x{}.png", i + 1, c.cx as u32, c.cy as u32);
        crop.save(&fname).ok();
        println!("      crop -> {} (from {},{})", fname, sx0, sy0);
    }
}
