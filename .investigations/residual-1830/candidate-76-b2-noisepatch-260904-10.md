# 残留 76 主族候选 .b2 —— 「噪声 patch 阈值擦边」机制候选分析

- 状态：**draft**（全部静态读码，未运行时验证；验证分层 = Degraded 静态审查）
- 产物：`.investigations/residual-1830/candidate-76-b2-noisepatch-260904-10.md`
- 输入证据：`.tmp/p2full/residual76-dump-260904-10.txt`（76 残差，seed=8576294172403134396，origin=(200,200) 4x4 chunks，盘上双臂直读，主会话 4-A1/A2 实测）
- 代码来源：`WorldgenRust/src/surface_rules.rs`（本仓库）、`versions/1.20.1/docs/06-surface-rules.md`

---

## 1. 候选机制（.b2 原表述）

Rust 侧 ocean/deep-ocean floor 的 gravel/sand patch 由噪声阈值条件选择
（`noise_threshold_sample` ∈ 带、`minecraft:surface` /8.25 分带），CS 与 vanilla
在少数 patch 边界带上翻转 → 成簇双向互换（gravel↔sand 主族 73 块）。

## 2. 代码定位（行号，surface_rules.rs）

### 2.1 深海底 gravel/sand 的实际选择路径 —— **不含任何噪声阈值条件**

规则树中海洋/深海洋底段（`build_overworld_rule`，mr9 海洋段 L966-1037）：

- **海底下深水柱到达的分支 = L1020-1037**：`StoneDepth{floor}` → Seq：
  - L1023-1027: frozen_peaks/jagged_peaks → stone（本区 biome 为深海，不命中）；
  - L1029-1034: `warm_ocean / lukewarm_ocean / deep_lukewarm_ocean` → `mr2`（L624-630：stone_depth ceiling → sandstone，否则 **sand**）；
  - L1035: fallback `mr3`（L631-637：stone_depth ceiling → stone，否则 **gravel**）。
- **沙/砾的选择器 = 纯 biome 等值匹配（L1029-1034 vs L1035 fallback），没有任何 NoiseThreshold 条件参与**。
- mc8（water(-1)，L610）门控的海洋段 L967-986：深水柱 `blockY >= fluid_height-1=61` 不可能为真（海底 y∈[35,48]），不可达；其中 mr8 内的噪声条件（L767/771 packed_ice/ice 等）也随之不可达。

规则树内全部 NoiseThreshold 清单（行号：key, 带）：
- L650 calcite [-0.0125,0.0125]（stony_peaks 门控）
- L660 gravel [-0.05,0.05]（stony_shore 门控）
- L669/726/734/738/742/804/808/817/821/825/837/841 surface ±n/8.25（windswept/taiga 等陆地 biome 门控）
- L682/686 powder_snow [0.45,0.58]/[0.35,0.6]（snowy_slopes/grove 门控）
- L696/700/767/771 packed_ice、ice（frozen_peaks 门控）
- L756-758 mc16-18 surface [-0.909,-0.5454]/[-0.1818,0.1818]/[0.5454,0.909]（wooded_badlands/badlands 门控）
- L888/901 surface_swamp ≥0（swamp/mangrove_swamp 门控）

**全部被非深海 biome 的 Biome 条件（L85 `SurfaceCond::Biome`）短路。** 本区 dump biome 仅
raw[20]/raw[29] 两种（按字节长度 20/29 推断 = `minecraft:deep_ocean`(20 字符) /
`minecraft:deep_lukewarm_ocean`(29 字符)），无任一门控 biome。

### 2.2 noise_threshold_sample 本体（L136-157）

- L111-114 `NoiseThreshold::test`：`d >= min_th && d <= max_th`（**双端闭区间**，L113）。
- L136-157：thread_local `HashMap<noise_key,(col_key,value)>` 单列缓存（L78-80、L138-156）。
- **col_key 折叠碰撞推演（L40-42）**：`(((x as u32) as u64) << 32) ^ (z as u32 as u64)`
  是 (u32,u32)→u64 的**双射**——高 32 位 = x 的位型、低 32 位 = z 的位型，负数 `(x as u32)`
  只是按位重解释（Java `x << 32` 的 int 移位等价），**不同 (x,z) 对不可能折叠出相同 key**。
  负坐标碰撞：不成立（这是双射，不是截断折叠）。缓存键无碰撞通路。
