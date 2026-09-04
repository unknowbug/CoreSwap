# 审查意见书：fanout-writer-260904-06 候选包（OOB dirt/sand 写者课题）

> 审查者：core.judge（subagent，260904-06）。只出意见，不改任何 status / 产物。
> 审查对象：b1-surface-oob-writer.md（§1-§10，R1-R7）、b2-java-feature-placement.md、d-preliminary-surface-cmp.md、evidence-pack.md、convergence-260904-06.md、cmd-output/。
> **总裁定：PASS-with-conditions**（2 项 MUST 条件 + 3 项 SHOULD 条件，见 §5）。

---

## 1. 三源核对

### 1.1 产物快照 vs git ✅
- `git status --short`：新增 `.investigations/lossless-accel/fanout-writer-260904-06/`（未跟踪，含全部受审产物）+ `000-架构设计/架构计划-260904-06.md`；`git log -3` = 9157191 / ecba4bc / 759d556——ecba4bc 即上轮 writer-verdict（被本轮取代的「双跑伪影 best-explanation」）原始提交，取代链起点可溯源。✅
- 申报引用文件全部在盘（glob 实证：b1/b2/d/evidence-pack/convergence + 6 个 cmd-output 文件）。✅

### 1.2 关键数据抽核（抽 5 项，全部与 cmd-output 原文一致）✅
| 申报 | 原文核验 | 判定 |
|---|---|---|
| E1 臂 (195,199) 列 dirt/sand/gravel 空 | e1-skipsurface-result.txt L22 `[E1] column (195,199) dirt/sand/gravel: []` | ✅ |
| E2' sha 2ff71249 两臂相同 | e2-estshared0-result.txt L1 `e2 sha: 2ff71249e9c5cf97  off sha: 2ff71249e9c5cf97`；L2 列零差异 | ✅ |
| est 四角 = 24 | .tmp/p2full/e3-est-dump.csv 首批行全部 `c0..c3:...:24`（在盘可引用） | ✅ |
| biome 七点 deep_lukewarm_ocean | cpp-biomedump-244col.txt L2-8，(244,-60..-54,244) 七点全同 | ✅ |
| id 970 = deepslate | e4lite-id970-columns.txt L1；E1 臂最高 solid y=318（L27）、col(198) 水口袋 @197/212/228/244/261/277（L36-56）与 R7 陈述逐点吻合 | ✅ |

### 1.3 验证记录（seed 三查 / dll / arm 标识）✅（一项弱化见 §3-C1）
- e3-e5-run-full.log L113-117 与 e3-rerun-full.log L112-116：`init seed / initNether seed / BlockProbe seed / worldSeed` 四处全 = 8576294172403134396；dll sha256=ec4a9aedf4104788（1897984B）两日志一致；stageMask=3（L113）。e2-run-full.log 亦有 worldSeed 行。✅
- §9.7 口径三要素声明：**齐**。b1 §8.2 对 243/111 vs 43/11 明确声明①载体变（Rust NOISE-only vs mod FULL）②方向反③双维混合，判定不可比——这是上轮 judge 教训（执行体口径）的正确落实。b2 §5 的 A−B 四源混淆声明同族。✅
- **弱化项（C1）**：E1 臂的 skip 激活证据引用了 log `[Mixin] buildSurface skipped`——经核 NoiseChunkGeneratorMixin.java L95，该行是 **mixin 常规 cancel 日志（CppBridge.enabled 即打，E3 各臂日志同样满屏此行）**，不能作为 WG_SKIP_SURFACE=1 生效的独立证据。E1 的 env 激活目前只有行为旁证（列清空 + 243 方向翻转），无日志级直接证据。不推翻结论（行为证据强），但引用必须修正。

## 2. 逻辑链核查

### 2.1 结构消去链闭合 ✅
mask=3 下 NOISE（terrain.rs 只写 air/stone/water/lava）/ ore_vein（铜铁）/ aquifer（水/岩浆）/ features（bit1 跳过）逐阶段排除 + E1 运行时确证（关 surface → dirt 消失）+ E2' 排除 est 路径混淆变量。**写者身份的证据链完整且闭环。**

### 2.2 R1-R7 取代链 ✅
- 七条取代记录均有「被取代结论原文位置 + 一行理由 + 依据」，原正文（§2/§3/§8/§9.4）未删未改、保留前向指针——符合 §15.4 形态。R2（est=24 推翻 k≈208-224）、R3（E2' sha 同证伪 est 路径分叉）、R6（biome 七点证伪 m3）均有 cmd-output 原文支撑。✅

### 2.3 E3 零输出矛盾登记 — **不完整（MUST 条件 C2）**
- b1 §10.2 列了三个解释，把「chunk(12,12) 未生成（懒生成/范围外）」列为最可能，并要求复跑裁决。
- **复跑已发生**（e3-rerun-full.log，点集扩至 7 点，L126）：结果仍仅 244 行输出（L834），且 **log L682-683 实证 chunk(12,12) 已生成、populateNoise 已拦截、Rust surface 已跑、dump 已启用**——解释 #1「chunk 未生成」被复跑日志**直接证伪**，但 b1 §10.2 **未回填复跑结果**，被证伪的假设仍挂着「最可能」标签。
- 判定：矛盾本身已被部分登记（idk 意识存在），但登记不完整——MUST 在结案/移交前补一条取代记录（R8）：「E3 复跑零输出 + chunk(12,12) 已生成 → 解释 #1 证伪，剩余 #2（运行链路/dll 代次差）/ #3（坐标口径差）待裁决，未定位前保持 idk」。

