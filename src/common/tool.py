import ctypes
import datetime
import json
import os
import random
import subprocess
import sys
import cv2
import pyautogui
import win32con
import win32gui
from playsound import playsound
from pynput.keyboard import Controller, Key
import numpy as np
import time
import threading
from common import config
keyboard = Controller()
user32 = ctypes.windll.user32
_missing_image_logs = set()


def log(message):
    text = f"[{datetime.datetime.now().strftime('%Y-%m-%d %H:%M:%S')}] {message}"
    print(text, flush=True)
    try:
        base_dir = os.path.dirname(sys.executable) if getattr(sys, 'frozen', False) else os.getcwd()
        with open(os.path.join(base_dir, 'yingxionggu.log'), 'a', encoding='utf-8') as f:
            f.write(text + '\n')
    except Exception:
        pass


def runtime_dir():
    return os.path.dirname(sys.executable) if getattr(sys, 'frozen', False) else os.getcwd()


def ce_pid_file():
    return os.path.join(runtime_dir(), config.CE_PID_FILE)


def write_ce_pid(pid, exe_path):
    try:
        with open(ce_pid_file(), 'w', encoding='utf-8') as f:
            json.dump({'pid': pid, 'exe_path': os.path.abspath(exe_path)}, f)
    except Exception as e:
        log(f'write ce pid failed: {e}')


def read_ce_process_info():
    try:
        with open(ce_pid_file(), 'r', encoding='utf-8') as f:
            text = f.read().strip()
        try:
            data = json.loads(text)
            return int(data.get('pid')), os.path.abspath(data.get('exe_path', ''))
        except Exception:
            return int(text), ''
    except Exception:
        return None, ''


def read_ce_pid():
    pid, _ = read_ce_process_info()
    return pid


def clear_ce_pid():
    try:
        os.remove(ce_pid_file())
    except FileNotFoundError:
        pass
    except Exception:
        pass


def is_pid_running(pid):
    if not pid:
        return False
    try:
        output = subprocess.check_output(
            ['tasklist', '/FI', f'PID eq {pid}', '/FO', 'CSV', '/NH'],
            text=True,
            errors='ignore',
            creationflags=getattr(subprocess, 'CREATE_NO_WINDOW', 0),
        )
    except Exception:
        return False
    return str(pid) in output


def get_process_exe_path(pid):
    if not pid:
        return ''
    try:
        output = subprocess.check_output(
            ['wmic', 'process', 'where', f'ProcessId={pid}', 'get', 'ExecutablePath', '/value'],
            text=True,
            errors='ignore',
            creationflags=getattr(subprocess, 'CREATE_NO_WINDOW', 0),
        )
    except Exception:
        return ''
    for line in output.splitlines():
        if line.lower().startswith('executablepath='):
            return os.path.abspath(line.split('=', 1)[1].strip())
    return ''


def close_cheat_engine():
    pid, expected_exe = read_ce_process_info()
    if not is_pid_running(pid):
        clear_ce_pid()
        return False
    if not expected_exe:
        log(f'skip closing Cheat Engine pid={pid}: missing recorded exe path')
        return False
    current_exe = get_process_exe_path(pid)
    if os.path.normcase(current_exe) != os.path.normcase(expected_exe):
        log(f'skip closing Cheat Engine pid={pid}: exe path mismatch')
        clear_ce_pid()
        return False

    subprocess.run(
        ['taskkill', '/F', '/T', '/PID', str(pid)],
        stdout=subprocess.DEVNULL,
        stderr=subprocess.DEVNULL,
        creationflags=getattr(subprocess, 'CREATE_NO_WINDOW', 0),
    )
    clear_ce_pid()
    log(f'closed Cheat Engine pid={pid}')
    return True

def turn_half():
    press_with_correction('a', 1.175)
    time.sleep(0.5)


def turn_one():
    press_with_correction('a', 2.35)
    time.sleep(0.5)


def turn_left(period=0.587):
    press_with_correction('a', period)
    time.sleep(0.5)


def turn_right(period=0.587):
    press_with_correction('d', period)
    time.sleep(0.5)


