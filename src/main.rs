use std::error::Error;
use std::fs;
use std::path::PathBuf;
use std::thread::sleep;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use image::{ImageBuffer, Luma, Rgb, RgbImage, RgbaImage};
use rand::Rng;
use rustautogui::{MouseClick, RustAutoGui};
use wow_fishbot_rs::lure_detect::{find_lure, LureOptions};

const SPLASH_PIXEL_THRESHOLD: u8 = 50;
const SPLASH_MIN_CHANGED_PIXELS: u32 = 250;
/// Square region around the lure used to watch for a fish splash.
const SPLASH_REGION_SIZE: u32 = 120;
/// Directory where full-screen shots are saved when the lure is not detected.
const FAILURES_DIR: &str = "failures";

/// The game shows a 17s "no bite" timer; add 1s of headroom for lag.
const SPLASH_TIMEOUT: Duration = Duration::from_secs(18);

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

/// Introduces a random delay to make the bot behavior less predictable
/// This might helps avoid detection by anti-cheat systems
fn random_delay(min_delay: u64, max_delay: u64) {
    let delay_ms = rand::thread_rng().gen_range(min_delay..=max_delay);
    sleep(Duration::from_millis(delay_ms));
}

/// Counts pixels that changed more than the threshold between two grayscale frames
fn count_changed_pixels(
    prev: &ImageBuffer<Luma<u8>, Vec<u8>>,
    current: &ImageBuffer<Luma<u8>, Vec<u8>>,
    threshold: u8,
) -> u32 {
    prev.pixels()
        .zip(current.pixels())
        .filter(|(a, b)| a[0].abs_diff(b[0]) > threshold)
        .count() as u32
}

/// Continuously monitors for a fish splash within the specified timeout period
fn wait_for_splash(
    gui: &mut RustAutoGui,
    region: (u32, u32, u32, u32),
    timeout: Duration,
) -> Result<bool, Box<dyn Error>> {
    let mut prev_frame = gui.grab_screen_grayscale(region)?;
    let start_time = Instant::now();

    // Keep checking for splashes until timeout
    while start_time.elapsed() < timeout {
        let current_frame = gui.grab_screen_grayscale(region)?;
        let changed = count_changed_pixels(&prev_frame, &current_frame, SPLASH_PIXEL_THRESHOLD);

        if changed > SPLASH_MIN_CHANGED_PIXELS {
            println!("Splash detected! ({} changed pixels)", changed);
            return Ok(true);
        }

        prev_frame = current_frame;
        sleep(Duration::from_millis(50));
    }

    Ok(false) // Timeout occurred without detecting a splash
}

/// Parses an optional `--runs N` / `--runs=N` CLI flag limiting how many cast
/// cycles the bot performs before stopping. `None` means run until interrupted.
fn parse_max_runs() -> Option<usize> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let mut i = 0;
    while i < args.len() {
        if args[i] == "--runs" {
            if let Some(v) = args.get(i + 1) {
                return v.parse().ok();
            }
            return None;
        }
        if let Some(rest) = args[i].strip_prefix("--runs=") {
            return rest.parse().ok();
        }
        i += 1;
    }
    None
}

/// Saves a full-screen frame to the failures directory for later inspection.
fn save_failure_shot(frame: &RgbaImage, seq: u32) -> Result<PathBuf, Box<dyn Error>> {
    fs::create_dir_all(FAILURES_DIR)?;
    let t = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
    let path = PathBuf::from(format!("{}/lure_miss_{:04}_t{}.png", FAILURES_DIR, seq, t));
    frame.save(&path)?;
    println!("Saved failure shot -> {}", path.display());
    Ok(path)
}