- 采样语义 L148：`n.sample(x as f64, 0.0, z as f64)`——y 恒 0.0（对齐 Java NoiseThresholdCondition
  采样 (x,0,z) 列语义，06 篇条件表 L30）。
- 缺 key 回退（L147-153 + L160-170）：回退 0.0 且每 key 全局 warn 一次——**会打
  `[SURFACE-WARN]` 日志**；若 .b2 成立（某 key 缺失致回退 0.0），CS 运行日志必有该 warn。可作廉价反证探针。

### 2.3 ENGINE_NOISE_KEYS（L48-58）覆盖面

含 `gravel`/`calcite`/`surface_swamp` 等 9 key，**不含 `minecraft:surface`**——但按 L44-47
注释契约，rule 树内引用 key 由 `collect_rule_noise_keys`（L207-232，启动期机械收集）在
worldgen_handle 预加载，ENGINE_NOISE_KEYS 只管 rule 树外引擎调用（如 sample_run_depth L564
用 get_noise，缺 key 直接 panic L551，非静默）。故「缺 key→0.0」只可能命中**条件树内**未被
预加载覆盖的 key——而收集函数覆盖所有 NoiseThreshold/带 noise 字段节点（L176-178 注释，#26
判据 1 泛化），结构性缺口已被关闭。

## 3. 与 Java 对拍点逐项（06 篇语义表 vs Rust）

| 对拍项 | Java 语义（06 篇） | Rust（行号） | 一致? |
|---|---|---|---|
| NoiseThreshold 采样点 | (x, 0, z) 列 | L148 `sample(x,0.0,z)` | ✓ |
| 带端点开闭 | `min ≤ v ≤ max`（闭区间） | L113 `>= && <=` | ✓ |
| 海底 sand/gravel 选择器 | mr9 海洋段 biome 等值匹配（mr2 sand / mr3 gravel fallback），**无噪声条件**（06 篇 L81：mr9 海洋段 = water/frozen/sand + gravel fallback） | L1020-1037 | ✓（两侧同构） |
| 8.25 分带 | 仅 windswept/gravelly/taiga 等陆地分支 | L669 等 12 处，均在陆地 biome 门控内 | ✓（深海不可达） |
| runDepth 公式 | `(int)(surface*2.75+3.0+split*0.25)` | L564-566 | ✓（06 篇 L118 已验证） |

**结论：Java 参照本身在深海底就不存在「噪声阈值选 gravel/sand」的机制**——.b2 假设的
机制在参照侧语义表中即不存在，Rust 与 Java 在该点同构。

## 4. 形状一致性论证（对 dump residual76-dump-260904-10.txt）

- 主族 73 块全部分布 y∈[35,42]、列顶 62、wd 14-27、biome 双侧相同、dtop=0（L90-95 统计）：
  差异块 = 每列**海底表层那一格**（62-wd=y 逐行吻合，如 L7：y36/wd26）。
- **双向互换空间上交错混杂**：`sand→gravel` 24 块与 `gravel→sand` 49 块在相邻坐标穿插
  （如 (238,193-196) sand→gravel 与 (244,196-200) gravel→sand 相距 ≤8 格；(239,245)
  sand→gravel 紧邻 (240,244) gravel→sand）。若机制是「噪声值整体差极小量 → 阈值等值线
  附近翻转」，翻转带应沿单一等值线集中、单侧分布；**双向交错分布在同一噪声场里要求两侧
  同时贴着上下两条带界**，而该处根本没有噪声带——形状证据反对 .b2。
- (237,224,41/42) 的 `granite→gravel` + `gravel→water` 成对（L13-14）：同列 ref 在 y42 有
  gravel、y41 是 granite，CS 把 y42 变 water、y41 变 gravel——**表层序列整体错一格**的
  形态，指向「每列表层选择器差一位」（分支命中/floor 判定），非连续噪声场翻转。
