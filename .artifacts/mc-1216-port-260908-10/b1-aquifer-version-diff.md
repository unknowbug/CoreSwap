---
candidate: b1
假设: 1.21.6 的 `AquiferSampler` 语义相对 1.20.1 发生了变化，且该变化会改变地下 aquifer 单元的 air/water 方块放置，足以解释上述簇状残差。
判定: REFUTES
置信度: candidate
module: re-code
来源定位: versions/1.20.1/data/mc_src_extract/net/minecraft/world/gen/chunk/AquiferSampler.java:52-64,143-251,335-450；versions/1.21.6/data/mc_src_extract/net/minecraft/world/gen/chunk/AquiferSampler.java:52-57,136-271,355-470；versions/1.20.1|1.21.6/data/mc_src_extract/net/minecraft/world/biome/source/util/VanillaBiomeParameters.java:1206-1208（两版）；ChunkNoiseSampler.java 1.20.1:158-181,222-240 / 1.21.6:157-180,221-239；NoiseChunkGenerator.java 1.20.1:78-84,414-422 / 1.21.6:76-82,410-418；Carver.java 1.20.1:143-146 / 1.21.6:144-146；DimensionType.java 1.20.1:41-47 / 1.21.6:43-49；worldgen-core/src/aquifer.rs:66-169,455-530；.investigations/mc-1216-port-260908-08/diff/cmd-output/w1-full.diff:532-744
---

# b1 — 1.21.6 vs 1.20.1 `AquiferSampler` 逐处语义 diff（H-b1 判定：REFUTES）

> 方法：只读静态读码（无 shell）。三源交叉 = ① 我独立通读两侧源码全文；② 已有
> `w1-full.diff:532-744` 的 unified diff（复核 hunk 数=3，无遗漏）；③ `worldgen-core/src/aquifer.rs`
> 的 Rust 实现（确认 Rust 复刻的是 1.20.1 语义）。原始对照摘录见
> `.artifacts/mc-1216-port-260908-10/b1-cmd-output/aquifer-diff-excerpts.txt`。

## 判定摘要

**REFUTES**。1.21.6 `AquiferSampler` 相对 1.20.1 确实有 3 处代码级语义变化，但**其中 0 处能改变
`apply` 返回的方块状态**：全部差异落在 `needsFluidTick`（流体后处理 tick 队列）与
`FluidLevel` 的 `equals` 语义上。`apply` 的 7 个方块返回出口（4 个 `null` + 3 个 `blockState`）
在两版中**条件与返回表达式逐字相同**，且决定它们取值的 top-3 最近邻**可证恒等**。
因此「把 Rust 的 1.20.1 语义换成 1.21.6 语义」在 P7 残差列上**一个方块都不会变**。

置信度上限 = **candidate**（本判定是静态读码结论，Degraded 层，无运行时验证；`confirmed` 留人类）。

---

## 1. 语义差异清单（逐处：行号 + 两侧代码 + 是否影响落方块）

文件级 diff 只有 **3 个 hunk**（w1-full.diff:537 / 563 / 728），以下 D1-D5 即全部差异。

| # | 差异 | 1.20.1 行号 | 1.21.6 行号 | 实质 | 影响落方块？ |
|---|---|---|---|---|---|
| **D1** | `FluidLevel`：`static final class`（含显式构造器）→ `record` | A:52-64 | B:52-57 | record 生成 value-based `equals`/`hashCode`（class 版是 identity）；字段读取语义不变 | **否**（`equals` 全文件仅 4 处调用，全在 needsFluidTick：B:219,256,257,261） |
| **D2** | 最近邻追踪：top-3 → **top-4** | A:161-207 | B:154-208 | 邻居集**不变**（仍 12 格，同序遍历）；多记第 4 近的「距离+位置」 | **否**（第 4 点仅在 B:260-261 的 needsFluidTick 表达式中使用，不进任何 `calculateDensity` 调用） |
| **D3** | `d <= 0.0` 分支的 needsFluidTick | A:213 | B:217-222 | `d >= 阈值` → `若 d>=阈值 则 getWaterLevel(第2近) 比 equals 否则 false` | **否**（分支返回的 `blockState` 相同；多出的 `getWaterLevel` 是纯 cache warm-up，见 §2.3） |
| **D4** | else 分支末尾 needsFluidTick | A:245 | B:255-264 | `= true` → 用 4 个 equals + `maxDistance(o,r)` 的 4 条件式 | **否**（该分支返回 `blockState` 相同） |
| **D5** | 深暗门方法改名 | A:395 `method_43718` | B:415 `inDeepDarkParameters` | **纯改名**，方法体两版逐字相同（VanillaBiomeParameters.java:1206-1208，连行号都相同） | **否** |

