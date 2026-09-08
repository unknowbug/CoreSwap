# batch0-worker-delivery — mc-1216 features 接管 批次 0（分发修正）

> worker 交付（core-worker + anchor-write）。**只读现有代码，产出 patch，未应用、未编译。**
> 日期标签：260909-##（宿主时间未重取——本文件为交付产物非 git 记录，主会话应用时按 §三.5 纪律重取日期回填 commit 信息）。
> 状态：**draft / 未编译验证（Degraded=纯静态审查）**。置信度：candidate 前置（需主会话应用+编译+WG_FEATURE_UNKNOWN_LOG 实跑验收）。

---

## ① 未编译验证声明

本交付全部代码块**未经编译**（沙箱无 shell）。主会话应用后必须：
1. `cargo build --offline -p worldgen --release`（或 `-p WorldgenRust` 全量绿）；
2. 跑一次 features，`WG_FEATURE_UNKNOWN_LOG=1` 验收：`[FEATURE-UNKNOWN]` 行的 unknown 集合 == 批次 A-C 完成后的预期残差清单（防 catch-all 假绿）。

## ② Java 一手源引用清单（versions/1.21.6/data/mc_src_extract/，只读参照权威）

| 注册名 | 一手源（文件:行） |
|---|---|
| `minecraft:ore` | net/minecraft/world/gen/feature/Feature.java:76 `register("ore", new OreFeature(...))` |
| `minecraft:scattered_ore` | Feature.java:103 `register("scattered_ore", new ScatteredOreFeature(...))` |
| `minecraft:disk` | Feature.java:74 `register("disk", new DiskFeature(...))` |
| `minecraft:spring_feature` | Feature.java:36 `register("spring_feature", ...)` — **注意是 `spring_feature` 不是 `spring`** |
| `minecraft:freeze_top_layer` | Feature.java:54 `register("freeze_top_layer", ...)` |
| `minecraft:underwater_magma` | Feature.java:67-69 `register("underwater_magma", ...)` |
| modifier `minecraft:count` | net/minecraft/world/gen/placementmodifier/PlacementModifierType.java:17 |
| modifier `minecraft:count_on_every_layer` | PlacementModifierType.java:22-24 |
| modifier 其余精确名 | PlacementModifierType.java:8-30（block_predicate_filter/rarity_filter/surface_relative_threshold_filter/surface_water_depth_filter/biome/noise_based_count/noise_threshold_count/environment_scan/heightmap/height_range/in_square/random_offset/fixed_placement） |
| CountMultilayer 语义 | net/minecraft/world/gen/placementmodifier/CountMultilayerPlacementModifier.java:35-58（getPositions 外层 do-while 逐层）/ 65-85（findPos 逐列下扫找第 targetY 个「可生成面」）/ 87-89（blocksSpawn = air ∪ water ∪ lava） |
| 误捕面证据 | `forest_rock`（Feature.java:73）/ `nether_forest_vegetation`（nether 系，未在本 grep 窗口但 1.20.1 数据集存在）均含子串 `ore`（f-**ore**-st）→ 旧 `contains("ore")` 分支误捕 |

注：1.20.1 与 1.21.6 的这些注册名一致（1.16 起稳定）；本项目数据集为 1.20.1，按任务指定以 1.21.6 一手源核对。

### 误捕面全量排查（contains 各分支）

| 旧分支 | 误捕类型（1.20.1 数据集 type 名全扫） |
|---|---|
| `contains("ore")` | `minecraft:forest_rock`、`minecraft:nether_forest_vegetation`（"forest" 含 "ore"）|
| `contains("disk")` | 无（仅 `minecraft:disk` 本身） |
| `contains("spring")` | 无（仅 `minecraft:spring_feature`） |
| `contains("underwater_magma")` / `contains("freeze_top_layer")` | 无 |
| placement `contains("count") && !contains("noise")` | `minecraft:count_on_every_layer`（语义完全不同）；`noise_threshold_count` 被 noise 排除后落 unknown 告警（本批次不改，残差清单登记） |

