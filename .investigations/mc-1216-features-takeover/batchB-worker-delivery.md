# batchB worker 交付 —— mc-1216 features 接管批次 B（水下植被族 kelp/seagrass/sea_pickle + Heightmap.Types 塌缩处置）

> 角色：worker（代码交付）。**未编译验证**（沙箱无 shell，静态对拍交付）。
> 一手源：`versions/1.21.6/data/mc_src_extract/net/minecraft/`（主）+ `versions/1.20.1/data/mc_src_extract/net/minecraft/`（交叉核对——引擎目标版本 1.20.1，两版三 feature 逐行一致，见 §一.1 版本核对）。
> 引擎参照：`worldgen-core/src/feature_loader.rs`（HEAD 批次 A 后）/ `placement.rs` / `feature.rs`（批次 A Lake/Geode 实装风格、set_block_if_allowed、expand_tag）/ `tree.rs`（is_solid_id）。
> 状态：draft（候选需主会话应用 + 编译 + 实跑 unknown 残差验收后才可升 candidate）。
> 日期标签：本文不写 YYMMDD 标签（260904-06 纪律：主会话落盘/归档前 Get-Date 取真实时间补）。

## 〇、本批覆盖面声明

| # | 目标 | 本批做了 | 本批留了（残差 → 登记） |
|---|------|---------|--------------------------|
| 1 | Heightmap.Types「两桶塌缩」修正（placement.rs Heightmap / SurfaceRelativeThreshold 二分支） | **方案 a（主 patch，P1）**：一手源确认 7 常量全表 + 落桶表 + 逐 type 等价性/残差声明（§一.0）；二分支改显式 match + 未知 type 告警进 unknown 哨兵（`mod:` 前缀）。**结论：塌缩在 feature 阶段语义成立，不可修也无需修**（方案 b 评估后否决，见 §一.0.4） | 残差：引擎两张图为 pre-carver 快照，Java POST_CARVER 四图为 post-carver——被 carver 蚀空的列 top 偏高（既有共享残差，非本批引入） |
| 2 | minecraft:kelp（KelpFeature） | **全量**：DefaultFeatureConfig 空配置；`getTopY(OCEAN_FLOOR)` 特征级采样（chunk 内走高度图桶 / 邻域走 block_at 下扫近似，§一.1）；k 段生成循环 + 顶端 kelp AGE 消费 + 断点回接逻辑；canPlaceAt 按 AbstractPlantPartBlock 语义近似 | KELP.AGE 属性丢弃（state=i32，nextInt(4) 消费保留）；`isSideSolidFullSquare` → is_solid_id 近似 + !magma_block；邻域高度图走 block_at 逐列下扫（Java 读邻 chunk 高度图——时序差声明，idk-2） |
| 3 | minecraft:seagrass（SeagrassFeature + ProbabilityConfig） | **全量**：probability 1 字段（简报「provider」不存在，E-1）；双 nextInt(8) 差分 + OCEAN_FLOOR 采样 + nextDouble<probability 高草判定 + 上半 water 检查 | **双高海草 HALF 属性位丢失（照实声明）**：TALL_SEAGRASS 上下两半在引擎 state=i32 下不可区分，两格均放 `minecraft:tall_seagrass` 裸 id；`isSideSolidFullSquare && !magma` 同 #2 近似 |
| 4 | minecraft:sea_pickle（SeaPickleFeature + CountConfig） | **全量**：count = **IntProvider**（简报「不是 IntProvider / 1..25」与一手源不符，E-1——1.20.1 与 1.21.6 的 CountConfig.count 均为 IntProvider，数据集 `sea_pickle.json` count=20 constant）；每轮恒 `nextInt(4)` PICKLES 消费序对拍；PICKLES 属性丢弃 | PICKLES 1..4 属性位丢弃（state=i32，消费保留）；canPlantOnTop（碰撞面/侧满方近似 → is_solid_id）；邻域高度图同 #2 |

**分发接入后预期 new unknown 集合变化**：批次 A 实跑 unknown n=19 中，`minecraft:kelp`（scout-b 引用面 6）、`minecraft:seagrass`（族 7——4 个 configured_feature JSON 的 type 均为 `minecraft:seagrass`，落同一 type 名）、`minecraft:sea_pickle`（单例 1）应**从 unknown 集合消失**；预期验收 **n = 16**，且剩余集合 == n=19 清单 − {kelp, seagrass, sea_pickle}（无新增；P1 未知 heightmap 告警面在当前数据集 0 触发——97 个 `heightmap` 引用全部 ∈ 已映射 6 type，grep 实证 §一.0.3）。验收时用 WG_FEATURE_UNKNOWN_LOG=1 比对。

---

（骨架完，以下逐项补齐）

## 一、逐项对拍

### §一.0 Heightmap.Types 塌缩处置（任务 1）

#### §一.0.1 一手源全表（Heightmap.java:150-161，1.21.6；1.20.1 同构逐字核对）

| Type（index/id 顺序即枚举序） | Purpose | predicate | 语义 |
|---|---|---|---|
| WORLD_SURFACE_WG (0) | WORLDGEN | NOT_AIR（!isAir，**水计入**） | 特征阶段世界表面 |
| WORLD_SURFACE (1) | CLIENT | NOT_AIR | 存档/客户端表面 |
| OCEAN_FLOOR_WG (2) | WORLDGEN | SUFFOCATES（blocksMovement） | 特征阶段海底/地表 |
| OCEAN_FLOOR (3) | LIVE_WORLD | SUFFOCATES | 存档侧海底 |
| MOTION_BLOCKING (4) | CLIENT | blocksMovement \|\| !fluidState.isEmpty() | 实体阻挡面 |
| MOTION_BLOCKING_NO_LEAVES (5) | CLIENT | 同上 && 非树叶 | 同上去树叶 |

offset 语义：`populateHeightmaps`（Heightmap.java:43-79）存的是满足 predicate 的最高块 **y+1**（`heightmap.set(k, l, m + 1)` L64）＝首个不满足 predicate 的方块 y；`ChunkRegion.getTopY`（ChunkRegion.java:406-407）= `sampleHeightmap(...) + 1`。即**存储值已是「面上空气位」，getTopY 再 +1？**——否：`sampleHeightmap` 内部（Heightmap.java:95-107 一带）返回 `get(x,z)-1`（取存储值再 -1 得实体块 y），+1 还原 = 存储值。**引擎口径**（placement.rs:337-339 注释，260905-06 off-by-one 修正）：引擎高度图数组存「最高实体块 y」，取用点 `+1` = Java getTopY。本批特征级采样沿用该口径（P4 `get_top_y_ocean_floor` 返回 `hm+1`）。

