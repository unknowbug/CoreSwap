# A 线 260911-05 知识库成稿草稿（10 时间线追加块 + 07 主题篇小节）

> **产出角色**：`core.worker` 知识库落盘 worker（subagent）。**本文件是草稿**——只提供「可直接粘贴的成稿正文 + 插入锚点」，主会话负责应用 + scan/一致性验证；本 worker **未修改** `versions/1.20.1/docs/`、`knowledge/` 任何既有文件（fs 只读 + 单文件写入）。
> **源材料（一手，整读，结论只能来自这里）**：`record-260911-05.md`（A1a/A1b/A1c/A1d + 行为门 + §9.7）、`a2-dim-gate-260911-05.md`（A2 三维门）、`review-260911-05.md`（judge 原文 + §7 C1-C12 处置表）。
> **锚点图例**：`record:N` = `.investigations/a1-opt-pool-260911-05/record-260911-05.md:N`；`a2:N` = 同目录 `a2-dim-gate-260911-05.md:N`；`review:N` = 同目录 `review-260911-05.md:N`；`10:N` = `versions/1.20.1/docs/10-timewise-archive.md:N`；`07:N` = `versions/1.20.1/docs/07-block-pipeline.md:N`。
> **纪律**：① 数字/结论均带 `文件名:行号`，材料没有的不推断；② 状态一律 **candidate**（AI 永不写 confirmed）；③ 凡百分比/耗时缺 §9.7 三要素（载体/覆盖面/可比性）者，一律标 **（材料未声明，应用时需补）**，不代填。

---

## 0. 摘要（应用清单）

| 项 | 落点 | 状态 | 主提交 |
|---|---|---|---|
| A1a 写回跳空气（`CppBridge.writeChunk` raw id 0 跳过 + `-Dcoreswap.skipair=0` 回退 + `WBCHECK` 自检） | 07 新小节 + 10 时间线块 | candidate | `b2b2f26` |
| A1b 诊断扫描门控（`nz` 全 buffer 扫描 + nether/end 16 点读回移入 `MIXLOG`，保留 O(1) 全空气短路探测） | 07 新小节 + 10 时间线块 | candidate | `6b90998`（材料零命中，见 §5-1） |
| A1c 6 高度图单遍 | **仅 10 时间线**（07 只留一行 ❌ 排除清单） | ❌ 已评估·不实施（用户裁决） | — |
| A1d `adaptive_threads` count=1 clamp（**Rust 共享层** `worldgen-core/src/api.rs`） | 07 新小节（A1d 段）+ 07:601 取代指针 + 10 时间线块 | candidate（清理项） | `b53b23f` |
| A2 nether/end `[WG-CONTENT]` 逐 chunk 指纹门（三维齐备） | 10 时间线块（07 只留一行交叉引用） | candidate | `8d8075a` |
| judge 审查（PASS-with-conditions，C1-C12 + I1-I4 已应用） | 10 时间线块 | candidate | `e8decef`（材料零命中，见 §5-1） |

**两个插入锚点（速览）**：
1. `10-timewise-archive.md` → **文件末尾行 `3154` 之后追加**（EOF，行 3154 是 260911-06 块最后一条 open 条目）。
2. `07-block-pipeline.md` → **文件末尾行 `1289` 之后追加**（EOF，行 1289 是 C 线小节「遗留 / 未覆盖」长行）；另有 A1d 取代指针**可选补丁**打在 `07:601`。

---

## 1.【成稿 A】`versions\1.20.1\docs\10-timewise-archive.md` 追加块

### 1.1 插入锚点（精确）

- **位置**：文件**末尾**，行 `3154` 结束后追加（当前 total = 3154 行）。
- **行 3153-3154（前文一行 + 文件真正末行）**：
  - `3153: - ✅ **R8/R9 论证**：R9 = 新路径不比老路径弱且在锁粒度/原子性上更强（…）…`
  - `3154: - 🔍 **open（未核/降级/边界）**：① R9-b 并发可见性专项未做；…⑨ **块号**：与同日 E5 复算块（19:37-20:52）同用 `260911-05` 标签，本行按 `-06` 记、目录名不改（见 `knowledge-update-draft-260911-05.md` §3）。`
- **追加格式**（与本文件既有块一致：空行 + `---` + 空行 + `##` 标题）：
  `3154` 之后插入：空行 → `---` → 空行 → `## 260911-05（…）`（下文 §1.2 正文）。
- **时序说明（应用时保留一行声明）**：本块实际时点（19:2x–20:5x）**早于**文件现有末块 260911-06（20:56–22:3x），按任务指定**追加在 EOF**（与本文件既有「追记」实践一致：见 `3123` 用户实机观察追记、`3135` Maint 回执追记）；块号与同日 B1（E5 复算 19:37-20:52）、C 线（20:56-22:3x）**同用 `260911-05` 标签**（既有 ⑨ 已就 C 线声明）。

### 1.2 可直接粘贴正文（从 `---` 起，整段复制）

---
---

## 260911-05（A 线：1.20.1 优化池 + nether/end 维度门；实际 2026-09-11 19:2x–20:5x）🔍 candidate（judge PASS-with-conditions，C1-C12 + I1-I4 已应用；confirmed 留人类）（追记于 260911-06 块之后；目录标签与同日 B1/C 块同用 -05）

