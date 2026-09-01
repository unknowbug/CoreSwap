# C2/P2 ore-attribution verdict —— nether 矿石/岩浆块 save 偏高（~2.2×）主导机制候选

> **修订 v2（2026-09-05，V1 阶段分离探针并入；supersedes v1 的 H_A 排序，v1 正文保留于文末历史节）**：主候选由 H_A（基底命中差）**翻转为 H_B'「双重 feature 应用 = Rust 自身 feature 阶段未在 cppReplace 存档链路关闭」**。依据见 §0。

- **status**: draft（未做任何消融/探针消融验证；仅存量数据归因，按 workflow-patterns #10「先消融后归因」纪律不得升 candidate）
- **验证分层**: Degraded（静态数据解读，无 trace/probe 新证据；结论仅到「候选归因 + 可证伪设计」层）
- **可比性声明（§9.7）**: 载体 = FULL vanilla 参照 blocks vs Rust 存档 MCA 直解；覆盖面 = 4×4 @3200,3208，seed B=8576294172403134396；与旧粗口径（同数值）及 B1 SURFACE 口径（77.49%）不可直接比（SURFACE 无 carvers/features）。
- **module**: Minecraft worldgen / cppReplace 存档口径归因

## 〇、修订 v2：V1 阶段分离探针并入（关键新证据）

### V1 数据（纯 Rust 全管线，seed/origin 校验通过）
| id | ref | pureRust | save | pureRust+ref | save−(pureRust+ref) |
|---|---|---|---|---|---|
| nether_quartz_ore | 1992 | 2381 | 4478 | 4373 | +105（+2.4%） |
| nether_gold_ore | 728 | 793 | 1525 | 1521 | +4（+0.3%） |
| magma_block | 1533 | 2073 | 3814 | 3606 | +208（+5.8%） |
| basalt | 172704 | 5514 | — | — | — |

### 解读
1. **pureRust 已含矿石/magma** → Rust 管线自身在放这些块。代码路径核实：`worldgen_handle.rs fill_chunk_blocks`（:361-452）**第 5 步就是 `apply_features`**（:442-449），feature_loader.rs 支持 `ore`/`scattered_ore`/underwater_magma（feature.rs:602），nether biome JSON 装配的 ore feature 被 Rust 自己跑一遍。**V1 的「pureRust」不是纯 surface 口径，而是 Rust 全管线（含 Rust feature）**——这正是「Rust surface 阶段为何多放矿石」的答案：不是 surface/density 放的（nether surface_rule 无 ore 目标，代码 grep 无 quartz/nether_gold 放置点），是 **Rust feature 阶段放的**。
2. **save ≈ pureRust + ref 的加和形态**（gold 误差 0.3%、quartz 2.4%、magma 5.8%，远小于两侧各自残差）→ 主候选翻转为 **H_B'：cppReplace 存档链路上 Rust 的 feature 阶段未被关闭，Java features 又在 Rust 地形上独立跑一遍 → 两套独立随机序列的 feature 应用叠加**，计数近似相加。basalt（pureRust 5514 vs ref 172704）反证 B1 定论不变：Rust feature 阶段没有 basalt 大宗 feature，大宗石互换仍是 Java feature 在两种基底上的命中/形态差。
3. magma 残差 +208 略大：两套 magma 来源（Rust underwater_magma + Java blob）对同一 LAVA 基底有争用覆盖，加和模型在此有非线性项，属预期。
4. v1 的 H_A（基底命中差）**降级为次级贡献候选**：它解释的量级被 H_B' 的加和项吸收大半；仍可解释 save−(pureRust+ref) 的小残差（+105/+208）方向。
5. v1 已排除项不变：A1 未实现 ❌、seed 错位 ❌；v1 互斥候选 H_B 升为主候选（变体：不是「Java 跑两次」，是「Rust feature 阶段没关」）。

### H_B' 的机制链（可证伪表述）
cppReplace 契约 = Rust 只接管 populateNoise + buildSurface，Java 跑 carvers/features。若宿主桥接层调用 Rust `fill_chunk_blocks`（含 apply_features）而未设置 `WG_SKIP_FEATURES`（worldgen_handle.rs:443 的 env 门）或未走 skip 路径，则 Rust feature 输出已写入 chunk，Java 再叠 → save 偏高 ~2.2× 且逐 id 加和吻合。**证伪条件**：① 存档链路 Rust 侧带 WG_SKIP_FEATURES=1（或等效 skip）重跑后 ore 计数不降；或 ② 存档链路 run 的 Rust 侧日志（WG_FEATURELOG）显示 `[FEATURE] chunk placed 0 blocks`——二者任一成立则 H_B' 被推翻，回 H_A 主导。