#### §一.0.2 特征阶段到底有几张图（塌缩「是否可修」的事实基础）

1.20.1 ChunkStatus.java:33-35 / 1.21.6 ChunkStatus.java:17-28：EMPTY..SURFACE 挂 `PRE_CARVER/WORLDGEN` 两图（OCEAN_FLOOR_WG + WORLD_SURFACE_WG）；**CARVERS 起（含 FEATURES）挂 `POST_CARVER/NORMAL` 四图（OCEAN_FLOOR / WORLD_SURFACE / MOTION_BLOCKING / MOTION_BLOCKING_NO_LEAVES）**，在 carver 蚀刻后的地形上重建。

⇒ scout-b 报告的「feature 阶段只有两张图、MOTION_BLOCKING(_NO_LEAVES) 全落 world_surface 桶」的**前提部分不成立**：Java 特征阶段实际有 4 张（重建后的）图。但**结论（塌缩语义成立）仍然成立**，证明链见 §一.0.3。

引擎现实：`FeaturePlacementContext` 只有 `ocean_floor` / `world_surface` 两个 `Option<&[i32]>` 桶（placement.rs:139-140）；构造点全仓 2 处（worldgen_handle.rs:1127 / bin-diag/b5b6_smoke.rs:129，grep 实证）。引擎 `world_surface` = `cd.surface_height`（terrain.rs:248/291：NOT_AIR，**水计入**——Java WORLD_SURFACE 语义 ✓）；引擎 `ocean_floor` = 特征预处理时逐列下扫跳过 {air, water, lava} 取首个固体（worldgen_handle.rs:998-1014——SUFFOCATES 的 skip 集近似）。

#### §一.0.3 逐 type 落桶表 + 等价性/残差声明（方案 a 核心交付）

| Type | 引擎落桶 | 等价性论证 | 残差 |
|---|---|---|---|
| WORLD_SURFACE_WG | world_surface | 同名同谓词（NOT_AIR 含水，terrain.rs:288-291 注释明示对齐） | 无 |
| WORLD_SURFACE | world_surface | 特征时点地形 = noise+surface+carver，NORMAL 图是 carver 后重建；WG 图是 pre-carver → 被 carver 蚀空的列 Java 偏低 | 引擎两图均 pre-carver（ocean_floor 在特征预处理构建、surface_height 在 surface 阶段输出）——carver 蚀空列 top 偏高（既有共享残差） |
| OCEAN_FLOOR_WG | ocean_floor | 同名，skip {air,water,lava} ≈ SUFFOCATES（状态无属性，植物/雪层类不可表达——Java SUFFOCATES 还排除雪片/植被，引擎会误计入；雪片在引擎 state=i32 下本就不存在） | 同 skip 集近似残差 |
| OCEAN_FLOOR | ocean_floor | 同 WORLD_SURFACE→WG 的 carver 时差论证 | 同上 |
| MOTION_BLOCKING | world_surface | 特征时点无树叶、无玩家实体；地形面块 = 固体（blocksMovement ✓）或水/岩浆（fluid ✓）→ **MOTION_BLOCKING 顶 ≡ WORLD_SURFACE 顶**。分面验证：① 陆地列：表面 = 固体，两谓词同真 → 同值；② 水列：WORLD_SURFACE 计水（NOT_AIR），MOTION_BLOCKING 计流体 → 同值（= 水面）；③ 差集 = 「非 air、非流体、不阻挡运动」块（植被/雪片）——特征时点地形不存在（surface 阶段产物均为固体/流体；雪片 layer 属性引擎不可表达） | 仅 snow layer 存在的世界（Java 有 1 格 snow_layer 时 WORLD_SURFACE 不计、MOTION_BLOCKING 也不计、NOT_AIR 计）→ Java 侧两图本就差 1；引擎无 snow_layer，此差不存在。声明为「引擎无雪片 ⇒ 差异面为空」 |
| MOTION_BLOCKING_NO_LEAVES | world_surface | 特征时点树叶不存在（heightmap 不随 feature 放置更新，树叶是 FEATURES 阶段产物）→ NO_LEAVES ≡ MOTION_BLOCKING → 同 MOTION_BLOCKING 论证 | 同 MOTION_BLOCKING |

**数据面实证**：1.20.1 placed_feature `heightmap` 引用 97 处（grep 实证），type 分布 ⊆ {WORLD_SURFACE_WG, OCEAN_FLOOR_WG, OCEAN_FLOOR, MOTION_BLOCKING}——即现存塌缩口径已覆盖全部实际引用；count_on_every_layer 的 MOTION_BLOCKING 依赖已由 batch0 论证不依赖（placement.rs:412-416 注释）。

#### §一.0.4 方案 b（加第三桶）评估 → 否决

接线面：新增 `motion_blocking` 桶需动 ① FeaturePlacementContext + OreFeatureContext 各加字段 ② worldgen_handle.rs 构建循环（L998 一带）+ 两处构造点 ③ bin-diag/b5b6_smoke.rs 构造点 ④ placement.rs 两分支 match。**判定：否决**。理由：§一.0.3 已证 MOTION_BLOCKING 在特征时点与 WORLD_SURFACE **逐列同值**（谓词差集在引擎方块分类下为空集）——第三桶与 world_surface 位位相同，纯 plumbing 成本零 parity 收益。若未来引擎引入 snow_layer/植被态表达（属性位引擎级课题 R-4 家族），届时随属性位一起重评（登记 §六）。

**主 patch（P1）**：显式 match + 未知 type 告警进哨兵（`mod:heightmap:<type>` 前缀，batch0 命名空间区分风格），兜底落 world_surface。

### §一.1 minecraft:kelp = KelpFeature（任务 2）

**版本核对**：1.20.1 与 1.21.6 的 KelpFeature.java **逐行一致**（两版均 57 行；本轮两版全文比对）。scout-b 引用面 kelp 6（kelp_cold/kelp_warm 等 placed_feature）。

**config 修正**：**1.20.1/1.21.6 的 KelpFeature 均为 `Feature<DefaultFeatureConfig>`（KelpFeature.java:14），无 KelpFeatureConfig**（provider+spread 是 1.13~1.16 时代的旧 config，早已删除；`kelp.json` config={} 实证）。任务简报「KelpFeatureConfig：provider + spread」与一手源不符 → E-1。parse 侧按 DefaultFeatureConfig 空配置直发（对齐 batchA monster_room 先例）。

