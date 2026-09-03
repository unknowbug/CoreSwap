# gpu-batch-merge 天花板重议（P3.1，260903-12）

- id: `re-code:lossless-accel:gpu-merge-revisit-260903-12`
- session: 260903-12
- status: **candidate**（决策建议，待用户拍板）
- 依据：est L2 优化落地（260903-11 confirmed + 本 session P2 复核）后的端到端新基线。

## 新基线（§9.7：同 region/size，256 chunks median）

| 路径 | 吞吐 | 来源 |
|---|---|---|
| Rust off（旧默认） | 75.94 ms/chunk | pc_e2e 260903-11 |
| **Rust l2（候选新默认）** | **27.69 ms/chunk** | pc_e2e 260903-11 + 本 session P2.1 复核（T=1 口径 35.8，同向） |
| Rust l2 8 线程 | ~4.5 ms/chunk | estopt_mt_bench 本 session |
| Java vanilla FULL | ~32-33 ms/chunk | WorldGenBench 260903-09（run B，fresh world） |

## 判断

gpu-batch-merge 立项时的目标差距（Rust 全管线慢于 Java，72-77 vs 33，260903-08 记录）**已被 est L2 无损优化消除**：单线程 27.69 < Java 33（快 ~1.2×，恢复 260903-10 大样本结论方向）；8 线程吞吐 ~4.5 ms/chunk ≈ **7× Java**。
⚠️ 可比性标注（judge D）：Java 33ms（WorldGenBench fresh world）vs Rust 27.69（pc_e2e）为**跨 bench 近似比较**——harness/预热/样本口径不同，1.2×/7× 两句引用均应视为量级判断而非精确比值；方向稳健性依据 = 保守口径 T=1 35.8 同量级 + 260903-10 大样本同向 + 260903-08 confirmed 独立记录 33。

## 建议

**gpu-batch-merge 降级为「低优先级保留」**（用户拍板 260903-12：不删除——未来确有「超越 Java 10×+」目标，届时 GPU 批量合并是候选路线）：
- 当前不立项：追平 Java 已由 est L2 无损路径达成（27.69 vs 33ms/chunk），GPU 小批量 dispatch/readback（369ms/chunk）在现有目标下为负收益。
- **重议触发条件（用户确认）**：目标升级为「大幅超越 Java 10×+」时重新立项——届时的量化门槛：8 线程 CPU 吞吐 ~4.5ms/chunk，10× Java（~3.3ms/chunk）需 GPU 批次摊薄后 readback+dispatch 摊销 < ~1ms/chunk 且计算侧不劣于 CPU（56 chunk 批次级摊销起算）。
