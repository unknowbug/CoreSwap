# 草稿：里程碑「运行环境迁移 + Rust/Java 性能基准真正确认」— 结论 docs 草稿

> **用途**：给主会话的结论 docs 草稿（knowledge subagent 产出，主会话应用 + 验证）。本文件是「草稿容器」，非最终落盘载体。主会话按「载体映射」把各段落到位到对应知识库文件，并做一致性扫描验证。
>
> 错误台账本体：`.investigations/perf-e2e/perf-e2e-errors.md` 已追加 **P5**（五段式 + 速查表一行）。
>
> **本里程碑核心是「结论修正」**：早前「Rust 慢 Java 5 倍」被大样本推翻（Rust 全管线反而快 ~1.2 倍）。因此本草稿是 **07 篇既有小节 + 10 时间线既有条目的修正稿**，不是纯新增。主会话须按修正稿更新既有内容（遵守「追加不覆盖 / 被推翻假说保留 ❌ 排除清单」纪律，把错误基准下的旧读数标注为历史保留）。
>
> **载体映射**（按「记录价值门」分层）：
> - **性能基准真正确认（高价值教训）** → `versions/1.20.1/docs/07-block-pipeline.md` 端到端小节**修正** + `10-timewise-archive.md` 追加 2026-08-29 修正条目；教训 P5 完整沉淀于错误台账。
> - **运行环境迁移（中价值可复用模式）** → `10-timewise-archive.md` 追加条目；可复用模式（gradle home 放工作区免提权）考虑同步进 `knowledge/discovered/build-tooling.md`「发现 #4」（跨版本通用，见第三部分）。
> - **一次性对齐数值**（低价值快照）：Java FULL 55ms / Rust 45.48ms 等当前数字会随优化变化，07 篇**只记结论性要点**（Rust 反快 ~1.2× / aquifer 慢 ~1.4× 需优化），不复述会变化的快照细则。

---

## 一、07 篇「Rust worldgen 端到端性能定位」小节修正草稿

> 修正对象：`versions/1.20.1/docs/07-block-pipeline.md` L785「2026-08-29 Rust worldgen 端到端性能定位（aquifer 是最大头，整体慢 Java 5 倍）」小节。
> **修正方式**：小节标题去掉「慢 Java 5 倍」（错误结论），改为反映真实结论；「端到端对比（Java 充分预热）」子节按下方修正稿更新；`/根根本洞察` 中「慢 5 倍」表述改为修正后表述；旧读数保留标注为「被大样本推翻的历史快照」。
> **注意**：07 篇 06-08-29 小节内「aquifer 内部精确定位」「宏观采样对齐」等小节（L813-839）**不在本次修正范围**（本次修的是「端到端对比的数字结论」，aquifer 内部分析与宏观采样方向不受端到端反转影响，仍有效）；但「根本洞察：Java 宏观网格 vs Rust 逐点 ~80× 采样差是慢 5 倍的根本」**需要修正**——该归因基于错误的「慢 5 倍」，真实是 aquifer 慢 ~1.4×（宏观专项），不是整体慢 5 倍。主会话需核对 L829-833 归因文本一并更新。

```markdown
## 2026-08-29 Rust worldgen 端到端性能定位（大样本修正：Rust 全管线反快；aquifer 宏观仍慢需优化）

> 背景：Rust 全量重写 worldgen（WorldgenRust/）功能链闭合后进入性能定位。本小节记性能定位结论与优化方向（中价值）；错误链条见 .investigations/perf-e2e/perf-e2e-errors.md（P1-P5）。
> ⚠️ **本小节含重大修正**：早前「Rust 慢 Java 5 倍」结论（下方【历史快照】）基于「Java 8-9ms」错误基准（P5），被大样本推翻。

### 端到端修正对比（大样本，region 200,200，2026-08-29）

- **Java FULL（256 chunks，含树花一切，充分预热）**：≈ **55ms/chunk**（稳定 54-57ms，avg 51.7 含冷启动）。
- **Java 宏观 NOISE（256 chunks）**：≈ **23-25ms/chunk**（avg 25.4，稳定 20-27ms）。
- **Rust 全管线（400 chunks，无树花）**：**45.48ms/chunk**。
- **Rust 宏观（400 chunks，density+aquifer）**：**34.66ms/chunk**（aquifer 增量 ~21.5ms）。
- ✅ **修正结论**：**Rust 全管线 45.48ms < Java FULL 55ms → Rust 反而快 ~1.2 倍**（尽管 Rust 无树花做更少工作）。「Rust 慢 5 倍」不成立。
- ⚠️ **但宏观专项 Rust 34.66 > Java 23-25 → aquifer 慢 ~1.4-1.5 倍**（真实差距，需优化）。
- 后续阶段（carver/features）Rust 应比 Java 更省（Java FULL 的宏观 ~25 + 后续 ~30ms）。

### 域/边界

- 验证分层 = Partial；数值为当前快照，随优化变化。端到端对比必须用充分预热的 Java 基准（AGENTS.md 铁律）+ **大样本排除缓存/冷启动**（P5 教训）。

---

### 【历史快照 · 已被大样本推翻，勿再引用为当前结论】端到端对比（Java 充分预热，早期小样本）

> 这一段是早前结论，被 P5 推翻，保留作历史排除清单：
- ❌「Java 原版稳定 8-9ms/chunk」——16 chunks 小样本 + 相邻 chunk 缓存假象，真实 55ms（6 倍低估，见 P5）。
- ❌「Rust 44.9ms 慢 Java 5 倍」——基于错误基准的错误结论，已被大样本修正推翻。
- ⚠️「Java 60ms 是 JIT 未热错误基准」这一半仍成立（P3）；但「真实 Java 只有 8-9ms」这一半错误（P5 修正为 55ms）。
```