### 2.4 b2/d 与 b1 的相互矛盾处理
- **b2 vs b1**：无硬矛盾。b2 诚实声明对三关键形态无解释力（周期 16 / 7 深 sand / 负 y dirt），其 §3.2 的「结构写者未排查」方法论发现是有价值的独立贡献（见 C4）。
- **d vs b1 R4/R6**：无直接冲突但有**未整合的分叉**（MUST/SHOULD 边界，判 SHOULD 偏 MUST，列 C3）：d §0/§2 把真分叉定位在 heightmap 填充判据（C++ `block!=air` 含水 vs Rust `d>0` 不含水，仅开放海洋列触发），并论证 195 列（幕帘高位、顶块 stone）不由 preliminary surface 家族解释。这直接关系 R6 的 (244) 归因框架（d 承认可贡献 fluid_height/WaterCond/steep 谓词输入差）与 (m2) 的最终裁决（d 实质裁决了 m2：est 层零差、heightmap 层有差且域已收窄）——但 **convergence-260904-06.md 仍写「待 (d)：运行中」，d 的结论完全未进入收敛摘要与 b1 R 链**。移交新线时若不整合，新线的「幕帘差异全归 NOISE」前提对海洋列不成立。

## 3. 已知风险点复查（上轮 judge 教训）

- **执行体口径**：各臂 dll（ec4a9aed）/mode（stageMask=3）/env（WG_SKIP_SURFACE / WG_EST_SHARED=0 / SOUL-CTX 点数 5→7）在 e3 两日志中声明到位；E1/E2 结果文件无内嵌 header（弱，见 C5）；E1 的 skip 激活日志证据无效（C1）。
- **C-gate / evidence saturation**：逐轮核对——初始探针→E1→E2'/E3/E5→E4-lite/biomeDump，**每轮都有新数据层证据（probe/dump 级），无 3 轮空转**，retry cap 合规。✅

## 4. 置信度与落盘契约

- **status 合法**：全部产物自标 draft；b1 §10.3 为 candidate「申报」而非授予；无任何 confirmed。✅
- **落盘契约**：.investigations/ 齐全；但本课题结论均尚未进 .artifacts/index.yaml（writer-verdict-260905 在 ecba4bc 有登记，本轮取代后 **.artifacts 侧的取代登记未见**）——主会话在授予 candidate 后应同步登记 .artifacts（含 supersedes 双指针）。列条件 C6（流程项）。
- **模块边界**：无跨模块 skill 正文引用；b2/d 引用 Java/C++ 源码 file:line 属领域数据非 skill 正文。✅

## 5. 条件清单与结论

**MUST（授予 candidate 前补齐）**
- **C1**：修正 E1 证据引用——`[Mixin] buildSurface skipped` 是常规 mixin 日志非 WG_SKIP_SURFACE 证据；E1 skip 激活改以行为证据（列清空 + 243 方向翻转）+ 建议后续在 Rust skip 分支加一条独立日志。
- **C2**：b1 补 R8 取代记录：E3 复跑结果回填（e3-rerun-full.log L126/L834 + L682 chunk(12,12) 已生成），撤销解释 #1「chunk 未生成」，剩余 #2/#3 明示为 idk。

**SHOULD**
- **C3**：d 的结论（est 零差 / heightmap 判据分叉仅开放海洋列 / 195 列指向 m1 或规则树差）整合进 convergence 与 b1 R 链（R8 或交叉引用），convergence 的「待 (d) 运行中」更新。
- **C4**：新调查线移交文本补入 b2 §3.2 的结构写者警示——vanilla ref 深 dirt（-59..-42）可能来自 structures，新线「vanilla vs NOISE-only 三方对比」必须控制结构写者混杂，否则幕帘结论可被污染。
- **C5**：今后 cmd-output 产物统一内嵌 header（seed/dll sha/env/点集），消除 E1/E2 臂三查只能靠 convergence 转述的弱点。

**candidate 授予建议**
- ✅ **「写者 = Rust SURFACE」建议授予 candidate**（限定于写者身份本身）：结构消去链 + E1 运行时决定性实验 + E2' 混淆变量排除 + 关键数据五项抽核全对。机制细项（SurfaceCondC 输入分叉、16 周期触发输入、E3 零输出之谜）**保持 draft/idk**，不随写者结论升级。
- ✅ **「双跑伪影」假设结案 + NEXT_SESSION 前提推翻的取代记录：通过**（b611fcb 时间戳 < dll 构建时间戳有实证，原 verdict 未改写）。
- ✅ **新调查线移交建议：支持**（附 C4 条件；d 的 heightmap 分叉建议一并纳入新线范围，海洋列谓词输入差属 surface 层遗留）。

> 本意见书只表达审查结论；status 变更与 confirmed 授予权在宿主人类。
