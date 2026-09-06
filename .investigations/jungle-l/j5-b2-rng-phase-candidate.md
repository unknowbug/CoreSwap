# J5 fan-out 候选 .b2：树位选择层 RNG 流相位差（j5-b2-rng-phase-candidate）

> 角色：core.worker（fan-out .b2，re-code + swe 混合）；状态：**draft**。
> 验证分层：**Degraded（静态）**——全部结论来自两侧一手源码逐行核对（Java 参照 = E:\PYTHON\MC\data\mc_src_extract\；Rust = worldgen-core\src\），无运行时对拍数据。置信度全部 ≤ candidate 候选，confirmed 留人类。
> 课题：seed 8576294172403134396，chunk (29,-16)（x∈[464,480), z∈[-256,-240)）；Java 2 棵 mega vs Rust native 载具 0 棵（.investigations/jungle-l/j5-trace-verdict-260906-01.md 附录 2）。
> 候选边界：本文件只管「**位置随机流的种子派生 + 消费相位**」；attempt/selector 计数、遍历语义归 .b1，不深钻。
> seed 三查声明：本文为静态分析未接触运行数据；任何探针对比第一动作 = 核对两侧 `worldSeed=8576294172403134396`（见 §4 P0 行）。

## 0. 关键结构事实（定相位的骨架，静态已核）

Java（ChunkGenerator.java:334-423，一手源）：
- :343 `chunkRandom = new ChunkRandom(new Xoroshiro128PlusPlusRandom(RandomSeed.getSeed()))` —— **FEATURES 阶段 baseRandom = Xoroshiro**（每 chunk 新建；初始 seed 非确定但立即被 setSeed 覆盖，不影响确定性）。
- :344 `l = chunkRandom.setPopulationSeed(world.getSeed(), blockPos.getX(), blockPos.getZ())`，blockPos = chunk 起点 (chunkStartX, *, chunkStartZ)，Y 不参与。
- :360-414 按 step k 外层、每 step 内 structures（:364 setDecoratorSeed(l,m,k)）+ placed features（:402 **setDecoratorSeed(l,p,k)**，p=全局索引，intSet 升序）逐个放置。
- **每个 placed feature 都重设种子**（:402）——消费相位**不跨 feature 累积**：一棵树的位置 (x,z) 完全由 (populationSeed l, p, k, 该 feature 自身 modifier 链消费序) 决定。

因此 .b2 的可分差点只有三类：① populationSeed 派生差；② decoratorSeed 输入 (p,k) 差（→ 实质是 .b1 的 p 映射，本文只列不钻）；③ **同 feature 内消费序/消费次数差**（modifier 语义、IntProvider/HeightProvider 退化边界）。

## 1. 派生链逐段核对（Degraded 静态）

| # | 段 | Java 证据 | Rust 证据 | 结论 |
|---|---|---|---|---|
| A1 | populationSeed：setSeed(worldSeed)→2×nextLong 各 |1，n=bx*l+bz*m ^ worldSeed | ChunkRandom.java:54-61 | chunkrandom.rs:163-171（wrapping 乘加、next_long=2×next(32) 有符号拼接） | **对齐**（静态） |
| A2 | xoroshiro 种子派生：lo=seed^SILVER、hi=lo+GOLDEN、stafford13×2 | RandomSeed.java:23-31, 56-58 | xoroshiro.rs:24-31（常量一致） | **对齐**（静态） |
| A3 | xoroshiro128++ next() 本体 + 0 值回退 | Xoroshiro128PlusPlusRandomImpl.java:28-45 | xoroshiro.rs:40-54（回退 lo=GOLDEN hi=SILVER 同） | **对齐**（静态） |
| A4 | ChunkRandom.next(bits)（xoroshiro 基类取高 bits） | ChunkRandom.java:29-32（`>>>`） | chunkrandom.rs:111-117（算术 `>>` + `as i32`） | **等价**：bits≤32 时低 32 位截断后与 `>>>` 逐位相同（静态推演，见 @anchor.idk-1） |
| A5 | BaseRandom 默认 nextInt(bound)/nextLong/nextFloat | BaseRandom.java:14-48 | chunkrandom.rs:119-157 | **对齐**（含 16 幂 2 → next(31)，拒绝采样回绕判据） |
| A6 | decoratorSeed = population + index + 10000*step（全 64 位，无 48 位截断） | ChunkRandom.java:75-78 | chunkrandom.rs:174-177 | **对齐**（静态） |
| A7 | in_square：nextInt(16) 先 x 后 z | SquarePlacementModifier.java:19-23 | placement.rs:300-305 | **对齐**（消费序一致） |
| A8 | PlacedFeature 惰性深度优先（位置 1 走完整链→位置 2） | PlacedFeature.java:48-63 | placement.rs:470-494（visit 递归） | **对齐**（静态） |
| A9 | Biome modifier 0 RNG 消费 | AbstractConditionalPlacementModifier 族 | placement.rs:322-334 | **对齐** |

## 2. 相位/派生差异点清单（本候选核心产出）

> 置信度：均为 draft 级静态判断；「静态已核」= 有双边行号证据；「候选」= 单边/推演。

