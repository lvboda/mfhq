import time

from common import const, tool

round_count = 0


def start(round_no):
    tool.log(f"第 {round_no} 轮开始")
    tool.focus_mofa_haqi_window()
    tool.find_and_click(const.fuwenka)
    tool.find_and_click(const.jinu)
    tool.find_and_click(const.maichongaoyi)
    tool.log(f"第 {round_no} 轮完成")


def main():
    global round_count

    tool.log("魂珠脚本启动")
    while True:
        round_count += 1
        start(round_count)
        if tool.find_img(const.ditu) is not None:
            tool.press_with_correction('a', 0.5)
            tool.press_with_correction('w', 0.5)
        time.sleep(2)


if __name__ == '__main__':
    main()
