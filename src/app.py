import argparse
import ctypes
import importlib
import os
import shlex
import subprocess

SCRIPTS = {
    "yingxionggu": "yingxionggu",
    "pangpang": "pangpang",
    "controller": "controller",
}


def current_session_id():
    session_id = ctypes.c_ulong()
    ok = ctypes.windll.kernel32.ProcessIdToSessionId(os.getpid(), ctypes.byref(session_id))
    if not ok:
        return None
    return session_id.value


def detach_to_console():
    if os.name != "nt":
        print("-c only works on Windows; skipped", flush=True)
        return False
    if os.environ.get("SESSIONNAME", "").lower() == "console":
        print("already running on console session; skipped tscon", flush=True)
        return True

    session_id = current_session_id()
    if session_id is None:
        print("failed to get current session id; skipped tscon", flush=True)
        return False

    tscon = os.path.join(os.environ.get("WINDIR", r"C:\Windows"), "System32", "tscon.exe")
    cmd = [tscon, str(session_id), "/dest:console"]
    try:
        result = subprocess.run(cmd, capture_output=True, text=True, errors="ignore", timeout=10)
    except Exception as e:
        print(f"failed to run tscon for session {session_id}: {e}", flush=True)
        return False
    if result.returncode != 0:
        error = (result.stderr or result.stdout or "").strip()
        print(f"tscon failed for session {session_id}: {error}", flush=True)
        print("try running mfhq.exe as Administrator", flush=True)
        return False

    print(f"detaching RDP session {session_id} to console", flush=True)
    return True


def build_parser():
    parser = argparse.ArgumentParser(prog="mfhq")
    parser.add_argument("script", choices=sorted(SCRIPTS), nargs="?", help="script to run")
    parser.add_argument(
        "-c",
        "--console",
        action="store_true",
        help="detach the current RDP session to console with tscon before running",
    )
    return parser


def prompt_args(parser):
    choices = ", ".join(sorted(SCRIPTS))
    while True:
        try:
            command = input(f"script ({choices})> ").strip()
        except EOFError:
            raise SystemExit(1)
        if not command:
            continue
        if command in {"help", "-h", "--help", "?"}:
            parser.print_help()
            continue
        try:
            return parser.parse_args(shlex.split(command))
        except SystemExit:
            continue


def main():
    parser = build_parser()
    args = parser.parse_args()
    if args.script is None:
        args = prompt_args(parser)

    if args.console:
        detach_to_console()

    module = importlib.import_module(SCRIPTS[args.script])
    module.main()


if __name__ == "__main__":
    main()