---

## 二、10 时间线追加条目草稿（修正 + 双里程碑）

> 追加位置：`versions/1.20.1/docs/10-timewise-archive.md` 末尾（L2201 后）。

```markdown
## 2026-08-29 运行环境迁移到 CoreSwap（免提权）+ Rust/Java 性能基准真正确认（大样本推翻「慢 5 倍」）

> 承接 07 篇「Rust worldgen 端到端性能定位」小节修正 + .investigations/perf-e2e/ + perf-e2e-errors.md 错误台账（P1-P5，本次新增 P5）。

### ✅ 一、运行环境迁移到 CoreSwap（免提权，b2b9bea + 50ba9a4）
- 原运行环境在 MC 侧（E:\PYTHON\MC\versions\1.20.1\java），每次 gradle 运行需 **danger-full-access 提权**（native-platform.dll 在 C:\Users\NDark\.gradle 外部）。
- 迁移三步：
  1. `git mv versions/1.20.1/java → runtime/1.20.1/java`（验证 client 作为独立 runtime，与数据/参考 versions 分离）——versions 回归纯数据/参考（cpp/data/docs），runtime 承载运行环境。
  2. gradle home `C:\Users\NDark\.gradle`（2.6GB）→ `CoreSwap\.gradle`（robocopy 秒级），native-platform.dll + 依赖缓存在工作区内。
  3. `GRADLE_USER_HOME=CoreSwap\.gradle`（run_rust_client.ps1 设 `$runJava=runtime\1.20.1\java` + GRADLE_USER_HOME）→ **gradle classes 编译 + runServer 启动 + bench 探针全免提权**。
- bench.out 默认改 CoreSwap；.gitignore 更新（runtime java build/cache + .gradle home 忽略）。
- **原理**：gradle home 放工作区 → native-platform.dll + 依赖缓存在沙箱可见区内 → 免提权。

### 🔄 二、端到端基准重大修正（P5，推翻「慢 5 倍」）
- 早前「Java FULL 8-9ms → Rust 慢 5 倍」是**小样本（16 chunks）+ 相邻 chunk 缓存假象**（顺序生成相邻 chunk 复用缓存，getChunk(FULL) 缓存共享），与 P3（JIT 未热）同族——**基准不可靠连续两次**。
- 大样本修正（region 200,200）：Java FULL（256 chunks）≈ **55ms/chunk**；Java 宏观 NOISE（256 chunks）≈ 23-25ms；Rust 宏观（400 chunks）**34.66ms**；Rust 全管线（400 chunks）**45.48ms**。
- ✅ **修正结论**：**Rust 全管线 45.48 < Java FULL 55 → Rust 反快 ~1.2 倍**（「慢 5 倍」不成立）；但**宏观专项 Rust 34.66 > Java 23-25 → aquifer 慢 ~1.4-1.5 倍（真差距需优化）**。
- ❌ **铁律更新**：早前「端到端性能对比铁律」主张「充分预热 Java（8-9ms）为准绳」——该数值已被大样本修正；铁律保留「端到端对比充分预热的 Java」核心，**追加「必须大样本 + 排除缓存/冷启动」**（P5 教训）。
- 数据源：`cmd-output/java_full_correction.txt` / `cmd-output/macro_java_vs_rust.txt` / `cmd-output/fair_comparison_corrected.txt`。

### 📌 记录指引
- 错误台账：`.investigations/perf-e2e/perf-e2e-errors.md` P5（五段式 + 速查表）。
- 结论：07 篇「Rust worldgen 端到端性能定位」小节修正（Rust 反快 / aquifer 宏观慢需优化）。
- 通用模式：`knowledge/discovered/build-tooling.md`「发现 #4」（gradle home 放工作区免提权，跨版本可复用）——见本草稿第三部分。
- 域边界：数字 = Partial 快照（随优化变化）；aquifer 宏观优化（对齐 Java 网格 / aquifer 每点开销）= candidate 待立项。
```

