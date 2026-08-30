# soul_sand 残差诊断结论（2026-08-30）

- WG_BIOMEDUMP：全部 y=1..2 mismatch 位置 Rust biome 判定 = minecraft:nether_wastes（dist≈0.022，6维 t-0.115 h-0.092 余 0）
- vanilla 参照 want = soul_sand(257)/soul_soil(258) → vanilla 该处 biome = soul_sand_valley（其表面规则限定）
- biome_params_nether.json：soul_sand_valley humidity 点 = -0.5，nether_wastes 全 0 点
- Rust 采样 h=-0.092 落 nether_wastes 盒（给定采样值下最近邻判定正确）→ **根因 = humidity 采样值与 Java 不一致**（非匹配算法错）
- 待查：nether humidity router 定义（shift/noise）+ legacy 下噪声种子派生链 + Java 同位置 h 参照值（DensityProbe nether）
- 附：legacy 激活后 first mismatches 从「bedrock 错位」质变为「soul_sand 表面」→ bedrock 错位顺带解决（真根因=随机源，非反锚序）
