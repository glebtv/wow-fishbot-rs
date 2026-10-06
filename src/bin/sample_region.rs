use image::RgbImage;

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

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let path = args.get(1).cloned().unwrap_or_default();
    let x0: u32 = args.get(2).and_then(|a| a.parse().ok()).unwrap_or(0);
    let y0: u32 = args.get(3).and_then(|a| a.parse().ok()).unwrap_or(0);
    let mut x1: u32 = args.get(4).and_then(|a| a.parse().ok()).unwrap_or(0);
    let mut y1: u32 = args.get(5).and_then(|a| a.parse().ok()).unwrap_or(0);
    let img: RgbImage = image::open(&path).unwrap().to_rgb8();
    let (w, h) = img.dimensions();
    x1 = x1.min(w);
    y1 = y1.min(h);
    println!("{} region ({},{})-({},{})", path, x0, y0, x1, y1);
    let mut cnt = 0u32;
    let mut hmin = 360.0f32;
    let mut hmax = 0.0f32;
    let mut smin = 1.0f32;
    let mut smax = 0.0f32;
    let mut vmin = 1.0f32;
    let mut vmax = 0.0f32;
    for y in y0..y1 {
        for x in x0..x1 {
            let p = img.get_pixel(x, y);
            let rf = p[0] as f32 / 255.0;
            let gf = p[1] as f32 / 255.0;
            let bf = p[2] as f32 / 255.0;
            let mx = rf.max(gf).max(bf);
            let mn = rf.min(gf).min(bf);
            let d = mx - mn;
            if mx < 0.15 || d < 0.08 {
                continue;
            }
            let h = hue_deg(rf, gf, bf);
            let s = d / mx;
            // only warm colors (brown/gold/orange), exclude purple/blue/green
            if !(15.0..=70.0).contains(&h) {
                continue;
            }
            cnt += 1;
            hmin = hmin.min(h);
            hmax = hmax.max(h);
            smin = smin.min(s);
            smax = smax.max(s);
            vmin = vmin.min(mx);
            vmax = vmax.max(mx);
            if x % 2 == 0 && y % 2 == 0 {
                println!(
                    "  ({},{}) RGB=({},{},{}) H={:.0} S={:.2} V={:.2}",
                    x,
                    y,
                    p[0],
                    p[1],
                    p[2],
                    h,
                    s,
                    mx
                );
            }
        }
    }
    if cnt > 0 {
        println!(
            "warm px={} H:[{:.0},{:.0}] S:[{:.2},{:.2}] V:[{:.2},{:.2}]",
            cnt, hmin, hmax, smin, smax, vmin, vmax
        );
    } else {
        println!("no warm pixels");
    }
}