---

## 三、通用模式草稿（discovered/build-tooling.md「发现 #4」可选落盘）

> **价值门**：中价值（跨版本/跨项目可复用的工具链模式，"gradle home 放工作区避免沙箱提权"）。若主会话觉得单次环境配置无跨项目复用价值，可不落盘（留给 .investigations/ 即可）。以下为可选草稿（discovered 格式按 knowledge/INDEX.md 写入规则）。

```markdown
## 发现 #4: gradle home 放工作区内（GRADLE_USER_HOME 指向项目 .gradle）→ native-platform.dll + 依赖缓存在沙箱区内，免提权

**发现时间:** 2026-08-29
**发现者:** knowledge subagent（运行环境迁移里程碑）
**来源定位:** b2b9bea（git mv versions/1.20.1/java → runtime；gradle home C:\Users\NDark\.gradle → CoreSwap\.gradle）
**置信度:** confirmed
**module:** build / env

### 观察
gradle 默认 home（`C:\Users\...\.gradle`）位于项目外部，其 `native-platform.dll`（gradle 原生平台库，位于 ~/.gradle 外部）与依赖缓存不在沙箱可见区内 → 每次 gradle 运行需 danger-full-access 提权。把 gradle home 整个迁到工作区内（robocopy 现有 home + 设 `GRADLE_USER_HOME=项目\.gradle`）→ 原生库 + 依赖缓存在沙箱区内，gradle classes 编译 / runServer / bench 探针全免提权。

### 证据
- 迁移前：`E:\PYTHON\MC` 下每个 gradle 运行需 danger-full-access（native-platform.dll 在 `C:\Users\NDark\.gradle` 外部）。
- 迁移后：`GRADLE_USER_HOME=CoreSwap\.gradle`（robocopy 秒级复制 2.6GB home）+ `git mv versions/1.20.1/java → runtime/1.20.1/java` → gradle classes + runServer + bench 探针**全免提权**（b2b9bea + 50ba9a4）。
- run_rust_client.ps1 设 `$runJava=runtime\1.20.1\java` + `$env:GRADLE_USER_HOME=CoreSwap\.gradle`。

### 如何利用
- 沙箱/受限环境跑 gradle：把 gradle home 迁进工作区（robocopy/copy 现有 home，避免重新下载依赖），`GRADLE_USER_HOME` 指向之，native-platform.dll + 依赖缓存在可见区内 → 免提权。
- 配合把验证工程作为「独立 runtime」目录（如 versions/ 回归纯数据/参考，runtime/ 承载运行/验证环境）——运行环境与数据/参考分离，职责清晰。
```

---

## 四、主会话应用清单（自检）

- [ ] **错误台账**：`.investigations/perf-e2e/perf-e2e-errors.md` P5 已追加（本 subagent 直接落盘为草稿，主会话已可引；如需合并校验请复核五段式 + 速查表行）。
- [ ] **07 篇**：按 §一 修正「端到端性能定位」小节——改标题（去掉「慢 5 倍」）、更新「端到端对比」子节数字、修正「根本洞察 ~80× 采样差 = 慢 5 倍的根本」归因（真实是 aquifer 宏观慢 ~1.4×）、旧读数标「历史快照 ❌ 排除清单」。aquifer 内部分析/宏观采样方向小节保留（不受端到端反转影响）。
- [ ] **10 时间线**：按 §二 追加 2026-08-29 条目（环境迁移 + 基准修正双里程碑）。早前 2026-08-29「端到端性能定位（慢 5 倍）」条目保留，标注「已被大样本修正，见新修正条目」。
- [ ] **discovered**：按 §三 可选落盘 build-tooling「发现 #4」（gradle home 放工作区免提权）。同步 INDEX。
- [ ] **铁律**：AGENTS.md「端到端性能对比铁律」需追加「大样本 + 排除缓存/冷启动」（P5 教训）；「Java 充分预热后只要 8-9ms」数值已被修正为 55ms，主会话需核对铁律条文更新。
- [ ] 一次性对齐数值（Java 55ms / Rust 45.48ms / aquifer 34.66 等）在 07 篇只记结论性要点，不复述变化快照细则（低价值快照不写 docs 细则）。
- [ ] 应用后跑一致性扫描（确认无时间线式章节误入主题篇；数字与 cmd-output 三份记录一致）。
- [ ] 结论 candidate → 主会话验证 / judge 审查 → 用户拍板 confirmed 后方可标 confirmed。
