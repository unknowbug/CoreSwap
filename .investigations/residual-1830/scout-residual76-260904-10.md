# scout-residual76-260904-10.md ——「新残留 76」（surface 材质微族）只读勘探

> 角色：recode-scout（subagent 隔离，只读，静态读码；**全部结论未运行时验证**，除引用文件内的既有运行时记录）
> 日期标签：260904-10（按任务方命名，日期锚由主会话 Get-Date 复核）
> 勘探对象：aquifer splitter 修复（f41555d，dll 5E2ACB7F）后残差 76：gravel→sand 49 / sand→gravel 24 / water→dirt 1 / granite→gravel 1 / gravel→water 1

---

## 1. 残差 76 坐标/明细清单定位

### 1.1 现状：**76 块的逐块坐标清单尚未落盘**

- 存档口径对比脚本 `.tmp/p2full/verify_aquifix_260904-09.py`（来源：本文件 L16-34）只输出 pair 计数（top12 对）与总量 `residual: 1830 -> 76`，**不打印坐标**。
- 双臂输入文件都在盘上（可直接提取坐标，零重导成本）：
  - ref：`.tmp/p2full/ref/vanilla_8576294172403134396_4_200_200.blocks`（seed=8576294172403134396，origin=(200,200)，chunk(12,12)..(15,15)——来源 `.tmp/p2full/judge-independent-260904-08.out.txt` L1-3）
  - new：`.tmp/p2full/off-aquifix-260904-09/vanilla_8576294172403134396_4_200_200.blocks`
- ⚠️ 文件头四查提示（build-tooling #7/#10/#11 判据）：对比前核对 header magic `WG B2` / seed / size / origin；`has_biome=True`（biome 通道在文件内，可一并提取 diff 列 biome——见 §4 探针）。

### 1.2 已知历史样本坐标（旧 dll 时代，**当前 76 内是否仍存未验证**）

| 坐标 (x,y,z) | vanilla→CS | 来源文件 |
|---|---|---|
| (236, 33, 198) | gravel→sand | `.tmp/p2full/transpose_check_260904-08.py` L18（旧 off 臂实测） |
| (211, 20, 229) | coal_ore→无（矿脉族，非本课题） | 同上 L19 |
| (192, 20, 197) | 水洞形状差（aquifer 族，已修） | 同上 L20 |

- (236,33,198) 的意义：**y=33 深层 surface 材质差**——该深度只可能是 **cave/carver 腔壁腔底上的 surface 规则产物**（地表 y≈60+ 的 beach/gravelly_hills 分支到不了这里），直接支持「surface 邻接块级联 / 腔面规则输入差」方向（judge 零星 3 块的猜测可扩展到主族）。

### 1.3 坐标提取命令模板（主会话执行，产物落 `.tmp/p2full/`）

```powershell
# 写 .tmp/p2full/residual76_dump.py 后执行（无 shell 权限，模板交主会话）：
# 复用 coltool：逐 cell 比对 ref vs off-aquifix-260904-09，对每个 diff 打印
#   (x, y, z, ref块, new块, 列顶非air y, 列水深, biome通道值)
# 并分别汇总 gravel->sand / sand->gravel / 零星3 三个子清单（≥10 样本）
python .tmp/p2full/residual76_dump.py > .tmp/p2full/residual76-dump-260904-10.txt
```

脚本要点（从 recon_b2_260904-08.py L30 抄正确索引）：
`y = MINY + i // 256; lz, lx = divmod(i % 256, 16)`（y-major 布局 `i = y'*256 + z*16 + x`——**勿用旧 divmod(i,4096)**，workflow-patterns #40 反模式）；列顶/水深直接抄 recon_b2_260904-08.py L14-55。

---

## 2. 分歧点清单（surface 规则域，gravel↔sand 候选机制点）

Rust 实现 = `WorldgenRust/src/surface_rules.rs`（对齐 C++ surface.h / Java VanillaSurfaceRules/MaterialRules）。规则树里产出 gravel/sand 的分支（对拍点）：

