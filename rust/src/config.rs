pub const ACTIVE_WEEKDAYS: [u16; 3] = [5, 6, 0]; // GetLocalTime: 0=周日, 5=周五, 6=周六
pub const START_HOUR: u16 = 10;
pub const STOP_HOUR: u16 = 22;
pub const CHECK_INTERVAL_SECONDS: u64 = 30;

pub const APP_EXE_NAME: &str = "mfhq.exe";
pub const APP_EXE_ENV: &str = "MFHQ_APP_EXE";
pub const SCHEDULED_SCRIPT: &str = "yingxionggu";
pub const SCRIPT_PID_FILE: &str = "yingxionggu.pid";

pub const GAME_PROCESS_NAMES: [&str; 2] = ["paraengineclient.exe", "00000BAC-paraengineclient.exe"];
pub const GAME_DEFAULT_PATH: &str = r"C:\魔法哈奇\魔法哈奇.exe";
pub const GAME_PATH_ENV: &str = "MFHQ_GAME_PATH";
pub const GAME_WINDOW_KEYS: [&str; 3] = ["魔法哈奇", "哈奇", "paraengine"];

pub const CE_DIR: &str = r"C:\Users\Cheat Engine 6.5 (1)\Cheat Engine 6.5";
pub const CE_DIR_ENV: &str = "MFHQ_CE_DIR";
pub const CE_BUNDLED_DIR: &str = "resources/Cheat Engine 6.5";
pub const CE_PID_FILE: &str = "cheat_engine.pid";
