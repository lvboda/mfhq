# 魂珠脚本设计

## 目标

新增 `hunzhu` 脚本，循环执行魂珠的固定操作，并用 `--level` 选择激怒等级。

## 回合流程

一个回合按顺序执行四步：

1. 点击 `photo/fuwenka.png`（符文卡）
2. 点击 `photo/jinu{level}.png`（激怒，level 取 1/2/3）
3. 选中敌人：找到 `photo/weijinu.png`，点击其中心上方 30 像素处
4. 点击 `photo/maichongaoyi.png`（脉冲奥义）

每回合开头先 `focus_mofa_haqi_window()` 聚焦游戏窗口。

回合结束后检查地图是否在屏幕上（`find_img(const.ditu)`）。命中则依次按 `a` 0.5 秒、`w` 0.5 秒，然后进入下一回合。未命中则直接进入下一回合——地图检测只用来触发走位，不作为错误条件。

## 等级参数

取值 1/2/3，默认 1，决定第 2 步点击哪张激怒图。`hunzhu.py` 用 `JINU_BY_LEVEL` 字典把等级映射到 `const.jinu1` / `jinu2` / `jinu3`，argparse 的 `choices` 直接取该字典的键，等级集合只需在一处维护。

以下五种写法等价：`hunzhu 2`、`hunzhu -l 2`、`hunzhu -level 2`、`hunzhu --l 2`、`hunzhu --level 2`。

实现上用了两个 argparse 条目：位置参数 `level`（`nargs="?"`，默认 `None`）和选项 `-l` / `-level` / `--level`（`dest="level_option"`，默认 `None`）。两者不能共用 dest，因此在 `parse_args` 末尾合并——选项优先于位置参数，都缺省时取 1。`--l` 这种前缀缩写由 argparse 的 `allow_abbrev` 自动支持，无需声明；`-level` 因为是显式声明的选项串，不会被误解析成 `-l` 附带值 `evel`。

`app.py` 无需为此改动：它的父解析器用 `parse_known_args`，位置参数 `script` 消费掉脚本名后，多余的位置参数和无法识别的选项都会进入 extras 原样透传，且不会误吞 `-c`。

## 实现

沿用 `yingxionggu.py` 的结构：脚本文件放在 `src/` 下，流程用 `common/tool.py` 的图像助手驱动，图片路径统一走 `common/const.py`。

`src/common/const.py` 的路径常量：`fuwenka`、`jinu1`、`jinu2`、`jinu3`、`weijinu`、`maichongaoyi`。旧的 `fuwenka.jpg` 与 `jinu.jpg` 已删除——前者被重新截取的 `.png` 取代，后者是技能图标，改用文字标签定位后不再需要。

`src/hunzhu.py`：

- `start(round_no, level)`：聚焦窗口 + 四步操作，前后各打一条日志（日志含等级）。
- `parse_args(argv)`：只有 `--level` 一个参数。
- `main(argv=None)`：`while True` 递增轮次调用 `start()`，回合后做地图检测与走位，每轮间隔 2 秒。

选中敌人用 `find_and_clickIfExist(const.weijinu, dis=(0, -30))`。`find_img` 返回匹配区域中心，屏幕坐标 y 向下增长，因此上方 30 像素是 `-30`。

注意：按名字最贴合这一步的 `find_and_click_dis(image_path, dis, duration)` 是反编译残缺函数，函数体被截断、调用后无任何效果，不可使用。同类残缺函数还有 `find_img_in_roi`、`find_and_sort_image_positions`、`find_and_click_num`，它们与唯一调用者 `judge_master_dis` 构成一个不可达的死代码簇。

`src/app.py`：`SCRIPTS` 增加 `"hunzhu": "hunzhu"`；入口改为 `parse_known_args()`，新增 `run_script()` 用 `inspect.signature` 判断脚本 `main()` 是否接受参数，接受则透传剩余参数，不接受但收到参数则报错退出。这段实现刻意与 `feature-split-screen-layout` 分支为 `--screens` 所写的版本保持一致，以便两条分支合并时不产生冲突。

`scripts/build.py` 的 `hidden_import_args()` 增加 `hunzhu`——`app.py` 用 `importlib` 动态导入，PyInstaller 静态分析扫不到，遗漏会导致打包后的 exe 运行 `hunzhu` 时 ModuleNotFoundError。

`README.md` 的 Structure 与 Run 两节补充 `hunzhu`。

## 明确不做的部分

- **不自动登录**：脚本假定游戏已启动并登录，`main()` 中不含 `is_mofa_haqi_running()` / `start_mofa_haqi()` / `login_mofa_haqi()`。
- **无 CE**：流程不涉及 Cheat Engine，因此不需要 `atexit` 与信号处理的 cleanup（`yingxionggu.py` 的 cleanup 仅用于关闭 CE 进程）。
- **不加失败处理**：三次点击均使用 `find_and_click` 的默认 `duration=40`，某步 40 秒未找到图时打印提示并继续下一步，不重试、不中断。与 `yingxionggu.py` 的调用风格一致。选中敌人一步用 `find_and_clickIfExist`，只匹配一次，未命中直接跳过。

## 待真机验证

以下两点在 Windows 上跑通前均未经确认：

- 选中敌人的 30 像素偏移是否准确，以及该步不做等待重试是否足够——若战斗中敌人出现有延迟，`find_and_clickIfExist` 会直接跳过。
- 第 3 步在流程中的位置按"点击激怒后、脉冲奥义前"实现。
