# 结论 docs 草稿：Rust worldgen 整体功能实现（功能链路 + 对齐快照 95.40%）

> **状态**：draft（本文件是知识库 subagent 产出的草稿，待主会话应用 + 验证后定稿）。
> **载体建议**：结论进 `versions/1.20.1/docs/07-block-pipeline.md` 或 `versions/1.20.1/docs/11-features-stage.md` **末尾追加小节**（追加不覆盖）；过程进 `versions/1.20.1/docs/10-timewise-archive.md`（时间线追加）；错误台账已独立成篇 `.investigations/rust-mod-load/functional-errors.md`（F1-F3 五段式）。
> **价值门**：
> - 架构/链路（功能链路完整接进生成管线 + 零锁设计）= **中价值（简记）**——跨模块通用模式，记"是什么"；
> - 锁清理（生成路径零锁设计）= **中价值（简记）**——可复用并发模式（写少读多 RwLock / 只读预加载 / clone 降持锁跨度）；
> - **对齐率 95.40%（对齐快照）= 低价值（快照）**——**用户明确「先整体功能实现 + 只记录对齐程度不纠结」，故记录但不作主内容**；数值一次性，排查时参考 `.investigations/rust-mod-load/cmd-output/`。

---

## 一、本 session 功能实现（整体功能链路，中价值简记）

> Rust worldgen 从「块级管线跑通」到「FEATURES 功能真正接进生成管线」。功能链路完整闭合，见 §二 提交映射。用户指令：「先整体功能实现 + 跑测试记录对齐程度，不纠结为什么没对齐」。

1. **apply_features 补 OCEAN_FLOOR_WG 高度图**（09d85e8）：每列从顶向下扫，跳过 air/water/lava，取第一个固体 y（海底/地表）= OCEAN_FLOOR_WG；`FeatureContext.ocean_floor` 由 `None` 改 `Some(&ocean_floor)`。让水下 ore/disk/spring 能按海洋底部实际高度放置（错误台账 F2）。
2. **ore_vein 矿脉接入生成管线**（79daf17）：`create()` 构建 vein_toggle/ridged/gap DF + `split("minecraft:ore")` splitter；`fill_chunk_blocks` 在 rock 分类处用 `ore_vein.apply` 替换为铜/铁矿脉块；`apply` 从 `&mut self` 改 `&self`（只读，无需锁，错误台账 F1）。
3. **Beardifier 结构密度修正接入**（a6a53f7）：`beardifiers` 从 `Mutex` 改 `RwLock`（写读分离，读并发无争用）；`fill_chunk_blocks` 读当前 chunk beardifier 后 clone 传给 fill_chunk（不持锁跨调用，错误台账 F3）。
4. **wg_fill_density 实现**（4ac3a00）：`WorldgenHandle.fill_density` 按 `size×size` chunk、`POINTS_PER_CHUNK` 网格做 finalDensity 采样；`api.rs wg_fill_density` 拷到 C out buffer（fillDensity API）。
5. **生成路径锁清理**（ed59f50）：`feature_indexer` Mutex→只读共享（建一次，`&self` 并发）；`carver_cache`/`feature_cache` Mutex 懒加载→create 时预加载只读。**生成路径（fill_chunk_blocks）实现零锁**（仅 beardifiers set/clear 外部写）。

## 二、提交映射（过程定位）

| 提交 | 功能 | 载体/错误 |
|---|---|---|
| ed59f50 | 锁清理（生成路径零锁） | 结论（中价值） |
| 09d85e8 | OCEAN_FLOOR_WG 高度图 | 错误台账 F2 |
| 79daf17 | ore_vein 矿脉接入 | 错误台账 F1 |
| a6a53f7 | Beardifier 接入（RwLock | 错误台账 F3 |
| 4ac3a00 | wg_fill_density | — |

## 三、功能验证结果（低价值对齐快照，用户指示「只记录不纠结」）

> 对齐率 = **低价值快照**——**主会话明确「先整体功能实现 + 只记录对齐程度，不纠结为什么没对齐」**，数值仅记录不作主内容、不展开分析。验证记录见 `.investigations/rust-mod-load/cmd-output/`。

| 验证 | 结果 | 说明 |
|---|---|---|
| **features_probe（完整管线）** | **match 95.40%** / nonAir 85.84% | 整体对齐快照（用户指示只记录） |
| vein_probe（矿脉） | 2295 矿脉块（**1849 铜 + 19 生铜 + 427 深板岩铁**） | 功能验证（矿脉接入生效） |
| fill_density_probe | **3072 点全部非零** | 功能验证（finalDensity 网格采样生效） |

