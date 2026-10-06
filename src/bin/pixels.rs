use image::RgbImage;

fn main() {
    let path = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "./test/WoWScrnShot_100626_002451.jpg".to_string());
    let img: RgbImage = image::open(&path).unwrap().to_rgb8();
    let (w, h) = img.dimensions();
    println!("{} {}x{}", path, w, h);

    let pts: [(u32, u32); 16] = [
        (100, 300),
        (400, 300),
        (720, 300),
        (1000, 300),
        (1200, 300),
        (1350, 300),
        (1200, 450),
        (1350, 450),
        (1000, 450),
        (800, 450),
        (1200, 600),
        (1350, 600),
        (1300, 250),
        (720, 450),
        (720, 700),
        (1200, 750),
    ];
    for (x, y) in pts {
        if x < w && y < h {
            let p = img.get_pixel(x, y);
            let luma = (0.2126 * p[0] as f32 + 0.7152 * p[1] as f32 + 0.0722 * p[2] as f32) as u32;
            println!("  ({:>4},{:>4}) RGB=({:>3},{:>3},{:>3}) luma={:>3}", x, y, p[0], p[1], p[2], luma);
        }
    }
}