被剥离的类型在新 catch-all 中**显式告警 + 计数**，不再静默错配。

---

## ③ 改动 patch（文件:行号 + old→new）

### Patch 1 — feature_loader.rs parse 分发改精确匹配（L52-61）

**old**（feature_loader.rs:52-61）：
```rust
        if type_name.contains("ore") {
            cf.ore_config = OreFeatureConfig::parse(cfg, blocks);
        } else if type_name.contains("disk") {
            cf.disk_config = DiskFeatureConfig::parse(cfg, blocks);
        } else if type_name.contains("spring") {
            cf.spring_config = SpringFeatureConfig::parse(cfg, blocks);
        } else if type_name.contains("underwater_magma") {
            cf.magma_config = UnderwaterMagmaFeatureConfig::parse(cfg, blocks);
        } else if type_name.contains("freeze_top_layer") {
            cf.freeze_top = true;
        } else if type_name == "minecraft:tree" {
```

**new**：
```rust
        // batch0（mc-1216）：contains 子串分发 → 精确匹配（1.21.6 Feature.java 注册名逐一核对，
        // 见 .investigations/mc-1216-features-takeover/batch0-worker-delivery.md §②）。
        // contains("ore") 曾误捕 forest_rock / nether_forest_vegetation（"forest" 含 "ore" 子串）
        // → 配置全错且无告警；误捕类型现落 catch-all 显式告警 + unknown 计数。
        if type_name == "minecraft:ore" {
            cf.ore_config = OreFeatureConfig::parse(cfg, blocks);
        } else if type_name == "minecraft:scattered_ore" {
            cf.ore_config = OreFeatureConfig::parse(cfg, blocks);
        } else if type_name == "minecraft:disk" {
            cf.disk_config = DiskFeatureConfig::parse(cfg, blocks);
        } else if type_name == "minecraft:spring_feature" {
            cf.spring_config = SpringFeatureConfig::parse(cfg, blocks);
        } else if type_name == "minecraft:underwater_magma" {
            cf.magma_config = UnderwaterMagmaFeatureConfig::parse(cfg, blocks);
        } else if type_name == "minecraft:freeze_top_layer" {
            cf.freeze_top = true;
        } else if type_name == "minecraft:tree" {
```

### Patch 2 — feature_loader.rs parse catch-all 加哨兵计数（L81-84）

**old**（feature_loader.rs:81-84）：
```rust
        } else {
            // 未知 configured type 显式告警（消除静默丢弃，b2 S2）——不 panic（S4 全量加载门）
            eprintln!("[feature-loader] unknown configured feature type: {type_name} ({id})");
        }
        cf
```

**new**：
```rust
        } else {
            // 未知 configured type 显式告警（消除静默丢弃，b2 S2）——不 panic（S4 全量加载门）
            eprintln!("[feature-loader] unknown configured feature type: {type_name} ({id})");
            // batch0 哨兵：去重收集（BTreeSet），阶段末汇总（WG_FEATURE_UNKNOWN_LOG 门控）
            record_unknown_type(&type_name);
        }
        cf
```

### Patch 3 — feature_loader.rs generate 分发改精确匹配（L384-398）

**old**（feature_loader.rs:384-398）：
```rust
    if cf.type_name.contains("ore") {
        let is_scattered = cf.type_name.contains("scattered_ore");
        if is_scattered {
            crate::feature::ScatteredOreFeature.generate(octx, &cf.ore_config, random)
        } else {
            crate::feature::OreFeature.generate(octx, &cf.ore_config, random)
        }
    } else if cf.type_name.contains("disk") {
        crate::feature::DiskFeature.generate(octx, &cf.disk_config, random)
    } else if cf.type_name.contains("spring") {
        crate::feature::SpringFeature.generate(octx, &cf.spring_config, random)
    } else if cf.type_name.contains("freeze_top_layer") {
        crate::feature::FreezeTopLayerFeature.generate(octx, biome_temp, biome_rainfall, random)
    } else if cf.type_name.contains("underwater_magma") {
        crate::feature::UnderwaterMagmaFeature.generate(octx, &cf.magma_config, random)
    } else if cf.type_name == "minecraft:tree" {
```

