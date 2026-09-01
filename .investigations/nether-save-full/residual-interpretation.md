# nether 存档级 Full 化 — 双 seed 残差机制解读

> status: **draft**（Partial/Degraded：纯静态 + 数据解读，未做任何新探针运行，无 confirmed 结论）
> worker 解读，2026-09-04。数据源：
> - `.tmp/compare_nether_seedA_rust.txt` / `.tmp/compare_nether_seedB_rust.txt`（MCA 直解，per-chunk + top mismatch 分类）
> - `.tmp/rust_nether_seedA_v2.log` / `.tmp/rust_nether_seedB_v2.log`（ReadWorldProbe mismatch 前 15 行 + layerMatch）
> - `.investigations/nether-save-full/cmd-output/seedA-contradiction-facts.md`（历史，其"Rust gen"前提已推翻，仅参考现象记录）
> 实验口径：真 Rust run，dll=C5AC5309…（1.0.22 M17），worldgen_dir=versions/1.20.1/data/worldgen，区域 4×4 @(3200,3208) nether。口径三要素（v0.20 §9.7）：载体 = MCA 存档直解 vs vanilla 参照；覆盖面 = 4×4 chunk 全高度（nether 256）；可比性 = 与 docs/09 的 ReadWorldProbe 内存级口径不同载体（存档 vs 内存），数值不可直接互比。

## 0. 总量

| seed | 三口径 | 残差块数 | 残差分布 |
|---|---|---|---|
| A = -2032795982907864146 | 内存 = 存档读回 99.9376%（精确同值）；MCA 99.9278%（+103 cave_air） | 757（MCA）/ 655（内存） | 均匀散布（16 chunk 均 99.79–99.98%），无大宗簇 |
| B = 8576294172403134396 | 三口径精确同值 93.5156% | 67,994 | 集中在 y≤127（layerMatch：y0..31=82% / 32..63=88% / 64..95=88% / 96..127=90%；y≥128=100%）|

**结构性判读（来自 layerMatch + vanilla buildSurface 被 skip 的管线事实）**：seed B 残差全部落在噪声高度（≤127）内 = vanilla 侧 buildSurface surface rule 输出 vs Rust 生成输出的差异（本次 run buildSurface 被 Mixin skip，存档里的表层全来自 Rust 生成）；seed A 残差无层级聚集，是散点级 feature/尾随差异。

## 1. 残差机制分类与占比

### seed A（757 块，MCA 口径）

| 机制类别 | 组成 | 块数 | 占比 |
|---|---|---|---|
| **A1. nether 矿石 feature 差异** | quartz x492 + gold x124 + ancient_debris x21 + debris→quartz x1 + quartz→gold x1 + blackstone→debris x1 | **640** | **84.5%** |
| **A2. air↔cave_air 尾随阶段** | air→cave_air x103（chunk(203,200) y70-72 一簇）+ cave_air→air x1 | **104** | **13.7%** |
| **A3. magma block 点差** | blackstone→magma x4 + netherrack→magma x4 | 8 | 1.1% |
| **A4. 熔岩湖边界点差** | lava→netherrack x3 + lava→blackstone x2 | 5 | 0.7% |

- A1 方向全是 `vanilla 有矿 → 存档无矿`（netherrack/黑石 占位）→ Rust nether 侧 ore feature（quartz/gold/debris）未放置或放置错位，与 seed B 同家族。
- A2 = 已知候选机制（vanilla 尾随阶段 carver/feature 在 probe 读取后写存档）在存档级的首次量化：103 块全部集中在单 chunk 单簇（y69=0/70=4/71=23/72=53，见 contradiction-facts 独立 MCA 解析），形状像一次 carver 挖洞或 feature 写入。注意 gen1 内存态 ≈ gen2 存档态（cave_air 都在）、gen2 内存态无 —— 存在**跨运行不确定性**（biome.rs HashMap→BTreeMap（M4）修复前嫌疑未排除，需复核 1.0.22 构建是否含该修复）。
- A3/A4 是 seed B 大类的缩微版（同机制，量少）。

