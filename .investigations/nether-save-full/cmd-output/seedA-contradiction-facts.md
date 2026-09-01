# nether 存档级 Full 化 — seed A 矛盾现场（2026-09-04）

> ⚠️ **supersedes（§15.4，2026-09-04 回填）**：本文所有 run 的前提已被推翻——三场 gen 的 CppBridge 均 `enabled=false`（cppWorldgenDir 传错一层 → wg_create 返回 0），世界为 vanilla 生成而非 Rust。本文数据仅作过程记录保留（正文不删不改）；有效数据以 `.tmp/rust_nether_seedA_v2*.log`、`.tmp/compare_nether_seed{A,B}_rust.txt`（v2 真 Rust run）为准。取代详情见 `../nether-save-errors.md` E1/E5 与 `versions/1.20.1/docs/10-timewise-archive.md` 2026-09-04 节。

## 实验设置
- 区域：4×4 chunks @ (3200,3208)，nether，dll sha256=C5AC5309F3C59A044（1.0.22 M17 版）
- 参照：.tmp-coreswap-data/vanilla_-2032795982907864146_4_3200_3208_nether.blocks（vanilla BlockProbe nether 导出，WGB2，min_y=0 height=256）
- seed 三查：server.properties ↔ level.dat ↔ ref header 全部 = -2032795982907864146 ✓
- 流程：vanilla 导出 → 清 world → Rust gen（cppReplace+readWorldProbe 同跑）→ reconfirm（只 readWorldProbe，从盘读）→ compare_save_region.py（MCA 直解）
- ReadWorldProbe 本次新增 nether 支持（dim 属性 + 动态 min_y/height + _nether 参照后缀）；gen1/gen2/reconfirm 用的是同一份改动后代码

## 三条关键观察（seed A，同 dll 同参数）
1. gen1（pwsh-18，20:09）内存级：match=1048445/1048576 (99.9875%)，131 差 = 1×quartz→gold + 130×air→cave_air（chunk(203,200) y70-71 一簇）
2. gen2（pwsh-21 gen，20:13:15）内存级：match=1048575 (99.9999%)，1 差 = quartz→gold only，**无 cave_air 差**
3. gen2 存档（compare_save_region.py 解 r.6.6.mca）：1048472 (99.9901%)，104 差 = 103×air→cave_air（同 chunk(203,200) 同簇）+ 1×quartz→gold
4. reconfirm（pwsh-21 reconfirm，20:13:47，从盘读）内存级：1048575 (99.9999%)，1 差 = quartz→gold only
5. 独立 MCA 解析确认：r.6.6.mca chunk(203,200) (3263,70,3211)=cave_air；(3200,13,3208)=netherrack；y69:air256 / y70:air252+cave4 / y71:air233+cave23 / y72:air203+cave53

## seed B（对照，正常）
- gen 内存级 = 存档 MCA = 1014474/1048576 (96.7478%) 精确同值；残差 = blackstone/basalt/netherrack 大宗互换 + 矿石/熔岩（另有议题）

## 矛盾点
- gen1 内存 vs gen2 内存：同参数跨运行不同（M4 家族嫌疑）
- gen2 内存（1 差）vs gen2 存档（104 差）：同一次运行读与存不同
- gen2 存档有 cave_air vs reconfirm 读盘无 cave_air：同文件两读不同
- gen1 内存状态 ≈ gen2 存档状态（cave_air 都在）

## 原始文件
- .tmp/rust_nether_save_seedA.log（gen1）、.tmp/rust_nether_save_seedA_gen.log（gen2）、.tmp/rust_nether_save_seedA_reconfirm.log、.tmp/compare_nether_seedA.txt、.tmp/dump_chunk_203_200.py（解析脚本）、.tmp/rust_nether_save_seedB.log、.tmp/compare_nether_seedB.txt
- 工具源码：runtime/1.20.1/java/src/main/java/wg/bench/ReadWorldProbe.java、.investigations/multiworld-port/cmd-output/compare_save_region.py、runtime/1.20.1/java/src/main/java/wg/bench/mixin/NoiseChunkGeneratorMixin.java