**非差异项（逐字相同，w1-full.diff 无 hunk 覆盖）**：`CHUNK_POS_OFFSETS`（13 项 section 偏移）、
全部常量与 `NEEDS_FLUID_TICK_DISTANCE_THRESHOLD`、构造器与 `startX/sizeX/startY/startZ/sizeZ`、
`index()`、`calculateDensity()`（含两版一致的死局部 `px=2.0`）、`getLocalX/Y/Z`、`getWaterLevel()`、
`getFluidLevel()`、`getFluidBlockY()` 主体、`getNoiseBasedFluidLevel()`、`getFluidBlockState()`、
`seaLevel()` 匿名实现。上游输入链（`ChunkNoiseSampler` 的 `apply(pos, density)` 实参
`cacheAllInCell(add(finalDensity, Beardifier))`、`estimateSurfaceHeight`、
`NoiseChunkGenerator.createFluidLevelSampler`、`DimensionType.field_35479 = MIN_HEIGHT<<4`）
亦逐字相同。

**关键正面证据（`apply` 的 7 个方块返回出口逐字对照）**

| 出口 | A(1.20.1) | B(1.21.6) | 条件/返回 |
|---|---|---|---|
| density>0 → `null` | 149-151 | 142-144 | 相同 |
| `d<=0` → `blockState` | 212-214 | 216-224 | 条件相同（B 在 return 前插 needsFluidTick） |
| water-over-lava → `blockState` | 215-217 | 225-227 | 相同 |
| `density+e>0` → `null` | 222-224 | 232-234 | 相同 |
| `density+g>0` → `null` | 230-233 | 240-243 | 相同 |
| `density+h>0` → `null` | 239-242 | 249-252 | 相同 |
| 末尾 → `blockState` | 245-246 | 255-266 | 返回相同（B 改了 needsFluidTick 赋值） |

`needsFluidTick` 全项目消费者只有 2 处（grep 全 extract 树）：
`NoiseChunkGenerator` 1.21.6:415 / 1.20.1:419 与 `Carver` 1.21.6:144 / 1.20.1:143，
两者都只在**已 `setBlockState` 之后**判断是否 `chunk.markBlockForPostProcessing(pos)`
（= 生成后流体 tick 队列），**不写回方块状态**。

---

## 2. 机制链（REFUTES 的反证链 + 差异真正的落点）

### 2.1 落方块路径的可证等价
`apply` 的返回值是纯函数：`f(pos, density; top-3 邻居的 FluidLevel, 噪声采样)`。
- **邻居集恒等**：两版都枚举 {x-cell +0..+1} × {y-cell -1..+1} × {z-cell +0..+1} = 12 格，
  遍历嵌套顺序相同（x-cell 外层 / y-cell 中层 / z-cell 内层）⇒ `randomDeriver.split(cx,cy,cz)`
  调用序列、`blockPositions` 缓存写入序列完全一致 ⇒ 每格锚点坐标逐位相同。
