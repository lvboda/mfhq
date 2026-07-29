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

检查间隔 `DISCONNECT_CHECK_SECONDS = 300`（5 分钟）。该值决定 A 类掉线后脚本空转多久才被发现；单次检查是两次全屏截图加模板匹配，约上百毫秒，5 分钟一次的开销可忽略。

进程已经不在时只记日志、不执行 `taskkill`——没有可关闭的对象，恢复交由主循环的 `ensure_game_ready()` 完成。

A 类只检测掉线提示框一张图。该提示框的文案是"请退出游戏并重新登陆"，需要人工处理，不会自行消失，因此 300 秒的采样周期足以捕捉——掉线状态只有"提示框在"与"进程已不在"两种，不存在客户端悄悄退回登录界面的中间态。

`kill_mofa_haqi()` 返回是否真的关掉了进程。权限不足或进程名不匹配时 `taskkill` 会失败，若不检查返回值，日志会显示"已关闭游戏进程"而实际什么都没发生，恢复永远不会到来。

整个循环体包在 `try/except` 内。`find_img` 依赖 `pyautogui.screenshot()`，在会话锁屏或 RDP 断开时会抛异常；不捕获的话线程会永久死亡，而且表现与"一直没有掉线"完全一致。本项目自带 `tscon` 断开 RDP 的功能，该路径是现实存在的。

### 恢复路径

守护线程杀掉游戏进程后，无需跨线程传递信号：主循环每轮开头的 `ensure_game_ready()` 会发现进程不在，走重启与登录流程。掉线恢复与首次启动因此共用同一条代码路径。

`ensure_game_ready()` 返回是否就绪，主循环据此决定要不要跑这一轮——游戏没起来就跑，只会在一连串 40 秒超时里白耗约 130 秒。轮次计数放在检查之后递增，失败的恢复尝试不占用轮次编号。

```python
while True:
    if not ensure_game_ready():
        continue
    round_count += 1
    start(round_count)
    time.sleep(2)
```

`start_mofa_haqi()` 与 `login_mofa_haqi()` 的返回值都必须检查。两者失败时都只用 `print` 而非 `tool.log`，日志文件里看不到任何痕迹；不检查返回值就会形成一个完全静默的无限空转。失败时递增 `recovery_failures` 并等待 `RECOVERY_RETRY_SECONDS`（60 秒）再返回。

**恢复成功后重置 `ce_patch_attempts`。** CE 补丁是写进游戏进程内存的，进程一重启补丁即失效。而该计数器是进程级全局、上限 2：若在重启前已用满，重启后的新客户端没有补丁，之后每一轮都会撞上 10 次限制，打一行"已达最大尝试次数"后睡 10 秒返回——脚本会以看似健康的日志无限空转下去。这是自愈功能引入的新故障模式，必须一并重置。

**退避期间 `recovering` 保持置位。** 等待 60 秒的 `time.sleep` 在 `try` 块内，因此守护线程在这段时间不会动作。否则会形成如下死循环：守护线程杀掉客户端 → 重启 → 登录失败（提示框仍在屏幕上）→ `finally` 清除保护 → 下一个检查点再次匹配到提示框 → 再杀。每轮消耗一次完整启动，且没有任何上限。

### recovering 标志

`ensure_game_ready()` 执行期间必须让守护线程停手，否则有两个问题：

1. 杀掉游戏到重新启动之间存在窗口期，守护线程看到进程不在会反复触发。
2. 登录过程中掉线提示框可能仍在屏幕上，守护线程会把刚启动的游戏再次杀掉，形成死循环。

因此用一个 `threading.Event` 在 `ensure_game_ready()` 前后 set/clear，守护线程在其置位时跳过检查。

### 当前轮次不主动中断

守护线程发现掉线时，主线程可能正阻塞在 `start()` 中某个 `find_and_click` 上。设计上不中断该轮：剩余步骤会各自空等 40 秒后跳过，轮次结束后回到循环顶部由 `ensure_game_ready()` 恢复。

代价是掉线后最坏白跑一到两分钟。换来的是不需要跨线程中断机制，也不必把状态检查穿插进 `start()` 的每一步。掉线是低频事件，这个取舍成立。

### 等待比赛成绩

`start()` 中等待比赛成绩原本是 `wait_img_appear(const.saichangchengji, 0)`，`duration=0` 表示永不超时。比赛期间是一轮里最长的一段，也最可能掉线；一旦在此掉线，游戏进程被关闭后该图永远不会出现，主线程会永久阻塞在这一行，轮次无法结束，`ensure_game_ready()` 也就永远不会执行——整个自愈机制形同虚设。

替换为 `wait_arena_result()`，两个退出条件：

**进程消失即退出**是主要机制：比赛打多久都不打断，掉线后十几秒内退出。单纯用固定超时不行——超时从等待开始计时而非从掉线计时，掉线越早浪费越多，最坏接近超时全长。

**`ARENA_RESULT_TIMEOUT = 2400`（40 分钟）是兜底**，覆盖进程检查盖不住的情况：游戏进程存活但比赛因界面异常等原因永远出不了成绩。

**进程检查每 `PROCESS_CHECK_SECONDS = 10` 秒一次，不跟随 0.2 秒的截图节奏。** `is_mofa_haqi_running()` 执行的是无过滤的 `tasklist` 全进程表 dump，若每次循环都调用，一场十分钟的比赛就是三千次进程创建。

**进程判定用 `game_is_gone()` 做二次确认。** `is_mofa_haqi_running()` 把所有异常吞成 `False`，单次抖动就会误判成掉线，中断一场正在进行的比赛，并可能让 `start_mofa_haqi()` 对着已在运行的游戏再执行一次 `os.startfile`。因此首次读到"不存在"后间隔一秒复查，两次都为假才认定。

两条退出路径打不同的日志，便于在实跑日志中区分掉线与卡死。**返回值必须被使用**：`start()` 在其为假时直接返回，否则会继续执行两个各 40 秒、注定落空的点击，并在最后打印"第 N 轮完成"，把一次掉线记录成成功。

## 改动清单

| 文件 | 改动 |
|---|---|
| `src/common/tool.py` | 新增 `kill_mofa_haqi()`，按 `config.MOFA_HAQI_PROCESS_NAMES` 执行 `taskkill` |
| `src/common/const.py` | 新增 `diaoxian` 路径常量 |
| `src/common/photo/` | 新增掉线提示框模板图 |
| `src/yingxionggu.py` | 新增守护线程、`ensure_game_ready()` 与 `wait_arena_result()`；`main()` 中的启动检查挪入循环 |

## 明确不做的部分

- **不处理登录凭据**：假定客户端记住账号密码，`login_mofa_haqi()` 现有的"只点按钮"流程足够。`const.zhanghao` / `const.mima` 仍不使用。
- **不改 `controller.py`**：进程内自愈方案对 controller 透明，PID 不变，controller 无感知。直接运行 `mfhq.exe yingxionggu` 与通过 controller 运行的行为一致。
- **不覆盖其他脚本**：`hunzhu`、`pangpang` 不在本次范围内。

## 待真机验证

- `taskkill` 能否可靠关闭游戏进程（`paraengineclient.exe` 与 `00000BAC-paraengineclient.exe` 两个名字都在配置里）。
- 掉线提示框模板在当前分辨率下的匹配率——项目中已有多张模板因分辨率变动而失配的先例。
- 恢复后游戏是否停留在可继续刷的状态，即 `login_mofa_haqi()` 的 `ditu` 判定是否成立。
