# Rust 重写设计

## 目标

用 Rust 重写脚本本体，解决两个问题：

1. **打包体积** —— 当前 `mfhq.exe` 为 72 MB，因为 PyInstaller 打包了整个 Python 解释器、opencv 与 numpy。Rust 静态二进制预期为个位数 MB。
2. **无法本地交叉编译** —— PyInstaller 不支持交叉编译，必须在 Windows 上构建。Rust 原生支持交叉编译，macOS 上即可产出 Windows exe。

Rust 代码放在 `rust/` 子目录，与现有 Python 代码并行，不影响主项目。

## 移植面

`src/common/tool.py` 共 79 个函数，`yingxionggu.py` 与 `hunzhu.py` 实际只用到 14 个：

```
log                          find_img                     wait_img_appear
find_and_click               find_and_clickIfExist        find_and_click_within_region
press_with_correction        focus_mofa_haqi_window       is_mofa_haqi_running
start_mofa_haqi              login_mofa_haqi              beibao100s
run_ce_double_patch          close_cheat_engine
```

其余 65 个是反编译遗留的死代码，不移植。

模板图 222 张、合计 1.0 MB，用 `include_bytes!` 编译进二进制，保持单文件分发。

## 目录结构

```
mfhq/
├── src/          Python，不改动
├── scripts/      不改动
├── .github/      新增独立 workflow，不改动现有的
└── rust/
    ├── Cargo.toml
    ├── .cargo/config.toml
    └── src/
        ├── main.rs        CLI 入口，对应 app.py
        ├── vision.rs      截图与模板匹配
        ├── input.rs       鼠标键盘
        ├── window.rs      窗口枚举与前台切换
        ├── process.rs     进程列举、终止、启动
        ├── assets.rs      内嵌模板图
        └── tasks/
            ├── hunzhu.rs
            └── yingxionggu.rs
```

## 技术选型

| 需求 | 方案 | 理由 |
|---|---|---|
| win32 API | `windows` crate | 微软官方，纯 Rust 绑定，无 C 依赖，交叉编译干净 |
| 图片解码 | `image` crate | 纯 Rust，支持 jpg 与 png |
| 模板匹配 | 自行实现 ZNCC | 见下节 |
| CLI 参数 | `clap` | — |
| CE 集成 | 保持现状，拉子进程执行 Lua | 与语言无关 |

选型的硬约束是**不得引入 C 依赖**。任何 CGO 式的绑定（如 OpenCV 的 Rust 绑定）都会使交叉编译失效，而交叉编译正是本次重写的目标之一。

## 模板匹配：自行实现 ZNCC

`imageproc::template_matching` 提供的 `CrossCorrelationNormalized` **不减均值**，而现有代码使用的 `cv2.TM_CCOEFF_NORMED` 是**零均值**归一化互相关。两者分数不可比，直接采用 imageproc 会使现有阈值与模板全部作废。

因此按 OpenCV 的公式一比一实现。收益是：

- 判定阈值 0.8 原样沿用
- 222 张模板无需重新截取
- 现有的匹配行为可直接对照验证

**等价性可以在 macOS 上验证完毕**：对同一批模板与截图，分别用 Python/OpenCV 与 Rust 实现计算分数并比对。这将本次重写最大的风险（重新校准所有匹配行为）前置到不依赖 Windows 的阶段解决。

性能上朴素实现不足以跟上 0.2 秒的轮询节奏：均值与方差用积分图做到 O(1) 查表，相关运算部分视实测结果考虑多线程。

## 分阶段

| 阶段 | 内容 | macOS 上可完成 |
|---|---|---|
| 0 | 骨架、ZNCC 实现、与 OpenCV 的等价性验证 | 全部 |
| 1 | 截图/输入/窗口/进程封装，移植 `hunzhu` | 仅编译 |
| 2 | 移植 `yingxionggu`，含 CE 调用与掉线自愈 | 仅编译 |
| 3 | 移植 `controller` | 仅编译 |

`pangpang` 不移植。它是独立的反编译产物，与主体架构不共用 `tool.py`，且是否仍在使用未确认。

两套实现并行存在，Python 版全程保持可用，按脚本逐个切换，不做一次性替换。

## 交叉编译与 CI

本地：`brew install mingw-w64`，然后 `cargo build --release --target x86_64-pc-windows-gnu`。

CI：新增独立 workflow，在 ubuntu runner 上交叉编译。相比现有的 `windows-latest` 构建更快，产物从 81 MB 降至个位数 MB——同时缓解私有仓库 500 MB artifact 配额的问题。现有 Python 构建的 workflow 不改动。

## 风险

1. **ZNCC 浮点等价性** —— 阶段 0 即可量化。若差异显著，需决定是调整阈值还是继续对齐实现。
2. **匹配性能** —— 需跟上 0.2 秒轮询。积分图之外可能还需多线程。
3. **pyautogui 的隐式行为** —— `find_and_click` 在点击后执行 `moveTo(200, 200)`，点击前有 `random.uniform(0, 0.2)` 的随机延迟；`press_with_correction` 使用忙等待而非 `sleep`。这些细节可能与游戏的反检测行为有关，移植时照抄，不做"优化"。

## 明确不做的部分

- 不改动 `src/` 下的 Python 代码与 `scripts/build.py`
- 不移植 `tool.py` 中未被使用的 65 个函数
- 不移植 `pangpang`
- 不引入任何带 C 依赖的 crate