> 过程产物 `.investigations/a1-opt-pool-260911-05/`（`record-260911-05.md`：A1a/A1b/A1c/A1d + 行为门 + §9.7 降级声明 / `a2-dim-gate-260911-05.md`：nether/end 全维行为门 / `review-260911-05.md`：judge 原文 + §7 C1-C12 处置表）；数据载体 `.tmp/a1-260911-05/`、`.tmp/a2-260911-05/`（驱动 + 比对脚本，临时区不入库）+ 日志 `.tmp/vivo-stutter-260911-02/ab-threads/logs/`、`.tmp/a2-260911-05/logs/`；`.artifacts/index.yaml` 四条（`swe:a1-opt-pool-260911-05:{plan,record,a2-dim-gate,judge-review}`，均 candidate）；提交 A1d `b53b23f` / A1a `b2b2f26` / A1b `6b90998` / A2 `8d8075a` / judge 修正 `e8decef`。
> ⚠️ **与 C 线的关系（勿写成「已消失」）**：C 线（260911-06）把默认写回改走 bulk section 路径后，A1a/A1b 所改的**旧逐块路径仍在库**——回退开关 `-Dcoreswap.bulkwb=0` 走旧路径，A1a 的跳空气写回与 A1b 的诊断门控即位于该路径（见 07 篇 C 线小节「回退开关 `-Dcoreswap.bulkwb=0`（旧逐块路径保留不删…）」）。

