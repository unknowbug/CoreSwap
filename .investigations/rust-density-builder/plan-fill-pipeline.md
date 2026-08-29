# 架构 plan: Rust 通用 fill 管线 (fill_chunk) + 性能直排

**状态**: candidate (用户授权"按判断做"; 通用 + 性能 + 宏观正确)
**日期**: 2026-08-27
**目标重构**: 从"有 density 无 fill" → "通用、高性能、能生成 chunk 的端到端 fill"。
**用户验收标准**(2026-08-27 明确):
- 通用性: 方法可通用,未来接 MOD 不出兼容性问题(density/aquifer/surface/ore 可替换)
- 性能: 主要考虑性能实现
- 正确性: 宏观(地形/biome/水)对; 微观(块id/tuff/ore/树/矿物/村庄)不追; 别出严重 BUG

## 为什么是现在做 fill

Rust 现状: density.rs(551行,完整,DensityFunction 树+缓存)、density_builder.rs(完整)、aquifer.rs(完整)、surface.rs(深带半成品)、ore_vein.rs(已移植)。**但 terrain.rs / api.rs / spline.rs 均 1 行空壳**——没有任何东西把这些组装成一个能从 seed 生成 chunk 的生产路径。rust_vs_vanilla 等探针在"逐块算分类",但那只是验证脚本,不是可调用的 fill。

## 目标管线

```
fill_chunk(seed, cx, cz) -> ChunkData
  ChunkData { surface_height: [i32; 16*16],  // 每列地表高度(宏观地形锚点)
              biome: [BiomeId; 16*16],        // 每列 biome(宏观)
              blocks: [BlockId; 16*16*H],      // 块(宏观水/岩/空气即可, 不追具体block id)
            }

  链: density(network) -> aquifer -> surface(深带) -> [MOD扩展点] -> 块分类
```

**通用性设计**:
- 每个阶段一个 `trait`/struct,独立可替换: `DensitySource`, `AquiferSource`, `SurfaceRule`, `OreVein`.
- `fill_chunk` 接收这些 trait 对象(或默认 vanilla 实现),MOD 可注入替代实现.
- 不做 vanilla 硬编码: 每个阶段只依赖上游接口,不依赖具体 block id 语义(宏观层).

**性能设计(关键根因)**: 
- C++ perf-rework 已定位: SplineDF 树遍历虚调用(含 6 SplineDF, loc_fn 可嵌套 Interpolated/Spline → 递归指数膨胀)慢 11×.
- Rust 侧 **spline 采样必须用扁平表 + 显式栈**, 避免递归虚调用(density.rs 现有 SplineData::sample_node 是**递归**的 — 性能隐患).
- 缓存: density 已有 Interpolated/Cache2D/FlatCache 每线程缓存; fill_chunk 需复用这些缓存避免重复采样.

## 分步执行(每步可独立验证)

1. **fill_chunk 骨架**: 组装 density+aquifer+surface 深带, 输出 ChunkData(surface_height + 水/岩/空气块分类). 验证: 宏观地形图(山/湖)大体对. 【本步】 ✅ 完成
   - `terrain.rs` 从空壳 → 完整 fill_chunk(trait 化 DensitySource/AquiferSource/BiomeSource).
   - `fillmap.rs`(end-to-end): 从 spawn 区生成 4x4 chunk 宏观地形图 → 西山地/中平原/东湖, 与正确种子宏观一致.
2. **Spline 直排**: 把 SplineData::sample_node 递归 → 扁平表+显式栈(消除虚调用/递归膨胀). 用 buildnode_test 对拍递归版逐位一致. 【性能核心 — 未做, 下一步】
3. **biome 分类**: 采 6 climate params(已能), 输出每列 biome(宏观). 【宏观 biome — 进行中】
   - `biome.rs` BiomeClassifier 实现(盒包含判定). 机械可用(产出标签不崩), 但**宏观质量需调**:
     已知 cherry (64,-176) 6 params 正确(temp .038/hum -.425/cont .523/ero -.082/weird .653) 标 pliains——因 **dep=0.465 不满足 cherry 盒 depth:[0,0]**, 样本点不在表面层导致 depth≠0; (0,0) weird=1.749 越界=非表面采样.
   - 需修正: ① biome 采样必须在**列表面**(fill_chunk 已做 top 对齐); ② depth 盒放松(宏观 depth 是二级信号, 不在表面即非0); ③ 调试点在 y=64 不代表性(非表面).
   - 判定: biome 宏观正确性是"群系要对"的验收项, 调好 BiomeClassifier 再验 cherry_grove.
4. **通用抽象收口**: trait 化各层, DOC 扩展点. 【通用性 — 已部分(trait), 完善】
5. **性能基准**: fill_chunk 生成 N chunk 计时; 对比递归版/DensityProbe. 【性能验证 — 待做】

## 关键约束 / 不做

- ❌ 不追 block id 精确对齐(tuff/ore/树/矿物/村庄).
- ❌ 不为"算得和 vanilla 一样"绕(用户已多次纠正).
- ✅ 宏观(地形/biome/水)必须大体对.
- ✅ Spline 直排必须逐位对拍递归版(它是唯一"要精确"的性能点——保证直排不引入正确性回归,否则性能优化会埋严重 BUG).

## 风险
- fill_chunk 涉及"何时调 aquifer/surface"的时序,需和 C++ fillFromNoise 对齐(不是逐位,是**阶段顺序**宏观一致).
- Spline 直排改动 density 核心,必须用 buildnode_test/探针对拍保正确(这是唯一要严格的点).

## 归档
- 本 plan 为候选; 分步产物落 .investigations/rust-density-builder/; 错误入 rust-errors.md.