**new**（与 parse 侧分支清单严格一致）：
```rust
    // batch0（mc-1216）：generate 分发同步改精确匹配（清单与 parse 侧一致），
    // 误捕类型不再错误消费 RNG 走 ore/disk 生成路径。
    if cf.type_name == "minecraft:ore" {
        crate::feature::OreFeature.generate(octx, &cf.ore_config, random)
    } else if cf.type_name == "minecraft:scattered_ore" {
        crate::feature::ScatteredOreFeature.generate(octx, &cf.ore_config, random)
    } else if cf.type_name == "minecraft:disk" {
        crate::feature::DiskFeature.generate(octx, &cf.disk_config, random)
    } else if cf.type_name == "minecraft:spring_feature" {
        crate::feature::SpringFeature.generate(octx, &cf.spring_config, random)
    } else if cf.type_name == "minecraft:freeze_top_layer" {
        crate::feature::FreezeTopLayerFeature.generate(octx, biome_temp, biome_rainfall, random)
    } else if cf.type_name == "minecraft:underwater_magma" {
        crate::feature::UnderwaterMagmaFeature.generate(octx, &cf.magma_config, random)
    } else if cf.type_name == "minecraft:tree" {
```

### Patch 4 — feature_loader.rs generate catch-all 加告警+哨兵（L471-473）

**old**（feature_loader.rs:471-473）：
```rust
    } else {
        false
    }
}
```

**new**：
```rust
    } else {
        // batch0：generate 侧 catch-all 从静默 false → 显式告警 + unknown 计数
        //（原误捕类型在此被静默跳过；新路径与 parse 侧 catch-all 同口径）
        eprintln!("[feature-loader] unknown configured feature type at generate: {} ({})", cf.type_name, cf.id);
        record_unknown_type(&cf.type_name);
        false
    }
}
```

### Patch 5 — feature_loader.rs 哨兵本体（新增，插在 L12 `use` 之后 / L14 注释之前）

**new**（新增代码，无 old）：
```rust
// ===== unknown-type 计数哨兵（batch0，mc-1216）=====
// catch-all 告警路径去重收集 unknown type_name；features 阶段结束（apply_features 尾部）
// WG_FEATURE_UNKNOWN_LOG=1 时打一行汇总。验收判据：unknown 集合 == 预期残差清单（防 catch-all 假绿）。
// 并发性：apply_features 可能多线程跑（CoreSwapPool），用 Mutex<BTreeSet>；仅 unknown 路径加锁
// （正常热路径零成本），门控 env 进程级 OnceLock 读一次（对齐 treediag_enabled 模式，placement.rs:283）。
static UNKNOWN_TYPES: std::sync::OnceLock<std::sync::Mutex<std::collections::BTreeSet<String>>> =
    std::sync::OnceLock::new();

pub fn record_unknown_type(t: &str) {
    let set = UNKNOWN_TYPES.get_or_init(|| std::sync::Mutex::new(std::collections::BTreeSet::new()));
    if let Ok(mut s) = set.lock() {
        s.insert(t.to_string());
    }
}

fn unknown_log_enabled() -> bool {
    static ENABLED: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *ENABLED.get_or_init(|| std::env::var("WG_FEATURE_UNKNOWN_LOG").is_ok())
}

pub fn report_unknown_types() {
    if !unknown_log_enabled() { return; }
    match UNKNOWN_TYPES.get() {
        Some(set) => {
            if let Ok(s) = set.lock() {
                let list: Vec<&str> = s.iter().map(|x| x.as_str()).collect();
                eprintln!("[FEATURE-UNKNOWN] n={} list=[{}]", s.len(), list.join(", "));
            }
        }
        None => eprintln!("[FEATURE-UNKNOWN] n=0 list=[]"),
    }
}
```

### Patch 6 — placement.rs Count 分支剥离 + count_on_every_layer 实装：parse 侧（L391）

