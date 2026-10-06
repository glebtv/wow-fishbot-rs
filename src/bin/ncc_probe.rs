use wow_fishbot_rs::detect;

fn main() {
    let shot = image::open("./test/WoWScrnShot_100626_004009.jpg")
        .unwrap()
        .to_luma8();
    let tpl = image::open("./template/bobber3.png").unwrap().to_luma8();
    let (tw, th) = tpl.dimensions();
    println!("shot {}x{}, template {}x{}", shot.dimensions().0, shot.dimensions().1, tw, th);

    // all matches
    let thr: f64 = std::env::args()
        .nth(1)
        .and_then(|a| a.parse().ok())
        .unwrap_or(-1.0);
    let all = detect::ncc_matches(&shot, &tpl, thr);
    println!("positions scored: {}", all.len());
    if thr > -0.99 {
        // consistency check: fetch everything, count positives
        let full = detect::ncc_matches(&shot, &tpl, -1.0);
        let pos = full.iter().filter(|r| r.corr > 0.0).count();
        let max = full.iter().map(|r| r.corr).fold(f64::MIN, f64::max);
        println!(
            "consistency: full={} positives={} max={:.4}",
            full.len(),
            pos,
            max
        );
    }
    println!("top 10:");
    for r in all.iter().take(10) {
        println!(
            "  ({},{}) center=({},{}) corr={:.4}",
            r.x,
            r.y,
            r.x + tw / 2,
            r.y + th / 2,
            r.corr
        );
    }

    // corr at the lure location (top-left ~ (845,175))
    let target = (845u32, 175u32);
    let best_near = all
        .iter()
        .filter(|r| (r.x as i64 - target.0 as i64).abs() <= 15 && (r.y as i64 - target.1 as i64).abs() <= 15)
        .max_by(|a, b| a.corr.partial_cmp(&b.corr).unwrap());
    if let Some(r) = best_near {
        println!(
            "best near lure ({},{}): ({},{}) corr={:.4}",
            target.0, target.1, r.x, r.y, r.corr
        );
    } else {
        println!("no positions near lure found");
    }
}
