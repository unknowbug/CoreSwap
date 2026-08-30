# soul_sand 残差诊断结论（2026-08-30）

- WG_BIOMEDUMP：全部 y=1..2 mismatch 位置 Rust biome 判定 = minecraft:nether_wastes（dist≈0.022，6维 t-0.115 h-0.092 余 0）
- vanilla 参照 want = soul_sand(257)/soul_soil(258) → vanilla 该处 biome = soul_sand_valley（其表面规则限定）
- biome_params_nether.json：soul_sand_valley humidity 点 = -0.5，nether_wastes 全 0 点
- Rust 采样 h=-0.092 落 nether_wastes 盒（给定采样值下最近邻判定正确）→ **根因 = humidity 采样值与 Java 不一致**（非匹配算法错）
- 待查：nether humidity router 定义（shift/noise）+ legacy 下噪声种子派生链 + Java 同位置 h 参照值（DensityProbe nether）
- 附：legacy 激活后 first mismatches 从「bedrock 错位」质变为「soul_sand 表面」→ bedrock 错位顺带解决（真根因=随机源，非反锚序）

## Java 6 维参照（BIOME6 探针，2026-08-30 深夜）

Java（yarn NoiseRouter 直采，mismatch 同坐标）：t=+0.077~+0.119（正），h=-0.149~-0.175，c/e/d/w=0。
Rust（无特例）：t=-0.115（负），h=-0.092。

**结论**：Java legacy 下界的 temperature/vegetation 噪声 = NoiseConfig LegacyNoiseDensityFunctionVisitor
的固定种子特例（CheckedRandom(0)/(2) + (-7,[1,1]) createLegacy）——biome 分类输入即此特例噪声。
Rust 未启用特例时 t 采样值连符号都不同 → soul_sand_valley 误判 nether_wastes。
Rust 的 visitor 特例实现（WG_LEGACY_CLIMATE）方向正确，但 Legacy-Perlin 数值细节仍有偏差
（v7 净负的原因），需专项对拍：Java CheckedRandom(0)+createLegacy(-7,[1,1]) vs Rust
LegacyRandom(0)+new_legacy(-7,[1,1]) 在同坐标的逐点值。
