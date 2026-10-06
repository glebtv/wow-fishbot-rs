# Session Summary (combined)

A detailed record of the work across the **previous** and **current** sessions on
`C:\data\fish\wow-fishbot-rs` (a Rust World of Warcraft fishing bot).

## Objective

- Make the Rust WoW fishing bot run on **Windows** using the local `rustautogui`
  crate and fix bobber/lure detection, which had been reporting
  "Bobber not detected."
- Use the user-provided screenshots in `test/` to try and compare detection
  approaches.

## Previous session

### Stack replacement
- Replaced OpenCV / xcap / enigo with **rustautogui** (local path dependency).
- Added a `grab_screen_rgba` wrapper to rustautogui.
- Rewrote `main.rs` for Windows (winapi window find/activate); removed the Linux
  `xdotool` path.
- Built a custom **FFT NCC** template matcher (`src/detect.rs`), fixed a threshold
  bug, and evaluated it offline: best correlation ~**0.65** — below the 0.8 live
  threshold, and it kept locking onto the character's armor and UI text.

### Detection experiments
- Grayscale NCC / multi-scale NCC proved insufficient.
- Developed a **color-based (HSV) detector** in scratch tooling: classify pixels as
  blue / red / tan, group blue+red into connected components, filter by size /
  position / balance, and score by cork proximity.
- Validated on both reference screenshots — the lure ranked **#1** in each:
  - `WoWScrnShot_100626_002451.jpg` → lure at **(627, 202)**
  - `WoWScrnShot_100626_004009.jpg` → lure at **(869, 203)**

## Current session

### Detection integrated into the bot
- Added the `src/lure_detect.rs` library module
  (`find_lure`, `find_lure_candidates`, `LureOptions`, `LureMatch`).
- Exposed the modules from `src/lib.rs`.
- Rewrote `src/main.rs` to use the color detector
  (`grab_screen_rgba` → `find_lure`) with a fixed 120×120 splash region.

### Live-capture investigation
- Live GDI capture returned an **all-black** frame (1440×900).
- Confirmed a single monitor and that the WoW window was visible but not foreground.
- Root cause: the game was in **exclusive fullscreen**, which GDI screen capture
  cannot read.
- Resolution: run the game in **borderless windowed** for a capturable frame.
  (Offline detection on the user's screenshots already validates the detector.)

### Behavior changes (this session's requests)
- Added a **`--runs N`** CLI flag to stop after N cast cycles.
- **Failure screenshots** are saved to `failures/` on every lure miss.
- **Splash timeout set to 18 s** (17 s game timer + 1 s headroom).
- On a splash, the **bobber is re-located and right-clicked** at its current
  position (fixes the click landing off the bobber).
- Cleaned up unused imports; the whole workspace now builds warning-free.

### Docs & packaging
- Rewrote `README.md` (tech stack, Windows usage, CLI flags, per-file source
  layout, WIP / private-server note, AI disclosure).
- Added `docs/changes.md` and `docs/session-summary.md` (this file).
- Added `.gitignore`.
- Committed all Rust sources + `Cargo.lock` + docs and pushed to the `glebtv` fork.

## Key technical notes

- `image` 0.25: `ImageBuffer<Rgba<u8>>` has **no** `to_rgb8()` (that's on
  `DynamicImage`); convert manually with `ImageBuffer::from_fn`.
- winapi `keybd_event` signature is `(u8, u8, u32, usize)`; `VK_MENU` must be cast
  to `u8`.
- rustautogui `get_screen_size()` returns `(i32, i32)`; the bool in
  `RustAutoGui::new(false)` is just a debug flag.
- `LureOptions` defaults: `min_n 25`, `max_n 900`, `max_dim 80`, `min_blue 8`,
  `min_red 8`, `tan_full 300`, `min_balance 0.30`, `water_frac 0.45`.
- Score = `(blue + red) * balance * (1.0 + min(tan, tan_full) / tan_full * 0.5)`.

## Open items / next steps

- Verify live detection with the game in **borderless windowed** and an active
  fishing scene.
- Tune `LureOptions` per resolution / lighting using the `failures/` screenshots.
- Optionally merge the most useful debug binaries into the main CLI.