**old**（placement.rs:390-394）：
```rust
        let type_name = m.get("type").and_then(|t| t.as_str()).unwrap_or("");
        if type_name.contains("count") && !type_name.contains("noise") {
            if let Some(c) = m.get("count") {
                return Some(PlacementModifier::Count(IntProvider::parse(Some(c))));
            }
        } else if type_name.contains("rarity_filter") {
```

**new**：
```rust
        let type_name = m.get("type").and_then(|t| t.as_str()).unwrap_or("");
        // batch0（mc-1216）：contains("count") 曾把 count_on_every_layer 吸进 Count 分支
        //（语义完全不同：Count=同点复制 n 次，CountOnEveryLayer=逐层找可生成面）。
        // 改精确匹配（PlacementModifierType.java:17/22-24 核对）。
        if type_name == "minecraft:count" {
            if let Some(c) = m.get("count") {
                return Some(PlacementModifier::Count(IntProvider::parse(Some(c))));
            }
        } else if type_name == "minecraft:count_on_every_layer" {
            // CountMultilayerPlacementModifier.MODIFIER_CODEC：IntProvider count 字段（0..=256）
            // water/lava/bedrock id 在 parse 期解析（get_positions 无 BlockRegistry 访问）
            return Some(PlacementModifier::CountOnEveryLayer {
                count: m.get("count").map(|c| IntProvider::parse(Some(c))).unwrap_or(IntProvider::Constant(1)),
                water_id: blocks.id("minecraft:water"),
                lava_id: blocks.id("minecraft:lava"),
                bedrock_id: blocks.id("minecraft:bedrock"),
            });
        } else if type_name.contains("rarity_filter") {
```

### Patch 7 — placement.rs 枚举变体（L268-279 之后插入）

**old**（placement.rs:272-279）：
```rust
    /// environment_scan（EnvironmentScanPlacementModifier.java:46-69）
    EnvironmentScan {
        down: bool,
        max_steps: i32,
        target: BlockPredicate,
        allowed: Option<BlockPredicate>, // alwaysTrue 缺省 → None 表示恒真
    },
}
```

**new**：
```rust
    /// environment_scan（EnvironmentScanPlacementModifier.java:46-69）
    EnvironmentScan {
        down: bool,
        max_steps: i32,
        target: BlockPredicate,
        allowed: Option<BlockPredicate>, // alwaysTrue 缺省 → None 表示恒真
    },
    /// count_on_every_layer（batch0 实装，CountMultilayerPlacementModifier.java:35-85）
    /// 语义：外层 do-while 逐层（layer 0 = 最顶可生成面），每层随机 count 个 (x,z)，
    /// findPos 从列顶下扫描找第 layer 个「air/water/lava 之下是实体非基岩」的界面，
    /// 返回该空气位 y；某层全空即停。
    CountOnEveryLayer { count: IntProvider, water_id: i32, lava_id: i32, bedrock_id: i32 },
}
```

### Patch 8 — placement.rs get_positions 实装（NoiseBasedCount 分支后、match 收尾前，L379-386）

**old**（placement.rs:379-387）：
```rust
            PlacementModifier::NoiseBasedCount { max_count, noise_name, scale, count } => {
                // Java：count + floor(noise(x*scale, 0, z*scale) * maxCount)
                let _ = (noise_name, scale);
                let noise = 0.0; // 需要 noise sampler——Phase 3 简化 0
                let n = (count.get(random) + (noise * *max_count as f64).floor() as i32).max(0);
                (0..n).map(|_| [x, y, z]).collect()
            }
        }
    }
```

