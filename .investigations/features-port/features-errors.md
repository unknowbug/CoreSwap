# FEATURES 阶段 Rust 移植错误台账

> 课题：Rust FEATURES 阶段（装饰层）移植（C++ feature.h/placement.h/feature_loader.h → WorldgenRust/src）
> 提交：6934ea4 feat(rust): port FEATURES stage
> 载体：`.investigations/features-port/features-errors.md`（项目级错误台账，五段式）
> 价值门：全部高价值（Rust 借用/闭包/索引语义坑），必记
> 自检：每条约五段式（现象/根因/定位/修复/教训）；末尾速查表已同步

---

## 错误列表

### F-1 OreFeatureContext 持有 `&mut random` 的双重借用（E0525 / cannot borrow as immutable）

**现象：**
编译报错，形如：
```
error[E0502]: cannot borrow `*random` as immutable because it is also borrowed as mutable
   --> feature.rs
     | OreFeatureContext 持有 &mut random
     | PlacedFeature.generate 也持有 &mut random 遍历 modifiers
```
即 OreFeatureContext 想缓存一个 `&mut ChunkRandom` 引用，同时 `PlacedFeature.generate` 又要用同一个 `&mut random` 去遍历 modifiers 链——同一随机源的两次可变借用冲突。

**根因：**
机制层面——`&mut T` 在同一作用域内只能活一个活跃借用。OreFeature.generate（Java `OreFeature.generate`）内部要消费随机数（矿脉 3D 形状/位移），而调度层 `PlacedFeature.generate` 也要消费随机数（PlacementModifier 链）。设计上想让 OreFeatureContext 把 random 引用"随身带着"，就必然与调用方传入的 `&mut random` 形成双重可变借用。这是 Rust 所有权模型下"共享可变的生成状态"的典型冲突——**一个会被多个层消费的随机源不能作为引用被结构体持有**，Java 用共享可变对象（Random 是类实例字段）没这个问题，Rust 直接搬结构会撞所有权。

**定位：**
rustc 编译器直接报 E0502/E0525，定位到 OreFeatureContext 的字段持有 `&mut ChunkRandom`。诊断方法：看报错指出的两个借用点，确认是「同源两次 `&mut`」而非「不可变/可变混用」。

**修复：**
把 random **从结构体字段中移除**，改为**函数参数传递**——`OreFeature::generate(random: &mut ChunkRandom, ...)` / `ScatteredOreFeature::generate(... random: &mut ChunkRandom ...)`。OreFeatureContext 只保存静态配置（targets/size/state），随机数由调用方按需传入，每次调用借用一个 fresh 的 `&mut`，避免长期持引用。这样各 generate 函数签名统一为「context + &mut random + 坐标」，借用在函数边界即释放。

**教训：**
- **跨层共享的可变状态（随机源、累加器）在 Rust 里不要设计成被结构体持有引用，优先用参数传递**，尤其在「多个嵌套生成器都要消费同源随机数」的场景。
- Java「可变对象作字段共享」→ Rust 移植的第一排查点：凡 Java 把 Random/Mutable 状态放字段，Rust 大概率要改成参数化或 RefCell/Cell 包装。
- 可复用判错经验：**Rust 编译期借用冲突是结构设计问题，先想「谁拥有可变状态、借用在哪个边界释放」，别靠 unsafe 强解**。

---

### F-2 PlacedFeature.generate 嵌套 fn 无法捕获 FnMut（E0434/E0575，闭包递归 + Cell 变通）

**现象：**
`PlacedFeature.generate` 要模拟 Java Stream 惰性深度优先 flatMap（每个位置走完所有 modifier 再下一个），用递归实现。直接写**内部嵌套 `fn`（不是闭包）**时编译报错，因为嵌套 `fn` 无法捕获外层捕获了 `FnMut` 的 `generate_configured` 回调，也无法捕获 `&mut random` 的借用环境。

**根因：**
机制层面——Rust 里 `fn item`（嵌套函数）**不捕获环境**，它只有显式传入的参数，不能引用外部局部变量。而这里需要的递归结构要么闭包递归（但 `FnMut` 闭包捕获后又要被递归可变借用，同样冲突）,要么把遍历状态显式传入。另外 `generate_configured` 是 `FnMut`（调用方在遍历中不断调用），递归每一层都要可变借用它，直接裸递归会撞借用规则。**嵌套 fn 方案在两处同时失效**：捕获 `FnMut`（语法不允许）+ 传递可变借用（借用规则不允许在同一递归路径上多个活跃 `&mut`）。

**定位：**
rustc 报 E0434（嵌套函数里引用了外部变量）/E0575 类错误，指向 `visit` 内部函数体引用了 `generate_configured` 和 `random`。诊断方法：把"递归函数"改成"显式参数化"后，报错转移到借用冲突，才意识到真正的难点是 `FnMut` 的可变传递。