### D1（候选，**优先级最高**）IntProvider::Uniform 退化边界 0 消费
- Java：`UniformIntProvider.get` → `MathHelper.nextBetween` → `nextInt(max-min+1)+min`（UniformIntProvider.java:40-42、MathHelper.java:846-848、Random.java:76-78）——**min==max 时仍消费 1 次 nextInt(1)**（幂 2 → next(31) = 1 个 xoroshiro 输出）。
- Rust：`IntProvider::Uniform(a,b) if a >= b => return *a`（placement.rs:27-31）——**0 消费**。
- 机制：凡 placed feature 的 count/xz_spread/y_spread 等用 `uniform` 且 min==max（JSON 写成 uniform 而非常量 int），Rust 少消费 1 输出 → **其后 in_square 的 dx/dz 全体错位 → 同 chunk 选出不同 (x,z) 甚至 0 棵**。这精确复现「同结构、位置不同」的 .b2 表征。
- 验证缺口（@anchor.idk-2）：目标 chunk 实际加载的 trees placed feature JSON 的 count 是否为退化 uniform 未核（vanilla 数据包常见写法是常量 int，但 biome-specific 派生链需实测）；探针 P2 直接判别。

### D2（候选，中）IntProvider::Trapezoid 退化边界 0 消费
- placement.rs:37-39 `if k <= 0 return *a`（0 消费）；对照 HeightProvider 侧同族实现 carver.rs:104-106 `plateau >= k → next_int_bound(k+1)`（恒 1 消费）——同一退化条件两处消费语义不一致，IntProvider 侧与 Java nextBetween 恒消费语义相悖。树链不含 trapezoid IntProvider 时无影响（先验低，登记不钻）。

### D3（候选，中）RarityFilter 消费算子不同（nextFloat vs nextInt）
- Java：`random.nextFloat() < 1.0F/chance`（RarityFilterPlacementModifier.java:25-27）——消费 1 次 nextFloat = next(24) = 1 输出，恒定。
- Rust：`random.next_int_bound(*chance) == 0`（placement.rs:297-299）——next(31) + **拒绝采样可再消费 0..n 次**，且分布语义不同。
- 机制：rarity_filter 在链上位于 square 之前时直接改变 square 输入相位；位于其后时改变后续 count 位置相位。需查目标链是否含 rarity_filter（@anchor.idk-3）。

### D4（候选，低-中）同 feature 第 2+ 位置相位被树内消费量放大
- 每个 placed feature 重设种子后，count 的第 2..N 个位置经过前序位置的 configuredFeature.generate（树内消费大量输出）。树内消费总量两侧若差 1（如 D6 旁证：柱 ok 计数口径差 C-R1 未证伪，j5-trace-verdict §5），后续位置 (x,z) 全错。此点与 .b1/树内候选交界——本文只登记机制，不归本候选裁决。

### D5（低，登记）NoiseBasedCount noise 恒 0 占位
- placement.rs:379-384 `let noise = 0.0`（Phase 3 简化）——count 值系统性偏小（消费次数仍 1 次，纯值差非相位差）。仅当树链含 noise_based_count 才相关（先验低）。

### 排除清单（静态已核，不留死记）
- ❌ populationSeed 派生差（A1/A2 对齐，含负坐标 chunk：wrapping 已处理，chunkrandom.rs:168 注释 260906-04 修复）。
- ❌ xoroshiro 引擎本体差（A3 对齐）。
- ❌ decoratorSeed 公式差（A6 对齐；若实测 dseed 差 → 指向 p/k 输入差 = .b1 域）。
- ❌ next(bits) `>>>` vs `>>` 差（A4 等价推演）。
- ❌ 相位跨 feature 累积差（每 feature 重设种子，Java:402）。

## 3. 判别探针设计（主会话可执行；格式两侧对齐）

**P0（前置，seed 三查）**：两侧行首固定 `seed=<worldSeed>`；对比脚本先断言两侧 `seed=8576294172403134396` 且 chunk=(29,-16)，不一致即废。

**P1 populationSeed 对拍**
- Rust（env 门控 `WG_J5B2=1`，chunk 级一次，热路径零成本）：worldgen_handle.rs:1011 后打
  `[J5B2] pop seed=<seed> chunk=(cx,cz) pop=<population_seed>`
- Java（mixin @Inject HEAD/RETURN 于 ChunkGenerator.generateFeatures，或等价：在 :344 后打）
  `[J5B2] pop seed=<world.getSeed()> chunk=(<cx>,<cz>) pop=<l>`
- 判据：pop 不等 → 派生链实现差（推翻 A1/A2，本候选升级）；相等 → 进 P2。

**P2 decoratorSeed + 引擎首值对拍（判别 D1/D2/D3 的关键）**
- 两侧在每次 placedFeature 种子设定处（Java :402 后 / Rust worldgen_handle.rs:1032 后）打印，且**从克隆的随机流取首 4 个输出**（不消费真实流）：
  `[J5B2] deco chunk=(cx,cz) k=<k> p=<p> fid=<id> dseed=<l+p+10000k> next0..3=<n0,n1,n2,n3>`
  - Rust：`let mut probe = feat_random.clone(); probe.next_long(); probe.next_int_bound(16); ...`（clone 后消费）。
  - Java：同构——用 `new ChunkRandom(new Xoroshiro128PlusPlusRandom(dseed))` 局部实例取值（注意不要直接 `nextLong()` 在真实流上）。
