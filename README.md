# WoW Fishing Bot (Rust)

A research / proof-of-concept fishing bot for World of Warcraft, written in Rust.
It detects the fishing lure (bobber) **by color**, watches a region around it for a
splash, and right-clicks the bobber to catch the fish.

> This is a fork of [zuernerd/wow-fishbot-rs](https://github.com/zuernerd/wow-fishbot-rs),
> reworked to build on Windows with a different capture/input stack and a
> color-based detector instead of OpenCV template matching.

## ⚠️ Status & Disclaimer

**Work in progress — research project.** This is an ongoing research experiment,
not a finished product. Expect rough edges, incomplete features, and changes
without notice.

**For private servers only.** Please use this only on your own (or a friend's)
**private server**. Do **not** use it on official / live servers.

Using automation tools in World of Warcraft violates the terms of service and can
result in account suspension or permanent bans. Use this code at your own risk —
you will probably be banned!!

### AI Disclosure

**AI usage: YES** — all changes in this fork were done with the
**Qwen 3.8 27b IQ4_NL** model.

## How It Works

1. **Window activation** — finds the `World of Warcraft` window and brings it to
   the foreground (an ALT-tap is sent first so Windows allows the focus change).
2. **Cast** — presses the fishing key (**F4**) with randomized delays.
3. **Lure detection** — captures the screen and runs a **color-based (HSV)
   detector** to find the bobber. The bobber is recognized by its blue + red
   feathers on a tan cork float; pixel clusters are filtered by size, position
   (upper "water" region above the character) and blue/red balance, then ranked
   by a score that is boosted when a cork float is nearby.
   *(This replaced the original grayscale template matching, which tended to lock
   onto the character's armor and UI text.)*
4. **Cursor placement** — moves the mouse cursor onto the detected lure.
5. **Splash monitoring** — watches a 120×120 region around the lure and compares
   consecutive grayscale frames; if enough pixels change, a fish has taken the
   bait.
6. **Catch** — when a splash is detected, the bobber is **re-located** (in case it
   drifted) and **right-clicked** to reel the fish in.
7. **Loop** — repeats with randomized delays. If the lure is **not** detected, a
   full-screen shot is saved to `failures/` for later review.

## Tech Stack

- **Rust** (edition 2021)
- **[rustautogui](../rustautogui)** — a local fork (path dependency) used for
  screen capture (`grab_screen_rgba`, `grab_screen_grayscale`) and input
  simulation (keys + mouse). Replaces the original `xcap` + `enigo`.
- **image** — image loading, pixel access, PNG/JPEG encoding.
- **rand** — randomized, human-like delays.
- **winapi** (Windows) — finding/activating the game window.
- Custom **HSV color detector** (no OpenCV) for lure detection.
- *(Superseded, kept for research)* **rustfft** — a custom FFT-based normalized
  cross-correlation template matcher in `src/detect.rs`.

## Prerequisites

- Rust toolchain (`cargo`)
- **Windows** (this fork targets Windows; the game window must be capturable — see
  Window Setup)
- The local `rustautogui` crate at `../rustautogui` (path dependency)

## Build

```bash
cargo build --release
```

## Usage

1. Position your character in front of a fishing spot.
2. **Hide the UI (Alt+Z) and zoom into first-person view.**
3. Make sure you have a fishing pole equipped.
4. Run the bot:

```bash
cargo run --release
# or run a fixed number of cast cycles and then stop:
cargo run --release -- --runs 5
```

### CLI flags

| Flag | Meaning |
| --- | --- |
| `--runs N` / `--runs=N` | Stop after `N` cast cycles (default: run until interrupted). |

### Stopping the bot

- Press `Ctrl+C` in the terminal, or
- launch with `--runs N` to stop automatically after `N` cycles.

### Window setup

- Run World of Warcraft in **borderless windowed** (or windowed) mode.
  **Exclusive fullscreen cannot be captured** by the GDI screen grab and will
  produce an all-black frame.
- Keep the game window visible and not minimized.
- The bot looks for a window titled `World of Warcraft`.

## Configuration

The main tunables are constants at the top of `src/main.rs`:

| Constant | Default | Meaning |
| --- | --- | --- |
| `SPLASH_PIXEL_THRESHOLD` | `50` | per-pixel change that counts as "changed" (0–255). |
| `SPLASH_MIN_CHANGED_PIXELS` | `250` | changed pixels in the region that trigger a splash. |
| `SPLASH_REGION_SIZE` | `120` | size of the square splash-watch region (px). |
| `SPLASH_TIMEOUT` | `18 s` | how long to wait for a splash (the game's no-bite timer is 17 s). |
| `FAILURES_DIR` | `failures` | where lure-miss screenshots are saved. |

The lure detector thresholds live in `LureOptions::default()` in
`src/lure_detect.rs` (min/max cluster size, min blue/red, balance, cork boost,
water fraction). Adjust them to your resolution, textures and lighting.

## Failures folder

Every time the lure is **not** detected, the bot saves the full-screen frame to
`failures/lure_miss_NNNN_t<epoch>.png`. Review these manually to understand and
tune the misses (lighting, UI, camera angle, zoom, etc.).

## Source Layout

### Library — `src/lib.rs`

| File | What it does |
| --- | --- |
| `lure_detect.rs` | **Color-based lure detector** (the one the bot uses). HSV classifies pixels into blue/red/tan, groups blue+red into connected clusters, filters by size/position/balance, and scores by cork proximity. Exposes `find_lure`, `find_lure_candidates`, `LureOptions`, `LureMatch`. |
| `detect.rs` | Custom **FFT normalized cross-correlation (NCC) template matcher** (research; superseded by the color detector). Exposes `ncc_matches`, `GrayImage`. |

### Main binary — `src/main.rs`

The bot itself: window activation → cast → color lure detection → cursor
placement → splash watch → re-locate + right-click. Supports `--runs N` and saves
failure screenshots to `failures/`.

### Research / debug binaries — `src/bin/`

Small one-off tools used while developing the detection. Each compiles to its own
binary (`cargo build --release` builds all of them). These are development aids —
**none are required to run the bot**.

| Binary | What it does |
| --- | --- |
| `check_lure` | Runs the color detector offline on the two `test/` screenshots and prints the best match. |
| `color_detect` | Standalone HSV color-detector harness with CLI threshold overrides; prints ranked candidates for a screenshot. |
| `live_lure` | Live one-shot: captures the screen, runs the color detector, prints the top candidates, and saves the frame plus a crop around the best hit. |
| `window_check` | Diagnostics for the WoW window (exists / visible / foreground / title). |
| `ncc_probe` | Probes NCC correlation between a template and a screenshot; prints top matches and the correlation near the lure. |
| `eval_detect` | NCC evaluation harness: loads templates from a directory, runs NCC, and renders ASCII previews. |
| `debug_detect` | Live NCC debug: captures the screen, matches templates, and prints ASCII + the best match. |
| `selftest` | NCC self-test: cuts a patch out of a screenshot and verifies NCC finds it at ~1.0. |
| `find_bobber` | Experimental bobber finder (color/edge based). |
| `color_map` | Dumps the per-pixel HSV class for debugging classification. |
| `sample_lure` | Inspects HSV of lure-region pixels for tuning. |
| `sample_region` | Prints HSV statistics for a rectangular region of an image. |
| `pixels` | Prints RGB/luma values at specific pixel coordinates. |
| `crop_img` | Crops a rectangle out of an image: `crop_img <in> <out> <x> <y> <w> <h>`. |

## Troubleshooting

- **All-black capture / "Lure not detected" every time** — the game is probably in
  exclusive fullscreen. Switch to **borderless windowed**.
- **Lure detection is intermittent** — review the `failures/` screenshots and tune
  `LureOptions` (size, balance, water fraction) for your resolution/lighting.
- **Splash too (in)sensitive** — adjust `SPLASH_PIXEL_THRESHOLD` and
  `SPLASH_MIN_CHANGED_PIXELS`.

## Dependencies

- `rustautogui` (local path dependency) — screen capture + input simulation
- `image` — image processing/IO
- `rand` — randomized delays
- `winapi` — Windows window handling
- `rustfft` — used only by the (superseded) NCC research module

## License

This project is licensed under the MIT License — see the [LICENSE](LICENSE) file
for details.

---

*Work in progress research project. Private servers only. Use at your own risk and
respect the terms of service of any game you play.*
