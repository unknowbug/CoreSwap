# B1 — 「(-244,-256) 岛由 StructureWeightSampler(Beardifier) 抬升 density 形成」假设验证

> 角色：core.worker（分析，只读文件，无 shell）
> 任务：验证 B1 候选假设「(-244,-256) 列 NOISE 阶段岛（y=58-61 stone）由 Beardifier 结构密度修正抬升 density 形成——C++ 未实现故判 water」
> 种子：seed=-8248318472910187742；区域 origin=(-288,-256) 4×4 chunk；block_probe 匹配率 95.7376%
> 日期：本次分析
> 状态：**draft**（只写 draft；提升需 judge 意见 + 主会话裁决）
> retry：0
> 自检：本任务为纯分析（产物为 .md），不交付代码，subagent 写码强制自检清单不适用（N/A）；读文件路径全部在来源定位字段列出

---

## 0. 判定摘要（TL;DR）

| # | 结论 | 置信度 |
|---|---|---|
| 1 | **B1 假设被推翻**：(-244,-256) 不在任何参与 Beardifier 的结构影响区内，**Beardifier(-244,58,-256) = 0**，不可能抬升 density 形成岛 | 【确定】（源码距离语义） |
| 2 | Beardifier 精确空间语义 = **结构 piece bbox 外 11 格（x/z 每方向，m∈[0,11]），y 以 ground 为基准 ±12**；「24 格硬边界」是 phase6/7 对 STRUCTURE_WEIGHT_TABLE(24³) 的误读 | 【确定】（StructureWeightSampler.java L90-91/L140-152） |
| 3 | -288 区域**唯一**参与 Beardifier 的结构 = **plains 村庄**（beard_thin）；沉船/ocean_ruin/矿井/monument/ruined_portal = NONE（不参与）；地牢/紫晶洞 = FEATURE（不参与）；trail_ruins（bury）不在该区域（全量结构块无其标志块） | 【确定】（JSON + 全量块分类） |
| 4 | (-244,-256) 距村庄 z 方向 32 格 > 12 → 权重 0；phase7 的「距村庄 24 格外」结论**方向正确但口径错误**（应为 12 格） | 【确定】 |
| 5 | 岛在 **NOISE chunk status 已存在**（stone@58-61）→ 排除「结构 FEATURES 覆盖」解释（phase5 出路 B 对本列不成立） | 【确定】（noiseblk_blockprobe.txt） |
| 6 | 岛 solid 的真正机制候选 = **Java AquiferSampler 的 e/g/h 液面修正**（L221-241 `density + e > 0 → return null`）——即 phase5 出路 A「e 翻转」：C++ 的 e≡0（相邻液面同 63），Java 若 13 邻居液面输入不同则 e≠0 可翻转 -0.0744 判 solid | 【推测·高置信】（源码 + 排除法） |
| 7 | phase6 的「Beardifier(58) > +0.0744」反推**错误**（把「有 Beardifier 项」误推为「该位置非零」）；phase6 对「块生成路径 Beardifier = StructureWeightSampler」的源码引用本身正确 | 【确定】 |
| 8 | Beardifier 实现对 -288 修复收益**很小**（仅村庄 12 格内候选，粗估 <1000 块），不应作为 -288 对齐主路径；真正待修方向 = **C++ aquifer 液面链对比** | 【推测】 |

---

## 1. 源码证据：Beardifier 精确空间语义【确定】

### 1.1 StructureWeightSampler.java（`versions/1.20.1/data/mc_src_extract/net/minecraft/world/gen/StructureWeightSampler.java`）

```
L21   public class StructureWeightSampler implements DensityFunctionTypes.Beardifying
L22   public static final int INDEX_OFFSET = 12;
L23   private static final int EDGE_LENGTH = 24;
L24   private static final float[] STRUCTURE_WEIGHT_TABLE = new float[13824];  // 24×24×24
```

- L41 收集条件：`world.getStructureStarts(pos, structure -> structure.getTerrainAdaptation() != StructureTerrainAdaptation.NONE)`
  → **只有 terrainAdaptation ≠ NONE 的结构才进入 Beardifier**。
- L46-66 逐 piece 构造：`structurePiece.intersectsChunk(pos, 12)` 通过才入列。
- L90-91（piece 贡献的 x/z 距离）：
  ```
  int m = Math.max(0, Math.max(box.getMinX() - i, i - box.getMaxX()));   // 到 bbox x 方向距离（≥0）
  int n = Math.max(0, Math.max(box.getMinZ() - k, k - box.getMaxZ()));   // 到 bbox z 方向距离（≥0）
  ```