- 判据矩阵：
  | 现象 | 结论 |
  |---|---|
  | dseed 同、next0..3 不同 | RNG 引擎/播种差（推翻 A2/A3/A6，罕见） |
  | dseed 不同、pop 相同 | (p,k) 映射差 → **移交 .b1** |
  | 目标 mega 的 (p,k) 行 Rust 缺失 | feature 未进入放置集（.b1/Fix-1 域） |
  | dseed、next 全同 | 派生与引擎无差，进 P3 |
- `next0..3` 选取注意：树链首消费一般是 count（uniform，next(31) 型）；`next0=next_long()`、`next1..3=next_int_bound(16)` 足以暴露引擎级差；若两侧 JSON 链一致，首值必逐位相同。

**P3 位置消费逐点对拍（判别 D1/D3/D4）**
- Rust：placement.rs 已有 `treediag_enabled()`（:283-286）+ `[SQ]`（:303）/`[CNT]`（:294）打点，env `WG_TREEDIAG=1`；需补在 Square 打点行扩为 `[SQ] fid=<fid> k= p= i=<第几个位置> dx= dz= x= z=`（或新 env `WG_J5B2` 独立打，避免混入旧口径）。
- Java：mixin 于 SquarePlacementModifier.getPositions RETURN（capture i,j——无 locals 亦可从参数+返回值拼出 dx=i-pos.getX()），打
  `[J5B2] sq fid=<placeId 经 context 或栈顶推断，缺省空> i=<流内序号，用 sampleCount 替代：chunkRandom.getSampleCount()> dx= dz= x=<i> z=<j>`
  （教训复用：不使用 @LocalCapture，走 RETURN capture 返回值 + ChunkRandom.getSampleCount() 当相位计，规避 260906-01 E9 风险。）
- 判据：同 (k,p) 下首个 [SQ] 的 dx/dz 不等且 next0..3 全同 → **消费序/算子差坐实（D1 或 D3）**，再按该 feature JSON 链二分定位到具体 modifier；首个 [SQ] 相等而第 2 个不等 → D4（树内消费量差，转树内候选）。

**P4（廉价独立验证，≤ 一轮）**：对目标 chunk 单独跑 P1——若 pop 相等即可先行排除「populationSeed 派生差」这一整层，再决定是否值得进 P2/P3（交接结论廉价验证纪律）。

## 4. 诚实声明

- 本候选全文 **Degraded（静态）**：所有「对齐」结论仅为静态逐行核对，未经任何运行时对拍；A4 等价性为位级推演。
- @anchor.idk-1：`ChunkRandom.next(bits)` Java `>>>` vs Rust `>>` 在 bits∈[1,32] 的逐位等价是推演结论，未做向量级测试（可并入 P2 next0..3 顺带验证）。
- @anchor.idk-2：目标 chunk 参与放置的 mega jungle placed feature 完整 JSON 链（count/spread 是否退化 uniform、是否含 rarity_filter/random_offset）未核。
- @anchor.idk-3：rarity_filter 在目标链中的存在性与位置未核。
- 状态机：本文件 draft；升 candidate 需 P2/P3 数据层证据；confirmed 归人类。

## 5. 错误→教训（本分析过程，五段式，供台账挑拣）

1. **「每个 placed feature 重设种子」这一结构事实险些被漏看**
   - 现象：初版分析计划按「整 chunk 一条装饰流，消费差跨 feature 累积」展开。
   - 根因：ChunkGenerator.java:402 的 per-feature `setDecoratorSeed(l,p,k)` 在分层阅读中易被 :344 的 populationSeed 掩盖。
   - 定位：grep setDecoratorSeed 全部调用点 + 读 :360-414 循环结构。
   - 修复：候选空间从「全局流相位」收窄为「(l,p,k) 输入 + 单 feature 内消费序」，并把 dseed 差显式划给 .b1。
   - 教训：相位类问题先画「种子重置点图」再谈消费序——重置点是相位的自然分段。
2. **同名概念两套 RNG 消费语义并存于 Rust 侧**（HeightProvider vs IntProvider 退化边界不一致，D2/D1 对照）——教训：复刻侧对「边界退化（min==max）」要建统一判据表，逐 API 对 Java 恒消费语义核一遍，不是逐点随手写。

## 6. 自检清单（SUBAGENT-KNOWLEDGE-GUIDE §四）

- [x] 价值门：差异点清单/探针设计=高价值（可复用判据：种子重置点分段法、退化边界恒消费判据）；本文载体 = .investigations（过程性候选文档，非结论性 docs）
- [x] 每个差异点带置信度 + 双边行号证据或显式 idk
- [x] 排除假说留 ❌ 清单（§2 尾）
- [x] 降级声明 + 无编造数字（本文无运行时数值）
- [x] 不越界：p/k 计数语义归 .b1，仅登记移交条件
