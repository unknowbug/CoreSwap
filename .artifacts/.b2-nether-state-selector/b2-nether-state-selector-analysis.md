# .b2 — minecraft:nether_state_selector 噪声采样不一致（basalt ↔ blackstone 成片互换）

- **status**: draft（AI 绝不自授 candidate/confirmed；Degraded = 纯静态审查 + 主会话回传的列级 trace 解读）
- **retry 轮次**: 1（R1 = 主会话采到新数据层证据 b1-column-trace.txt，本节 §7 为该轮解读——新数据层证据，saturation 计数重置）
- **自检声明**: 所有行号/代码引用均来自本 session 实读文件；未编造数值；双向证据矛盾点显式标注（§4）

---

## 1. 数据链事实（静态实读）

### 1.1 vanilla 侧（nether.json surface_rule）

`versions/1.20.1/data/worldgen/data/minecraft/worldgen/noise_settings/nether.json`：
- **L249-271（basalt_deltas floor 带）**：`stone_depth(floor, add_surface_depth=true)` 内 sequence：
  1. patch 噪声 [-0.012, +inf] × y_above(30..35) → gravel
  2. `noise_threshold(nether_state_selector, min=0.0, max=+inf)` → **basalt(axis=y)**
  3. 兜底 → **blackstone**
- **L300-322（soul_sand_valley ceiling 带）+ L334-397（floor 带）**：同一 `nether_state_selector` 阈值 min=0.0 → **soul_sand**，兜底 **soul_soil**（同一噪声同时决定 SSV 双子族！）
- **关键**：`nether_state_selector` 全仓只出现 3 次（L254/307/383），**全部在 surface_rule**，不在 noise_router / density_function/nether/ 任何文件（grep versions/1.20.1/data/.../worldgen 全目录仅 3 命中）。
- 参数（`worldgen/noise/noise/nether_state_selector.json`）：`firstOctave=-4, amplitudes=[1.0]`（单 octave，amplitude 1.0）。nether `legacy_random_source=true` → 走 LegacyRandom 派生族。

### 1.2 Rust 侧求值路径

`WorldgenRust/src/surface_rules.rs`：
- `SurfaceCond::NoiseThreshold`（L94-98）：`d >= min_th && d <= max_th`——判定语义正确。
- `noise_threshold_sample`（L120-137）：
  ```rust
  ctx.noise_samplers.get(noise_key)
      .map(|n| n.sample(x, 0.0, z))
      .unwrap_or(0.0)          // ← L129-133：查不到 sampler 恒回退 0.0
  ```
  y=0.0 二维采样、列缓存——与 Java MaterialRuleContext 的 surface 噪声 (x, 0, z) 语义一致，这部分无偏差。
- 规则来源：`worldgen_handle.rs L220-229`——非 overworld 走 `parse_surface_rule`（JSON 数据驱动），noise_key 解析为完整 `"minecraft:nether_state_selector"`（L1020-1025），解析无命名空间截断。

### 1.3 sampler 构建（缺失点）

`worldgen_handle.rs create_for_dim` **step4（L192-197）surface 用 sampler 预加载表**：
```rust
for key in ["minecraft:surface", "minecraft:surface_secondary", "minecraft:clay_bands_offset",
            "minecraft:badlands_surface", "minecraft:badlands_pillar", "minecraft:calcite",
            "minecraft:gravel", "minecraft:powder_snow", "minecraft:packed_ice", "minecraft:ice",
            "minecraft:surface_swamp"] { ... }
```
——**不含 `minecraft:nether_state_selector`**（也不含 nether 用到的 `patch`/`soul_sand_layer`/`gravel_layer`/`netherrack`/`nether_wart`，见 §4.3 连带影响）。此表是 overworld 硬编码表，多世界改造时未泛化。

sampler 的另一创建路径是 density 树构建时经 `get_noise_sampler_from_obj`（density_builder.rs L181-184）——但 §1.1 已证 density 树不引用该噪声，故**该 sampler 在 Rust 进程内从未被创建**。

