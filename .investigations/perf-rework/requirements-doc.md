# 需求文档（requirements-doc.md）— 优化转向（draft）

> 轻量采纳 req-scout 产出契约（E:\PYTHON\req-scout\templates\requirements-doc.md）。
> 声称清单可翻译成验证断言；边界清单对应 `@anchor.idk` 语义。
> 状态：draft → candidate（Judge 技术审核通过）→ **confirmed（仅用户可给）**

## 元信息

- 项目代号：CoreSwap 1.20.1 worldgen 优化转向
- 需求 Scout：Reasonix 主会话
- 技术审核（Judge）：待派
- 客户：unknowbug（用户本人）
- 日期：2026-08-11
- 状态：☑ draft ☐ candidate ☑ **confirmed**（2026-08-11 用户确认 = 实施授权）

---

## 背景（本需求文档的来源）

1. 用户实测（2026-08-11）：CoreSwap 版（`-PcppReplace=1`）**性能反降**——传送后区块生成卡很久才出现；纯 vanilla（`-PcppDisable=off`）对照确认。
2. 静态分析定位：C++ 引擎本身快（block_probe 16 chunks 并行 49.4ms = 3.1ms/chunk），瓶颈在 **Java 桥接层（CppBridge.java）的并发设计**——JNI 调用被全局锁串行化、chunk 写入锁内串行、攒批 wait 强制延迟。
3. 用户决策：**放弃噪声 100% 对齐目标，转向优化优先**；有损容忍度 = **宏观一致**（用户实测地下也几乎看不出差异）。
4. 方向性验证结论：300515 种子下与 Java 差异过大**不是本项目问题**（用户实测 vanilla 对照确认；参照含废弃前脏数据）。

---

## 一、声称清单（deliverable MUST satisfy）

### RQ-001（Java 桥 JNI 并发化）

- **声称**：CppBridge 的 JNI `fillBlocks` 调用不再被全局锁串行化——多个 Java 线程可并发调用 native 方法（JNI 本身多线程安全）。**C++ 并发层改造纳入本 RQ 范围**（客户拍板）：签名/对齐输出不变，去 `CoreSwapPool::run` 的 `runMtx` 全局串行化或改 per-caller 池，使批间并发安全且真并行；不碰对齐语义。
- **验证方案草案**：多线程并发生成 N chunk，`[CppBridge] batch` 计时日志显示 C++ 耗时随线程数伸缩；体感传送加载明显加快。
- **source**：用户 2026-08-11 拍板「JNI 本身就是多线程的」+ 静态审查 P0-1 + Judge 第 2 轮 C1（worldgen_api.cpp L954-976 runMtx 实证）+ 用户拍板「纳入 C++ 并发层改造」。
- **范围声明**：仅 Java 桥（CppBridge.java + Mixin）+ C++ 并发层（CoreSwapPool::run 去 runMtx/per-caller 池，签名与对齐输出不变），不改 wg_fill_blocks_multi 签名、不碰对齐语义。
- **状态**：☑ draft ☐ candidate ☐ confirmed

### RQ-002（writeChunk 锁外并行写）

- **声称**：chunk 写入（writeChunk）不在 `synchronized(BATCH_LOCK)` 内执行——不同 Chunk 对象写入可并行（vanilla populateNoise 同构）。
- **验证方案草案**：写入阶段耗时不再随 chunk 数线性叠加阻塞攒批线程；日志 `write=ms` 占比显著下降。
- **source**：静态审查 P0-2。
- **范围声明**：写入的 Chunk 对象各自独立；CppBridge 内部状态（handle/缓存）线程安全。
- **状态**：☑ draft ☐ candidate ☐ confirmed

### RQ-003（去攒批 wait 强制延迟）

- **声称**：移除 `BATCH_LOCK.wait(2ms)` 强制等待。**调度语义 = M=1 非空即处理**（客户拍板）：队列非空即调 fillBlocks（低并发零等待），高并发时 C++ 批内并行兜底 + 多线程同时调用自然攒批摊薄 JNI 边界；无固定等待。**测试策略：最激进方式保留 C++ 全部优势，崩了再说**（先摸边界，不一点点加）。
- **验证方案草案**：体感传送加载明显加快（客户拍板：体感验收，不采量化基线）；无锁路径输出与 block_probe 对拍。
- **source**：静态审查 P1-1 + 用户「传送后区块卡很久」实测 + 用户 2026-08-11 拍板「M=1 非空即处理」+「激进方式保留 C++ 全部优势，崩了再说」。
- **状态**：☑ draft ☐ candidate ☐ confirmed

### RQ-004（per-thread buffer 池去共享锁）

- **声称**：BATCH_BUFS 共享复用池改为 per-thread 分配/池（激进模式：每线程 16×384KB ≈ 6MB，随线程数线性；接受内存代价），消除「为 buf 安全而锁」的必要性。
- **验证方案草案**：**随机抽 1 个种子对拍 + 统计差异分布**（客户拍板：只统计差异分布，留下知识，不做修复）；**种子范围排除 BK-003 脏参照种子集（300515 类）**；无锁路径不崩、无竞态污染。
- **source**：静态审查 P1-2 + 用户 2026-08-11 拍板「随机抽种子确定情况，统计差异分布，留下对应知识，不做其他操作」+ Judge C3（种子范围排除脏参照）。
- **状态**：☑ draft ☐ candidate ☐ confirmed

### RQ-005（stateById 进程级缓存）