### 2.1 规则树产 gravel/sand 的分支（Rust 侧定位）

| 分支 | 材质 | 触发条件（Rust surface_rules.rs 行号） | 静态对拍点 |
|---|---|---|---|
| mr2（sand patch） | sandstone/sand | mc14=BEACH/warm_ocean/snowy_beach、mc15=desert（L639-674）；STONE_DEPTH_CEILING 快捷 | mr2 L624-630 |
| mr3（gravel patch） | stone/gravel | stony_shore（noise `minecraft:gravel` ∈ [-0.05,0.05]，L656-665）；windswept_gravelly_hills（`minecraft:surface` ≥2.0/8.25 与兜底，L730-747/813-829）；badlands 段兜底（L943） | mr3 L631-637 |
| windswept 系列 stone/dirt | （间接把 sand/gravel 分支挤出） | `minecraft:surface` 1.0/8.25、-1.0/8.25 阈值（L737-744） | 阈值是否与 Java NoiseParametersKeys.SURFACE 分母 8.25 一致 |
| 海洋段 mr8/mr4 | 海底床面 | STONE_DEPTH_FLOOR + water(-1,0)（L966-985） | B2 修复域 |

### 2.2 候选机制点（对拍面，均「未运行时验证」）

- **D-1 噪声采样输入/阈值（gravel/surface patch 边界）**：`noise_threshold_sample`（surface_rules.rs L136-157）= `sampler.sample(block_x, 0.0, block_z)` + thread_local **单列缓存**（每 key 每列一个值）。Java NoiseThresholdCondition 同为 (x,0,z) 列采样（docs/06 L30 语义表）。候选：① sampler 装配（JSON→sampler 的 octave/amplitude/种子派生）微差；② **列缓存 col_key 折叠**（L40-42 `x<<32 ^ z`——碰撞域理论存在，且 `(x as u32) as u64` 对负数是按位折叠非符号扩展，(-1,z) 与某个正 x 同 key 需具体验证，本勘探未算术证明）。
- **D-2 ENGINE_NOISE_KEYS 完备性**：L48-58 清单含 `minecraft:gravel` 但缺 `minecraft:surface`/`calcite` 之外的 rule 树 key 依赖说明——注释 L47 声明 rule 树 key 由 collect_rule_noise_keys 启动期机械核对兜底（#26 判据），风险低但值得在探针 stderr 里确认无 `warn_unknown_noise_key`（缺 key 时回退 0.0 → **threshold 带边界系统性翻转**，恰好是 gravel↔sand 双向互翻的形状；L145-153 fail-fast 只 warn 不 panic）。
- **D-3 扫描起点 surface_height 级联（B2 同源臂）**：`terrain.rs:284` `if top == i32::MIN && kind != BlockKind::Air`（260904-08 B2 修复后，水计入）。surface 扫描起点 o = surface_height+1 → q（runDepth）/r（fluidHeight）/s（首非默认块）全链随起点走（docs/06 L54-64 列引擎）。起点评级差 ±1 → STONE_DEPTH_CEILING/FLOOR 在腔底/海底边界翻转 → gravel↔sand 互换。
- **D-4 biome 分支判定**：beach/stony_shore/warm_ocean/windswept_gravelly_hills 的 biome 判定差 → sand 分支 vs gravel 分支整体切换（docs/06 L99-105 的 8 邻域选点族）；列级 biome 在 blocks 文件 biome 通道里可直接读（探针 §4-B1）。
- **D-5 列引擎 s/default 集合**：`isDefaultBlock`（==stone）集合与 `s = v+1` 语义（docs/06 L62-64、L94）——腔底 gravel 被当非默认块后其上规则层切换；granite→gravel / water→dirt 零星块更像此层。
- **D-6 off-by-one 家族**（compiler-idioms #10）：VerticalGradient / 区间 rev 类（260903-13 已修 est 臂一处）；腔面扫描起点与 bedrock/deepslate 段边界再查一遍半开区间形态（静态 grep `rev()` 即可低成本复核）。

### 2.3 零星 3 块（water→dirt / granite→gravel / gravel→water）