- L92-93（ground 基准）：`o = box.getMinY() + groundLevelDelta; p = j - o;`
- L95-99（y 距离口径）：`BEARD_THIN → q = p`；`BEARD_BOX → q = max(0, max(o-j, j-box.getMaxY()))`
- L101-105：`BEARD_THIN/BEARD_BOX → getStructureWeight(m, q, n, p) × 0.8`
- L140-152（**权重非零的硬边界**）：
  ```
  int i = x + 12; int j = y + 12; int k = z + 12;
  if (indexInBounds(i) && indexInBounds(j) && indexInBounds(k)) { ... }  // 需 0 ≤ x/y/z+12 < 24
  else return 0.0;
  ```
  → **m, q, n 均须 ∈ [-12, 11]**；因 m,n ≥ 0（到 bbox 的距离），**非零区 = piece bbox 每面外扩 12 格（x/z），y 方向以 ground 为基准 ±12**。
- 权重公式（L140-169）：`weight = f × e^(-(x²+(y+0.5)²+z²)/16)`，其中 `f = -d·fastInverseSqrt(e/2)/2`，`d = p+0.5`
  → ground 之下（p<0）f>0 → **正权重抬 density → 更易判 solid**；ground 之上 f<0 → 挖空。

**结论：phase6/7 的「STRUCTURE_WEIGHT_TABLE 24 格外恒 0」是正确的表象，但半径口径应为「bbox 外 12 格」而非「结构块外 24 格」。** 对判定 (-244,-256) 无实质影响（32 > 12），但影响后续实现与影响面估算。

### 1.2 参与结构清单（terrainAdaptation 来源）【确定】

| 结构 | JSON 文件 | terrain_adaptation 字段 | 参与 Beardifier |
|---|---|---|---|
| village_plains（村庄） | village_plains.json L13 | `"beard_thin"` | **✓** |
| shipwreck（沉船 A/B/C） | shipwreck.json | 无（默认 NONE） | ✗ |
| ocean_ruin_cold | ocean_ruin_cold.json | 无（默认 NONE） | ✗ |
| mineshaft（矿井） | mineshaft.json | 无（默认 NONE） | ✗ |
| ocean_monument | monument.json | 无（默认 NONE） | ✗ |
| ruined_portal_ocean | ruined_portal_ocean.json | 无（默认 NONE） | ✗ |
| trail_ruins（考古遗迹） | trail_ruins.json L13 | `"bury"` | ✓（但不在 -288 区域，见 §1.3） |
| 地牢 Dungeon / 紫晶洞 / 矿脉 | FEATURE | — | ✗（FEATURE 不参与） |

- Structure.java L75-77：`getTerrainAdaptation()` 返回 `config.terrainAdaptation`；Structure.java L207：JSON 缺省 = `NONE`。
- OceanRuinStructure.java / ShipwreckStructure.java 均未重写 → 继承默认 NONE。
- ChunkNoiseSampler.java L177-181 + L469-470：块生成路径 `add(finalDensity, Beardifier.INSTANCE)` 经 `getActualDensityFunctionImpl` 替换为 `this.beardifying` = StructureWeightSampler（真实注入点，phase6 引用正确）。

### 1.3 trail_ruins 排除【确定】

- trail_ruins 有 `terrain_adaptation: "bury"`（会参与 Beardifier），phase7/8 结构清单遗漏了它。
- 但 `m288_vanilla_cat.txt`（全量 67042 块分类）**无 suspicious_sand/suspicious_gravel/砖块**（trail_ruins 标志块）→ **-288 区域不存在 trail_ruins**。
- trail_ruins biome tag 为陆地（taiga/snowy 等），cold_ocean 不生成——与块分类一致。
- 即便存在：BURY 分支用 `getMagnitudeWeight`（L132-135，半径 ~6 格、clampedMap(magnitude,0,6,1,0)），且 start_height=-15（地下），距 y=58 更远，权重也为 0。

---

## 2. 距离计算：(-244,-256) 的 Beardifier = 0【确定】

### 2.1 对村庄（唯一参与结构）

- 村庄结构块范围（phase7 §1 实测）：x∈[-275,-233]，z∈[-224,-193]，y=62-70。
- (-244,-256)：z = -256，村庄北缘 minZ = -224（z 更小为北）。
  ```
  n = max(0, max(box.minZ - k, k - box.maxZ))
    = max(0, max(-224 - (-256), -256 - (-193)))
    = max(0, 32) = 32
  ```
