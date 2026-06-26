# Shared config for scripts and the controller.
# Monday=0 ... Sunday=6
ACTIVE_WEEKDAYS = {4, 5, 6}
START_HOUR = 10
STOP_HOUR = 22
CHECK_INTERVAL_SECONDS = 30

APP_EXE_NAME = "mfhq.exe"
APP_EXE_ENV = "MFHQ_APP_EXE"
SCHEDULED_SCRIPT = "yingxionggu"
SCRIPT_PID_FILE = "yingxionggu.pid"

MOFA_HAQI_PROCESS_NAMES = [
    "paraengineclient.exe",
    "00000BAC-paraengineclient.exe",
]
MOFA_HAQI_DEFAULT_PATH = r"C:\魔法哈奇\魔法哈奇.exe"
MOFA_HAQI_PATH_ENV = "MFHQ_GAME_PATH"
MOFA_HAQI_WINDOW_KEYS = ["魔法哈奇", "哈奇", "paraengine"]

CE_DIR = r"C:\Users\Cheat Engine 6.5 (1)\Cheat Engine 6.5"
CE_DIR_ENV = "MFHQ_CE_DIR"
CE_BUNDLED_DIR = "resources/Cheat Engine 6.5"
CE_TARGET_PROCESS = "paraengineclient.exe"
CE_PID_FILE = "cheat_engine.pid"
