import atexit
import signal
import threading
import time

from common import const, tool

round_count = 0
ce_patch_attempts = 0
MAX_CE_PATCH_ATTEMPTS = 2
DISCONNECT_CHECK_SECONDS = 300
ARENA_RESULT_TIMEOUT = 2400

recovering = threading.Event()


def ensure_game_ready():
    if tool.is_mofa_haqi_running():
        return

    recovering.set()
    try:
        tool.log("魔法哈奇未运行，尝试启动并登录")
        tool.start_mofa_haqi()
        tool.login_mofa_haqi()
    finally:
        recovering.clear()


def watch_disconnect():
    while True:
        time.sleep(DISCONNECT_CHECK_SECONDS)
        if recovering.is_set():
            continue
        if not tool.is_mofa_haqi_running():
            tool.log("守护线程：魔法哈奇进程不在")
            continue
        if tool.find_img(const.diaoxian) is not None:
            tool.log("守护线程：检测到掉线提示，关闭游戏进程")
            tool.kill_mofa_haqi()


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

    tool.log(f"第 {round_no} 轮开始")
    tool.focus_mofa_haqi_window()
    tool.wait_img_appear(const.ditu)
    tool.find_and_click(const.yingxionggu)
    tool.find_and_click(const.jiaruyingxionggu)
    time.sleep(2)

    if tool.find_img(const.yingxionggu10ci) is not None:
        tool.find_and_click(const.queding)
        if ce_patch_attempts < MAX_CE_PATCH_ATTEMPTS:
            ce_patch_attempts += 1
            tool.log(f"第 {round_no} 轮：检测到 10 次提示，第 {ce_patch_attempts} 次启动 CE 修改")
            tool.run_ce_double_patch(50417, -1)
            time.sleep(1)
            tool.focus_mofa_haqi_window()
        else:
            tool.log(f"第 {round_no} 轮：检测到 10 次提示，CE 修改已达最大尝试次数")
        time.sleep(10)
        return

    if tool.find_img(const.shi) is not None:
        tool.log(f"第 {round_no} 轮：购买门票")
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

    run_arena_bag_guard(lambda: tool.wait_img_appear(const.saichangchengji, ARENA_RESULT_TIMEOUT))
    tool.find_and_click(const.fanhuizhucheng)
    tool.find_and_click(const.shi)
    tool.log(f"第 {round_no} 轮完成")


def main():
    tool.log("英雄谷脚本启动")
    atexit.register(cleanup)
    for signum in (signal.SIGINT, signal.SIGTERM):
        signal.signal(signum, exit_with_cleanup)

    ensure_game_ready()

    watcher = threading.Thread(target=watch_disconnect)
    watcher.daemon = True
    watcher.start()

    try:
        while True:
            global round_count
            round_count += 1
            ensure_game_ready()
            start(round_count)
            time.sleep(2)
    finally:
        cleanup()


if __name__ == '__main__':
    main()