- **top-3 恒等**（逐 tie 例，`>=` 语义下 4 槽插排的前 3 槽与 3 槽插排完全相同）：
  | 插入分支 | 3 槽结果 | 4 槽结果 | top-3 |
  |---|---|---|---|
  | `o>=d` | [new, old1, old2] | [new, old1, old2, old3] | 相同 |
  | `p>=d` | [old1, new, old2] | [old1, new, old2, old3] | 相同 |
  | `q>=d` | [old1, old2, new] | [old1, old2, new, old3] | 相同 |
  | `r>=d` | 不变 | [old1, old2, old3, new] | 相同 |
- 7 个出口的条件与返回表达式逐字相同（§1 表）⇒ **同输入 ⇒ 同方块输出**。

### 2.2 观测签名反向验证（决定性）
P7 三列（223,237)/(222,238)/(223,238) 的 **vanilla 侧整列全为 deepslate**
（p7-column-profiles.txt:2-62）。整列 deepslate 意味着 vanilla 的 `apply` 在这些点**全部走
`null` 出口**——要么 `density>0` 早退，要么某个 blend `null` 出口。而：
- `density>0` 早退两版逐字相同（A:149-151 / B:142-144）；
- 三个 blend `null` 出口两版逐字相同（A:222/230/239 / B:232/240/249）且只依赖 top-3（已证恒等）。

⇒ **把 1.21.6 的 aquifer 语义移植进 Rust，在这三列上不会产生任何 air/water**。H-b1 的
「足以解释簇状残差」这一步不成立。

### 2.3 D3 多出的 `getWaterLevel(第2近)` 为何不泄漏到落方块
`getWaterLevel(pos)` 的缓存键 = `(floorDiv(x,16), floorDiv(y,12), floorDiv(z,16))` 单元，
值 = `getFluidLevel(锚点精确坐标)`。锚点 = `cell*尺寸 + rand(<10/<9/<10)`，且
`10<16, 9<12, 10<16` ⇒ 锚点必落在其所属单元内 ⇒ **锚点与单元一一对应**；
`getWaterLevel` 的唯一实参来源是 `blockPositions` 里的锚点（A:209/220/226；
B:213/218/230/236/261）⇒ 该缓存是「单元 → getFluidLevel(锚点)」的纯记忆化，
**与首次调用时机无关**。故 B:218/230 的额外调用只是 cache warm-up，不改变任何取值。

### 2.4 差异真正的落点（供主会话转 b2 / 另设候选）
既然 Java 侧 aquifer 的方块语义两版等价、Rust 复刻的是 1.20.1 语义（aquifer.rs:472-530），
P7 残差只能来自 `apply` 的**输入**或**前序阶段**：
1. **`density` 实参**（首选嫌疑）：`cacheAllInCell(add(finalDensity, Beardifier)).sample(pos)`
   （ChunkNoiseSampler 1.21.6:176-180 / 1.20.1:177-181）。若该簇 Rust 侧 `density<=0`
   而 vanilla 侧 `>0`，aquifer 立刻走「air/water 出口」vs「null 出口」——与观测签名
   （Rust=air/water、vanilla=deepslate）方向完全吻合。Beardifier（结构包围盒）是
   1.20.1↔1.21.6 最容易在局部簇产生差异的输入。
2. **fluid level 输入链**：`estimateSurfaceHeight → initial_density_without_jaggedness > 0.390625`
   （ChunkNoiseSampler 1.21.6:221-239 / 1.20.1:222-240）与 barrier/floodedness/spread/lava 采样。
3. **Rust aquifer 自身偏离**（b2 域）。
4. **carver/feature 阶段的地形依赖**：Rust 臂的 Java carver 在 Rust 地形上运行，
   carver 的 `isReplaceable`/填方决策依赖输入方块 ⇒ 「carve mask 一致」不等于
   「carved 结果一致」。此项不在 b1/b2 假设内，若 1/2/3 被排除需另设候选。

