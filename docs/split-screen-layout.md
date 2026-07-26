# Split Screen Layout Branch

Branch: `feature-split-screen-layout`

This branch is an experimental branch for future same-desktop multi-instance automation. The goal is to let one physical/remote desktop be divided into four equal regions and eventually run one game client per region:

```text
lt / 1: left top
rt / 2: right top
lb / 3: left bottom
rb / 4: right bottom
```

The branch is intentionally separate from `main` because the multi-window workflow is not complete yet.

## What Changed

### CLI Argument Forwarding

`src/app.py` now forwards script-specific arguments to the selected script.

Examples:

```cmd
mfhq.exe yingxionggu --screens 1 2
mfhq.exe yingxionggu --screens 1-2-3-4
mfhq.exe yingxionggu --show-screens --screens lt rb
```

The existing interactive mode still works, so running `mfhq.exe` directly can accept commands such as:

```cmd
yingxionggu --screens 1 2
controller -c
```

### Screen Region Planning

Added `src/common/screen.py` with helpers for four-way screen planning.

It calculates screen regions as `(x, y, width, height)`. For a `1920x1080` screen, the regions are:

```text
lt: x=0,   y=0,   w=960, h=540
rt: x=960, y=0,   w=960, h=540
lb: x=0,   y=540, w=960, h=540
rb: x=960, y=540, w=960, h=540
```

If the screen size is odd, the right/bottom regions keep the extra pixels so the full desktop is covered.

Supported `--screens` forms include:

```cmd
--screens 1 2 3 4
--screens 1,2,3,4
--screens 1，2，3，4
--screens 1-2-3-4
--screens lt rt
--screens lt,rt
--screens lt，rt
--screens lt、rt
--screens lt;rb
--screens lt；rb
--screens lt/rt
--screens lt|rb
--screens all
--screens *
```

### Yingxionggu Split-Screen Options

`src/yingxionggu.py` now supports:

```cmd
--screens <selection>
--show-screens
```

Behavior:

- Without `--screens`: normal behavior, no region restriction.
- With `--screens`: split-screen mode is enabled and the selected regions are used.
- With `--show-screens`: print/log the selected region plan and exit without running the game workflow.

Examples:

```cmd
mfhq.exe yingxionggu --show-screens
mfhq.exe yingxionggu --show-screens --screens 1 4
mfhq.exe yingxionggu --screens 1 2
```

### Region-Aware Image Matching

`src/common/tool.py` now has an active region concept:

```python
tool.set_active_region((x, y, w, h), name)
tool.get_active_region()
tool.get_active_region_name()
```

When an active region is set:

- `find_img()` screenshots only that region.
- Returned match coordinates are converted back to absolute screen coordinates.
- `find_img_within_region()` also respects the active region.
- Mouse rest position after clicks stays inside the active region.

This means the existing click helpers can mostly keep using absolute screen coordinates while matching is restricted to one quadrant.

### Region Loop In Yingxionggu

When `--screens` selects more than one region, `yingxionggu` currently runs them sequentially in one process:

```text
round 1 [lt]
round 1 [rt]
round 1 [lb]
round 1 [rb]
round 2 [lt]
...
```

Logs include the region name, for example:

```text
第 1 轮[lt]开始
第 1 轮[rt]开始
```

### CE Patch Preparation For Multi-Instance

`tool.run_ce_double_patch()` now accepts an optional `target_pid`.

In split-screen mode, `yingxionggu` tries to get the PID of the window at the active region center and passes it into the CE patch. This is meant to reduce the chance of patching the wrong game process when multiple clients exist.

## Multi-Instance Note

For 3-4 accounts, the game supports opening multiple instances natively — no special multi-open technique is required. Just launch the game multiple times. A dedicated multi-open approach is only needed for 10+ accounts, which is out of scope for now.

### Multi-Instance Startup Flow

When `--screens` is specified, the startup flow is:

1. **Start instances** — `start_mofa_haqi_instances(count)` checks how many game windows already exist. If fewer than the selected screen count, it launches additional instances via `os.startfile()` and waits up to 2 minutes for all windows to appear.
2. **Arrange windows** — `arrange_game_windows()` uses `win32gui.SetWindowPos()` to move and resize each game window into its assigned screen region. The window size matches the outer window rect (including title bar/borders).
3. **Login each** — For each region, the script sets the active region, focuses the bound window, and runs the standard login flow. If a window is already logged in (map icon visible), login is skipped.

The window-to-region binding (`RuntimeOptions.window_binding`) maps each screen key to `(hwnd, pid)` and is used throughout the automation loop for:

- Focusing the correct window before operating a region.
- CE patch target PID (uses bound PID instead of the region-center heuristic).

### Window Discovery

```cmd
mfhq.exe yingxionggu --show-windows
```

Lists all detected game windows with hwnd, pid, and title:

```text
[1] hwnd=123456 pid=8596 title=魔法哈奇
[2] hwnd=123888 pid=9120 title=魔法哈奇
```

## Current Limitations

- `SetWindowPos()` sets the outer window rect — the actual game client area will be slightly smaller due to title bar and borders.
- `pyautogui` actions are still system-wide, not true background window messages. Multiple independently running `mfhq.exe` processes can compete for mouse/keyboard input. The single-process sequential orchestrator avoids this.
- The controller still manages a single scheduled script/PID model.
- The code has not been validated on Windows with real game windows yet.

## Remaining Tasks

### 1. Safer Mouse/Keyboard Coordination

If multiple script processes are used, they can still fight over the mouse and keyboard.

Possible approaches:

- Use one orchestrator process that runs regions sequentially (current approach).
- Add a global lock file/mutex around mouse/keyboard actions.
- Avoid launching four independent processes until locking exists.

### 2. Controller Multi-Instance Support

The current controller starts/stops one `yingxionggu` process with one PID file.

Future options:

- Keep controller single-instance and run one orchestrator process for all regions.
- Or extend controller to manage one PID per region.

The orchestrator option is probably safer for `pyautogui`.

### 3. Real Windows Validation

The region automation must be validated on Windows with real game windows.

Test checklist:

```cmd
mfhq.exe yingxionggu --show-windows
mfhq.exe yingxionggu --show-screens --screens 1 2 3 4
mfhq.exe yingxionggu --screens 1
mfhq.exe yingxionggu --screens 1 2
mfhq.exe yingxionggu --screens 1 2 3 4
```

Things to verify:

- game instances start correctly and windows appear;
- windows are arranged into the correct quadrants;
- login flow works for each window;
- screenshots are restricted to the selected region;
- clicks land in the correct quadrant;
- mouse rest position stays inside the selected quadrant;
- CE patch targets the intended game process;
- logs are readable and include region names.
