#[cfg(target_os = "windows")]
fn main() {
    use winapi::um::winuser::{
        FindWindowW, GetForegroundWindow, GetWindowTextW, IsWindowVisible, ShowWindow, SW_RESTORE,
    };

    let title: Vec<u16> = "World of Warcraft"
        .encode_utf16()
        .chain(std::iter::once(0))
        .collect();
    let hwnd = unsafe { FindWindowW(std::ptr::null(), title.as_ptr()) };
    if hwnd.is_null() {
        println!("WoW window NOT found");
        return;
    }
    let visible = unsafe { IsWindowVisible(hwnd) };
    let fg = unsafe { GetForegroundWindow() };
    let mut buf = [0u16; 512];
    let n = unsafe { GetWindowTextW(hwnd, buf.as_mut_ptr(), buf.len() as i32) };
    let wtext = String::from_utf16_lossy(&buf[..n as usize]);
    unsafe {
        ShowWindow(hwnd, SW_RESTORE);
    }
    println!("WoW hwnd={:?}", hwnd);
    println!("  title={:?}", wtext);
    println!("  visible={} (1=yes)", visible);
    println!("  is_foreground={}", !fg.is_null() && fg == hwnd);
    println!("  foreground hwnd={:?}", fg);
}

#[cfg(not(target_os = "windows"))]
fn main() {
    println!("windows only");
}