一手源：`world/gen/feature/KelpFeature.java`（1.21.6 版 file:line）。

对拍点表：

| Java | 语义 | Rust（P4） |
|---|---|---|
| :25 `getTopY(OCEAN_FLOOR, x, z)` | 特征级海底采样，`blockPos2 = (x, k, z)` | `get_top_y_ocean_floor`（chunk 内 = ocean_floor 桶 +1；邻域 = block_at 下扫近似，idk-2） |
| :27 首格 `isOf(WATER)` 门 | 不在水里 → false（0 消费已发生） | `block_at != water → return false` |
| :30 `k = 1 + nextInt(10)` | 段数 1..10 | `1 + next_int_bound(10)` |
| :32 循环 `l = 0..=k`（含 k+1 轮） | 含端点 | `for l in 0..=k` |
| :33-35 三条件门（本格水 ∧ 上格水 ∧ kelp_plant.canPlaceAt） | 全过才放置 | 同序短路 |
| :36-38 `l == k` → 放 KELP，`AGE = nextInt(4)+20`，i++ | 顶端 kelp（age≥20 不再生长） | 放 kelp id，`nextInt(4)` 消费保留（AGE 丢弃声明） |
| :39-40 else → 放 KELP_PLANT | 中段茎 | 放 kelp_plant id |
| :42-48 `l > 0` 失败分支：`blockPos3 = down()`；kelp.canPlaceAt(blockPos3) ∧ blockPos3.down() 非 KELP → 放 KELP+AGE，i++；**恒 break** | 断点回接（上方断裂处重新着根） | 同序；break |
| :51 `blockPos2 = up()` | 步进 | `py += 1`（在循环尾，break 分支不步进） |
| :55 `return i > 0` | 放置量判定 | `placed` |

（以上 Rust 列均指 P5 实现。）

canPlaceAt 近似（块级）：
- kelp_plant（AbstractPlantPartBlock.java:38-43，1.20.1；1.21.6 :43-48 同）：`below = pos.down()`；`!canAttachTo(below) → false`，否则 `below ∈ {KELP, KELP_PLANT} ∨ isSideSolidFullSquare(UP)`。KelpPlantBlock.canAttachTo → stem(KelpBlock).canAttachTo = **非 magma_block**（KelpBlock.java:35/1.20.1，1.21.6 :43-45 同）。Rust：`below==kelp ∨ below==kelp_plant ∨ (is_solid_id(below) ∧ below!=magma)`，`below<0`（不可读）→ false。
- kelp（断点回接放 KELP 的 canPlaceAt 同为 AbstractPlantPartBlock 语义——KelpBlock 也是 AbstractPlantPartBlock 子类，同一实现）。

RNG 消费序声明（独立 decorator seed 流内）：
1. `getTopY` 0 消费；首格非水 → false（**0 消费**）
2. `nextInt(10)`（:30）
3. 循环：三条件门 0 消费；命中且 `l==k` → `nextInt(4)`（:37，**实参恒求值**）；命中且 `l<k` → 0；`l>0` 断点分支命中（canPlaceAt ∧ 下下格非 kelp）→ `nextInt(4)`（:45）后 break，不命中 → **0 消费** break
4. ⚠️ 三条件门失败且 l==0 时（海底顶格即非水/上格非水）不 break，继续 `l=1` 轮——每轮门失败 l>0 即走断点分支 → l=1 轮必 break（0 或 1 次 nextInt(4)）。l=0 轮门失败时 break 不触发（`l > 0` false）→ 继续循环，与 Java 一致（P4 结构同构）

### §一.2 minecraft:seagrass = SeagrassFeature（任务 3）

**版本核对**：1.20.1 与 1.21.6 逐行一致（两版均 53 行）。scout-b seagrass 族 7：`seagrass_{short,mid,slightly_less_short,tall,normal,warm,cold,deep,deep_cold,deep_warm,river,swamp}` 中的 7 个引用——**4 个 configured JSON 的 type 均为 `minecraft:seagrass`**（probability 0.3/0.4/0.6/0.8，1.20.1 数据 grep 实证），单 type 名接入即全族消失。

**config 修正**：ProbabilityConfig 仅 `probability` 1 字段（ProbabilityConfig.java:8-16，`Codec.floatRange(0,1).fieldOf("probability")`）。任务简报「probability + provider」与一手源不符 → E-1。

对拍点表（SeagrassFeature.java:22-52，1.21.6 版；Rust 列 = P5 实现）：

| Java | Rust（P4） |
|---|---|
| :28-29 `i = nextInt(8)-nextInt(8)`、`j = 同`（x 偏移先、z 偏移后，各 2 次消费） | 同序 |
| :30 `getTopY(OCEAN_FLOOR, x+i, z+j)` | `get_top_y_ocean_floor` |
| :32 本格 `isOf(WATER)` 门（失败 → false，**双 4 次消费已发生**） | 同 |
| :33 `bl2 = nextDouble() < probability`（**水门通过后才消费**） | `next_double() < probability as f64` |
| :35 `blockState.canPlaceAt` = SeagrassBlock.canPlantOnTop（SeagrassBlock.java:34-35）: `below.isSideSolidFullSquare(UP) ∧ !magma` | `below>=0 ∧ below!=magma ∧ is_solid_id(below)` |
| :36-42 tall：上半 `blockPos3 = up()` 是 WATER 才**双格放置**（lower + upper HALF） | 双格均放 `tall_seagrass` 裸 id；**HALF 属性位丢失照实声明**（上半非水 → 两格都不放，但 `bl=true` 已成立） |
| :43-45 short：单格放 | 放 `seagrass` id |
| :47 `bl = true` 位置：canPlaceAt 通过后**即置位**（上半水检失败也 true） | `true` 位置对齐 |
| :51 `return bl` | return |

RNG 消费序：`nextInt(8)×4` → 水（0 消费）→ `nextDouble`（仅水时）→ canPlaceAt/放置 0 消费。

### §一.3 minecraft:sea_pickle = SeaPickleFeature（任务 4）

**版本核对**：1.20.1 与 1.21.6 逐行一致（两版均 42 行）。

**count 修正**：1.20.1 与 1.21.6 的 CountConfig **均为 IntProvider**（1.21.6 CountConfig.java:9-25 `IntProvider.createValidatingCodec(0, 256).fieldOf("count")`；1.20.1 同构），`getCount().get(random)` :26。任务简报「count 字段 1..25 特殊 RNG 用法、不是 IntProvider」与一手源不符 → E-1（疑为 1.13 时代 `CountConfig(int)` 或旧 JSON 记忆；本数据集 `sea_pickle.json` count=20 constant，1.20.1/1.21.6 双版一致）。P4 按 IntProvider 接线（constant→0 消费，uniform 等自动正确）。

