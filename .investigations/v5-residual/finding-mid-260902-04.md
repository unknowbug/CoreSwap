# V5 残差中间结论（260902-04，draft，探测进行中）

## 已验证事实（数据层证据）
1. 存档口径复现：nether 4x4@3200,3208 seed B = **96.6215%**（与 confirmed 逐位一致）。seed 三查 ✓。
2. 残差构成：35426 mismatch，93.6% = {netherrack 256, basalt 259, blackstone 849} 互混淆；空间聚集西北半区（chunk 200-201 行），y 全带分布（0-15 与 80-103 双峰）→ 整柱级材质差。
3. **一步裁决 1（biome 通道）**：35426 个 mismatch 列 **100% world=warped_forest**（vanilla 参照列 biome：94.4% basalt_deltas + 5.6% soul_sand_valley）。残差 = warped_forest 误判区的表面规则产物。
4. **一步裁决 2（输入对拍）**：Java [BIOME6] 与 Rust biome6_dump 同 4 点 6 维值**逐位一致**（t=-0.5598/h=-0.2920 等）；两侧数学均判 basalt_deltas（dist 0.119 vs warped 1.080，非平局）。
5. Rust 独立分类器（biome6_dump，生产同路径组件重组）判定 = basalt_deltas/soul_sand_valley = **与 vanilla 参照一致**。
6. fillChunkNether 只写方块（CppBridge L242-280，无 biome 写入）；populateNoise 拦截时 chunk status 已是 minecraft:biomes（biome 容器在 BIOMES 阶段已由 Java 填充）。

## 当前矛盾（待解）→ 已解（260902-04 15:40）
「全 warped」是**探针坐标 bug 假象**：ReadWorldProbe 的 wBiome 误用 chunk 局部 x,z 查 biome（实际查到 chunk(0,0) 区域=warped_forest）。修正为世界坐标后（mismatch-nether-run6.csv）：
- **96.3% 残差列 biome 完全一致**（basalt→basalt 32817 列 + ssv→ssv 1306 列）；
- biome 真差仅 1303 列（3.7%）：ssv→basalt 676 + basalt→ssv 627（边界互换，签名 A 仅此占比）。
- **残差主体 = 同 biome（basalt_deltas）下表面规则判定差**：vanilla 写 basalt/blackstone 的列，Rust 写 netherrack 或 basalt/blackstone 互换——即 B1 家族规则执行层（与 NEXT 台账 B1 basalt −1736/blackstone −434 同族）。

## 最终判定（T2 闭合，candidate 已 judge：有条件 PASS 260902-04，P1-P5 已落实）
1. 残差主体 = 同 biome 表面规则差（**96.32%** biome 一致，其中 **basalt→basalt 32817 列 = 92.6%** 主体、ssv→ssv 1306）→ B1 家族规则执行层（下钻候选：selector 采样/delta 分支进出/basalt·blackstone 分配）；签名 A（ssv↔basalt 互换）= 1303 列 ≈ **3.68%**，可单独修。
2. 分类器排除（.b1/.b2/.b4 + 4 点 6 维逐位一致）成立。
3. per-id 精确量化（vanilla→save 净差，match 行精确等价法，正负总和归零）：netherrack(256) **+1539** / basalt(259) **−1050** / blackstone(849) **−652** / soul_soil(258) +297 / soul_sand(257) +37 / **gravel id=?? 净差 0**；tail：nether_quartz_ore(417) +82 / lava(33) −109 / air(0) −128 / nether_gold_ore(45) −26 / magma_block(607?) +9 / red_mushroom(162?) +1（id→名映射 = versions/1.20.1/data/blocks.json；judge P1 已澄清：417=quartz 非 gravel）。

## judge 补充条件落实（P1-P5）
- P1 映射声明：如上，417=nether_quartz_ore ≠ gravel，gravel 净差确为 0。
- P2 措辞：主体=同 biome 96.32%（basalt→basalt 92.6%），已改。
- P3 git 声明：**wBiome 坐标修正探针在 runtime/（gitignored），不在 git diff 可审范围**——由 judge 以 run5（100% warped）/run6（修正后）数据侧独立复算兜底。
- P4 覆盖面：单 seed B、4x4@3200,3208 单域，96.32%/3.68% 占比外推性有限，升级跨域结论前需扩 seed/区域。
- P5 取代链：fanout-biome-candidates.md 前提「100% warped」被本 finding 取代（supersedes → finding-mid-260902-04.md），日期标签统一 260902-04。

## 探针过程教训（知识库候选）
- 探针自身坐标 bug 会制造 100% 单向假象（wBiome 局部坐标）——「对比前先核坐标语义」铁律需扩为「探针输出先做 sanity check：对照组/已知区抽查」；
- 同步/残留进程占 session.lock 导致静默失败；kill java 后重跑。

## fan-out 候选状态（.investigations/v5-residual/fanout-biome-candidates.md）
- .b1 offset 维语义：排除（worker 静态对拍 vanilla 一致）
- .b2 shift/Perlin 种子差：**排除**（4 点 t/h 逐位一致）
- .b4 SearchTree/平局：排除（距离差 9 倍非平局）
- **新候选 .b6（本轮新立）**：cppReplace 存档的 biome 存储填充路径异常——populateBiomes 的输入/时机被接管链改变（如 NoiseConfig 实例不同、BIOMES 状态下的 storage 被后续覆写、或 vanilla biome fill 对被拦截 chunk 的行为差异）
- 下一步一步裁决探针：cppReplace 产物存档（run/world，seed B）**直接读 DIM-1 region 原始 biome cell**（getBiomeForNoiseGen，无 BiomeAccess 平滑），对照 vanilla 数学期望图（由 t/h 场线性复算 5 条目距离）。若原始 cell=warped → .b6 存储填充异常坐实，再查 populateBiomes 在拦截链的输入；若原始 cell=basalt 而仅 BiomeAccess 平滑翻转 → 8 邻域选点差。

## 口径声明
- 残差/对齐：存档读回 vs vanilla 参照 blocks（四要素：B/4/3200,3208/nether），与 M16 起 96.62% 口径可比。
- biome 对比：ReadWorldProbe CSV（vanilla 列 biome @y=100 world.getBiome 平滑值 vs save 同方法）+ BIOME6/biome6_dump 原始 router 采样（UnblendedNoisePos 直采）。
