# 英雄谷掉线自愈设计

## 目标

`yingxionggu` 脚本在游戏掉线后自动恢复：关闭游戏进程，重新启动并登录，继续下一轮。只针对 `yingxionggu`，不涉及其他脚本。

## 掉线的两种形态

- **A 类**：游戏进程仍在，屏幕弹出掉线提示框，之后回到登录界面。
- **C 类**：游戏进程直接崩溃退出。

## 现状

`yingxionggu.main()` 只在进入主循环**之前**检查一次游戏是否运行：

```python
if not tool.is_mofa_haqi_running():
    tool.start_mofa_haqi()
    tool.login_mofa_haqi()

while True:
    round_count += 1
    start(round_count)
    time.sleep(2)
```

进入循环后不再有任何进程检查，因此两类掉线目前都无法自愈——脚本会在登录界面或空桌面上持续空点。

## 设计

### 守护线程

在 `main()` 中首次确保游戏就绪之后启动，`daemon=True`，跟随进程生命周期，不按轮次起停——掉线可能发生在轮次之间或等待比赛结果时。脚本退出或被 kill 时线程随之消失，无需手动管理。

线程只负责**检测**与**关闭游戏进程**，不执行恢复。恢复需要点击登录界面，若由守护线程执行会与主线程抢夺鼠标。

```python
def watch_disconnect():
    while True:
        time.sleep(DISCONNECT_CHECK_SECONDS)
        if recovering.is_set():
            continue
        if not tool.is_mofa_haqi_running():
            tool.log("守护线程：魔法哈奇进程不在")
            continue
        if tool.find_img(const.diaoxian) is not None:
            tool.log("守护线程：检测到掉线提示，关闭游戏进程")
            tool.kill_mofa_haqi()
```

检查间隔 `DISCONNECT_CHECK_SECONDS = 300`（5 分钟）。该值决定 A 类掉线后脚本空转多久才被发现；单次检查是一次全屏截图加一次模板匹配，约几十毫秒，5 分钟一次的开销可忽略。

进程已经不在时只记日志、不执行 `taskkill`——没有可关闭的对象，恢复交由主循环的 `ensure_game_ready()` 完成。

### 恢复路径

守护线程杀掉游戏进程后，无需跨线程传递信号：主循环每轮开头的 `ensure_game_ready()` 会发现进程不在，走重启与登录流程。掉线恢复与首次启动因此共用同一条代码路径。

```python
def ensure_game_ready():
    if tool.is_mofa_haqi_running():
        return
    recovering.set()
    try:
        tool.log("魔法哈奇未运行，尝试启动并登录")
        tool.start_mofa_haqi()
        tool.login_mofa_haqi()
    finally:
        recovering.clear()


while True:
    round_count += 1
    ensure_game_ready()
    start(round_count)
    time.sleep(2)
```

### recovering 标志

`ensure_game_ready()` 执行期间必须让守护线程停手，否则有两个问题：

1. 杀掉游戏到重新启动之间存在窗口期，守护线程看到进程不在会反复触发。
2. 登录过程中掉线提示框可能仍在屏幕上，守护线程会把刚启动的游戏再次杀掉，形成死循环。

因此用一个 `threading.Event` 在 `ensure_game_ready()` 前后 set/clear，守护线程在其置位时跳过检查。

### 当前轮次不主动中断

守护线程发现掉线时，主线程可能正阻塞在 `start()` 中某个 `find_and_click` 上。设计上不中断该轮：剩余步骤会各自空等 40 秒后跳过，轮次结束后回到循环顶部由 `ensure_game_ready()` 恢复。

代价是掉线后最坏白跑一到两分钟。换来的是不需要跨线程中断机制，也不必把状态检查穿插进 `start()` 的每一步。掉线是低频事件，这个取舍成立。

### 等待比赛成绩的超时

`start()` 中等待比赛成绩原本是 `wait_img_appear(const.saichangchengji, 0)`，`duration=0` 表示永不超时。比赛期间是一轮里最长的一段，也最可能掉线；一旦在此掉线，游戏进程被关闭后该图永远不会出现，主线程会永久阻塞在这一行，轮次无法结束，`ensure_game_ready()` 也就永远不会执行——整个自愈机制形同虚设。

改为 `ARENA_RESULT_TIMEOUT = 2400`（40 分钟）。该值需大于一场比赛的实际最长耗时，以免正常比赛被提前中断。代价是若掉线发生在比赛刚开始，脚本要空等满 40 分钟才超时并进入恢复。

## 改动清单

| 文件 | 改动 |
|---|---|
| `src/common/tool.py` | 新增 `kill_mofa_haqi()`，按 `config.MOFA_HAQI_PROCESS_NAMES` 执行 `taskkill` |
| `src/common/const.py` | 新增 `diaoxian` 路径常量 |
| `src/common/photo/` | 新增掉线提示框模板图 |
| `src/yingxionggu.py` | 新增守护线程与 `ensure_game_ready()`；`main()` 中的启动检查挪入循环；等待比赛成绩由无限等待改为 40 分钟超时 |

## 明确不做的部分

- **不处理登录凭据**：假定客户端记住账号密码，`login_mofa_haqi()` 现有的"只点按钮"流程足够。`const.zhanghao` / `const.mima` 仍不使用。
- **不改 `controller.py`**：进程内自愈方案对 controller 透明，PID 不变，controller 无感知。直接运行 `mfhq.exe yingxionggu` 与通过 controller 运行的行为一致。
- **不覆盖其他脚本**：`hunzhu`、`pangpang` 不在本次范围内。

## 待真机验证

- `taskkill` 能否可靠关闭游戏进程（`paraengineclient.exe` 与 `00000BAC-paraengineclient.exe` 两个名字都在配置里）。
- 掉线提示框模板在当前分辨率下的匹配率——项目中已有多张模板因分辨率变动而失配的先例。
- 恢复后游戏是否停留在可继续刷的状态，即 `login_mofa_haqi()` 的 `ditu` 判定是否成立。