对拍点表（SeaPickleFeature.java:21-41，1.21.6 版；Rust 列 = P5 实现）：

| Java | Rust（P4） |
|---|---|
| :26 `j = count.get(random)`（循环前一次） | `config.count.get(random)` |
| :29-30 每轮 `l,m = nextInt(8)-nextInt(8)` | 同序（x 先 z 后） |
| :31 `getTopY(OCEAN_FLOOR, x+l, z+m)` | `get_top_y_ocean_floor`（邻域不可读时 **nextInt(4) 照常消费后再跳过**，保 RNG 流——见下） |
| :33 `PICKLES = nextInt(4)+1`——**每轮恒消费**（state 构造实参，在任何检查之前） | `nextInt(4)` 无条件消费，紧跟 l/m 之后、水检查之前（顺序对拍：Java :31 采样→:33 抽 PICKLES→:34 检查） |
| :34 本格 WATER ∧ canPlaceAt（SeaPickleBlock.canPlantOnTop :55-56 = 碰撞上面非空 ∨ 侧满方——无 magma 排除） | `below>=0 ∧ is_solid_id(below)`（碰撞形状不可表达，is_solid_id 近似声明；注意 **sea_pickle 无 magma 排除**，勿抄 seagrass 的） |
| :35 放置 + i++ | `set_block(sea_pickle)`（PICKLES 属性丢弃声明） |
| :40 `return i > 0` | `placed > 0` |

RNG 消费序：`count.get`（constant 0）→ 每轮恒 `nextInt(8)×2 → nextInt(4)`（**无论后续检查成败**；这是本 feature 的特殊点——PICKLES 抽取不被水门短路）。

## 二、Patch 清单（old 串均先读 HEAD 现文核对，file:line 以当前工作区为准）

### Patch P1 — placement.rs：heightmap 落桶显式 match + 未知告警（Heightmap 分支）

文件 `worldgen-core/src/placement.rs`。先在 `impl PlacementModifier` 之前（`PlacementModifier` 枚举定义之后、`treediag_enabled` 之前均可）加自由函数；本文选 `find_multilayer_pos` 之前的文件尾区（L540 前）。为免 old 串歧义，函数体追加在 `pub fn treediag_enabled` 结束与 `impl PlacementModifier` 之间：

old（L304-310）:
```rust
pub fn treediag_enabled() -> bool {
    static ENABLED: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *ENABLED.get_or_init(|| std::env::var("WG_TREEDIAG").is_ok())
}

impl PlacementModifier {
```
new:
```rust
pub fn treediag_enabled() -> bool {
    static ENABLED: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *ENABLED.get_or_init(|| std::env::var("WG_TREEDIAG").is_ok())
}

/// batchB（mc-1216）§一.0：Heightmap.Types 7 常量 → 引擎两桶显式映射（落桶表 + 等价性声明见
/// .investigations/mc-1216-features-takeover/batchB-worker-delivery.md §一.0.3）。
/// OCEAN_FLOOR_WG / OCEAN_FLOOR → ocean_floor；WORLD_SURFACE_WG / WORLD_SURFACE /
/// MOTION_BLOCKING / MOTION_BLOCKING_NO_LEAVES → world_surface（特征时点 MOTION_BLOCKING(NO_LEAVES)
/// 与 WORLD_SURFACE 逐列同值：树叶/植被/雪片差集在引擎方块分类下为空）。
/// 未知 type：显式告警 + mod: 前缀哨兵（batch0 风格），兜底 world_surface 不静默。
fn heightmap_bucket<'a>(heightmap_type: &str, ctx: &'a FeaturePlacementContext) -> Option<&'a [i32]> {
    let t = heightmap_type.to_ascii_uppercase();
    if t.contains("OCEAN_FLOOR") {
        ctx.ocean_floor
    } else if t.contains("WORLD_SURFACE") || t.contains("MOTION_BLOCKING") {
        ctx.world_surface
    } else {
        eprintln!("[placement] unknown heightmap type: {heightmap_type} (fallback world_surface)");
        crate::feature_loader::record_unknown_type(&format!("mod:heightmap:{heightmap_type}"));
        ctx.world_surface
    }
}

impl PlacementModifier {
```

再替换 Heightmap 分支（L331-333）：

old:
```rust
            PlacementModifier::Heightmap(heightmap_type) => {
                let hm = if heightmap_type.contains("OCEAN_FLOOR") { ctx.ocean_floor } else { ctx.world_surface };
                let hm = match hm { Some(h) => h, None => return vec![[x, y, z]] };
```
new:
```rust
            PlacementModifier::Heightmap(heightmap_type) => {
                // batchB（mc-1216）：二分支 contains → heightmap_bucket 显式 match + 未知告警（§一.0）
                let hm = heightmap_bucket(heightmap_type, ctx);
                let hm = match hm { Some(h) => h, None => return vec![[x, y, z]] };
```

### Patch P2 — placement.rs：SurfaceRelativeThreshold 分支同口径

old（L389-391）:
```rust
            PlacementModifier::SurfaceRelativeThreshold { heightmap_type, has_min, has_max, min_inclusive, max_inclusive } => {
                let hm = if heightmap_type.contains("OCEAN_FLOOR") { ctx.ocean_floor } else { ctx.world_surface };
                let hm = match hm { Some(h) => h, None => return vec![[x, y, z]] };
```
new:
```rust
            PlacementModifier::SurfaceRelativeThreshold { heightmap_type, has_min, has_max, min_inclusive, max_inclusive } => {
                // batchB（mc-1216）：同 P1 显式 match + 未知告警（§一.0）
                let hm = heightmap_bucket(heightmap_type, ctx);
                let hm = match hm { Some(h) => h, None => return vec![[x, y, z]] };
```

### Patch P3 — feature_loader.rs：ConfiguredFeature 扩字段 + init None

kelp 走 DefaultFeatureConfig 空配置（无字段）；seagrass/sea_pickle 各一：