def forward(period):
    press_with_correction('w', period)
    time.sleep(0.5)


def goBack(period):
    press_with_correction('s', period)
    time.sleep(0.5)


def space(period=0.5):
    press_with_correction(Key.space, period)
    time.sleep(0.5)


def keyIn(keys):
    for char in keys:
        press_with_correction(char, 0.1)


def select_all_and_delete(period=0.5):
    '''
    模拟 Ctrl+A 进行全选，然后按下 Space 键删除所选内容。
    :param period: 每个按键按下的持续时间
    '''
    keyboard.press(Key.ctrl_l)
    time.sleep(0.5)
    keyboard.press('a')
    time.sleep(0.5)
    keyboard.release(Key.ctrl_l)
    time.sleep(0.5)
    keyboard.release('a')
    time.sleep(0.5)
    press_with_correction(Key.backspace, period)


def press_with_correction(key, duration):
    keyboard.press(key)
    start_time = time.perf_counter()
    while time.perf_counter() - start_time < duration:
        pass
    keyboard.release(key)


def get_all_windows(key):
    '''
    获取当前系统中所有窗口的标题及句柄。
    :param key: 需要匹配的窗口标题关键字
    :return: 一个包含符合条件的窗口句柄的列表
    '''
    handles = []

    def enum_windows_callback(hwnd, _):
        if win32gui.IsWindowVisible(hwnd):
            title = win32gui.GetWindowText(hwnd).strip()
            if title and key in title:
                handles.append(hwnd)
        return True

    win32gui.EnumWindows(enum_windows_callback, None)
    return handles


def get_new_title(original_handles, key):
    '''
    通过关键字获取新的列表。
    :param original_handles: 初始句柄列表
    :param key: 需要匹配的窗口标题关键字
    :return: 新出现窗口的句柄列表
    '''
    nowhandle = get_all_windows(key)
    new_handles = list(set(nowhandle) - set(original_handles))
    return new_handles


def check_window_titles(keyword, *handles):
    '''
    检查给定句柄列表中是否每个窗口标题都包含关键字
    '''
    for handle in handles:
        if handle:
            window_title = win32gui.GetWindowText(handle)
            if keyword in window_title:
                continue
            print(f'''句柄 {handle} 的窗口标题不包含关键字: {keyword}''')
            handles
            return False
        print(f'''句柄 {handle} 无效''')
        return False
    print(f'''所有句柄的窗口标题都包含关键字: {keyword}''')
    return True


def get_screen_dist():
    (screen_width, screen_height) = pyautogui.size()
    left_top = (10, 10)
    right_top = (screen_width - 10, 10)
    left_bottom = (10, screen_height - 70)
    right_bottom = (screen_width - 10, screen_height - 70)
    return (left_top, right_top, left_bottom, right_bottom)


def show_window(handle):
    user32 = ctypes.windll.user32
    user32.ShowWindow(handle, 9)
    time.sleep(0.5)


def minimize_window(handle):
    user32 = ctypes.windll.user32
    user32.ShowWindow(handle, 6)
    time.sleep(0.5)


def maximize_window(handle):
    user32 = ctypes.windll.user32
    user32.ShowWindow(handle, 3)
    time.sleep(0.5)


def set_window_pos(handle, x, y):
    win32gui.SetWindowPos(handle, 0, x, y, -1, -1, win32con.SWP_NOSIZE | win32con.SWP_NOZORDER)
    time.sleep(0.5)

import ctypes
WM_ACTIVATE = 6
WA_ACTIVE = 1

def activate_window(handle):
    user32.PostMessageW(handle, WM_ACTIVATE, WA_ACTIVE, 0)


def switch_window(handle):
    user32.SetForegroundWindow(handle)
    time.sleep(0.5)
    activate_window(handle)


def get_handle(position):
    return win32gui.WindowFromPoint(position)


def op_handle(handle, callback, *args, **kwargs):
    switch_window(handle)
    return callback(*args, **kwargs)


