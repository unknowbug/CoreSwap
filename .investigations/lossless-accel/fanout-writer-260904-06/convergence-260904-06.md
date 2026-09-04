# 收敛摘要（260904-06，主会话阶段性记录，非结论性 docs）

## 已裁决
- **写者 = Rust SURFACE（strong candidate，待 judge）**：结构消去链 + E1 运行时确证（WG_SKIP_SURFACE=1 后 (195,199) 列 dirt/sand/gravel 全空）+ E2' 排除 est 路径混淆（shared 开/关 sha 相同）。
- **原「双跑伪影」假设结案**：stageMask=3 下 OOB dirt 依旧 → 双跑非该写者成因；b611fcb 修复在线（dll EC4A9AED 含 wg_set_flags，日志 stageMask=3）。
- **NEXT_SESSION 前提推翻（§15.4 取代）**：「现役 dll 早于 stage-skip 修复」不成立（b611fcb 09-02 01:17 < dll 构建 09-03 23:47）。
- **m3（参照 biome 差）证伪**：C++ -biomeDump (244,-60..-54,244) 七点 = deep_lukewarm_ocean，与 mod SOUL-CTX 一致 → C++ 同 biome 下写 gravel 而 Rust 写 sand = **C++ 侧 surface 规则分支差**（新线索，C++ 侧待查）。

## 新世界图景（E4-lite 列剖面 + E5）
- mod NOISE（与 C++ 共有）在 y≈192-318 产出「stone 幕帘」：stone/granite/copper_ore 交替 run，run 间含水层 water 口袋（col(195,198)：water @ 197,212,228,244,261,277,~16 间距）→ **16 周期 = 幕帘 run + aquifer 流体打断的复合结构**。
- 两臂 surface 都在幕帘上刷 dirt（E2 臂全列 OOB dirt 4618 点）；mod↔C++ 的 43/11 只是作画边界微差。
- **真根因上移 NOISE**：vanilla 该区域 y≥201 无 stone → 「0<d≤0.39 幕帘」是 C++/Rust 共有的 NOISE↔vanilla 分叉（新调查线，未开）。
- **(d) m2 裁决（judge C3 整合）**：est 本体六维零语义差（步长 8/域 [320,-64]/阈值 0.390625/DF=without_jaggedness/哨兵 INT32_MAX/四角 +16）；**唯一真分叉 = heightmap 填充判据**：C++ `block != air`（含水）vs Rust `d > 0.0`（不含水）→ 仅「顶块=水」的开放海洋列两臂扫描起点/fluid_height 不同（(244) 列 sda=48/fluid_height=MIN 即此签名）。195 列（顶块 stone）不受此影响，指向 m1 NOISE 分类边界或规则树内容差 → **新线范围须含：海洋列 heightmap 判据分叉（部分 (244) 类差异归属此处，不能全记 NOISE 幕帘）**。
- **⚠️ 结构写者混杂警示（judge C4，源自 b2 §3.2）**：vanilla ref 列 (195,199) 自有深 dirt @ -59..-42，超出 features 可达域——新线三方列剖面对比 MUST 控制 structures（fossil/trail_ruins/ocean_ruins 等）混杂，避免把结构写入误归 NOISE/surface。
- id 970 = **deepslate**（applied 于 y=-58 ≤0 深板岩梯度，合理）。

## 数据/产物索引
- 探针脚本：.tmp/p2full/{confirm_doublerun_260904-06,ref_col_check_260904-06,e1_check_260904-06,e2_check_260904-06,e4lite_260904-06}.py
- 导出臂：.tmp/p2full/{off,e1-skipsurface,e2-estshared0,e3-ctxdump}\vanilla_...blocks（seed 三查全过，dll ec4a9aed）
- cmd-output/：e1-skipsurface-result / e2-estshared0-result / e3-e5-run-full.log / cpp-biomedump-244col / e4lite-id970-columns
- worker 产物：b1-surface-oob-writer.md（§8/§9 已回填，R1-R4 取代链）/ b2-java-feature-placement.md
- 待 (d)：~~运行中~~ → **已完成**：d-preliminary-surface-cmp.md（est 零差 + heightmap 判据分叉，已整合上节）→ **judge 已过（PASS-with-conditions，C1-C6 补正全部应用）**，candidate 授予见 .artifacts 登记

> ⚠️ 标签勘误：初版命名误用「260906」（日期预推违纪律），**已全部改名为 260904-06 系**（目录 fanout-writer-260904-06、writer-verdict-260904-06.md 等），真实锚 = 2026-09-04 17:35（宿主时间）。