old（L66-69）:
```rust
    pub lake_config: Option<crate::feature::LakeConfig>,              // minecraft:lake
    pub geode_config: Option<Box<crate::feature::GeodeConfig>>,       // minecraft:geode（大结构，Box 减负）
    pub multiface_config: Option<crate::feature::MultifaceGrowthConfig>, // minecraft:multiface_growth
}
```
new:
```rust
    pub lake_config: Option<crate::feature::LakeConfig>,              // minecraft:lake
    pub geode_config: Option<Box<crate::feature::GeodeConfig>>,       // minecraft:geode（大结构，Box 减负）
    pub multiface_config: Option<crate::feature::MultifaceGrowthConfig>, // minecraft:multiface_growth
    // —— batchB（mc-1216）：kelp 走 DefaultFeatureConfig 空配置（无字段，分发直发）——
    pub seagrass_config: Option<crate::feature::SeagrassConfig>,      // minecraft:seagrass（ProbabilityConfig）
    pub sea_pickle_config: Option<crate::feature::SeaPickleConfig>,   // minecraft:sea_pickle（CountConfig=IntProvider）
}
```

old（init，L88-91）:
```rust
            lake_config: None,
            geode_config: None,
            multiface_config: None,
        };
```
new:
```rust
            lake_config: None,
            geode_config: None,
            multiface_config: None,
            seagrass_config: None,
            sea_pickle_config: None,
        };
```

### Patch P4 — feature_loader.rs：parse 分发（multiface_growth 分支尾后）

old（L139-144）:
```rust
        } else if type_name == "minecraft:multiface_growth" {
            cf.multiface_config = crate::feature::MultifaceGrowthConfig::parse(cfg, blocks);
            if cf.multiface_config.is_none() {
                eprintln!("[feature-loader] multiface_growth config parse failed: {id}");
            }
        } else {
```
new:
```rust
        } else if type_name == "minecraft:multiface_growth" {
            cf.multiface_config = crate::feature::MultifaceGrowthConfig::parse(cfg, blocks);
            if cf.multiface_config.is_none() {
                eprintln!("[feature-loader] multiface_growth config parse failed: {id}");
            }
        } else if type_name == "minecraft:kelp" {
            // KelpFeature = DefaultFeatureConfig 空 config（KelpFeature.java:14；kelp.json config={}）。
            // batchB E-1：简报「KelpFeatureConfig provider+spread」系旧版残留，1.20.1/1.21.6 均无
        } else if type_name == "minecraft:seagrass" {
            // ProbabilityConfig：probability 单字段（ProbabilityConfig.java:8-16；E-1「provider」不存在）
            cf.seagrass_config = crate::feature::SeagrassConfig::parse(cfg, blocks);
            if cf.seagrass_config.is_none() {
                eprintln!("[feature-loader] seagrass config parse failed: {id}");
            }
        } else if type_name == "minecraft:sea_pickle" {
            // CountConfig：count = IntProvider（CountConfig.java:9-25；E-1「非 IntProvider / 1..25」不符；
            // 数据集 sea_pickle.json count=20 constant）
            cf.sea_pickle_config = crate::feature::SeaPickleConfig::parse(cfg, blocks);
            if cf.sea_pickle_config.is_none() {
                eprintln!("[feature-loader] sea_pickle config parse failed: {id}");
            }
        } else {
```

### Patch P5 — feature.rs：新增水下植被族实现（追加到文件尾）

追加到 `lava_pool_stone_cannot_replace` 之后（文件尾）。全部全限定路径，不动 use 区（ChunkRandom/JsonValue/BlockRegistry 均已在文件头；IntProvider 走 crate::placement:: 全路径与 batchA geode 同口径）。