def get_mource_position(period=0.5):
    time.sleep(period)
    (x, y) = pyautogui.position()
    print((x, y))
    return (x, y)


def click(position):
    if position is not None:
        pyautogui.click(position[0], position[1])


def click_right(position):
    if position is not None:
        pyautogui.moveTo(position)
        pyautogui.mouseDown(button='right')
        time.sleep(0.1)
        pyautogui.mouseUp(button='right')


def read_template_image(image_path):
    template = cv2.imread(image_path, cv2.IMREAD_UNCHANGED)
    if template is None:
        if image_path not in _missing_image_logs:
            log(f'template image not found or unreadable: {image_path}')
            _missing_image_logs.add(image_path)
        return None
    if len(template.shape) == 2:
        return cv2.cvtColor(template, cv2.COLOR_GRAY2BGR)
    if template.shape[2] == 4:
        return cv2.cvtColor(template, cv2.COLOR_BGRA2BGR)
    return template


def find_img_within_region(base_image_path, target_image_path):
    base_image = read_template_image(base_image_path)
    if base_image is None:
        return None
    screenshot = pyautogui.screenshot()
    screenshot_cv = cv2.cvtColor(np.array(screenshot), cv2.COLOR_RGB2BGR)
    result = cv2.matchTemplate(screenshot_cv, base_image, cv2.TM_CCOEFF_NORMED)
    (min_val, max_val, min_loc, max_loc) = cv2.minMaxLoc(result)
    if max_val > 0.8:
        base_image_y = max_loc[1]
        base_image_x = max_loc[0]
        base_image_region = screenshot.crop((base_image_x, base_image_y, base_image_x + base_image.shape[1], base_image_y + base_image.shape[0]))
        base_image_region_cv = cv2.cvtColor(np.array(base_image_region), cv2.COLOR_RGB2BGR)
        target_image = read_template_image(target_image_path)
        if target_image is None:
            return None
        result_within_region = cv2.matchTemplate(base_image_region_cv, target_image, cv2.TM_CCOEFF_NORMED)
        (min_val_within_region, max_val_within_region, min_loc_within_region, max_loc_within_region) = cv2.minMaxLoc(result_within_region)
        if max_val_within_region > 0.8:
            target_height = target_image.shape[0]
            target_width = target_image.shape[1]
            target_x = base_image_x + max_loc_within_region[0] + target_width // 2
            target_y = base_image_y + max_loc_within_region[1] + target_height // 2
            return (target_x, target_y)


def find_and_click_within_region(base_image_path, target_image_path):
    position = find_img_within_region(base_image_path, target_image_path)
    time.sleep(random.uniform(0, 0.2))
    while position is None:
        time.sleep(random.uniform(0.2, 0.4))
        position = find_img_within_region(base_image_path, target_image_path)
    original_position = pyautogui.position()
    click(position)
    print('点击成功')
    pyautogui.moveTo(*original_position)
    time.sleep(1)


def find_and_click_within_region_ifExist(base_image_path, target_image_path):
    position = find_img_within_region(base_image_path, target_image_path)
    if position is None:
        return False
    click(position)
    time.sleep(random.uniform(0.2, 0.4))
    return True


def right_click_drag(x_distance):
    '''
    按住鼠标右键并横向移动指定距离。

    :param x_distance: 横向移动的距离（正数向右，负数向左）
    '''
    (current_x, current_y) = pyautogui.position()
    pyautogui.mouseDown(button = 'right')
    time.sleep(0.1)
    pyautogui.moveRel(x_distance, 0, duration = 10)
    pyautogui.mouseUp(button = 'right')
    print(f'''鼠标从 ({current_x}, {current_y}) 横向移动了 {x_distance} 像素。''')


def find_img(image_path, threshold=0.8):
    screenshot = pyautogui.screenshot()
    screenshot = cv2.cvtColor(np.array(screenshot), cv2.COLOR_RGB2BGR)
    template = read_template_image(image_path)
    if template is None:
        return None
    result = cv2.matchTemplate(screenshot, template, cv2.TM_CCOEFF_NORMED)
    (_, max_val, _, max_loc) = cv2.minMaxLoc(result)
    if max_val > threshold:
        center_x = max_loc[0] + template.shape[1] // 2
        center_y = max_loc[1] + template.shape[0] // 2
        return (center_x, center_y)


