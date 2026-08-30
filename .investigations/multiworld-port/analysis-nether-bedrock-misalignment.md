# nether 基岩层错位根因分析（static，Degraded 层级：只读静态分析，未跑运行时验证）

- 分析者：core-worker subagent（隔离，只读）
- 输入探针：`.investigations/multiworld-port/cmd-output/nether_blocks_match_v3_current.txt`
- 状态：draft（candidate 需运行时复现：修 legacy deriver 后 block_probe 复测）
- 验证分层声明：**Degraded**（静态审查），运行时确认步骤见文末

## 0. 探针数据再解读（重要修正）

`v3` 输出按 y 带统计：

| y 带 | 匹配率 |
|---|---|
| y=0..31 | 59.50% |
| y=32..63 | 7.92%（熔岩带，另一 worker 范围） |
| y=64..95 | 49.36% |
| y=96..127 | 75.51% |
| y≥128 | 100%（双方均为 air） |

即 nether 实心区域整体差异很大，不只是 y=1..2 基岩带；`first mismatches` 只列了扫描顺序（自下而上）最先命中的 10 条，全部落在 y=1..2 是扫描序偏差，**不代表错位仅限基岩带**。但基岩带的双向错（got=31/want=256 与 got=256/want=31 并存）本身仍指向随机流不一致，见下。

## 1. 根因结论

### A. vertical_gradient 随机流用错 random source —— **nether 是 legacy_random_source 维度，Rust 只实现了 Xoroshiro**（根因，结构级，置信度高）

**语义链（vanilla 1.20.1，Mojang mappings 记忆级 + JSON 铁证）：**
- `nether.json` L13：`"legacy_random_source": true`（overworld 为 false）。
- Java `RandomState` 构造：`legacyRandomSource ? new LegacyRandomSource(worldSeed) : new XoroshiroRandomSource(worldSeed, 1033234444, 1024269689)`。该 base source 决定两件事：
  1. **全部 perlin 噪声采样器的种子派生**；
  2. `getOrCreateRandomDeriver(id)`（即 surface rule 的 positional deriver）。
- `SurfaceRules.VerticalGradientCondition.test()`：`randomState.getOrCreateRandomDeriver(randomName).at(x, y, z).nextFloat() < d`。legacy 分支下 `at(x,y,z)` 走 **java.util.Random LCG**（`LegacyRandomSource.LegacyPositionalRandomFactory`），与 Xoroshiro 的 `split(x,y,z)` 完全不同的随机流。

**Rust 侧证据：**
- `WorldgenRust/src/surface_rules.rs` L289-299 `splitter_for`：`self.splitter.split_str(name).next_splitter()` —— 全 Xoroshiro。
- `surface_rules.rs` L177-190 `vertical_gradient_test`：`s.split_xyz(x, y, z)` + `r.next_float()` —— 全 Xoroshiro。
- `rg -n "legacy" WorldgenRust/src/`：**不存在任何 LegacyRandomSource / java.util.Random LCG 实现**；命中全是 perlin 的 `new_legacy` 构造分支（种子消费方式，非 LCG 算法）。
- `worldgen_handle.rs` L158：噪声采样器 `db.random_deriver().split_str(id)` —— 同样只有 Xoroshiro 路径。

**与观测症状的吻合：**
- bedrock_floor 概率带：d = clampedMap(y, 0, 5, 1, 0) → y=1:0.8、y=2:0.6、y=3:0.4、y=4:0.2。两条独立随机流在该带内逐列独立判 true/false → **双向错**（Rust 中 bedrock 处 vanilla 出 netherrack/soul_sand，反之亦然），与 first mismatches 的 `(5,1,0) got=256 want=31` / `(12,1,0) got=31 want=257` 完全一致。
- 反证排除：若 d 方向反了（假设 A 的子项），Rust bedrock 会系统集中在 y=3..4 而 vanilla 在 y=1..2，错的应是"层带整体位移"，且 got/want 不会出现 `got=31 want=257` 这种"vanilla 在 y=1 出 soul_sand"（vanilla soul_sand 路径 `not(hole)`+`y_above 30..35 add_stone_depth` 在深部由 stone_depth_above 满足——Rust 与 vanilla 在该列 random roll < 0.8 差异即可解释）。d 方向核对无误：L185 `lerp_clamp(y, true_y, false_y, 1.0, 0.0)`，y=0→1、y=5→0，与 Java clampedMap 同序。
- `next_float` 语义本身无错（`xoroshiro.rs` L96：`(next() >> 40) as f32 * 5.9604645E-8`，24 位定点 [0,1)，= Java Xoroshiro `nextFloat`）——错的是送进它的随机源，不是 nextFloat 本身。

**附带证据（C++ 同病）**：`versions/1.20.1/cpp/worldgen/src/surface.h` L282-293 `VerticalGradientCond::test` 同样 `ctx.splitterFor(name)`（Xoroshiro）。docs/09 的修复链只修了"反锚序（先 false 后 true）"，没处理 legacy random source——**C++ 对 nether 也有同一 bug**，不能当已修复参照，只可参照其反锚序结论。

### A-extension. legacy_random_source 同样决定 nether 全部噪声种子 → y=32..127 的大面积错位可能同根（置信度中，建议另行运行时消融）

