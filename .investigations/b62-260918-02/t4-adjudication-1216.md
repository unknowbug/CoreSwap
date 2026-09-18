# B6-2 T1 逐项定性：1.21.6 侧 21 项缺口（260918-02）

```yaml
status: draft
block: 260918-02
role: 主会话（收敛型定性，判据固定——计划 §9 预置）
criterion: B6-1 t4-adjudication.md §2 族内对称性判据 + §3.4 豁免子句（合读，#171）
carrier_discipline: 全部消费点按 1.21.6/共享载体一手重核，未套用 1.20.1 结论（#105）
verification_layer: Degraded（静态审查；补映射后发射面 MUST 经哨兵实测）
inputs:
  - 门禁实测（python scripts\check_switch_mapping.py，2026-09-18 12:43）：1.21.6 声明 106 / 本版+共享消费 124 → 缺口 21
  - versions/1.21.6/java/build.gradle :69-259（一手全量 findProperty/vmArg 名单）
  - versions/1.21.6/java/src 消费点 grep + java-core/src（共享载体）消费点 grep
```

## 0. 范围修正（对架构计划 §0 的一处机械更正）

计划 §0 提到「exec 族（exec/maxinflight）疑 SCOPED」。**实测：这两项不在 21 项缺口清单内**——
`coreswap.exec`/`coreswap.maxinflight` 的唯一消费点在 `versions/1.20.1/.../NoiseChunkGeneratorMixin.java:106/:138`（1.20.1 树独有），
1.21.6 树与共享载体**均无消费点** → 非 1.21.6 缺口，无需定性。门禁 gap 清单为准（机械证据 > 计划转述，#90 转抄漂移家族的预防性更正）。

## 1. 一手族事实（1.21.6 build.gradle，本轮实测）

| 族 | 1.21.6 映射现状（行号） | 缺口项 |
|---|---|---|
| surfaceDump | :171-177 probe/chunkX/chunkZ/size/out/outPost（5 子参数） | `surfacedump.dim` |
| blobProbe | :160-162 blobProbe(父门)+blobProbeOut | `blobProbe.chunkX/chunkZ/size/dim` |
| colProf | :168-170 probe/mode/r | `colprof.x/z` |
| bulkwb | **零映射**（1.20.1 跨载体已有 4 项，B6-1 :174-180） | `bulkwblog/wbcontent/bulkwbtest/bulkwbsentinel` |
| stallwatch | **零映射**（1.20.1 跨载体已有，:182） | `coreswap.stallwatch` |
| bench | :70-80 seed/size/originX/originZ/out 等（运行参数传递） | `bench.threads/bench.worldgen` |
| chunkRandom | :131 仅父门 chunkRandomProbe | `chunkRandom.seed/seed288` |
| colDump | :164-166 仅 colDumpOut | `colDump.targets` |
| height | :122 仅父门 heightProbe | `height.x/z` |
| wbcheck | 零映射（1.20.1 post-B6-1 亦零映射） | `coreswap.wbcheck` |
| JVM 标准属性 | — | `java.io.tmpdir` |

共享载体消费点（计入两版消费面，`(shared)`）：`java-core\src`（BulkWb.java / CppBridge.java / StallWatch.java / CoreSwapFixHelper.java）。

## 2. 定性表（21 项）

### 2.1 DEFECT-MISSING-MAPPING（12 项 → T2 补映射）

| # | 开关 | 1.21.6 消费点（一手） | 族内先例 | 定性理由 |
|---|---|---|---|---|
| 1 | `surfacedump.dim` | SurfaceDumpProbeMixin.java:46（DIM_FILTER） | 同族 5 子参数 | 族内不对称；dim 是探针**过滤参数**非输出导向（豁免①不满足）→ DEFECT |
| 2 | `blobProbe.chunkX` | BlobProbeMixin.java:73 | blobProbeOut | 族内不对称 → DEFECT |
| 3 | `blobProbe.chunkZ` | BlobProbeMixin.java:76 | blobProbeOut | 同上 → DEFECT |
| 4 | `blobProbe.size` | BlobProbeMixin.java:77 | blobProbeOut | 同上 → DEFECT |
| 5 | `blobProbe.dim` | BlobProbeMixin.java:66 | blobProbeOut | 同上（过滤参数）→ DEFECT |
| 6 | `colprof.x` | ColProfProbeMixin.java:29 | Probe/Mode/R（3 项） | 族内不对称（调 r 行、调 x 不行）→ DEFECT |
| 7 | `colprof.z` | ColProfProbeMixin.java:30 | 同上 | → DEFECT |
| 8 | `coreswap.bulkwblog` | java-core BulkWb.java:91 + CppBridge.java:628（WBLOG） | 跨载体先例（1.20.1 :177，B6-1） | 「等价性×臂变量生效自证」MUST 级判据的唯一仪器 → DEFECT（severity 最高） |
| 9 | `coreswap.wbcontent` | BulkWb.java:93（WBCONTENT 指纹门） | 跨载体先例（1.20.1 :178） | 验证面缺席（#150 家族）→ DEFECT |
| 10 | `coreswap.bulkwbtest` | BulkWb.java:95（WBTEST 首跑合成自检） | 跨载体先例（1.20.1 :179） | → DEFECT |
| 11 | `coreswap.bulkwbsentinel` | BulkWb.java:97（SENTINEL 并发冲突检测器） | 跨载体先例（1.20.1 :180） | → DEFECT |
| 12 | `coreswap.stallwatch` | java-core StallWatch.java:21 + CppBridge.java:120 | 跨载体先例（1.20.1 :182） | 沙箱禁 jstack 时唯一停滞诊断手段 → DEFECT |

