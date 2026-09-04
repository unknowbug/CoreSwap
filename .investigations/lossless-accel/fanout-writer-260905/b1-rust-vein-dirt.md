# fan-out .b1 — H1：Rust NOISE(vein)/SURFACE 阶段写出界 dirt/sand？

- 状态：**draft**（Degraded：纯静态审查，无运行时探针）
- worker：fanout-writer-260905（b1 候选）
- 结论：**H1 不成立**（高置信，静态证据完整闭环）

## 排查范围与方法

只读静态审查 `WorldgenRust/src`，沿生产 fill 路径 `fill_chunk_blocks`（jni_bridge → wg_fill_blocks）逐阶段追踪所有 `col.at_mut` / `set_block` 写入点，重点查 dirt/sand/gravel/granite 写入与竖列签名（同一列散布 30 个 dirt，y=-41..263）能成立的机制。

## 1. NOISE 阶段（宏观填充 + ore_vein）

写入点 = `worldgen_handle.rs:536-553`（步骤 2，BlockColumn 占位）：

- 每 cell 只写四类：`Air/Stone/Water/Lava`（L541-546，`BlockKind` 映射，`terrain.rs`）。
- 唯一额外写者 = `ore_vein.apply`（L548-551），仅替换 `BlockKind::Rock`。

`ore_vein.rs`（全 69 行）：
- 块表仅两组（L31-32）：copper = `copper_ore / raw_copper_block / granite`（y∈[0,50]）；iron = `deepslate_iron_ore / raw_iron_block / tuff`（y∈[-60,-8]）。
- y 硬门（L46）：`if y < -60 || y > 50 { return -1; }` → y=263 与 y>50 区域 ore_vein 数学上不可能写任何块。
- 块表中 **无 dirt/sand/gravel**；granite 只出现在铜矿脉 stone 兜底（L65），且被同一 y 门约束。
- 与 vanilla VeinType 语义一致（C++ ore_vein.h 71 行移植，注释 L1-2），无块表错位迹象。

**结论：NOISE 阶段零 dirt/sand 写入路径。**

## 2. SURFACE 阶段（surface_rules.rs build_surface）

循环结构（L1253-1343）：全列 `wy` 从 `heightmap+1` 扫到 `min_y` —— **y 域本身覆盖 -41 与 263**（深度方向可达），但 dirt 写入受三重约束：

1. **只替换 default_block（stone）**（L1317-1318）——原块非 stone 不写。
2. 所有 dirt 写入规则都在 `mr7` 内（L720 grove / L743 windswept_gravelly_hills / L753 兜底 dirt），而 `mr7` 的**唯一可达入口** = L987-1001：外层 `mc10 = Water{offset:-6, mult:-1, add_stone_depth:true}`（L612）+ 内层 `StoneDepth{add_surface_depth:true, ceiling:false}`（L999，STONE_DEPTH_FLOOR_WITH_SURFACE_DEPTH）。
3. `StoneDepth` 语义（L100-115 + 循环内 s/q 计算 L1274-1313）：`stone_depth_below` 从该列**连续固体顶部**（s = 第一个非 default 块位置）向下计数，命中条件 ≈ 顶面下 surface_depth（5±）块内的**连续薄层**。

竖列签名不兼容：y=-41..263 散布 30 个 dirt cell 要求同一列出现 30 个互不相邻的「局部固体顶面带 dirt」，且被 air/stone 间隔——surface 规则只能产出每个固体区域顶部一条连续带，不能产出散点。另外该 chunk surface_height 若在 ~263，产出的是 263 附近一条带，不会同时在 -41。

旁支区分：windswept /8.25 阈值分歧（L669/726/734/738/742）只影响**同一 surface 带**内 stone vs dirt 的选择，不扩大 y 域——与本课题无关。
`apply_material_rule_single`（L1349-1391，carver 用）：init_vertical(1,1,…) 固定 stone_depth=1，且调用点 `carver.rs:386-393` 只在「grass 被挖掉后其下方**已是 dirt** 的块」上做规则替换——只能改写既有 dirt，**不能把 stone 变 dirt**。非候选。
`place_badlands_pillar`（L1394+，写 L1447）：只写 default_block（stone），且仅 eroded_badlands biome。非候选。

## 3. 「按列/全 y 写 dirt」模式排查

全 src 的列级/全 y 写入点穷举：
- `worldgen_handle.rs:536-553`（及镜像 `diag_pre_surface_column` L664-679）：四类块 + vein，无 dirt。
- `surface_rules.rs:1336`：rule 门控（见 §2）。
- `surface_rules.rs:1447`：badlands pillar，写 stone。
- `carver.rs:385/392`：carve 点 + grass-dirt 联动，见 §2，不能 stone→dirt。
- `feature.rs` set_block（L316/383/453/529/560/627）：FEATURES 阶段（步骤 5），dirt 可经 ore/disk target 写入，但 (a) 属 FEATURES 非 NOISE/SURFACE，(b) 位置/数量由数据驱动 JSON height_range 决定，(c) 竖列「30 个散布 cell」形态与 ore vein 放置（椭圆簇）和 disk（浅盘）都不符。

**不存在任何「按列全 y 写 dirt/sand」的代码模式。**

## 结论

**H1 不成立。** Rust 侧：
- NOISE/ore_vein 块表不含 dirt/sand/gravel，且 y∈[-60,50] 硬门排除 y>200/y<-32 出界样本；
- SURFACE 的 dirt 写入被 StoneDepth 链约束在局部固体顶面连续薄层，机制上无法产出「同一列 30 个散布 dirt cell、y=-41..263」签名；
- 全库无按列/全 y 写 dirt 的代码模式。

指向修正：出界 dirt 写者应在 **H1 以外**——最大嫌疑转移到 (a) FEATURES 阶段（Java 侧共享，或 Rust feature placement 的 y/origin 计算错位）与 (b) 数据/导出链路（cppReplace 世界导出时 mod 对 feature stage 的裁剪差异使 vanilla feature 产物暴露为残差）。建议 b2+ 候选沿 FEATURES 阶段（含 `worldgen_handle.rs:789+` apply_features 的 OCEAN_FLOOR origin 选择）与导出链路对齐方向展开。

## 证据索引

| 结论 | 文件:行 |
|---|---|
| NOISE 每 cell 四类块 | WorldgenRust/src/worldgen_handle.rs:536-553 |
| vein 块表无 dirt + y 硬门 | WorldgenRust/src/ore_vein.rs:31-32,44-67 |
| surface 全列循环 + default_block 门 | WorldgenRust/src/surface_rules.rs:1253-1343（L1317） |
| mr7 dirt 唯一入口 mc10+STONE_DEPTH_FLOOR | surface_rules.rs:987-1001,612,999 |
| dirt 规则点位 | surface_rules.rs:622,720,743,753 |
| carver dirt 只改写不新建 | carver.rs:381-394; surface_rules.rs:1349-1391 |
| FEATURES 阶段 set_block（旁支嫌疑） | feature.rs:316,383,453; worldgen_handle.rs:619-626,789+ |