def find_img_in_roi(base_image_path, target_image_path, roi_size, threshold=0.8):
    base_center = find_img(base_image_path, threshold)


def find_and_sort_image_positions(image_path, threshold=0.8):
    '''
    查找屏幕中所有与指定图片匹配的位置，并确保去重后按从左到右、从上到下的顺序返回。

    :param image_path: 模板图片的路径
    :param threshold: 匹配阈值（0~1）
    :return: 排序后的唯一匹配位置的中心坐标列表
    '''
    pass


def find_and_click_r(image_path, duration=40):
    position = find_img(image_path)
    start_time = time.perf_counter()
    time.sleep(random.uniform(0, 0.2))
    while position is None:
        time.sleep(random.uniform(0.2, 0.4))
        position = find_img(image_path)
        if duration != 0 and time.perf_counter() - start_time >= duration:
            print('卡片{}没有找到'.format(image_path))
            return False
    click_right(position)
    pyautogui.moveTo(200, 200, duration=0.2)
    return True


def find_and_click(image_path, duration=40):
    position = find_img(image_path)
    start_time = time.perf_counter()
    time.sleep(random.uniform(0, 0.2))
    while position is None:
        time.sleep(random.uniform(0.2, 0.4))
        position = find_img(image_path)
        if duration != 0 and time.perf_counter() - start_time >= duration:
            print('卡片{}没有找到'.format(image_path))
            return False
    click(position)
    pyautogui.moveTo(200, 200, duration=0.2)
    return True


def find_and_click_dis(image_path, dis, duration=40):
    position = find_img(image_path)
    start_time = time.perf_counter()
    time.sleep(random.uniform(0, 0.2))


def find_and_click_num(image_path, number, duration=40):
    '''
    如果屏幕中存在多个，点击屏幕中的第number个，从0开始
    从左到右，从上到下
    :param duration:
    :param image_path:
    :param number:
    :return:
    '''
    positions = find_and_sort_image_positions(image_path)
    start_time = time.perf_counter()


def find_and_click_All_condition():
    '''
    选择满足条件的一堆图片中所有满足条件的
    :return:
    '''
    pass


def find_and_click_doble(image_path, offset=(0, 0)):
    position = find_img(image_path)
    times = 0
    while position is None and times < 2:
        time.sleep(random.uniform(0.2, 0.4))
        position = find_img(image_path)
        times += 1
    if position is None:
        return False
    click((position[0] + offset[0], position[1] + offset[1]))
    time.sleep(0.2)
    click((position[0] + offset[0], position[1] + offset[1]))
    return True


def find_and_click_two_or(image_path, image_path2):
    position = find_img(image_path)
    position2 = find_img(image_path2)
    while position is None and position2 is None:
        time.sleep(random.uniform(0.2, 0.4))
        position = find_img(image_path)
        position2 = find_img(image_path2)
    click(position or position2)
    return True


def find_and_click_two_or_IfExist(image_path, image_path2):
    position = find_img(image_path)
    position2 = find_img(image_path2)
    if position is not None or position2 is not None:
        click(position or position2)
        return True
    return False


def find_and_clickIfExist(image_path, threshold=0.8, dis=(0, 0)):
    position = find_img(image_path, threshold)
    if position is not None:
        click(tuple(x + y for x, y in zip(position, dis)))
        time.sleep(1)
        return True
    return False


def wait_img_appear(image_path, duration=40):
    position = find_img(image_path)
    start_time = time.perf_counter()
    while position is None:
        time.sleep(0.2)
        position = find_img(image_path)
        if duration != 0 and time.perf_counter() - start_time >= duration:
            return None
    return position


def wait_img_appear_any(image_paths, duration=40):
    position = None
    start_time = time.perf_counter()
    while position is None:
        for img in image_paths:
            position = find_img(img)
            time.sleep(0.1)
            if position is not None:
                return True
            if duration != 0 and time.perf_counter() - start_time >= duration:
                return False
    return None