### 2.5 对 port-list §B 裁决的影响
`port-list-260908-08.md:22`「AquiferSampler 邻居 4 点化 + needsFluidTick 重写（不进方块放置；
tick 队列不复刻）」——**裁决方向维持正确，但表述需修正**：邻居集**没有** 4 点化
（仍是 12 格），变的是「最近邻追踪槽位 3→4」，且第 4 点只喂 `needsFluidTick`。
建议改写为：「AquiferSampler 最近邻追踪 3→4 槽 + needsFluidTick 重写 + FluidLevel record 化
（均不进方块放置；tick 队列不复刻）」。

---

## 3. 反证 / 盲区（§9.7 覆盖面声明）

**已覆盖（读过全文/相关段）**：两侧 `AquiferSampler.java` 全文；`w1-full.diff` 的 3 个 hunk；
两版 `VanillaBiomeParameters:1206-1208`；两版 `ChunkNoiseSampler` 的 aquifer 构造、
`estimateSurfaceHeight`/`calculateSurfaceHeightEstimate`；两版 `NoiseChunkGenerator` 的
`createFluidLevelSampler`、生成主循环 needsFluidTick 消费点、carver 入口；
两版 `Carver:136-149`；两版 `DimensionType:38-53`；Rust `aquifer.rs:452-530` 与 WG_AQDUMP 钩子；
`p7-column-profiles.txt` 全文。

**未覆盖（未看/未验，判定不得外推）**：
1. **Rust density 求值链一行未读**（`density.rs`/插值/Beardifier/transpiler 产物）——§2.4 的
   「输入侧差异」是**推断**，不是验证。
2. **P7 对拍的运行口径未核**：两臂是否都执行了 `markBlockForPostProcessing` 的流体 tick
   （若一臂 tick 过、另一臂未 tick，needsFluidTick 差异会在**已 tick 的存档**里造成水流差异）。
   注：水流只会让水→空气，不能把 deepslate 变成水，故不改变本判定方向，但属口径盲区。
3. **carver 决策对地形块的依赖未审**（只看了 needsFluidTick 消费点）——§2.4 第 4 项的
   「carve mask 一致 ⇒ carved 结果一致」是主会话给的约束，本 worker 未独立验证。
4. **未编译/未运行**（沙箱禁止 shell）——本判定为 Degraded 层静态审查，无运行时证据。
5. **未复核 w1 的「density 数学零变」结论**（P3 的 SHA256 逐字节一致结论按既有记录引用，未重算）。
6. **`DensityFunction` 各节点的 1.21.6 语义未读**；`SharedConstants.isOutsideGenerationArea`
   （1.21.6 新增写入门控，NoiseChunkGenerator:411）只确认在 (200,200) 区域恒 false，未展开验证。
7. **top-3 恒等的证明依赖「两版遍历顺序相同」**——用源码字面核对，**未做运行期 dump 对拍**；
   若两侧 yarn 反编译有隐蔽的循环重排，此证明失效（但 w1-full.diff 逐行显示枚举顺序一致）。
8. **density function 内部缓存的顺序副作用未审计**——我只论证了 `waterLevels` 与
   `surfaceHeightEstimateCache` 两个缓存是纯记忆化，未审 `CacheAllInCell`/`InterpolatedDF`
   在「多一次 `getWaterLevel` 调用」下是否可能改变后续采样（理论上纯函数，但未读实现）。

**结论强度**：REFUTES 的核心论证（7 出口逐字相同 + top-3 恒等 + needsFluidTick 只进后处理队列）
不依赖任何未覆盖项；未覆盖项只影响「真因在哪」的指向（§2.4），不影响「H-b1 不是真因」的判定。

---

## 4. 一锤定音的运行时探针设计（可执行、坐标具体）

> 前置口径（§四 seed 三查）：两臂必须同为 seed `-8248318472910187742`，探针输出头行核对 seed；
> 坐标语义按「探针/参照数据采集核对铁律」第 2 条区分 BlockProbe / RouterProbe / Rust dump 三套坐标。
> 点集（共 164 点）：
> - 簇 3 列：世界 (223,237)、(222,238)、(223,238)，y = -34..6（3×41 = 123 点）
> - 对照 1 列：chunk(0,0) 世界 (8,8)，y = -34..6（41 点，应两臂全 match）

