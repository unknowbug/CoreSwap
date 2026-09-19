# Scope B/C 体量重估（共享 Java 适配核 B/C 波次，HOOK-2 前置）

- status: draft
- 日期标签: 260919-06（承 backlog-map §2；本块为 core.worker 分析，只读核对 + 本产物落盘）
- 角色: core.worker（分析解读；产物本文件，diff 原始证据 `.tmp/scopebc/diffs.txt`）
- 任务: 重估「范围 B（探针 16 文件入共享核）」「范围 C（NoiseChunkGeneratorMixin 主体抽 FillDispatch）」在当前代码现状下的真实体量。原定义：`.investigations/000-架构设计/架构计划-260912-01-共享Java适配核.md:82-84`。
- 方法: git diff --no-index 逐文件（versions/1.20.1/java/src/main/java/wg × versions/1.21.6/java/src/main/java/wg，含 bench/mixin），逐 diff 人工分类；java-core/build.gradle 现状直读。
- 验证分层: Degraded（纯静态文件核对，无编译/运行验证）；本结论不改变任何既有 status。

---

## 1. 文件清点（现状实测）

| 类别 | 1.20.1 | 1.21.6 | 说明 |
|---|---|---|---|
| wg/ 全部 .java | 49 | 47 | |
| 两树同名文件 | 43（25 逐字节相同 + 18 有差异） | | |
| 仅 1.20.1 | FormProbe / LightDomainBatch / LightPalDump / mixin×3（ServerChunkManagerAccessor / ServerWorldFormProbeMixin / ThreadedAnvilChunkStorageAccessor） | | 1.20.1 超前功能（形态审计 CP-1 线 / 光照域批线） |
| 仅 1.21.6 | | BlobProbeStats / mixin×2（ChunkGeneratorFeaturesMixin / NoiseChunkGeneratorTimingMixin） | 1.21.6 独有（stats holder / features takeover / 分项计时） |
| java-core（范围 A 已实施） | 8 文件：CppWorldgen + bench 7（CppBridge/BulkWb/StallWatch/ChunkTiming/CoreSwapFixHelper/WgDiag/BenchMod） | 同一份 | WgCompat shim **不在** java-core，仍分版（与原计划 D-2「shim 分版」一致） |

**探针文件现状（范围 B 的对象）**：两树同名非 mixin bench 探针/bench 类 = **19 个**（原计划说 16，7 个月间新增 LightDataDump 等，计数已变）。另有 WgCompat（shim，按设计留分版，不计）。

## 2. 探针 19 文件 diff 分类表

| # | 文件 | 行数 20.1/21.6 | 分类 | 差异点 |
|---|---|---|---|---|
| 1 | AquiferDumpProbe | = | ① 相同 | — |
| 2 | ChunkRandomProbe | = | ① | — |
| 3 | EstDumpProbe | = | ① | — |
| 4 | HeightProbe | = | ① | — |
| 5 | JniProbe | = | ① | — |
| 6 | OreProbe | = | ① | — |
| 7 | ReadWorldProbe | = | ① | — |
| 8 | WorldGenBench | = | ① | — |
| 9 | Biome6Probe | 313/313 | ② | Identifier.of ×2 |
| 10 | DiagNetherProbe | 53/53 | ② | Identifier.of ×1 |
| 11 | NoiseProbe | 71/71 | ② | Identifier.of ×1 |
| 12 | NoiseParamProbe | 85/85 | ② | getOrThrow ×1 |
| 13 | RouterProbe | 393/393 | ② | Identifier.of ×3 |
| 14 | SurfaceColDumpProbe | 132/132 | ② | Identifier.of ×1 |
| 15 | BiomeParamProbe | 78/78 | ② | getOrThrow + Identifier.of |
| 16 | BlockProbe | 969/969 | ② | Identifier.of ×1 + getOrThrow ×1 |
| 17 | DensityProbe | 623/621 | ②⁺ | Identifier.of ×7 + getOrThrow ×2；**另 1 处 API 变体**：`getEntry(RegistryKey)` → `getEntry(Identifier)`（1.21.x Registry 移除旧重载） |
| 18 | LightDataDump | 68/68 | ②⁺ | `getOpacity(View,BlockPos)` → `getOpacity()` 签名去参 |
| 19 | BlobProbe | 52/45 | ③ 实质 | 1.21.6 计数器移入普通类 BlobProbeStats（260918-03 transformer 自变换修复）；1.20.1 仍走 Class.forName 反射（= 已登记 open②，恒 -1 已知坏） |

**汇总**：① 8 / ② 纯改名 8 + ②⁺ API 变体 2（合计 ② 类 10/19 ≈ 53%）/ ③ 实质差异 1（BlobProbe，且是已知缺陷修复不对称）。

## 3. mixin 侧现状（范围 C 对象 + 周边）

