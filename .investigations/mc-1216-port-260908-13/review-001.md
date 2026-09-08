# core.judge 审查意见——P5 判定（260908-13）review-001

> 审查角色：core.judge（subagent，隔离）；只出意见，不改 status。
> 审查对象：`E:\PYTHON\CoreSwap\.artifacts\mc-1216-port\p5-verdict-260908-13.md`（draft，建议 candidate）。
> 审查基线（三源）：① artifacts 快照 + index.yaml diff ② git（HEAD=9057871，工作区：仅 index.yaml 修改 + verdict/计划两个未跟踪新文件，与本块范围一致）③ 独立抽查一手源码与历史记录。

## 逐项结论

| # | 审查项 | 结论 | 依据（独立复核） |
|---|---|---|---|
| 1 | 证据完整性（三源一致） | **通过** | verdict 引用的源码点逐一实读：`ChunkStatus.java`（111 行全文）确无 range/邻居半径字段，仅 previous 链 + heightMapTypes；`ChunkGenerationSteps.java` GENERATION 链的 dependsOn 声明与 verdict 列举逐条吻合（STRUCTURE_REFERENCES(STRUCTURE_STARTS,8)、BIOMES(STRUCTURE_STARTS,8)、NOISE(STARTS,8)+(BIOMES,1)+writeRadius 0、CARVERS(STARTS,8)+0、FEATURES(STARTS,8)+(CARVERS,1)+1、LIGHT(INITIALIZE_LIGHT,1)、SPAWN(BIOMES,1)）。`ChunkGenerationStep.java`/`GenerationDependencies.java` 实存。git 锚 9057871 与 verdict 头一致。 |
| 2 | 置信度合法性 | **通过** | index.yaml 登记 status: draft + 候选建议 candidate 注释；无任何 confirmed 越权。verdict 自己声明 Degraded 分层（静态审查），与计划「静态核对为主 → Degraded」一致。 |
| 3 | §9.7 覆盖面声明 | **通过（附 SHOULD-1）** | 载体（静态一手源码+grep+人工判读，Degraded）、覆盖（Rust 接管域；明确排除 Java 调度层 ticket 传播且说明其不属接管域）、可比性（同 260908-10 问题定义+新增一手证据可升级）、重开判据——三要素齐备。范围决策非对齐结论，无数值口径要求，声明充分。 |
| 4 | 判定逻辑支撑 | **通过** | 独立 grep 复核：`worldgen-core/src` 全量对 ChunkStatus/dependsOn/direct_dependencies/accumulated/write_radius 仅 2 处命中且均为 `bin-diag/*.rs` 注释行（非消费）；`versions/1.21.6/rust/src`（lib.rs + jni_bridge.rs，全目录就 2 文件）对 status/neighbor/accumulated/ChunkStatus 等零命中。`worldgen_handle.rs:889` 确为 `n_neighbor` 计数变量，891-914 为 17×17（-8..=8）邻域 carver seeding 自算——功能性复刻判读成立（复刻正确性本身由既往 carver 对齐结论承担，不在本判定范围）。 |
| 5 | 历史一致性 | **通过** | `.investigations/mc-1216-port-260908-10/progress-260908-10.md:68-71` P5 挂起小节判据（依赖表属 Java 侧、Rust 无消费点、重开判据=「邻 chunk 状态不足」定位到依赖推进）与本 verdict 逐条承接且升级为已验证——符合「交接结论廉价独立验证后才可继承」纪律。知识库处置（knowledge-drafts.md:164 判为一次性范围决策不入知识库）符合记录价值门。 |
| 6 | 漏项（rust 薄壳 / 1.20.1 rust 壳消费点） | **通过** | 1.21.6 薄壳 2 文件 grep 零命中（含 jni_bridge）；`versions/1.20.1/rust` 全目录 grep ChunkStatus/depends_on/accumulated/ChunkGenerationStep 零命中——两壳均无依赖表消费点，verdict 判定外延成立。 |
| 7 | 计划一致性 | **附条件** | 见 SHOULD-1/2。 |
| 8 | retry cap / 模块边界 | **通过** | 静态核对一轮闭合，无多轮假设验证；无跨模块 skill 正文引用。 |

## 结论：PASS-with-conditions

建议：**candidate 可授予**（SHOULD 级 judge 通过，confirmed 留人类）。无 MUST 级问题。条件均为 SHOULD：

- **SHOULD-1**：计划（`架构计划-260908-13-p5-chunkstatus.md` 拆解 1）承诺 Java 侧定位笔记落 `.investigations/mc-1216-port-260908-13/`，该目录实际不存在（内容直接进了 verdict）。主会话应二选一：补建过程笔记目录（Java 侧定位过程），或在计划/verdict 里加一行说明「笔记并入 verdict 正文」——消除计划-产物偏差。
- **SHOULD-2**：verdict 表格「`worldgen-core/src/*.rs` 全量 grep 零依赖表消费点」宜精确化：`bin-diag/b1_surfaceonly_dump.rs` 与 `b1_surface_dump.rs` 各有 1 处 ChunkStatus 字样命中（纯注释、非消费）——建议改为「零功能性消费点（仅 bin-diag 注释 2 处）」，防后续复核者 grep 命中后误判 verdict 不实。
- **SHOULD-3**（记录性，不阻塞）：重开判据依赖「定位到依赖推进」这一 Java 调度层归因——未来若真出现该类失败，归因本身属「机制未明」排查，届时应先 scout 勘探再归因，不直接套用本 verdict 的调度层预设。

## 约束重申

本意见不改任何 status；candidate/confirmed 授予由主会话与用户拍板。
