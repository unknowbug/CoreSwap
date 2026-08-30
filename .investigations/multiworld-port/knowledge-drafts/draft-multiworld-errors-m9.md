# 草稿：multiworld-errors.md 追加 M9（subagent 产出，主会话应用）

> **应用位置**：`.investigations/multiworld-port/multiworld-errors.md`——M8 条目之后、「附：错误 → 根因 速查表（一页索引）」之前。
> **应用动作**：① M9 全文插入；② 速查表追加 M9 一行（见文末「速查表追加行」节）。
> **status**：candidate（源码机制部分源码确认；「净效果为负的解读」与「legacy 下界 worldSeed 无关性」为未验证推论/假设，已标注）。
> **证据源**：yarn sources（2026-08-30 从 loom sources jar 提取，落盘 `.investigations/multiworld-port/NoiseConfig.java` / `OctavePerlinNoiseSampler.java` / `DoublePerlinNoiseSampler.java`）；实测数字来自主会话 blocks_cmp 记录（v6/v7 两轮）。

---

## M9. legacy climate visitor 固定种子特例：逐语义正确移植后净效果为负（82.72% → 77.01%）——env 门控回退保留

> 本条特殊：**实现逐条语义正确（源码逐行核对），但实测净效果为负，已回退默认态**——错误不在「移植错了」，在「全局耦合改动的消融缺失 + biome 判定前置未闭合」。记录它是因为「正确移植 ≠ 正确合入」这条判错经验比一般 bug 更易复发。

### 现象
- M6 修复 `legacy_random_source` 布尔解析后，legacy 分流首次真正激活。实现本轮 legacy climate visitor 特例（`density_builder.rs get_noise_sampler` 特例 + `old_blended_noise` legacy 分支）后实测：
  - **v6（无特例基线）**：nether **82.72%**（Hole 修后）；y32..63 **65.78** / y64..95 **55.17** / y96..127 **61.03**。
  - **v7（特例启用）**：nether 跌至 **77.01%**——y32..63 **暴跌 65.78 → 22.37**；y0..31 79.57 → 72.57；y64..95 55.17 → 55.51（微升）；y96..127 61.03 → 65.64（微升）；nonAir 63.3 → 70.5（提升）。
- 回退默认（`WG_LEGACY_CLIMATE=1` 门控关闭）后 nether 恢复 82.72%；overworld 全程 95.40% 零回归。

### 根因（机制——两层：机制发现为源码确认，净负解读为假设）

**机制发现（源码确认）**：`legacy_random_source=true`（下界）时，NoiseConfig 构造 noiseRouter 前，`LegacyNoiseDensityFunctionVisitor` 对**整棵 router 树**做替换（`.investigations/multiworld-port/NoiseConfig.java`，搜 "OFFSET" 定位 bl 分支）：
1. temperature noise → `DoublePerlinNoiseSampler.createLegacy(CheckedRandom(0+0), NoiseParameters(-7, [1.0, 1.0]))`（`.investigations/multiworld-port/DoublePerlinNoiseSampler.java` create/createLegacy）；
2. vegetation noise → `createLegacy(CheckedRandom(1+1), NoiseParameters(-7, [1.0, 1.0]))`；
3. offset noise → `create(randomDeriver.split("minecraft:offset"), NoiseParameters(0, [0.0]))`——**振幅全零 → 采样恒 0**（legacy 下界 shift_x/shift_z 无偏移）；
4. `InterpolatedNoiseSampler`（old_blended_noise，地形主干）→ `copyWithRandom(createRandom(0))` = **CheckedRandom(0) 完整替换随机源**——不是 overworld 的 `randomDeriver.split("minecraft:terrain")`（`.investigations/multiworld-port/OctavePerlinNoiseSampler.java` createLegacy/useLegacy 构造）；
5. 其余 noise → 常规 `getOrCreateSampler`（`randomDeriver.split(id)`）。

即：**legacy 下界的 climate 噪声与地形主干全是固定种子特例**，随 worldSeed 变化的分量为零。⚠️ 推论「legacy 下界地形 worldSeed 无关」——**未验证**（仅由固定种子清单反推，未做双 seed 对拍实验，待验证后再定）。

