# Judge 审查意见 — P2（est 翻默认三件套）+ P3.1（gpu-batch-merge 重议）+ session 收尾三源核对（260903-12）

- reviewer: core-judge（subagent，隔离）
- 审查对象:
  1. `.artifacts/lossless-accel/est-l2-defaultflip-p2-260903-12.md`（candidate）
  2. `.artifacts/lossless-accel/gpu-merge-revisit-260903-12.md`（candidate，decision）
- 性质: 只出意见，不改 status；confirmed 留人类。

## 〇、三源核对

1. **产物快照**: 两份 artifact 均已落盘；`index.yaml` 尾部已登记两条（est-l2-defaultflip-p2 / gpu-merge-revisit，均 status: candidate，含摘要注释）✅。
2. **原始证据**: cmd-output 三份均在且与文档声明吻合——estopt-mt-baseline（16 跑全量、交错双跑、l2 stats 逐条）、estopt-sweep（9 个 256-chunk 块 + panic 尾巴原文）、est-price-p24（hot 65/57ns、cold 5751/5721ns）✅。
3. **git 现场**: 工作区改动 = `WorldgenRust/src/worldgen_handle.rs`（+16 行，仅 WG_EST_DUMP 门控 dump，env 未设时零行为差异）+ index.yaml + 新增 bin-diag/estopt_mt_bench.rs、est_price_probe.rs（隔离区，不参与默认构建，符合「临时文件唯一区纪律」）+ artifacts/investigations 文档。**无任何生产逻辑改动** ✅。
   - ⚠️ **范围限制**: 任务清单中的 Java 侧改动（EstDumpProbeMixin/EstDumpProbe/BenchMod/build.gradle）不在本仓库 git 内；在 E:\PYTHON\MC 常见位置（tools/mc-src、mc-src2、coreswap-pkg）检索未定位到 EstDumpProbeMixin——本 judge 无法三源核对该部分，请主会话在 Java 探针工程所在仓库补核（不影响本两条 candidate 的 P2/P3 证据链，Java 探针主要服务 est-shared-verdict 的 +16 角课题）。

**「dump 门控 hash 不变复跑证据」成立**: estopt-ab-arms-260903-11.txt（HEAD 基线，git stash a3c0154 重建）与 estopt-ab-arms-p0-260903-12.txt（加门控后）off 臂 hash 同为 `74f5dfc4eede8ef4`，且 12 日 l2 臂 hash 亦等于 off（无损不变量）——门控加入前后默认路径逐位一致 ✅。

## A. P2.1「无 Mutex 争用退化」 — **PASS（附小注）**

- 数据支撑成立：L2 加速比随线程 2.55→2.76→3.12×（T8 复算 ~3.10，文档 3.12 属舍入级偏差，不影响量级）；off 臂扩展 6.48×、l2 臂 7.89×——若存在 Mutex 争用，l2 臂扩展性应劣于 off，实际反向，结论方向正确。
- ⚠️ 小注：「交错双跑偏差 <2%」表述略偏乐观——复算：T1 off 对 91.43/93.96 差 2.8%，T8 l2 对 4.49/4.62 差 2.9%，均略超 2%。建议改为「偏差 <3%，双跑均值差分」。不改变结论。

## B. P2.2 淘汰结论边界 — **CONCERN（数值错误，结论方向反被强化）**

- ✅ 如实声明到位：文档明写「sweep panic 截止、4096 全程未完成、数据截止 2304 chunk、typical region 限定」；panic 原文落盘（surface_rules.rs:505）。
- ❌ **数值错误**: 文档称「inserts ~70k，FIFO 上限的 ~54%」——按 sweep 原始输出逐块累加 inserts = 7194+4032+4310+3982+4090+4212+4122+4106+3655 = **39,703（~40k，约 30%）**；实际 ≈17 条/chunk，非注释与文档所称 ~30 条/chunk。修正后触顶投影从 ~4370 chunk 推后至 ~7600+ chunk——**「typical region 无淘汰风险」结论方向不变、反而更强**，但量化声明不可溯源（违反「每量化声明可溯源」），MUST 修正后才能升 confirmed。
- 命中率 92±1%（实际 91.5-92.7）、每块 wall 9.0-9.6s、evictions=0 全部与原始输出吻合 ✅。

