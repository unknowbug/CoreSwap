# .b1 surface_depth(runDepth) 机制分析 —— nether basalt_deltas 三大宗石互换

- **status**: draft（已含数据层证据复核；见 §7 —— **verdict 已更新：机制不成立**）
- **candidate**: `.b1` — Java getSurfaceDepth/sampleRunDepth 2D 噪声不一致 → stone_depth 带厚差 → netherrack↔basalt/blackstone 成片互换（≈28k 块，seed B=8576294172403134396）
- **载体声明**（§9.7）：首轮静态审查（Degraded）；本轮新增数据层证据 = b1_column_trace 4 列逐层 id 对比（`.investigations/nether-save-full/cmd-output/b1-column-trace.txt`）+ 数据包 feature/configured_feature JSON 核对

---

## §7【本轮更新 · 取代 §6 结论】verdict：机制**不成立**（surface_depth 带厚差不是 28k 子族的主驱动）

### 现象（数据层证据，b1-column-trace.txt，4 列 chunk(200,200)）

错位不是「薄带 ±1 层」签名，而是**整段大宗块体差**：
1. 列(7,0) y=68..107：vanilla=basalt（约 40 层连续）+ blackstone 夹心（y=81-86）/ rust=**netherrack 全差**；
2. 列(11,1) y=12-16、19-23：vanilla=blackstone 薄夹层（上下皆 netherrack，附近无 air/流体）；
3. 列(12,2) y=10-15 blackstone、列(0,2) y=2 blackstone、y=34-36 blackstone——同为无暴露面的内部夹层；
4. 列(7,0)/(12,2) y=46-58：vanilla=basalt / rust=**air**（vanilla 该区实心本身是后装进去的）；
5. y=27-30 rust=magma_block（607）vanilla 无；列(0,2) y=33-36 vanilla=blackstone rust=magma——magma/岩浆湖相关错位。

### 根因（机制层）

**带厚差机制可解释的最大带厚 = 1+runDepth ≤ 6 层，且只能出现在暴露面（floor/ceiling）附近**。逐点检验：

- 对上述所有内部夹层（黑曜石/玄武岩被 7+ 层 netherrack 上下包裹、整列无 air/流体断点，如列(11,1) 全列 y=0..127 连续实心）：q（stone_depth_above）≥ 100、vx（stone_depth_below）≥ 8，floor/ceiling 两个 stone_depth 条件**均不可达**；
- 40 层连续 basalt（列(7,0) y=68-107）绝无可能由任何 runDepth∈[0,5] 的带厚产生；
- 若 runDepth 两侧差 ±1，预测签名 = 暴露面处 ±1 层错位——实测非此签名（列(7,0) y=68 rust=basalt（ceiling 带 1 层，说明 rust runDepth=0）而 y=69-107 整段差）。

**真正机制（本轮定位）**：vanilla basalt_deltas 的宗石大宗来自 **FEATURE 阶段的 nether 专属 feature 类型**，Rust feature 管线未实现：
- `basalt_blobs` / `blackstone_blobs`（biome features step 6 underground_ores）= **`minecraft:netherrack_replace_blobs`**：以 uniform(3..7) 半径的 blob 把 netherrack 整块替换为 basalt/blackstone——内部夹层 + 大宗块体的直接来源（configured_feature/basalt_blobs.json、blackstone_blobs.json 实证）；
- `large_basalt_columns` / `small_basalt_columns` = `minecraft:basalt_columns`、`delta` = `minecraft:delta_feature`（熔岩湖 + magma/basalt 边缘）、`basalt_pillar`——解释 y=46-58 vanilla 实心/rust air（vanilla 该处是 feature 填充的 delta/柱体）与 magma 错位（delta 的 magma 边缘 rust 未实现，rust 的 magma 来自已实现的 ore_magma/underwater_magma，位置随地形/特征差漂移）。

surface_rule 本体（nether.json L105-736）核对完毕：**除薄带外无任何产大宗玄武岩/黑曜石的分支**（basalt_deltas 分支只有 ceiling/floor 两个 stone_depth 条件 + 最终 fallback netherrack）——排除了「规则解析漏分支」。

### 判定与取代记录

- **.b1 surface_depth 带厚机制：不成立**（❌ 取代 §6「机制成立（候选）」；supersedes 指针 → 本节；§1-6 保留不改）。
- §1 的静态对拍结论仍有效且有价值：runDepth/StoneDepth 引擎语义两侧一致（非根因），后续无需再查此链。
- **新候选（建议立 .b1' 或并入 feature 课题）**：Rust 缺失 nether feature 类型 `netherrack_replace_blobs` / `basalt_columns` / `delta_feature` / `basalt_pillar` 的实现——B1 大类（52,078 块）的主体应归此。

### 下一轮探针/工程建议（写给主会话）

1. **归因确认（廉价）**：Rust 侧 `cargo run --release --bin b1_column_trace`（已有）+ 临时 `WG_SKIP_FEATURES=1` 双跑——预期 rust magma（607）消失（确认 rust 的 magma 全来自 feature）；vanilla 侧残差（basalt/blackstone 宗石）在 rust 关 features 后不变化（它们本来就缺）。
2. **blob 形状对拍**：实现 `netherrack_replace_blobs` 前先写 bin-diag 探针：读 placed_feature/basalt_blobs.json（placement/count/rarity + InSquareHeightmap）+ configured_feature radius uniform(3,7)，对 chunk(200,200) 邻域模拟 vanilla blob 中心，与参照 blocks 的 basalt 宗石形状对拍（半径 3-7 的球状/椭圆替换 netherrack）。BlobFeature 的随机序列（count → 每次位置 + radius roll）用 legacy LCG splitter——注意 `split()` 字符串派生走 LCG hashCode 路径。
3. **实现范围清单**（数据已核对）：`netherrack_replace_blobs`（target=netherrack, state=basalt/blackstone, radius uniform 3..7）、`basalt_columns`（large/small 两配置）、`delta_feature`（block=magma/rim, contents=lava）、`basalt_pillar`；placed_feature 参数从 data 目录 JSON 读（数据驱动纪律）。
4. B1 台账修正建议：09 篇 L192 B1 行的「surface rule 条件链系统性偏差」候选应改指向 feature 缺失（此为 worker 草稿意见，改 docs 走知识库更新流程）。

