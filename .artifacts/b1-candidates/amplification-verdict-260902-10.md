# feature/carver 放大系数量化 + 存档残差改判（260902-10 · candidate，待 judge + 用户拍板）

> session 260902-10 · seed 8576294172403134396（worldSeed 三查 ✓ 全部 4 次运行）· 域：chunk 3200..3211 区 + chunk 200..203 区（各 4×4，nether）
> 取代对象（§15.4）：four-candidate-verdict-260902-09.md **C5 条**的「真实存档残差 ~3.4% 主体归因 feature/carver 链路差（26 单元种子 × 放置放大，放大系数未量化）」——原条不删不改，本文取代其残差归因与放大假设。

## 结论（candidate）

1. **feature/carver 放大系数 = 不存在（不适用）**。cppReplace 存档（stageMask=3，Java carver/feature 照跑）vs 同期新鲜 vanilla FULL 参照：
   - **200 区（biome 边界混合区）**：0 / 1,048,576 = **0.0000%** 失配（逐位一致）；
   - **3200 区（basalt_deltas 单一 biome 区）**：16 / 1,048,576 = **0.0015%** 失配，**16 块全部落在 B1 已判定的 13 列 NOISE 微差 + 1 列 surface-only 差**（种子列集合上 100%）。
   - 26 种子单元 → 16 块存档失配，系数 0.62 < 1：surface 层微差在 feature/carver 链路**无放大**，端到端残差在 surface 阶段即闭合。
2. **历史「~3.4% 存档残差」改判为参照口径污染（supersedes）**。run3-6 mismatch（35,426 块，96.6215% 口径）的对比参照 = `versions/1.20.1/data/vanilla_8576294172403134396_4_3200_3208_nether.blocks`（sha256 **02b94092f917cb5d**，mtime 2026-09-02 16:52）——该文件是 **SURFACE 阶段参照**（无矿石 417/607/45、无 cave_air 730、basalt 仅 15k，docs/09「真 SURFACE 参照 hash 02B94092」即此文件），不是 FULL 参照。「3.4%」= FULL 存档 vs SURFACE 参照的**跨阶段伪残差**（feature/carver 产物被计为失配），与 verdict 已废除的 13.70% air / 22.5% SURFACE 同病（测量口径阶段污染）。
3. **当前 dll 端到端对齐真实水平**：两区域合计 16 / 2,097,152 = 99.9992%，且 16 块全部可归因 B1 NOISE 单元格微差（candidate，单 seed/双区域/4×4 外推边界）。
4. **区域桥接缺口（附带发现，已闭合）**：C5 原文以 3200 区 26 种子桥接 200 区 3.4% 残差——两区数据本就不同域（本 session 盘点证实），该桥接作废。
5. **区域间差异自洽性（judge CONCERN 补充）**：200 区 0 残差与 3200 区 16 块残差不矛盾——反向命题成立：200 区 NOISE 层与 vanilla 完全一致（不含 B1 的 13+1 差异列种子），故无种子即无残差；残差只出现在有 surface 微差种子的列，这正是「放大系数=不存在、残差在 surface 层闭合」的预期结果。另：两区 cppReplace 数据来自两次独立运行（同 dll sha256=68d7f401、同 seed、同 stageMask=3），跨运行稳定。

## 证据链（§9.7 口径声明同行）

| 对比 | 载体 | 覆盖面 | 失配 | 可比性 |
|---|---|---|---|---|
| fresh vanilla FULL vs cppReplace 存档（200 区） | 存档读回 MCA vs BlockProbe FULL 导出（同日采集） | 4×4@200..203 × y0..255 | 0（0.0000%） | 与历史 96.62%/3.4%/93.x 不可比（那些参照=SURFACE 阶段） |
| fresh vanilla FULL vs cppReplace 存档（3200 区） | 同上 @3200..3211 | 4×4 × y0..255 | 16（0.0015%），100% 落 B1 差异列 | 同上 |
| old ref(02b94092) vs fresh vanilla（200 区） | — | 同上 | 214,474（20.4538%）：old ref 缺 feature/carver 产物 | 证明 old ref 非 FULL |
| cppReplace 内存导出 vs 存档读回（200 区） | benchOut .blocks vs MCA | 同上 | 0（0.0000%） | 读回无损 |

## 采集与工具

- 4 次运行均 `CppBridge enabled=true stageMask=3` / `worldSeed=8576294172403134396` 日志核对（log：.tmp/amp-van-ref-run.log、amp-cppreplace-run.log、amp-cppreplace-run200.log）。
- 采集流：快照/清空 DIM-1 region → BlockProbe FULL（benchOrigin 为**块坐标**：200 区=3200,3208；3200 区=51200,51328）→ 存档快照 → python MCA 解包对拍。
- 脚本：.tmp/amp_step0_regioncheck.py（区域考证）、amp_step1_taxonomy.py、amp_step2_join.py、amp_step3_region200.py、amp_step4_crosscheck.py（判别）+ 各 .out.txt 原始输出。
- sanity 行：每次对拍打印 chunks loaded/ref coords/id dist/总数（防 #12 假阴性）。

## 错误/教训记录（本 session 新增）

1. **参照文件四要素核对不够**——文件名（seed/size/origin/dim）不含**阶段**，SURFACE 参照被当 FULL 参照使用贯穿 M16→V5 多轮（96.62% 口径链）。判据升级：参照文件核对五要素 = seed/size/origin/dim/**stage（内容指纹：矿石/cave_air 等阶段特征 id 有无）**。
2. **上一轮 b1_blob_amp_sim（INSUFFICIENT 10.8×）输入 = 200 坐标旧 bug dump**，结论作废（本次盘点发现，未再继承）。
3. Region 语义坑：benchOriginX/Z = 块坐标（wx=origin/16+cx）；chunk 3200 区要传 51200/51328。

## 待办/移交

- judge 审查（MUST，重大转向）→ 用户拍板：C5 改判 supersedes 回写 docs/09 + 10 时间线 + knowledge 新发现。
- signature A（biome 3.7% 真差）与 soul_soil V1 维持原判，不在本文范围。
- 3200 区 16 块明细可下钻 B1 NOISE 微差（B1 外推边界不变：单 seed）。
