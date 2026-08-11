# mfhq (Rust)

A port of the Python scripts under `../src/`. Two reasons it exists:

- **Size** — the PyInstaller exe is 72 MB because it bundles the Python interpreter, opencv and
  numpy. This one is about 2 MB.
- **Cross-compilation** — PyInstaller cannot cross-compile, so the Python exe can only be built on
  Windows or in CI. This one builds a Windows exe from macOS in about 20 seconds.

## Build

```bash
brew install mingw-w64
rustup target add x86_64-pc-windows-gnu
cargo build --release --target x86_64-pc-windows-gnu
```

Output is `target/x86_64-pc-windows-gnu/release/mfhq.exe`, a single file — the 27 templates are
embedded with `include_bytes!`, so no `photo/` directory needs to ship alongside it.

The linker is configured globally in `~/.cargo/config.toml`:

```toml
[target.x86_64-pc-windows-gnu]
linker = "x86_64-w64-mingw32-gcc"
ar = "x86_64-w64-mingw32-ar"
```

There is deliberately no CI for this build — it is faster locally than any runner.

## Run

```cmd
mfhq.exe                    interactive script picker
mfhq.exe hunzhu [1|2|3]     hunzhu loop, level defaults to 1
mfhq.exe yingxionggu        yingxionggu loop
mfhq.exe controller         scheduler
mfhq.exe diag [out.png]     capture the screen and score every template
mfhq.exe match <a> <b>      score one template against one image (works off Windows too)
mfhq.exe -c <script>        run tscon first to move the RDP session to the console
```

The level accepts five equivalent spellings: `hunzhu 2`, `-l 2`, `-level 2`, `--l 2`, `--level 2`.

`diag` is the first thing to run when recognition fails: it prints the screen size, the size the
capture actually came back as, and every template's best score and position. That separates "the
platform layer is broken" from "one template no longer matches".

### Environment

| Variable | Meaning |
|---|---|
| `MFHQ_GAME_PATH` | game executable, defaults to `C:\魔法哈奇\魔法哈奇.exe` |
| `MFHQ_CE_DIR` | Cheat Engine directory or exe |
| `MFHQ_APP_EXE` | what `controller` launches, defaults to itself |
| `MFHQ_THRESHOLD` | match threshold, defaults to 0.7 |

Cheat Engine is only needed by `yingxionggu`. Put it in `resources/Cheat Engine 6.5/` next to the
exe, or point `MFHQ_CE_DIR` at it.

Logs go to `yingxionggu.log` next to the exe, same format as the Python version.

## Layout

```
photo/              27 templates, embedded at compile time
src/
  vision.rs         ZNCC template matching — portable, no win32
  bot.rs            find / wait / click, threshold, game start and login
  assets.rs         embedded templates, each carrying its own filename
  ce.rs             Cheat Engine: locate, write the Lua autorun script, launch
  pidfile.rs        pid files, shared by ce.rs and controller
  config.rs         constants mirroring ../src/common/config.py
  log.rs            log lines matching the Python format
  platform/         screen, input, window, process, console — all win32
  tasks/            hunzhu, yingxionggu, controller, diag
```

`vision.rs` is portable and `assets.rs` is gated `cfg(any(windows, test))`; everything else is
Windows-only. That is what lets `cargo test` and the `match` subcommand run on macOS.

## Template matching

`cv2.matchTemplate` with `TM_CCOEFF_NORMED` is reimplemented rather than bound to OpenCV, because
any C dependency would break cross-compilation — the thing this port exists for.

The implementation is equivalent to OpenCV's, not merely similar. Since `Σ(T − mean T) = 0`, the
numerator `Σ(I − mean I)(T − mean T)` reduces to `Σ I·(T − mean T)`, a plain cross-correlation
computed by FFT; the denominator's `Σ(I − mean I)²` comes from integral images in O(1) per position.
FFT runs in f32 (as OpenCV's does), integral images in f64 because the squared sums reach 1e11 and
f32's mantissa is not enough.

`cargo test` pins this: five template pairs with scores from 0.108 to 0.964, expected values taken
from OpenCV, tolerance 1e-3. Measured agreement is 6e-4 across all 27 templates, with identical
match positions. **This is why the 0.7 threshold and the existing templates carry over unchanged
from the Python version** — the numbers mean the same thing in both.

A match costs roughly 90 ms on a 1470×923 screen on an M-series Mac, and is constant in template
size. Remaining known waste: the template's FFT, the FFT plans, and about 185 MB of buffers are
rebuilt on every call even though only the screenshot changes. Caching them across a polling loop
would cut roughly a third.

## Templates are per-resolution

`photo/` is deliberately separate from `../src/common/photo/`. Templates must be recaptured whenever
the game window size changes, and during the port the two implementations may run at different
sizes — sharing the directory would make each one break the other.

Recapture at 1:1 pixels. A screenshot saved from a zoomed view is upscaled and will not match.

Templates currently in `photo/` come from three different sessions. Only the hunzhu set has been
verified at the current window size; the 18 that `yingxionggu` depends on date from the original
project and have not been checked since.

## Not ported

- `pangpang` — a standalone decompiled script that shares nothing with `tool.py`.
- Disconnect recovery — implemented in Python on the `feature-reconnect` branch, not yet here.
- The 65 unused functions in `tool.py`.