**new**：
```rust
            PlacementModifier::NoiseBasedCount { max_count, noise_name, scale, count } => {
                // Java：count + floor(noise(x*scale, 0, z*scale) * maxCount)
                let _ = (noise_name, scale);
                let noise = 0.0; // 需要 noise sampler——Phase 3 简化 0
                let n = (count.get(random) + (noise * *max_count as f64).floor() as i32).max(0);
                (0..n).map(|_| [x, y, z]).collect()
            }
            PlacementModifier::CountOnEveryLayer { count, water_id, lava_id, bedrock_id } => {
                // Java CountMultilayerPlacementModifier.java:35-58 精确移植：
                // - k/l = nextInt(16)+pos.x/z（自身含 square，列在本 chunk 内 → block_at 可读整列）
                // - 每层（do-while 一轮）先 count.get(random)（每层消费），再每点 2 次 nextInt(16)
                // - findPos 无 RNG；返回 Integer.MAX_VALUE → Java 跳过（Rust None）
                // 高度图依赖：Java getTopY(MOTION_BLOCKING, k, l) 定扫描起点 m——本实现从列顶
                // （min_y+height-1）起扫。等价性：MOTION_BLOCKING 顶之上 air-over-air 不满足
                // findPos 条件（上方块 air/water/lava 即 blocksSpawn=true 被排除），首个命中界面
                // 与从 MOTION_BLOCKING 顶起扫相同 → 不依赖 heightmap，**不引入 scout-b 两桶塌缩
                // 依赖**（placement.rs:311/369 塌缩修正属批次 B）。
                // @anchor.idk("block_at 越界读返回 -1 时按非可生成面处理（保守拒绝）；Java 读邻 chunk
                //  实况。本 modifier k/l 恒在 chunk 内、整列可读，实际不可达，但依赖 worldgen_handle
                //  block_at_col 的越界语义保持 -1", source="memory:batch0-worker-delivery.md §④")
                let Some(block_at) = ctx.block_at else { return vec![]; };
                let bottom = ctx.min_y;
                let top = ctx.min_y + ctx.height - 1;
                let mut out = Vec::new();
                let mut layer = 0i32;
                loop {
                    let mut found_any = false;
                    for _ in 0..count.get(random) {
                        let k = random.next_int_bound(16) + x;
                        let l = random.next_int_bound(16) + z;
                        if let Some(n) = find_multilayer_pos(block_at, k, top, l, bottom, layer,
                                                             *water_id, *lava_id, *bedrock_id) {
                            out.push([k, n, l]);
                            found_any = true;
                        }
                    }
                    if !found_any { break; }
                    layer += 1;
                }
                out
            }
        }
    }
```

### Patch 9 — placement.rs find_multilayer_pos 自由函数（impl PlacementModifier 块结束后、`pub fn parse` 之前插入，L388 附近）

**new**（新增，无 old；放在 `impl PlacementModifier` 的 `}` 之后）：
```rust
/// CountMultilayerPlacementModifier.findPos（Java :65-85）精确移植：
/// 从 (x, top, z) 起逐格下扫到 bottom+1，数「上方可生成（air/water/lava）、自身实体非基岩」
/// 的界面，第 target 个界面的空气位 y；不足返回 None（Java Integer.MAX_VALUE）。
fn find_multilayer_pos(
    block_at: &dyn Fn(i32, i32, i32) -> i32,
    x: i32, top: i32, z: i32, bottom: i32, target: i32,
    water_id: i32, lava_id: i32, bedrock_id: i32,
) -> Option<i32> {
    let air = crate::blocks::AIR;
    let spawns = |id: i32| id == air || id == water_id || id == lava_id;
    // -1（不可读）不满足 spawns → 保守不生成；state2 == bedrock 排除（Java :73）
    let mut prev = block_at(x, top, z);
    let mut found = 0i32;
    let mut j = top;
    while j >= bottom + 1 {
        let cur = block_at(x, j - 1, z);
        if !spawns(cur) && spawns(prev) && cur != bedrock_id {
            if found == target {
                return Some(j); // Java mutable.getY()+1 = (j-1)+1 = j
            }
            found += 1;
        }
        prev = cur;
        j -= 1;
    }
    None
}
```

### Patch 10 — placement.rs unknown modifier 告警接入哨兵（L449-451，可选但建议）

**old**（placement.rs:449-451）：
```rust
        // —— 尾部：未知 modifier 显式告警（b2 S2，消除静默丢弃）——
        eprintln!("[feature-loader] unknown placement modifier type: {type_name}");
        None
```