- ✅ **A1a 写回跳空气（`CppBridge.writeChunk` 对 raw id 0 跳过；回退 `-Dcoreswap.skipair=0`）**：**改了什么** = `writeChunk` 遇 raw id 0 不再逐格写回；开关为静态 final ⇒ 常量折叠（生产路径实际新增每格一次 `id==0` 判断，judge I4 修正措辞）。**判据（预登记）** = 跳过空气写 ≡ 逐格写 ⟺ 被跳格当前恰为 `Blocks.AIR`；判据谓词经 judge C5 严格化为 **`!isOf(Blocks.AIR)`**——`isAir()` 会漏计持 `cave_air`(730)/`void_air`(729) 的格子，而跳过的写是「写 `minecraft:air` 默认态」。**vanilla 一手源依据（judge 补引）** = `ChunkSection.setBlockState`（1.20.1）只在 `isAir` 变化时增减 `nonEmptyBlockCount`、`PalettedContainer.swap` 同值时写回同一 palette index ⇒ 对空气格写 air 是语义 no-op。**实测（最终构建 + 严格谓词，三维全格计数）** = overworld **276,171,456** / nether **194,630,604** / end **154,752,709** 个空气格被跳过，`stale_nonair` **三维均 0**（judge C4 由「机制外推」补跑为三维实证：wb-ow/wb-n/wb-e）；机制依据 = 三维 mixin 均 `populateNoise` HEAD cancel（`NoiseChunkGeneratorMixin` 279/333/364）⇒ 目标 chunk 全新。**行为门** = 跨臂 + 跨 dll + 跨 session 归档臂逐 chunk 指纹全等（4140/4140，`hash_diff=0`）。**证据路径** = `record-260911-05.md` §2-§4 + 日志 `[WB-CHECK]` 行。**commit `b2b2f26`**。
- ✅ **A1a 收益不主张（Degraded）**：cpuSec **493→476（−3.4%）**——**落在 ±10% 机器噪声带内、单对跨 run ⇒ 只作趋势**；且两臂均开 `WBCHECK`（每空气格一次 `getBlockState` = **2.76 亿次**）⇒ 该数应读作**含诊断成本的收益下界**，要引用定量数字须关 WBCHECK 重跑（judge C6）。结论性质 = 「等价性成立 + 无回归」，非「性能提升 N%」。
- ✅ **A1b 诊断扫描门控（`CppBridge.fillChunk`）**：**改了什么** = `nzBuf` 全 buffer 扫描（**98,304 读/chunk**）与 nether/end 16 点读回**整块门控到 `MIXLOG`**；生产侧「Rust 输出全 0」异常信号改以 **O(1) 短路探测**保留（`buf[0]==0` 才扫、遇首个非零即停）。**判据** = 纯诊断路径 ⇒ 无行为变化（证据 = 代码 diff + 各臂指纹全等）。**取舍声明（judge C2）** = 改前 `buf-all-air`/`buf-sparse` 是**无条件 println**（原记录「只门控 println」的理由句错误，已改正；类 javadoc 旧措辞作废，C10），`buf-sparse` 分级诊断随全量扫描一并移除，需要时用 `-Pmixlog=1` 的 `nz` 字段。**证据路径** = `record-260911-05.md` §3 A1b + `CppBridge.java` javadoc。**commit `6b90998`**。
- ❌ **A1c 6 高度图「全量重扫」——前提被一手源证伪，用户裁决放弃（已评估·不实施）**：一手源 `Heightmap.java:37-71` = vanilla `populateHeightmaps` **本就是单遍**（每列 (x,z) 只做**一次**下行扫描，6 个类型的谓词在**同一次扫描内**逐个匹配并移除，非「6 张各扫一遍」）⇒ 优化池该项描述**不成立**；残余候选只剩「读 `buf`（raw id）替代 `chunk.getBlockState`」，而代价/风险 > 收益（该项 0.42ms/chunk = 0.4%；`Heightmap.set` private、storage final ⇒ 外部写入须反射或自实现位打包）。**裁决** = 用户 HOOK-2 放弃实施。**证据路径** = `record-260911-05.md` §3 A1c + `review-260911-05.md` §3（judge 对照 `Heightmap.java` 核实）。
- ✅ **A1d `adaptive_threads` count=1 clamp（Rust 共享层 `worldgen-core/src/api.rs`，清理项）**：**改了什么** = `if count > 1 { min } else { max }` → 一律 `threads.min(count).max(1)`（批量路径 count>1 语义**逐字未变**）。**机制自证（运行期）** = `[WG-THREADS] count=1 threads_param=-1 nthreads=1`（新 dll，两条：a1-skipair0/1 的 `.log.err`）。**MT3 的历史前提已消失**——当时是**常驻池**（clamp 把池 worker 永久压到 1 = 结构性串行），现实现是 per-call `std::thread::scope`（唯一调用点 `api.rs:143`、无池，调用结束即回收）⇒ clamp 安全（judge 独立核对）。**无性能归因**（双臂都含 clamp，未做隔离 A/B）。**证据路径** = `record-260911-05.md` §3 A1d + `review-260911-05.md` §3。**commit `b53b23f`**。
- ✅ **A2 nether/end 全维行为门（开工发现真缺口 → 补载体）**：**改了什么** = `CppBridge.fillChunkEnd` 的 MIXLOG 块内补 `[WG-CONTENT]` 指纹行——首轮 end 臂 `intercepted=4761` 而 `contentLines=0` ⇒ 全维行为门在 end 维**载体缺失**（判据意义：**「接管生效」与「行为门可判」是两件事**）。**判据** = 三维「跨形态 + 跨 run 逐 chunk 指纹 sorted diff = 0」。**实测** = **overworld 4140/4140、nether 4761/4761、end 4761/4761 逐 chunk 指纹差 0**（`nz_sum` = 130,807,104 / 117,386,292 / 1,255,739）；**正对照** overworld vs nether `hash_diff=4140/4140`（门有检测力）；**分母语义** = 载体条数 = **经接管的 chunk 数（≠ Chunky Processed 4225）**，门判据不依赖任何分母（judge C3）。**证据路径** = `a2-dim-gate-260911-05.md` §1-§2 + 日志 `[WG-CONTENT]` 行。**commit `8d8075a`**（judge C9 改正早前误引的不存在提交号 `3c1f9c6`）。
- 🔍 **A2 附带读数（非门判据，Degraded）**：nether async **25s/24s** vs sync **69s**（形态效应 ~2.8×，cpuSec 212/202 vs 149）；end async **11s/8s** vs sync **20s**（~2.0-2.5×，cpuSec 105/84 vs 69）⇒ 与 260910-06（1.21.6 同款异步化）方向一致（单车道同步形态在 nether/end 都是净亏损）；但跨 run 摆动 **±27%** ⇒ **不宣布定量收益**。
- ✅ **judge（candidate 授予 SHOULD 触发，隔离子进程）**：A1 / A2 均 **PASS-with-conditions**、无 FAIL，**全部量化数字复现一致**；C1-C12 + I1-I4 **逐条已应用**（处置表 = `review-260911-05.md` §7）——重点：C1 可比性改为「a1 双臂同 dll `838e8979`；三归档臂为 A1d 前构建 `dd3b645f`」并补「跨 dll 指纹全等 = A1d 输出中立性独立证据」、C3 分母语义声明、C4 由外推补跑为**三维实证**、C5 判据谓词严格化、C7 比对脚本加最小载体阈值断言（空载体拒绝出结论——曾复现「两边都空 ⇒ 报 diff=0」假通过）、C8 下游归因降为**未测外推**、C9 提交号改正、C12「85 缺口」说法作废。judge **未改任何 status**；**本块结论维持 candidate，confirmed 留人类**（judge 修正 commit `e8decef`）。judge 盲区如实登记：旧二进制 clamp 前 `nthreads` 只能推导（无日志行）、`dd3b645f` 是否确为 A1d 前构建按时间线推断、f2 成本拆分（0.42ms/chunk）本块未复核。
- 🔍 **open（未核/降级/边界）**：① **写回后内容指纹（post-write hash）仍未做**——三维均未覆盖（260910-06 open ⑤ 延续；A1a 以「前提全格实证」替代）；② overworld 相对 nether/end **少 621 条载体**成因未查（原「85 缺口」说法已废，降为 open 假设）；③ nether run 级非确定的**成因域**只排除「Rust 填充层」（本载体只覆盖 `buf`），下游层未测（judge C8 未测候选：Java carver/feature/装饰层、写回路径、存档序列化）；④ A1d「改前 nthreads=10」为公式推导非实测；⑤ `buf-sparse` 分级诊断已移除；⑥ 本项改动**尚未随任何 release 出货**（1.0.29 不含 A1 改动，出单评估见计划 §2 A4）；⑦ dll 硬门禁只比日志里 **16 hex 前缀**（非全 sha256，judge I2）；⑧ **块号**：与同日 B1（E5 复算）/C 线 bulk 块同用 `260911-05` 标签，本块按 `-05` 记、目录名不改。

---
---

### 1.3 应用时需核（**不属于粘贴正文**）

1. `6b90998`（A1b）与 `e8decef`（judge 修正）**在三份源材料中零命中**——见 §5-1，应用前用 `git log --oneline` 核对后保留/改正。
2. 「实际 2026-09-11 19:2x–20:5x」为主会话给定；材料内可见时间锚只有「18:02 运行 / 19:36 提交」（`review:62`）⇒ 精确区间 **（材料未声明，应用时需补）**。
3. 粘贴后建议 `git add` 前扫一遍：本块是否只在 10 篇（时间线），07 篇只放结论小节（防止主题篇堆时间线）。

