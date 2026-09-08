# verdict-f1f2-chunky-260908-11.md — 1.21.6 移植收口结论（draft → 提交 judge）

> 块 260908-11（实际 2026-09-08 18:27-20:1x）；计划 = 架构计划-260908-11-mc1216-收口.md（已批准）。
> 状态：draft（judge MUST 审查 + 用户 confirmed 前不升级）。

## F2 探针修复（诊断工具，非生产）

- **一手验证**：vanilla `getNoiseBasedFluidLevel`（1.20.1 与 1.21.6 同语义）= `q = base + prnd; return Math.min(surfaceHeightEstimate, q)`；两侧探针 mixin L302 均漏 `+ base`。修复 = `Math.min(surfaceHeightEstimate, base + prnd)`（1.20.1 + 1.21.6 同款）。
- **运行时验证**（部分，降级声明）：链级采样多轮（164 地表柱点 / 1408 深部网格 / 144 空气点 / 28 水洞点）全部未观测到 spread-replay 行（缓存命中/早退主导）。**降级声明（§9）**：spread 路径的字段级手算对拍未达成；F2 验证 = 静态行级对齐（一手源码）+ 运行时 sane（链字段 populated、fl 值与 vanilla 缓存真值一致）。
- **跨执行体旁证**：Rust 臂（自算链，公式含 +base）28 点 fl/decision 与 vanilla 缓存真值逐点一致（差异仅注记粒度 + f32/f64 尾数）→ +base 语义两侧一致。

## F1 water-over-lava 对齐（生产行为改动）

- **一手验证（Java）**：`NoiseChunkGenerator.createFluidLevelSampler` = 静态 lambda `y < min(-54, seaLevel) ? LAVA@-54 : (seaLevel, defaultFluid)`；`AquiferSampler.apply:225` 用该 sampler 查 (i, j-1, k)。Rust 旧代码错用全价噪声链 `get_fluid_level`。
- **修复**：`worldgen-core/src/aquifer.rs` 检查点改为 `FluidLevel::default_level(block_y - 1).get_block_state(block_y - 1) == LAVA`（overworld 判据 = (y-1) < -54）。
- **适用域声明（@anchor.idk 性质）**：Java lambda 含 `min(-54, settings.seaLevel())`——seaLevel≠63 的维度/数据包下 Rust 硬编码判据不适用；Rust 引擎当前 overworld 专用（min_y=-64/height=384 硬编码），非 overworld 接入时须参数化（挂账，多世界数据驱动升级点）。
- **验证（Full）**：
  - `cargo test -p WorldgenRust --lib` 14/14 绿；
  - 1.21.6 P7 R 臂复跑 vs 真参照：**99.9993%（41/6,291,456）/ biome 0** = confirmed 结论逐位复现；
  - 新 R 臂 vs 260908-10 旧 R 臂：**100.0000% 逐位一致**；
  - 1.20.1 双臂：pre-F1 vs F1 = **100.0000% 逐位一致**（54 残差为既有 feature 族，与 fresh V 臂对比 99.9991% 不变）；
  - Chunky 区域级：F1 域（y≤-54）水差 = **0**。
- **§9.7 可比性声明**：载体 = BlockProbe .blocks（FULL 逐位）+ Chunky region（palette 计数）；覆盖面 = 1.20.1/1.21.6 各 64 chunk FULL + 1.21.6 1089 chunk 区域；与既有口径可比性 = 与 260908-10 P7 同协议同参照直接可比，Chunky 口径与 BlockProbe 口径不可互比（#33 简记）。

## Chunky 区域级复核（1.21.6，#26 confirmed 载体）

- Chunky-Fabric-1.4.40（Modrinth 官方，game_versions 含 1.21.6）；URL `https://cdn.modrinth.com/data/fALzjamp/versions/inWDi2cf/Chunky-Fabric-1.4.40.jar`；sha256 `4dd5430ed9b42bfb7e4ed10a3ec04dee08c6bf57493ba872769a3a8ef10a70c2`（本地计算，size 349,221）。
- 协议：runServer 同实例对称双臂（vanilla=`-PcppVanilla=true` / coreswap=`-PcppReplace` stageMask=3 + F1 dll 2180608B）；seed=-8248318472910187742 三查 ✓；center(464,-248) radius 256 → **1089 chunks/臂**；region 8 mca × 2 臂落盘 `.tmp/p7-chunky-260908-11/region-{vanilla,coreswap}/`。
- **结果**：3698 chunk-slot 全 common；**625/3698 有差**；分类差值 **terrain=27,367 / veg=6,473 / air=0**。
  - 家族构成与 1.20.1 confirmed 基线（chunky-trial-260906-09：113k terrain/3025 chunk）**同构**：blob 石（diorite/granite/andesite ≈6.3k each，已知地形残差域）、水 2.6k + 海草/kelp 下游、mineshaft 结构件（chest/cobweb/cobblestone）。**无新差异家族**。
  - 相对量级优于 1.20.1 基线口径（可算口径：terrain/chunk 25.1→9.0 = 2.8×；terrain 总量 113,454→27,367 = 4.1×；有差 chunk 占比 946/3025→625/3698 = 1.8×；总量口径不同 chunk 数不可直接互比，§9.7）。
  - 水差 y 剖面：section -2..+3（y≈-32..63）均有分布（y≥32 主峰 2518 块，y=-32..31 共 116 块），**F1 域（y≤-54，section ≤ -4）= 0**。
- **残差定性**：0.0256%（terrain/总块，区域口径）属已知残差域的首次 1.21.6 全区域量化；**不构成新机制簇**（同构确认），无需 fan-out（R3 未触发）；单点位精确对齐由 BlockProbe 载体（99.9993%）承担。

## 收口建议

- F1/F2 修复 → **candidate**（建议授予；judge 通过后用户 confirmed）。
- 1.21.6 移植 T1 数据驱动兑现度结论维持（无新增引擎必修项）。
- 遗留：F2 spread-replay 字段级验证（挂账，低价值——诊断工具且静态对齐已闭合）；C1 豁免清单候选 fallen_tree/place_on_ground（下块顺带）。