- judge 猜测「surface 邻接块级联」：静态支持——三对都指向**腔底/水线单格边界**，与 D-3（起点 ±1 级联）或 D-5（default 集合）形状吻合；granite 参与说明有 feature/ore 或 blob 相邻位，级联假设优先。
- #15 签名预判：76 块若为「孤立单格 + 方向不系统 + 分布在 patch 边界」→ 阈值擦边类（D-1/D-2）；若成簇/成列 → 结构类（D-3/D-4/D-5）。**探针先验签名再做机制下钻**（#15 判据）。

---

## 3. B2 同源性判定

### 3.1 B2 修复是什么（口径还原）

- 260904-08 工作块（git `977454f`，本 subagent 无 shell 权限未读 commit，引用 NEXT_SESSION.md L4/L32 转述）= 「heightmap/margin 修复」；其直接落地代码即 `WorldgenRust/src/terrain.rs:279-286` 现行判据：`kind != BlockKind::Air`（**水计入 surface_height**），注释 L281-283 自证来源 = `heightmap-criterion-divergence-260904-06.md`（海底床面材质族 ~12000 块根因：Rust 原判据 `d > 0.0` 使水柱不计入 → 海洋列 surface_height 偏低 → surface 起点偏低）。
- B2 修复域 = **开放海洋列**（heightmap-criterion-divergence-260904-06.md §触发域：仅「列顶块=流体」的海洋列；陆地列两侧一致）。

### 3.2 对残差 76 的判定

- **部分同源（级联余量），非同一机制根**：B2 修的是「海洋列 heightmap 判据」这一**大方向**；76 块是其修复后仍在的**同层级残差**——同用 surface_height 起点（terrain.rs:284 是唯一填充点，heightmap-criterion-divergence-260904-06.md §影响：worldgen_handle.rs:556/681 均消费它），但 76 的量级（76 vs 原族 ~12000）与构成（gravel↔sand 双向微族）表明：① 大头已闭合；② 剩余为**边界带**——最可能 = surface_height 与 Java WORLD_SURFACE_WG（Heightmap.NOT_AIR 语义，同 md §结论3）在**少数列**仍差（如 carver/aquifer 修改后的列顶、浮点密度擦边的列）→ D-3。
- **判别证据**（§4 探针）：76 块所在列的「列顶非 air y」双侧是否相等。若 diff 列两侧列顶全相等 → B2 起点链排除，转 D-1/D-2/D-4；若存在列顶差 1 → D-3 实锤。
- 已知样本 (236,33,198)：y=33 深层腔面，**不在海洋列 heightmap 触发域**——若它在现 76 内，则该块必然非 B2 域，直接证明 76 ≠ 单一 B2 余量。

---

## 4. 判别探针设计（最小成本，主会话执行）

### 4-A1【零重导，第一优先】残差 76 全量 dump + 列上下文
- 命令：见 §1.3 模板（`python .tmp/p2full/residual76_dump.py`）。
- 产出判据：y 分布（腔面 y<50 vs 地表 y≥55）/ 列顶差 / 双向方向系统性 / 簇形态。一次运行同时回答 #15 签名分类 + B2 同源（§3.2）。
- 成本：纯盘上双文件直读，~1 分钟，无 Java/Rust 重导。

### 4-A2【零重导】diff 列 biome 提取
- blocks 文件 `has_biome=True`（judge-independent-260904-08.out.txt L3）：residual76_dump.py 顺带打印 biome 通道值 → 与规则树 biome 分支比对（D-4 判据：diff 列是否落在 beach/stony_shore/windswept_gravelly_hills 等分支边界 biome 上）。

### 4-B【需重导，仅 A1/A2 不能分类时】噪声值配对（D-1 判据）
- Rust：复用 `WG_BIOMEDUMP`/`-biomeDump` 机制模式加门控 dump，或 bin-diag 单编探针打印 diff (x,z) 的 `noise_threshold_sample("minecraft:gravel"/"minecraft:surface")` 值（seed 三查全套）。
- Java：需 RouterProbe 型反射 dump 对应 noise 值（`.tmp/aqdump/` 的 AquiferDumpProbeMixin 模式可克隆）；对比时坐标钉死（workflow-patterns #17）。
- 判据：diff 点噪声值距阈值带边界的 |d| 量级——擦边带内 → D-1/D-2；远离边界 → 排除。