def wait_img_disappear(image_path, duration=40):
    position = find_img(image_path)
    start_time = time.perf_counter()
    while position is not None:
        time.sleep(0.2)
        position = find_img(image_path)
        if duration != 0 and time.perf_counter() - start_time >= duration:
            return False
    return True


def find_and_move(image_path):
    position = find_img(image_path)
    if position is not None:
        pyautogui.moveTo(position)
        return True
    return False


def move(position):
    pyautogui.moveTo(position)


def play_sound(path):
    
    try:
        playsound(path)
        # return None
    except Exception:
        e = None
        print(f'''An error occurred while playing sound: {e}''')
        e = None
        del e
        # return None
        e = None
        del e



def op_handle(handle, callback, *args, **kwargs):
    switch_window(handle)
    return callback(*args, **kwargs)


def op_handle_exception(handle, callback, *args, **kwargs):
    switch_window(handle)
    call = callback(*args, **kwargs)
    if call is None or call is False:
        raise ValueError(f'执行函数{callback.__name__}, 参数{args}失败')


def asyncDoSomeThing(callback, *args, **kwargs):
    thread = threading.Thread(target = callback, args = args, kwargs = kwargs)
    thread.start()
    return thread


def get_file_path(path):
    base_path = getattr(sys, '_MEIPASS', None)
    if base_path:
        exe_dir = os.path.dirname(sys.executable)
        candidates = [
            os.path.join(exe_dir, path),
            os.path.join(os.getcwd(), path),
            os.path.join(base_path, path),
        ]
        for image_path in candidates:
            if os.path.exists(image_path):
                return os.path.abspath(image_path)
        return os.path.abspath(candidates[0])

    candidates = [
        os.path.join(os.getcwd(), path),
        os.path.join(os.getcwd(), 'shared', path),
        os.path.join(os.path.dirname(__file__), path),
        os.path.join(os.path.dirname(__file__), '..', 'shared', path),
    ]
    for image_path in candidates:
        if os.path.exists(image_path):
            return os.path.abspath(image_path)
    return os.path.abspath(candidates[0])





def is_mofa_haqi_running():
    try:
        output = subprocess.check_output(['tasklist', '/FO', 'CSV', '/NH'], text=True, errors='ignore')
    except Exception:
        return False
    output = output.lower()
    return any(name.lower() in output for name in config.MOFA_HAQI_PROCESS_NAMES)



def focus_mofa_haqi_window():
    for key in config.MOFA_HAQI_WINDOW_KEYS:
        handles = get_all_windows(key)
        if handles:
            show_window(handles[0])
            switch_window(handles[0])
            time.sleep(0.5)
            return True
    return False

def kill_mofa_haqi():
    killed = False
    for name in config.MOFA_HAQI_PROCESS_NAMES:
        try:
            result = subprocess.run(
                ['taskkill', '/F', '/IM', name],
                capture_output=True,
                text=True,
                errors='ignore',
                creationflags=getattr(subprocess, 'CREATE_NO_WINDOW', 0),
            )
        except Exception as e:
            log(f'taskkill {name} failed: {e}')
            continue
        if result.returncode == 0:
            killed = True
        elif 'not found' not in (result.stderr or '').lower():
            log(f'taskkill {name} returned {result.returncode}: {(result.stderr or "").strip()}')
    return killed


def start_mofa_haqi(game_path=None):
    if is_mofa_haqi_running():
        return True

    game_path = game_path or os.environ.get(config.MOFA_HAQI_PATH_ENV) or config.MOFA_HAQI_DEFAULT_PATH
    if not game_path:
        print(f'魔法哈奇未启动，且未设置 {config.MOFA_HAQI_PATH_ENV}')
        return False

    try:
        os.startfile(game_path)
    except Exception as e:
        print(f'启动魔法哈奇失败: {e}')
        return False

    for _ in range(60):
        if is_mofa_haqi_running():
            return True
        time.sleep(1)
    return False