---

## §8【补充复核 · 主会话全量分类数据】固体↔固体 98.3% 口径下逐层带预测检验 —— 维持「机制不成立」

数据：b1-family-split.txt——B1 残差 solid↔solid 66844 (98.3%)、van_solid_rust_air 584 (0.9%)、van_air_rust_solid 566 (0.8%)。固体↔固体压倒性 → 按 main 会话要求聚焦固体↔固体层位做 floor/ceiling 带位置预测 vs 实测。

### 逐层带可达性检验（列(7,0)，预规则地形 y=0..6 连续 default、s=0）

- **y=1,3-6（底部 ceiling 带）**：条件 vx=wy+1 ≤ 1+runDepth。y=6 需 runDepth≥6 > 理论上限 5（(int)(d·2.75+3+0.25)，d≤1）→ **vanilla y=6 basalt 对任意 runDepth 均不可由带产生**。y=1 rust=netherrack 确证 rust runDepth=0，vanilla 即便取上限 5 也只覆盖到 y=5。
- **y=69-107（大块体内部，39 层）**：位于 y=59-67 air 段之上的连续实心 68..124。floor 带 q=1..40 只覆盖 y≤68+runDepth≤73；ceiling 带 vx≤7 只到 y≤74。**39 层中至多 7 层带可达，其余 32 层对任意 runDepth 不可达**。且 y=68 两侧相等（rust ceiling 带 1 层 → rust runDepth=0），vanilla 同列延续到 107——同列 runDepth 不可能既 0 又 39。
- **列(11,1) y=12-16/19-23、列(12,2) y=10-15、列(0,2) y=2/34-36**：全列无 air/流体断点（熔岩除外），q≥100、floor 带不可达；ceiling 带 vx=wy+1≤6 仅覆盖 y≤5——y=10..36 全部不可达。

### 判定

固体↔固体聚焦口径下否证**加固**：带机制签名应为「错位集中于暴露面 1..6 层内、深层完美匹配」，实测深层恰是大头（98.3% 的主体层带不可达，且 y=6 超出 runDepth 数学上限）。98.3% 固体↔固体恰为 `netherrack_replace_blobs` 精确签名（blob 替换两侧皆固体）。y=46-58 形态差 ~1.2% 按主会话指示不在 .b1 范围解释。

### 可判别预测（下一轮廉价验证）

若主体是 blob：vanilla basalt/blackstone 错位区在水平方向呈**半径 ≤7 的团块**（随机游走中心连通）；若是带差：错位层在每个暴露面处严格 ≤6 层且跨列等厚连续。`WG_B1_COLS` 增采 chunk(200,200) 相邻 4-8 列即可判别（b1_column_trace 已支持 env 传列）。

---

## §1-6 首轮分析（保留存档，结论已被 §7 取代）

### 1. Rust 侧实现核对（代码事实，首轮静态层——仍有效）

| 项 | Rust 实现 | 位置 | 对齐判定 |
|---|---|---|---|
| 噪声 key | `minecraft:surface` | surface_rules.rs L440 | ✓ |
| 噪声参数 | firstOctave -6, amplitudes [1.0,1.0,1.0] | density_builder.rs L57 | ✓（Java NoiseConfig 特判值） |
| 坐标输入 | 原始块坐标无 scale | L440 | ✓ |
| 公式 | `(d*2.75+3.0+split_xyz(x,0,z).nextDouble()*0.25) as i32` | L440-442 | ✓（06 篇已验证） |
| nether legacy | 噪声种子派生 + split 全走 LCG | legacy_random.rs | ⚠️（本轮未再涉及，非根因） |
| StoneDepthCond | `i <= 1+offset+j+k`，(int) 截断 | L84-93 | ✓ |
| nether 规则来源 | settings.surface_rule JSON 数据驱动 | worldgen_handle.rs L222-229 | ✓ |
| 引擎列扫描 | q/vx/r/s 语义对齐 06 篇 | L1131-1181 | ✓ |

### 2. nether.json basalt_deltas 分支结构（仍有效）

ceiling→basalt 无条件；floor→patch/y_above→gravel、nether_state_selector≥0→basalt、否则 blackstone；带外 netherrack。带厚 = 1+runDepth ∈ [1..6]。

### 3. 首轮 3 样本自洽性检验（历史推理，已被 §7 数据否定其归属层）

首轮将双向带翻转归因于 runDepth 逐列漂移——该推理在「错位只发生在带可达层」前提下自洽，但 b1-column-trace 显示错位大量出现在**带不可达的内部层**，机制归属不成立。

### 4-6. 知识库关系 / 探针建议 / 自检

- §4 知识库关系仍有效；§5 的 runDepth dump 探针**降级为可选**（静态层已对齐 + 机制归属已否证，不必优先）；§6 的自检清单中「未验证假设 ①②③」本轮已被数据部分覆盖（biome 两侧一致由 vanilla basalt 列证明；q 语义差异已由大宗块体证据压倒）。
