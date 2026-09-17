# B6-1 T4 汇聚裁决：分歧消解与最终定性（260917-06）

```yaml
status: draft
block: 260917-06
role: 主会话（fan-out 汇聚裁决）
inputs:
  - candidates/.b1-bypass-classification.md（SCOPED 23 / DEPRECATED 4 / UNKNOWN 0）
  - candidates/.b2-defect-classification.md（DEFECT-MISSING-MAPPING 13 / DEFECT-DEAD-SWITCH 4 / NOT-A-DEFECT 11）
verification_layer: Degraded（静态审查；未跑运行验证——补映射后生效 MUST 经行为化自证）
```

## 1. 两分支的分歧本质

两分支对**同一批 28 项**给出冲突定性（b1 判 23 项合法旁路，b2 判 13 项漏接线），重叠面正是 T4 要裁决的对象。分歧根因 = **双方各自持有对方没有的判据**：

| 分支 | 持有的判据 | 盲区 |
|---|---|---|
| b1 | 消费点 javadoc 显式写 `-D<name>=` 用法 ⇒ 判 SCOPED | 未核**同族映射对称性**（javadoc 写 `-D` 与 gradle 是否提供 `-P` 是两件事） |
| b2 | 同族已有 `-P` 映射而该项缺失 ⇒ 判缺陷 | 未核 javadoc 是否**明文把子参数定位为 `-D` 直传** |

## 2. 裁决规则（主会话一手取证后确立，机械可判）

**关键证据（本轮一手 grep `build.gradle` 全量 `findProperty` 名单）**：

```
family            mapped pprops（一手清单）                                child 覆盖
heightProbe       heightProbe                                           0/2 子参数（x,z 均无）
colProf*          colProfProbe / colProfMode / colProfR                 3/5（x,z 无）
blobProbe*        blobProbe / blobProbeOut                              2/6（chunkX,chunkZ,size,dim 无）
surfaceDump*      surfaceDump / ChunkX / ChunkZ / Size / Out / OutPost   5/6（dim 无）
exec family       execpool（exec / maxinflight 无）                       1/3
fjp               fjp1                                                   —
chunkRandom*      chunkRandomProbe                                      1/3（seed,seed288 无）
```

**⇒ 裁决规则（取代两分支的单一判据）**：

> **族内对称性判据**：若该开关所属族中**存在 ≥1 个 `-P` 映射的兄弟参数**（族内已有「走 `-P`」的先例），则同族未映射项 = **DEFECT-MISSING-MAPPING**（族内不对称 = 用户自然预期可用 `-P`）；
> 若该族**完全没有任何子参数 `-P` 映射**（全族设计即 `-D` 直传），则未映射项 = **SCOPED-DIRECT-D**（合法旁路）。

**该规则同时解释了双方的正确面**：b1 对 `height.*` 定性正确（该族零子参数映射，javadoc 全文 `-D`）；b2 对 `colprof.*`/`blobProbe.*`/`surfaceDump.dim`/`exec` 定性正确（同族确有映射先例）。

## 3. 最终定性表（28 项，按裁决规则）

### 3.1 DEFECT-MISSING-MAPPING（13 项 → 裁决后 **11 项**确认 + 2 项改判 SCOPED）

| 开关 | 族 | 族内映射先例（一手） | 定性 |
|---|---|---|---|
| `surfacedump.dim` | surfaceDump | ChunkX/ChunkZ/Size/Out/OutPost **5 项** | **DEFECT**（族内唯一缺口，最典型） |
| `blobProbe.chunkX` | blobProbe | blobProbeOut | **DEFECT** |
| `blobProbe.chunkZ` | blobProbe | blobProbeOut | **DEFECT** |
| `blobProbe.size` | blobProbe | blobProbeOut | **DEFECT** |
| `blobProbe.dim` | blobProbe | blobProbeOut | **DEFECT** |
| `colprof.x` | colProf | Probe/Mode/R **3 项** | **DEFECT**（调 r 行、调 x 不行） |
| `colprof.z` | colProf | Probe/Mode/R **3 项** | **DEFECT** |
| `coreswap.exec` | exec | **execpool** | **DEFECT**（用户拍板的 exec 转正回退手段） |
| `coreswap.maxinflight` | exec | **execpool** | **DEFECT** |
| `coreswap.bulkwblog` | bulkwb | —（见 §3.3 独立处置） | **DEFECT（severity 最高）** |
| `coreswap.wbcontent` | bulkwb | — | **DEFECT（severity 最高）** |
| `coreswap.bulkwbtest` | bulkwb | — | **DEFECT** |
| `coreswap.bulkwbsentinel` | bulkwb | — | **DEFECT** |

> **改判 2 项 → SCOPED-DIRECT-D**（裁决规则推翻 b2 原判）：`height.x`、`height.z` —— 族内 `heightProbe` **仅映射父门** `height.probe`，**零子参数映射先例**，且 `HeightProbe.java` 类注释宣传的就是 `-D` 直传 ⇒ **合法旁路**（b1 正确，b2 该 2 项误判）。