### seed B（67,994 块；top15 覆盖 63,496，其余 4,498 = 6.6% 为 top15 以下散点）

| 机制类别 | 组成 | 块数 | 占比 |
|---|---|---|---|
| **B1. basalt deltas / 表面规则三大宗石互换**（biome→surface rule 链） | bs→basalt 12039 + basalt→nr 11915 + basalt→bs 10294 + nr→basalt 8042 + bs→nr 5165 + nr→bs 4623 | **52,078** | **76.6%** |
| **B2. soul sand valley 表面/涂布边界** | soul_soil→nr 3344 + soul_soil→soul_sand 1274 + soul_sand→nr 1102 | **5,720** | **8.4%** |
| **B3. 熔岩湖边界** | basalt→lava 1375 | 1,375 | 2.0% |
| **B4. nether 矿石 feature 差异**（与 A1 同家族） | basalt→quartz 1178 + nr→quartz 1037 + nr→gold 414 | 2,629 | 3.9%（+ top15 以下散点中可能更多） |
| **B5. magma block** | nr→magma 1098 + basalt→magma 596 | 1,694 | 2.5% |
| 未分类 | top15 以下 | 4,498 | 6.6% |

- B1 是决定性大类：basalt deltas 的 vanilla 表面规则（basalt/blackstone 按 surface rule 噪声涂布 + 填充-under 液面判断）与 Rust 实现大面积不一致——互换是双向大宗（basalt↔blackstone↔netherrack），呈"整片区域选错分支/阈值"形态，而非散点，指向 surface rule 条件链（biome 判定、noise 条件、steady/variant 分支）系统性偏差。
- B2 与 docs/09 遗留「soul_sand_valley 表面残差（y=1..2）」吻合，但块数放大说明本次 4×4 区域 soul valley 面积更大；soul_soil↔soul_sand↔netherrack 互换同为表面规则涂布家族。
- B3 vanilla=basalt→存档=lava：Rust 在 vanilla 铺 basalt 的位置留了 lava——熔岩湖边界/液面判定差（M7 已修 aquifers=false seaLevel 机制，边界条件仍有残差）。
- B5 magma_block 在 vanilla 来自 underwater_magma feature / 表面规则熔岩邻接判定，Rust 侧未放置或邻接判定差。

## 2. 与知识库已知残差家族吻合度

| 类别 | 吻合判定 | 依据 |
|---|---|---|
| B1 表面规则三大宗石互换 | **部分吻合（扩展）** | docs/09 遗留课题有 soul_sand_valley 表面残差与 Hole 语义不一致（`surface_depth<=0` vs `stoneDepthAbove<=0`，影响 nether lake/not(hole) 门控）——同属 nether surface rule 链家族；但 basalt deltas 大宗互换本身**无直接先例记录**，视为该家族的新成员（或 Hole 语义 bug 的下游表现之一，未验证） |
| B2 soul_sand valley | **吻合已知** | docs/09 遗留课题原文（y=1..2 soul_sand_valley 表面残差） |
| B3/A4 熔岩湖 | **部分吻合已知** | M7 lava 机制（aquifers=false → seaLevel 实现）已修主机制；边界残差是已知遗留（10 时间线：熔岩海带 y=32..63 遗留记录） |
| A1/B4 nether 矿石 | **吻合已知方向** | 10 时间线 L2160：`fill_chunk_blocks 的 carver/features/ore_vein 仍是主世界逻辑，nether/MOD 维度生成逻辑差异化`——nether ore feature 未实现是已知缺口，本次给出其存档级块数 |
| A2 cave_air 尾随簇 | **新类别（首次存档级量化）** | 知识库无「vanilla 尾随阶段写存档」家族条目；且伴随 gen1/gen2 内存态不一致的**跨运行不确定性**信号（M4 家族嫌疑），是全新课题 |
| B5 magma | **新类别** | 知识库无 nether magma_block 残差记录（overworld underwater_magma 有间接关联记录，nether 侧无） |

