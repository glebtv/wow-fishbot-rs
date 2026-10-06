use std::fs;
use std::path::{Path, PathBuf};

use image::ImageBuffer;
use wow_fishbot_rs::detect::{self, GrayImage};

fn ascii_render(img: &GrayImage, w: u32, h: u32) -> String {
    let (iw, ih) = img.dimensions();
    if iw == 0 || ih == 0 {
        return String::new();
    }
    let chars = " .:-=+*#%@";
    let mut out = String::new();
    for gy in 0..h {
        for gx in 0..w {
            let x = (gx as f32 * iw as f32 / w as f32).min(iw as f32 - 1.0) as u32;
            let y = (gy as f32 * ih as f32 / h as f32).min(ih as f32 - 1.0) as u32;
            let v = img.get_pixel(x, y)[0] as f32 / 255.0;
            let idx = ((1.0 - v) * (chars.len() - 1) as f32) as usize;
            out.push(chars.as_bytes()[idx] as char);
        }
        out.push('\n');
    }
    out
}

fn list_png(dir: &str) -> Vec<PathBuf> {
    let mut v = fs::read_dir(dir)
        .expect("cannot read dir")
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| {
            p.extension()
                .and_then(|e| e.to_str())
                .map(|e| e.eq_ignore_ascii_case("png"))
                .unwrap_or(false)
        })
        .collect::<Vec<_>>();
    v.sort();
    v
}

fn list_jpg(dir: &str) -> Vec<PathBuf> {
    let mut v = fs::read_dir(dir)
        .expect("cannot read dir")
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| {
            p.extension()
                .and_then(|e| e.to_str())
                .map(|e| e.eq_ignore_ascii_case("jpg") || e.eq_ignore_ascii_case("jpeg"))
                .unwrap_or(false)
        })
        .collect::<Vec<_>>();
    v.sort();
    v
}

fn to_gray(path: &Path) -> GrayImage {
    let img = image::open(path).expect("cannot open image");
    img.to_luma8()
}

fn main() {
    let screenshots = list_jpg("./test");
    let templates = list_png("./template");
    if screenshots.is_empty() {
        panic!("no jpg screenshots in ./test");
    }
    if templates.is_empty() {
        panic!("no png templates in ./template");
    }

    let scales: Vec<f32> = (5..=15)
        .map(|i| i as f32 / 10.0)
        .collect();

    for shot_path in &screenshots {
        let name = shot_path.file_name().unwrap().to_string_lossy().to_string();
        let shot = to_gray(shot_path);
        let (sw, sh) = shot.dimensions();
        println!("\n########## Screenshot: {} ({}x{}) ##########", name, sw, sh);
        println!("{}", ascii_render(&shot, 144, 66));

        for t_path in &templates {
            let tname = t_path.file_name().unwrap().to_string_lossy().to_string();
            let tpl = to_gray(t_path);
            let (tw, th) = tpl.dimensions();

            println!("\n--- Template: {} ({}x{}) ---", tname, tw, th);

            // single-scale
            let single = detect::ncc_matches(&shot, &tpl, 0.0);
            if let Some(top) = single.first() {
                println!(
                    "  NCC      best: ({}, {}) center=({},{}) corr={:.4}",
                    top.x,
                    top.y,
                    top.x + tw / 2,
                    top.y + th / 2,
                    top.corr
                );
            } else {
                println!("  NCC      no matches");
            }

            // multi-scale
            let (multi, mscale, mdims) = detect::ncc_multiscale(&shot, &tpl, 0.0, &scales);
            if let Some(top) = multi.first() {
                println!(
                    "  NCC-MS   best: ({}, {}) center=({},{}) scale={:.2} dims={}x{} corr={:.4}",
                    top.x,
                    top.y,
                    top.x + mdims.0 / 2,
                    top.y + mdims.1 / 2,
                    mscale,
                    mdims.0,
                    mdims.1,
                    top.corr
                );
            } else {
                println!("  NCC-MS   no matches");
            }

            // save best single-scale crop
            if let Some(top) = single.first() {
                let crop = ImageBuffer::from_fn(tw, th, |x, y| {
                    *shot.get_pixel(top.x + x, top.y + y)
                });
                let crop_name = format!("test/best_{}_{}", tname, name);
                crop.save(&crop_name).ok();
            }
        }
    }

    println!("\nDone. Best-match crops saved in ./test/ as best_<template>_<screenshot>");
}