- `getStructureWeight(..., n=32, ...)`：`k = n + 12 = 44 ≥ 24` → **返回 0.0**（L144-151）。
- 即使村庄 piece bbox 因 jigsaw 扩展外扩，z 方向影响下界最多到 -236（-224-12），(-244,-256) 仍在 20 格之外。
- 同理 x 方向 m = max(0, -275-(-244)) = 31 > 12，也越界。
- **Beardifier(-244,58,-256) = 0**。【确定】

### 2.2 对其他结构

| 结构 | 参与？ | 距 (-244,-256) 最近距离（z/x 主导） | Beardifier |
|---|---|---|---|
| 村庄 | ✓ | z 差 32（>12） | 0 |
| 沉船 A/B/C | ✗（NONE） | — | 0 |
| 地牢（FEATURE） | ✗ | z 差 47（>12） | 0 |
| 矿井（NONE）/ 其余 | ✗ | — | 0 |

### 2.3 与 phase6/7 距离算法裁决

- phase6 §2.3 推断「Beardifier(58) > +0.0744 把 -0.0744 顶正」——**仅由矛盾反推，未实测；实际权重 0**。【确定推翻】
- phase7 §4.3 判「(-244,-256) 距村庄切比雪夫 32 > 24 → Beardifier 应为 0」——**结论正确、口径错误**（24 是 TABLE 尺寸误读；实际影响半径 12）。【确定：结论对/口径错】
- phase8 §3.2 称「该位置在村庄/沉船 Beardifier 24 格内（y≈58 vs 村庄 y=62-70，dy≈4-12 ✓）」——**y 方向确实在 ±12 内，但 x/z 方向 n=32 越界**，故仍为 0；phase8 未做 x/z 精确计算。【确定纠正】
- beard_run.txt L302-333：[BEARD] 实测仅覆盖 (-278,-240) y=0-30（全 0），(-244,-256) 列从未实测——任务背景属实；按源码语义实测也必为 0（z 差 32 > 12）。

---

## 3. 反证：岛 solid 的真正机制方向【推测·高置信】

### 3.1 排除法（链式反证）

1. NOISE-BLK 铁证（noiseblk_blockprobe.txt L27-54）：chunk status=minecraft:noise，(-244,-256) 列 y=58-61 = **stone（raw=1）**，y=51-57 water / 62 water / 63+ air。
   → **岛在 NOISE 阶段（含水层输出）已判 solid**，早于 FEATURES 阶段 → **phase5 出路 B「ocean_ruin 结构覆盖」对本列不成立**。【确定】
2. Java 噪声阶段 density(-244,58,-256) = **-0.074427**（phase5 §3.2：cns idx0(58) = -0.233015，squeeze(0.64×idx0) 验算），C++ 同 -0.074424（≤3e-6）【确定】。
3. Beardifier(-244,58,-256) = 0（本报告 §2）【确定】。
4. 由 1+2+3：Java aquifer 输入 = -0.0744 + 0 < 0，若判定链与 C++ 完全一致应判 water；但实际判 stone → **必存在 density 输入之外的修正**。

### 3.2 唯一剩余机制：AquiferSampler 的 e/g/h 液面修正（phase5 出路 A）

`versions/1.20.1/data/mc_src_extract/net/minecraft/world/gen/chunk/AquiferSampler.java`：

```
L145  public BlockState apply(DensityFunction.NoisePos pos, double density) {
L149    if (density > 0.0) { ... return null; }            // 主判 solid
L221    double e = d * this.calculateDensity(pos, mutableDouble, fluidLevel2, fluidLevel3);
L224    if (density + e > 0.0) { ... return null; }         // ← e 修正可翻转 -0.0744 → solid
L229    double g = d * f * this.calculateDensity(pos, mutableDouble, fluidLevel2, fluidLevel4);
L232    if (density + g > 0.0) { ... return null; }
L238    double h = d * g * this.calculateDensity(pos, mutableDouble, fluidLevel3, fluidLevel4);
L241    if (density + h > 0.0) { ... return null; }
L263    private double calculateDensity(...)               // 依赖 13 邻居液面差（fl2/fl3/fl4）
```

