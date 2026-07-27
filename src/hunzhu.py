import argparse
import time

from common import const, tool

round_count = 0

JINU_BY_LEVEL = {
    1: const.jinu1,
    2: const.jinu2,
    3: const.jinu3,
}

SELECT_ENEMY_OFFSET = (0, -30)


def start(round_no, level):
    tool.log(f"第 {round_no} 轮开始[{level} 级激怒]")
    tool.focus_mofa_haqi_window()
    tool.find_and_click(const.fuwenka)
    tool.find_and_click(JINU_BY_LEVEL[level])
    tool.find_and_clickIfExist(const.weijinu, dis=SELECT_ENEMY_OFFSET)
    tool.find_and_click(const.maichongaoyi)
    tool.log(f"第 {round_no} 轮完成")


def parse_args(argv=None):
    parser = argparse.ArgumentParser(prog="mfhq hunzhu")
    parser.add_argument(
        "--level",
        type=int,
        choices=sorted(JINU_BY_LEVEL),
        default=1,
        help="hunzhu level, decides which jinu image to click",
    )
    return parser.parse_args(argv)


def main(argv=None):
    global round_count

    args = parse_args(argv)
    tool.log(f"魂珠脚本启动[{args.level} 级激怒]")
    while True:
        round_count += 1
        start(round_count, args.level)
        if tool.find_img(const.ditu) is not None:
            tool.press_with_correction('a', 0.5)
            tool.press_with_correction('w', 0.5)
        time.sleep(2)


if __name__ == '__main__':
    main()