`DensityBuilder::get_noise_sampler`（L152-178）本身具备正确构建能力：legacy 路径 `random_deriver.split_str(key)`（LegacyRandom split 家族，与 Java RandomState legacy 分支同构）+ noise_params.json 参数（含 nether_state_selector -4/[1.0]）——即**只要预加载表补上它，参数/种子派生链即存在且结构正确**（正确性待探针逐位验证，§5）。

## 2. 机制推演（b2 若成立的错位形态）

- Rust：selector 恒 0.0 → `0.0 >= 0.0` 恒 true → basalt_deltas floor 带内**恒 basalt**，**blackstone 分支在当前代码下不可达**（blackstone 只可能由该兜底写出）。
- vanilla：selector 是真实 Perlin 噪声（-4 octave 单振幅，~十块级斑块），floor 带内 basalt/blackstone ≈ 各半成片。
- ⇒ 预测错位模式：**floor 带内，vanilla=blackstone 的整片区域 save 全为 basalt**（即 vanilla blackstone 斑块系统性消失）；vanilla=basalt 区域两者一致。

### 2.1 floor 带约束判据（任务 2 要求，可检验）

b2（selector 噪声差）成立的**必要条件**：全部错位块满足——
1. **floor 带内**：该点在 vanilla 与 Rust 两侧都命中 `stone_depth(surface_type=floor, add_surface_depth=true)`（basalt_deltas 分支）或 floor/ceiling 带（SSV 分支）。等价表述：错位块所在列，该块正上方第一个非固体块与本块之间的固体深度 ≤ 1 + surface_depth(x,z)（surface_depth = `surface` 噪声×2.75+3.0+rand×0.25 取整）。y=1..3 底层点能否命中 floor 取决于该列「第一个非 default 块」扫描（Rust build_surface L1131-1168 的 s/q 扫描），**高层悬空地板同样命中**——故「出现在高处」不排除 b2。
2. **biome = basalt_deltas**（本子族）或 soul_sand_valley（连带预测，§4.3）。
3. **非 bedrock 带**（y≥5，bedrock_floor vertical_gradient 先于 selector 判定）。
4. **同列同 (x,z) 必然同错位方向**：selector 是 2D 列噪声（y 输入恒 0），同一 (x,z) 整个 floor 带厚度内取值相同 → 错位若由 selector 造成，**同一列内应整段一致**（不会同列 y=1 basalt 错、y=3 blackstone 对，除非两 y 的 floor 带命中不同）。

**反证判据**：发现任一 save=blackstone 的错位块 → 当前 Rust 代码不可达该分支 → b2-as-implemented（恒 0.0）直接证伪（见 §4.2，bucket 数据疑似已有反例）。

## 3. 结论三态

**部分成立 + 证据不足（对原始候选表述修正）**：

- ✅ **机制存在的代码级实证（高置信，Degraded 静态）**：候选方向「selector 噪声采样不一致」命中一个真实缺陷，但**真实形态不是参数/坐标/Perlin 实现偏差，而是 sampler 缺失 → 恒 0.0 → 条件恒 true**。参数加载（noise_params 表含 -4/[1.0]）、坐标（x,0,z 与 Java 一致）、Perlin 实现均无偏差——因为根本没走到。
- ❌ **与双向 bucket 证据矛盾**：恒 0.0 只能产生 **vanilla=blackstone → save=basalt 单向错位**。任务给的两点交叉证据中，`local(0,2) y=2 vanilla=blackstone save=basalt` ✅ 吻合；`local(12,2) y=1 vanilla=basalt save=blackstone` **无法由本机制产生**（Rust 侧 blackstone 分支不可达）。两种可能：① bucket 统计的方向标注/配对有待复核（如 chunk 原点或 local 索引 x/z 序约定差）；② 存在第二机制（如 floor 带命中差异 + 另一写 blackstone 的路径——当前代码内未发现）。**在双向证据复核前，b2 不能整体定论。**
- ⚠️ 修复预期量级参考：若 22k 全部为单向错位，补一行预加载 + 逐位探针验证即可闭合该子族；若确有反向块，须开新候选（建议 .b3 = floor 带命中/stone_depth 扫描差异）。

## 4. 连带发现（同根因，一并修复）