- phase5 §1.3：C++ 的 e=0（`j=|fl2.y-fl3.y|=0`，相邻网格液面同 63 → calculateDensity 返回 0）【确定】。
- **注意（与 b2-aquifer-pocket.md 交叉核对）**：C++ aquifer.h 的 pocket/液面场（fluidFloodedness/fluidSpread/barrierNoise、getFluidBlockY/getNoiseBasedFluidLevel/calculateDensity）已**完整实现**（b2 §0 结论【确定】）。故本假设不是「C++ 缺 e 修正实现」，而是 **Java 与 C++ 的 13 邻居液面输入值不同**（`getFluidLevel` 邻居链），导致 Java j≠0、C++ j=0【推测】。
- 若 Java 的 13 邻居液面输入（`getFluidBlockY` / `estimateSurfaceHeight` / `fluidLevelSampler`）与 C++ 不同（例如某邻居液面 ≠63），则 `j≠0 → calculateDensity≠0 → e≠0`，`density+e` 可 >0 → 判 null(stone)【推测】。
- 需要的量级：e > +0.0744（y=58）；d=0.64（phase5：o=90/p=99 → d=1-|99-90|/25=0.64），故 `calculateDensity > 0.116`——对应液面差数格量级，合理【推测】。
- 佐证：beard_run.txt densFn（CellCache 缓存值）在 (-244,-256) y=48-64 全正（+0.034~+0.052），虽因反射污染不可作精确参照（phase5 §2 铁律），但其「cell 角点 y=48/56/64 恒 0.037482、cell 内 y=52/58/60 变化」的符号与「该列密度输入偏正」不矛盾——注意这与 Beardifier 无关（Beardifier=0），更支持「含水层/液面系统在该列整体偏 solid」【推测】。

### 3.3 与 phase8 含水层谜团的一致性

- phase8 核心矛盾（(-278,-240) y=15-19 finalDensity>0 却判 water、y=23 air）与 (-244,-256) 岛（finalDensity<0 却判 stone）**同为「density 符号与 aquifer 判定不一致」**，均指向 **aquifer 内部液面/含水层场**，而非 density 输入链。
- 两个样本并列后，最合理的统一机制：**Java Aquifer 的液面网格（含 pocket 边界）与 C++ 实现有差异**（C++ aquifer.h 的 getFluidLevel/13 邻居液面链）。

---

## 4. Beardifier 可解释块数估算（修正 phase7）【推测】

- phase7 §4.1 估「结构 24 格内正向 water→solid ≈2000-4000 块」，**口径有误**：
  a) 把沉船 A/B（NONE，不参与）附近的 water→stone 也列为 Beardifier 候选 —— 排除后这些是真差异（e 翻转/海底边界机制，与 (-244,-256) 同族）；
  b) 用「结构块 24 格」口径 —— 实际是 **piece bbox 外 12 格**；
  c) 参与结构仅村庄（+ 不存在的 trail_ruins）。
- 修正后的 Beardifier 候选 = **村庄（x∈[-275,-233], z∈[-224,-193], ground≈62-70）bbox 外扩 12 格、y∈[50,73] 内的 C++ water ↔ vanilla 实心块**：对应 phase7 的「村庄西缘 chunk(-18,-13)」等少数区域。
- 粗估：该影响区覆盖 x∈[-287,-221]×z∈[-236,-181]×y∈[50,73]，其中海底边界（water→stone/dirt/sand，y=52-62）仅占少数（多数海底边界在 z∈[-256,-240] 或 x≤-276 之外，如 chunk(-16,-16)/(-15,-16) 远离村庄）。**估算 <1000 块，可能仅数百块**【推测·低置信，需逐块统计确定】。
- 任务口径的「海底边界 ≈6710 块」中，**Beardifier 可解释的占比很小**（<15%）；主体（含 (-244,-256) 岛同族）应归 aquifer 液面/e 翻转机制。

---

## 5. 对「C++ 是否范围内待修」的建议（交主会话裁决）

1. **Beardifier（StructureWeightSampler）**：是结构系统组件（依赖 structure starts/pieces/junctions 数据，C++ 目前无结构生成）。**对 -288 对齐收益很小（<1000 块），且对 (-244,-256) 岛零收益**。建议：**不作为 -288 未闭合 23% 的修复路径**；若未来实现结构系统，按 StructureWeightSampler.java L21-174 复刻（注意影响半径为 bbox 外 12 格，非 24）。
2. **(-244,-256) 岛 + 海底边界 6710 块主体**：真正待修方向 = **Java vs C++ 的 aquifer 13 邻居液面输入值逐项对比**（b2-aquifer-pocket.md 已证 C++ aquifer.h pocket/液面场实现完整【确定】，故焦点不是「实现缺失」而是「输入链逐值差异」）。一次决定性实验（phase5 §5.1）：Java 侧 dump (-244,55..62,-256) 的 aquifer apply 中间量（o/p/q/d/fl2.y/fl3.y/e + density+e），与 C++ trace 逐项 diff：
   - 若 o/p/q/d 一致但 fl.y 不同 → C++/Java 液面输入链差异（getFluidLevel / estimateSurfaceHeight 邻居链，C++ 侧需对照 13 邻居液面值）；
   - 若 o/p/q 不同 → splitter/md5 派生链验证（phase5 §5.1 第 3 分支）；
   - 若全部一致且 e=0 → 需要重新审视 NOISE-BLK 探针口径（本报告已排除结构覆盖与 Beardifier，届时只剩探针疑点）。
   - 另注：b2 判定「4416 深层含水层 = carvers 产物（已闭合）」针对 chunk status=carvers 样本；(-244,-256) 的 NOISE-BLK 是 **status=noise**（noiseblk_blockprobe.txt L27），岛是含水层直接输出，不适用 b2 的 carvers 解释——两个样本机制可能不同，需分别验证。
