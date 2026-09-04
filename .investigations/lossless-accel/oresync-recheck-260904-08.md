# B2 族分解重算记录（260904-08 · .investigations 中间产物）

## B1 对账（三项全过，coltool y-major 可信）
- ref↔off y>200 diff = 0（verdict OOB 结论独立复现）
- ref 列 (195,199) y180-319 全 air = True
- ref↔cppNS mismatch 78107 → match 95.033% ≈ block_probe 引擎内 TOTAL 95.03%（口径：4×4 chunks @200,200，seed 8576294172403134396，y-major coltool 重读 vs 引擎内计数——载体/覆盖面/可比性：同 dump 载体、全 chunk 覆盖、与引擎内口径首次建立可比）
- **旧底账 22653 作废**（x-major 污染脚本产物）；新底账 = **13328**

## B2 分解（ref vanilla FULL ↔ off mod Rust FULL，4×4 @200,200）
- 13328 diff：全部 y≤200；y32-47 带 11268、y16-31 带 1731
- 全部在含水列（water≥8：13284；415/1024 列受影响）；列顶 y ref↔mod 全一致（topdiff=0）
- 全部在列顶以下 >8 格 = 海底床面区（水深 16-24 列，床面 y≈38-54）
- 主族（表面材质错位，~12000+）：stone→dirt 4165 / gravel→sand 1687 / stone→sand 1581 / gravel→dirt 1359 / granite→dirt 439 / sand→dirt 399 / …——vanilla 留 stone 的深度 Rust 放 dirt/sand
- 次族（独立记账）：
  - ore：coal_ore→stone 158（Rust 缺 coal ore 放置，ore desync 家族残差）
  - 流体/aquifer 域：water→air 520 / stone→water 217 / air→water 170 / deepslate→air 116（~1023）

## 归因（收敛门：单假设）
主族与 heightmap-criterion-divergence-260904-06.md 静态坐实的分叉预测吻合：
- Java 权威 Heightmap.java:24 NOT_AIR = !isAir()（水计入 heightmap）
- Rust terrain.rs:279 判据 d>0（水不计）→ surface 规则的 surface_depth/基准面错位 → 床面材质族错位
- 判定探针 = T3 修复后重导 off 臂重跑本对比（decisive probe）

## T3 修复与 decisive probe（260904-08，commit 2bc6503）
- 修复两处（WorldgenRust/src/terrain.rs）：
  - :234 aquifer.classify：apply 返回 -1（barrier/margin）→ `BlockKind::Rock`（原归 Air 丢 barrier stone）
  - :284 surface_height 判据：`d>0` → `kind != Air`（水计入；= C++ :1045 / Java Heightmap.java:24 NOT_AIR）
- 新 dll sha256 = 0D247E03…（已同步 MC java resources；注意：run_rust_client.ps1 走 -PcppLib 直连 target/release，resources 仅 jar 链路需要）
- **decisive probe**（ref↔off-fixed，全新 world 重导 19:24，seed 三查过）：mismatch 13328 → **1830**（99.153%→99.884%）
  - 床面 (236,198)：y35 gravel/y34 stone = vanilla 精确恢复
  - 煤矿脉 (209-212,20,229-232)：16/16 恢复（ore desync 在本域随 heightmap 修复消失——旧 ore 差异是 heightmap 连带，非独立 ore 放置 bug）
- **残留 1830 新族**（下一阶段课题）：water→air 314 / stone→water 217 / water→stone 182 / air→water 170（aquifer 流体域 ~883）、deepslate→air 106（carver 域?）、stone→dirt 85 / stone→grass_block 66 / stone→air 53 / dirt→stone 51 / gravel→sand 49（零散 surface/feature 小族）
- 探针纪律事故自查：第一次重导命中旧 world 缓存（chunk FULL 0-1ms，#19 家族）——备份 world 为 world.bak-260904-08-fixprobe 后重导，pregen 24s 确认全新生成
- 三查：log worldSeed=8576294172403134396 ✓ header seed ✓ origin (200,200) size 4 ✓

## 命令记录
- python .tmp/p2full/recon_b1_260904-08.py（TOTAL/per-chunk/pairs）
- python .tmp/p2full/recon_b1b_260904-08.py（y>200=0 / cppNS 对账 / 列 air）
- python .tmp/p2full/recon_b2_260904-08.py（族分解）
- python .tmp/p2full/rescan_260904-08.py / col236_260904-08.py（实机对照重扫）
- 脚本 bug 两处（低价值不进知识库，family 已有 #17/#40）：①divmod(i,256)→(i,256*16) 仍错，正解 rem=i%256; lz,lx=divmod(rem,16)——witness 版坐标 z 混入 y 作废；②"distinct columns /1024" 标注错，区域 4×4 chunk=64×64=4096 列（修正后 4096 列均有 diff）

## 实机对拍（260904-08 用户 -Vanilla 实测，全吻合）
- 前置修复：BenchMod 无参启动也会 init 原生 dll → 崩溃（cpp.vanilla 门控修复：BenchMod.java + build.gradle 两段映射 + run_rust_client.ps1 -Vanilla）
- (236,198) 列剖面：床面 y34-35，vanilla gravel@35/stone@34 ↔ CS sand@35/sand@34（用户实机"vanilla 沙砾 CS 沙"精确吻合）；dump coal@y33 (236-239,200-202) 埋于床面下（用户未见，合理）
- (211,20,229)：vanilla 煤矿脉 (209-212,20,229-232) ↔ CS 全 stone——**ore desync 实机确认**
- (192,20,197)：两侧都有水下洞窟、形状差异大——水洞/材质族差异实机可见
- **blocks.h:69 布局（y-major, z*16+x）经实机对拍确证**——coltool 直读可信
- 用户观察"CS 沙 更合理"：vanilla 是唯一权威；CS 的 sand/dirt 带来自 surface 规则错位（heightmap 判据分叉），非刻意改进

## 修正后的 B2 列级统计（坐标 bug 修复后）
- y 带：y32-47 = 11268 / y16-31 = 1731（不变，buffer 级本就不依赖坐标还原）
- 含水列：water≥8 = 13272、water1x8 = 56（微调）
- diff 列：4096/4096 列均有 ≥1 diff；列顶 y 双方一致（topdiff=0）
- 命令记录族计数（top pairs）不变

## T4 隔离
- .tmp/p2full/quarantine-260904-08/：22 个 pre-correction（<18:29 y-major 修正）污染脚本及派生输出 + README
- 保留：coltool.py / recheck_y_major / rederive_e1* / 全部原始 dump / extract_df（读点采 txt，非列读，低风险）