---

## 2.【成稿 B】`versions\1.20.1\docs\07-block-pipeline.md` 小节

### 2.1 插入锚点（精确）

- **主锚点（推荐）**：文件**末尾**，行 `1289` 结束后追加（当前 total = 1289 行）。
- **行 1288-1289（前文一行 + 文件真正末行）**：
  - `1288: ### 遗留 / 未覆盖`
  - `1289: - **遗留 / 未覆盖**：R9-b 并发可见性专项未做；`plan §10.3`（消费者缓存前置）已收口——…**Tier 3 弱信号未闭合**（bulk 臂自身离散度略高，n=3 未达显著）；共享 Java 适配核抽取在 C 完成后另立（本块只落 1.20.1 一份）。`
- **追加方式**：`1289` 之后插入：空行 + `## 2026-09-11 A 线：…`（下文 §2.2 正文）。
- **备选锚点（若主会话偏好严格时序）**：插到 C 线小节**之前**，即 `1233: ## 2026-09-11 C 线：Java 侧 bulk section 写回（D1，Rust 零改动）— candidate（judge PASS-with-conditions；M1/M2/M3 已应用 / confirmed 留用户）` 之前（**锚点文本 = 行 1233 整行**，插在其上方 + 空行）。二选一，勿两处都插。

### 2.2 可直接粘贴正文（从 `##` 起，整段复制）

## 2026-09-11 A 线：Java 侧写回优化池（A1a 跳空气 / A1b 诊断门控）+ Rust 共享层线程 clamp（A1d）+ nether/end 全维行为门（A2）— candidate（judge PASS-with-conditions；C1-C12 + I1-I4 已应用 / confirmed 留用户）

> 载体与依据：`.investigations/a1-opt-pool-260911-05/record-260911-05.md`（A1a/A1b/A1c/A1d + 行为门 4140/4140 + §9.7 降级声明）+ `a2-dim-gate-260911-05.md`（nether/end 各 4761/4761）+ `review-260911-05.md`（judge 原文 + §7 处置表）；`.artifacts/index.yaml` 四条（`swe:a1-opt-pool-260911-05:{plan,record,a2-dim-gate,judge-review}`）。提交 A1a `b2b2f26` / A1b `6b90998` / A1d `b53b23f` / A2 `8d8075a` / judge 修正 `e8decef`。
> ⚠️ **与 C 线小节的路径关系**：C 线（本篇「2026-09-11 C 线」小节）把默认写回改走 bulk section 路径；**A1a/A1b 所在的旧逐块路径保留不删**（回退 `-Dcoreswap.bulkwb=0`）⇒ 本节结论对该回退路径**仍然有效**，不得读作「已消失 / 已被 C 线取代」。

### A1a 写回跳空气（`CppBridge.writeChunk`，回退 `-Dcoreswap.skipair=0`）
- **形态**：`writeChunk` 遇 raw id 0（= `minecraft:air`，`data/blocks.json` 0）不再逐格 `setBlockState`；开关静态 final ⇒ 常量折叠，生产路径实际新增每格一次 `id==0` 判断（judge I4）。
- **判据（预登记）**：跳过空气写 ≡ 逐格写 ⟺ 被跳格当前恰为 `Blocks.AIR`；谓词取 **`!isOf(Blocks.AIR)`**（`isAir()` 会漏计 `cave_air` 730 / `void_air` 729，而跳过的写是「写 air 默认态」）。
- **vanilla 依据**：`ChunkSection.setBlockState` 只随 `isAir` 变化增减 `nonEmptyBlockCount`；`PalettedContainer.swap` 同值写回同一 palette index ⇒ 对空气格写 air 为语义 no-op（judge 补引）。
- **等价性（Full，噪声无关；§9.7 三要素）**：
  - **载体**：1.20.1 + Chunky radius 500（Processed 4225 chunks/臂，中心 `-48,-11`，seed `417950215108767439`），`[WB-CHECK]` 全格计数。
  - **覆盖面**：三维全格计数，非抽样——overworld 276,171,456 / nether 194,630,604 / end 154,752,709 空气格被跳过，`stale_nonair` **三维均 0**；覆盖面分母（经接管 chunk 数）见 A2 段（overworld 4140 / nether·end 4761）。⚠️ 该项为**单 run 单维、无负对照、与计时同 run**（judge 核对表）。
  - **可比性**：a1 双臂同 dll `838e89794a54e19d`、同 seed/区域中心/工具修订、背靠背串行；三归档臂（exec-def/cap0-fresh/exec-16）为 **A1d 前构建** `dd3b645f2c79d2cb`，跨 dll 逐 chunk 指纹全等（4140/4140）⇒ 兼作 **A1d 输出中立性独立证据**。
  - **结果**：等价性成立（三维实证）；行为门跨臂 + 跨 dll + 跨 session 归档臂指纹 `hash_diff=0`。
- **收益：Degraded（不主张定量）**——cpuSec 493→476（**−3.4%**），单对跨 run 且**落在 ±10% 机器噪声带内 ⇒ 只作趋势**；两臂均开 `WBCHECK`（2.76 亿次 `getBlockState`）⇒ **含诊断成本的收益下界**，引用定量数字须关 WBCHECK 重跑。结论性质 = 「等价性成立 + 无回归」。

