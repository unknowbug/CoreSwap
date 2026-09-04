# b2-dll-probe-parity（H2 候选：mod 路径 dll 与 block_probe C++ 在 NOISE+SURFACE 行为不一致）

- **status: draft**（静态排查，Degraded 分层——未运行任何 exe；只读分析）
- 候选编号：b2（H2）；日期标签 260905；分析者：fan-out worker
- 验证可比性声明（§9.7）：本结论载体 = 源码静态证据链（Java + gradle + C++ + Rust 源），覆盖面 = mod 加载链 + 两套生成管线默认模式，无运行时数值对比。

## 结论

**H2 成立，且根因是候选①（dll 是 Rust 而非 C++），叠加②的模式差异。mod 路径 NOISE+SURFACE 的执行体 = Rust `WorldgenRust.dll`（改名 worldgen.dll），与 block_probe 的 C++ 实现**根本不同源**（两套独立代码库，非同一产物的新旧版本）。**

### 逐项裁定

| 候选 | 裁定 | 依据 |
|---|---|---|
| ① dll 是 Rust | ✅ **实锤** | 见证据 E1/E2/E3 |
| ② fillBlocks 与探针管线模式不同 | ✅ 附加成立 | mod 存档链路 dll 侧跑完整管线（carver+features，旧 dll 无 stage-skip）；block_probe 默认 = genMode 0（仅 density→aquifer+oreVein→surface，无 carver/feature）。见 E4/E5/E6 |
| ③ dll 版本旧于探针（历史 bug） | ❌ 不适用 | 差异不是版本新旧，是语言/代码库不同（Rust 复刻 vs C++ 原版）；不过「mod 导出世界时 dll 尚无 09-08 stage-skip 修复」属实（时间线），见 E7 |

## 证据链

**E1 — CppBridge/CppWorldgen 加载链**（`runtime\1.20.1\java\src\main\java\wg\CppWorldgen.java` L18-37）：`System.load(cpp.worldgen.lib)` 或 `System.load(extractNativeDll())`——extractNativeDll 委托 `CoreSwapFixHelper.extractNativeDll()`（CoreSwapFixHelper.java L62-96）：从 **mod jar 内 `native/worldgen.dll`** 解压到临时目录后 System.load。搜索路径 = jar 资源，不是 java.library.path。

**E2 — jar 里的 worldgen.dll 的产源 = cargo（Rust）**（`runtime\1.20.1\java\build.gradle` L27-47）：processResources doFirst **硬编码拷贝 `E:\PYTHON\CoreSwap\WorldgenRust\target\release\WorldgenRust.dll` → rename `worldgen.dll` → resources/native**，注释明示「CoreSwap 唯一权威（Rust 主线）」「2026-08-30 从 C++ 转向 Rust：C++ 已归档，Rust JNI 桥 Java_wg_CppWorldgen_* 导出同名符号」。→ mod 路径经 JNI 调的 fillBlocks = **Rust `fill_chunk_blocks`**（WorldgenRust\src\worldgen_handle.rs），不是 C++ wg_fill_blocks。

**E3 — 现役 dll EC4A9AED… = Rust dll**（NEXT_SESSION.md L62）：「现役 dll（EC4A9AED…，260903-03 构建）未被本轮改动覆盖……**WorldgenRust dll 未动**；两处诊断门控只加了 C++ block_probe 路径」。即 sha256 EC4A9AED 的生产 dll 与本轮重编的 C++ block_probe.exe/worldgen_core.lib 是**两个不同产物**；AGENTS「build-msvc 唯一权威」约束的是 C++ 验证工具链/历史 C++ dll，生产 mod 链路 2026-08-30 起已切 Rust 主线。

**E4 — C++ block_probe 默认模式**（worldgen_api.cpp L227-230/L357-358 + L1124）：`genMode=0`（默认）= SURFACE 零退化模式：**density→aquifer(+oreVein)→surface**；`if (h->genMode == 1 && hasAquifer) applyCarversAndFeatures(...)`——carver/feature 只在 `WG_GEN_MODE=full`（block_probe `-features` 设，block_probe.cpp L99）执行。**默认 NOISE+SURFACE 确实不含 feature 阶段。**

**E5 — Rust 生产管线阶段**（worldgen_handle.rs L608-621）：fill 管线 = density→aquifer→oreVein→**surface→carver→features**，由句柄 flags（bit0=SKIP_CARVER bit1=SKIP_FEATURES bit2=SKIP_SURFACE）或 env 门控；flags=0 时回落 env 判定 = **全阶段执行**。

**E6 — 存档链路 stage-skip 是 2026-09-08 才加的**（CppBridge.java L63-79）：默认 stageMask 0b011（skip carver+features），经 `CppWorldgen.setFlags`（native，worldgen_handle.rs L98-109 对应实现）。**现役 dll 构建于 260903-03，早于 09-08 修复**——若该 dll 内无 wg_set_flags 导出/修复，则 mod cppReplace 世界导出时 **Rust features（ore_dirt/ore_gravel/disk_ 等）全量运行**。

**E7 — 消融旁证**（knowledge/discovered/workflow-patterns.md L530 + worldgen_api.cpp L1613-1623）：WG_FEATURE_SKIP=ore_dirt,ore_gravel,disk_ 消融实锤 features 直接改写这些方块；versions\1.20.1\docs\10 L2389：Rust features 在存档链路确实运行（8137 条 [FEATURE] 行）。

## 与现象的对齐（机制解释，draft 级）

mod 残差（stone/gravel/granite→dirt/sand、y 全高均匀、出界大量）与 block_probe 出界 cell 全 stone 的组合，**最佳解释 = 残差来自 Rust features/feature placement 阶段（含出界写入 bug），而 C++ block_probe 默认模式根本不跑该阶段**——所以「block_probe 无此行为」是模式差异 + 代码库差异的叠加，不是 C++ 同源实现的缺失。逐位验证需运行时确认（本 worker 只读未运行）：
- 用 `block_probe -features`（FULL 模式）重导同 seed 坐标，看出界 dirt 是否出现（区分「模式差异」与「Rust-only bug」）；
- 核对现役 dll 是否含 wg_set_flags 导出（dumpbin /exports，主会话执行）；
- 若残差在 FULL 探针也不复现 → 差异是 Rust feature placement 与 C++ feature 的实现偏差（Rust 复刻 bug）。

## 置信度与后续

- H2 ①③结论：**candidate 级证据**（多源静态交叉一致：Java 源 + gradle + C++ + Rust + NEXT_SESSION 台账）；候选②「模式不同」draft（未运行时消融）。
- 建议主会话收敛动作：dumpbin 查现役 dll 导出表（wg_set_flags 有无）+ block_probe -features 同点重导对比。
