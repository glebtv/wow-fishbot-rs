use image::RgbImage;
use wow_fishbot_rs::lure_detect::{find_lure, LureOptions};

fn main() {
    let opts = LureOptions::default();
    let shots = [
        "./test/WoWScrnShot_100626_002451.jpg",
        "./test/WoWScrnShot_100626_004009.jpg",
    ];
    for path in shots {
        let img: RgbImage = image::open(path).unwrap().to_rgb8();
        match find_lure(&img, &opts) {
            Some(m) => println!(
                "{} -> lure at ({},{}) score={:.0} blue={} red={} tan={} bal={:.2}",
                path, m.x, m.y, m.score, m.blue, m.red, m.tan, m.balance
            ),
            None => println!("{} -> NOT FOUND", path),
        }
    }
}
