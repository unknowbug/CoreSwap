# C4 overworld 双跑消融量化（采集记录，260902-03，draft）

## 设置
- seed B = 8576294172403134396，overworld 4×4@3200,3208，参照 `versions/1.20.1/data/vanilla_8576294172403134396_4_3200_3208.blocks`（四要素核对通过，无 missing ref）。
- 每次 run 前：Stop-Process java → 删 run/world → 删 .tmp\java-tmp\coreswap-native；gradle 双 env（E8/E10）。
- A：默认 mask=0b011（不双跑）；B：`-Dcoreswap.rust.stages=all` → stageMask=0（旧双跑行为）。

## 结果（ReadWorldProbe 存档口径）
| run | stageMask | match | 对齐率 | nonAir |
|---|---|---|---|---|
| A（新，不双跑） | 3 | 1556380/1572864 | **98.9520%** | 97.1763% |
| B（旧，双跑） | 0 | 1530815/1572864 | 97.3266% | 92.0692% |

- 消融效应：双跑修复带来 **+1.6254 pp**（+25565 块）。
- 分层：差异集中在 y=-64..63（features 活跃层）；y≥64 全部 100% 两 run 一致——符合 features 双跑集中在地表/地下的机制预期。
- nonAir 差异更大（97.18% vs 92.07%）：双跑多放的 features 块（矿石等）把 air 变 nonAir，修复后回落。
- 日志：`cmd-output/c4-overworld-mask011.log` / `c4-overworld-mask-all.log`。

## 解读（采集层，待 judge）
- judge C4 CONCERN（overworld 句柄默认 mask=0b011 行为变更未回归量化）已量化闭合：默认 mask 上线后 overworld 存档对齐 **上升** 1.63 pp（97.33→98.95），无回归证据。
- 与 nether 方向一致（94.42% vs 修复前 93.90%），跨维度行为一致。
- ⚠️ 本 run 为单 region 单次（4×4@3200,3208）：覆盖面口径按 C1 修正后的判据声明——这是「消融方向性量化」，非多 region 覆盖面结论。