### A1b 诊断扫描门控（`CppBridge.fillChunk`）
- **形态**：`nzBuf` 全 buffer 扫描（98,304 读/chunk）+ nether/end 16 点读回**整块移入 `MIXLOG`**；生产侧「Rust 输出全 0」异常信号改以 **O(1) 短路探测**保留（`buf[0]==0` 才扫、遇首个非零即停）。
- **判据**：纯诊断路径 ⇒ 无行为变化（证据 = 代码 diff + 各臂指纹全等）。
- **取舍声明**：改前 `buf-all-air`/`buf-sparse` 为**无条件 println**（原「只门控 println」理由句错误、类 javadoc 旧措辞作废）；`buf-sparse` 分级诊断随全量扫描移除，需要时用 `-Pmixlog=1` 的 `nz` 字段。

### A1d `adaptive_threads` count=1 clamp（Rust 共享层，清理项）
- **形态**：`worldgen-core/src/api.rs` 的 `if count > 1 { min } else { max }` → 一律 `threads.min(count).max(1)`；批量路径（count>1）语义逐字未变。
- **机制自证（运行期）**：`[WG-THREADS] count=1 threads_param=-1 nthreads=1`（新 dll 两条日志行）。
- **与上文 2026-08-16 clamp 小节的关系（本条为该待办在 Rust 共享层的收口）**：8-16 发现的历史前提是**常驻池**（clamp 把池 worker 永久压到 1 = 结构性串行），现实现是 **per-call `std::thread::scope`**（唯一调用点 `api.rs:143`、无池，调用结束即回收）⇒ 前提消失、clamp 安全（judge 独立核对）。
- **归口理由**：本篇已承载 clamp 课题的发现与待办（上文「[B]/实机 M=1 结构性串行（threads clamp 发现，candidate）」），且 `03-density-functions.md` 无 threading/池宽章节（`^## ` = 功能目的 / 1.20.1 工作机制 / 版本敏感点 / 已验证的坑 / 2026-08-08 已验证结论；`线程池|adaptive_threads|池宽` 在其中零命中）⇒ **不归 03 篇**。
- **无性能归因**（双臂都含 clamp，未做隔离 A/B）；**改前 nthreads=10 为公式推导非实测**（旧 dll 无自证行）。

### A2 nether/end 全维行为门（`CppBridge.fillChunkEnd` 补指纹载体）
- **开工缺口（judge 核实属实）**：end 路径原本**没有 `[WG-CONTENT]` 指纹行**——首轮 end 臂 `intercepted=4761` 而 `contentLines=0`；判据意义 = **「接管生效」与「行为门可判」是两件事**，门禁类课题 MUST 先核「该维是否有载体行」再谈门值。修复 = MIXLOG 块内补指纹行。
- **判据（三条可复用）**：① 该维有载体行（否则先补载体）② 跨形态 diff=0（证 exec/异步化不改内容）③ 跨 run diff=0（证该维 Rust 输出确定性）。
- **结果（Full，噪声无关；§9.7 三要素）**：
  - **载体 / 覆盖面**：1.20.1 + Chunky radius 500（Processed 4225 chunks/臂，中心 `-48,-11`，seed `417950215108767439`），**经接管的全部 chunk 均进门、非抽样**——overworld 4140、nether 4761、end 4761；**分母 = 接管调用数 ≠ Chunky Processed**（门判据不依赖分母）。
  - **可比性**：9 条维度臂 dll 全为 `838e89794a54e19d`、同 seed/区域中心/工具修订；nether 臂与 end 臂**不同 Java 构建态**（end 在补指纹行后重跑）已声明——`git show 8d8075a --stat` = 2 files/57 insertions，Java 仅 +4 行且全在 `fillChunkEnd`，不影响 nether/overworld 路径。
  - **结果**：三维跨形态 + 跨 run 逐 chunk 指纹差 **0**（`nz_sum` = 130,807,104 / 117,386,292 / 1,255,739）；正对照 overworld vs nether `hash_diff=4140/4140`（门有检测力）；比对脚本已加**最小载体阈值断言**（空载体拒绝出结论——曾复现「两边都空 ⇒ 报 diff=0」假通过）。
  - **打印位置**：overworld 的 `[WG-CONTENT]` 在 `writeChunk` **之前**、nether/end 在其**之后**；三者都对**只读 `buf`** 计算 ⇒ hash 语义一致。
- **附带读数（非门判据，Degraded）**：nether async 25s/24s vs sync 69s（~2.8×，cpuSec 212/202 vs 149）；end async 11s/8s vs sync 20s（~2.0-2.5×，cpuSec 105/84 vs 69）⇒ 与 260910-06（1.21.6 同款异步化）方向一致；跨 run 摆动 **±27%** ⇒ 不宣布定量收益。
- **❌ 排除清单（一行，防重走弯路）**：① 「nether run 级非确定来自 Rust 填充层」以外的下游归因（Java carver/feature/装饰层、写回路径、存档序列化）= **未测外推**，不得当结论引用（judge C8）；② 早前「85 个 Chunky 已处理但无接管行」算式**不成立**——期望集不是 Processed 集（judge C12，取代指针见 10 时间线 260910-06 块 ⑥）；③ A1c「6 张高度图全量重扫」= 前提被 vanilla 一手源证伪（`Heightmap.java:37-71` 本就单遍）⇒ 已评估·不实施（用户裁决）。

