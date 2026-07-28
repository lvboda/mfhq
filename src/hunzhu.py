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
MAP_WAIT_SECONDS = 40
TURN_LEFT_SECONDS = 0.25
FORWARD_SECONDS = 1.0


def start(round_no, level):
    tool.log(f"第 {round_no} 轮开始[{level} 级激怒]")
    tool.focus_mofa_haqi_window()
    tool.find_and_click(const.fuwenka)
    tool.find_and_click(JINU_BY_LEVEL[level])
    tool.find_and_clickIfExist(const.weijinu, dis=SELECT_ENEMY_OFFSET)
    tool.find_and_click(const.maichongaoyi)
    tool.log(f"第 {round_no} 轮完成")


def parse_args(argv=None):
    levels = sorted(JINU_BY_LEVEL)
    parser = argparse.ArgumentParser(prog="mfhq hunzhu")
    parser.add_argument(
        "level",
        type=int,
        choices=levels,
        nargs="?",
        default=None,
        help="hunzhu level, decides which jinu image to click",
    )
    parser.add_argument(
        "-l",
        "-level",
        "--level",
        dest="level_option",
        type=int,
        choices=levels,
        default=None,
        help="same as the positional level",
    )
    args = parser.parse_args(argv)
    if args.level_option is not None:
        args.level = args.level_option
    elif args.level is None:
        args.level = 1
    return args


def main(argv=None):
    global round_count

    args = parse_args(argv)
    tool.log(f"魂珠脚本启动[{args.level} 级激怒]")
    while True:
        round_count += 1
        start(round_count, args.level)
        if tool.wait_img_appear(const.ditu1, MAP_WAIT_SECONDS) is not None:
            tool.log(f"第 {round_count} 轮：检测到地图，走位")
            tool.press_with_correction('a', TURN_LEFT_SECONDS)
            tool.press_with_correction('w', FORWARD_SECONDS)
        else:
            tool.log(f"第 {round_count} 轮：等待地图超时，跳过走位")
        time.sleep(2)


if __name__ == '__main__':
    main()
