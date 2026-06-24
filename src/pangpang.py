"""
挂胖胖卡牌自动点击脚本。

逻辑来源于 card_clicker.exe 的反编译整理版：
- 使用代码里的固定区域/卡牌配置
- 将屏幕分成四宫格，在指定区域识别指定卡牌图片
- 识别成功后点击卡牌中心，按配置顺序循环执行
- 按 ESC 结束脚本
"""

import logging
import os
import sys
import threading
import time

import cv2
import numpy as np
import pyautogui
from pynput.keyboard import Controller, Key, Listener

IMG_DIR = "pan_pan_img"
CARD_THRESHOLD = 0.75
MATCH_THRESHOLD = 0.8
RETRY_INTERVAL = 0.5
MAX_WAIT_NEXT_ROUND = 120
MAX_RETRIES = 5

keyboard = Controller()
stop_flag = threading.Event()


def resource_path(path):
    base_path = getattr(sys, "_MEIPASS", os.getcwd())
    candidates = [
        os.path.join(os.getcwd(), path),
        os.path.join(base_path, path),
        os.path.join(os.path.dirname(sys.executable), path) if getattr(sys, "frozen", False) else "",
    ]
    for candidate in candidates:
        if candidate and os.path.exists(candidate):
            return candidate
    return candidates[0]


def on_key_press(key):
    if key == Key.esc:
        print("\n🛑 检测到 ESC 按键，脚本终止...")
        stop_flag.set()
        return False
    return None


def start_key_listener():
    listener = Listener(on_press=on_key_press)
    listener.daemon = True
    listener.start()
    return listener


def check_stop():
    if stop_flag.is_set():
        print("🛑 脚本已终止")
        sys.exit(0)


PANGPANG_CARD_CONFIG = [
    {"region": 1, "card": "潘多拉守护.png", "region_name": "1号"},
    {"region": 2, "card": "雷云风暴（紫）.png", "region_name": "2号"},
    {"region": 3, "card": "风鹰之眼（紫）.png", "region_name": "3号"},
]


def get_card_config():
    logging.info(f"📋 配置: {len(PANGPANG_CARD_CONFIG)} 个区域")
    for item in PANGPANG_CARD_CONFIG:
        logging.info(f"  区域{item['region']}: {item['card']}")
    return list(PANGPANG_CARD_CONFIG)


def get_screen_regions():
    screen_w, screen_h = pyautogui.size()
    half_w = screen_w // 2
    half_h = screen_h // 2
    regions = {
        1: (0, 0, half_w, half_h),
        2: (0, half_h, half_w, half_h),
        3: (half_w, 0, half_w, half_h),
        4: (half_w, half_h, half_w, half_h),
    }
    logging.info(f"屏幕: {screen_w}x{screen_h}")
    return regions


def screenshot_region(region):
    x, y, w, h = region
    img = pyautogui.screenshot(region=(x, y, w, h))
    return cv2.cvtColor(np.array(img), cv2.COLOR_RGB2BGR)


def get_img_path(filename):
    if "/" in filename or "\\" in filename:
        return resource_path(filename)
    return resource_path(os.path.join(IMG_DIR, filename))


def load_template(template_path):
    return cv2.imdecode(np.fromfile(template_path, dtype=np.uint8), cv2.IMREAD_COLOR)


def find_card_in_region(card_name, region_id, regions, threshold=CARD_THRESHOLD):
    if region_id not in regions:
        return False, 0.0

    card_path = get_img_path(card_name)
    if not os.path.exists(card_path):
        return False, 0.0

    template = load_template(card_path)
    if template is None:
        return False, 0.0

    screen_img = screenshot_region(regions[region_id])
    result = cv2.matchTemplate(screen_img, template, cv2.TM_CCOEFF_NORMED)
    _, max_val, _, _ = cv2.minMaxLoc(result)
    return max_val >= threshold, max_val


def find_and_click(template_path, region_id, regions, threshold=MATCH_THRESHOLD):
    if region_id not in regions:
        print(f"[错误] 无效的区域编号: {region_id}")
        return False

    region = regions[region_id]
    rx, ry, _, _ = region
    template = load_template(template_path)
    if template is None:
        print(f"[错误] 无法读取模板图片: {template_path}")
        return False

    th, tw = template.shape[:2]
    screen_img = screenshot_region(region)
    result = cv2.matchTemplate(screen_img, template, cv2.TM_CCOEFF_NORMED)
    _, max_val, _, max_loc = cv2.minMaxLoc(result)
    print(f"[匹配] 模板={template_path}, 区域={region_id}, 最大匹配度={max_val:.4f}")

    if max_val >= threshold:
        center_x = rx + max_loc[0] + tw // 2
        center_y = ry + max_loc[1] + th // 2
        print(f"[点击] 找到目标! 屏幕坐标=({center_x}, {center_y})")
        pyautogui.click(center_x, center_y)
        return True

    print(f"[未找到] 匹配度 {max_val:.4f} < 阈值 {threshold}")
    return False


