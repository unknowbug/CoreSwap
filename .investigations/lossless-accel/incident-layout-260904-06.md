# 数据完整性事故：blocks 列读布局错误（260904-06，主会话 incident 记录）

> status: draft（事实一手；波及重算待办）
> 触发：ore-scout W-A 候选（列布局未核）→ 主会话查 blocks.h:69 实证

## 根因（机制）

`.blocks` 文件布局 = **y-major**：`index = (y−minY)*256 + z*16 + x`（`blocks.h:64-70` 权威；`block_probe.cpp:127` blockDump 同式；Rust `b1_column_trace.rs:69` 同式）。而多个列读分析脚本用 **x-major**：`x = idx//(H*16); z = (idx//H)%16; y = idx%H`——把 16×16×384 的立方体读成沿错误轴的「列」，产生**幻影列剖面**。

## 受污染脚本清单（grep `\* \(H\*16\)` 实证）

| 脚本 | 波及裁决 |
|---|---|
| `probe_ns_cells_260905.py` / `extract_oob_samples_260905.py` | writer-verdict-260905（OOB 样本/列签名） |
| `e1_check_260904-06.py` / `confirm_doublerun_260904-06.py` | writer-verdict-260904-06（E1 列清空 / OOB 族计数 / 列 (195,199) dirt ys） |
| `ref_check_p4_260904-06.py` / `xcheck_p4_260904-06.py` / `ref_terrain_p4_260904-06.py` | **本 session** p4-reference-check + curtain-verdict-260904-06 |

## 什么还成立 / 什么作废

**仍成立（布局无关）**：
- block_probe 引擎内对比 TOTAL（95.03% 等，engine 内部索引一致）
- WG_DBDEBUG 生产密度 dump（按 y 直印，不经过 blocks 布局）：**(195,199) 列 y192-318 密度全负（−0.02/−0.46）**——这是硬数据
- aquifer/b1/b2 静态对拍结论（不依赖 blocks 读数）
- seed/两侧一致性（header 直读）

**作废/需重推导（布局相关）**：
- 我今天的 P4「ref 列含 131 stone 族」——**正确布局重读：ref/cpp 列 (195,199) y180-319 全 air**。原始前提「vanilla 无幕帘」是对的，「幻幕帘」= 布局 bug 伪影
- curtain-verdict-260904-06 的裁决理由（基于误读数据）——连带失效；judge 独立复算也用了同一错误布局脚本（复算复现的是同一个 bug）
- 「两臂幕帘 / 43/11 边界微差 / E1 列清空 / (195,199) 30 dirt」等坐标性结论——臂间差异数值本身真实（同错位索引两侧一致），但**位置归属全错**，需正确布局重跑 E1/E2 型判别
- ore 族 desync 底账 22653 的列级归因（cmp_spatial 同布局）——族计数（值对统计）或可救，坐标归因作废

## 正确重读快照（y-major，列 (195,199)）
- ref y180-319：全 air（140 块）；cpp NOISE+SURFACE 同：全 air
- ref↔cpp 全列 diff 仅 22：y−45..−44 cave_air↔deepslate、y6-31 andesite/diorite↔stone——后者 = cpp 导出无 FEATURE 阶段的预期差，非 desync

## 教训（知识库候选，另派草稿）
1. 列剖面脚本第一行 MUST 引布局权威（blocks.h:69）并做 sanity（打印已知地形特征，如海平面 y62-63 水）自检——#13 探针 sanity 家族。
2. judge「独立重跑」若复用同一变换代码，复现的是同一个 bug——独立复算必须独立实现变换（本例 judge 用我的脚本复算，双错一致通过）。
