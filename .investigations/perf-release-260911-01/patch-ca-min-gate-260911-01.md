# Patch 草稿 —— ca_min 缓存门控追加「features 未跳过」条件（260911-01）

> 状态：**draft，未编译验证**（本 subagent 无命令权限；主会话负责 `cargo build --offline -p worldgen --release` 编译 + 验证）。
> 目标文件：`worldgen-core\src\worldgen_handle.rs`，`fill_chunk_blocks`（613-678 行）。
> 依据：260910-08 块实测——出厂 `stageMask=3` 时 features/carver 均跳过，terrain_cache 只写不读，每 chunk 纯付 ~393KB col clone（hit）或 clone+insert（miss）。
> 语义约束：features 启用路径（含 WG_CA_MIN=0）逐位不变；仅新增「features 跳过 ⇒ 不走缓存，直接 fill_terrain_column」。

---

## 一、修改前后代码对照（精确到行，基于 260911-01 读到的现状）

### 1.1 修改前（行 619-642）

```rust
619:        let ca_min = std::env::var("WG_CA_MIN").map(|v| v != "0").unwrap_or(true);
620:        let (mut col, heightmap) = if ca_min {
621:            let hit = self.terrain_cache.lock().ok()
622:                .and_then(|c| c.get(&(cx, cz)).cloned());
623:            match hit {
624:                Some(e) => ((*e.col).clone(), (*e.heightmap).clone()),
625:                None => {
626:                    let (c, hm) = self.fill_terrain_column(cx, cz);
627:                    let entry = CaTerrainEntry {
628:                        col: std::sync::Arc::new(c.clone()),
629:                        heightmap: std::sync::Arc::new(hm.clone()),
630:                    };
631:                    if let Ok(mut cache) = self.terrain_cache.lock() {
632:                        let cap = Self::ca_cap();
633:                        if cache.len() >= cap { cache.clear(); }
634:                        cache.insert((cx, cz), entry);
635:                    }
636:                    (c, hm)
637:                }
638:            }
639:        } else {
640:            self.fill_terrain_column(cx, cz)
641:        };
642:        let flags = self.flags.load(std::sync::atomic::Ordering::Relaxed);
```

### 1.2 修改后（同一区域）

```rust
        let ca_min = std::env::var("WG_CA_MIN").map(|v| v != "0").unwrap_or(true);
        // 260911-01：flags 提前 load + skip_features 判定同源提取（原 669 行判法，逐字一致）。
        // stageMask=3（features 跳过）时 terrain_cache 只写不读（消费方在 apply_features 邻 chunk 读），
        // ca_min 缓存路径纯成本（260910-08 实测 ~393KB col clone/chunk）——追加门控：仅 features
        // 实际启用时走缓存；features 跳过时与 else 分支同为直算 fill_terrain_column，零缓存开销。
        let flags = self.flags.load(std::sync::atomic::Ordering::Relaxed);
        let skip_features = flags & FLAG_SKIP_FEATURES != 0 || std::env::var("WG_SKIP_FEATURES").is_ok();
        let (mut col, heightmap) = if ca_min && !skip_features {
            let hit = self.terrain_cache.lock().ok()
                .and_then(|c| c.get(&(cx, cz)).cloned());
            match hit {
                Some(e) => ((*e.col).clone(), (*e.heightmap).clone()),
                None => {
                    let (c, hm) = self.fill_terrain_column(cx, cz);
                    let entry = CaTerrainEntry {
                        col: std::sync::Arc::new(c.clone()),
                        heightmap: std::sync::Arc::new(hm.clone()),
                    };
                    if let Ok(mut cache) = self.terrain_cache.lock() {
                        let cap = Self::ca_cap();
                        if cache.len() >= cap { cache.clear(); }
                        cache.insert((cx, cz), entry);
                    }
                    (c, hm)
                }
            }
        } else {
            self.fill_terrain_column(cx, cz)
        };
        // （原 642 行的 flags load 删除——已提前，复用同一变量）
```

### 1.3 后续行联动修改

| 行 | 修改前 | 修改后 |
|---|---|---|
| 642 | `let flags = self.flags.load(std::sync::atomic::Ordering::Relaxed);` | **整行删除**（变量已提前定义，保留会 E0428 重复声明编译错） |
| 642-646 注释 | 保留原 260910-04 注释块 | 保留不动（CONF_ECHO 语义注释仍有效） |
| 651-659 CONF_ECHO | 打印 `ca_min`、`flags` | **不动**（`ca_min` 打印的是 env 判定值，见 §三声明） |
| 669 | `let skip_features = flags & FLAG_SKIP_FEATURES != 0 \|\| std::env::var("WG_SKIP_FEATURES").is_ok();` | **删除**（同源判定已提前；保留会 duplicate variable，且两处分离正是本次要消除的双判源） |
| 670 | `if !skip_features {` | 不动（复用提前定义的同名变量） |

净效果 diff（三处）：
1. 行 619 之后插入：flags 提前 load + `skip_features` 同源判定（含注释）；
2. 行 620 门控 `if ca_min` → `if ca_min && !skip_features`；
3. 删除原行 642 的 flags load 与原行 669 的 skip_features 声明（均由提前定义的单份替代）。

---

## 二、逐行对拍点清单（与 Java / 既有行为）