- Java：legacy=true 时 perlin/双 perlin 采样器由 `LegacyRandomSource(seed)` 派生；Rust `worldgen_handle.rs` L158 用 Xoroshiro `split_str`。种子不同 → 地形整体不同 → 解释 y=64..95 只有 49.36%、y=96..127 75.51%。
- 此项超出本次基岩范围，但**若只修 surface deriver 不修噪声种子，nether 对齐率不会有数量级改善**。消融建议：`WG_SKIP_CARVER/WG_SKIP_SURFACE/WG_SKIP_OREVEIN` 逐层开关 + density probe 对照 nether，先定位噪声层差异占比。

### B. anchor 解析 —— 排除（置信度高）

`surface_rules.rs` L940-945 `parse_anchor_abs_y`：
- `above_bottom: v` → `min_y + v`；nether bedrock_floor true=0、false=5 ✓
- `below_top: v` → `min_y + height - v`；bedrock_roof true_at_and_below=`below_top 5`→123、false_at_and_above=`below_top 0`→128 ✓（vanilla `YOffset.belowTop` 语义一致，128 为顶上开区间界）

nether min_y=0 无 absolute/相对转换 off-by；overworld `absolute` 直读也无转换。假设 B 不成立。

### C. hole 条件 —— **确认为与 Java 的真实分歧，但不是基岩错位的原因**（分歧判定高置信；对基岩无影响确定）

- Rust `surface_rules.rs` L98-100：`Hole => ctx.surface_depth <= 0`，注释声称"对齐 Java runDepth、C++ 用错字段是 bug"。
- Java `MaterialRules.HoleMaterialCondition`（Yarn `hole()`）：`return this.context.stoneDepthAbove <= 0;` —— 用的是**垂直扫描的 stoneDepthAbove**（`initVerticalContext(q, vx, r, ...)` 的第一参，L1160 传入的 `q`），不是 runDepth/sampleRunDepth 噪声。**Rust 注释对 Java 的引用是错的，C++ L251（`stoneDepthAbove <= 0`）才是对的**。
- 影响：`hole`/`not(hole)` 参与的规则——nether L432 熔岩 lake 判定、L600/L710 nether_wastes soul_sand/gravel 门控、overworld 水湖/熔岩湖边缘。**不进 bedrock_floor/roof 两条规则，故与本课题 y=1..2 基岩错位无关**；但它是独立真实 bug（建议单开课题修，且 overworld 已有 @anchor 验证结论覆盖 hole 的场景需复核）。

### D. 规则树解析丢分支 —— 排除（对 nether.json；置信度高），但有一个潜在静默丢弃路径要登记

- 逐节点核对 `parse_surface_rule`（L946-975）/`parse_surface_cond`（L976-1032）对 nether.json 全部节点类型：sequence/condition/block/not/biome/y_above/stone_depth/noise_threshold/vertical_gradient/hole —— 全部支持，递归处理 `condition→sequence→condition`（L949-958、L963）无死路。无静默丢支。
- 潜在隐患（nether 未触发，登记）：① 未知 cond 类型 → `parse_surface_cond` 返回 None → L962 `?` **整条 condition 分支静默消失**（sequence L954 同样静默跳过）；② then_run 解析失败 → L964 回退 `SurfaceRule::Block(0)` → 写 block id 0。建议改为带告警日志/显式错误，防未来 JSON 演进再踩。

## 2. 修复方向（伪代码级）

1. **实现 LegacyRandom（java.util.Random LCG）+ LegacyPositionalRandomFactory**（对应 Java `LegacyRandomSource`）：
   - `setSeed(s)`: `seed = (s ^ 0x5DEECE66D) & ((1<<48)-1)`；`next(bits)`：`seed = seed*0x5DEECE66D + 0xB`（mod 2^48），返回 `(seed as i64) >> (48-bits)`；`next_float = next(24) * 2^-24`。
   - 位置派生 `at(x,y,z)` 与 `fromHashOf(name)` 的混合常数必须对照 yarn 源码 `LegacyRandomSource.LegacyPositionalRandomFactory` 逐行抄（此为本次静态分析未覆盖点，@anchor.idk：常数未核）。
2. **RandomState 等效分流**：`create_for_dim` 读 settings JSON 的 `legacy_random_source`（nether.json L13）→ legacy 时：噪声采样器派生与 `splitter_for`/`split_str` 全部切 LegacyRandomSource 路径；非 legacy 保持现 Xoroshiro（overworld 不动，避免污染已验证结论）。
3. **Hole 复位**：`SurfaceCond::Hole => ctx.stone_depth_above <= 0`（Java 对齐；重跑 overworld surface 回归确认无退化）。
4. 验证路径：修 1+2 后 block_probe nether 复测 → 预期 y=1..4 基岩带双向错消失；再按 A-extension 消融噪声种子差异。

## 3. 置信度汇总

| 项 | 结论 | 置信度 |
|---|---|---|
| A 主体 | vertical_gradient 用 Xoroshiro 而 vanilla nether 用 legacy LCG → 基岩带双向错 | 高（结构级铁证）；运行时复现后升 candidate |
| A d 方向 / next_float | 无错 | 高 |
| A-ext | legacy 同污染 nether 噪声种子 → y=32..127 大面积错位同根 | 中（需消融定位） |
| B | 排除 | 高 |
| C | 与 Java 真实分歧（应改 stone_depth_above<=0）；非基岩原因 | 分歧判定高 / 因果排除确定 |
| D | nether.json 无丢支；静默丢弃路径登记为隐患 | 高 |