def login_mofa_haqi(timeout=120):
    from common import const
    start_time = time.perf_counter()

    if find_img(const.ditu) is not None:
        return True

    find_and_click(const.jinruyouxi)
    time.sleep(2)
    focus_mofa_haqi_window()
    find_and_click(const.denglu)
    find_and_click(const.jinruyouxi2)
    find_and_click(const.paopao)

    while time.perf_counter() - start_time < timeout:
        find_and_clickIfExist(const.cha, 0.8)
        if find_img(const.ditu) is not None:
            return True
        time.sleep(1)

    print('登录魔法哈奇超时')
    return False


def run_ce_double_patch(scan_value, write_value):
    ce_candidates = [
        os.environ.get(config.CE_DIR_ENV),
        get_file_path(config.CE_BUNDLED_DIR),
        config.CE_DIR,
    ]
    ce_dir = None
    ce_exe = None
    ce_exe_names = [
        'cheatengine-x86_64.exe',
        'Cheat Engine.exe',
        'cheatengine-i386.exe',
    ]

    for candidate in ce_candidates:
        if not candidate:
            continue
        candidate = os.path.abspath(candidate)
        if os.path.isfile(candidate):
            ce_dir = os.path.dirname(candidate)
            ce_exe = candidate
            break
        if os.path.isdir(candidate):
            ce_dir = candidate
            ce_exe = next((os.path.join(ce_dir, name) for name in ce_exe_names if os.path.exists(os.path.join(ce_dir, name))), None)
            if ce_exe is None:
                exe_files = [os.path.join(ce_dir, name) for name in os.listdir(ce_dir) if name.lower().endswith('.exe')]
                ce_exe = exe_files[0] if len(exe_files) == 1 else None
            if ce_exe is not None:
                break

    if ce_dir is None or ce_exe is None:
        log('Cheat Engine not found. Put it in src/common/resources or set {}'.format(config.CE_DIR_ENV))
        return False

    close_cheat_engine()
    python_log_path = os.path.join(runtime_dir(), 'mfhq_ce_patch.log')
    try:
        with open(python_log_path, 'a', encoding='utf-8') as f:
            f.write(f"[{datetime.datetime.now().strftime('%Y-%m-%d %H:%M:%S')}] preparing CE patch with {ce_exe}\n")
    except Exception:
        pass

    autorun_dir = os.path.join(ce_dir, 'autorun')
    os.makedirs(autorun_dir, exist_ok=True)
    for filename in os.listdir(autorun_dir):
        if filename.startswith('mfhq_patch_') and filename.endswith('.lua'):
            try:
                os.remove(os.path.join(autorun_dir, filename))
            except OSError:
                pass
    script_name = f"mfhq_patch_{str(scan_value).replace('-', 'neg').replace('.', '_')}_to_{str(write_value).replace('-', 'neg').replace('.', '_')}.lua"
    script_path = os.path.join(autorun_dir, script_name)
    lua_script_path = script_path.replace('\\', '\\\\')
    lua_log_path = python_log_path.replace('\\', '\\\\')

    lua_script = """local processName = \"{process_name}\"
local processKeyword = \"{process_keyword}\"
local scanValue = \"{scan_value}\"
local writeValue = {write_value}
local scriptPath = [[{script_path}]]
local logPath = [[{log_path}]]

local function log(message)
  local f = io.open(logPath, "a")
  if f then
    f:write(os.date("[%Y-%m-%d %H:%M:%S] ") .. message .. "\\n")
    f:close()
  end
end

local function findTargetPid()
  local okPid, pid = pcall(getProcessIDFromProcessName, processName)
  if okPid and pid and pid ~= 0 then
    log("found exact process: " .. processName .. " pid=" .. tostring(pid))
    return pid
  end

  local list = createStringlist()
  getProcesslist(list)
  local keyword = string.lower(processKeyword)
  for i = 0, list.Count - 1 do
    local entry = list[i]
    if entry and string.find(string.lower(entry), keyword, 1, true) then
      local hexPid = string.sub(entry, 1, 8)
      local matchedPid = tonumber(hexPid, 16)
      list.destroy()
      log("found keyword process: " .. entry .. " pid=" .. tostring(matchedPid))
      return matchedPid
    end
  end
  list.destroy()
  return nil
end

local function runPatch()
  log("patch script started")
  local ok, err = pcall(function()
    local opened = false
    for i = 1, 10 do
      local pid = findTargetPid()
      if pid and pid ~= 0 then
        openProcess(pid)
        opened = true
        break
      end
      sleep(1000)
    end

    if not opened then
      log("failed to open process by keyword: " .. processKeyword)
      return
    end

    local ms = createMemScan()
    ms.firstScan(
      soExactValue,
      vtDouble,
      rtRounded,
      scanValue,
      \"\",
      \"00000000\",
      \"7fffffff\",
      \"\",
      fsmNotAligned,
      \"\",
      false,
      false,
      false,
      false
    )
    ms.waitTillDone()

    local fl = createFoundList(ms)
    fl.initialize()

    local count = tonumber(fl.Count) or 0
    for i = 0, count - 1 do
      writeDouble(fl.Address[i], writeValue)
    end

    log("patched " .. count .. " address(es): " .. scanValue .. " -> " .. tostring(writeValue))
    fl.destroy()
    ms.destroy()
  end)

  if not ok then
    log("patch error: " .. tostring(err))
  end
  pcall(function() os.remove(scriptPath) end)
end

local timer = createTimer(nil, false)
timer.Interval = 3000
timer.OnTimer = function(t)
  t.destroy()
  runPatch()
end
timer.Enabled = true
""".format(
        process_name=config.CE_TARGET_PROCESS,
        process_keyword='paraengineclient',
        scan_value=scan_value,
        write_value=write_value,
        script_path=lua_script_path,
        log_path=lua_log_path,
    )

    with open(script_path, 'w', encoding='utf-8') as f:
        f.write(lua_script)
    try:
        with open(python_log_path, 'a', encoding='utf-8') as f:
            f.write(f"[{datetime.datetime.now().strftime('%Y-%m-%d %H:%M:%S')}] wrote CE autorun script: {script_path}\n")
    except Exception:
        pass
    process = subprocess.Popen([ce_exe], cwd=ce_dir)
    write_ce_pid(process.pid, ce_exe)
    return True

