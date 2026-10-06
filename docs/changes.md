# Change Log

All changes made in this fork relative to the original
[`zuernerd/wow-fishbot-rs`](https://github.com/zuernerd/wow-fishbot-rs).

## 1. Capture / input stack — OpenCV + xcap + enigo → rustautogui

- Removed the `opencv`, `xcap`, and `enigo` dependencies.
- Added a local **rustautogui** fork as a path dependency (`../rustautogui`):
  - Added a `grab_screen_rgba` wrapper in `rustautogui_impl/mod.rs` that returns
    `ImageBuffer<Rgba<u8>>`, alongside the existing grayscale grab.
- Rewrote `src/main.rs` to use rustautogui for screen capture and input.
- Added `winapi` (`winuser`, `windef`) for Windows window find/activate.
- Removed the Linux-only `xdotool` window handling; added a Windows
  `find_and_activate_wow_window()` (`FindWindowW` + `SetForegroundWindow`, with an
  ALT-tap so Windows allows the focus change).

## 2. Detection — template matching → color-based lure detector

- The original grayscale NCC / template matching locked onto the character's armor
  and UI text (best offline correlation ~0.65, below the 0.8 live threshold).
- Added `src/lure_detect.rs`: an **HSV color detector** that finds the bobber by its
  blue + red feathers and tan cork float (connected components + size/position/
  balance filters + cork-proximity scoring). Validated on both reference
  screenshots — the lure ranks #1 in each.
- Kept the custom FFT NCC matcher in `src/detect.rs` for research (no longer used by
  the bot).

## 3. Bot behavior (`src/main.rs`)

- Detection now uses `grab_screen_rgba` + `find_lure` instead of templates.
- Splash-watch region is a fixed **120×120** square centered on the lure (replaces
  the old template-dimensioned region).
- **Splash timeout set to 18 s** (the game's no-bite timer is 17 s + 1 s headroom).
- **On a detected splash, the bobber is re-located and right-clicked** at its current
  position (previously the click used the stale initial position and often missed).
- **CLI: `--runs N` / `--runs=N`** — stop after N cast cycles (default: run until
  `Ctrl+C`).
- **Failure capture:** when the lure is not detected, the full-screen frame is saved
  to `failures/lure_miss_NNNN_t<epoch>.png` for manual review.

## 4. Research / debug binaries (`src/bin/`)

Added a set of small tools used to develop and validate detection (see the table in
the README): `check_lure`, `color_detect`, `live_lure`, `window_check`, `ncc_probe`,
`eval_detect`, `debug_detect`, `selftest`, `find_bobber`, `color_map`, `sample_lure`,
`sample_region`, `pixels`, `crop_img`.

Cleaned up unused imports so the whole workspace builds warning-free.

## 5. Packaging & docs

- Added `.gitignore` (`target/`, generated screenshots, scratch files, debug dirs).
- Rewrote `README.md`: new tech stack, Windows build/usage, CLI flags, per-file
  source layout, WIP / private-server status note, and AI disclosure.
- Added `docs/changes.md` (this file) and `docs/session-summary.md`.
- Committed `Cargo.lock` for reproducible builds.