### 4-C【静态零成本】warn 扫描
- 重导或任意 cppReplace 运行的 stderr 里 grep `warn_unknown_noise_key` 输出（每 key 仅一次）——出现即 D-2 实锤（缺 key 回退 0.0 → 阈值带翻转）。

### 4-D【静态零成本】rev()/区间扫描复核（D-6）
- `grep -n "rev()" WorldgenRust/src/surface_rules.rs WorldgenRust/src/terrain.rs`，逐处对照 Java 含两端语义（compiler-idioms #10 判据）。

---

## 5. 互斥候选分叉清单（供 fan-out，.bN 候选）

| 候选 | 机制 | 判别证据类型 | 优先级 |
|---|---|---|---|
| **.b1 B2 起点级联**（D-3） | surface_height 少数列仍与 Java WORLD_SURFACE_WG 差 ±1（carver/aquifer/密度擦边列顶差）→ 列引擎 q/r/s 级联 → gravel↔sand | 4-A1 的「diff 列列顶差」计数；证据 = 块级双臂直读 | 高（先验最强，探针同轮免费） |
| **.b2 噪声 patch 擦边**（D-1/D-2） | gravel/surface 噪声值在阈值带边界微差或缺 key 回退 0.0 → patch 归属翻转 | 4-A1 签名（孤立单格/不系统）+ 4-B 配对噪声值 \|d\| + 4-C stderr | 高 |
| **.b3 biome 分支切换**（D-4） | diff 列 biome 判定差（8 邻域/选点）→ sand 系分支 vs gravel 系分支 | 4-A2 biome 通道 vs 规则树分支归属 | 中 |
| **.b4 腔面 default 集合/s 语义**（D-5） | isDefaultBlock 集合或 s=v+1 在腔底边界差 → 规则层错位（零星 3 块重点嫌疑） | 4-A1 零星块邻接剖面（±1 格 6 邻域直读） | 中（样本少，可与 .b1 合并验证） |
| **.b5 off-by-one 残留**（D-6） | 区间/rev 形态残留 | 4-D 静态扫描 | 低（静态即判） |

- fan-out 触发建议：4-A1+A2 跑完后若签名分裂（如主族指向 .b1、零星 3 块指向 .b4），按判据分组派 .bN worker；**互斥性说明**：.b1/.b4 同属「列引擎输入差」可共存，.b2/.b3 属「判定输入差」可共存——严格互斥对 = (.b1 起点) vs (.b2 噪声) vs (.b3 biome) 三选一主导，建议 fan-out 只按这三分叉派发。

---

## 6. 来源与纪律声明

- 全部行号/阈值引用来源：`WorldgenRust/src/surface_rules.rs`（L40-157/597-1000）、`WorldgenRust/src/terrain.rs`（L241-286）、`versions/1.20.1/docs/06-surface-rules.md`、`.artifacts/lossless-accel/{residual1830-verdict-260904-09, heightmap-criterion-divergence-260904-06}.md`、`.tmp/p2full/{verify_aquifix_260904-09, transpose_check_260904-08, recon_b2_260904-08}.py`、`NEXT_SESSION.md`、`knowledge/discovered/workflow-patterns.md` #15/#16/#17、`knowledge/discovered/compiler-idioms.md` #10。
- 本 subagent 无 shell：未执行任何脚本/git/导出；§1.3/§4 命令模板交主会话。
- 260904-08 B2 修复 commit 977454f 本体未读（无 shell），B2 内容口径 = NEXT_SESSION.md 转述 + terrain.rs 现行代码注释自证，已双源交叉；主会话如有 shell 可 `git show 977454f --stat` 一行复核。
- 状态：本产物 = scout 勘探 draft，不作结论；未运行时验证。
