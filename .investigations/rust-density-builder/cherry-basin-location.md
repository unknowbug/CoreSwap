# 正确种子(-2032) cherry basin 定位 —— 确认（完整范围）

> **记录价值门标注（2026-08-21 对齐框架升级）**：本文件属**低价值（不记入知识库）**——特定种子 `-2032795982907864146` 的一次性地形位置（"cherry grove 在 x 64..256"），自推可得、无跨项目复用价值。按价值门它**不进** `docs/` 主题篇 / `knowledge/`，只留 `.investigations/`（过程性载体，主会话可写）。保留它仅为本次排查的过程跟踪；真正的高价值是"**如何定位** cherry basin"（判据：6 climate param 采样 + biome_params 比对），不在此文件。

**状态**: candidate（cherry 带全范围已圈定）
**种子**: -2032795982907864146（canonical, 用户拍板）
**真实 spawn**: (-96,118,-48)（正确种子; 非污染旧的 (320,63,-96)）

## cherry_grove 连续带（世界坐标）
- **x ∈ [64, ~256], z ∈ [-256, -128]**，spawn 的东/东南方。
- surface Y = 68~130（丘陵/低山，60+ 高差 → 「山 + 盆地」宏观）。
- 约 70+ 个连续 CHERRY 命中（交叉验证带内连续，非零星）。

## 关键 6-climate-param 范围（该带特征）
- temp ≈ +0.03 ~ +0.10
- hum  ≈ -0.39 ~ -0.44
- cont ≈ +0.40 ~ +0.61
- ero  ≈ -0.08 ~ -0.135
- weird≈ +0.27 ~ +0.99
全部落在 biome_params.json 的 cherry_grove 多 box 内。

## 样例（代表性）
```
(  64,-208) y= 68  temp=+0.029 hum=-0.410 cont=+0.492 ero=-0.079 w=+0.345
( 128,-160) y=124  temp=+0.063 hum=-0.430 cont=+0.522 ero=-0.109 w=+0.784
( 192,-144) y=126  temp=+0.086 hum=-0.417 cont=+0.560 ero=-0.121 w=+0.802
( 224,-176) y=122  temp=+0.087 hum=-0.398 cont=+0.465 ero=-0.124 w=+0.532
( 256,-128) y=130  temp=+0.101 hum=-0.393 cont=+0.614 ero=-0.103 w=+0.579
```

## 结论
- **cherry basin（山+群系+湖宏观）确实存在于正确种子**，位于 spawn 东南 x=64..256, z=-256..-128。
- Rust finalDensity（正确种子）在参照区已逐点吻合 vanilla（maxDiff 6.8e-5）。
- **可交付锚点**：若要验证 Rust 在 cherry 带复现，导出该带 vanilla 参照（如 chunk(4,-11)~(16,-8)）并与 Rust 对拍 terrain+surface。

## 下一步（真正剩余工作量 = surface/块层，非 density）
`block-gap-deepslate-band.md` 已单列：Rust 块管线缺 stone→deepslate/tuff 转换、地表块状态（草方块/雪/gravel/sand）、bedrock 底部强制 → 这是「做1/做3」要补的层，与 density 层无关（density 已对齐）。