```rust

// ===== batchB（mc-1216）：kelp / seagrass / sea_pickle =====
// 一手源：versions/1.20.1 + versions/1.21.6 双版 mc_src_extract（两版逐行一致，批次 B §一.1 版本核对）：
//   world/gen/feature/KelpFeature.java / SeagrassFeature.java / SeaPickleFeature.java
//   world/gen/ProbabilityConfig.java / world/gen/CountConfig.java
//   block/AbstractPlantPartBlock.java / KelpBlock.java / SeagrassBlock.java / SeaPickleBlock.java
// 对拍表 + RNG 消费序声明：.investigations/mc-1216-features-takeover/batchB-worker-delivery.md §一
// 已知限制（本族）：state=i32 无属性位（KELP.AGE / TALL_SEAGRASS.HALF / SEA_PICKLE.PICKLES 不可表达，
// 对应 RNG 消费全部保留）；isSideSolidFullSquare/碰撞形状 → tree::is_solid_id 近似；
// 方块实体/流体 tick no-op（同 batchA 全族声明）。

/// 特征级 getTopY(Heightmap.Type.OCEAN_FLOOR, x, z)（KelpFeature.java:25 / SeagrassFeature.java:30 /
/// SeaPickleFeature.java:31）。chunk 内 = ocean_floor 桶 +1（引擎口径：数组存最高实体块 y，
/// placement.rs:337-339 同源）；邻域列（Java 读邻 chunk 高度图）→ block_at 逐列下扫近似
/// （skip {air,water,lava} = 现行 ocean_floor 桶同口径）；不可读 → None（调用点按「跳过但保 RNG 流」处理，
/// batchB idk-2）。
fn get_top_y_ocean_floor(ctx: &OreFeatureContext, wx: i32, wz: i32) -> Option<i32> {
    let air = crate::blocks::AIR;
    let water = ctx.blocks.id("minecraft:water");
    let lava = ctx.blocks.id("minecraft:lava");
    let lx = wx - ctx.chunk_start_x;
    let lz = wz - ctx.chunk_start_z;
    if lx >= 0 && lx < 16 && lz >= 0 && lz < 16 {
        let hm = ctx.ocean_floor?;
        let top = hm[(lz * 16 + lx) as usize];
        return Some(top + 1); // ChunkRegion.getTopY = sampleHeightmap + 1（placement.rs:337 同源）
    }
    // 邻域：自顶下扫首个非 {air,water,lava}（≈SUFFOCATES），返回其上一位
    let mut y = ctx.min_y + ctx.height - 1;
    while y >= ctx.min_y {
        let b = ctx.block_at(wx, y, wz);
        if b < 0 { return None; }
        if b != air && b != water && b != lava { return Some(y + 1); }
        y -= 1;
    }
    None
}

/// kelp 族 canPlaceAt（AbstractPlantPartBlock.java:38-43 + KelpBlock.canAttachTo = !magma）：
/// below ∈ {kelp, kelp_plant} ∨ isSideSolidFullSquare(UP)（→ is_solid_id ∧ !magma 近似）。
/// 不可读（<0）→ false 保守拒绝。
fn kelp_can_place_at(ctx: &OreFeatureContext, x: i32, y: i32, z: i32,
                     kelp: i32, kelp_plant: i32, magma: i32) -> bool {
    let below = ctx.block_at(x, y - 1, z);
    if below < 0 || below == magma { return false; }
    below == kelp || below == kelp_plant || crate::tree::is_solid_id(ctx, below)
}

// ===== minecraft:kelp（KelpFeature.java，DefaultFeatureConfig 空配置）=====
pub struct KelpFeature;
impl KelpFeature {
    /// RNG 消费序（§一.1）：首格非水 0 消费 false → nextInt(10) → 循环门 0 消费；
    /// l==k 放置 nextInt(4)（AGE，属性丢弃消费保留）；l>0 断点命中 nextInt(4) 后恒 break。
    /// y 形参不参与（Java 仅用 origin.x/z + 高度图）。
    pub fn generate(ctx: &mut OreFeatureContext, random: &mut ChunkRandom, x: i32, _y: i32, z: i32) -> bool {
        let blocks: &BlockRegistry = ctx.blocks;
        let water = blocks.id("minecraft:water");
        let kelp = blocks.id("minecraft:kelp");
        let kelp_plant = blocks.id("minecraft:kelp_plant");
        let magma = blocks.id("minecraft:magma_block");
        let Some(mut py) = get_top_y_ocean_floor(ctx, x, z) else { return false; };
        if ctx.block_at(x, py, z) != water { return false; }                      // :27
        let k = 1 + random.next_int_bound(10);                                    // :30
        let mut placed = false;
        for l in 0..=k {                                                          // :32（含端点）
            if ctx.block_at(x, py, z) == water
                && ctx.block_at(x, py + 1, z) == water
                && kelp_can_place_at(ctx, x, py, z, kelp, kelp_plant, magma) {    // :33-35
                if l == k {
                    let _age = random.next_int_bound(4);                          // :37 AGE=nextInt(4)+20（丢弃）
                    ctx.set_block(x, py, z, kelp);
                    placed = true;
                } else {
                    ctx.set_block(x, py, z, kelp_plant);                          // :40
                }
            } else if l > 0 {
                let by = py - 1;                                                  // :43 down()
                if kelp_can_place_at(ctx, x, by, z, kelp, kelp_plant, magma)      // :44 canPlaceAt
                    && ctx.block_at(x, by - 1, z) != kelp {                       // :44 下下格非 KELP
                    let _age = random.next_int_bound(4);                          // :45
                    ctx.set_block(x, by, z, kelp);
                    placed = true;
                }
                break;                                                            // :48 恒 break
            }
            py += 1;                                                              // :51（break 分支不达）
        }
        placed                                                                    // :55 i > 0
    }
}

// ===== minecraft:seagrass（SeagrassFeature.java + ProbabilityConfig）=====
#[derive(Clone)]
pub struct SeagrassConfig {
    pub probability: f32,
}
impl SeagrassConfig {
    /// ProbabilityConfig.java:8-16：probability 单字段（floatRange 0..1）。E-1：无 provider 字段。
    pub fn parse(cfg: Option<&JsonValue>, _blocks: &BlockRegistry) -> Option<SeagrassConfig> {
        let p = cfg?.get("probability")?.as_f64()?;
        Some(SeagrassConfig { probability: p as f32 })
    }
}
pub struct SeagrassFeature;
impl SeagrassFeature {
    /// RNG 消费序（§一.2）：nextInt(8)×4 → 水（0）→ nextDouble（仅水时）→ 0。
    /// 双高海草 HALF 属性位丢失声明：两格均放 tall_seagrass 裸 id（上/下半不可区分）。
    pub fn generate(ctx: &mut OreFeatureContext, config: &SeagrassConfig,
                    random: &mut ChunkRandom, x: i32, _y: i32, z: i32) -> bool {
        let blocks: &BlockRegistry = ctx.blocks;
        let water = blocks.id("minecraft:water");
        let seagrass = blocks.id("minecraft:seagrass");
        let tall_seagrass = blocks.id("minecraft:tall_seagrass");
        let magma = blocks.id("minecraft:magma_block");
        let i = random.next_int_bound(8) - random.next_int_bound(8);              // :28
        let j = random.next_int_bound(8) - random.next_int_bound(8);              // :29
        let (px, pz) = (x + i, z + j);
        let Some(py) = get_top_y_ocean_floor(ctx, px, pz) else { return false; }; // :30
        if ctx.block_at(px, py, pz) != water { return false; }                    // :32
        let tall = random.next_double() < config.probability as f64;              // :33
        // SeagrassBlock.canPlantOnTop（SeagrassBlock.java:34-35）：isSideSolidFullSquare(UP) ∧ !magma
        let below = ctx.block_at(px, py - 1, pz);
        if !(below >= 0 && below != magma && crate::tree::is_solid_id(ctx, below)) {
            return false;                                                         // :35
        }
        if tall {                                                                 // :36-42
            if ctx.block_at(px, py + 1, pz) == water {                            // :39 上半水检
                ctx.set_block(px, py, pz, tall_seagrass);                         // :40 HALF=lower（丢弃）
                ctx.set_block(px, py + 1, pz, tall_seagrass);                     // :41 HALF=upper（丢失）
            }
            // 上半非水：两格都不放，但 bl 已置位（Java :47 位置在 canPlaceAt 后）
        } else {
            ctx.set_block(px, py, pz, seagrass);                                  // :44
        }
        true                                                                      // :47/:51
    }
}

// ===== minecraft:sea_pickle（SeaPickleFeature.java + CountConfig=IntProvider）=====
#[derive(Clone)]
pub struct SeaPickleConfig {
    pub count: crate::placement::IntProvider,
}
impl SeaPickleConfig {
    /// CountConfig.java:9-25：count = IntProvider（validating 0..=256）。E-1：非「1..25 非 IntProvider」。
    pub fn parse(cfg: Option<&JsonValue>, _blocks: &BlockRegistry) -> Option<SeaPickleConfig> {
        let c = cfg?.get("count")?;
        Some(SeaPickleConfig { count: crate::placement::IntProvider::parse(Some(c)) })
    }
}
pub struct SeaPickleFeature;
impl SeaPickleFeature {
    /// RNG 消费序（§一.3）：count.get 一次 → 每轮恒 nextInt(8)×2 → nextInt(4)（PICKLES，
    /// **水门之前恒消费**，Java :33 实参求值序）→ 检查 0 消费。PICKLES 属性丢弃消费保留。
    /// 邻域高度图不可读：nextInt(4) 照常消费后再跳过（保 RNG 流，idk-2）。
    pub fn generate(ctx: &mut OreFeatureContext, config: &SeaPickleConfig,
                    random: &mut ChunkRandom, x: i32, _y: i32, z: i32) -> bool {
        let blocks: &BlockRegistry = ctx.blocks;
        let water = blocks.id("minecraft:water");
        let sea_pickle = blocks.id("minecraft:sea_pickle");
        let n = config.count.get(random);                                         // :26
        let mut placed = 0i32;
        for _ in 0..n {                                                           // :28
            let i = random.next_int_bound(8) - random.next_int_bound(8);          // :29
            let j = random.next_int_bound(8) - random.next_int_bound(8);          // :30
            let py_opt = get_top_y_ocean_floor(ctx, x + i, z + j);                // :31
            let _pickles = random.next_int_bound(4);                              // :33 PICKLES=nextInt(4)+1（恒消费）
            let Some(py) = py_opt else { continue; };
            if ctx.block_at(x + i, py, z + j) != water { continue; }              // :34 前半
            // SeaPickleBlock.canPlantOnTop（:55-56）：碰撞上面非空 ∨ 侧满方（无 magma 排除——勿抄 seagrass）
            let below = ctx.block_at(x + i, py - 1, z + j);
            if !(below >= 0 && crate::tree::is_solid_id(ctx, below)) { continue; } // :34 后半（形状近似声明）
            ctx.set_block(x + i, py, z + j, sea_pickle);                          // :35
            placed += 1;
        }
        placed > 0                                                                // :40
    }
}
```

