use image::{ImageBuffer, Rgb, RgbImage};
use rustautogui::RustAutoGui;
use wow_fishbot_rs::lure_detect::{find_lure_candidates, LureOptions};

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
    unsafe {
        // ALT-tap so SetForegroundWindow is allowed to take focus
        use winapi::um::winuser::{keybd_event, KEYEVENTF_KEYUP, VK_MENU};
        keybd_event(VK_MENU as u8, 0, 0, 0);
        keybd_event(VK_MENU as u8, 0, KEYEVENTF_KEYUP, 0);
        SetForegroundWindow(hwnd);
    }
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut gui = RustAutoGui::new(false)?;

    #[cfg(target_os = "windows")]
    find_and_activate_wow_window()?;

    std::thread::sleep(std::time::Duration::from_millis(1500));

    let (sw, sh) = gui.get_screen_size();
    let sw = sw as u32;
    let sh = sh as u32;
    let frame = gui.grab_screen_rgba((0, 0, sw, sh))?;
    let rgb: RgbImage = ImageBuffer::from_fn(frame.width(), frame.height(), |x, y| {
        let p = frame.get_pixel(x, y);
        Rgb([p[0], p[1], p[2]])
    });
    println!("captured {}x{}", sw, sh);

    let start = std::time::Instant::now();
    let cands = find_lure_candidates(&rgb, &LureOptions::default());
    println!("detection took {:?}, candidates={}", start.elapsed(), cands.len());

    for (i, c) in cands.iter().take(5).enumerate() {
        println!(
            "  #{}: ({},{}) score={:.0} blue={} red={} tan={} bal={:.2}",
            i + 1,
            c.x,
            c.y,
            c.score,
            c.blue,
            c.red,
            c.tan,
            c.balance
        );
    }

    frame.save("./test/live_frame.png")?;
    println!("saved ./test/live_frame.png");

    if let Some(c) = cands.first() {
        let pad = 40u32;
        let sx0 = c.x.saturating_sub(pad);
        let sy0 = c.y.saturating_sub(pad);
        let swc = 2 * pad;
        let shc = 2 * pad;
        let sx0 = sx0.min(rgb.width().saturating_sub(swc));
        let sy0 = sy0.min(rgb.height().saturating_sub(shc));
        let crop: RgbImage = ImageBuffer::from_fn(swc, shc, |x, y| *rgb.get_pixel(sx0 + x, sy0 + y));
        crop.save("./test/live_lure_crop.png")?;
        println!("saved ./test/live_lure_crop.png (around {},{})", c.x, c.y);
    }
    Ok(())
}