### 遗留 / 未覆盖（A 线）
- **写回路径未覆盖（降级）**：指纹取在 `writeChunk` 前/后但**只哈希只读 `buf`**，不覆盖写回结果；**写回后内容指纹（post-write hash）三维均未做**（260910-06 open ⑤ 延续）。
- overworld 相对 nether/end **少 621 条载体**成因未查（原「85 缺口」已废，降为 open 假设）；nether run 级非确定**成因域未测**（只排除 Rust 填充层）。
- A1d 改前 `nthreads=10` 为公式推导非实测；`dd3b645f` 是否确为 A1d 前构建按时间线推断（未反汇编）。
- 1.0.29 **不含 A1 改动**（尚未随任何 release 出货）；1.21.6 侧同构问题属 Phase B 范围。

### 2.3 A1d 归口判定（写明理由）+ 可选补丁

**判定：A1d 归 07（本节 A1d 段）+ 07:601 取代指针，不只进时间线。** 理由：① 07 已承载 clamp 课题的**发现 + 待办**（`07:596-601`「[B]/实机 M=1 结构性串行（threads clamp 发现，candidate）」，`07:601` = 「修复待办（clamp 改 `if (threads > count && count > 1)`…）」），A1d 是同一课题在 **Rust 共享层**的收口 ⇒ 检索入口在 07；② `03-density-functions.md` **无论证归口**（章节 = 功能目的 / 1.20.1 工作机制 / 版本敏感点 / 已验证的坑 / 2026-08-08 已验证结论；`线程池|adaptive_threads|池宽` 零命中，其 `clamp` 命中全是数学 clamp）——线程池宽不属密度函数域；③ A1c 才是「只进时间线」项（07 只留一行 ❌ 排除清单），因其为**未实施**且归口主题（高度图/方块写回）无新增实施面。

**可选补丁（A1d 取代指针，建议应用；行 601 末尾追加，原句不删不改——§15.4）**：

- 在 `07:601` 该行**末尾追加**：
  ` ⬆️ **260911-05 A1d 收口（Rust 共享层同族 clamp）**：`worldgen-core/src/api.rs` 的 `adaptive_threads` 已改为一律 `threads.min(count).max(1)`，运行期自证 `[WG-THREADS] count=1 nthreads=1`；本条历史前提（常驻池）在现实现（per-call `std::thread::scope`，唯一调用点 `api.rs:143`）中已消失 ⇒ clamp 安全（清理项、无性能归因；详见 10 时间线 260911-05 块）。旧措辞按 §15.4 保留不改。`
- **同步可选**：`10:1475`（2608-16 块的 clamp 修复待办行）与 `10:1485`（open「实机实跑对比」）同法追加一行取代指针；材料未要求在 10 篇改写历史块 ⇒ **由主会话裁决是否加**（不加也不影响本块 A1d 结论成立）。

---

## 3. §9.7 / 分层声明检查表（草稿中每个百分比 / 耗时 / 计数）

> 判据：任何量化输出 MUST 同行声明**载体 + 覆盖面 + 与既有口径可比性**（§9.7 三要素）。下方「三要素」列缺项处**一律不代填**，按任务要求标 **（材料未声明，应用时需补）**。

