# residual76-verdict-260904-10 — 残留 76 surface 微族收口 verdict

- **课题**：260904-10 残留 76（aquifer splitter 修复后的 surface 残差：gravel→sand 49 / sand→gravel 24 / 零星 3）
- **状态**：**candidate**（confirmed 待用户拍板）
- **验证分层**：Full（存档口径 blocks 全量逐位，见 §6 可比性声明）
- **日期标签**：260904-10（锚宿主真实时间 2026-09-04 21:26-22:0x，Get-Date 复核 ✓）

## 1. 结论

### 1.1 根因（主族 73 块）
Java surface 阶段的 biome 判定入口携带 **BiomeAccess**：
- `ChunkRegion` 构造时以 `BiomeAccess.hashSeed(seed)`（sha256-asLong）派生 access seed（一手源码 ChunkRegion.java L102）；
- surface 规则采样点为精确块坐标（MaterialRules.java L464：`posToBiome.apply(this.pos.set(blockX, blockY, blockZ))`），biome 查询经 BiomeAccess.getBiome 8 邻域 zoom + hash jitter；
- **Rust surface 实装用 4x4x4 单元直读，漏掉 BiomeAccess zoom 层**——biome 边界带（deep_ocean↔deep_lukewarm）选点不同 → gravel/sand 双向漂移。
- C++ 侧 `biomeCellKey`（worldgen_api.cpp L1080）4 站点全部 hashed 已对齐——**Rust↔C++ 实装分歧，非跨载具共性缺陷**。

### 1.2 修复
- commit **ab56706**：`biome_at_surface`（biome_pick_cell + biome_hash_seed）仅接 `build_surface`；picked-cell packed 缓存 key（u32 截断，对齐 C++）；`biome_hash_seed` = sha256 移植（biome.rs，C++ biome.h biomeHashSeed 同构）。

### 1.3 验证数字（decisive，全新 world 重导）
- 残差 **76 → 12**（99.999% 对齐）
- sand→gravel 24 → **0（全闭合）**；gravel→sand 49 → **9**；零星 3 → 3（归因 §4）
- 现役 dll 基线：5E2ACB7F → **6B31129E**（resources 已同步同哈希）

## 2. 证据链

| 步骤 | 内容 | 产物 |
|---|---|---|
| scout | D-1~D-6 + .b1~.b5 候选 | `.investigations/residual-1830/scout-residual76-260904-10.md` |
| 盘上双臂 dump | 76 签名：dtop 全 0（❌.b1）/ biome 通道双侧同（❌.b3）/ 无 rev()（❌off-by-one） | `.tmp/p2full/residual76-dump-260904-10.txt` + `residual76_dump.py` |
| fan-out .b2 | 噪声 patch 静态排除（VanillaSurfaceRules.java L263-270 深海分支无噪声条件） | `.investigations/residual-1830/candidate-76-b2-noisepatch-260904-10.md` |
| fan-out .b4 | 腔底语义覆盖不了 3 块（❌），副产物炸出 docs/06 口径失准 | `.investigations/residual-1830/candidate-76-b4-cavefloor-260904-10.md` |
| P0 一手核对 | SurfaceBuilder.java L181-183 isDefaultBlock = 非空非流体（≠docs/06 L62/L94）；Rust/C++ 本就一致 | `.investigations/residual-1830/cmd-output/java-sscan-source-SurfaceBuilder-1.20.1.txt` |
| res76_probe | pre/post/biome/r/sda/sdb/applied 七通道；零星 3 块归因 ore_vein/aquifer 域 | `.tmp/p2full/res76-probe-260904-10.csv`（驱动 `WorldgenRust/src/bin-diag/res76_probe.rs`） |
| 修复 | commit ab56706 | git |
| decisive | 全新 world 重导 76→12 | `.tmp/p2full/off-surfacefix-260904-10/` + `.investigations/residual-1830/cmd-output/verify-surfacefix-260904-10.out.txt` |
| judge | 建议 candidate（C-1/C-2 已补） | `.investigations/residual-1830/review-residual76-260904-10.md` |

被排除假说：❌ .b1 列顶级联（dtop 全 0）　❌ .b3 biome 记录差（双侧同）　❌ off-by-one rev()（无此代码）　❌ .b2 噪声 patch（分支无噪声条件）　❌ .b4 腔底语义（覆盖不了 3 块）　❌ isDefaultBlock 口径差（本就一致）

## 3. 派生产物：docs/06 口径失准（待 docs 修正）

docs/06 L62/L94 记 surface default 判定「==stone」；一手 SurfaceBuilder.java L181-183 实为「非空非流体」。二手转述失准（不影响本轮结论——Rust/C++ 实装一致）。**建议 docs/06 追加修正小节，不覆盖原文**。

## 4. 剩余 12 归因

| 残差 | 数量 | 归因 | 状态 |
|---|---|---|---|
| granite→gravel | 1 | ore_vein 域（(237,224,41) CS pre=stone，vanilla 有 granite——ore_vein 漏放） | 归因已明，未立案 |
| water→dirt + gravel→water | 2 | aquifer 域固/液边界差一格（(198,18,18)/(237,42,224) pre 态差） | 归因已明，未立案 |
| gravel→sand | 9 | 未立案 | 🔍 待新课题 |

## 5. open 疑点（未动）

**Rust carver/feature 侧 `biome_pick_cell(self.seed)` 用裸 seed 非 hashSeed**（worldgen_handle.rs L722/849 一带）——C++ 同功能 4 站点全部 hashed。Rust↔C++ 实装分歧，**风险：中**（与本次根因同族）。judge 建议下轮换 seed 回归时低成本 A/B。对应知识库发现 #44。

## 6. 验证可比性声明（§9.7）

载体 = 存档口径 blocks 全量逐位（主载体）；覆盖面 = 全新 world 重导残差全集；与 1830→76 收口同载体同口径同 seed 流程**直接可比**。res76_probe 七通道为探针口径仅作归因，不与存档口径数字混用。

## 7. 置信度

根因/修复/验证数字 **candidate**（judge 已过）；confirmed 待用户拍板。剩余 12 三项归因 candidate（探针级）。
