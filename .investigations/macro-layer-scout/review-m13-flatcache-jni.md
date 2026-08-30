# review-m13-flatcache-jni（core.judge 精简审查，2026-08-30）

基线：git log -3 = `411cae3`（JNI 桥）← `ae3a3ad`（M13 flat_cache 修复）← `6e57100`（扩样探针）；`git status --porcelain` 空，工作区 clean。

## 项 1：M13 修复代码语义（build/density.rs L234-248）—— PASS
- a) 负坐标量化：`(((x as i64) >> 2) << 2) as f64`——i64 算术右移，x=-4607 → -1152 → -4608（floor 到 4 倍数）；与 Java `BiomeCoords.fromBlock`（int `>> 2` 算术右移）语义一致。
- b) y 遮蔽：闭包体 `let x=__fcq_x; let y=0.0f64; let z=__fcq_z; (inner)`——遮蔽作用域覆盖整个 inner（含嵌套 spline_helper_N(noises,x,y,z)），inner 收到的全是量化后的 x/z 与 y=0。量化键缓存（`transpiler_cache_2d(id, __fcq_x, __fcq_z, …)`）保证 cell 内共享。
- c) 域/边界声明存在且合理：docs/07 M13 节 L1103 明确「Java per-chunk 实例（越界 delegate 直算）vs 无状态 transpiler 按 pos 推导量化角（无越界概念）」，并注明生产 in-chunk 恒界内不受影响、诊断跨 chunk 抽查注意——与 flat_cache 分支注释（L235-240）自洽。

## 项 2：JNI 桥逐方法（jni_bridge.rs vs jni_bridge.cpp）—— PASS
- init：C++ `wg_create(seed, dir)` 2 参重载（默认 overworld/384）与 Rust `wg_create(seed, dir, null, null, 384)` 等价——api.rs L64-69 证实 null settings_name/biome_params_file → 默认 `"overworld.json"`/`"biome_params.json"`。
- fillBlocks：两边同为「本地 buffer → 主线程拷回」模式，拷回上限均为 `r.min(count)`（Rust L124 `r.min(count)` ≙ C++ L95 `i<r && i<count`）；length 前置校验等价（C++ 额外校验 outs 长度==count，Rust 未校验——极小差异，JVM 侧契约已保证，不构成问题）。
- setBeardifier：pieces×8 / junctions×3 展开一致（L147/L153 vs C++ 直传数组由 wg_set_beardifier 内解释），空数组 → null 指针传递一致。
- 差异点：C++ `Java_wg_WorldGen_nativeProbe`（L11-14）未在 Rust 复刻——`runtime/1.20.1/java/src` 下无 `wg/WorldGen.java`（只有 `wg/CppWorldgen.java` 与 `wg/bench/*`），该 native 方法无 Java 声明方，**无影响**。

## 项 3：落盘契约抽查（5 个关键数字）—— PASS（5/5 一致）
| 数字 | 记录 | 证据文件 | 一致 |
|---|---|---|---|
| 修前 ch0 内部点 diff=0.065101 | transpiler-errors.md L369 | transpiler_ch0_decompose.txt L5 | ✓ |
| 修后对比1 max_diff=0.000000 + 160/3584 | errors.md L402 | transpiler_exactpoint_verify_after_flatcachefix.txt L2 | ✓ |
| FULL 94.27% match=1482808/1572864 | errors.md L404 | transpiler_prod_vanilla_full_after_flatcachefix.txt L3 | ✓ |
| 块级 99.30% match=1561802/1572864 | errors.md L404 | transpiler_prodblocks_after_flatcachefix.txt L1 | ✓ |
| census FlatCache=363、Interpolated=0 | errors.md L375 | transpiler_ch0_census.txt L2 | ✓ |

## 结论
三项全 PASS，无 CONCERN 级以上问题。**推荐：candidate 维持**（本审查只出意见，不改 status；confirmed 留给用户拍板）。

下一步建议（≤3）：
1. M13 相关结论落盘（07 篇 M13 节 + transpiler-errors.md 台账）已齐备且数字闭环，可提请用户对 M13 修复授予 confirmed。
2. Rust JNI 桥（411cae3）尚未做端到端运行时冒烟（本审查为纯静态）——建议主会话跑一次 Java wg.CppWorldgen fillBlocks 小样本 vs C++ dll 输出对比后再单独 candidate。
3. 可选低优：Rust fillBlocks 补 `outs.length == count` 校验，与 C++ 完全对齐（防御性，非必需）。