**new**：
```rust
        // —— 尾部：未知 modifier 显式告警（b2 S2，消除静默丢弃）——
        eprintln!("[feature-loader] unknown placement modifier type: {type_name}");
        // batch0 哨兵：mod: 前缀与 configured type 命名空间区分
        crate::feature_loader::record_unknown_type(&format!("mod:{type_name}"));
        None
```

### Patch 11 — worldgen_handle.rs features 阶段末打汇总（L1184-1185）

**old**（worldgen_handle.rs:1184-1186）：
```rust
        }
        if ca_log {
            eprintln!("[CA] chunk({},{}) out_reads={} pending_writes={} placed={}", cx, cz,
```

**new**：
```rust
        }
        // batch0（mc-1216）：features 阶段结束 unknown-type 汇总（进程级去重集合，
        // 每 chunk 一次调用；env 门控 OnceLock 进程级读一次，热路径近零成本）
        crate::feature_loader::report_unknown_types();
        if ca_log {
            eprintln!("[CA] chunk({},{}) out_reads={} pending_writes={} placed={}", cx, cz,
```

---

## ④ 静态自检清单

| 项 | 检查结果 |
|---|---|
| 类型宽度 | 全部 i32（block id / 坐标），与 ctx/block_at_col 返回 i32 一致；无 usize 混用；`(lz*16+lx) as usize` 模式未新增（不走高度图数组） |
| move/借用语义 | `ctx.block_at` 是 `Option<&'a dyn Fn>`，`let Some(block_at) = ctx.block_at` 复制的是引用（Copy），无 move 冲突；`find_multilayer_pos` 只借 `&dyn Fn`，不捕获 random；water/lava/bedrock id 以值存于变体（parse 期 `blocks.id()` 解析，get_positions 无 BlockRegistry 依赖） |
| panic/throw 路径 | 无 unwrap/expect/索引越界：`set.lock()` 均 `if let Ok`（poison 容忍）；do-while 循环终止性：`count.get` 为 Constant 时每层 0 次尝试 → found_any=false 立即停；Uniform 等每层消费 RNG 但 layer 增长受列内界面数硬上界（≤height），layer 超界面数后 find_pos 恒 None → found_any=false 停。无死循环面 |
| 与 Java 逐行对拍点 | ① getPositions do-while（CountMultilayer:40-55）↔ Rust `loop { … if !found_any break; layer+=1 }`；② `count.get(random)` 在每层内层 for 头（:43，每层消费一次）↔ Rust `for _ in 0..count.get(random)`；③ `nextInt(16)+pos.getX()/getZ()`（:44-45）↔ `next_int_bound(16)+x/z`；④ findPos `mutable.setY(j-1)` + 条件（:70-82）↔ Rust while j>=bottom+1 中 cur=block_at(j-1)、`found==target` 返回 j；⑤ blocksSpawn = air/water/lava（:87-89）↔ spawns 闭包（id 由 parse 期解析，语义等价）；⑥ bedrock 排除（:73）↔ `cur != bedrock_id` |
| RNG 消费序 | 见 ⑤ |
| catch-all 行为变化 | parse 侧原已告警（新增计数，行为兼容）；generate 侧原静默 false → 新增 eprintln+计数。误捕类型（forest_rock 等）从「错误配置+错误生成」变为「不生成+告警」——**这是有意的语义修正**，对齐引擎现有 catch-all 风格（feature_loader.rs:83） |
| 环境单线程/并发性声明 | apply_features 可经 CoreSwapPool 多线程；哨兵用 `OnceLock<Mutex<BTreeSet>>`，锁仅在 unknown 稀疏路径获取，无热路径争用；env 门控为进程级 OnceLock 单次读（对齐 placement.rs:283 treediag_enabled 模式，符合「诊断门控 chunk 级一次」铁律——report 是每 chunk 一次、门内一条 eprintln） |

## ⑤ RNG 消费序影响声明（隔离性）