| # | 数字 / 耗时（草稿出处） | 载体 | 覆盖面 | 可比性（含与既有口径） | 材料锚点 | 处置 |
|---|---|---|---|---|---|---|
| 1 | A1a 空气格 276,171,456 / 194,630,604 / 154,752,709；`stale_nonair=0` | ✅ 1.20.1 + Chunky radius 500（Processed 4225/臂，中心 `-48,-11`，seed `417950215108767439`） | 三维全格计数（非抽样）；但各维**经接管 chunk 分母在 A1 记录中未给**（仅 A2 记录给 4140/4761） | ✅ 同 dll `838e8979` / 同 seed·中心 / 背靠背；限定「单 run 单维、无负对照、与计时同 run」 | `record:19-20,33-38,84-86`；`review:38` | 分母项标 **（材料未声明，应用时需补）**：若引用分母须交叉引 A2 表并注明跨记录来源 |
| 2 | A1a 收益 cpuSec 493→476（−3.4%） | ✅ 同上 | 双臂各 4225 chunks（`record:19-20`）；**cpuSec 的覆盖面口径未单独声明** | ✅ 同 dll/seed/中心/背靠背串行；**单对跨 run、±10% 带宽内只作趋势**；含 WBCHECK 诊断开销 ⇒ 下界 | `record:20,22,41,86`；`review:49` | ⚠️ **与既有口径可比性**（260911-03 归档臂 / 其他块 cpuSec 可否互引）**（材料未声明，应用时需补）**；草稿已禁止宣布定量收益 |
| 3 | A1a 诊断开销 2.76 亿次 `getBlockState` | ✅ 同 #1（276,171,456 空气格 × 1 次） | 未单独声明 | 计数（非跨 run 测量）；与计时同 run | `record:22`；`review:25,39` | 保持为「开销说明」，不作收益论据 |
| 4 | A1b 98,304 读/chunk | 代码内计数（1 chunk buffer 规模） | 不适用 | 不适用（非测量） | `record:46` | 草稿已标「实现内读次数」，不列 §9.7 量化 |
| 5 | A2 门 4140/4761/4761 差 0 + `nz_sum` 130,807,104 / 117,386,292 / 1,255,739 + 正对照 4140/4140 | ✅ a2 §5 | ✅ 经接管全部 chunk、非抽样；分母 = 接管调用数（≠ Processed 4225） | ✅ 9 臂同 dll/seed/中心/工具；nether 与 end 不同 Java 构建态**已声明** | `a2:18-29,49-51`；`review:14,45` | 三要素齐备 ✅（**可直接引用**） |
| 6 | 载体条数 4761 / 4140 vs Chunky Processed 4225 | ✅ | ✅ | ✅ 同 center/radius/Processed | `a2:26-27` | 已声明「分母 ≠ Processed」⇒ 引用时**必须带该句** |
| 7 | 621 条差（4140 vs 4761） | ✅ 同 #5 | ✅ 同 #5 | ✅（同 center/radius/Processed） | `a2:27` | 只作「未查起因的 open 假设」，不得归因 |
| 8 | A2 附带读数 wall 25s/24s/69s、11s/8s/20s；cpuSec 212/202/149、105/84/69 | ✅ 同 #5 | 各维 run 数：nether async×2 + sync×1、end async×2 + sync×1（4 臂/维） | 部分 ✅（同 dll/seed/中心；**±27% 跨 run 摆动已声明** ⇒ Degraded/只作趋势）；**与既有口径可比性**（端到端 wall vs Java 原版等）**（材料未声明，应用时需补）** | `a2:21-22,35-39,50-52` | 草稿已标 Degraded；引用前需补口径（或维持「不宣布定量」） |
| 9 | A2 end `nz_sum/条数 ≈ 264 非空格/chunk（32768 格）` | ✅ 该维（end） | 条数 4761 | **（材料未声明，应用时需补）**——材料只标为「sanity 参照」 | `a2:45` | 草稿已标「非门判据 · sanity 参照」，不当判据用 |
| 10 | A1c 成本 0.42ms/chunk = 0.4% | 继承更早工作块（f2 成本拆分） | — | **本块未复核**（judge 明示） | `record:55`；`review:63` | 标 **（材料未声明本块复核，应用时需补/或注明为历史值）** |
| 11 | A1a ≤6.2% 上限（judge 盲区 3 提及） | 继承更早工作块 | — | **本块未复核** | `review:63` | 草稿**未引用**（如需引用须先补核对） |
| 12 | −3.4% 百分比本身 | = (476−493)/493，两数均出自 `record:20` | 同 #2 | 同 #2 | `record:20,41` | 算术可核 ✅；结论限定「趋势」 |

**分层声明复核**：草稿两段正文均写明——指纹门 = **Full**（逐 chunk 位级哈希、噪声无关）；A1a 等价性 = **Full**（三维全格计数）；收益/附带读数 = **Degraded**（噪声带内只报趋势）；**写回路径 = 未覆盖（降级）**（`record:87`、`a2:51`）。与材料声明一致，无升格。

---

## 4. §15.4 取代链建议（原结论正文不删不改，加取代指针）

> 依据：材料明确出现「早前说法不成立」的项。以下指针**建议**由主会话应用；本草稿不修改任何既有行。

| # | 被取代的原陈述（锚点） | 取代内容 | 建议指针文本（追加到原处，不删原文） |
|---|---|---|---|
| S-1 | `10:3103` ①「nether/end 全维行为门未做（指纹门只覆盖 overworld 4140/4225 = 98.0%）」 | A2 已建三维载体与门（`a2:18-22`）；**且「4140/4225 = 98.0%」的分母口径被 judge C3 判为应改为「经接管 chunk 数」**（`a2:26`、`review:22`） | `⬆️ 260911-05 A2 取代：nether/end 全维行为门已做（nether/end 各 4761/4761 逐 chunk diff=0）；原「4140/4225 = 98.0%」分母口径作废——分母应为**经接管 chunk 数**（≠ Chunky Processed），门判据不依赖分母。写回路径仍未覆盖。` |
| S-2 | `10:3103` ⑥「85 个『Chunky 已处理但无接管行』chunk 与 621 格空白成因未查」 | 「85」算式不成立（期望集不是 Processed 集）；实测 overworld 相对 nether/end **少 621 条载体**，净值成因未查、降为 open | `⬆️ 260911-05 A2 取代：「85 个已处理但无接管行」说法不成立（judge C12）；实测 overworld 相对 nether/end 少 **621 条载体**（4140 vs 4761，同 center/radius/Processed），净值成因未查，降为 open 假设。` |
| S-3 | `07:601`「修复待办（clamp 改 `if (threads > count && count > 1)` 或实机改批量调用）」；`10:1475` 同源待办行 | Rust 共享层已实施 **`threads.min(count).max(1)`**（与待办建议写法不同）+ 前提（常驻池）消失（`record:60-64`、`review:42`） | 见 §2.3 可选补丁（`07:601` 追加一行；`10:1475`/`10:1485` 由主会话裁决是否同加） |
| S-4（延续，非取代） | `10:3103` ⑤「b3『写回/并发路径非确定』未被排除」 | A2 仍**未排除**（写回后指纹三维均未做）⇒ 状态不变，仅延续登记 | 无需指针（本块 open ① 已承接）；如需可加「⬆️ 260911-05：仍未排除（写回后指纹三维均未做）」 |

---

## 5. 未核 / 存疑清单（**不替它们补论证**）

