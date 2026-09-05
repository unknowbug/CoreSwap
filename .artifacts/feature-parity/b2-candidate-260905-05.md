# b2 candidate: placement modifier 链结构/消耗语义改变（blob 置换层回归）

- id: b2-candidate-260905-05
- status: draft
- 置信度: draft（假设 b2 基本否定；真实根因 UNCERTAIN）
- 验证分层: Degraded（静态逐行对拍 + 数据 JSON 比对，无运行时 probe）
- 输入: .tmp/feature-parity-260905-05/working-tree.diff；worldgen-core/src/{placement,feature_loader,feature,worldgen_handle,carver}.rs 现文件；Java 权威 .tmp/net/.../placement/{CountPlacement,RarityFilter,RepeatingPlacement}.java 与 versions/1.20.1/data/mc_src_extract/（同包）；数据 JSON .tmp/jar-check/worldgen-data/data/minecraft/worldgen/{configured,placed}_feature/ore_andesite*.json

## 1. 假设拆解与逐项判定

### 子机制乙（blob 个数同、单 blob 存活块多——discard_chance 层）：**否定（高置信）**

- 数据权威：`configured_feature/ore_andesite.json` → `"discard_chance_on_air_exposure": 0.0`。
- Rust 侧 `feature.rs:356-360` `should_not_discard`：chance<=0 直接 `return true`（不消费 RNG、不调 is_exposed_to_air）——新旧 dll 同一段代码，且 **working-tree.diff 未触碰 feature.rs 任何一行**。
- 解析键名对拍：JSON `discard_chance_on_air_exposure` ↔ feature.rs:113 同名，无键名漂移。
- 量化：乙要求「此前丢弃 ~19% 的尝试现在放行」；但 andesite 两版都从不进入 discard 路径（0 丢弃 → 0 丢弃），**所需 discard 差 = 不存在**。OreFeature.generateVeinPart 几何 RNG（feature.rs:213-330）新旧逐字一致，单 blob 几何不可能变。
- 乙剩余变体（target 匹配层放行更多）也不成立：RuleTest::parse 与 target.test 未被 patch 触碰。

### 子机制甲（blob 个数多——count/raffle 层）：**否定（中高置信）**

- andesite 链逐字对拍：ore_andesite_lower = count(2)→in_square→height_range(uniform 0-60)→biome；ore_andesite_upper = rarity_filter(6)→in_square→height_range(uniform 64-128)→biome。patch 对这条链的解析/执行路径零改动（placement.rs diff 只动 Trapezoid/BiasedToBottom IntProvider、新增 modifier 类型、BlockPredicateFilter 谓词树、Biome 分支）。
- count/raffle 语义对拍 Java：
  - CountPlacement（RepeatingPlacement.count = IntProvider.sample）↔ `Count(count) => count.get(random)` 循环 n 次——一致（Constant 恒 0 消费，与 ConstantInt 一致）。
  - RarityFilter.java:23 `nextFloat() < 1.0f/chance` ↔ Rust `next_int_bound(chance)==0`——**分布等价但 RNG 调用形态不同**（nextInt(31) 拒绝采样 vs next(24)），这是一个预先存在的偏差（新旧 dll 同此实现），不构成回归，登记为 idk。
- **跨 feature RNG 隔离（决定性）**：worldgen_handle.rs:879 每个feature 前执行 `feat_random.set_decorator_seed(population_seed, p, k)`——每个 placed feature 从自己的种子起步。patch 中所有 RNG 消费语义改动（HeightProvider trapezoid、IntProvider::Trapezoid、BiasedToBottom 公式）只改变**该 feature 自身**流内的消费，**不会传播进 andesite 的流**。andesite 自身走 uniform 路径（i<j → 恒 1 消费），新旧消费数相同。
- patch 里唯一实际改变 andesite 链行为的项：**Biome modifier 从恒直通变为真过滤**（placement.rs Biome 分支 + anchor_biome 注入）。方向为**减少** blob（4×4 cell biome ≠ chunk biome 的位置被滤掉），与 +19% **增多**符号相反。且 patch 的树/植被仅进 VEGETAL step(k=9)，ores 在 k=6 之前，p 索引按 step 内列表，互不移位。

## 2. 能否解释证据三要素？

| 证据 | 甲 | 乙 | b2 整体 |
|---|---|---|---|
| 全带均匀 +19%（Y 域不变） | 形态可容（多 blob 按同 Y 分布比例落带） | 形态可容但机制不存在（discard=0） | ✗ |
| count-equal 51%（旧 83%） | 半数 chunk 增益，勉强相容 | 若几何同则应 ~100%相等 | ✗ |
| 机制入口 | 链代码逐字未变 + RNG per-feature 隔离 | 未触碰 feature.rs | **✗ 无法成立** |

数量级核对：738.8→881.4 = +142.6/chunk；size-64 blob 实际落块 ~300-350 → 约 **+0.4 blob/chunk**，非整数 blob——形态上更像「部分 chunk 多 1 个 blob」或统计口径因素，不是单 blob 均匀增厚。

## 3. 结论

- **b2（placement 链结构/消耗语义改变）不能解释 andesite +19%**：乙被数据（discard=0.0）+ 代码未触碰双重否定；甲被「链逐字未变 + set_decorator_seed per-feature 隔离」否定。唯一行为变化（Biome filter 激活）方向相反。
- 真实根因：**UNCERTAIN**。+19% 必然来自 patch 之外与 b2 范围之外的层。提示方向（供 b3+ 竞争）：
  1. **blob 数/尺寸经验切分缺失**——v4 只测了总量； decisive probe = WG_FEATURELOG（已存在 env 门控，worldgen_handle.rs:880/923）统计 per-chunk andesite blob 尝试数与 placed 数（新 vs 旧），直接切「blob 多」vs「单 blob 块多」；
  2. 或 blob 连通域统计（v5 脚本：per-chunk connected components 的 count 与 size 分布）；
  3. 或导出/统计口径：Y 分布归一是跨域除法（vanilla 1333 chunks vs new 3009 chunks，§9.7 可比性——pairs 内均值 +19% 可信，Y 分布表跨域不可直接比）；
  4. 注意 Biome filter 激活本身是新引入的独立偏差（方向：某些多 biome chunk 少 blob），修复验证时应一并核。

## 4. 自检与 idk

- 已对拍：CountPlacement/RarityFilter/RepeatingPlacement.java、OreFeature discard 路径、HeightProvider/IntProvider 消费数、set_decorator_seed 隔离、andesite 数据 JSON。
- @anchor.idk: RarityFilter Rust 用 nextInt 形态 vs Java nextFloat 形态，分布等价但流形态不同——预存偏差，未量化对 andesite_upper（rarity 6）位置选择的影响。
- @anchor.idk: `biome_at` 闭包（worldgen_handle.rs:888）具体为 jitter 还是 no-jitter 未逐行核实——只影响 Biome filter 偏差幅度，不影响本判定。
- 未做运行时验证（subagent 无 shell）；本产物全部为静态一手对拍，Degraded 声明。
- retry 轮次：1（静态一轮，无数据层新证据消耗）。
