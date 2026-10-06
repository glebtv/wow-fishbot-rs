use std::collections::VecDeque;

use image::{ImageBuffer, Luma, Rgb, RgbImage};

fn is_bobber(p: &Rgb<u8>, loose: bool) -> bool {
    let r = p[0] as i32;
    let g = p[1] as i32;
    let b = p[2] as i32;
    let luma = 0.2126 * r as f32 + 0.7152 * g as f32 + 0.0722 * b as f32;
    if loose {
        r > 120 && (r - b) > 40 && luma > 70.0
    } else {
        r > 150 && (r - b) > 60 && luma > 100.0
    }
}

fn main() {
    let path: std::path::PathBuf = std::env::args()
        .nth(1)
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| std::path::PathBuf::from("./test/WoWScrnShot_100626_004009.jpg"));
    let loose = std::env::args().any(|a| a == "--loose");
    let img: RgbImage = image::open(&path).unwrap().to_rgb8();
    let (w, h) = img.dimensions();
    println!(
        "{} {}x{} (loose={})",
        path.to_string_lossy(),
        w,
        h,
        loose
    );

    // mask of bobber-like pixels
    let mask: Vec<bool> = img.pixels().map(|p| is_bobber(p, loose)).collect();

    // connected components (4-neighbor)
    let mut visited = vec![false; (w * h) as usize];
    let mut clusters: Vec<Vec<(u32, u32)>> = Vec::new();
    for y in 0..h {
        for x in 0..w {
            let i = (y * w + x) as usize;
            if mask[i] && !visited[i] {
                let mut comp = Vec::new();
                let mut queue = VecDeque::new();
                queue.push_back((x, y));
                visited[i] = true;
                while let Some((cx, cy)) = queue.pop_front() {
                    comp.push((cx, cy));
                    for (nx, ny) in [
                        (cx.saturating_add(1), cy),
                        (cx.saturating_sub(1), cy),
                        (cx, cy.saturating_add(1)),
                        (cx, cy.saturating_sub(1)),
                    ] {
                        if nx >= w || ny >= h {
                            continue;
                        }
                        let ni = (ny * w + nx) as usize;
                        if mask[ni] && !visited[ni] {
                            visited[ni] = true;
                            queue.push_back((nx, ny));
                        }
                    }
                }
                if comp.len() >= 9 {
                    clusters.push(comp);
                }
            }
        }
    }

    clusters.sort_by(|a, b| b.len().cmp(&a.len()));
    println!("clusters (>=9 px): {}", clusters.len());
    for (ci, comp) in clusters.iter().take(12).enumerate() {
        let mut minx = u32::MAX;
        let mut miny = u32::MAX;
        let mut maxx = 0u32;
        let mut maxy = 0u32;
        let mut r = 0.0;
        let mut g = 0.0;
        let mut b = 0.0;
        for (x, y) in comp {
            minx = minx.min(*x);
            miny = miny.min(*y);
            maxx = maxx.max(*x);
            maxy = maxy.max(*y);
            let p = img.get_pixel(*x, *y);
            r += p[0] as f32;
            g += p[1] as f32;
            b += p[2] as f32;
        }
        let n = comp.len() as f32;
        println!(
            "  #{}: {} px, bbox ({},{})-({},{}) size {}x{}, avg RGB=({:.0},{:.0},{:.0})",
            ci + 1,
            comp.len(),
            minx,
            miny,
            maxx,
            maxy,
            maxx - minx + 1,
            maxy - miny + 1,
            r / n,
            g / n,
            b / n
        );

        // save a padded crop of the top 3 clusters
        if ci < 3 {
            let pad = 12u32;
            let cx0 = minx.saturating_sub(pad);
            let cy0 = miny.saturating_sub(pad);
            let cw = (maxx - minx + 1 + pad * 2).min(w - cx0);
            let ch = (maxy - miny + 1 + pad * 2).min(h - cy0);
            let gray_crop = ImageBuffer::from_fn(cw, ch, |x, y| {
                let p = img.get_pixel(cx0 + x, cy0 + y);
                Luma([(0.2126 * p[0] as f32 + 0.7152 * p[1] as f32 + 0.0722 * p[2] as f32) as u8])
            });
            let name = format!("test/blob{}_gray_{}.png", ci + 1, path.file_name().unwrap().to_string_lossy());
            gray_crop.save(&name).ok();
            let name2 = format!("test/blob{}_rgb_{}.png", ci + 1, path.file_name().unwrap().to_string_lossy());
            img.save(&name2).ok();
            // also save just the blob crop in color
            let color_crop: RgbImage =
                ImageBuffer::from_fn(cw, ch, |x, y| *img.get_pixel(cx0 + x, cy0 + y));
            color_crop
                .save(format!(
                    "test/blob{}_crop_{}.png",
                    ci + 1,
                    path.file_name().unwrap().to_string_lossy()
                ))
                .ok();
        }
    }
    println!("crops saved to ./test/blobN_*");
}