1. **提交号 `6b90998`（A1b）与 `e8decef`（judge 修正）在三份源材料中零命中** —— 材料内可核的提交号只有：`b53b23f`（A1d diff，`review:42`）、`b2b2f26`（`review:21`）、`8d8075a`（`review:12/28/47`）；本 worker **无 shell/无 git**，无法核验 ⇒ 应用前 MUST `git log --oneline -10` 核（若不匹配，改号并在块内注明）。
2. **A1d「改前 `nthreads` = 10」为公式推导、非实测**（旧 dll 无 `[WG-THREADS]` 自证行；judge 全 `.tmp` grep 确认仅新 dll 两条）——材料已如实声明。
3. **`dd3b645f` 是否确为 A1d 前构建** = 按「18:02 运行 vs 19:36 提交」时间线**推断**，未反汇编确认（judge 盲区 2）。
4. **f2 成本拆分（A1c 0.42ms/chunk = 0.4%；A1a ≤6.2% 上限）来自更早工作块、本块未复核**（judge 盲区 3）⇒ 草稿只引用 0.42ms 并加标注，刻意**不引用** ≤6.2%。
5. **A1a 收益数字（493→476）未在关 WBCHECK 条件下重跑** ⇒ 只能作「含诊断成本的收益下界 + 趋势」（`record:22`、judge C6）。
6. **A1a 等价性为「单 run 单维、无负对照、与计时同 run」**（judge 核对表）——三维虽已实证，但无负对照。
7. **dll 硬门禁只比 16 hex 前缀**（`record:11`、judge I2）⇒ 「执行体一致」的粒度弱于全 sha256。
8. **写回后内容指纹（post-write hash）三维均未做**（`record:94`、`a2:56`）⇒ 「buffer 层等价 ≠ 写回后等价」这条边界必须在两处正文保留。
9. **nether run 级非确定的成因域未测**（`a2:57`、judge C8）⇒ 三个未测候选只可作「未测候选」列出。
10. **overworld 少 621 条载体的成因未查**（`a2:57`）。
11. **A2 附带读数跨 run 摆动 ±27%**（`a2:39`）⇒ 「~2.8× / ~2.0-2.5× 形态效应」只能作方向性；与既有端到端口径的可比性材料未声明（见 §3 #8）。
12. **1.21.6 侧同构问题（Phase B 范围）未处理**（judge 盲区 5）；**A1 改动尚未随任何 release 出货**（1.0.29 不含 A1，`record:96`）。
13. **judge 未编译、未运行 gradle/cargo**（`review:66`）⇒ 所有复核为产物 + 只读脚本重跑层；本块无新增编译/运行证据。
14. **judge 未审 `.investigations/e5-recompute-260911-05/`（B1，超范围）**（`review:67`）。
15. **块号/时点**：`260911-05` 标签同日被 A 线 / B1（E5 复算 19:37-20:52）/ C 线（20:56-22:3x）三块共用；精确时点区间「19:2x–20:5x」材料未声明（材料内仅见「18:02 运行 / 19:36 提交」，`review:62`）。
16. **A2 §1 早前误引的提交号 `3c1f9c6` 不存在**（judge C9 已改正为 `8d8075a`）——引用时勿回溯误号。

---

## 6. 应用前自检（逐条核对，应用后请照抄进签核证据）

1. **主题篇未堆时间线**：07 新小节只含「形态 / 判据 / §9.7 三要素 / ❌ 排除清单 / 遗留」；judge 逐条处置、C1-C12 明细、块号讨论**全部留在 10 时间线块与 `.investigations/`**，07 只有一行交叉引用。✅
2. **结论与过程分流**：结论性内容 → 07（A1a/A1b/A1d/A2/❌ 清单）；过程、被推翻假说（A1c 前提证伪链、C7 假通过复现、C12 算式作废）、工具演进、open → 10 时间线 + `.investigations/`。✅
3. **candidate 状态未被升格**：两段正文一律 `candidate`；`confirmed` 明确写「留人类」；无任何「confirmed / 已验证通过」措辞用于 AI 结论；A1c 用 ❌（放弃）而非 ✅/confirmed。✅
4. **未出现编造数字**：所有数字均带 `文件名:行号`（§3 表逐项列出）；两个材料零命中的提交号进 §5-1 存疑而非静默采用；缺三要素处一律标「（材料未声明，应用时需补）」，未代填任何分母/口径/百分比。✅
5. **§9.7 三要素完整性**：可直接引用的只有 §3 #5/#6（A2 门，三要素齐备）；其余缺项处均已就地标注。✅
6. **A1a/A1b 未写成「已消失」**：两段正文均在引文行 + 07 小节显式声明「旧路径经 `-Dcoreswap.bulkwb=0` 保留不删」。✅
7. **锚点可核**：10 篇追加锚点 = 行 3154 之后（EOF，total 3154）；07 篇主锚点 = 行 1289 之后（EOF，total 1289），备选锚点 = 行 1233 之前；A1d 补丁 = 行 601 末尾追加。三处文本均已在草稿写出前后文一行。✅
8. **A1d 归口已给理由**：07 + 07:601 指针（理由 = 07 承载 clamp 课题待办；03 篇无论证归口，grep 零命中）；A1c 才判「只进时间线」（07 留一行 ❌）。✅
9. **模块边界 / 落盘纪律**：本草稿只写 `.investigations/`；未改 docs/knowledge；未触碰 confirmed；AI 不授予状态。✅

---

> **本文件自身状态**：draft（草稿，未应用）。应用后建议状态：A1 / A2 均 **candidate**（judge 建议保持；confirmed 由人类授予——`review:71`）。