## 〇.1 修订后下一步命令模板（主会话只执行不解读）
- **V2-A（判 H_B'，最高优先）存档链路 Rust feature 侧证**：cppReplace 存档 run 设置 `$env:WG_FEATURELOG="1"`，重生成同 seed 同区域，stderr 落盘 `.artifacts/.c2-p2-ore-attribution/featurelog-v2.txt`——统计 `placed N blocks` 的 N 总和；N>0 即 Rust feature 在存档链路活跃，H_B' 直接实锤。
- **V2-B（同判，消融版）**：同上 run 加 `$env:WG_SKIP_FEATURES="1"`，MCA 直解重算 ore per-id：预期 quartz/gold/magma 回落到 ≈ref 水平（±H_A 残差）。对照命令模板：`python .tmp/ore_per_id.py --save <save_mca_dir> --region 3200,3208 --ids nether_quartz_ore,nether_gold_ore,magma_block,ancient_debris --out .artifacts/.c2-p2-ore-attribution/v2b-ore-per-id.json`
- **V2-C（定 surface/ore_vein 清白）**：纯 surface 口径确认（`WG_SKIP_FEATURES=1 WG_SKIP_CARVER=1` rlib 直跑同区域）ore per-id 应 = 0；非 0 才需要查 surface_rule/density 附属（当前代码路径审查预期 = 0）。
- **V1 遗留 P1（ore-mask 重叠）保留**：双重应用下两套独立随机序列 → ref∩save 重叠率应显著低于「同一序列」水平（~独立交集），可与 H_B' 交叉印证。
- 执行前铁律：两侧 worldSeed 三查 + 参照 header 核对（M11 教训）。

---

## 历史 v1 正文（归因排序已被 §0 取代，判据与排除项仍有效）

## 一、问题 1：主导机制候选（三阶段归因法组织）

### 阶段分解
- 替换方（Rust）= populateNoise + buildSurface；存续方（Java）= carvers + features。矿石/岩浆块全部由存续方 feature（nether ore_blob / magma blob）产生——**两侧是同一套 Java feature 代码、同一 chunk 随机序列**（feature 种子由 chunk pos 派生，seed 一致）。因此差异只能来自**输入侧（基底地形）对命中率的调制**，不可能是「feature 实现差」。

### 已可排除（本轮数据内即可排除）
- ❌ **A1「Rust feature 未实现」**：save 侧三种 nether 矿石 + magma + debris 全部非零且高于 ref——feature 明确在跑，且 gold_ore / quartz_ore（非 nether 前缀 id）两侧均 0，overworld feature 路由正确。
- ❌ **seed/坐标错位类**：同 seed 同区域同随机序列下 ref→save 的 mismatch 块对呈**局部相邻块转换**形态（netherrack→magma 1101 等），非整体平移/翻倍形态。
- ❌ **B1 大宗石互换本身**：B1 已定论为 surface 层候选；本问题矿石 id 的 2~2.5× 偏高是 feature 层产物，不与 B1 重复计因（但可能由 B1 的基底分布差**派生**，见主候选）。

### 互斥候选（fan-out 应并行验证）
- **H_A（主候选，倾向）：基底可替换性差 = 同一 blob feature 在 Rust 基底上的命中差**。
  - 机制：ore blob 随机走每步仅在命中 `#base_stone_nether`（netherrack/basalt/blackstone）时转换。Rust surface 残差（B1：22.5%，basalt→netherrack 157,658 主导）+ netherrack 总量 ref 204,903→save 210,861，意味着 blob 落点在 Rust 地形上命中可替换基底的概率系统性更高（尤其 vanilla 侧为熔岩海/洞穴空气的位置 Rust 侧为 netherrack——mismatch 对 netherrack→magma 1101、basalt→magma 595、lava→magma 30 正是「blob 在 Rust 多出的实心基底上额外命中」的直接形态证据）。
  - 对「~2.2× 均匀倍率」（quartz 2.25 / gold 2.10 / magma 2.49）的解释：三者共用同一 base_stone_nether target，命中率的同一上调自然产生近均匀倍率；ancient_debris 1.77× 略低（debris blob discard_on_air 行为 + 稀有采样对小样本波动不敏感的量级差）与 H_A 相容。
- **H_B（互斥）：feature 重复应用/注册差**（Java feature 在 Rust 管线上被应用两次，或 ref 侧被后续阶段覆盖）。若成立也应呈均匀倍率，是 H_A 的最强竞争者。
- **H_C（次级，可与 H_A 并存非互斥）：air_discard 差**——blob 边界块贴空气时按概率丢弃；Rust 洞穴/carver 命中面不同 → 保留率不同。cave_air→gold 27 量级小，最多解释个位百分比，非主导。

