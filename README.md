# MFHQ scripts

This project uses a simple `src` layout: shared modules stay in `src/common`, and executable scripts stay directly under `src`.

## Structure

- `src/common/`: shared code, config, and image assets.
  - `tool.py`: shared automation helpers.
  - `const.py`: shared image path constants.
  - `config.py`: shared runtime/build config.
  - `photo/`: shared image templates.
- `src/yingxionggu.py`: yingxionggu script entry point.
- `src/controller.py`: resident scheduler/controller entry point.
- `scripts/build.py`: shared build implementation.
- `scripts/*.cmd`: Windows double-click build wrappers.

## Build on Windows

- Build both exes: run `scripts\\build_all.cmd`.
- Yingxionggu script only: run `scripts\\build_yingxionggu.cmd`.
- Controller only: run `scripts\\build_controller.cmd`.

The build script creates `.venv-build`, installs dependencies from `pyproject.toml`, and outputs executables to `dist/`.