3. **phase8 含水层（stone→water 4416 + deepslate→water 635）**：与岛同为 aquifer 内部机制样本，建议并入同一「Aquifer 液面/含水层场」课题。

---

## 6. 置信度与边界说明

- 【确定】：StructureWeightSampler 空间语义（源码 L90-91/L140-152）；-288 区域参与结构清单（JSON 字段 + 全量块分类 m288_vanilla_cat.txt）；(-244,-256) 距村庄 n=32>12 → 权重 0（坐标计算）；NOISE-BLK 岛在 NOISE 阶段已存在（noiseblk_blockprobe.txt L27-54）；phase6/7/8 口径裁定（§2.3）。
- 【推测·高置信】：岛 solid = AquiferSampler e/g/h 液面修正（排除法 + 源码 L221-241）；e 翻转的量级合理性。
- 【推测·低置信】：Beardifier 可解释块数 <1000（未做逐块统计，需村庄 piece bbox 精确数据 + 逐块距离统计）。
- 本报告只做解读，不修改代码；未运行任何命令（沙箱无 shell 铁律遵守）；所有引用文件路径见 §7。
- 遗留：Java 侧 (-244,55..62,-256) aquifer 中间量 dump 未执行（需主会话/后续 phase 在 Java 探针侧补打）；村庄 piece bbox（structure start 数据）未实测，Beardifier 候选块数仍为量级估算。

---

## 7. 来源定位

| 引用 | 路径 |
|---|---|
| StructureWeightSampler | `versions/1.20.1/data/mc_src_extract/net/minecraft/world/gen/StructureWeightSampler.java` |
| StructureTerrainAdaptation | `versions/1.20.1/data/mc_src_extract/net/minecraft/world/gen/StructureTerrainAdaptation.java` |
| Structure 默认 adaptation | `versions/1.20.1/data/mc_src_extract/net/minecraft/world/gen/structure/Structure.java` L75-77/L207 |
| OceanRuin/Shipwreck（未重写） | `.../structure/OceanRuinStructure.java` / `.../structure/ShipwreckStructure.java` |
| Beardifier 占位/注入 | `.../world/gen/densityfunction/DensityFunctionTypes.java` L290-315；`.../world/gen/chunk/ChunkNoiseSampler.java` L177-181/L469-470 |
| Aquifer e/g/h 修正 | `.../world/gen/chunk/AquiferSampler.java` L145/L221-241/L263/L387-391 |
| 结构 JSON | `versions/1.20.1/data/worldgen/data/minecraft/worldgen/structure/{village_plains,shipwreck,ocean_ruin_cold,mineshaft,monument,ruined_portal_ocean,trail_ruins}.json` |
| NOISE-BLK 铁证 | `.investigations/-288-reopen/cmd-output/noiseblk_blockprobe.txt` L27-54 |
| [BEARD] 实测 | `.investigations/-288-reopen/cmd-output/beard_run.txt` L302-333（仅 (-278,-240) y=0-30，全 0） |
| densFn/est/cns-ini | `.investigations/-288-reopen/cmd-output/beard_run.txt`、`noiseblk_blockprobe.txt`、`aqfj_blockprobe.txt` |
| 参照列 | `.investigations/-288-reopen/ref_col_-244_-256.txt`（y=59-61 dirt 顶面） |
| 结构块全量分类（无 trail_ruins） | `.investigations/-288-reopen/m288_vanilla_cat.txt` |
| pair/chunk 分布 | `.investigations/-288-reopen/m288_pair_counts.txt`、`m288_chunk_counts.txt` |
| phase5/6/7/8 前序 | `.investigations/-288-reopen/analysis-phase5.md`、`analysis-phase6.md`、`analysis-phase7.md`、`analysis-phase8.md` |
| 课题上下文 | `.investigations/-288-unclosed/pipeline-map.md` |
