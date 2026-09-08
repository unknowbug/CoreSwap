# W1 scout 摸底——C1 豁免清单 B3「SPAWN barrier 差异」溯源与机制摸底（架构计划-260908-14 P6）

> 只读勘探（recode 域 scout）。置信度：**draft**（静态溯源 + 静态机制核对，Degraded 分层——无运行时探针）。
> 禁触 E:\PYTHON\MC（未访问）。除本文件外零写入。

## 1. B3 首次取证溯源（任务 1）

### 1.1 首次具体取证 = w2-chunk-pipeline.md（260908-08 块，静态源码 diff）

- **最早的具体机制描述**：`.investigations/mc-1216-port-260908-08/diff/w2-chunk-pipeline.md:30`——
  「**SPAWN 是明确的语义变化**：旧 taskMargin=0（ChunkRegion radius -1），新 `SPAWN.dependsOn(BIOMES, 1)` → 邻域 ring-1 需要 BIOMES 级 chunk 才能推进 SPAWN——**邻居推进的 barrier 变了**」。
- 同文件逐处变更表 `w2-chunk-pipeline.md:51`（「SPAWN dependsOn(BIOMES,1)｜数据级（语义变化）｜旧 margin 0；邻居推进 barrier 变化」）与裁决表 `:64`（「1 处语义变化 = SPAWN 邻居依赖 0→ring-1@BIOMES」）。
- 遗留验证要求原文 `:67`：「SPAWN barrier 差异需运行时验证（本结论 Degraded 分层，未做行为探针）」——这就是 P6 探针挂账的源头。
- 工单化时已自带裁决性限定词：`.investigations/mc-1216-port-260908-08/port-list-260908-08.md:15`——「P6 | SPAWN barrier 差异（**0→ring-1@BIOMES**）运行时探针验证」。**「0→ring-1@BIOMES」六字就是权威歧义裁决**。

### 1.2 传播链（首次取证 → B3 条目）

1. 260908-08：w2-chunk-pipeline.md（上述，首次取证）。
2. 260908-09：架构计划 C1 清单挂名——`.investigations/000-架构设计/架构计划-260908-09-mc1216-phase3.md:24`（豁免清单列举「…/SPAWN barrier」，只出现名词，无机制描述）；P6 挂账 `:13`。
3. 260908-10：`.investigations/000-架构设计/架构计划-260908-10-mc1216-phase3b.md:14,40`（同上，名词挂账）+ `.investigations/mc-1216-port-260908-10/progress-260908-10.md:71`「1.21.6 SPAWN barrier 差异只在 Java 调度层，不影响 Rust 生成正确性」。
4. 260908-12 收口：`.artifacts/mc-1216-port/c1-exemption-list.md:23` B3 行——机制列写「**spawn 区 barrier 放置**」。

### 1.3 歧义裁决（三候选）

| 候选 | 裁决 | 依据 |
|---|---|---|
| aquifer barrierNoise（含水层屏障噪声） | ❌ 排除 | w2 首次取证全文无 aquifer 语境；aquifer 差异另由 C1 A1（F1 修复，confirmed 260908-11）覆盖，归属不同条目 |
| surface/地形 barrier 块（bedrock/barrier 块放置） | ❌ 排除 | 无任何一手记录描述过「spawn 区 barrier 块」现象；1.21.6 ChunkStatus/surface diff（w2 全文）零涉及 |
| **ChunkStatus.SPAWN 邻居依赖语义（调度 barrier）** | ✅ **成立** | 首次取证机制描述（w2:30/51/64）+ 工单限定词「0→ring-1@BIOMES」（port-list:15）+ P5 一手源码复证（见 §2.2） |

**B3 精确定义（溯源后重建）**：1.21.6 把 ChunkStatus 邻居依赖从单一 taskMargin 改为逐依赖表后，SPAWN 阶段邻居门槛从 0（1.20.x taskMargin=0）提高到 ring-1@BIOMES（`SPAWN.dependsOn(BIOMES, 1)`）——**这是 chunk 调度推进的依赖 barrier 变化，不是任何方块放置差异**。c1-exemption-list.md:23 B3 行机制列「spawn 区 barrier 放置」为**转抄漂移**（「邻居推进 barrier」→「barrier 放置」），语义失真；「影响域：spawn 区」同样失真——该差异影响的是任意 chunk 推进到 SPAWN 的调度条件，与出生点区域无绑定（SPAWN status 是全管线的第 11 阶段，全服所有 chunk 都要过）。