def wait_until(target_hour):
    '''等待到指定时间'''
    now = datetime.datetime.now()
    target_time = now.replace(hour = target_hour, minute = 0, second = 0, microsecond = 0)
    if now >= target_time:
        return None
    delta = target_time - now
    time.sleep(delta.total_seconds())


def shanchuka(images):
    all_false = False
    if not all_false:
        all_false = True
        for i in range(len(images)):
            result = find_and_click_r(images[i], 1)
            if not result:
                continue
            all_false = False
        if not all_false:
            return shanchuka(images)
    return True


def buchongjingli():
    jinglitiao = get_file_path('photo/jingli.jpg')
    shi1 = get_file_path('photo/shi.jpg')
    shi2 = get_file_path('photo/shi2.jpg')
    if find_and_clickIfExist(jinglitiao, 0.97):
        find_and_click(shi1)
        find_and_click(shi2)
        print('补充精力成功')
        return None


def huan_chong_wu(image_path):
    chongwuka = get_file_path('photo/chongwuka.jpg')
    huanchongwu = get_file_path('photo/huanchongwu.jpg')
    chongwu = image_path
    find_and_click(chongwuka)
    find_and_click(huanchongwu)
    find_and_click(chongwu)


def chongwujineng(image_path, duration=40):
    chongwuka = get_file_path('photo/chongwuka.jpg')
    jineng = image_path
    if find_and_click(chongwuka, duration):
        return find_and_click(jineng)


def close_zidonggensui():
    zidonggensui = get_file_path('photo/zidonggensui.jpg')
    fou = get_file_path('photo/fou.jpg')
    while True:
        time.sleep(0.5)
        if find_img(zidonggensui) is not None:
            find_and_clickIfExist(fou)


