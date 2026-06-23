# Shared config for yingxionggu script and the controller.
# Monday=0 ... Sunday=6
ACTIVE_WEEKDAYS = {4, 5, 6}
START_HOUR = 10
STOP_HOUR = 22
CHECK_INTERVAL_SECONDS = 30

SCRIPT_EXE_NAME = "hero_recovered.exe"
SCRIPT_EXE_ENV = "MFHQ_SCRIPT_EXE"

MOFA_HAQI_PROCESS_NAMES = [
    "paraengineclient.exe",
    "00000BAC-paraengineclient.exe",
]
MOFA_HAQI_DEFAULT_PATH = r"C:\魔法哈奇\魔法哈奇.exe"
MOFA_HAQI_PATH_ENV = "MFHQ_GAME_PATH"
MOFA_HAQI_WINDOW_KEYS = ["魔法哈奇", "哈奇", "paraengine"]

CE_DIR = r"C:\Users\Cheat Engine 6.5 (1)\Cheat Engine 6.5"
CE_TARGET_PROCESS = "paraengineclient.exe"