1. **soul_sand_valley 同噪声**：sampler 缺失同样使 SSV floor/ceiling 恒 soul_sand、soul_soil 不可达——若 bucket 有 soul_sand↔soul_soil 子族，其方向应同样纯单向（soul_soil→soul_sand）。这是 b2 的**免费交叉验证点**。
2. **预加载表还缺**：`minecraft:patch`（basalt_deltas/SSV 的 gravel 带 [-0.012,inf]）、`minecraft:soul_sand_layer`（nether_wastes 表面）、`minecraft:netherrack`、`minecraft:nether_wart`、`minecraft:gravel_layer`（warped/crimson nylium 带）——这些 nether surface_rule 用到的噪声全部不在 L192-195 表内，`unwrap_or(0.0)` 使 patch 条件恒 true（gravel 带、soul_sand_layer 条件恒 true）、`not(noise_threshold(netherrack,0.54))` 恒 true 等，可能构成 B1 其余子族（三大宗石互换的另外两族）的独立根因。**建议探针一并 dump。**
3. **防回归模式**（对齐 knowledge/discovered/build-tooling.md as_bool 坑）：「JSON 驱动的条件引用了预加载/注册表外的键 → 静默回退默认值」。`unwrap_or(0.0)` 无任何告警，建议改 `eprintln!("[SURFACE-WARN] missing noise sampler: {key}")` 一次性告警或返回 NaN 短路。

## 5. 下一轮探针命令建议（写给主会话执行，本 worker 不跑）

1. **Rust 侧 selector dump（env 开关设计）**：在 `noise_threshold_sample`（surface_rules.rs L120）加 chunk 级 env 门控 dump（防逐点 env 查询污染，对齐 AGENTS.md 探针污染铁律）：
   - `WG_SELECTOR_DUMP=<x0>,<z0>,<x1>,<z1>`：进程启动时读一次 env；命中列时 `eprintln!("[SELECTOR] {key} {x} {z} {v}")` 输出 `nether_state_selector` 采样值。
   - **前置修复探针**：临时在 create_for_dim step4 表加入 `"minecraft:nether_state_selector","minecraft:patch","minecraft:soul_sand_layer","minecraft:netherrack","minecraft:nether_wart"`，再跑 `nether_blocks` 对比（`multiworld_nether_blocks`，WGB2 4×4@0,0 h256 参照）——若 basalt↔blackstone 子族归零/大降且 soul_soil 同步归零，b2 闭环。
2. **Java 侧参照**：DensityProbe/reflection 取 `RandomState(nether).legacyRandomDeriver.split("minecraft:nether_state_selector")` 的 DoublePerlinNoiseSampler，采 4×4@0,0 floor 带若干列 (x,0,z)，与 Rust dump 对拍（先核对两侧 worldSeed=8576294172403134396，seed 三查纪律）。
3. **方向复核（零成本，先行）**：用既有 bucket 脚本对 22k 互换块统计 (vanilla,save) 方向直方图 + 同列一致性（§2.1 判据 4）+ 错位块与 floor 带命中重合率——若存在 save=blackstone 反向块或错位越出 floor 带，b2-as-implemented 证伪，转 .b3（floor 带命中/stone_depth s/q 扫描差异）。

## 6. 排除清单

- ❌ 参数加载偏差：noise_params.json/hardcode 表均含 nether_state_selector (-4,[1.0])，且 sampler 从未构建，未走到参数。
- ❌ 采样坐标偏差：surface 条件噪声 y=0.0 二维采样与 Java 一致（build-tooling/M17 后 below_top 锚已修，与本候选无关）。
- ❌ y_above `surface_depth_multiplier` 解析缺口（parse_surface_cond L1002 恒 mult=0）：nether.json 内所有 multiplier 均为 0，本课题不触发（记为潜在坑）。
- ⚠️ 未验证：legacy split_str("minecraft:nether_state_selector") 派生值与 Java 逐位一致——待探针 2。

---

## 7. R1 更新（2026-09-XX，b1-column-trace.txt 解读——回答主会话问题）

**数据源**：`.investigations/nether-save-full/cmd-output/b1-column-trace.txt`（b1_column_trace.exe，chunk(200,200) 4 列，van vs rust 逐层 id；ref seed=8576294172403134396 与目标一致，seed 三查通过）。