| 文件 | 行数 20.1/21.6 | 性质 |
|---|---|---|
| **NoiseChunkGeneratorMixin** | **463 / 253** | **实质分叉**：① 注入签名 5 参（含 Executor）vs 4 参；② 1.20.1 侧 260910-06 后长出 ~200 行执行体机器（`wgDispatch` 三级分派 SYNCFILL→EXEC_MODE 自有有界池→P1 信号量 + FormProbe wrapper），1.21.6 仍是裸 `supplyAsync(WG_FILL_POOL)`；③ 1.20.1 多一个 FormProbe RETURN 注入方法；④ ChunkTiming.addBeard 计时仅 1.21.6 有。**原计划设想「主体抽到 FillDispatch、mixin 只留签名」——现状分派层本身已分叉（exec/P1 是 1.20.1-only）** |
| ServerLightingProviderMixin | 903 / 241 | 最重分叉（非 B/C 范围但同树）：1.20.1 = 域批 CP-1 + packed + P-β 探针全家桶；1.21.6 = 旧 per-chunk 形态 + 1.21.2 管线重构（releaseLightTicket 移除、TACS→ServerChunkLoadingManager） |
| NoiseDumpProbeMixin | 105/106 | ② populateNoise 签名 5→4 参 |
| BlobProbeMixin | 99/99 | ③ 计数器外移 BlobProbeStats + 死字段清理（1.21.6 超前） |
| ConfiguredFeatureProbeMixin | 79/79 | ② getOrThrow |
| ChunkSectionAccessor | 56/61 | 仅头注释差异（代码逐字同） |
| 其余 mixin | 19 个 | ① 逐字节相同 |

## 4. 共享核机制现状与 B/C 所需增量

- **机制（已落地，直读核对）**：两版 `build.gradle` 各一行 `srcDir '../../../java-core/src/main/java'`（1.20.1:31 / 1.21.6:33）+ 铁律「一个类只有一个家」。java-core 源在两版映射下各自编译（M1 形态）。WgCompat = 分版 shim（常量 `BULKWB_ON`/`SKIPAIR_ON` + 版本缝方法 `writeStorageLongs`），是唯一的分版 API 缝载体。
- **范围 B 需要的增量机制**：
  1. WgCompat 扩缝：`Identifier.of` / `getOrThrow` / `getEntry(Identifier)` / `getOpacity()` 等 ≈ 4-5 个缝方法（原计划 D-3-3 已预见，缝面与当时估计基本一致，略多 2 项）。
  2. ~10 个探针文件逐个把版本 API 调用改走 shim（机械改写）+ 每文件双映射编译核对（原计划成本项，仍成立）。
  3. **BlobProbe 是例外**：直接共享会把 260918-03 的 stats 修复不对称问题带进共享核——要么先修 1.20.1 侧反射路径（出货冻结面，需 HOOK），要么 BlobProbe 留分版（B 范围 19→18）。
  4. WorldGenBench 等含 `-P`/`-D` 开关消费点的文件入共享核后，`check_switch_mapping.py` 的 SRC_DIRS 覆盖面表须同步（B6-1 门禁 MUST 时机③「新增载体」）。
- **范围 C 需要的增量机制**：共享 FillDispatch 必须是**超集形态**——SYNCFILL / EXEC_MODE（自有有界池）/ P1 信号量 / FormProbe wrapper 全收，1.21.6 侧用 WgCompat 常量关掉 exec/P1/FormProbe（对齐 D-4(i) 解耦模式）；否则就得先把 exec/P1 移植到 1.21.6（= 已挂起的 B3 余额，超出 B/C 范围）。注入签名 5/4 参差由分版 mixin 壳承载（原计划设计不变）。

## 5. 结论（技术面，不做用户决策）

- **范围 B：体量与原估计基本一致，且比 7 个月前更规整**——19 个同名探针中 18 个差异仅为 API 改名/签名变体（②类 ~90% 的差异文件），无语义分叉；缝面 4-5 个方法。唯一实质障碍 = BlobProbe 的 stats 修复不对称（1.20.1 侧是已知坏路径 open②）。**技术面可行，工作量 ≈ 10 文件机械改写 + 双版编译核对，属低成本波次**。但注意：探针全部是 dev/诊断件，共享收益是「消重复」而非生产行为，1.20.1 独有的 FormProbe/LightPalDump/LightDomainBatch 3 件入不了共享核（1.21.6 无对应），B 做完仍有分版探针残留。
- **范围 C：体量显著大于原计划**——原计划假设「mixin 主体 = 单一 fill 逻辑」，现状 1.20.1 侧已长出 exec/P1/FormProbe ~200 行分派机器且 1.21.6 无对应，FillDispatch 必须按超集+开关设计，且其正确性验证面（线程模型）远重于探针改名；而 exec 模式现役 candidate（CP-1 线）仍在演进中（260919-05 C-1a），**现在抽取 = 在活跃演进的代码上做等价重构，返工风险高**。
- **风险点**：① 1.20.1 是出货冻结面——B 的机械改写若触发重发，须走「抽取后全量回归 + 三元组重算」，不得沿用 1.0.29 验证记录（原计划 D-5，仍有效）；② BlobProbe 共享化隐含要求先裁决 open② 修复（动出货树）；③ 探针入共享核后 refmap/AP 无影响（非 mixin），但两版编译均须过 V1 条目级 sha 门（原计划 §5）；④ 本重估为 Degraded 静态核对，未做双版编译实测，「每文件可编译」是基于差异性质的推断。

## 6. 证据引用

- 逐文件 diff 全文：`.tmp/scopebc/diffs.txt`（18 差异文件 unified diff，本块生成）
- 原始定义：`.investigations/000-架构设计/架构计划-260912-01-共享Java适配核.md:82-84`
- 溯源：`.investigations/legacy-sweep-260919-06/backlog-map.md` §2
- open② 现状：`.investigations/legacy-sweep-260919-06/backlog-map.md` §7（1.20.1 BlobProbe.java:39/42 仍反射路径）
