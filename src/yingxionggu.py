import atexit
import signal
import threading
import time

from common import const, tool

round_count = 0


def cleanup(*_):
    tool.log("清理 CE 进程")
    tool.close_cheat_engine()


def exit_with_cleanup(signum, _frame):
    cleanup()
    raise SystemExit(128 + signum)


def start(round_no):
    tool.log(f"第 {round_no} 轮开始")
    tool.wait_img_appear(const.ditu)
    tool.find_and_click(const.yingxionggu)
    tool.find_and_click(const.jiaruyingxionggu)
    time.sleep(2)

    if tool.find_img(const.yingxionggu10ci) is not None:
        tool.log(f"第 {round_no} 轮：检测到 10 次提示，启动 CE 修改")
        tool.find_and_click(const.queding)
        tool.run_ce_double_patch(50417, -1)
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

    tool.wait_img_appear(const.saichangchengji, 0)
    tool.find_and_click(const.fanhuizhucheng)
    tool.find_and_click(const.shi)
    tool.log(f"第 {round_no} 轮完成")


def main():
    tool.log("英雄谷脚本启动")
    atexit.register(cleanup)
    for signum in (signal.SIGINT, signal.SIGTERM):
        signal.signal(signum, exit_with_cleanup)

    if not tool.is_mofa_haqi_running():
        tool.log("魔法哈奇未运行，尝试启动并登录")
        tool.start_mofa_haqi()
        tool.login_mofa_haqi()

    thread = threading.Thread(target=tool.beibao100s)
    thread.daemon = True
    thread.start()
    try:
        while True:
            global round_count
            round_count += 1
            start(round_count)
            time.sleep(2)
    finally:
        cleanup()


if __name__ == '__main__':
    main()