### 7.1 对主会话问题的直接回答

**selector 噪声差不能解释「rust 完全不涂布」形态——只能解释分支内的二选一翻转。** 机制上：selector 恒 0.0 使 basalt_deltas floor 分支**恒写入 basalt**——前提是分支被进入。分支一旦进入，rust 侧输出必为 259 或 849 之一，**绝不可能落回 netherrack**（写 netherrack = 条件链在 stone_depth/biome 等上游就未命中，sequence 未执行）。

trace 实测形态（4 列全部错位层）：
- rust 侧错位输出 ≈ **100% netherrack(256)**（另有 607 magma / 0 air 若干——属宏观地形差异，非 surface 层）；
- rust 侧全 4 列 **从未写出 849(blackstone)**；仅零星 259：col(7,0) y=27/45、col(11,1) y=26/45、col(12,2) y=27/45、col(0,2) y=31/45/68；
- 其中 col(11,1) y=45：van=849 rust=259 —— 这是**一个真正的分支内翻转样本**（van blackstone→rust basalt），与 §1.3 恒 0.0 预测**方向一致**；col(7,0) y=45、col(12,2) y=45 的 rust=259 同理（van 该点分别 259/259）。
- 错位层 rust 全为 256 → **主导机制 = 分支根本未进入**：stone_depth 带判定（.b1）、biome 判定、或宏观地形差异（rust 在 y=46-58 成段 air、y=28-36 成段 607，说明两侧柱体形状/流体面都不同 → q/s 扫描与 floor 带命中天然错位）。

### 7.2 修正后的三态

- **b2 降级为次要贡献者（局部成立）**：恒 0.0 缺陷真实存在（代码级实证不变），且 trace 给出 1+ 个方向吻合的分支内样本、0 个反方向样本（rust 无一处 849，反而强化「blackstone 分支不可达」）；但它只解释已进入分支点位的「恒 basalt」，解释不了 22k 错位中的主体形态。
- **主导机制指向 .b1**：分支未进入的直接候选 = ① stone_depth 带判定差（q/s 扫描语义或 surface_depth 输入差）；② biome 差（rust 在此区域未判成 basalt_deltas → 落 nether_wastes 等分支；注意 nether_wastes 的 soul_sand_layer sampler 同样缺失恒 0.0 → 条件恒 true，但其后续 y_above(30..35) 不命中深层 → 深层落回不写=netherrack，**与 rust 全 256 的形态同样自洽**）；③ 宏观地形差（rust air/magma 段）导致 floor 带错位——此项在 surface 之前的 fill 阶段，可能是更大的上游根因。
- **鉴别要点**：vanilla 侧 849/259 出现在被 bedrock/流体切开的多个「伪表层」上（van bedrock 不止 y=0，y=2 也有 31），每次非 default 块（bedrock/lava）都重置 stone_depth 扫描形成新 floor 带——rust 侧若 bedrock 分布或 lava 面位置与 van 有差，floor 带整段错位即大面积「不涂布」。

### 7.3 探针精确规格（主会话实现执行）

**P-A（分支入口/stone_depth 追踪，最高优先）**：在 `surface_rules.rs build_surface` 列循环内加 chunk 级 env 门控：
- env：`WG_STRACE="bx,bz"`（block 坐标列，进程启动读一次，不逐点查 env）；
- 命中列时对每个 `state==default_block` 的实心点输出一行：
  `[STRACE] x y z biome=<biome_id> q=<stone_depth_above> vx=<stone_depth_below> s=<s> sd=<surface_depth> fluid=<r> paint=<rule.apply结果或-1>`
- 同时在 `parse_surface_cond` 的 `noise_threshold` 分支记录本 rule 树引用了哪些 noise_key、 sampler 命中与否（构建期一次性打印 `[SNOISE] key=... present=<bool>`）。
- 判读：错位层若 `biome≠basalt_deltas` → biome 候选；若 biome 对但 `q > 1+offset+sd` → stone_depth 带候选（.b1）；若 q 在带内但 paint=-1 → 上游条件（y_above/hole）候选。

