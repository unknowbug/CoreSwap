# b1 候选：count/IntProvider 采样语义错误（blob 置换层 +19% 回归）

- **status: draft**（置信度：判定 DENY 为静态一手对拍结论；未做动态探针，验证分层 = Degraded/静态审查）
- 课题：feature parity Phase 4b 回归定位（260905-05）
- 证据包：`.tmp/feature-parity-260905-05/working-tree.diff`；一手权威：`versions/1.20.1/data/mc_src_extract/`
- retry 轮次：1（纯静态对拍，无工程修复）

## 1. 假设（b1）

placement.rs 的 count modifier / UniformInt / IntProvider「修正」改动使 blob 数量分布偏移（均值 +19%，相等率 83%→51%）。

## 2. 对拍表（Rust 现文件 ↔ Java 1.20.1 一手源码）

| 项 | Java 权威（行号） | Rust 现实现（placement.rs） | 对拍结论 |
|---|---|---|---|
| UniformIntProvider.get | UniformIntProvider.java:40-42 = `nextBetween(random,min,max)` = `nextInt(max-min+1)+min`，**1 次消费** | L27-31 `next_int_bound(b-a+1)+a`，1 次消费 | ✅ 一致，且本次 diff **未改动**该分支 |
| BiasedToBottomIntProvider.get | BiasedToBottomIntProvider.java:39-41 = `min + nextInt(nextInt(max-min+1)+1)`，恒 2 次消费 | L47-52 `inner=next_int_bound(b-a+1); a+next_int_bound(inner+1)` | ✅ 修后与 Java 逐 token 一致（旧实现确错，但消费次数新旧均=2，**流长度不变**） |
| TrapezoidIntProvider | **1.20.1 不存在**（intprovider 目录 8 文件实证：BiasedToBottom/Clamped/ClampedNormal/Constant/IntProvider/IntProviderType/Uniform/WeightedList，无 Trapezoid） | L32-45 改写为 TrapezoidHeightProvider 语义 | ✅ 改写内容与 TrapezoidHeightProvider.java:49-64 逐行一致，但 **该分支对 1.20.1 数据是死代码**（见 §3.2） |
| CountPlacementModifier | AbstractCountPlacementModifier.java:13-15 = `IntStream.range(0, count.get(random))`；CountPlacementModifier.java:10-24 | placement.rs L279-280 `Count(count) => n=count.get(random)` | ✅ 语义一致；**受影响矿脉的 count 全部是 JSON 常量**（见 §3.1） |
| TrapezoidHeightProvider.get | TrapezoidHeightProvider.java:49-64：i>j→warn 返 i；plateau≥k→nextBetween(i,j)；否则 i+nextBetween(0,m)+nextBetween(0,l)，l=(k-plateau)/2 | carver.rs HeightProvider::get（diff hunk）同构 | ✅ 公式一致；但这是 **HeightProvider（carver.rs），不是 b1 所指的 IntProvider** |
| MathHelper.nextBetween | MathHelper.java:846-848 = `nextInt(max-min+1)+min`（1 次消费）；nextInt(random,min,max):183 = min≥max 时 0 消费 | `next_int_bound` | ✅ |

## 3. 判定：**DENY**（b1 不能解释「+19% 均值 + 相等率 83%→51%」）

### 3.1 无作用面：受影响矿脉的 count 语义根本没被本次 diff 触到

受影响方块（andesite/tuff/diorite/granite/copper_ore）的 placed_feature JSON（`versions/1.20.1/data/worldgen/data/minecraft/worldgen/placed_feature/`）：

| placed feature | count | height | 与本次 diff 的接触面 |
|---|---|---|---|
| ore_andesite_lower | **常量 2** | uniform(0,60) | 无（IntProvider::Uniform 未改） |
| ore_andesite_upper | rarity_filter 6（无常量 count） | uniform(64,128) | 无 |
| ore_tuff / granite_lower / diorite_lower | 常量 2 | uniform | 无 |
| ore_copper / ore_copper_large | 常量 16 | **trapezoid(-16,112)** | HeightProvider（carver.rs）被改，**但那是 HeightProvider 不是 IntProvider** |

即：这些 blob 的「数量分布」输入是 JSON 常量整数，UniformInt 分支本次零改动。b1 声称的「UniformInt/count 分布偏移」**没有可达作用面**。

