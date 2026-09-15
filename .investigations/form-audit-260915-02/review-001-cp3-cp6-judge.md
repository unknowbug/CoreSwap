# review-001 — CP-3 / CP-6 candidate 审查（260915-02，core.judge）

> 审查人：core.judge（隔离 subagent）。只出意见，不改 status；confirmed 留人类。
> 三源核对：① git HEAD + 工作区 diff ② 产物/记录快照 ③ 验证记录（cargo 声明**未独立重跑**，见 C1）。

## 总评：**PASS-with-conditions**（条件 C1-C4，均不阻塞 candidate 推荐）

---

## 一、CP-3（scratch RefCell→Mutex）

1. **diff 一致性 PASS**：`git diff` 显示 light/mod.rs 恰 13 行（9+/4-），四个改动点与声称逐一对应——line 14 `use std::sync::Mutex`、line 64-69 字段声明+注释、line 115 构造、line 293 `lock().unwrap_or_else(|e| e.into_inner())`。**无越范围改动**（全工作区 diff 仅此一文件）。
2. **改动完整性 PASS**：全仓 grep，light/mod.rs 内无残留 RefCell；LightEngine 唯一构造点 = `from_json_str`（mod.rs:112-122，from_json_file 委托 :85），jni_bridge（1.20.1/1.21.6 薄壳）与 bin-diag 全部经 handle 引用同一构造点——无未覆盖构造。worldgen-core 其余模块的 RefCell（density.rs/surface_rules.rs 等）不在 CP-3 范围（架构计划-260915-02 :10 明确不做解粘）。
3. **lock 恢复策略 PASS**：poison→`into_inner` 合理——scratch 每次调用开头 clear+resize（mod.rs:299-307），poison 残留内容无跨调用有效性，注释声明与实现一致。
4. **行为不变 PASS（单线程语义等价）**：调用图无重入（锁全程持有，bfs_propagate/export_center 均只持切片引用）；RefCell 冲突=panic、Mutex 单线程=无竞争阻塞，单线程下逐位等价。唯一理论差：同线程重入时 RefCell panic vs Mutex 死锁——本代码无重入路径，不成立。
5. **judge 历史条件核对 PASS**：
   - **C-4**（07-block-pipeline.md:1549「落地时声明防御性修复」）：mod.rs:64-67 注释明写「UB 可达性未实证，防御性修复」✅。
   - **#150 同批纪律**（workflow-patterns.md:2592 判据 2「防御性修复可先行、解粘不得先行」）：本块只做 CP-3、CP-2 解粘未动（架构计划 :10 明确不做）——**符合**。#150 判据 1 的「显式承接物」= Mutex 本身，在解粘（解除车道串行化）落地前 Mutex 即承接物，完备。

### CP-3 条件
- **C1 验证未独立重跑**：`cargo build/test 14/14 绿` 为主会话声明，judge 环境未重跑。旁证：worldgen.dll 16:29:55 / worldgen1216.dll 16:30:14（均晚于 3c5df2c），两个 dll 同长度 2468352（同一 core 源产物，结构自洽）。建议收尾三源时把 cargo 输出落 .investigations/form-audit-260915-02/cmd-output/。
- **C2「近零成本」为未测量声明**：uncontended Mutex lock 近零是通行结论，但无本仓 benchmark——建议文案保留「预期近零、未 benchmark」措辞（或随预验证 3/4 顺带计数），不阻塞。
- **C3 产物登记缺口**：.artifacts/index.yaml 尚无 260915-02 条目（CP-3 记录与 verdict 均只在 .investigations）——收尾交付前按 core.artifact 补登记。

## 二、CP-6（needsSaving 结案 verdict）

1. **一手源逐行抽查 PASS（6/6 引用点全实证，零漂移）**：
   - TACR.save 门控：ThreadedAnvilChunkStorage.java:797-802，`!chunk.needsSaving() → return false`（:799-800）+ 置 false（:802）✅。
   - ProtoChunk.setBlockState :108-158：section/heightmap/light 检查，**确无 needsSaving 置位** ✅。
   - ChunkStatus.runGenerationTask :357-363：`doWork(...).thenApply` 内 `instanceof ProtoChunk && !getStatus().isAtLeast(this) → setStatus(this)`（:361-362）✅。
   - ProtoChunk.setStatus :215-222，末行 `setNeedsSaving(true)`（:221）✅。
   - Chunk.setLightOn :389-391（:391 置位）✅；Chunk 四方法 :214/224/235/247 全为 `needsSaving = true` ✅。
   - 补充面抽核：ChunkHolder.markForLightUpdate → :190 `setNeedsSaving(true)` ✅；ChunkSerializer（实路径 net/minecraft/world/ChunkSerializer.java，verdict 写「ChunkSerializer 载入 :213」）:212-214 `shouldSave → setNeedsSaving(true)` ✅。
2. **推理链无跳步（关键问询均闭合）**：
   - **NOISE 阶段是否真经 runGenerationTask**：是——runGenerationTask 是所有生成 status 的统一包装（:357 doWork→thenApply），NOISE 的 populateNoise 经 ChunkGenerationStep.doWork 进入同一 thenApply；CoreSwap 拦截点在 NOISE 内 ⇒ 阶段完成即标脏，成立。
   - **WrapperProtoChunk 遗漏？** 不构成缺口：:361 的 `instanceof ProtoChunk` 只匹配生成期中心 chunk（chunks 列表中心恒为 ProtoChunk，:355）；WrapperProtoChunk 是世界访问侧包装，不进生成任务链。verdict 未展开此点，建议补一句（C4，可选）。
   - **反例排查**：① `isAtLeast(this)` 已达标路径不 setStatus——良性（首次达标时已置位）；② verdict 边界 5（阶段完成前 abort/unload 不标脏）如实声明且 vanilla 同构，成立；③ 真正范围外场景 = **全部 status 完成后**的非生成期 bulk 写（无后续 thenApply、lightOn 已置）——本结案范围是生成管线（候选池原文即生成期），不构成反例，但建议在 verdict 里显式圈出此范围边界（C4）。
3. **与候选池原始描述对得上 PASS**：07-block-pipeline.md:1552 CP-6 行「bulk 原地替换不经 setBlockState/标脏链 → 早 unload 存盘可能不落盘（推理级）…核对优先于立项」——verdict 逐条回应（门控在 :799、标脏靠阶段完成不靠块写），且 :11 声明「交接结论廉价验证通过」（#90 转抄核对，符合 AGENTS 交接结论验证纪律）。
4. **置信度合法**：candidate + Degraded（静态）如实声明，无越权 confirmed ✅。判据沉淀候选已按流程「交知识库 subagent 评估」而非直接写入 ✅。

### CP-6 条件
- **C4 verdict 补两句范围边界（可选，不动结论）**：① `isAtLeast` 已达标路径为良性（首达已标脏）；② 本结案限生成管线期，全部 status 完成后的非生成期 bulk 写不在结论覆盖面（该场景 vanilla 走 WorldChunk.setBlockState 自标脏，非本疑点）。③ WrapperProtoChunk 不经生成链一句带过。补后可随知识库判据一并由 subagent 草稿应用。

## 三、结论

- **CP-3：建议 candidate**（条件 C1/C2/C3 随收尾闭环；C-4 与 #150 纪律均合规）。
- **CP-6：建议 candidate**（结案「不立项」成立；C4 为可选补充）。
- 状态提升（candidate 拍板 / 后续 confirmed）由主会话与用户裁决。