def close_jiaruduiwu():
    jiaruduiwu = get_file_path('photo/jiaruduiwu.jpg')
    fou = get_file_path('photo/fou.jpg')
    while True:
        time.sleep(0.5)
        if find_img(jiaruduiwu) is not None:
            find_and_clickIfExist(fou)


def judge_master_dis(jineng, threshold=0.8):
    zhugong = get_file_path('photo/zhugong.jpg')
    return find_img_in_roi(jineng, zhugong, (200, 200), threshold) is not None


def taopao(handle):
    switch_window(handle)
    ditu = get_file_path('photo/ditu.jpg')
    jishouhang = get_file_path('photo/jishou.jpg')
    taopao = get_file_path('photo/taopao.jpg')
    taopao2 = get_file_path('photo/taopao2.jpg')
    fou = get_file_path('photo/fou.jpg')
    quxiao = get_file_path('photo/quxiao.jpg')
    cha = get_file_path('photo/cha.jpg')
    chongxinxuanpai = get_file_path('photo/chongxinxuanpai.jpg')
    print('查看是否可重开...')
    if find_img(ditu) or find_img(jishouhang):
        return None
    find_and_click_two_or_IfExist(taopao, taopao2)
    find_and_clickIfExist(fou)
    find_and_clickIfExist(quxiao)
    find_and_clickIfExist(cha)
    find_and_clickIfExist(chongxinxuanpai)
    time.sleep(random.uniform(0.2, 0.4))


def gofuben_3(hand1, hand2, hand3, fubenName):
    fuben = get_file_path('photo/fuben.png')
    yongshidating = get_file_path('photo/yongshidating.png')
    createTeam = get_file_path('photo/createTeam.jpg')
    nextPage = get_file_path('photo/nextPage.jpg')
    create = get_file_path('photo/create.jpg')
    ok = get_file_path('photo/ok.jpg')
    queding = get_file_path('photo/queding.jpg')
    go = get_file_path('photo/go.jpg')
    op_handle(hand1, find_and_click, fuben)
    op_handle(hand1, find_and_click, yongshidating)
    op_handle(hand1, find_and_click, createTeam)
    op_handle(hand1, wait_img_appear, nextPage)
    if op_handle(hand1, find_and_clickIfExist, fubenName) is False:
        op_handle(hand1, find_and_click, nextPage)
        op_handle(hand1, find_and_click, fubenName)
    op_handle(hand1, find_and_click, create)
    op_handle(hand1, find_and_click, ok)
    op_handle(hand2, find_and_click, queding)
    op_handle(hand3, find_and_click, queding)
    op_handle(hand1, find_and_click, go)


def yiqiganxiaoguai(hand1, hand2, hand3):
    ditu = get_file_path('photo/ditu.jpg')
    daliyuan = get_file_path('photo/daliyuan.jpg')
    shijieditu = get_file_path('photo/shijieditu.jpg')
    haqidao = get_file_path('photo/haqidao.jpg')
    hands = [
        hand1,
        hand2,
        hand3]
    funcs = []
    for hand in hands:
        op_handle(hand, find_and_click, ditu)
        if not find_and_clickIfExist(daliyuan) is False:
            continue
        find_and_click(shijieditu)
        find_and_click(haqidao)
        find_and_click(ditu)
        find_and_click(daliyuan)


def beibao100s(stop_event=None):
    from common import const
    while stop_event is None or not stop_event.is_set():
        if find_img(const.ditu) is not None:
            press_with_correction('b', 0.5)
            if stop_event is not None and stop_event.wait(0.5):
                break
            if find_img(const.zhuangbei) is not None:
                press_with_correction('b', 0.5)
            if stop_event is not None:
                stop_event.wait(100)
            else:
                time.sleep(100)
        else:
            if stop_event is not None:
                stop_event.wait(10)
            else:
                time.sleep(10)

if __name__ == '__main__':
    time.sleep(2)
    asyncDoSomeThing(forward, 4)
    time.sleep(1)
    asyncDoSomeThing(space)
