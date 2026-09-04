# 收敛摘要：幕帘成因 fan-out（260904-06 幕帘线，主会话过程记录，非结论性 docs）

> 三臂产物：b1-density-tree.md / b2-est-divergence.md / b3-aquifer-fluid.md（均 candidate-pending，Degraded 静态）
> 勘探地图：curtain-scout-260904-06/map.md

## 候选裁决（静态层）

| 候选 | 裁决 | 依据 |
|---|---|---|
| B2 est 扫描差 | ❌ 排除出成因集（降为 B1 下游投影 + 口袋形态调制项） | 幕帘 stone 在 `density>0` 前置判定出局（aquifer.h:74 / aquifer.rs:295），est 零贡献；scout P3 前提被勘误（est 全绿验证域恰为 chunk region (200,200) 8×8，est-shared-verdict-260903-12.md:7，idk-4 已闭合） |
| B3 aquifer d≤0 误回 solid | ❌ 结构性排除 | 三方 apply 均无此代码路径；aquifer 本体 17 项逐分支对拍零共享偏离 |
| B1 density 抬升 | ✅ 唯一存活共有成因候选（待运行时判别） | 理论地板 y256-318 ≈ −0.025（翻号只需 ≥0.03 加性偏差）；口袋 @197-277 反演 est≳200 同指高海拔密度抬升；两臂读同一 noise_params.json（worldgen_api.cpp:143 / worldgen_handle.rs:202 → data\noise_params.json，已核实唯一份） |

## 新账目（单臂偏离，独立登记）
- **Rust margin→air 分叉**：barrier margin 返回 -1 时 C++ 写 stone（worldgen_api.cpp:1043），Rust 写 air（terrain.rs:232 `match apply{1=>Water,2=>Lava,_=>Air}`）→ Rust 丢失全部 margin stone。与幕帘无关（幕帘是共有偏离），但属 Rust 单臂 vanilla 偏离，待独立修复微计划。
- **heightmap 判据分叉**：Rust `d>0`（terrain.rs:279）vs C++/Java `非air`（worldgen_api.cpp:1045 / Heightmap.java:24 NOT_AIR）——已另立产物 heightmap-criterion-divergence-260904-06.md（draft）。
- **granite/copper_ore run 写者 idk**：oreVein y≤50 上限排除两臂本地成因；G-A feature height_range 转录错位（vanilla 上界 128/112 到不了 192-318，b3 亲核 JSON）vs G-B id 错位，未裁决。

## 下一步（数据层判别，主会话执行）
1. **P-B1-2 总判别探针**：C++ WG_DBDEBUG vs Rust qaq1_initdensity_cost 口径，同列（195,199）y180-320 剖面互 diff——两臂逐位同 → 共享偏离实锤（再分 B1a jar diff / B1b 移植语义）；两臂不同 → B1 整域证伪（转 B6 核对）。
2. P-B2-1/3（est 三方 dump / 高位 without_jaggedness 剖面）作 B1 辅证。
3. vanilla 参照「y≥201 无 stone」存在性复验（scout P4：防参照误读）。
4. granite/copper 写者判别（WG_FEATURELOG / skip A/B）。

> 命令模板：b1 §5 / b2 §探针 / b3 §5。原始输出落 cmd-output/，解读回 worker/收敛。