**修复：**
改成**闭包递归 + Cell 存 placed 标志**（placement.rs L293-309）：
- 外层用 `Cell<bool>` 存「是否已放置」标志；
- 递归函数 `visit` 仍写成嵌套 fn，但把 `generate_configured: &mut F`、`random: &mut ChunkRandom`、`placed: &Cell<bool>` **全部作为显式参数传入**——这样嵌套 fn 不捕获环境，借用也是逐层传引用；
- `generated_configured` 只在最深一层（`mi == modifiers.len()`）调用一次，调用顺序天然保持「位置逐个 depth-first」，且每层递归结束时借用即释放，不产生同路径多重可变借用。

**教训：**
- **深度优先的惰性消费用「嵌套 fn + 全显式参数」比闭包递归干净**——闭包递归捕获 FnMut 会二次撞借用，显式参数化一箭双雕（不捕获 + 借用边界清晰）。
- **「递归里要跨层传递可变状态」的通用形态：状态（placed 标志）用 Cell/RefCell 共享，可调用回调用 `&mut` 显式参数逐层传**。
- 惰性 flatMap 语义（深度优先 vs 广度优先）会改变**随机数消费顺序**，进而改变所有依赖随机的输出——移植 stream flatMap 必须先想清楚遍历顺序（详见本课题结论 docs 与 placement.rs L286-288 注释）。

---

### F-3 PlacedFeatureIndexer 从单 biome features 构建导致 p 值错（lastIndex 索引错位）

**现象：**
验证对拍时 ore 等 feature 生成位置大面积错位；排查发现 `PlacedFeature.generate` 里 `setDecoratorSeed(l, p, k)` 的 `p` 值（`global_index`）不对。feature 的随机序列全偏（p 是随机种子的输入分量），导致放的位置全错。

**根因：**
机制层面——Java `PlacedFeatureIndexer` 的 `featureIndex` 和 `indexMapping`（`lastIndexGetter`）是**从所有 biome 的 features 集合**构建的（ChunkGenerator.generateFeatures 遍历 biomes 的 Object2IntMap computeIfAbsent + Util.lastIndexGetter map.put 覆盖）。`p` = feature 在 `features[step]` 列表里的 **lastIndex**（Java `indexMapping`），**不是 featureIndex 全局首现编号**。若只从单一 biome 构建 indexer，则 (a) featureIndex 编号与全局不一致，(b) lastIndex 依赖 step_features 顺序，单 biome 的子集会让 feature 的 p 索引整体错位，而 setDecoratorSeed 的 `l = populationSeed + index + 10000 * step`、`p` 直接进种子混合，p 错则后续所有随机序列（size/位置/形状）全错。

**定位：**
features_probe 对拍 vanilla 时 ore 位置 1/13 匹配、p 值相关的随机序列错位 → 回溯 indexer 构建入参，确认 build() 喂入的是「当前 biome features」而非「全部 biomes features」。诊断方法：对比单 biome vs 全 biome 构建时同一 feature 的 index/lastIndex 是否一致，不一致即 indexer 入参错。

**修复：**
`PlacedFeatureIndexer.build()` 改从 **`all_features_lists()`（所有 biome 的所有 step 的 features）** 构建（feature_loader.rs L79、worldgen_handle.rs L153 `bc.all_features_lists()`），保证 featureIndex 全局递增、lastIndex 全局最后出现索引，与 Java computeIfAbsent + lastIndexGetter 语义一致。

**教训：**
- **全局索引型数据结构（PlacedFeatureIndexer）必须以全量数据构建**，不能只从「当前处理的子集」构建——索引编号/ lastIndex 依赖遍历顺序，子集会让所有下游随机序列错位。
- **seed 分量错位 = 随机输出大面积错位但「形状看起来对」**：p/step 是 setDecoratorSeed 的种子输入，错了不会报错，只是位置/数量全偏（对比对齐率会略降，需对拍精确定位）。
- 可复用判错经验："位置对但数量/分布偏"或"对齐率小幅下降"先查 **随机种子分量（index/p/step）** 是否与参照一致，再看公式精度。

---

## 错误 → 根因 → 判错经验 速查表

| # | 错误 | 根因 | 可复用判错经验 |
|---|---|---|---|
| F-1 | OreFeatureContext 持有 `&mut random` 双重借用 | 跨层共享随机源不能作为结构体引用持有 | Rust 借用冲突=结构设计问题：先想可变状态归属与借用边界，Java 字段可变对象大概率要参数化 |
| F-2 | 嵌套 fn 无法捕获 FnMut / 闭包递归二次撞借用 | 递归惰性扁平化需显式参数化 + Cell 存状态 | 深度优先惰性消费用「嵌套 fn+全显式参数」；惰性遍历顺序会改变随机消费顺序 |
| F-3 | Indexer 单 biome 构建 p 值错 | lastIndex/index 依赖全量构建顺序 | 位置对但分布偏/对齐率微降 → 先查随机种子分量（index/p/step）；全局索引须全量构建 |
