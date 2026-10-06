use image::RgbImage;

fn hsv(r: u8, g: u8, b: u8) -> (u8, u8, u8) {
    let rf = r as f32 / 255.0;
    let gf = g as f32 / 255.0;
    let bf = b as f32 / 255.0;
    let mx = rf.max(gf).max(bf);
    let mn = rf.min(gf).min(bf);
    let d = mx - mn;
    let h = if d == 0.0 {
        0.0
    } else if mx == rf {
        let mut h = (gf - bf) / d % 6.0;
        if h < 0.0 {
            h += 6.0;
        }
        h * 60.0
    } else if mx == gf {
        (bf - rf) / d * 60.0 + 120.0
    } else {
        (rf - gf) / d * 60.0 + 240.0
    };
    let s = if mx == 0.0 { 0.0 } else { d / mx };
    (
        (h / 360.0 * 255.0) as u8,
        (s * 255.0) as u8,
        (mx * 255.0) as u8,
    )
}

fn main() {
    let path = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "./test/WoWScrnShot_100626_004009.jpg".to_string());
    let img: RgbImage = image::open(&path).unwrap().to_rgb8();
    let (w, h) = img.dimensions();
    println!("{} {}x{}", path, w, h);

    // sample a dense grid over the lure region (855-905, 165-215)
    let (x0, y0, x1, y1) = (850u32, 160u32, 910u32, 220u32);
    let mut blue = 0u32;
    let mut red = 0u32;
    let mut sat = 0u32;
    for y in y0..y1 {
        for x in x0..x1 {
            if x >= w || y >= h {
                continue;
            }
            let p = img.get_pixel(x, y);
            let (hh, ss, vv) = hsv(p[0], p[1], p[2]);
            let is_blue = (150..=195).contains(&hh) && ss > 40 && vv > 30;
            let is_red = (hh <= 15 || hh >= 240) && ss > 40 && vv > 30;
            if is_blue {
                blue += 1;
            }
            if is_red {
                red += 1;
            }
            if ss > 60 && vv > 40 {
                sat += 1;
            }
            // print only strongly colored pixels
            if (is_blue || is_red) && (x % 3 == 0) {
                println!(
                    "  ({},{}) RGB=({},{},{}) HSV=({},{},{}) {}",
                    x,
                    y,
                    p[0],
                    p[1],
                    p[2],
                    hh,
                    ss,
                    vv,
                    if is_blue {
                        "BLUE"
                    } else {
                        "RED"
                    }
                );
            }
        }
    }
    println!(
        "lure region: blue_px={} red_px={} saturated={}",
        blue, red, sat
    );
}
