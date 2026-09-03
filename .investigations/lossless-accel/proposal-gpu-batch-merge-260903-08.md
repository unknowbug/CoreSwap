# 提案：GPU 批量合并优化工作包（gpu-batch-merge）（260903-08 立项讨论，未开工）

> 状态：**提案（proposed）**——优先级已排入 NEXT_SESSION（P-C 之后、欠账之前），开工前须过 Phase 0 架构确认。
> 来源：260903-08 P-C1 端到端判读 + 用户「GPU 预热/预计算」讨论（两条合并路径的评估结论）。

## 背景数据（260903-08 实测）

- GPU 通路逐 chunk 小批量 fill = 369ms/chunk（慢 CPU 管线 72ms 约 5×）；成本结构 = 每 fill 固定成本（CPU split + upload + 5 通道 dispatch+readback fence 阻塞）× 次数。
- 8 chunk 批（6144 点/fill）稳态已折算 172ms/chunk——**攒批本身已砍半，未做任何其他优化**。
- 预热假说检验：预热收益仅 ~20-25%，非主因；「保持预热」的正确工程化 = 队列里永远有活（流水化），不是提前跑热。

## 方案评估（用户提出两路径 + 会话讨论结论）

### ② CPU 侧攒大块提交（主路径，先行）
- 攒批摊薄固定成本 + dispatch 资源复用（command pool/fence 重建，route2 judge 已列待办）+ **异步 readback 双缓冲**（CPU 提交 N 批 / GPU 算 N-1 / 读 N-2，消除 fence 阻塞串行）。
- 已有数据支撑：批粒度 1→8 chunk 成本减半；vulkan-proto 批量 256 chunk 吞吐上限 ~10000 chunk/s（vs CPU 14）。

### ① GPU 预运算大块缓存（互补，②之上做）
- 形态 = 有界 prefetch ring（按移动方向预取 N chunk + LRU）；缓存本体小（~15KB/chunk 角点密度）。
- 三个约束：① 玩家按需生成的预测性（仅 pregend/基准稳赚）；② 边界 blending + 缓存键（seed/维度/stage）管理；③ **Amdahl 天花板——GPU 只覆盖 density 角点采样段**，aquifer/surface/features 全在 CPU。

### 优先级判据
- **Q-PD1（Rust features/carver 段 vs Java 2.2× 差距归因）必须先做**：若 features 段是大头，GPU density 优化的端到端收益被 Amdahl 截住，①的取舍也依赖此结论。

## 开工第一步（廉价判别实验，½ 天内）
**n 扫描**：768/6144/49152/196608 点 × 各 ≥20 次 fill（复用 gpu_mt_wall_retest 骨架），出「固定成本 vs 每点成本」分解曲线。一次实验同时回答：
1. ② 的收益上限（渐近线 = 纯 GPU 吞吐）；
2. `split()` CPU 每点成本何时成为新瓶颈（届时 split 挪 GPU）；
3. 最优批粒度初值（chunk/批）。

## 明确不做（提案边界）
- 不动生产默认路径（零退化铁律；仍 env 门控）。
- prefetch ring（①）不在本包首期，Q-PD1 归因后再决定立项与否。