> **跨载体先例的判据延伸（本表新增，须登记）**：bulkwb/stallwatch 族在 1.21.6 自己的 build.gradle 内**零先例**，
> 按字面规则应判 SCOPED；但消费点在**共享载体**（java-core，两版同一份代码），1.20.1 已为同一消费点建立 `-P` 先例
> （B6-1 确证真缺陷并修复）⇒ 同一共享消费点在另一载体缺映射 = 同源缺陷（#168 per-carrier 视角的判据面）。
> 且 1.20.1 的定性已确认这些**不是**「探针专用路径 + 缺省权威」豁免形态（bulkwblog = MUST 级判据仪器）。
> 注：`coreswap.bulkwb` 本体（BulkWb.java:89，经 `WgCompat.flag` 读、缺省走 `WgCompat.BULKWB_ON`）
> **不在缺口清单**（1.21.6 门禁消费面未计入 WgCompat.flag 形态——见 §4 未处置 1）。

### 2.2 SCOPED-DIRECT-D（合法旁路，9 项 → 不补映射）

| # | 开关 | 1.21.6 消费点（一手） | 定性理由（豁免子句①+② 合读） |
|---|---|---|---|
| 13 | `bench.threads` | JniProbe.java:83（缺省 "0"=自适应） | bench 族先例全是运行参数传递；本项=诊断性覆盖，**缺省权威** → SCOPED |
| 14 | `bench.worldgen` | JniProbe.java:26（缺省路径完整定义） | 同上形态 → SCOPED |
| 15 | `chunkRandom.seed` | ChunkRandomProbe.java（缺省 8576…） | 族内零子参数先例；缺省权威 → SCOPED |
| 16 | `chunkRandom.seed288` | 同文件 | 同上 → SCOPED |
| 17 | `colDump.targets` | SurfaceColDumpProbe.java:32-33（`targets != null` 才进定点模式） | 先例 colDumpOut 服务**输出导向**；targets 为过滤参数但**缺省权威**（默认模式完整定义）→ 豁免①② → SCOPED |
| 18 | `coreswap.wbcheck` | java-core CppBridge.java:616（自检门控，默认关） | 与 1.20.1 post-B6-1 **同判**：族内 bulkwb 4 项虽已映射（跨载体），但本项=一次性自检 + 缺省权威 + javadoc 明写 `-D` 用法 → 豁免①② → SCOPED |
| 19 | `height.x` | HeightProbe.java:19（null→默认坐标） | 族内仅父门 heightProbe，零子参数先例；缺省权威 → SCOPED |
| 20 | `height.z` | HeightProbe.java:20 | 同上 → SCOPED |
| 21 | `java.io.tmpdir` | java-core CoreSwapFixHelper.java:36/:90（有缺省回退） | JVM 标准属性，环境直传通道非 gradle `-P` 职责（#42 家族已按 env/TMP 固化处置）→ SCOPED |

### 2.3 计数核对

DEFECT 12 + SCOPED 9 = **21** ✅（与门禁 per-version 段逐名一致：12 项掩盖 + 9 项 union ORPHAN）。

## 3. 判据依赖与诚实声明

- 全部定性为**静态审查（Degraded）**；补映射后 MUST 哨兵实测发射面（Partial）——行为化自证 #37/#81。
- 判据前置集（v0.22 §15.1）：T3/T4 判据携带外部事实依赖——`key`=init script 在位（`.tmp/b61-260917-06/print_vmargs.gradle`）、`expected`=全部补映射项出现在 `t.jvmArgs`、`check`=门禁 per-version 段 1.21.6 缺口=0。
- 还原纪律：T2 改 build.gradle 前备份（git 为权威，改后 `git diff` 逐行核对）。

## 4. 未处置（诚实边界）

1. **门禁消费面抽取的 WgCompat.flag 盲区**：`WgCompat.flag("coreswap.bulkwb"/"coreswap.skipair", …)` 以**变量实参**读 sysprop，
   `RE_GETPROP`（`System.getProperty("字面量")`）抓不到 → 这两个开关**不在 M2 消费面**。本次未造成缺口误判（两版 build.gradle 均未声明它们，
   不产生假 DEAD/ORPHAN），但属**覆盖面缺口**（#169：对账工具覆盖面自身可核）——登记为门禁后续强化点，本块不改门禁（范围外）。
2. `max.bg.threads` DEAD 待核（MC 发行 jar 侧消费）——B6-1 遗留，非本块范围。
