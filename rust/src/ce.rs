use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::config;
use crate::log::log;
use crate::pidfile::{self, runtime_dir};

const CE_EXE_NAMES: [&str; 3] = [
    "cheatengine-x86_64.exe",
    "Cheat Engine.exe",
    "cheatengine-i386.exe",
];

pub fn close() {
    crate::pidfile::kill_if_owned(config::CE_PID_FILE, "Cheat Engine");
}

fn locate() -> Option<(PathBuf, PathBuf)> {
    let candidates = [
        std::env::var(config::CE_DIR_ENV).ok().map(PathBuf::from),
        Some(runtime_dir().join(config::CE_BUNDLED_DIR)),
        Some(PathBuf::from(config::CE_DIR)),
    ];

    for candidate in candidates.into_iter().flatten() {
        if candidate.is_file() {
            if let Some(dir) = candidate.parent() {
                return Some((dir.to_path_buf(), candidate));
            }
            continue;
        }
        if !candidate.is_dir() {
            continue;
        }
        for name in CE_EXE_NAMES {
            let exe = candidate.join(name);
            if exe.exists() {
                return Some((candidate, exe));
            }
        }
        // 单个候选目录读取失败只跳过它，不能放弃后面的候选。
        let Ok(entries) = fs::read_dir(&candidate) else {
            continue;
        };
        let exes: Vec<PathBuf> = entries
            .flatten()
            .map(|e| e.path())
            .filter(|p| p.extension().is_some_and(|e| e.eq_ignore_ascii_case("exe")))
            .collect();
        if exes.len() == 1 {
            return Some((candidate, exes[0].clone()));
        }
    }
    None
}

fn lua_script(scan_value: i64, write_value: i64, script_path: &Path, log_path: &Path) -> String {
    format!(
        r#"local processName = "{process}"
local processKeyword = "{keyword}"
local scanValue = "{scan}"
local writeValue = {write}
local scriptPath = [[{script}]]
local logPath = [[{logp}]]

local function log(message)
  local f = io.open(logPath, "a")
  if f then
    f:write(os.date("[%Y-%m-%d %H:%M:%S] ") .. message .. "\n")
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
    ms.firstScan(soExactValue, vtDouble, rtRounded, scanValue, "", "00000000", "7fffffff", "",
      fsmNotAligned, "", false, false, false, false)
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
"#,
        process = config::GAME_PROCESS_NAMES[0],
        keyword = config::GAME_PROCESS_NAMES[0].trim_end_matches(".exe"),
        scan = scan_value,
        write = write_value,
        script = script_path.display(),
        logp = log_path.display(),
    )
}

/// 对应 Python 的 run_ce_double_patch：写入 CE autorun 脚本并拉起 CE。
pub fn double_patch(scan_value: i64, write_value: i64) {
    let (ce_dir, ce_exe) = match locate() {
        Some(v) => v,
        None => {
            log(&format!(
                "Cheat Engine not found. Put it in resources or set {}",
                config::CE_DIR_ENV
            ));
            return;
        }
    };

    close();

    let patch_log = runtime_dir().join("mfhq_ce_patch.log");
    let autorun = ce_dir.join("autorun");
    if fs::create_dir_all(&autorun).is_err() {
        log("failed to create CE autorun directory");
        return;
    }
    if let Ok(entries) = fs::read_dir(&autorun) {
        for path in entries.flatten().map(|e| e.path()) {
            let name = path.file_name().map(|n| n.to_string_lossy().to_string());
            if let Some(name) = name {
                if name.starts_with("mfhq_patch_") && name.ends_with(".lua") {
                    let _ = fs::remove_file(path);
                }
            }
        }
    }

    let script_name = format!(
        "mfhq_patch_{}_to_{}.lua",
        scan_value.to_string().replace('-', "neg"),
        write_value.to_string().replace('-', "neg")
    );
    let script_path = autorun.join(script_name);
    if fs::write(&script_path, lua_script(scan_value, write_value, &script_path, &patch_log)).is_err() {
        log("failed to write CE autorun script");
        return;
    }

    match Command::new(&ce_exe).current_dir(&ce_dir).spawn() {
        Ok(child) => pidfile::write(config::CE_PID_FILE, child.id(), &ce_exe),
        Err(e) => log(&format!("failed to start Cheat Engine: {e}")),
    }
}
