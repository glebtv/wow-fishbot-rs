use image::ImageBuffer;
use wow_fishbot_rs::detect;

fn main() {
    let shot = image::open("./test/WoWScrnShot_100626_002451.jpg").unwrap().to_luma8();
    let (sw, _sh) = shot.dimensions();

    // take a 100x100 patch from a middle-ish spot as the "template"
    let (px, py) = (sw / 2, 450);
    let tpl = ImageBuffer::from_fn(100, 100, |x, y| *shot.get_pixel(px + x, py + y));

    let results = detect::ncc_matches(&shot, &tpl, -1.0);
    println!("total matches above -1.0: {}", results.len());
    for r in results.iter().take(8) {
        println!("  ({}, {}) corr={:.4}", r.x, r.y, r.corr);
    }
    println!("expected top: ({}, {}), corr ~1.0", px, py);
}
