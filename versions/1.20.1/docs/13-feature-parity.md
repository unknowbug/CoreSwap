# 13. FEATURE PARITY 课题（feature 放置对齐）

> 课题定位：光照课题 G2 收敛（12 篇）转出的独立课题——残差根因 = **feature 放置分歧**，非光照内核缺陷。
> 过程档案：`.investigations/feature-parity/` + `.artifacts/feature-parity/index.yaml`；时间线归 10 篇 260905-05 条目。
> 状态：**candidate**（judge APPROVE-WITH-CONDITIONS 260905-05；confirmed 已由用户拍板；S3 data/ 管理策略待拍板）。

## 课题定位与根因结论

- **G2 收敛定论**（260905-03）：光照内核 blocks 一致域无缺陷；光照残差的根因是 **worldgen feature 放置分歧**（树叶/藤蔓/矿石/安山岩）——光照课题只负责如实呈现这些方块差异，修方块差异属本课题。
- **范围**：FEATURES 阶段（Java `world/gen/feature/*` + placement modifier）Rust 侧的 feature 类型补齐与放置对齐；机制底座见 11 篇。
- **判据**：同方法论受控 A/B region 对比（载体 = region / 覆盖 = 双侧地形完整 chunk 交集 / 可比性 = 与旧口径 117/chunk 同族，§9.7 三要素随行声明）。

## 当前对齐状态（candidate 基线，260905-05）

- 受控 A/B（ab-vanilla vs ab-takeover，双侧完整 3009 chunk）：**239,594 diff 实例 / 1829 chunk = 80/chunk**（旧 dll 口径 155,470/1333 = 117/chunk，净改善 ~32%）。
- 主签名 = 树族残差（oak/jungle leaves + vine，Y4-6 主导）+ 双向 ore 小残差（andesite/granite/diorite 各 ~1-1.2 万，双向均衡）。
- ⚠️ 口径声明：80/chunk 是**相对旧 dll 的净改善结论，非绝对对齐结论**——per-feature 随机序列改变未逐 feature 定界，残留 80/chunk 须归因后才能关闭。

## 已完成（Phase 4a/4b，judge APPROVE-WITH-CONDITIONS）

- **tree.rs 移植**（~700 行）：oak/jungle/fancy_oak（=LargeOak placer）等树族 feature + 树放置管线。
- **biome filter**：feature 放置的 biome check 接入（`anchor_biome: Some(cur_biome_id)`）。
- **trapezoid height**：HeightProvider 补 trapezoid 分布。
- **BiasedToBottom**：IntProvider 补 biased_to_bottom。
- opacity clamp 修复（light_data.json 负 opacity 显式化 + 防回归单测，归 12 篇侧但同工作块完成）。

## 已登记偏差源（残留 80/chunk 的归因台账，均显式转出未核销）

| 项 | 内容 | 兑现签名 |
|---|---|---|
| idk-7 | random_selector/random_patch 占位公式（generate_nested 未接线） | 树族残差主要来源 |
| R-1 | decorator 同 Y 序 = Java HashSet 桶序（近似方案在码，口径需声明） | leaves/vine 双向差 |
| anchor_biome 口径 | chunk biome 锚定 vs Java posToBiome jitter，未单测 | biome 边界块差异 |
| fancy_oak | LargeOak placer 对齐假设，验证口径待拍板 | fancy oak 树形 |
| S3 | light_data.json 版本控制策略待用户拍板（judge 阻塞项，非放置偏差） | — |

## 明确不做（范围裁决留档）

- **光照内核**：G1 exact 100% 无缺陷，不在本课题（归 12 篇）。
- **永久挂起 4 项**（260904-15 用户拍板）：ore_vein 域 1 + aquifer 域 2 + C++ 载具 ore 族 desync + populationSeed 疑点。
- **D3 光照性能**：归 12 篇 round2 后续，不在本课题。

## 验证方法论（本课题判据体系，复用价值）