**净效果为负的解读（假设，非定论）**：legacy climate 替换改变 biome 判定 → nether 3D 表面规则（biome 条件 ×5：soul_sand_valley/crimson/warped/basalt 的涂布）连锁变化。y32..63 暴跌说明涂布结果离 vanilla 更远：**Rust 的 biome 判定本身仍是 nether_wastes 误判**（见 M8 附记 soul_sand 诊断）→ **正确的 climate 噪声 × 错误的 biome 判定 = 比之前更差**——之前错误的 climate 噪声碰巧让部分 biome 条件不命中 → 保持 base netherrack 反而多对。推论：**biome 分类精度修复（前置）与 legacy climate 移植（本条）存在顺序依赖**——先修 biome 判定，legacy climate 才能兑现正向收益。

### 定位（诊断链）
1. **分带对比**：v6 vs v7 按 y 分带 match——非均匀变化（两带暴跌 + 两带微升 + nonAir 提升）指向「表面规则/biome 涂布层」而非「密度主干」（密度错应全高度均匀低分）。
2. **源码核对定位实现内容**：读三份 yarn 权威源码（NoiseConfig/OctavePerlinNoiseSampler/DoublePerlinNoiseSampler，2026-08-30 从 loom sources jar 提取落盘）逐条确认 visitor 替换清单——确认 Rust 实现逐语义正确（temperature seed=0、vegetation seed=2、offset 恒零、blended CheckedRandom(0)），**排除「移植写错」假设**。
3. **净负判定 → 消融回退**：总分下降即触发回退验证（门控默认关 → 恢复 82.72%），确认净负来自本改动本身、非环境漂移。

### 修复（回退而非删除）
- 实现保留：`density_builder.rs get_noise_sampler` 特例——temperature → `DoublePerlinNoiseSampler::new_legacy(RsRandom::Legacy(LegacyRandom::new(0)), -7, &[1,1])`；vegetation 同上 seed=2；offset → `DoublePerlinNoiseSampler::zero()`（恒 0）；blended → `RsRandom::Legacy(LegacyRandom::new(0))`。
- **env 门控回退**：`WG_LEGACY_CLIMATE=1` 启用，默认关——保留已实现工作 + 维持最佳默认态。
- 前置课题登记：biome 分类精度修复（nether_wastes 误判）完成后复测本开关。

### 教训（可复用判错经验）
- **「逐语义正确 ≠ 整体正确」**：visitor 替换是**全局耦合改动**（改 climate → biome → 表面三层连锁），逐条核对 Java 源码只保证「每个叶子对」，不保证「耦合网络对」。这类改动**必须消融验证子项**（分带/分阶段看各自增减），不能只看总分——总分降不代表方向错，可能是「正确零件装进还错的机器」。
- **「正确实现 × 错误依赖 = 比错误实现 × 错误依赖更差」是真实风险**：两个 bug 之间可能存在负相关抵消（错误 climate 碰巧压制了错误 biome 条件的涂布）。修 bug 时若另一个已知 bug 未修，须预期「修对反而降分」，降分本身不是回滚正确修复的证据——应记录分带证据、门控回退、留前置依赖课题。
- **env 门控回退 = 净负结果的标准处置**：不删除已实现工作（源码核对过的移植成本真实存在），门控默认关维持最佳态，待前置修复后零成本复测。
- **净负结果不是白做**：固定种子特例清单本身澄清了「legacy 下界的 worldSeed 无关性」疑点（待验证推论）与 biome 采样的耦合关系——机制发现是资产，即使当下不启用。

---

## 速查表追加行（并入文末「错误 → 根因 速查表」）

| 现象/信号 | 根因 | 判错要点 |
|---|---|---|
| legacy climate 特例逐语义移植后 nether 82.72% → 77.01%（y32..63 暴跌 65.78→22.37，两带微升，nonAir 提升）（M9） | 实现**逐语义正确**（源码逐条核对排除移植错）；净负 = 全局耦合：正确的 climate 噪声 × 仍误判的 biome（nether_wastes）→ 涂布离 vanilla 更远（此前错误 climate 碰巧压制部分 biome 条件反而多对）。假设标注：biome 修复前置与 legacy climate 存在顺序依赖 | **逐语义正确 ≠ 整体正确**——visitor 全局替换类改动必须消融验证子项（分带/分阶段），不能只看总分；「修对反而降分」可能是负相关抵消，处置 = 分带证据 + env 门控回退（保留工作、维持最佳默认态）+ 前置依赖课题，而非回滚正确修复；附推论：legacy 下界 climate/地形主干全固定种子 → worldSeed 无关性（未验证） |
