# CoreSwap 项目 AGENTS.md（项目级常驻指令）

> Reasonix 在本仓库工作时自动加载本文件（沿目录链，本项目级优先于全局 AGENTS.md）。
> 全局 `global-workspace/AGENTS.md` 的 skills 触发条件依然有效（engineering-eight-precepts 按需手动触发）。

## 〇、开始工作前（每个 session 必做）

1. 读 `E:\PYTHON\MC\NEXT_SESSION.md`（会话交接，仅换 session 前更新——平时排查结论进 docs/ 知识库）
2. `git status` 确认工作区状态
3. 对照参照/导出前，确认 `[BlockProbe] worldSeed=` 打印与参照文件实际 seed 一致（server.properties `level-seed` 硬编码，`-PbenchSeed` 只设 Java 属性）

## 一、工具链铁律（用户严禁 MinGW）

- **C++ 一律本机 MSVC**（VS 2026 Community，cl.exe 14.51.36231），禁用 MinGW/gcc
- 构建：cmd 里 `call "...\VC\Auxiliary\Build\vcvars64.bat"` + `set PATH=<VS Ninja 目录>;%PATH%` + `cmake -G Ninja -DCMAKE_BUILD_TYPE=Release`
- CMake 必须加：`/utf-8`（中文注释 C4819）、`/DNOMINMAX`（windows.h 宏冲突）、`/EHsc`
- 构建目录 `build-msvc`；C++ 源码在 `E:\PYTHON\MC\versions\1.20.1\cpp\worldgen\src`

## 二、worldgen.dll 对齐铁律

- **唯一权威 = `cpp\build-msvc\bin\worldgen.dll`**；每次编译后同步到 `java\src\main\resources\native\worldgen.dll`；对比/打包前 sha256 校验（CppBridge 启动打印 dll sha256 前缀）
- **DensityProbe 导出 vanilla 参照必须禁用 CppBridge**（`densityProbe` 不在 BenchMod.anyProbe → 默认启用 C++ 接管 → 参照被污染）；DensityProbe.run 开头已 `CppBridge.enabled=false`
- gradle runServer 崩溃后 java 进程可能残留（占 world/端口）——先 `taskkill /F /IM java.exe`
- 参照导出保 cns 存活：`simulation-distance=2` + 删 `run/world`（否则 spawn 预生成连带推进 → cns null）

## 三、知识库（`E:\PYTHON\MC\docs\`，README 目录表为准）

**按主题选篇，禁止默认往 09 堆：**
- 01 架构 / 02 随机派生 / 03 密度函数 / 04 含水层 / 05 矿脉 / 06 表面规则 / 07 流水线+崩溃 / 08 版本迁移 / **09 = 排查时间线**（完整推理链、工具演进；已确认结论提炼到 01-08）
- **追加不覆盖**：新内容新增章节，不改写既有正文
- **已解决项标注 ✅/❌ 不删除**（历史保留）
- **每条结论附「猜测→验证→排除→发现」完整链条** + 数据可信度（block_probe 逐位 / RouterProbe / DensityProbe / cns 反射【不可信】）

## 四、全局铁律（来自 Memory，务必遵守）

- **不放弃原则**：除非用户明确命令停止/放弃/换方向，否则持续推进——不得以「已花很多轮」「剩余是小问题」主动收尾
- **崩溃日志铁律**：任何交付的程序/原生库必须带全局崩溃捕获（异常类型/地址/寄存器 + 调用栈 + 写 crash-*.txt + 不吞异常）——C++ 用 `AddVectoredExceptionHandler`（模板 `crash_handler.h`）
- **提交纪律**：`user.name=unknowbug`、`user.email=unknowbug@users.noreply.github.com`、中文提交信息
- **发布铁律**：dll 必须 MSVC + dumpbin 导入表验证（踩过 3 次坑：MinGW dll/旧 dll/打包不同步）
- **FEATURES（矿物/装饰/结构）绝不做 C++ 化**；全版本覆盖是真实目标（含 1.17 及更早），对外文档禁止写「不计划」
- **交接文档唯一**：HANDOFF.md / NEXT_SESSION.md 只在仓库根（`E:\PYTHON\MC\`），已移出 git 仅本地，禁止在 versions\ 下新建