### Patch P6 — feature_loader.rs：generate 分发（multiface_growth 分支尾后）

old（L547-551）:
```rust
    } else if cf.type_name == "minecraft:multiface_growth" {
        match &cf.multiface_config {
            Some(mc) => crate::feature::MultifaceGrowthFeature.generate(octx, mc, random, x, y, z),
            None => { eprintln!("[feature-loader] multiface_growth without config: {}", cf.id); false }
        }
    } else {
```
new:
```rust
    } else if cf.type_name == "minecraft:multiface_growth" {
        match &cf.multiface_config {
            Some(mc) => crate::feature::MultifaceGrowthFeature.generate(octx, mc, random, x, y, z),
            None => { eprintln!("[feature-loader] multiface_growth without config: {}", cf.id); false }
        }
    } else if cf.type_name == "minecraft:kelp" {
        crate::feature::KelpFeature::generate(octx, random, x, y, z)
    } else if cf.type_name == "minecraft:seagrass" {
        match &cf.seagrass_config {
            Some(sc) => crate::feature::SeagrassFeature.generate(octx, sc, random, x, y, z),
            None => { eprintln!("[feature-loader] seagrass without config: {}", cf.id); false }
        }
    } else if cf.type_name == "minecraft:sea_pickle" {
        match &cf.sea_pickle_config {
            Some(pc) => crate::feature::SeaPickleFeature.generate(octx, pc, random, x, y, z),
            None => { eprintln!("[feature-loader] sea_pickle without config: {}", cf.id); false }
        }
    } else {
```
（签名核对：generate_configured 现有形参 (cf, ctx, octx, random, x, y, z, biome_temp, biome_rainfall, cache)——三项均只需 octx/random/x/y/z。`FeaturePlacementContext` 未加字段 → 两个构造点（worldgen_handle.rs:1127 / bin-diag/b5b6_smoke.rs:129）零改动。）

## 三、静态自检清单（未编译验证声明）

**状态：全部 patch 未编译、未运行**（沙箱无 shell）。静态自检如下：

- [x] i32/usize 索引 cast：`hm[(lz * 16 + lx) as usize]`——lz/lx ∈ [0,15] 且已范围守卫，`lz*16+lx ≤ 255` 安全（与 placement.rs:336 同型同守卫）。
- [x] `as f64 <` 泛型歧义陷阱（batchA 实踩形态）：本批唯一浮点比较 `random.next_double() < config.probability as f64`（P5 seagrass）——next_double() 返回 f64 具体类型，`as` 只作用于 probability，无推断歧义面；无 `as f64` 后直接 `<` 字面量的写法。
- [x] panic 面：`next_int_bound(bound>0)`——kelp `nextInt(10)`/`nextInt(4)`；seagrass `nextInt(8)×4`；pickle `count.get`（IntProvider 各变体 bound≥1，constant 20）+ `nextInt(8)`/`nextInt(4)`；均 >0。`ctx.ocean_floor?` None → 返回 None 不 panic；`(lz*16+lx)` usize 转换有守卫。
- [x] 借用/所有权：`get_top_y_ocean_floor(ctx: &OreFeatureContext, ...)` 只读借用，与调用点的 `ctx: &mut` 分时相容（调用前无可变借用存活——各 generate 内均先算 `let Some(py) = ...` 再用 ctx）；`kelp_can_place_at` 同为 & 借用且 ids 按值传参；`config.count.get(random)` 为 `&IntProvider` 方法（Count 分支 L314 同型先例）。`let blocks: &BlockRegistry = ctx.blocks;`（& 是 Copy）后 ctx 仍可变用——batchA LakeFeature 同型先例（feature.rs:808）。
- [x] 区间/端点：kelp `0..=k`（Java `l=0; l<=k`，含端点——batchA 发现 #10 家族核对项）；seagrass/pickle 无 rev/负区间；pickle `0..n`（Java `k<j` 半开）。
- [x] 分发两侧清单一致（parse ↔ generate）：kelp/seagrass/sea_pickle 三项均精确匹配 + 双侧补齐；catch-all 告警 + record_unknown_type 保持不变。
- [x] 构造点完备：**未改任何 struct 字段**（FeaturePlacementContext/OreFeatureContext 均不动）→ 无构造点连锁；P1 自由函数只增不改。
- [x] RNG 消费序：三 feature 逐行对拍表见 §一.1/§一.2/§一.3；关键短路点（kelp 三条件门 0 消费 / seagrass nextDouble 仅水后 / pickle nextInt(4) 恒消费）均已按 Java 求值序落位；邻域高度图 miss 时 pickle 的 nextInt(4) 已先消费（保流）。
- [x] 行为回归面：P1/P2 替换后已知 4 type 落桶不变（WORLD_SURFACE*→world_surface、OCEAN_FLOOR*→ocean_floor、MOTION_BLOCKING*→world_surface——与旧 `contains("OCEAN_FLOOR")` 二分支对已知 type 的行为逐 type 等价，§一.0.3 表）；新增仅未知 type 告警路径（当前数据集 0 触发）。

## 四、错误记录（五段式）