**溯源完备性**：首次取证存在且具体（现象=依赖表 diff、坐标语义=依赖半径、载体=静态源码 diff），**无需「定义重建后回用户拍板」的兜底路径**（架构计划-260908-14 风险项 R2 不触发）；但机制列转抄漂移需要修正登记。

## 2. 机制摸底（任务 2）

### 2.1 Java 侧（1.21.6 一手源码，p5-verdict-260908-13 已复证）

- `versions/1.21.6/data/mc_src_extract/net/minecraft/world/chunk/ChunkStatus.java`：1.21.6 本体无 `range`/邻居半径字段（1.20.x 有）——语义迁出（`.artifacts/mc-1216-port/p5-verdict-260908-13.md:15`）。
- 迁移落点 = `ChunkGenerationStep.java` + `GenerationDependencies.java` + `ChunkGenerationSteps.java`；GENERATION 链中 **SPAWN dependsOn(BIOMES, 1)**（p5-verdict `:19`；与 w2:30 一致，双源）。
- 该语义由 Java 调度层（ChunkLoader / GenerationDependencies.getAdditionalLevel → ticket 附加层级）消费。

### 2.2 Rust 侧消费点

- p5-verdict-260908-13（candidate，260908-13）本轮一手核对：薄壳 `jni_bridge.rs` 零 status 引用；`worldgen-core/src` 全量 grep `dependsOn/accumulated/write_radius/ChunkStatus/neighbor` **零功能性消费点**（唯一 neighbor 命中 = worldgen_handle.rs:889 carver 半径 8 扫描，是对 STRUCTURE_REFERENCES 依赖的功能性复刻，非依赖表消费者）（p5-verdict `:25-27`）。
- 本轮补充：`worldgen-core/src` grep `spawn` 全部命中为线程 spawn / bin-diag 注释/探针命名，**无任何 spawn 区特殊生成路径**（worldgen-core 对 spawn 区与任意区块一视同仁）。
- 推论：SPAWN 依赖 barrier 只影响 Java 侧「一个 chunk 何时能推进到 SPAWN/FULL」，不影响任何阶段生成的方块内容；且双臂（Rust 接管 vs 1.21.6 vanilla）**都跑同一套 vanilla Java 调度管线**，调度层零差异。

### 2.3 与 P5 的同族关系（→ 任务 4 显式声明）

- **B3 与 P5 是同一 diff 的两个视角**：P5 = 依赖表整体（全 status），B3 = 其中被 w2 标为唯一「数据级语义变化」的 SPAWN 行。B3 没有任何超出 P5 覆盖面的独立机制成分。
- P5 判定（p5-verdict-260908-13）已覆盖 B3 的全部机制问题：属 Java 调度层、Rust 零消费点、不影响 Rust 生成正确性。**P6 若按「barrier 块放置差异」假设去跑运行时双臂对拍，预期结果是零差异**（无块级现象可采），探针形态与 B3 实际语义错位。
- 建议处置：B3 走 **§15.4 取代/勘误记录关账**（c1-exemption-list 追加取代行：B3 机制实为 ChunkStatus.SPAWN 邻居依赖 0→ring-1@BIOMES，属 Java 调度层，已被 p5-verdict-260908-13 裁决覆盖，P6 运行时探针无对象），原 B3 行不改。是否保留「零成本 sanity 双臂」由用户拍板（见 §3.3）。

## 3. W2 探针形态建议（任务 3）

### 3.1 首选：不跑运行时探针，B3 以取代记录关账

依据：§2.3——B3=P5 同族且已被 P5（一手源码 + Rust 零消费点清单）裁决；w2:64 原文「对『复刻 1.20.1 行为』目标零行为迁移」。首次取证要求的「运行时验证」在 P5 落盘后已失去验证对象（调度语义双方同源 vanilla，无 Rust 参与面）。