- **受影响链 = 且仅 = JSON placement 含 `minecraft:count_on_every_layer` 的 placed feature 链**（旧被 `contains("count")` 吸进 Count 分支：消费 count.get 一次 → 同点复制 n 份；新实装：逐层 count.get + 每点 2×nextInt(16)）——消费序完全不同，但对其他 feature 的 RNG 流零影响（各 placed feature 独立 `set_decorator_seed(l,p,k)`，setDecoratorSeed 重置流，见 feature_loader.rs:1079）。
- **分发精确化（Patch 1/3）**：误捕类型（forest_rock、nether_forest_vegetation 等）原先错误消费 ore/disk 的 RNG 并放置方块；修正后不再生成。**这些链的 RNG 消费归零，只影响原走错分支的链本身**；正确分支（ore/disk/spring/…）由 contains→== 语义等价替换（同一集合内 `contains("ore")` 为真当且仅当 type 恰为 ore/scattered_ore），RNG 流不变。
- **结论：隔离性成立**——全部 RNG 变化封闭在「原本就走错的链」内；正确 feature 的逐位输出不变（待 block_probe 回归确认，本交付未运行验证）。

## ⑥ @anchor.idk 汇总

1. **heightmap MOTION_BLOCKING**：本实装不依赖 heightmap（列顶下扫等价性论证见 Patch 8 注释），因此**不构成**对 placement.rs:311/369 两桶塌缩的依赖；塌缩修正仍属批次 B。若 judge/复核对等价性论证有疑，回退方案 = 给 ctx 增 MOTION_BLOCKING 桶（批次 B 一起做）。
2. **block_at 越界语义**：本 modifier 的 k/l 恒在本 chunk 内（nextInt(16)+chunk 内 x），整列可读，越界分支实际不可达；idk 标注见 Patch 8。
3. **noise_threshold_count**：1.20.1 存在（PlacementModifierType.java:19-21）但引擎未实装，落入 unknown 告警——进预期残差清单，本批次不实装。
4. 1.20.1 数据集中 count_on_every_layer 的实际使用链清单：**未逐一核对 JSON**（需主会话在数据目录 grep 确认，用于验收对账）——@anchor.idk("具体链数未核", source="memory:batch0-worker-delivery.md §⑥")。

## ⑦ 错误/踩坑记录（五段式，本批次静态发现，供错误台账归口）

- **现象**：`minecraft:forest_rock` 被 `contains("ore")` 分发进 OreFeatureConfig（"f-ore-st" 子串），配置全错无告警；`count_on_every_layer` 被 `contains("count")` 吸进 Count 分支。
- **根因**：用子串包含做注册名分发，而 MC 注册名空间里 `ore`/`count` 是高频短子串（forest_rock、nether_forest_vegetation 均含 "ore"）；且 parse 与 generate 两套独立分发只错不改告警。
- **定位**：主会话 contains 分支全量排查 + 1.21.6 Feature.java:27-121 / PlacementModifierType.java:8-30 注册名逐一核对。
- **修复**：本交付 Patch 1-11（精确匹配 + catch-all 显式告警 + unknown 哨兵 + count_on_every_layer 实装）。
- **教训**：注册名分发必须用完整 `== "minecraft:xxx"`（引擎 L399 起已有此风格，本批次把剩余 contains 分支全部对齐）；新增类型时 catch-all 哨兵集合就是回归验收面。

**速查表行**：`contains 子串分发误捕（forest_rock→ore / count_on_every_layer→count）| 注册名含高频短子串 | contains 分支全量排查+一手注册表核对 | 精确匹配+catch-all 告警+哨兵 | 新分发一律 == 完整注册名`。

## ⑧ 产出自检（SUBAGENT-KNOWLEDGE-GUIDE §四）

价值门：本交付 = patch 交付物（过程性 .investigations/ 载体，主会话可写层）+ ⑦ 错误记录（高价值，待主会话归口错误台账/时间线）；无低价值结论写 docs；未编造数字；@anchor.idk 均具体。