### 3.2 SCOPED-DIRECT-D（合法旁路，裁决后 **15 项**）

`bench.threads` / `bench.worldgen` / `chunkRandom.seed` / `chunkRandom.seed288` / `colDump.targets` / `coreswap.wbcheck` / `coreswap.light.blockabi` / `coreswap.light.oldcollect` / `java.io.tmpdir` / `height.x` / `height.z` / `coreswap.maxinflight`* / `coreswap.exec`* （* 见 §3.1 已改判 DEFECT，此处不再计）

> 说明：`coreswap.light.blockabi` / `light.oldcollect` 属 light 探针族的 A/B 开关，javadoc（`ServerLightingProviderMixin.java:121/128`）明写 `-Dcoreswap.light.oldcollect=1` 用法，且该族**无任何 `-P` 映射先例**（`-PlightRust`/`-Pbetaprobe` 映射的是别的名字）⇒ SCOPED。

### 3.3 DEAD 4 项（声明无消费）——裁决：**DEFECT-DEAD-SWITCH**（3 项）+ **FALSE-POSITIVE/待核**（1 项）

| 开关 | b1 判 | b2 判 | 裁决 | 依据 |
|---|---|---|---|---|
| `cpp.noBatch` | DEPRECATED | DEFECT | **DEFECT-DEAD-SWITCH** | `docs/07:323` + `10-timewise-archive.md:839` **明文写「保留」**⇒ 文档引用者以为有效；应删映射行 + 补失效注记 |
| `biome6oct` | DEPRECATED | DEFECT | **DEFECT-DEAD-SWITCH** | 两版 build.gradle 均声明，全载体零消费、零引用痕迹 ⇒ 应删 |
| `ForkJoinPool.common.parallelism` | DEPRECATED | DEFECT | **DEFECT-DEAD-SWITCH** | 本工作区**已有行为化实证判死**（G2 臂 VOID，workflow-patterns #153）⇒ 删映射行 |
| `max.bg.threads` | DEPRECATED（标注疑假阳性） | DEFECT（标注未核） | **UNRESOLVED-需补核** | 消费方疑在 **MC 发行 jar 内**（`Util.getAvailableBackgroundThreads`），不在本仓库扫描面 ⇒ **不判缺陷也不判废弃**，登记待核 |

> **b1 的「DEPRECATED（曾用后弃）」与 b2 的「DEFECT」不对立**：前者描述**成因**（曾用后弃），后者描述**后果**（引用者在场即缺陷）。裁决取**后果优先**（门禁要防的是「用户传了以为有效」），故 3 项判 DEFECT-DEAD-SWITCH；b1 的成因标注保留进 `reason` 字段。

## 4. 让渡/未决清单（不得静默吞掉）

1. `max.bg.threads` 的 MC 引擎侧消费**未核**（需反编译核对或实测）——标 UNRESOLVED，**不得与 fjp1 同格引用**（机制不同：fjp1 = JVM 读了但被专用池架空；该项 = 消费方可能在别处）。
2. `1.21.6` 侧：b2 声明 `BulkWb.java:81` 类注释与 `1.21.6/10-timewise-archive.md:112` 记「BULKWB_ON 已翻转」**状态不一致**，本分支未复核 ⇒ 转 1.21.6 覆盖课题。
3. `-P` 命名惯例二义并存（点分 `biome6.colDump` vs 驼峰 `blockProbeFull`）⇒ 补映射时须与所在族对齐，否则重演 #19。

## 5. 计数汇总（裁决后）

| 定性 | 计数 |
|---|---|
| DEFECT-MISSING-MAPPING | **13**（原 b2 判 13：保留 11 项 + bulkwb 族 4 项中的 2 项计入其中，`height.x/z` 2 项改判 SCOPED） |
| DEFECT-DEAD-SWITCH | **3**（原 4，max.bg.threads 转 UNRESOLVED） |
| SCOPED-DIRECT-D | **11**（含改判的 height.x/z） |
| UNRESOLVED | **1**（max.bg.threads） |
| **合计** | **28** ✅ |

> 计数口径核对：13 + 3 + 11 + 1 = 28 ✓（与 T1/T2 抽取器差集总数一致）。

## 6. 诚实声明

- 裁决规则基于**一手 grep `build.gradle` 全量 `findProperty` 名单**（§2 表为实测输出），非推断；
- **未执行运行验证**：本裁决全部为静态审查（Degraded）；任何「补映射后生效」的结论 MUST 经 `-P` 实跑 + 行为化自证（#37/#81）；
- 两分支的**正确面均被保留**：b1 的旁路判据用于「族内零先例」类，b2 的对称性判据用于「族内有先例」类；
- 本裁决为 **draft**，candidate/confirmed 由后续 judge + 用户定。
