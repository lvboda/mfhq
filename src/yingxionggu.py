import atexit
import signal
import threading
import time

from common import const, tool

round_count = 0
ce_patch_attempts = 0
recovery_failures = 0
MAX_CE_PATCH_ATTEMPTS = 2
DISCONNECT_CHECK_SECONDS = 300
ARENA_RESULT_TIMEOUT = 2400
PROCESS_CHECK_SECONDS = 10
RECOVERY_RETRY_SECONDS = 60

recovering = threading.Event()


def game_is_gone():
    if tool.is_mofa_haqi_running():
        return False
    time.sleep(1)
    return not tool.is_mofa_haqi_running()


def ensure_game_ready():
    global ce_patch_attempts, recovery_failures

    if tool.is_mofa_haqi_running():
        return True

    recovering.set()
    try:
        tool.log("魔法哈奇未运行，尝试启动并登录")
        if not tool.start_mofa_haqi():
            recovery_failures += 1
            tool.log(f"启动魔法哈奇失败，连续第 {recovery_failures} 次，等待 {RECOVERY_RETRY_SECONDS} 秒")
            time.sleep(RECOVERY_RETRY_SECONDS)
            return False
        if not tool.login_mofa_haqi():
            recovery_failures += 1
            tool.log(f"登录魔法哈奇失败，连续第 {recovery_failures} 次，等待 {RECOVERY_RETRY_SECONDS} 秒")
            time.sleep(RECOVERY_RETRY_SECONDS)
            return False
        recovery_failures = 0
        ce_patch_attempts = 0
        tool.log("魔法哈奇已恢复，CE 修改次数已重置")
        return True
    finally:
        recovering.clear()


def watch_disconnect():
    while True:
        time.sleep(DISCONNECT_CHECK_SECONDS)
        try:
            if recovering.is_set():
                continue
            if not tool.is_mofa_haqi_running():
                tool.log("守护线程：魔法哈奇进程不在")
                continue
            if tool.find_img(const.diaoxian) is None and tool.find_img(const.denglu) is None:
                continue
            tool.log("守护线程：检测到掉线，关闭游戏进程")
            if not tool.kill_mofa_haqi():
                tool.log("守护线程：关闭游戏进程失败")
        except Exception as e:
            tool.log(f"守护线程异常：{e}")


def cleanup(*_):
    tool.log("清理 CE 进程")
    tool.close_cheat_engine()


def exit_with_cleanup(signum, _frame):
    cleanup()
    raise SystemExit(128 + signum)


def wait_arena_result():
    start_time = time.perf_counter()
    last_process_check = start_time
    while True:
        if tool.find_img(const.saichangchengji) is not None:
            return True
        now = time.perf_counter()
        if now - last_process_check >= PROCESS_CHECK_SECONDS:
            last_process_check = now
            if game_is_gone():
                tool.log("等待比赛成绩期间魔法哈奇进程已不在")
                return False
        if now - start_time >= ARENA_RESULT_TIMEOUT:
            tool.log("等待比赛成绩超时")
            return False
        time.sleep(0.2)


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

    if not run_arena_bag_guard(wait_arena_result):
        return

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
            if not ensure_game_ready():
                continue
            round_count += 1
            start(round_count)
            time.sleep(2)
    finally:
        cleanup()


if __name__ == '__main__':
    main()