**P-B（selector/缺失噪声值 dump）**：先临时把 `minecraft:nether_state_selector, patch, soul_sand_layer, netherrack, nether_wart, gravel_layer` 加入 `create_for_dim` step4 预加载表，再在 `noise_threshold_sample` 里 `WG_SELECTOR_DUMP="bx,bz"` 门控输出 `[SELECTOR] key x z v`。对照 P-A 同列跑一次。
- 判读：重跑 b1_column_trace 后若「rust 全 256」层位不变 → 确认主导机制不在 selector（预期如此）；若 col(11,1) y=45 类分支内翻转消失 → 恒 0.0 修复生效，b2 子族闭合。

**P-C（biome 直查，低成本先行）**：b1_column_trace 同款 4 列直接打印 `biome_at(m, wy, n)`（每 4 层一次即可）——若 rust 判成 nether_wastes/crimson_forest，.b1/stone_depth 全线排除，直接转 biome 候选。

**P-D（宏观地形差隔离）**：对 4 列输出 `fill_chunk` 阶段原始 BlockKind（air/rock/water/lava）逐层对比（surface 前快照），量化「rust air/magma 段」占比——占比高则先修 fill 阶段（这可能同时是 607/0 段与 floor 带错位的共同上游）。

---

## 8. R2 补充（全量残差形态分类 b1-family-split.txt：固体↔固体 98.3% / 空气↔固体 1.2%）——机制签名匹配度

### 8.1 b2 机制的预期形态签名（机制定义严格推出）

若 22k 错位**由 selector 噪声差主导**（无论「恒 0.0」还是「参数/坐标偏差」），则必须同时满足：
1. **两侧都已进入 basalt_deltas floor 分支**（stone_depth 带命中、biome=basalt_deltas）——否则 selector 根本不被求值；
2. 错位块 ∈ {259, 849} 且**双向**（真噪声差）或**单向 van=849→rust=259**（恒 0.0 形态）；rust 侧在该子族内**不出现 256**；
3. 残差应表现为「**涂布颜色差**」，故 parent 分类必然落在固体↔固体（涂了但颜色不同）；
4. 同列整段同方向（selector 为 2D 列噪声）。

### 8.2 实测形态 vs 签名逐条比对

| 签名 | 实测（b1-column-trace + b1-family-split） | 匹配 |
|---|---|---|
| ① 两侧均进分支 | 错位层 rust ≈100% 涂 256（netherrack）→ rust 侧根本未进分支 | ❌ |
| ② 错位 ∈ {259,849} 双向/单向 | rust 在这些层位涂 256，不是 259/849；仅零星点（y=45 附近）符合 | ❌（主体）/ 局部✅ |
| ③ 固体↔固体（涂布差） | 98.3% 固体↔固体——**这一点表面兼容**，但列 trace 显示其中 rust 涂的是 256（未涂布的 default 块），不是「涂布颜色不同」 | ⚠️ 表面兼容、实质不符 |
| ④ 同列同方向 | 未系统统计（待 P-A 后可验） | 未测 |

**关键澄清（固体↔固体 98.3% 不构成对 b2 的支持）**：固体↔固体只说明两侧都非空气，**不区分「进了分支但颜色不同」与「未进分支落在 default(netherrack)」**——netherrack 是固体，未涂布也计入固体↔固体。列 trace 直接证伪了「颜色差」解释：rust 侧颜色几乎全是 256。

### 8.3 匹配度结论

**b2（selector 噪声差）与实测主导形态匹配度 = 低（仅解释零星分支内点位）**：
- 恒 0.0 形态可以解释的残差上限 = trace 中「rust 写 259 且 van 写 849」类点位（4 列中约 1 处/列量级）——占 22k 主体的小部分；
- 22k 主导形态「rust 涂 256 / van 涂 259|849」= **分支未进入**，归 .b1（stone_depth 带判定差）或 biome 判定 / fill 阶段上游差；
- **b2 的正确处理**：保留恒 0.0 缺陷作为已实证的真实 bug（预加载表补 5 噪声 = 一行修复，顺带闭合 soul_soil 子族与 patch/gravel 带），但**不作为 B1 主导根因候选**；B1 主导排查资源应投向 P-A/P-C/P-D 探针区分 stone_depth vs biome vs fill 上游。