**Probe 1（决定性，最便宜，无需改代码）——差分重放**
取主会话已计划的 Java 1.21.6 vanilla `-PaqDump=true`（aquifer 全链路 dump）在 164 点上的字段：
`density`、`o/p/q`（及 1.21.6 的 `r`）、三个锚点坐标、`fl2/fl3`（1.21.6 加 `fl4`）的 `{y, state}`、
`d`、`e/f/g`、`calculateDensity` 返回值、出口分支、返回方块。把**同一组字段**分别喂进
「1.20.1 版 apply」与「1.21.6 版 apply」两个离线重放器，逐点比对返回值。
- 预测（若本判定成立）：**返回方块 164/164 相同**，仅 needsFluidTick 列可能不同。
- 若出现任一不同 → 立即翻为 SUPPORTS 并升级。

**Probe 2（定位真因，Rust 侧已内建）——`WG_AQDUMP` 同点对拍**
`WG_AQDUMP=<点文件>`（`worldgen-core/src/aquifer.rs:66-169,455-459`，行格式 `x y z`）跑 Rust 接管臂，
与 Probe 1 的 Java vanilla dump 逐点比对：
- 若 Rust `density<=0`（出口 `Rock:density>0` 不触发、走 `BLOCK:*`）而 Java `density>0`
  （出口 `Rock:density>0`）→ **真因 = density 输入**，H-b1 死亡，转 density/Beardifier 候选（b2 之外）。
- 若两侧 `density` 一致但出口不同 → 才轮到 Rust aquifer 实现（b2）。

**Probe 3（Java 侧对称探针，需 P4 探针工程加 1 个 Mixin）**
在 `AquiferSampler$Impl.apply` 上 `@Inject(at=HEAD)` 记 `(pos, density)`、
`@Inject(at=RETURN)` 记返回值（含 `needsFluidTick` 快照），输出 `java-aqdump.txt`；**两臂各跑一次**：
- vanilla 臂 = Java 1.21.6 aquifer 的输入/输出；
- Rust 臂 = Java 1.21.6 aquifer 在 Rust 地形上被 carver 调用时的输入/输出。
逐点比对可把「base fill 差异」与「carve fill 差异」分离（Rust 臂 carver 调用的 `density` 恒为 0.0，
与 base fill 的 density 不同源，这正是分离依据）。

**Probe 4（上游排除）**——`density_probe`/`router_probe`（Partial）在 164 点 dump
`final_density` 与 `finalDensity+Beardifier` 的两臂值（先做坐标语义核对）。若该簇 Rust 与 vanilla
density 符号不同 → 真因锁定为密度侧，H-b1 永久排除。

**Probe 5（终局证伪实验，最贵）**——在 Rust 臂把 `aquifer.rs` 换成 1.21.6 语义（4 槽 + 新
needsFluidTick + record equals）重跑 P7。**预测 mismatch 数不变（±0）**；若不变则 H-b1
从「静态 REFUTES」升级为「运行期 REFUTES」。成本：一次 release 构建 + 一次双臂对拍。

**判据表**

| Probe 结果 | H-b1 | 下一步 |
|---|---|---|
| P1 重放 164/164 同 | 强化 REFUTES | 走 P2/P4 定位 density |
| P1 出现不同 | SUPPORTS | 立即升级人类 + 走 P3/P5 |
| P2 density 两侧不同 | REFUTES | 转密度/Beardifier 候选 |
| P2 density 同、出口不同 | 不能排除 b2 | 交 b2 对比 |
| P5 mismatch 不变 | 运行期 REFUTES | 关闭 b1 |

> 落盘说明：本产物为 b1 候选（draft→candidate），`confirmed` 只能由人类授予；索引条目由主会话
> 用 `ref_merge_index` 合并（本 worker 只写 `b1-*` 路径，不交叉 b2）。
