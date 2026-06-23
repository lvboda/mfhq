import argparse
import os
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
VENV_DIR = ROOT / ".venv-build"
SRC_DIR = ROOT / "src"
PHOTO_DIR = SRC_DIR / "common" / "photo"
DIST_DIR = ROOT / "dist"
PIP_INDEX = "https://mirrors.cloud.tencent.com/pypi/simple"
PIP_TRUST_HOST = "mirrors.cloud.tencent.com"
def run(cmd):
    print("+ " + " ".join(str(part) for part in cmd), flush=True)
    subprocess.check_call([str(part) for part in cmd], cwd=ROOT)


def venv_python():
    if os.name == "nt":
        return VENV_DIR / "Scripts" / "python.exe"
    return VENV_DIR / "bin" / "python"


def ensure_venv():
    python = venv_python()
    if not python.exists():
        run([sys.executable, "-m", "venv", VENV_DIR])
    return python


def install_dependencies(python):
    pip_args = [
        python,
        "-m",
        "pip",
        "install",
        "-i",
        PIP_INDEX,
        "--trusted-host",
        PIP_TRUST_HOST,
        "--timeout",
        "120",
        "--retries",
        "10",
    ]
    run([python, "-m", "pip", "install", "--upgrade", "pip", "-i", PIP_INDEX, "--trusted-host", PIP_TRUST_HOST])
    run([*pip_args, "-e", ".[build]"])


def pyinstaller(python, *args):
    run([python, "-m", "PyInstaller", *args])


def add_data_args(source, dest):
    return ["--add-data", f"{source}{os.pathsep}{dest}"]


def build_yingxionggu(python):
    pyinstaller(
        python,
        "--clean",
        "--onefile",
        "--console",
        "--name",
        "yingxionggu",
        *add_data_args(PHOTO_DIR, "photo"),
        "--paths",
        str(SRC_DIR),
        str(SRC_DIR / "yingxionggu.py"),
    )
    print(f"SUCCESS: {DIST_DIR / 'yingxionggu.exe'}")


def build_controller(python):
    pyinstaller(
        python,
        "--clean",
        "--onefile",
        "--noconsole",
        "--name",
        "mfhq_controller",
        "--paths",
        str(SRC_DIR),
        str(SRC_DIR / "controller.py"),
    )
    print(f"SUCCESS: {DIST_DIR / 'mfhq_controller.exe'}")


def main():
    parser = argparse.ArgumentParser(description="Build MFHQ Windows executables.")
    parser.add_argument("target", choices=["all", "yingxionggu", "controller"], nargs="?", default="all")
    args = parser.parse_args()

    if not (SRC_DIR / "yingxionggu.py").exists():
        raise SystemExit("ERROR: src/yingxionggu.py not found. Run this from the project folder.")
    if not (SRC_DIR / "controller.py").exists():
        raise SystemExit("ERROR: src/controller.py not found. Run this from the project folder.")

    python = ensure_venv()
    install_dependencies(python)

    if args.target in {"all", "yingxionggu"}:
        build_yingxionggu(python)
    if args.target in {"all", "controller"}:
        build_controller(python)


if __name__ == "__main__":
    main()