## C. P2.4 归因链 — **PASS（收敛判定标注已到位）**

- 形态探针数据吻合：hot 65/57ns（文档 ~60ns）、cold 5751/5721ns（5.7µs）✅。「形态差 ~95×」复算为 88-100×，取中可接受（带 ~ 号，建议写 88-100×）。
- 跨 session 单价核算复算通过：260903-11 48ms/(7342−1715)=8.53µs；本 session 55.6ms/5627=9.88µs（55.6 = T1 off−l2 吞吐差 91.43−35.82，iters 差 5627 与 hits+misses=57499 自洽）✅。冷形态 5.7µs 与 8.5-9.9µs 同量级，剩余 ~1.5× 已明确声明为 Partial（生产侧 aquifer/缓存压力）✅。
- **fan-out 免触发判定成立**: 单价跨 session 稳定证明 est 扫描成本主导且无漂移，b2（次级效应候选）因此不构成与主机制并存的互斥候选——文档已显式标注「判定依据：单价稳定性核算」，即作为**收敛判定依据**使用而非独立证实，符合 core.fanout 触发条件（≥2 互斥机制候选才强制）。无需补 fan-out。

## D. P3.1 gpu-batch-merge 决策建议 — **CONCERN（可比性应显式标注；权限无越界）**

- **无越权**: 文档通篇为「建议」「待用户拍板」，未宣布降级决定，status 亦为 candidate ✅。
- ⚠️ **Java 33ms 引用的口径可比性**: Java ~32-33ms 来自 WorldGenBench 260903-09 run B（fresh world，不同 harness/预热形态），Rust 27.69 来自 pc_e2e_bench（region 200,200 256 chunks median）——跨 bench 绝对值对比，§9.7 三要素只声明了「同 region/size」，**未声明跨 harness 可比性折扣**。方向上稳健（保守口径下 estopt_mt_bench T=1 l2=35.8 亦与 Java 33 同量级，且与 260903-10 大样本结论同向；260903-08 confirmed 条目独立记录 Java FULL 33），故「目标差距已消除」的判断成立；但「单线程 27.69 < Java 33（快 ~1.2×）」「8 线程 ~7× Java」两句应标注「跨 bench 近似比较」后引用。estopt_mt_bench 的 4.5ms 与 pc_e2e 的管线口径不同（文档已部分声明 T=1 35.8 同向）。
- 依据链核对：est L2 于 260903-11 confirmed（est-opt-result-260903-11.md 在库）✅。

## E. §9.7 / 溯源性 / 生产零影响 — **PASS（B 项数值修正后）**

- est-l2 文档三要素齐备（载体/覆盖面/历史口径，且显式声明跨 bench 不可比）✅；gpu-merge 文档三要素只写了载体口径一行，建议补「历史口径可比性」要素（对应 D 项 CONCERN）。
- 溯源性：除 B 项「~70k」外，P2.1/P2.3/P2.4 全部量化声明可溯源到三份 cmd-output 原始文件 ✅。
- 生产零影响：diff 仅默认关的 env 门控（env::var 每 chunk 一次、chunk 级判断符合「诊断门控 chunk 级一次」铁律；可选优化 = OnceLock 缓存 env 查询，INFO 级非必须）+ bin-diag 新文件；hash 复跑证据闭环（见〇）✅。

## 总体建议

- **无 BLOCK**。两份产物维持 **candidate** 合理；**est-l2-defaultflip-p2 建议修正 B 项数值（inserts ~40k、~17 条/chunk、触顶投影 ~7600+ chunk）与 A 项「<2%」表述（改 <3%）后可推荐 candidate→（人类拍板）confirmed**；gpu-merge-revisit 建议补跨 harness 可比性标注后同路径处理。
- 翻默认动作本身仍受 est-shared-verdict（+16 角参数修正）联动条件约束——两文档对此的联动表述一致且正确，翻默认时点留给用户。
- Java 侧探针文件未能在可达范围内定位/核对，属本 judge 范围限制，非产物缺陷；请主会话补核。
