# Phase 1 交接遗产廉价验证记录（260905-05，实际 2026-09-05 13:0x）

> 依据：架构设计-feature-parity-260905-05.md §2（交接结论验证纪律——G2 遗产不得当公理直接续推）。
> 状态：draft。所有结论为「已验证事实」级；未验证假设单独列出。

## V1：G2 差签名复现 ✅

- 复跑 `.tmp/light-g1/g2_residual_map.py` + `g2_verdict_probe.py`（2026-09-05 13:0x，region 导出 01:23/01:37 当日）：
  - common 3377 chunk，残差 **448/3377**，全部在 pregen 内部（edge=0）——与 G2「447 域内 346 差」口径吻合（本次统计含全域）。
  - palette 差签名逐块核对（worst 6 chunk）：jungle_leaves/oak_leaves/vine/jungle_log 增减 + oak_leaves↔jungle_leaves 串型 + **granite↔andesite 47 处**（chunk 26,-23）——与立项卡签名一致 ✅。
- **新观察（未验证候选，Phase 3 查）**：每 worst chunk 约 4000 处 `('minecraft:air', None)` ≈ 整 section（4096）量级——疑似 section 级缺键/缺 section 差异，非逐块 feature 差。量级上恰好 ≈ 一个 section，可能是 y 范围/section 存在性口径差，也可能是真整段差异。判定留 Phase 3。

## V2：seed/坐标三查 ✅

1. `runtime/1.20.1/java/run/server.properties` `level-seed=8576294172403134396` = charter 目标 seed ✅；
2. `world/level.dat` 实测 seed = **8576294172403134396**（DataVersion 3465）✅（本步脚本 `.tmp/feature-parity-260905-05/check_seed.py`，修了一处漏读根标签名后通过）；
3. 参照 region（vanilla 01:23 / rust 01:37）与 world 同源同 seed 期 ✅。
- 附带状态：RCON enable-rcon=true + password coreswap 在位（#21 纪律核过）。

## V3：影子残差保持推断级 ✅

- 「101/447 影子传播残差」未继承为事实，保持推断级标注，随 Phase 4 柱级验证收口。

## 附带：Q7 tree.rs「未编译验证」状态廉价核实 ✅

- scout1 疑点 Q7：`cargo build --offline -p worldgen --release` 绿（0.18s 缓存命中，32 warnings 0 error）——tree.rs 在编译域内可编译。
- 「编译绿」≠「行为对」：tree.rs 行为级验证仍归 Phase 3 树木分支。

## 交接结论继承裁定

| 遗产结论 | 裁定 |
|---|---|
| 346/447 差签名（trees+ore 两族） | ✅ 已验证，可继承 |
| 翻转集中 Y=3..4 树冠段 | 未单独复验，Phase 3 树木分支柱级验证时顺带核 |
| rust=vanilla−1~2 显式差剖面 | 同上 |
| 101/447 影子传播残差 | 保持推断级 |
| 光照内核无缺陷（G2 confirmed） | 已 confirmed，不重验 |

## 后续输入（scout1 已回，scout2 进行中）

- scout1 疑点 Q1-Q8 已落盘 `.investigations/feature-parity/scout1-pipeline-map.md`（draft，Degraded 静态勘探）。关键：Q3 random_selector/random_patch 占位公式（idk-7）且 generate_nested 未接线——trees_oak/jungle 标准链路实际走占位；Q1 SURFACE/CARVERS 顺序疑点；Q2 单 biome vs 3×3。
