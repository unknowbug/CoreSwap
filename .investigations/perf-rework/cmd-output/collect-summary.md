# 采集摘要：性能回归机制确认（2026-08-12 主会话采集）

> 原始输出：`cmd-output/wgprofile_8576_t1.txt`（单线程）、`wgprofile_8576_mt.txt`（多线程 24）、`splinedebug_8576_t1.txt`（537MB，仅统计摘要见下）
> 命令：`block_probe 8576294172403134396 versions\1.20.1\data\worldgen versions\1.20.1\data\vanilla_8576294172403134396_6_720_-432.blocks`（36 chunks 6×6）
> 构建：MSVC 强制重编（工作区含未提交改造 worldgen_api.cpp + bench_chunks），TOTAL 99.9994% 对齐保持

## 1. WG_PROFILE 计数器（WG_PROFILE=1）

| 指标 | 单线程 (-threads 1) | 多线程 (24) | 旧基线(07篇/8-06) |
|---|---|---|---|
| wall | 6533.3ms (181.48ms/chunk) | 8488.5ms (235.79ms/chunk) | 串行 28.1ms/chunk、并行 3.1ms/chunk |
| density 阶段/chunk | 41.3-50.5ms | — | 8.5-11.7ms |
| spline.sample 总 | 4,695,145（=130,420/chunk） | 4,703,488 | 6250/chunk（**20× 爆炸**） |
| spline 单次 | **1,714ns** | **27,155ns** | 992ns |
| base_3d_noise.sample | 106,452 | 106,540 | — |
| interpGrid.fill | 238 | 238 | — |

**关键：调用量爆炸（20×）才是主因；多线程单次耗时再 ×16（thread_local thrashing）。**

## 2. WG_SPLINEDEBUG 统计（单线程，537MB 输出）

| 指标 | 数值 |
|---|---|
| FLATCACHE rebuild 总数 | 36,252 次（涉及 112 chunk；per-chunk 峰值 1,223） |
| CACHE2D miss 总数 | 351,536 次（4 个 cacheId；cacheId=2 占 186,347） |
| SPLINE 调用总数 | 2,400,550 次（1885 个 (x,z) 列；单列峰值 7,305 次） |

- rebuild 的 chunk 覆盖 45..50 × -22..-27（36 生成 chunk）+ 邻居——**每生成 chunk rebuild ~1000 次**（期望 ~1-6 次/实例/chunk）
- CACHE2D miss 遍及全部生成 chunk（每 chunk ~8-11k 次）——**同列不连续访问 → 单槽缓存全 miss**

## 3. 代码事实（主会话静态确认）

- densityBuf 填充循环（worldgen_api.cpp L669-672）：`for by { for bz { for bx } }` = **y 主序**，同列 (x,z) 相邻两次访问隔 256 次
- Cache2DDF 注释（density.h L630）宣称「块循环 y→z→x 顺序下同列连续 384 次采样」——**与 y 主序矛盾**（Java ChunkNoiseSampler 为 x→z→y 列主序）
- FlatCacheDF key = (x>>4, z>>4) chunk 级；buildGrid 角点 i=4 时 p.x = (chunkX*4+4)*4 = (chunkX+1)*16 = **下一 chunk 首列**（key 指向邻居）
- 单槽 thread_local 缓存（density.h L660/718）：每线程每实例 1 槽

## 4. 待 worker 验证的假设

- H1（Cache2D 顺序失配）：y 主序循环 → 同列不连续 → Cache2D 单槽缓存 100% miss（351,536 次实锤）
- H2（FlatCache buildGrid 嵌套递归）：边界角点 key 指向邻居 chunk → 嵌套 spline 的 FlatCache 重建邻居网格 → 递归放大（rebuild 36,252 ≈ spline 调用 2.4M 的 1.5%）
- H3（多线程 thrashing）：thread_local 单槽 + 每 chunk 跨线程迁移 → 每线程每 chunk 首访即 miss（多线程单次 27,155ns vs 单线程 1,714ns 实锤）