/// Captures the screen and returns the current lure position, if found.
fn locate_lure(
    gui: &mut RustAutoGui,
    w: u32,
    h: u32,
    opts: &LureOptions,
) -> Result<Option<(u32, u32)>, Box<dyn Error>> {
    let frame = gui.grab_screen_rgba((0, 0, w, h))?;
    let rgb: RgbImage = ImageBuffer::from_fn(frame.width(), frame.height(), |x, y| {
        let p = frame.get_pixel(x, y);
        Rgb([p[0], p[1], p[2]])
    });
    Ok(find_lure(&rgb, opts).map(|l| (l.x, l.y)))
}

fn main() -> Result<(), Box<dyn Error>> {
    let max_runs = parse_max_runs();
    let mut gui = RustAutoGui::new(false)?;

    #[cfg(target_os = "windows")]
    find_and_activate_wow_window()
        .expect("Could not find World of Warcraft window");

    let (screen_w, screen_h) = gui.get_screen_size();
    let screen_w = screen_w as u32;
    let screen_h = screen_h as u32;
    let opts = LureOptions::default();
    println!("Screen {}x{}, lure detector ready", screen_w, screen_h);
    match max_runs {
        Some(n) => println!("Will stop after {} run(s)", n),
        None => println!("Running until interrupted (Ctrl+C)"),
    }

    sleep(Duration::from_millis(1000));
    println!("Starting!");

    let mut run: u32 = 0;
    let mut misses: u32 = 0;

    loop {
        if let Some(n) = max_runs {
            if (run as usize) >= n {
                println!("Completed {} run(s), stopping.", n);
                break;
            }
        }
        run += 1;
        println!("--- Run {} ---", run);

        // Simulates pressing the fishing key (F4) to cast the fishing line
        println!("Cast fishing...");
        gui.key_down("f4")?;
        random_delay(150, 350);
        gui.key_up("f4")?;

        random_delay(1800, 2200);

        // Capture the screen and find the lure with the color detector
        let frame = gui.grab_screen_rgba((0, 0, screen_w, screen_h))?;
        let rgb: RgbImage = ImageBuffer::from_fn(frame.width(), frame.height(), |x, y| {
            let p = frame.get_pixel(x, y);
            Rgb([p[0], p[1], p[2]])
        });
        let Some(lure) = find_lure(&rgb, &opts) else {
            misses += 1;
            println!("Lure not detected");
            let _ = save_failure_shot(&frame, misses);
            random_delay(684, 4833);
            continue;
        };
        println!(
            "Lure detected at ({}, {}), score {:.0} (blue {} / red {} / cork {})",
            lure.x, lure.y, lure.score, lure.blue, lure.red, lure.tan
        );

        // Move the mouse cursor to the lure (instant move)
        gui.move_mouse_to_pos(lure.x, lure.y, 0.0)?;

        random_delay(400, 600); // wait so the splash detector is not disturbed by the moving cursor

        let half = SPLASH_REGION_SIZE / 2;
        let region = (
            lure.x.saturating_sub(half),
            lure.y.saturating_sub(half),
            SPLASH_REGION_SIZE,
            SPLASH_REGION_SIZE,
        );

        let splash_detected = wait_for_splash(&mut gui, region, SPLASH_TIMEOUT)?;

        if splash_detected {
            // The bobber may have drifted since the initial detection, so re-locate it
            // and click exactly where it is now. Fall back to the original position if
            // the splash changed the scene enough that detection fails.
            let (cx, cy) = locate_lure(&mut gui, screen_w, screen_h, &opts)?
                .unwrap_or((lure.x, lure.y));
            if (cx, cy) != (lure.x, lure.y) {
                println!("Bobber re-located at ({}, {}) for the click", cx, cy);
            }
            gui.move_mouse_to_pos(cx, cy, 0.0)?;
            random_delay(50, 189);
            gui.click_down(MouseClick::RIGHT)?;
            random_delay(184, 483);
            gui.click_up(MouseClick::RIGHT)?;
        } else {
            println!("Timeout while waiting for splash");
        }

        random_delay(684, 4833);
    }

    Ok(())
}