- 关键反证组合：raw[29]（deep_lukewarm_ocean）列上 ref(Java)=sand（符合其 biome 分支）而
  CS=gravel（L47-48）；raw[20]（deep_ocean）列上反向（L19-21）。**biome 双侧记录相同**（.b3
  已排除记录层差异）→ CS 侧分支选择与记录 biome 不符，是「分支选择器内部差」（surface 规则
  实际读到的 biome / floor 判定），不是噪声值差。

## 5. 可排除性评估

**判定：.b2（噪声 patch 阈值擦边）对主族 73 块 gravel↔sand 可静态排除**（排除证据）：

1. 深海底 sand/gravel 选择路径 L1020-1037 内无任何 NoiseThreshold 条件——机制不存在通路（§2.1）。
2. Java 参照同构：深海底选择 = biome 等值，参照侧也无噪声 patch（§3）。
3. dump 中双向交错分布 + 错格形态与「阈值等值线翻转」形状不符（§4）。
4. col_key 双射无碰撞（§2.2），缺 key 回退有全局 warn 可廉价反证（§2.2/§2.3）。

剩余非主族 3 块（water→dirt 1、granite→gravel 1、gravel→water 1）同属海底表层序列错位
形态，同样不经噪声条件，不救活 .b2。

**限制声明（未运行时验证）**：可达性论证是静态规则树走读；「CS 分支选择器内部差」的精确
根因（surface 规则实际 biome 采样 y / stone_depth floor 判定 / biome 注册表名比对）属
.b3 家族内部细化，需 §6 探针裁决。

## 6. 判别/收尾探针设计（主会话执行，命令模板）

目的：裁决「CS surface 规则实际读到的 biome_id / 分支命中」与记录 biome 的偏差（.b3 内部
细化），并顺手反证 .b2 warn 通路。

1. **CS 侧 surface ctx dump**（surface_rules.rs 已有同款门控先例 WG_SOUL_CTX_DUMP L460-487，
   按其模式加 `WG_OCEANFLOOR_CTX_DUMP=<点文件>`，在 `build_surface` 命中点 dump
   `ctx.biome_id / stone_depth_above / fluid_height / 分支选择结果`）：
   ```
   $pts = @'
   236 36 248
   238 35 193
   243 38 240
   244 35 196
   237 41 224
   '@
   Set-Content -Path E:\PYTHON\CoreSwap\.tmp\p2full\oceanfloor-points.txt -Value $pts
   $env:WG_OCEANFLOOR_CTX_DUMP='E:\PYTHON\CoreSwap\.tmp\p2full\oceanfloor-points.txt'
   pwsh versions/1.20.1/cpp/build.ps1 -Target block_probe   # 或对应 Rust 探针构建链
   ```
   判据：diff 列上 CS 实际 biome_id vs 记录 raw[20]/raw[29]（deep_ocean / deep_lukewarm_ocean）；
   **若实际 biome_id 与记录不符 → .b3 内部（surface 规则 biome 采样点/名称比对）坐实，.b2 结案**。
2. **Java 侧配对**（坐标钉死，workflow-patterns #17）：RouterProbe 型 `SURFBIOME (x, floorY, z)`
   同 5 点，floorY=62-wd；判据 = Java 记录 biome 与 CS 记录逐点全等（已在 dump 见证，二次核对）。
3. **.b2 warn 反证**（零成本）：上述 CS 运行 stderr grep `\[SURFACE-WARN\]`；出现任何
   unknown noise key warn → .b2 复活重审（预期无）。
4. 若需完全闭环噪声通路（预期不需要）：bin-diag 单编探针打印 diff 列
   `gravel/surface` 噪声值与 |v-边界| 距离；判据 = |d| 不贴边（>0.01 量级）→ 与 §4 形状结论互证。

## 7. 诚实声明

- 本文全部结论 = **静态读码，未运行时验证**（Degraded 分层）；行号以本 session 读到的
  surface_rules.rs 为准。
- raw[20]/raw[29] → deep_ocean/deep_lukewarm_ocean 的映射由名字字节长度推断（`minecraft:`
  10 + 名长），未读 biome 注册表确证——探针 1 可一并确证。
- 未引用其他模块 skill 正文；所有行号/坐标引用均来自上文列出的两个输入文件。
