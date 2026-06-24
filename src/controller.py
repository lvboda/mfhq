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
    return Path(__file__).resolve().parents[1]


def creation_flags():
    return getattr(subprocess, "CREATE_NO_WINDOW", 0)


def app_exe_path():
    configured = os.environ.get(config.APP_EXE_ENV)
    if configured:
        return Path(configured)
    if getattr(sys, "frozen", False):
        return Path(sys.executable).resolve()
    return app_dir() / config.APP_EXE_NAME


def script_command():
    if getattr(sys, "frozen", False):
        return [str(app_exe_path()), config.SCHEDULED_SCRIPT]
    return [sys.executable, str(app_dir() / "src" / "app.py"), config.SCHEDULED_SCRIPT]


def pid_file():
    return app_dir() / config.SCRIPT_PID_FILE


def read_pid():
    try:
        return int(pid_file().read_text(encoding="utf-8").strip())
    except Exception:
        return None


def write_pid(pid):
    try:
        pid_file().write_text(str(pid), encoding="utf-8")
    except Exception as e:
        log(f"write pid failed: {e}")


def clear_pid():
    try:
        pid_file().unlink(missing_ok=True)
    except Exception:
        pass


def is_active_time(now=None):
    now = now or datetime.datetime.now()
    return now.weekday() in config.ACTIVE_WEEKDAYS and config.START_HOUR <= now.hour < config.STOP_HOUR


def is_pid_running(pid):
    if not pid:
        return False
    try:
        output = subprocess.check_output(
            ["tasklist", "/FI", f"PID eq {pid}", "/FO", "CSV", "/NH"],
            text=True,
            errors="ignore",
            creationflags=creation_flags(),
        )
    except Exception:
        return False
    return str(pid) in output


def start_script():
    pid = read_pid()
    if is_pid_running(pid):
        return
    clear_pid()

    cmd = script_command()
    target = Path(cmd[0])
    if not target.exists():
        log(f"app exe not found: {target}")
        return

    process = subprocess.Popen(cmd, cwd=str(app_dir()), creationflags=creation_flags())
    write_pid(process.pid)
    log(f"started: {' '.join(cmd)} pid={process.pid}")


def stop_script():
    pid = read_pid()
    if not is_pid_running(pid):
        clear_pid()
        return

    subprocess.run(
        ["taskkill", "/F", "/T", "/PID", str(pid)],
        stdout=subprocess.DEVNULL,
        stderr=subprocess.DEVNULL,
        creationflags=creation_flags(),
    )
    clear_pid()
    log(f"stopped pid={pid}")


def log(message):
    try:
        path = app_dir() / "controller.log"
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
