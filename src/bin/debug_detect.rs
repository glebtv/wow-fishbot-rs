use std::env;
use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};
use std::thread::sleep;
use std::time::Duration;

use image::{GenericImageView, ImageBuffer, Luma};
use rustautogui::{MatchMode, RustAutoGui};

const TEMPLATES_DIR: &str = "./template";

#[cfg(target_os = "windows")]
fn find_and_activate_wow_window() -> Result<(), String> {
    use winapi::um::winuser::{FindWindowW, SetForegroundWindow};

    let title: Vec<u16> = "World of Warcraft"
        .encode_utf16()
        .chain(std::iter::once(0))
        .collect();
    let hwnd = unsafe { FindWindowW(std::ptr::null(), title.as_ptr()) };
    if hwnd.is_null() {
        return Err("Could not find World of Warcraft window".to_string());
    }
    unsafe { SetForegroundWindow(hwnd) };
    Ok(())
}

fn ascii_render(img: &ImageBuffer<Luma<u8>, Vec<u8>>, w: u32, h: u32) -> String {
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

fn list_templates() -> Vec<PathBuf> {
    let mut templates = fs::read_dir(Path::new(TEMPLATES_DIR))
        .expect("cannot read template dir")
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| {
            p.extension()
                .and_then(|e| e.to_str())
                .map(|e| e.eq_ignore_ascii_case("png"))
                .unwrap_or(false)
        })
        .collect::<Vec<_>>();
    templates.sort();
    templates
}

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();
    let cast = !args.iter().any(|a| a == "--no-cast");

    let mut gui = RustAutoGui::new(true)?;

    #[cfg(target_os = "windows")]
    match find_and_activate_wow_window() {
        Ok(()) => println!("WoW window found and activated"),
        Err(e) => println!("WARNING: {}", e),
    }

    if cast {
        println!("Casting (F4)...");
        gui.key_down("f4")?;
        sleep(Duration::from_millis(250));
        gui.key_up("f4")?;
        println!("Waiting 3s for the bobber...");
        sleep(Duration::from_millis(3000));
    }

    let (sw, sh) = gui.get_screen_size();
    let sw = sw as u32;
    let sh = sh as u32;
    println!("Screen: {}x{}", sw, sh);

    let shot = gui.grab_screen_grayscale((0, 0, sw, sh))?;
    shot.save("debug_screen.png")?;
    println!("Saved full screen capture to debug_screen.png");

    println!("\nFull screen (ASCII, downscaled):");
    println!("{}", ascii_render(&shot, 120, 60));

    for t in list_templates() {
        let name = t.file_name().unwrap().to_string_lossy().to_string();
        let timg = image::open(&t)?;
        let (tw, th) = timg.dimensions();
        let tpl_gray = timg.to_luma8();

        println!("\n=== Template: {} ({}x{}) ===", name, tw, th);
        println!(
            "{}",
            ascii_render(&tpl_gray, tw.min(80), (th as f32 * 80.0 / tw as f32) as u32).trim_end()
        );

        gui.prepare_template_from_file(t.to_str().unwrap(), None, MatchMode::FFT)?;
        let found = gui.find_image_on_screen(0.0)?.unwrap_or_default();
        if found.is_empty() {
            println!("  No matches with corr > 0.0");
            continue;
        }
        println!(
            "  {} matches above 0.0; top matches (center x, y, corr):",
            found.len()
        );
        for (i, (x, y, c)) in found.iter().take(10).enumerate() {
            println!("   {}. ({}, {}) corr={:.4}", i + 1, x, y, c);
        }

        // save + render the best match crop from the screen capture
        let (cx, cy, _) = found[0];
        let top_x = cx.saturating_sub(tw / 2);
        let top_y = cy.saturating_sub(th / 2);
        let crop_view = image::imageops::crop_imm(&shot, top_x, top_y, tw, th);
        let crop = ImageBuffer::from_fn(tw, th, |x, y| crop_view.get_pixel(x, y));
        let crop_name = format!("debug_best_{}", name);
        crop.save(&crop_name)?;
        println!("  Best match crop saved to {}", crop_name);
        println!("{}", ascii_render(&crop, tw.min(80), th.min(40)).trim_end());
    }

    Ok(())
}
