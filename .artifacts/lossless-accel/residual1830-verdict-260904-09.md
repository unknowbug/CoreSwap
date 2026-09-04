# residual-1830 verdict — aquifer splitter 派生缺失（260904-09）
---
编号: residual-1830-verdict-260904-09
任务: 残留 1830 收口（流体族 ~700 + deepslate→air 106）
状态: confirmed（judge PASS 55bd2893；用户实机确认授予 260904-09 21:23）
---

## 结论（candidate）
Rust worldgen aquifer splitter 直传顶层 random_deriver()，漏 `split_str("minecraft:aquifer").next_splitter()`
（Java NoiseConfig.java:54 对应）→ 错误 splitter 种子 → 全部 aquifer blob 随机偏移错 → 液面/距离场全链分叉
→ 干/水双向互换（water→air 314 / air→water 170 / stone↔water 399）+ carver 域投影 deepslate→air 106。

## 证据链
- scout：scout-aquifer-map.md（公式/结构零分歧；缓存臂闭；双向签名预判 D4）
- 判别探针：WG_AQDUMP 12 点，修复前 opq/r/s/t 12/12 全异（judge 独立复跑复现）；修复后全对齐
- 修复：worldgen_handle.rs 一行，commit f41555d
- decisive：全新 world 重导（dll 5E2ACB7F），ref↔off mismatch **1830 → 76**（99.884% → 99.995%）
  - 口径（§9.7）：同 ref 载体 vanilla_8576294172403134396_4_200_200.blocks / 同 4×4@200,200 域 1572864 块 / 同 coltool 直读双臂对比，与 off-fixed-08 基线完全可比

## 残余 76（未修，诚实登记）
gravel→sand 49 / sand→gravel 24 / water→dirt 1 / granite→gravel 1 / gravel→water 1 —— surface 材质微族，
与 aquifer 无关，独立课题未立项（judge 建议：择期低成本 scout，零星 3 块可查 surface 邻接级联）。

## 遗留（judge 清单）
1. 「零开销」措辞 → docs 落盘时改「近零开销（每 apply 1 次 OnceLock 读）」
2. 现役 dll 基线 0D247E03 → 5E2ACB7F + resources 同步，待 confirmed 后执行
3. 残余 76 独立课题择期