### 3.2 IntProvider::Trapezoid 改写 = 死代码

1.20.1 无 TrapezoidIntProvider 类；grep 全部 231 个 placed_feature JSON，`"minecraft:trapezoid"` 只出现在 height_range 的 height 字段（13 个文件，全为 ore_ 系列）——该路径走 carver.rs `HeightProvider::parse`，不经过 placement.rs `IntProvider::parse`。因此 placement.rs L32-45 的改写**不可能被 1.20.1 数据触达**。

### 3.3 IntProvider::BiasedToBottom 修复 = 只影响 nether/debris 族

grep 证实 overworld placed_feature 无 biased_to_bottom count（BiasedToBottomIntProvider count 仅 NetherPlacedFeatures.java:84 nether ore(0,9)）。且新旧实现消费次数均为 2（旧：next_int_bound(b-a+1)+next_int_bound(inner+a)；新：同 2 次）——**RNG 流长度不变**，仅分布修正，且不在受影响方块族。

### 3.4 机制级算术：+142.6 块/chunk 无法归因到 b1

- 均值差 = 881.4 − 738.8 = **142.6 块/chunk**。ore_andesite blob size = 64（ore_andesite.json）→ ≈ **+2.23 个成功 blob/chunk**。
- andesite 每 chunk 期望尝试数 = 2（lower，count=2）+ 1×(1/6)（upper，rarity 6）≈ 2.17。相对 vanilla 多出 ~2.2 个 blob ≈ **每次尝试约多 1 个 blob**——这是「数量级」偏差（尝试→blob 映射被系统性改变），而 b1 所有可达路径都是「常量 count + 分布中性」：语义错误至多造成**位置/流内偏移**（影响相等率），**不可能造成每次尝试 +1 blob 的系统性均值上移**。相等率 83%→51% 同理不能由 b1 解释：Java 侧 `set_decorator_seed(population_seed, p, k)` 每 feature 独立播种（worldgen_handle.rs L879 复刻了同一点），单 feature 内流变化不泄漏到其他 feature；andesite 自身链路（count→in_square→uniform height→biome）本次 diff 零接触。

## 4. 残留疑点（同 diff 内、指向 b2/b3 方向，非本候选范围）

1. **BiomePlacementModifier 语义变更（placement.rs，anchor_biome 4×4 对齐比对）**：是 andesite 链路中唯一被本次 diff 触到的 modifier。若 `biome_at(4×4 对齐) ≠ anchor_biome(chunk 角采样)` 的口径差导致错误放行/拒绝，可双向扰动 per-chunk count——相等率下降的直接候选。
2. **HeightProvider trapezoid 修复（carver.rs）**：copper/diamond/iron/emerald 等 13 个 feature 的 Y 分布从「旧 uniform 近似」变为真三角分布（每 get 消费 1→2 次）；但每 feature 独立播种，影响限于 feature 内部（copper Y 分布），不解释 andesite。
3. **feature_loader.rs 树/植被新分支与 generate_configured 签名变更**：ore 分支本身未改；generate_nested 目前恒 false 并打日志。

## 5. 建议修复 / 下一步（数据层）

- 用 `WG_FEATURELOG=1` 跑 new dll 与 vanilla（同 seed 同 chunk 段），按 fid 统计每 feature「attempts vs 成功放置数」——把 +2.23 blob/chunk 定位到 andesite_upper（rarity 分母被改？）还是 lower（blob 形态/成功判据），一轮即可分流 b2/b3。
- andesite Y 直方图双侧对比（Y 域不变但分布可能变）。
- 核对 biome_at 回调具体绑定（jitter vs no-jitter）与 Java posToBiome 口径（8 邻域 jitter），验证残留疑点 1。

## 6. 自检声明

- 验证分层：**Degraded（静态审查）**——全部结论来自一手源码/JSON 逐行对拍与数据集 grep 实证，未运行动态探针（subagent 沙箱限制）；判定 DENY 的置信度较高，因为「无可达作用面」是结构性事实，不依赖统计。
- 未验证项：+2.23 blob 的实际来源（需 §5 数据层实验）；blob 判定脚本口径（v3_verify.py/v4_blob_cut.py 未运行核对，仅采信任务书给定的实测数字）。