- **同方法论受控 A/B**（workflow-patterns **#49**）：跨 run 存档/导出对比，完成度随停服时机/init 时序剧烈变化（同位置 andesite 551↔828 波动、空柱 213 个）——差异量级再大也不可作回归证据；必须同 fresh world / 同预生成域 / 同等待 / 同停服流程，再以**零改动正向对照**验伪（造零 rust 参与臂复现同签名即证伪归因——「14.8× 回归」即此被推翻为完成度伪影）。
- **接管生效范围核**（workflow-patterns **#50**）：[CppBridge] init 晚于 Done → 同一存档内 pre-Done vanilla 区与 forceload 接管区两种装饰来源并存；判别实验第一动作 = 核接管生效的 chunk 范围（装饰级 WG_FEATURELOG 日志优先于统计口径推断）。
- 对比一律 name 域（v3 口径），废弃跨 id 域对照（recheck 参照 C++ compact id 域错位，#9 家族）。


# 【草稿】13-feature-parity.md 末尾追加小节（260905-06）

> 供主会话应用：追加到 `versions/1.20.1/docs/13-feature-parity.md` 末尾。追加不覆盖。
> 本文件为草稿，正式 docs 未改动。

---

## 260905-06（实际 2026-09-05）：树族残差收敛 — idk-7 实装 ✅ 判据链闭合（candidate，确定性 dump 口径）

> 过程产物：`.investigations/feature-parity/260905-06-errors.md`（五段式错误台账）+ `.tmp/feature-parity-260905-06/`（v8-v12 脚本链 + dumpnew/dumpold）。
> 载体转折：本块起量级裁决弃用跨 run 受控 A/B（噪声基线 241,080 ≈ 信号 239,594，同阶，workflow-patterns #51 草稿），改用**确定性区域 dump 载体**（#52 草稿）。

### 完成内容

- **idk-7 实装**：random_selector/random_patch 占位公式替换为 generate_nested 真实接线（placed/configured 两级语义修正 + selector default 修正）。
- **四 bug 修复**（详见 `.investigations/feature-parity/260905-06-errors.md`）：① BlockPredicate `"predicate_type"`→`"type"` 键名错读（谓词全灭根因）② selector default 键错读 ③ generate_nested placed/configured 语义接反 ④ heightmap off-by-one。
- **jungle placers 实装**：jungle tree / mega jungle placer 补齐。
- **构建链修障**：`cargo -p` 依赖 rlib 陈旧假绿（build-tooling #23 草稿）——显式 `-p WorldgenRust` + mtime 核验纪律。

### 残差收敛数字（确定性 dump 口径）

**树族残差 171,442 → 118,697（−30.8%）**。§9.7 验证可比性声明三要素：

| 要素 | 声明 |
|---|---|
| 载体 | region name 域，对 **vanilla06 固定快照**（纯实现直接生成，`worldgen-core/src/bin-diag/idk7_region_dump.rs` + SHA256 对拍） |
| 覆盖面 | **2193 chunks 全域**（dumpnew/dumpold 交集全域） |
| 可比性 | 同快照、同脚本链（v10_dump_vs_region / v11_dump_diff / v12_tree_breakdown），与 260905-05 的 171,442 基线同口径 |

⚠️ 口径声明：本数字为**确定性载体上的量级收敛结论**；跨 run 受控 A/B（80/chunk 口径）仅保留作回归门，两口径不可互换（噪声基线同阶，#51）。

### 残余清单（更新后的归因台账）

| 项 | 内容 | 状态 |
|---|---|---|
| R-1 | decorator 同 Y 序 = Java HashSet 桶序（近似方案在码，口径需声明） | 未核销 |
| vines feature | vine 放置 feature 未实装（独立残差源） | 新登记 |
| distance 属性位 | 树干 blockstate distance 属性位差 | 新登记 |
| 树位置对齐 | 树放置坐标逐位对齐（随机序列 per-feature 定界） | 未核销 |
| anchor_biome 口径 | chunk biome 锚定 vs posToBiome jitter | 未核销（承前） |
| fancy_oak | LargeOak placer 对齐验证口径待拍板 | 未核销（承前） |

