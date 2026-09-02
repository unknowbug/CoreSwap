# P0-① transpiler vs vanilla 1.2pp 归因（260903-02）

status: candidate（judge 审查中；confirmed 留人类）
验证分层：Partial（块级管线对比 + 密度级既有 98304 点 `{:.6}` 口径，非 bit 级）
§9.7 可比性：载体 = Rust dll wg_fill_blocks 块输出 vs vanilla FULL 参照（seed -8248…，16 chunks，实际坐标 (-18..-15,-16..-13)）；覆盖面 = 4×4 chunk 全 y；与 07 篇 M12 既有 94.27/95.40 快照同源同工具（handle_probe）。

## 结论

transpiler vs vanilla FULL 94.27% vs 基线 macro 95.40%（-1.13pp）**不是实现 bug**，归因链闭合：

1. 上游：transpiler 密度 vs macro 密度 max_diff < 5e-7（`{:.6}` 舍入口径，98304 点全 0，07 篇既有）——浮点求和顺序/缓存实现（cache_all_in_cell 点级 vs Java cell 级）产生微残差。
2. surface 级：近零阈值块翻转 → 块级 99.30%（11062 块，0.70%）。
3. FULL 级：carver/features 级联放大 2.7× → td vs ms 差异 30262 块（98.08%）；分解 broke=19111 / gained=1387 / both_wrong=9764。
4. 恒等式：net = broke-gained = 17724 ≈ match_ms - match_td = 17725（差 1 块 = 已知同 dll 重跑非确定容差）→ 归因算术闭合。
5. 单向性解释：ms 与 vanilla 的匹配块集中在密度远离阈值处，翻掉的多；ms 错块翻对少 → 净 -1.13pp，符合残差翻转统计而非系统性语义差。

## 影响

- P2a 验收基线：可用 macro sampler 路径（与 transpiler 等价性已证到 <5e-7 + 级联解释）；1.2pp 欠账结清。
- 加速对比时该 1.13pp 属「固有残差敏感性」，不应记为实现回归。

## 证据

- `.investigations/lossless-accel/cmd-output/transpiler-full-cascade-26090302.txt`（FULL 分解输出）
- `WorldgenRust/src/bin/transpiler_prod_blocks.rs`（新增 WG_FULL_MODE=1 归因模式，正式 bin 编译绿）
- 既有：transpiler_prodblocks_after_flatcachefix.txt（99.30% surface 级）、handle_probe 两次运行（94.27/95.40）

## 附带发现（转 knowledge）

`.bak.blocks` 参照文件 header origin (-288,-256) 与实际内容坐标 (-18..-15, -16..-13) **不符**——header origin 不可信；跨工具配对必须用文件内 chunk 坐标（handle_probe 用文件内坐标故历史对比自洽，未被污染）。

## judge 审查记录（260903-02）

- **结论：accept-with-notes，candidate 维持**。三源核对通过；恒等式差 1 块属同 dll 重跑非确定容差（同 run 内数学恒等，17725 取自另一次 handle_probe 运行）；carver RNG 溢出对两路径同等影响（级联是「基底块差改变落点」非 RNG 分叉）。
- **N1（confirmed 前补强，非阻塞）**：broke/gained 14:1 不对称目前是相容断言非证据——需对 broke 与 gained 块采 |density| 离阈值距离分布对比。
- **N2（可选）**：探针同 run 直接算 match_ms/match_td 使恒等式精确闭合。
- N1/N2 完成前本结论保持 candidate。