def click_card_in_region(card_name, region_id, regions, threshold=CARD_THRESHOLD):
    return find_and_click(get_img_path(card_name), region_id, regions, threshold=threshold)


class StatsManager:
    def __init__(self):
        self.round_count = 0
        self.success_count = 0
        self.fail_count = 0
        self.start_time = time.time()

    def record_round(self, success):
        self.round_count += 1
        if success:
            self.success_count += 1
        else:
            self.fail_count += 1

    def get_summary(self):
        return {
            "total_rounds": self.round_count,
            "success": self.success_count,
            "fail": self.fail_count,
            "total_time": time.time() - self.start_time,
        }


def wait_for_card(card_name, region_id, regions, timeout=MAX_WAIT_NEXT_ROUND):
    start = time.time()
    while time.time() - start < timeout:
        check_stop()
        found, _ = find_card_in_region(card_name, region_id, regions)
        if found:
            return True
        time.sleep(0.5)
    return False


def run_card_clicker(config, regions, stats):
    first_card = None
    for item in config:
        if item["region"] == 1:
            first_card = item["card"]
            break

    if first_card:
        logging.info(f"等待1号区域「{first_card}」出现...")
        if not wait_for_card(first_card, 1, regions):
            logging.error(f"❌ 等待{MAX_WAIT_NEXT_ROUND}秒未检测到1号区域「{first_card}」，脚本停止")
            return

    while True:
        check_stop()
        stats.round_count += 1
        logging.info(f"\n========== 第 {stats.round_count} 轮 ==========")

        round_success = True
        for item in config:
            check_stop()
            region_id = item["region"]
            card_name = item["card"]
            logging.info(f"区域{region_id}: {card_name}")

            success = False
            for attempt in range(MAX_RETRIES):
                check_stop()
                found, score = find_card_in_region(card_name, region_id, regions)
                logging.info(f"  匹配度: {score:.4f}")

                if found:
                    click_card_in_region(card_name, region_id, regions)
                    time.sleep(RETRY_INTERVAL)
                    still_exists, check_score = find_card_in_region(card_name, region_id, regions)
                    if not still_exists:
                        logging.info("  ✅ 成功")
                        success = True
                        break
                    if attempt < MAX_RETRIES - 1:
                        logging.info(f"  卡牌未消失(匹配度: {check_score:.4f})，重试 {attempt + 1}/{MAX_RETRIES}")
                elif attempt < MAX_RETRIES - 1:
                    logging.info(f"  未找到(匹配度: {score:.4f})，重试 {attempt + 1}/{MAX_RETRIES}")

                time.sleep(RETRY_INTERVAL)

            if not success:
                round_success = False
                logging.warning(f"  ⚠️ 跳过: 区域{region_id}「{card_name}」{MAX_RETRIES}次未成功")

        if round_success:
            stats.success_count += 1
        else:
            stats.fail_count += 1
        logging.info(f"第 {stats.round_count} 轮完成")

        if first_card:
            logging.info(f"等待1号区域「{first_card}」出现...")
            if not wait_for_card(first_card, 1, regions):
                logging.warning(f"⚠️ 等待{MAX_WAIT_NEXT_ROUND}秒未检测到1号区域「{first_card}」，跳过等待")


def main():
    print("==================================================")
    print("卡牌自动点击脚本")
    print("==================================================")
    logging.basicConfig(level=logging.INFO, format="%(message)s")
    pyautogui.FAILSAFE = True
    pyautogui.PAUSE = 0.3
    start_key_listener()

    config = get_card_config()

    logging.info("\n3秒后开始，请切换到游戏窗口...")
    time.sleep(3)
    regions = get_screen_regions()
    stats = StatsManager()

    try:
        run_card_clicker(config, regions, stats)
    except KeyboardInterrupt:
        logging.info("\n用户中断脚本")
    except SystemExit:
        logging.info("\n脚本终止")
    finally:
        summary = stats.get_summary()
        logging.info(
            f"\n统计: {summary['total_rounds']}轮, 成功{summary['success']}, "
            f"失败{summary['fail']}, 耗时{summary['total_time']:.1f}秒"
        )


if __name__ == "__main__":
    main()