| # | 对拍点 | 现状 | patch 后 | 依据 |
|---|---|---|---|---|
| P1 | skip_features 判法与 fill_features 门控（行 669/670）同源 | 行 669 独立判定 | 同一表达式**逐字提取**到提前位置，669 行删除复用 | Java `GenerationChunkHolder`/`ChunkGenerator.generateFeatures` 的 stageMask 跳过语义 = flags 位 OR env 诊断覆盖；同源即零漂移 |
| P2 | flags load 时序 | 行 642 在缓存命中判定**之后** load | 提前到行 620 之前 load，Ordering::Relaxed 不变 | flags 在 chunk 生成中途理论可被外部改（Relaxed 竞口）；现状两处 load 本就允许读到不同值，patch 只是把「缓存判定用哪次 load」固定为早 load——与行 670 features 判定同一次读，**消除**了「缓存按旧 flags 走、features 按新 flags 判」的既有潜在不一致 |
| P3 | features 启用路径（WG_CA_MIN=1 或 0，无 stageMask skip） | hit→clone / miss→fill+insert | 完全不变（`!skip_features` 为 true，门控退化为原 `ca_min`） | 逐位不变要求 |
| P4 | features 跳过路径 | 仍走缓存（hit clone ~393KB / miss clone+insert） | 直算 `fill_terrain_column`，与原 else 分支同路径 | 260910-08 实测：缓存消费方仅在 features/carver 阶段邻读；跳过时无读方，col 值逐位相同（cache 条目本就是同一函数产物） |
| P5 | carver 单跳（stageMask 含 FLAG_SKIP_CARVER 但不含 SKIP_FEATURES） | 走缓存 | 仍走缓存（门控只看 SKIP_FEATURES） | 有意收窄：features 启用即有邻读消费方，缓存仍有收益；carver 语义已含在 fill_terrain_column 内 |
| P6 | CONF_ECHO 首次打印（行 651-659） | `ca_min=` 打 env 判定值 | 不变 | 见 §三声明 |
| P7 | heightmap 产物 | 缓存命中与直算产物均出自 `fill_terrain_column` 同一实现（行 616-618 注释：逐位一致） | 不变 | E1b 已验证缓存条目 = 同一函数产物 |
| P8 | WG_CA_MIN=0（关缓存） | else 直算 | 不变（ca_min=false 短路，`&&` 右侧不求值——但 skip_features 无副作用，语义等价） | 行为保持 |

---

## 三、静态自检清单

① **类型宽度**：无新增数值类型。`flags: u32`（`FLAG_SKIP_FEATURES: u32 = 1 << 1`，行 149），`flags & FLAG_SKIP_FEATURES != 0` 位宽一致，无溢出/截断点。返回 `(BlockColumn, Vec<i32>)` 元组类型不变。

② **move/克隆语义**：`skip_features` 为 `bool` Copy，无 move 影响。两个分支返回类型一致（`(BlockColumn, Vec<i32>)`）——`ca_min && !skip_features` 不改变 match 臂结构，col/heightmap 的 clone/所有权流向逐字未动。`self.terrain_cache.lock()` 用法（`.ok()` 吞 poisoned、`if let Ok` 吞锁失败）原样保留——锁语义零改动。

③ **锁/panic 路径**：无新增锁、无新增 unwrap/expect。`std::env::var` 提前一次（skip_features 判定处）——env 查询从每 chunk 1 次（行 669）变为 1 次（提前处），总次数不变（669 行删除抵消）；非每点查询，无热路径污染（符合「诊断门控 chunk 级判断一次」铁律）。flags 提前 load 无 panic 面。

④ **与原逻辑对拍**：见 §二 P1-P8。核心不变量——
   - 真值表：`(ca_min, skip_features)` = (任意, false) → 原行为逐位不变；(true, true) → 直算（原 else 路径）；(false, true) → 直算（原 else 路径）。即「cache 路径可达集」收窄 ⊆ 原可达集，收窄部分全部落到与 miss-body 同实现的直算，输出值域不变。
   - 缓存内容（CaTerrainEntry 写入逻辑、ca_cap 淘汰）逐字未动；features 启用时读方仍可命中（含 features 启用但其它 chunk 曾在 skip 期写入的条目——条目本就是 fill_terrain_column 产物，逐位一致，无污染）。

**CONF_ECHO 日志读法声明**（要求 3）：patch 后日志中 `ca_min=` 仍打印 **env 判定值**（WG_CA_MIN，默认 true），**不含**本次新增的 skip_features 门控——即日志 `ca_min=true` + `skip_features=true` 时实际未走缓存路径（门控 = `ca_min && !skip_features`）。日志的 `flags=`/`skip_features=` 字段足以还原实际门控状态；未改 CONF_ECHO 打印内容与一次性 swap 语义（行 649-650 的 `CONF_ECHOED` 原样）。

---

## 四、显式声明

**本 patch 未编译验证、未运行验证**（subagent 无命令权限，Degraded：纯静态审查）。主会话应用后需：
1. `cargo build --offline -p worldgen --release`（预期风险点：行 642/669 删除是否彻底，若漏删会出现 unused/重复声明编译错）；
2. A/B 验证建议：`stageMask=3`（出厂态）下对比 patch 前后吞吐 + `[WG-CONF]` 输出；features 启用态（`-PstageMask=0` 或无 mask）跑回归确认逐位不变（blocks 对比）。