knowledge/discovered/algorithm-fingerprints.md、workflow-patterns.md 中无 soul_sand/gravel/涂布边界/熔岩湖专项条目（grep 零命中）——相关记录都在 docs/09 篇与 10 时间线。

## 3. 下一轮深挖优先级（块数 × 可定位性）

| # | 类别 | 块数 | 可定位性 | 建议定位探针（主会话可执行） |
|---|---|---|---|---|
| 1 | **B1 表面规则大宗互换** | 52,078 (76.6%) | **高**——layerMatch 锁定 y≤127；互换按区域成片 | ① 改 `compare_save_region.py`：按参照 biome 段（每 chunk 256 项 writeUTF biome 名，发现 #9 格式已知）把 mismatch 按 biome 分桶，确认是否全部落在 basalt_deltas/soul_sand_valley 列；② 单列对拍：选互换边界列（如 chunk(200,200) x=3207..3215 y=1），用 Rust `-biomeDump`/surface-rule trace vs `versions/1.20.1/data/worldgen/worldgen/noise_settings/nether.json` surface_rule 逐步条件核对（重点：biome 条件、`surface_depth<=0` vs `stoneDepthAbove<=0` Hole 语义、steady/variant noise 阈值） |
| 2 | **A1+B4 nether 矿石 feature** | 3,269（A1 640 是 seed A 残差 84.5%） | **中高**——feature.rs 骨架已有，缺 nether ore 清单 | feature 阶段 A/B：同一 chunk fill 两次（features on/off）diff 出 Rust 实际放置的矿位集合 vs 参照矿位集合（参照由 vanilla 导出分矿统计），确认是「未实现」还是「放置错位」（若错位→按发现 #6 查 PlacedFeatureIndexer 编号/DFS 链） |
| 3 | **B2 soul_sand valley** | 5,720 (8.4%) | **中高**——已知遗留课题，限定 y1..2 | 单层切片 diff：compare 脚本限 y∈[0,4]，输出 soul_soil/soul_sand/nr 互换的 (x,z) 图，对 soul_sand_valley surface rule 的 top/under/filler 链逐步对拍（同 #1 探针 ②） |
| 4 | **B3 熔岩湖边界** | 1,375 | **中**——M7 机制已修，疑边界条件 | mismatch 的 y 分布直方图（compare 脚本加 y 统计），若聚在 y=31/32（sea_level）附近 → 液面严格 `<` 边界/`not(hole)` 门控（Hole 语义课题）单点对拍 `(x,y,z)` 密度 |
| 5 | **A2 cave_air 尾随簇** | 104（但含不确定性信号） | **中**——位置极集中（单 chunk y70-72）反而易定位 | ① 复核 1.0.22 构建是否含 M4 BTreeMap 修复（biome.rs）→ 排除/确认跨运行漂移；② vanilla run 加 env 门控禁 carvers/features 重跑该 4×4，看 cave_air 簇是否消失（定位尾随写入者）；③ 存档写前后 hook：Mixin 在 save 前后各 dump chunk(203,200) y69..73 段 |
| 6 | **B5 magma** | 1,694 (2.5%) | 中低 | 与 #1 同脚本按 biome/邻接 lava 统计分桶（magma 是否总在存档 lava 邻接面）后定 underwater_magma 或 surface rule 归属 |

排序理由：#1 占 seed B 残差 3/4 且层位/形状已锁定；#2 同时是 seed A 的 84.5%，一次探针双 seed 受益；#5 块数最少但携带跨运行不确定性风险信号，价值在排除 M4 家族复发而非块数。

## 4. 诚实声明

本解读为 **Partial/Degraded 级**：仅对已有 run 的输出文件做静态分类与知识库比对，未执行任何新探针，机制归属（尤其 B1 的具体 surface rule 条件、A2 的尾随写入者、gen1/gen2 不确定性根因）均为**候选方向**，全部结论 status: **draft**，不做 confirmed；类别的机制解释需按 §3 探针验证后方可升 candidate。
