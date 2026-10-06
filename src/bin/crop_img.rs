use image::{ImageBuffer, RgbImage};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let (in_path, out_path, x, y, w, h) = match (
        args.get(1),
        args.get(2),
        args.get(3),
        args.get(4),
        args.get(5),
        args.get(6),
    ) {
        (Some(a), Some(b), Some(c), Some(d), Some(e), Some(f)) => (
            a.clone(),
            b.clone(),
            c.parse::<u32>().unwrap(),
            d.parse::<u32>().unwrap(),
            e.parse::<u32>().unwrap(),
            f.parse::<u32>().unwrap(),
        ),
        _ => panic!("usage: crop_img <in> <out> <x> <y> <w> <h>"),
    };

    let img: RgbImage = image::open(&in_path).unwrap().to_rgb8();
    let (iw, ih) = img.dimensions();
    let x = x.min(iw.saturating_sub(1));
    let y = y.min(ih.saturating_sub(1));
    let w = w.min(iw - x);
    let h = h.min(ih - y);
    let crop: RgbImage = ImageBuffer::from_fn(w, h, |cx, cy| *img.get_pixel(x + cx, y + cy));
    crop.save(&out_path).unwrap();
    println!("saved {} ({}x{}) from ({},{})", out_path, w, h, x, y);
}
