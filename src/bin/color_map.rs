use image::RgbImage;

fn main() {
    let path = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "./test/WoWScrnShot_100626_002451.jpg".to_string());
    let img: RgbImage = image::open(&path).unwrap().to_rgb8();
    let (w, h) = img.dimensions();

    // block size for downsampled color map
    let bs = 8u32;
    let bw = w / bs;
    let bh = h / bs;

    let mut out = String::new();
    for by in 0..bh {
        for bx in 0..bw {
            let mut r = 0u32;
            let mut g = 0u32;
            let mut b = 0u32;
            let mut n = 0u32;
            for dy in 0..bs {
                for dx in 0..bs {
                    let p = img.get_pixel(bx * bs + dx, by * bs + dy);
                    r += p[0] as u32;
                    g += p[1] as u32;
                    b += p[2] as u32;
                    n += 1;
                }
            }
            r /= n;
            g /= n;
            b /= n;
            let mx = r.max(g).max(b);
            let mn = r.min(g).min(b);
            let sat = mx - mn;
            let c: char = if mx < 60 {
                ' ' // very dark
            } else if sat < 25 {
                if mx > 200 {
                    'W' // near white
                } else if mx > 140 {
                    'L' // light gray
                } else if mx > 90 {
                    'G' // mid gray
                } else {
                    'd' // dark gray
                }
            } else if r >= g && r >= b {
                if g > b && g > 100 && r > 150 {
                    'y' // yellow/orange
                } else {
                    'R' // red
                }
            } else if g >= r && g >= b {
                'g' // green
            } else if b > r && b > g {
                if r > 100 && g > 100 {
                    'c' // cyan-ish
                } else {
                    'B' // blue
                }
            } else {
                'M' // magenta-ish
            };
            out.push(c);
        }
        out.push('\n');
    }
    println!("{}", out);
    println!("legend: W=white L=light G=gray d=dark R=red y=yellow g=green B=blue c=cyan M=magenta ' '=black");
}
