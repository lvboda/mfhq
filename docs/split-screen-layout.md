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

## Current Limitations

This branch is not a complete multi-instance solution yet.

Important limitations:

- It does not automatically start multiple game clients.
- It does not automatically arrange game windows into quadrants yet.
- It does not verify that the window in a region is actually the matching game client.
- Multiple independently running `mfhq.exe` processes can still compete for mouse/keyboard input.
- `pyautogui` actions are still system-wide, not true background window messages.
- The controller still manages a single scheduled script/PID model.
- CE targeting by region-center PID is only a first step and needs real Windows validation.

## Remaining Tasks

### 1. Window Discovery

Add a way to list all current game windows.

Possible command:

```cmd
mfhq.exe yingxionggu --show-windows
```

Expected output/log:

```text
[1] hwnd=123456 pid=8596 title=魔法哈奇
[2] hwnd=123888 pid=9120 title=魔法哈奇
```

### 2. Multi-Client Window Arrangement

This is not implemented yet. The branch can calculate four screen regions, but it does not yet automatically move or resize multiple game clients into those regions.

Add a flag to arrange detected game windows into selected regions.

Possible command:

```cmd
mfhq.exe yingxionggu --screens 1 2 3 4 --arrange
```

Expected behavior:

```text
window 1 -> lt: move/resize to left-top region
window 2 -> rt: move/resize to right-top region
window 3 -> lb: move/resize to left-bottom region
window 4 -> rb: move/resize to right-bottom region
```

Implementation notes:

- Discover all visible game windows first.
- Use `win32gui.SetWindowPos()` to move and resize windows.
- Decide whether the target size should represent the full outer window rect or the game client area.
- If exact client-area sizing is required, use `GetWindowRect`, `GetClientRect`, and `AdjustWindowRectEx` to account for title bars/borders.
- Provide a dry-run/log mode so the mapping can be checked before moving windows.
- Log the mapping from region to hwnd/pid/title.
- Handle fewer windows than selected regions with a clear warning.
- Handle more windows than selected regions by ignoring extras unless explicitly requested.

### 3. Window-To-Region Binding

Once windows are arranged, persist or hold a mapping:

```text
lt -> hwnd/pid A
rt -> hwnd/pid B
lb -> hwnd/pid C
rb -> hwnd/pid D
```

This mapping should be used for:

- focusing the correct window before operating a region;
- CE patch target PID;
- debugging logs.

### 4. Safer Mouse/Keyboard Coordination

If multiple script processes are used, they can still fight over the mouse and keyboard.

Possible approaches:

- Use one orchestrator process that runs regions sequentially.
- Add a global lock file/mutex around mouse/keyboard actions.
- Avoid launching four independent processes until locking exists.

### 5. Controller Multi-Instance Support

The current controller starts/stops one `yingxionggu` process with one PID file.

Future options:

- Keep controller single-instance and run one orchestrator process for all regions.
- Or extend controller to manage one PID per region:

```text
yingxionggu_lt.pid
yingxionggu_rt.pid
yingxionggu_lb.pid
yingxionggu_rb.pid
```

The orchestrator option is probably safer for `pyautogui`.

### 6. Real Windows Validation

The current changes were syntax-checked locally, but the region automation must be validated on Windows with real game windows.

Test checklist:

```cmd
mfhq.exe yingxionggu --show-screens --screens 1 2 3 4
mfhq.exe yingxionggu --screens 1
mfhq.exe yingxionggu --screens 2
```

Things to verify:

- screenshots are restricted to the selected region;
- clicks land in the correct quadrant;
- mouse rest position stays inside the selected quadrant;
- CE patch targets the intended game process;
- logs are readable and include region names.

## Suggested Next Step

Implement `--show-windows` and `--arrange` first. Those are low-risk and will make it easy to confirm whether multiple game clients can be discovered and placed reliably before changing the main automation loop further.