- **声称**：writeChunk 的 `new BlockState[4096]` 缓存提升为进程级静态（vanilla 注册表运行时不变），消除每 chunk 重建。
- **验证方案草案**：内存/耗时微基准；与现状输出一致。
- **source**：静态审查 P2-1。
- **状态**：☑ draft ☐ candidate ☐ confirmed

### RQ-006（宏观一致容忍下的 C++ 有损加速【后续，不阻塞 RQ-001~005】）

- **声称**：在用户确认的「宏观一致」容忍度下，评估 C++ 有损优化（如 base_3d_noise 网格插值缓存）——**仅评估+用户逐项拍板后实施**，不默认开。
- **验证方案草案**：block_probe 宏观对比（地形/洞穴大体一致）+ 性能提升量化。
- **source**：用户 2026-08-11「彻底转向优化优先」+ 07 篇 L73-77 预留决策点。
- **状态**：☑ draft ☐ candidate ☐ confirmed（先不做，作为边界内待议）

---

## 二、边界清单（out of scope / 未验证 — @anchor.idk 语义）

### BK-001（对齐基线记录，不改 C++ 现有对齐）

- **what**：8576 SURFACE 99.9994% / 3200 SURFACE 99.9997% / -288 FULL 97.8460% / 300515 FULL 98.0975% 作为**历史基线记录**，优化过程中 C++ 对齐行为**默认不改**（除非用户对特定项拍板改）。
- **source**：知识库 docs/07 验证基线表（2026-08-10 实测）。
- **状态**：☑ draft ☐ candidate ☐ confirmed

### BK-002（性能基线不采集，体感验收）

- **what**：游戏内「传送后区块出现时间」的量化基线**明确不采集**——验收凭用户体感（用户拍板「体感验收」），不做 vanilla vs CoreSwap 量化对比基线。
- **source**：用户 2026-08-11 拍板「性能基线：体感验收」。
- **状态**：☑ draft ☐ candidate ☐ confirmed

### BK-003（300515 差异非本项目问题）

- **what**：300515 种子地下空间与 Java 差异过大 = 参照数据脏（花爆炸/树失败为废弃前实测），**不属本项目 bug**，不追责。
- **source**：用户 2026-08-11 实测 vanilla 对照确认。
- **状态**：☑ draft ☐ candidate ☐ confirmed

### BK-004（树花植被不回归）

- **what**：树花植被已废弃（用户拍板），优化不涉及恢复植被功能。
- **source**：2026-08-10 拍板 + NEXT_SESSION。
- **状态**：☑ draft ☐ candidate ☐ confirmed

---

## 三、术语表（操作化定义）

| 术语 | 操作化定义 | 备注 |
|------|-----------|------|
| 宏观一致 | 地形/洞穴大体位置一致，允许方块级差异（肉眼基本看不出） | 用户 2026-08-11 确认 |
| JNI 并发 | native 方法可被多线程并发调用，JVM 不串行化；线程安全由 native 实现负责 | JNI 规范 |
| 性能反降 | 游戏内传送后区块出现延迟 > vanilla 同场景 | 半操作化，基线待校准（BK-002） |
| 对齐基线 | 2026-08-10 block_probe 实测 TOTAL 数字 | 见 BK-001 |

---

## 四、假设表（暂定标注 — 假设 ≠ 需求）

| 编号 | 假设内容 | 依据 | 状态 |
|------|---------|------|------|
| AS-001 | C++ 侧 wg_fill_blocks_multi 被多线程并发调用（不同批）安全 | 现状：CoreSwapPool::run 用 runMtx 串行化批间（worldgen_api.cpp L954-976，32 视距崩溃根因修复）；本次改造后批间真并行（去 runMtx/per-caller 池） | ☑ 待验证（改造后需 JNI 并发回归确认） |
| AS-002 | vanilla populateNoise 的 chunk 写入本身就是并行的（不同 Chunk 对象） | MC 1.20.1 源码结构 + Mixin 拦截点 | ☑ 待验证 |
| AS-003 | BATCH_BUFS per-thread 后内存增量可接受 | 激进模式 16×384KB≈6MB/线程，随线程数线性（用户接受内存代价） | ☑ 待验证 |

---

## 五、技术约束规范（spec 摘要）

> v0.14 分层：需求侧只产「客户可确认事实」；架构设计（模块化/依赖方向/接口）归实施阶段。

- **平台/语言**：Java 17（Fabric loom 1.10.5）+ C++17（MSVC，不动）
- **外部依赖**：必须保持 JNI 接口（wg.CppWorldgen）签名不变；C++ worldgen_api.cpp 对齐代码不改（BK-001），**C++ 并发层（CoreSwapPool::run 去 runMtx/per-caller 池）纳入本次范围**（客户拍板，签名/对齐输出不变）
- **性能目标**：RQ-003 场景（传送后区块加载）**体感显著改善**（客户拍板：体感验收，不做量化基线）
- **客户强制的架构决策**：
  - 去全局锁（JNI 并发 + 写并行）
  - 不改 C++ 对齐代码（BK-001）
  - 宏观一致容忍（RQ-006 仅评估待议）
  - 优化程序规范 = 本需求文档（轻量采纳 req-scout 契约格式）

---

## 确认记录

- 客户逐条确认声称清单：☑ 是（2026-08-11）
- 客户逐条确认边界清单：☑ 是（2026-08-11）
- 客户点头：「需求清楚，可以实施」：☑ 是（2026-08-11）
- **本确认 = 实施授权**
