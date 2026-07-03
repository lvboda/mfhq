import argparse
import atexit
import signal
import threading
import time

from common import const, screen, tool

round_count = 0
ce_patch_attempts = {}
MAX_CE_PATCH_ATTEMPTS = 2
runtime_options = None


class RuntimeOptions:
    def __init__(self, split_screen=False, screens=None, regions=None):
        self.split_screen = split_screen
        self.screens = screens or []
        self.regions = regions or {}


def cleanup(*_):
    tool.log("清理 CE 进程")
    tool.close_cheat_engine()


def exit_with_cleanup(signum, _frame):
    cleanup()
    raise SystemExit(128 + signum)


def run_arena_bag_guard(callback):
    stop_event = threading.Event()
    thread = threading.Thread(target=tool.beibao100s, args=(stop_event,))
    thread.daemon = True
    thread.start()
    try:
        return callback()
    finally:
        stop_event.set()
        thread.join(timeout=2)


def start(round_no):
    global ce_patch_attempts

    region_name = tool.get_active_region_name()
    attempt_key = region_name or "default"
    prefix = f"第 {round_no} 轮" if not region_name else f"第 {round_no} 轮[{region_name}]"

    tool.log(f"{prefix}开始")
    if runtime_options is None or not runtime_options.split_screen:
        tool.focus_mofa_haqi_window()
    tool.wait_img_appear(const.ditu)
    tool.find_and_click(const.yingxionggu)
    tool.find_and_click(const.jiaruyingxionggu)
    time.sleep(2)

    if tool.find_img(const.yingxionggu10ci) is not None:
        tool.find_and_click(const.queding)
        attempts = ce_patch_attempts.get(attempt_key, 0)
        if attempts < MAX_CE_PATCH_ATTEMPTS:
            attempts += 1
            ce_patch_attempts[attempt_key] = attempts
            tool.log(f"{prefix}：检测到 10 次提示，第 {attempts} 次启动 CE 修改")
            target_pid = tool.get_active_region_window_pid() if runtime_options and runtime_options.split_screen else None
            tool.run_ce_double_patch(50417, -1, target_pid=target_pid)
            time.sleep(1)
            if runtime_options is None or not runtime_options.split_screen:
                tool.focus_mofa_haqi_window()
        else:
            tool.log(f"{prefix}：检测到 10 次提示，CE 修改已达最大尝试次数")
        time.sleep(10)
        return

    if tool.find_img(const.shi) is not None:
        tool.log(f"{prefix}：购买门票")
        tool.find_and_click(const.fou)
        tool.find_and_click(const.dianjichakan)
        tool.find_and_click_within_region(const.yingxionggumenpiao, const.goumai1)
        tool.find_and_click(const.mashanggoumai1)
        tool.find_and_click(const.cha)
        tool.find_and_click(const.jiaruyingxionggu)

    time.sleep(10)
    if tool.find_img(const.fanhuizhucheng) is None:
        if tool.find_img(const.zhuangbei) is not None:
            tool.press_with_correction('b', 0.5)
        return

    run_arena_bag_guard(lambda: tool.wait_img_appear(const.saichangchengji, 0))
    tool.find_and_click(const.fanhuizhucheng)
    tool.find_and_click(const.shi)
    tool.log(f"{prefix}完成")


def run_selected_screens(round_no):
    if not runtime_options or not runtime_options.split_screen:
        tool.set_active_region()
        start(round_no)
        return

    for screen_key in runtime_options.screens:
        tool.set_active_region(runtime_options.regions[screen_key], screen_key)
        try:
            start(round_no)
        finally:
            tool.set_active_region()


def parse_args(argv=None):
    parser = argparse.ArgumentParser(prog="mfhq yingxionggu")
    parser.add_argument(
        "--screens",
        nargs="+",
        help="enable split-screen planning and select quadrants: lt,rt,lb,rb or 1,2,3,4",
    )
    parser.add_argument(
        "--show-screens",
        action="store_true",
        help="print selected split-screen regions and exit",
    )
    args = parser.parse_args(argv)
    try:
        args.selected_screens = screen.parse_screen_selection(args.screens)
    except ValueError as e:
        parser.error(str(e))
    return args


def build_runtime_options(args):
    selected = args.selected_screens
    split_screen = args.screens is not None or args.show_screens
    regions = screen.get_split_regions() if split_screen else {}
    return RuntimeOptions(split_screen=split_screen, screens=selected, regions=regions)


def log_runtime_options(options):
    if not options.split_screen:
        return
    tool.log(f"分屏模式已启用：{screen.describe_regions(options.regions, options.screens)}")


def main(argv=None):
    global runtime_options
    args = parse_args(argv)
    runtime_options = build_runtime_options(args)

    tool.log("英雄谷脚本启动")
    log_runtime_options(runtime_options)
    if args.show_screens:
        return

    atexit.register(cleanup)
    for signum in (signal.SIGINT, signal.SIGTERM):
        signal.signal(signum, exit_with_cleanup)

    if not tool.is_mofa_haqi_running():
        tool.log("魔法哈奇未运行，尝试启动并登录")
        tool.start_mofa_haqi()
        tool.login_mofa_haqi()

    try:
        while True:
            global round_count
            round_count += 1
            run_selected_screens(round_count)
            time.sleep(2)
    finally:
        cleanup()


if __name__ == '__main__':
    main()
