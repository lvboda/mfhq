# 魂珠脚本设计

## 目标

新增 `hunzhu` 脚本，循环执行魂珠的固定三步操作。第一版只做最基本的循环，不带任何参数。

## 回合流程

一个回合按顺序点击三张图：

1. `photo/fuwenka.jpg`（符文卡）
2. `photo/jinu.jpg`（激怒）
3. `photo/maichongaoyi.png`（脉冲奥义）

每回合开头先 `focus_mofa_haqi_window()` 聚焦游戏窗口。

回合结束后检查地图是否在屏幕上（`find_img(const.ditu)`）。命中则依次按 `a` 0.5 秒、`w` 0.5 秒，然后进入下一回合。未命中则直接进入下一回合——地图检测只用来触发走位，不作为错误条件。

## 实现

沿用 `yingxionggu.py` 的结构：脚本文件放在 `src/` 下，流程用 `common/tool.py` 的图像助手驱动，图片路径统一走 `common/const.py`。

`src/common/const.py` 新增三条路径常量：`fuwenka`、`jinu`、`maichongaoyi`。三张图已存在于 `src/common/photo/` 下。

`src/hunzhu.py`：

- `start(round_no)`：聚焦窗口 + 三步点击，前后各打一条日志。
- `main()`：无参数，`while True` 递增轮次调用 `start()`，回合后做地图检测与走位，每轮间隔 2 秒。

`src/app.py` 的 `SCRIPTS` 增加 `"hunzhu": "hunzhu"`。因为 `main()` 不接受参数，现有的 `parse_args()` 入口机制无需改动。

`README.md` 的 Structure 与 Run 两节补充 `hunzhu`。

## 明确不做的部分

- **不自动登录**：脚本假定游戏已启动并登录，`main()` 中不含 `is_mofa_haqi_running()` / `start_mofa_haqi()` / `login_mofa_haqi()`。
- **无 CE**：流程不涉及 Cheat Engine，因此不需要 `atexit` 与信号处理的 cleanup（`yingxionggu.py` 的 cleanup 仅用于关闭 CE 进程）。
- **不加失败处理**：三步点击均使用 `find_and_click` 的默认 `duration=40`，某步 40 秒未找到图时打印提示并继续下一步，不重试、不中断。与 `yingxionggu.py` 的调用风格一致。

## 后续任务

`--level` 参数（取值 1/2/3、默认 1）已提出但尚未确定语义——目前的三步流程中没有等级分支点，需要先明确它是换图、多一步选择还是控制重复次数。

实现该参数时 `main()` 需要接受 argv，届时 `src/app.py` 必须改为 `parse_known_args()` 并透传剩余参数给脚本。`feature-split-screen-layout` 分支已为 `--screens` 写过同样的改造（`run_script()` + `inspect.signature` 分发），两条分支合并时这块会冲突。
