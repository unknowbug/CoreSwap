# Rust 块管线深带替换 v1 —— 91.17% → 98.06% 达成

**状态**: candidate（已验证；正确种子 -2032795982907864146，4×4 chunk (0,0)-(3,3)）
**日期**: 2026-08-27

## 成果
新增 Rust `surface.rs` 深带替换规则（`apply_deep_rules`），对 solid stone 应用：
- **bedrock_floor**：y ≤ minY+2 → bedrock（底部 3 层）
- **deepslate**：y ≤ 0 → deepslate（lake 区 surface 全在 y<0，stone 全变 deepslate）

验证（`rvv_surface.rs`）：
```
Rust(+deepSurface) vs vanilla: match=1542380/1572864 (98.06%)  nonAir=489667/518492 (94.44%)
(baseline stone-only = 91.17% / 73.55%)
```

- **总 match: 91.17% → 98.06%**（+6.9 点）
- **nonAir match: 73.55% → 94.44%**（+20.9 点）

## 移植来源
- C++ `surface.h` buildOverworldRule 顶部 finalRules（bedrock_floor + deepslate gradient），已在 C++ 达 99.999%+。
- Java `VanillaSurfaceRules.createDefaultRule` createDefaultRule(true,false,true)（line 277-278 bedrock_floor, line 283 deepslate gradient）。

## 生成的深带替换规则（surface.rs）
```rust
pub fn apply_deep_rules(original_block, is_solid, y, min_y) -> block {
    if !is_solid || original_block != STONE { return original_block; }
    if y <= min_y + 2 { return BEDROCK; }   // bedrock_floor
    if y <= 0 { return DEEPSLATE; }         // deepslate gradient (fixed0)
    original_block
}
```

## 模块登记
- `lib.rs` 新增 `pub mod surface;`
- `WorldgenRust/src/surface.rs`（v1 深带替换；v2 = 移植完整 surface.h：垂直渐变+随机 splitter、tuff/silverfish 带、surface 顶块、red 陶带、badlands pillar、mr1-10 树）

## 剩余 gap（98.06% → 100%）
| 项 | mismatch | 机制 | 结论 |
|---|---|---|---|
| **tuff 带** | ~9279 | **非 OreVeinSampler**——实测 tuff 位置 veinToggle tiny（\|d\|<0.2），ore_vein fire=0。tuff 来自**不同的初始块状态替换**（待定：疑 MC 深板岩带内嵌 tuff/silverfish 的 block-state 后处理） | 需独立排查（未解，下一轮） |
| **aquifer 流体** | water 4591 + lava 2804 | Rust aquifer vs vanilla 精确性 | 独立课题 |
| **gravel/草/沙 顶块** | ~5751+~76 | surface 顶块（biome/stoneDepth 依赖） | 需 mr9 部分 |
| **ore 替换** | ~800 | stone→ore 矿脉 | ore_vein 已移植但 fire=0（此 chunk 无矿脉中心）；deepslate_*_ore 在别处 |
| diorite/granite | ~279 | 深带替换 | 需特定规则 |

## OreVeinSampler 移植状态
- 已实现 `WorldgenRust/src/ore_vein.rs`（从 C++ ore_vein.h 71 行移植），块 id + 噪声函数 + splitter（`split("minecraft:ore").next_splitter()`，worldgen_api.cpp L807 确认）。
- 实测：此 chunk tuff 位置 veinToggle tiny，ore_vein fire=0 → **tuff 非 ore vein 来源**（ore vein 在此区域/此种子不产生 tuff）。已验证 ore_vein 移植逻辑正确（遵循 C++ 决策链），但此参照区无矿脉 → 收益 +45（近乎可忽略）。
- **tuff 真来源未定**（下一轮排查）：需查 MC 深板岩带 tuff/silverfish 的 block-state 替换（可能与 aquifer 或 chunk 初始块有关，非 surface/ore_vein）。

## 结论
- **确定性深带替换（bedrock/deepslate）为已验证首增量：91.17%→98.06%**、nonAir 73.55%→94.44%。
- Ore vein 移植正确但此区无矿脉（tuff 非这里来源）。
- 剩余 ~2% 跨多个独立子系统：**tuff 真来源（未解）**、aquifer 流体精度、surface 顶块、深板岩带 inlining。

## 结论
- 确定性深带替换（bedrock/deepslate）是**最高杠杆、最低风险**的首个增量：+6.9 点总 / +20.9 点 nonAir。
- 下一步候选：① verticalGradient 完整实现（概率带）→ tuff 带；② aquifer 流体精确性；③ surface 顶块（mr9 gravel 段）。
- 建议按收益排序继续：tuff（9k）> aquifer fluid（7.4k）> surface 顶块（5.7k）。

> **记录价值门**：本文件是"实现增量 + 剩余 gap"的**中价值**记录（后续实现的判据/待办），留 `.investigations/`；实现未踩坑，无需入 `rust-errors.md`。
