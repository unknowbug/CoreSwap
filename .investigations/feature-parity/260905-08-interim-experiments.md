# 260905-08 树位置对齐 · 实验中间记录（主会话执行日志）

> status: 过程记录（.investigations 中间产物，非结论）。载体 = 确定性 dump（#52，seed 8576294172403134396，cx -2..40 cz -26..24，2193 chunks）。

## 已完成实验链

| # | 实验 | 结果 | 判定 |
|---|---|---|---|
| V0 | 廉价独立验证：重跑 idk7_region_dump | SHA256 = CDC54CFA… 与 dumpnew 完全一致 | 载体可复现，上轮数字可继承 ✅ |
| E1 | .b1 MegaJungle 查表三角接线（tree.rs:592-593 → carver::math_cos/sin） | region_b1.bin vs region_new **0/2193 chunk 变化** | **.b1 块级排除**（j/k 翻转无块级效应，符合 bA 产物预设排除判据） |
| E2 | Fix-1（3×3 biome 并集 + 全局 max_step + int_set 并集） | region_fix1.bin：树族残差 119,198→131,211（+10.1%）；oak +19,262→+29,708 / jungle −20,464→−22,295 / vine −20,831→−22,236（signed rust−java） | **Fix-1 单独恶化**，符合 bB 预设判据「oak 增大→主体在位置级门/其他」 |
| E3 | Fix-2（Biome 门改 jitter 采样 + 允许集判定，删 anchor_biome） | region_fix2.bin（=Fix-1+2）：oak +26,385 / jungle −19,464 / vine −19,916；一手核对 BiomePlacementModifier.java:27 确用 ChunkRegion.biomeAccess（hashSeed jitter）✓ | 门语义正确（一手源码证实），部分回收但 oak 仍差于基线 |
| E4 | 单变量分解：Fix-2 only（临时撤并集） | region_fix2only.bin：oak +18,652 / jungle −20,792 / vine −20,963 ≈ 基线 | **恶化全部归因 Fix-1 并集**；Fix-2 门 ≈ 中性且为精确语义（保留） |
| E5 | Fix-1 差异块空间/id 分布（v14_edge_test.py） | 1.16M 差异块，id 全为 ore 族放置块（tuff/andesite/granite/gravel/dirt…），分布非边缘富集 | 并集效应主体 = ore feature 执行集扩大（自洽，非异常）；⚠️ 首跑踩 #40 幻影剖面（idx 384 步长误读），修正后重跑 |

## 当前代码状态
- Fix-2 门：已接线（placement.rs biome_allows/feature_id；worldgen_handle.rs jitter 允许集闭包），**保留**。
- Fix-1 并集：**临时撤单变量**（entries 只 push cur_biome_id，原始循环保留在 TODO(restoring) 注释中）——待 c-A/c-B/c-D/c-C 判别后决定恢复方式。
- .b1 查表三角：已接线（保留，语义正确；虽块级无效应）。
- 每次构建已核 rlib mtime（#23：曾抓到一次假绿，显式 -p WorldgenRust 修复）。

## 未闭合分叉（fan-out 已回收，判别完成）
- c-A（dbe5b150）：block_at 越界保守拒绝 —— E-cA3：E(tree new)=0.69<1.3，非 Fix-1 恶化源；独立候选（jungle/vine rust少 10-30%），修复方案=c-A-min 列缓存+pending_cross（见 260905-08-cA-block-boundary.md）
- c-B/c-D（034b193c）：c-B 门链静态排除；c-D「数据污染」被 E-cD-1 证伪（vanilla jar dripstone_caves 本就含 trees_plains——worker 先验错误，一手 jar 证据）
- c-C（623c0d66）：允许集语义证伪；但发现 B 实锤 = **p-index 构建序差**

## ★E-C2 主根因实锤与修复（260905-08 本轮最大成果）
1. Java WG_SEEDLOG mixin（ChunkRandomSeedLogMixin + BiomeSourceLogMixin，-PseedLog 通道）采集 chunk(29,-16) setDecoratorSeed (p,k)
2. E-C2 对拍：k=9 vegetal java {0,6,7,10,11,63,64,71,72,76,78} vs rust 完全不同 → p 错位实锤
3. 一手深挖：Java p = **PlacedFeatureIndexer.collectIndexedFeatures 拓扑排序**（非首现序！biome 内相邻 feature 建边 → (step,featureIndex) 比较器 DFS 后序反转 → 按 step 分组）
4. 修复三件套：
   - biome_registry_order.json（53 biome registry 序，从 [BIOMELOG] 提取，数据驱动；nether 缺失自动回退字典序不影响）
   - biome.rs all_features_lists 按 registry 序输出（Fix-1 并集配套）
   - feature_loader.rs build() 重写为 Java 拓扑排序精确移植
5. 验证：k=6 ores p 序列**逐位一致**（含空洞 23/26-28）、k=9 vegetal **逐位一致** ✓✓；区域 dump：jungle_leaves −20,464→−15,151（26%收敛）、vine −20,831→−16,829（19%）、jungle_log −3,949→−2,453（38%）

## ⚠️ 新焦点：oak_leaves +29,830（p-fix 后唯一恶化项）
- java k=4 有 26 个 decorator 调用 = 结构 start（SURFACE_STRUCTURES），rust 无结构 → 结构改地形后树可见性不同的候选
- java k=7 有 [0,1,2] rust 无（待查：UNDERGROUND_DECORATION 差异来源）
- 候选：c-A（边界 block_at）独立贡献 / 结构缺失耦合 / jitter 门非 surface 域 IDK
- 下轮从 oak 单点对拍（worst chunk featurelog 全链对 RNG 消费序）继续

## 基线 signed 残差对照表（rust−java，本轮口径）
| 配置 | oak_leaves | jungle_leaves | vine |
|---|---|---|---|
| baseline | +19,262 | −20,464 | −20,831 |
| fix1 only | +29,708 | −22,295 | −22,236 |
| fix1+2 | +26,385 | −19,464 | −19,916 |
| fix2 only | +18,652 | −20,792 | −20,963 |

注：本轮 signed 口径与 NEXT_SESSION 的「双向分解」绝对值口径（oak 38,250/jungle 22,611）不同——本轮为逐 id 带符号差，内部对比自洽，跨口径不可比（§9.7）。
