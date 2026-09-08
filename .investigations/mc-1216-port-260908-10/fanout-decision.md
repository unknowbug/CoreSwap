# fan-out 决策记录 — P7 残差归因（260908-10）

> 触发依据：计划 `架构计划-260908-10-mc1216-phase3b.md` §fan-out 预置第 2 条 + spec §4.5「同一现象多机制候选 → MUST 并行，禁止主会话自推」。

## 现象（共同输入，两 worker 各自独立获得，互不共享上下文）

P7 BlockProbe 双臂（1.21.6 vanilla vs Rust 接管臂；seed -8248318472910187742；origin (200,200) size 8；FULL）：
- 方块 6,279,591 / 6,291,456 = **99.8114%**；biome 16,384 格 **0 差**。
- 11,865 mismatch 的 87% 集中在 chunk (12..14, 12..17)。
- 签名：vanilla=deepslate/stone/tuff → Rust=**air**（≈6,400）或 **water**（≈4,600）。
- 列剖面：chunk(13,14) 世界 (223,237)/(222,238) vanilla 整列 deepslate，Rust = air y≈-31..-6 + water y≈-2..+2。
- 约束：R 臂 `stageMask=3`（Rust 跳过自身 carver/features）+ 两臂 Java carver 同码同 seed ⇒ carve mask 应一致 ⇒ R 臂 air 只能出自 Rust 填方块阶段（aquifer）。
- 数据面：P3 实证两版 `noise_settings/overworld` + 全部 `density_function/overworld` **零差异** ⇒ aquifer 噪声输入相同。

## 互斥候选

| 候选 | 假设 | Worker | 产物 |
|---|---|---|---|
| **b1** | 1.21.6 `AquiferSampler` 语义变化（邻居 4 点化等）改变 air/water 落方块 → 足以解释簇状残差；port-list §B「不进方块放置」裁决被挑战 | subagent `0161a630-0498-49d7-8abe-831281ac4e0f`（只读，无 shell） | `.artifacts/mc-1216-port-260908-10/b1-aquifer-version-diff.md` |
| **b2** | Rust `worldgen-core/src/aquifer.rs` 存在相对 1.20.1 Java 的潜在语义偏离（此前验证未覆盖的配置），可产生该签名 | subagent `28816948-9962-4322-a7bf-cdc10b1d7b65`（只读，无 shell） | `.artifacts/mc-1216-port-260908-10/b2-rust-aquifer-audit.md` |

两候选互斥（差异来自「Java 侧版本变化」vs「Rust 侧偏离基准」），不能同时成立为唯一根因。

## 隔离要求（已执行）

- 各 worker 只看自己的假设指令；不共享对方上下文（防锚定）。
- 写路径隔离：b1 只写 b1-*，b2 只写 b2-*，不交叉。
- 主会话**不自推候选**；主会话只做：数据层探针执行（subagent 无 shell，§九.12）、产物汇总、交 judge。

## 主会话并行采集（数据层，不参与候选判定）

- `-PaqDump=true` Java 1.21.6 vanilla aquifer 全链路 dump（点集 = 簇 3 列 + 对照 chunk(0,0) 1 列，y=-34..6）→ `.tmp/p7-aqdump-260908-10/java-vanilla-aqdump.txt`（供 b1/b2 的判定共享同一数据层证据）。

## 后续（预置）

1. 两 worker 回收 → **judge（MUST，只出意见）** 对比 .bN（审查对象：b1/b2 产物 + 本记录 + 数据层 dump + P7 对拍记录）。
2. judge 意见 + 数据层证据 → **用户拍板**（human hook：多假设竞争 + 若结论为「需移植 1.21.6 aquifer 语义」则属范围决策 = 重大转向回 Phase 0）。
3. 被淘汰候选保留（core.version：`.bN` 不删）。

## b3（主会话数据层新假设，judge SHOULD-⑤ 要求登记）

**H-b3：1.21.6 新增的 `StructureTerrainAdaptation.ENCAPSULATE`（序数 4，trial_chambers 唯一使用者）未被 Rust Beardifier 实现 → 结构处密度局部偏低 → 残差簇。**

- 来源：**非候选自推**——b1/b2 双否后由**数据层新证据**得出：① 同点双 aquifer dump 68/68 density 符号翻转（`.tmp/p7-aqdump-compare-260908-10.txt`）② 世界 NBT 直读残差点落在 trial_chambers 249 children BB±12 影响域 99.44%（`cmd-output/p7-beard-containment.txt`）③ 一手枚举 diff（`StructureTerrainAdaptation.java`）+ 全数据集 encapsulate 扫描（`cmd-output/p7-encapsulate-scan.txt`）。
- 反事实验证：实现 ENCAPSULATE 后同 seed/区域重跑 → 41/6,291,456 = 99.9993%，影响域内残差 0/41。
- 判定：**SUPPORTS（行为级闭环）**，置信度 candidate；judge 已判「主会话在双否后用数据层新证据定位根因**合规**（非候选自推）」（review-260908-10-001）。
- 产物：`.artifacts/mc-1216-port-260908-10/p7-e2e-verdict-260908-10.md` + `.investigations/mc-1216-port-260908-10/progress-260908-10.md` §P7 根因定位与修复。