### E-1 任务简报三处与一手源不符（发现于对拍，非代码错误）

- **现象**：简报称 ① kelp =「KelpFeatureConfig：provider + spread」② seagrass config =「probability + provider」③ sea_pickle count =「1..25 特殊 RNG 用法、不是 IntProvider」。
- **根因**：①② 为旧版残留记忆——1.13~1.16 曾有 KelpFeatureConfig(planted+provider+spread)，1.20.1/1.21.6 的 KelpFeature 均为 DefaultFeatureConfig（KelpFeature.java:14，kelp.json config={}）；ProbabilityConfig 仅 probability 一字段（ProbabilityConfig.java:8-16）。③ CountConfig.count 自 1.17 起即 IntProvider（1.20.1/1.21.6 CountConfig.java:9-25 双版核对），数据集 `sea_pickle.json` count=20 constant；「1..25」疑为更早版本 JSON 记忆。若按简报写字段，provider 类字段恒 miss → parse 返回 None → 全部落 catch-all「config parse failed」告警，特征静默全灭。
- **定位**：三个 feature + 两个 config 一手源逐文件读（1.20.1 与 1.21.6 双版交叉核对）+ 数据集 JSON grep。
- **修复**：全部以一手源为准——kelp 空配置直发（batchA monster_room 先例）、seagrass 单字段、sea_pickle IntProvider 接线（P4/P5）。
- **教训**：同 batchA E-1 结论——简报字段名不进 parse 前一手源核对不可用；本批升级为「双版（1.20.1/1.21.6）交叉核对」：引擎目标 1.20.1 有本地 extract 时必须两版都读，防 1.21.6 独有改动静默混入（本批三 feature 两版逐行一致，属实后按简报基准交付）。

### E-2 scout-b「两桶塌缩」前提部分不成立（机制澄清，非代码错误）

- **现象**：scout-b 报告 placement.rs:311/369 `contains("OCEAN_FLOOR")` 二分支把 MOTION_BLOCKING(_NO_LEAVES) 全塌进 world_surface 桶，列为待修正项；初步假设「feature 阶段只有两张高度图 → 塌缩不可修」。
- **根因**：前提半错——1.20.1 ChunkStatus.java:33-35（1.21.6 :17-28 同构）CARVERS 起挂 POST_CARVER 四图（OCEAN_FLOOR/WORLD_SURFACE/MOTION_BLOCKING/MOTION_BLOCKING_NO_LEAVES），feature 阶段 Java 实有 4 张。但**塌缩结论本身成立**：特征时点地形无树叶/植被态/雪片，MOTION_BLOCKING 谓词（blocksMovement∨fluid）与 WORLD_SURFACE（NOT_AIR）的差集在引擎方块分类下为空集 → 两图逐列同值（陆地列同固体顶、水列同为水面——NOT_AIR 计水），塌缩到 world_surface 是**正确映射而非近似损失**（逐 type 证明 §一.0.3）。
- **定位**：ChunkStatus 高度图挂载表逐行读 + Heightmap.Types 谓词逐常量核对 + terrain.rs:288-291 引擎 NOT_AIR 口径实证 + 1.20.1 placed_feature 97 处 heightmap 引用 grep 实证（全部 ∈ 已映射 4 type）。
- **修复**：不做第三桶（方案 b 否决，§一.0.4——接线面 4 处、零 parity 收益）；主 patch 为显式 match + 未知 type 告警哨兵（P1/P2），消除未来未知 type 静默落 world_surface 的面。
- **教训**：「不可修」类结论必须先核实「事实前提」再接受——scout 报告的机制方向按交接纪律做廉价独立验证（本次 = ChunkStatus/Heightmap 两文件核对）后才能继承；「前提错但结论对」也要把证明链写全，否则下次还会再验一遍。

## 五、@anchor.idk 清单（随 patch 注释落码）

- **idk-1**（本族通用）：方块 state 属性位不可表达——KELP.AGE（nextInt(4) 消费保留）、TALL_SEAGRASS.HALF 上下半、SEA_PICKLE.PICKLES 1..4 均丢失，palette 对比按「存在性」对齐而非状态（batchA §〇 #3/#4 同族口径）。
- **idk-2**（特征级高度图邻域）：Java `getTopY(OCEAN_FLOOR, x±8, z±8)` 读邻 chunk（post-carver 重建）高度图；引擎 chunk 内走 ocean_floor 桶（pre-carver 快照）、邻域走 block_at 逐列下扫近似。两重时序差：① 桶快照 pre-carver vs Java post-carver（carver 蚀空列 top 偏高）；② block_at 下扫读到的是**含本 chunk 已生成 feature 方块**的实况（Java 高度图不含 feature 产物）→ 早先生成的 kelp 自身会顶高后续列扫描。方向：局部多放/偏高，量级随海蚀洞密度。验收需覆盖 ocean biome 用例（ca_min 开/关各一）。
- **idk-3**（canPlaceAt 近似）：`isSideSolidFullSquare`/碰撞形状 → `is_solid_id`（APPROX_NON_SOLID 排除表口径，batchA P5 同源）；sea_pickle 的 canPlantOnTop 碰撞形状分支完全丢失，仅剩侧满方近似——非常规底块（如气泡柱/海泡石半砖）上的放置判定可能偏差。id 失配/不可读（-1）→ 保守拒绝。
- **idk-4**（seagrass tall 上半水检）：引擎 world 无流体 tick，上半 WATER 检查按放置时点实况读；Java 同点但放完会 scheduleFluidTick——引擎 no-op（batchA 全族声明），上半水柱稳定性差异不在此层修正。

## 六、主会话后续动作建议

1. 应用 P1→P6（P3 先于 P4/P6；其余无依赖）；`cargo build --offline -p worldgen --release` 编译门。
2. WG_FEATURE_UNKNOWN_LOG=1 实跑：验收 unknown 集合 == n=19 − {minecraft:kelp, minecraft:seagrass, minecraft:sea_pickle} = **n=16**（见 §〇 预期）；`mod:heightmap:*` 告警面应为 0。
3. judge 审查（candidate 前 SHOULD）：重点核 §一.1 kelp 循环 break/步进结构（`py += 1` 位于循环尾、break 分支不步进——与 Java :48/:51 位置等价）与 §一.3 pickle 的 nextInt(4) 恒消费位置。
4. 属性位引擎级课题（R-4 家族）扩容：AGE/HALF/PICKLES 入清单；若属性位课题启动，Heightmap 方案 b（第三桶）随之重评（§一.0.4）。