### 3.2 若用户坚持 P6 跑运行时（低成本形态）

- **形态 A（调度层观察，正对 B3 真语义）**：双臂 runServer（Rust 接管 vs 1.21.6 vanilla，同 seed 三查），Done 后 forceload 同一小域，log 观察 ChunkLoader 对 SPAWN/FULL 的推进顺序/邻域 ticket——预期两臂完全一致。判据：两臂 per-chunk status 推进序列无差异 → B3 关账；有差异才触发 fan-out。成本中、信息量低（两臂调度代码同源，几乎必零差）。
- **形态 B（块级 sanity，正对 c1 清单转抄后的误语义）**：Chunky 双臂 spawn 区对拍，复用发现 #26 载体（`.investigations/jungle-l/chunky-trial-260906-09.md`）与 1.21.6 基线 625/3698（发现 #26 补充案例，`.investigations/mc-1216-port-260908-11/progress-260908-11.md`）。注意事项：
  - #80（workflow-patterns.md:1397）：spawn 区 chunk 在 SERVER_STARTED 前已 vanilla 生成 → 必须 **post-Done forceload 新区块**，spawn 区本身不作为采集对象（或仅作已知-vanilla 域排除）；
  - #50（workflow-patterns.md:871）：先核接管生效 chunk 范围（init 晚于 Done，pre-init 域是 vanilla 生成），采集集不得混入；
  - #88（workflow-patterns.md:1518）：复跑采集必删 world，「在盘」≠「会重新经过探针」；
  - 预期结果 = 零 barrier 块差异；若出现 spawn 区块差，归因对象是 B5/B6/I1 族（feature 未支持/残差），不是 B3——判读时先剔豁免集（review-001.md:26 的预期差异豁免要求）。
- 两形态都建议配 §9.7 声明（Chunky 区域级 vs 单点口径分开标注，架构计划-260908-14:33 已预置）。

### 3.3 建议

向用户提交三态：①推荐 = §3.1 取代记录关账（零运行成本，机制已双源闭环）；②备选 = §3.2 形态 B 顺带 sanity（若本块本来就要跑 Chunky 双臂其它条目，边际成本≈0）；③不推荐 = 形态 A（为已裁决的调度语义专门搭载体，性价比最低）。

## 4. 附带发现（登记，不改他人文件）

- `c1-exemption-list.md:20-21`：B1「Shipwreck Y 重定位」**整行重复两条**（260908-12 收口时引入的转录重复）——建议随 B3 取代记录一并勘误登记（追加不覆盖）。
- p7-e2e-verdict 实际路径为 `.artifacts/mc-1216-port-260908-10/p7-e2e-verdict-260908-10.md`（c1 清单内引用写作 `.artifacts/mc-1216-port/p7-...`，目录名有出入）；其 §4/§5 与 C1 清单引用对得上，仅路径引用需留意。

## 证据指针汇总

| 结论 | 指针 |
|---|---|
| B3 首次取证（机制描述） | .investigations/mc-1216-port-260908-08/diff/w2-chunk-pipeline.md:30,51,64,67 |
| 工单限定词「0→ring-1@BIOMES」 | .investigations/mc-1216-port-260908-08/port-list-260908-08.md:15-16 |
| C1 挂账名词化 | 架构计划-260908-09:13,24；-10:14,40；-11:17；-12:14,45；-13:13 |
| 转抄漂移（「barrier 放置」） | .artifacts/mc-1216-port/c1-exemption-list.md:23 |
| Java 一手语义 + Rust 零消费点 | .artifacts/mc-1216-port/p5-verdict-260908-13.md:15-27 |
| 「只在 Java 调度层」初判 | .investigations/mc-1216-port-260908-10/progress-260908-10.md:71 |
| Rust 无 spawn 特殊路径 | worldgen-core/src grep spawn（本轮，仅线程/诊断 bin 命中） |
| 探针注意事项 #26/#50/#80/#88 | knowledge/discovered/workflow-patterns.md:337/871/1397/1512-1528 |
| 载体先例 | .investigations/jungle-l/chunky-trial-260906-09.md；.investigations/mc-1216-port-260908-11/progress-260908-11.md |