### 定论所需数据层证据（各候选的判别器）
| 候选 | 判别证据 |
|---|---|
| H_A vs H_B | **ore mask 重叠统计**：逐块计算 ref-ore∩save-ore / ref-only / save-only。H_B（重复应用）预测 ref-ore ⊆ save-ore 近乎完全（ref-only ≈ 0）；H_A 预测 ref-only 占比可观（Rust 侧也有 vanilla 没命中的位置，双向差）。这是**一步可算的廉价判别器，优先做**。 |
| H_A 定量 | base_stone_nether tag 命中率对比：同区域两侧在 blob 活动 y 带内 tag 石密度曲线；若 Rust 侧命中密度比 ≈ 2.2 且 save-only blob 位置落在 Rust 多出的 netherrack 区，H_A 定量闭合。 |
| H_B 排除侧证 | Java 侧 feature 注册清单审计（nether biome json loaded features 计数 + Rust 接管管线 feature 调用日志各一次），确认无双应用。 |
| H_C 定界 | 同数据内统计 blob 块 6 邻空气占比差；量级 <10% 即可关闭。 |

## 二、问题 2：「replaceable 依赖 + Rust surface 残差改变基底分布」线索判定

**方向成立，可证伪、且与本轮数据形态自洽**：
1. data-driven-boundary.md 确认 `feature.rs expand_tag`（base_stone_nether 类 tag 展开）与 carver replaceable 是**代码硬编码升级点**——若 Rust 侧 tag 展开遗漏/多含成员，直接改变可替换集。但注意：本场景 blob feature 由 **Java vanilla** 执行，tag 展开用的是 Java 数据侧定义，Rust 硬编码只在「Rust 自身要跑 feature/carver」时才生效——因此「Rust expand_tag 硬编码错」在 cppReplace 架构下**不是**本差异的机制（架构排除了这条通路）；它成为风险点仅在未来 Rust 接管 feature 时。
2. 真正成立的通路是 **B1 派生**：surface 残差改变基底空间分布（尤其 vanilla 熔岩海/洞穴带 → Rust 实心 netherrack），blob 走样在同一随机序列下命中更多可替换块 → save 偏高。这与 H_A 同构。
3. **证伪条件**：若 ore-mask 重叠统计显示 ref-only ≈ 0 且 save-only blob 块位置与「Rust 多出的实心基底」空间不相关（相关系数≈0），则 H_A 被证伪，转 H_B。

## 三、置信度与分层声明
- 主候选 H_A：**draft，倾向成立**（数据形态三处自洽：均匀倍率、ref→save 转换对形态、lava/cave 带证据），但零消融/零探针新证据——按 workflow-patterns #10「先消融后归因」禁止升 candidate。
- 排除项 A1/seed 错位：本轮数据内实锤（可作为候选级子结论引用）。
- H_B / H_C：未验证并存候选。

## 四、下一步数据层验证动作清单（主会话只执行不解读）
1. **P1（最高优先，判 H_A/H_B）ore-mask 重叠统计**：载入同区域两侧 blocks 数据，对 id ∈ {nether_quartz_ore, nether_gold_ore, magma_block, ancient_debris} 逐块输出三维计数（both/ref-only/save-only）与 save-only 块坐标采样（≤500 点 CSV）。
   - 命令模板（脚本落 `.tmp/ore_mask_overlap.py`）：
     `python .tmp/ore_mask_overlap.py --ref <FULL_ref_blocks> --save <save_mca_dir> --region 3200,3208 --size 4 --ids nether_quartz_ore,nether_gold_ore,magma_block,ancient_debris --out .artifacts/.c2-p2-ore-attribution/ore-mask-overlap.json --samples .artifacts/.c2-p2-ore-attribution/save-only-samples.csv`
2. **P2（H_A 定量）tag 命中密度**：两侧分别统计 y ∈ [10,120] 内 netherrack/basalt/blackstone 计数按 y 带分布（10 层一分桶），输出 JSON 对比。
   - 模板：`python .tmp/tag_density_by_y.py --ref <ref_blocks> --save <save_mca_dir> --region 3200,3208 --y0 10 --y1 120 --blocks minecraft:netherrack,minecraft:basalt,minecraft:blackstone --out .artifacts/.c2-p2-ore-attribution/tag-density.json`
3. **P3（H_B 排除侧证）feature 调用计数**：gradle runServer 一次同 seed 同区域，开 feature 调用日志（或 mixin 计数器），核对 nether ore_blob 每 biome 每 chunk 调用次数 == placed feature 配置期望值（无 ×2）。
4. **P4（H_C 定界）**：P1 脚本追加 `--air-adjacency` 开关，输出 ore 块 6 邻空气占比（两侧）。
5. 采样/参照铁律：执行前核对两侧 worldSeed 一致 + 参照文件 header（三查纪律），不一致立即废。

## 五、错误/教训沉淀提示（按价值门）
- 本轮可复用判法：「同源 feature、双地形基底」的均匀倍率残差 → 先怀疑基底命中差（派生自 surface 残差），用 ore-mask 重叠（⊆ 判据）一步区分「命中差 vs 重复应用」——若验证有效，建议沉淀进 workflow-patterns #10 的判别手段清单（knowledge subagent 产出，验证后再写）。