## 四、关键设计语义（中价值，可复用）

- **「并发生成路径零锁」设计（高价值判断链沉淀，详见错误台账 F1-F3）**：
  - ① `&mut` 方法体实际只读 → 改 `&self`（F1，签名谎报可变性 = 隐性锁来源）；
  - ② Option 高度图 None → 还原 Java 哨兵回退（F2，getOceanFloorTopY 返回 min_y-1）；
  - ③ 「低频写 + 高频并发读」用 RwLock 而非 Mutex（F3，读共享无争用）；
  - ④ 持锁跨度最小化：读出来 clone 释放，不跨大函数长持有（F3）。
- **写少读多用 RwLock / 只读预加载 / clone 降持锁跨度** = 并发生成路径的三件套可复用模式。

## 五、域/边界（必须写明）

- 验证分层 = **Partial**（探针对比 vanilla FULL 参照，非逐位 Full）；对齐率 95.40% 为当前快照，**用户指示「只记录不纠结」，不展开差异分析**。
- Beardifier 接入后探针无 beard 数据 → 对齐率不变（95.40%），即 Beardifier 接入**未改变当前探针对齐结果**（探针场景无结构区）。
- 矿脉验证：`vein_probe` 2295 块（铜/生铜/深板岩铁）证明矿脉替换生效（功能链路闭合），**对齐率是否对齐未深究（用户指示只记录）**。

## 六、排除清单（❌ 一行式）

- ❌ 「Beardifier 接入引入生成路径锁」——改 RwLock + clone 释放，零锁保持（a6a53f7 + ed59f50）。
- ❌ 「水下 ore/disk/spring 无法放置」——补 OCEAN_FLOOR_WG 后 ocean_floor 提供，功能闭合。

## 七、时间线条目草稿（追加到 10-timewise-archive.md 末尾）

> 载体：`versions/1.20.1/docs/10-timewise-archive.md`（时间线追加，每条带状态标注）。主会话应用时按日期（2026-08-29）追加。

### 2026-08-29（追加）：Rust worldgen 整体功能实现（✅ 关键里程碑）

> 从「Rust 块级管线跑通（mod-run）」推进到「FEATURES 功能真正接进生成管线 + 生成路径零锁」。用户明确「先整体功能实现 + 跑测试记录对齐程度，不纠结为什么没对齐」。配套：07/11 篇追加小节 + `.investigations/rust-mod-load/` + `functional-errors.md` 错误台账（F1-F3）。

### ✅ 一、功能链路完整接入（功能闭环）
- 09d85e8 补 **OCEAN_FLOOR_WG** 高度图（`ocean_floor: None`→`Some`）——水下 ore/disk/spring 按海底放置（F2）。
- 79daf17 接入 **ore_vein 矿脉**（vein_toggle/ridged/gap + `split("minecraft:ore")`；`apply` 改 `&self` 只读，F1）。
- a6a53f7 接入 **Beardifier**（`beardifiers` Mutex→RwLock，写读分离；fill 读 clone 不持锁，F3）。
- 4ac3a00 实现 **wg_fill_density**（finalDensity 网格采样，fillDensity API）。

### ✅ 二、生成路径零锁（perf 优化）
- ed59f50：feature_indexer/carver_cache/feature_cache 全部预加载只读共享；生成路径（fill_chunk_blocks）零锁。

### 🧪 三、功能验证（对齐快照，用户指示只记录）
- features_probe：**match 95.40%** / nonAir 85.84%（整体快照）。
- vein_probe：2295 矿脉块（1849 铜 + 19 生铜 + 427 深板岩铁）。
- fill_density_probe：3072 点全部非零。
- **对齐率只记录不纠结**（用户指令）：数值一次性，不展开差异，排查参考 `.investigations/rust-mod-load/cmd-output/`。

### 🧰 四、工具演进（本轮新增）
- `vein_probe.rs`（矿脉接入验证）、`fill_density_probe.rs`（finalDensity 网格采样验证）、features_probe（完整管线对齐快照）。

### 📌 记录指引（知识库归口）
- 错误台账：`.investigations/rust-mod-load/functional-errors.md`（F1-F3 五段式 + 速查表）。
- 结论：07/11 篇末尾追加「Rust worldgen 整体功能实现」小节（中价值简记 + 低价值对齐快照）。
- 过程：本节 + `.investigations/rust-mod-load/`。
- **价值门**：架构/锁清理 = 中价值简记；对齐率 95.40% = 低价值快照（用户指示只记录不纠结，不作主内容）。
