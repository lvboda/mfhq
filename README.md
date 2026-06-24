# MFHQ scripts

This project uses a simple `src` layout: shared modules stay in `src/common`, executable scripts stay directly under `src`, and `src/app.py` is the unified command entry.

## Structure

- `src/app.py`: unified CLI entry for all scripts.
- `src/common/`: shared code, config, and image assets.
  - `tool.py`: shared automation helpers.
  - `const.py`: shared image path constants.
  - `config.py`: shared runtime/build config.
  - `photo/`: shared image templates.
- `src/yingxionggu.py`: yingxionggu script.
- `src/pangpang.py`: pangpang card-clicker script.
- `src/controller.py`: resident scheduler/controller script.
- `scripts/build.py`: shared build implementation.
- `scripts/*.cmd`: Windows double-click build wrappers.

## Build On Windows

- Build the unified exe: run `scripts\\build.cmd`.

The build script creates `.venv-build`, installs dependencies from `pyproject.toml`, and outputs:

```text
dist\\mfhq.exe
```

## Run

```cmd
mfhq.exe yingxionggu
mfhq.exe pangpang
mfhq.exe controller
```

The controller starts and stops `mfhq.exe yingxionggu` by PID according to the schedule in `src/common/config.py`.
