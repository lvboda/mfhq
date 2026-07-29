use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::config;
use crate::log::log;
use crate::platform::process;

const CE_EXE_NAMES: [&str; 3] = [
    "cheatengine-x86_64.exe",
    "Cheat Engine.exe",
    "cheatengine-i386.exe",
];

pub fn runtime_dir() -> PathBuf {
    std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|d| d.to_path_buf()))
        .unwrap_or_else(|| PathBuf::from("."))
}

fn pid_file() -> PathBuf {
    runtime_dir().join(config::CE_PID_FILE)
}

fn write_pid(pid: u32, exe: &Path) {
    let value = serde_json::json!({ "pid": pid, "exe_path": exe.to_string_lossy() });
    let _ = fs::write(pid_file(), value.to_string());
}

fn read_pid() -> (u32, String) {
    let text = match fs::read_to_string(pid_file()) {
        Ok(t) => t,
        Err(_) => return (0, String::new()),
    };
    match serde_json::from_str::<serde_json::Value>(&text) {
        Ok(v) => (
            v.get("pid").and_then(|p| p.as_u64()).unwrap_or(0) as u32,
            v.get("exe_path")
                .and_then(|p| p.as_str())
                .unwrap_or("")
                .to_string(),
        ),
        // 兼容旧格式：文件里只有一个裸 pid
        Err(_) => (text.trim().parse().unwrap_or(0), String::new()),
    }
}

fn clear_pid() {
    let _ = fs::remove_file(pid_file());
}

/// 只有记录的 exe 路径与进程实际路径一致时才终止，避免误杀 PID 复用的无关进程。
pub fn close() -> bool {
    let (pid, expected) = read_pid();
    if !process::pid_alive(pid) {
        clear_pid();
        return false;
    }
    if expected.is_empty() {
        log(&format!("skip closing Cheat Engine pid={pid}: missing recorded exe path"));
        return false;
    }
    let current = process::exe_path(pid);
    if current.to_lowercase() != expected.to_lowercase() {
        log(&format!("skip closing Cheat Engine pid={pid}: exe path mismatch"));
        clear_pid();
        return false;
    }
    let ok = process::kill_pid(pid);
    clear_pid();
    if ok {
        log(&format!("closed Cheat Engine pid={pid}"));
    }
    ok
}

fn locate() -> Option<(PathBuf, PathBuf)> {
    let candidates = [
        std::env::var(config::CE_DIR_ENV).ok().map(PathBuf::from),
        Some(runtime_dir().join(config::CE_BUNDLED_DIR)),
        Some(PathBuf::from(config::CE_DIR)),
    ];

    for candidate in candidates.into_iter().flatten() {
        if candidate.is_file() {
            let dir = candidate.parent()?.to_path_buf();
            return Some((dir, candidate));
        }
        if candidate.is_dir() {
            for name in CE_EXE_NAMES {
                let exe = candidate.join(name);
                if exe.exists() {
                    return Some((candidate, exe));
                }
            }
            let exes: Vec<PathBuf> = fs::read_dir(&candidate)
                .ok()?
                .flatten()
                .map(|e| e.path())
                .filter(|p| p.extension().is_some_and(|e| e.eq_ignore_ascii_case("exe")))
                .collect();
            if exes.len() == 1 {
                return Some((candidate, exes[0].clone()));
            }
        }
    }
    None
}

fn lua_script(scan_value: i64, write_value: i64, script_path: &Path, log_path: &Path) -> String {
    format!(
        r#"local processName = "{process}"
local processKeyword = "paraengineclient"
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
        process = config::CE_TARGET_PROCESS,
        scan = scan_value,
        write = write_value,
        script = script_path.display(),
        logp = log_path.display(),
    )
}

/// 对应 Python 的 run_ce_double_patch：写入 CE autorun 脚本并拉起 CE。
pub fn double_patch(scan_value: i64, write_value: i64) -> bool {
    let (ce_dir, ce_exe) = match locate() {
        Some(v) => v,
        None => {
            log(&format!(
                "Cheat Engine not found. Put it in resources or set {}",
                config::CE_DIR_ENV
            ));
            return false;
        }
    };

    close();

    let patch_log = runtime_dir().join("mfhq_ce_patch.log");
    let autorun = ce_dir.join("autorun");
    if fs::create_dir_all(&autorun).is_err() {
        log("failed to create CE autorun directory");
        return false;
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
        return false;
    }

    match Command::new(&ce_exe).current_dir(&ce_dir).spawn() {
        Ok(child) => {
            write_pid(child.id(), &ce_exe);
            true
        }
        Err(e) => {
            log(&format!("failed to start Cheat Engine: {e}"));
            false
        }
    }
}
