import datetime
import os
import subprocess
import sys
import time
from pathlib import Path

from common import config


def app_dir():
    if getattr(sys, "frozen", False):
        return Path(sys.executable).resolve().parent
    return Path(__file__).resolve().parents[2]


def script_path():
    configured = os.environ.get(config.SCRIPT_EXE_ENV)
    if configured:
        return Path(configured)
    return app_dir() / config.SCRIPT_EXE_NAME


def is_active_time(now=None):
    now = now or datetime.datetime.now()
    return now.weekday() in config.ACTIVE_WEEKDAYS and config.START_HOUR <= now.hour < config.STOP_HOUR


def is_process_running_by_name(exe_name):
    try:
        output = subprocess.check_output(
            ["tasklist", "/FI", f"IMAGENAME eq {exe_name}", "/FO", "CSV", "/NH"],
            text=True,
            errors="ignore",
            creationflags=subprocess.CREATE_NO_WINDOW,
        )
    except Exception:
        return False
    return exe_name.lower() in output.lower()


def start_script():
    target = script_path()
    if not target.exists():
        log(f"script exe not found: {target}")
        return
    if is_process_running_by_name(target.name):
        return
    subprocess.Popen([str(target)], cwd=str(target.parent), creationflags=subprocess.CREATE_NO_WINDOW)
    log(f"started: {target}")


def stop_script():
    target = script_path()
    if not is_process_running_by_name(target.name):
        return
    subprocess.run(
        ["taskkill", "/F", "/T", "/IM", target.name],
        stdout=subprocess.DEVNULL,
        stderr=subprocess.DEVNULL,
        creationflags=subprocess.CREATE_NO_WINDOW,
    )
    log(f"stopped: {target.name}")


def log(message):
    try:
        path = app_dir() / "mfhq_controller.log"
        stamp = datetime.datetime.now().strftime("%Y-%m-%d %H:%M:%S")
        with path.open("a", encoding="utf-8") as f:
            f.write(f"[{stamp}] {message}\n")
    except Exception:
        pass


def main():
    log("controller started")
    while True:
        if is_active_time():
            start_script()
        else:
            stop_script()
        time.sleep(config.CHECK_INTERVAL_SECONDS)


if __name__ == "__main__":
    main()
