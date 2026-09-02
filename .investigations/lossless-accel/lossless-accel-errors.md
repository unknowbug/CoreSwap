# lossless-accel 错误台账（260903-02 立）

> 五段式：现象 → 根因 → 定位 → 修复 → 教训。末尾附「错误→根因」速查表。

## LL1. Rust 侧 MT3 同款 clamp 结构性串行（P0-② 欠账清偿，260903-02）

- **现象**：架构计划 §0 预置核对项——C++ `worldgen_api.cpp:1323` 有 `if (threads > count) threads = count`（MT3，count=1 时池恒 1 worker = 实机 M=1 结构性串行）；Rust 侧是否同款待核对。
- **定位**：grep `WorldgenRust` → `src/api.rs:38` `threads.min(count).max(1)`（env 覆盖分支 L27 同语义）——同款确认。
- **修复**：`api.rs` adaptive_threads 尾行改为 `if count > 1 { threads.min(count).max(1) } else { threads.max(1) }`——count=1 不 clamp，池按请求线程数建 worker 并保持；count>1 语义不变。`cargo check --lib` 绿（仅既有 267 warnings）。
- **状态**：代码级修复完成（candidate）；实机/批量性能影响随 P2a 端到端验证一并确认。C++ 侧同款修复仍是 worldgen-mt-scaling 课题 candidate 待办（本课题不动）。
- **教训**：跨语言移植的池化/调度参数逻辑（clamp/自适应）是同款 bug 高发位——移植核对项应 grep 两侧同语义表达式而非只看函数名。

## LL2. 参照文件 header origin 与内容坐标不符（260903-02，P0-① 探针踩坑）

- **现象**：FULL 归因探针按 header/文件名假设的 origin (-288,-256) 生成对比 chunk → vanilla 配对全 miss（分解计数 0，与同运行 12321 差异块矛盾）。
- **根因**：`vanilla_..._4_-288_-256_FULL.bak.blocks` 的 header origin 字段与实际 chunk 坐标（-18..-15, -16..-13）不符；文件名/注释同被误导。
- **定位**：同运行内恒等式自检（match 差 vs 分解差）矛盾 → python 直读参照文件逐 chunk 坐标（`/tmp .tmp/refkeys.py` 范式）→ 实际坐标曝光。
- **修复**：探针改按参照文件内坐标生成与配对；handle_probe 历史对比因「用文件内坐标生成」本就自洽，未污染。
- **教训**：参照五要素之外，**header 字段本身也可能是错的**——配对/对比永远以文件内容实测坐标为准；探针必须带恒等式自检（本例 0 vs 12321 矛盾 5 分钟暴露假配对）。

## 速查表

| 错误 | 根因 |
|---|---|
| LL1 Rust MT3 串行 | `threads.min(count)` 在 count=1 时把池 clamp 到 1 worker（C++ 同款移植） |
| LL2 归因探针全 miss | 参照 header origin 与内容坐标不符；配对用了硬编码坐标而非文件内坐标 |

