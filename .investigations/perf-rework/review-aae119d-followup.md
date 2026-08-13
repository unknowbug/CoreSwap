# review-aae119d 跟进（主会话处理 judge 意见）

> judge 审查（sa_20260813_044039）结论：代码语义无损成立，但证据链不闭合 + 术语不精确 + -1.7% 选择性报告。本文逐项处理。

## 1. 零退化证据落盘（已补）

judge 指出「99.9994%/99.9997% 复用上轮 c0ac286 regress 数字，无本提交落盘」。已针对 aae119d 重新跑 block_probe 并落盘：

- `cmd-output/regress_8576_aae119d.txt`：`TOTAL: match=3538922/3538944 (99.9994%) nonAir 99.9986%`
- `cmd-output/regress_3200_aae119d.txt`：`TOTAL: match=1572860/1572864 (99.9997%) nonAir 99.9992%`

两参照均为 `-threads 1` 单线程，与基线（c0ac286 修复后）逐位一致 → **零退化证据链闭合**。

## 2. 术语修正

commit aae119d message 中「root cause is FlatCache buildGrid tree traversal」表述不精确。修正：

- 实际是 **InterpolatedDF::buildGrid**（5×49×5 = 1225 角点，每 chunk 每实例 1 次）的树遍历是大头（phase0-interp-measurement 实测 86.5% 单线程 / 95.2% 多线程）。
- 该树遍历 = 1225 角点 × `arg->sample`（arg = blend_density 的 init 树），其中 **spline 递归 + FlatCacheDF 查表 + noise** 是 cache miss 大头。
- FlatCacheDF::buildGrid（5×5 = 25 角点）只是该树遍历内部的一层缓存构建，不是 density 的 buildGrid 大头。

## 3. 边界列复用 -1.7% 的完整报告（修正选择性报告）

judge 指出「声称③ -1.7% 为噪声级，端到端总 wall 反升 +0.5%，属选择性报告」。完整报告：

| 指标（单线程） | spline 扁平化后 | 边界列复用后 | 变化 |
|---|---|---|---|
| density wall（median） | 47.1ms | 46.3ms | **-1.7%**（接近噪声） |
| [A] threads=1 吞吐 | 71.68 ms/chunk | 72.06 ms/chunk | **+0.5%**（无改善） |

- **结论修正**：边界列复用省了 gx=0 列 245 角点的 `arg->sample`，但 edge 缓存的 resize + 保存开销抵消了收益，端到端吞吐无改善（甚至略慢）。
- **根因**：InterpolatedDF::buildGrid 的 1225 角点采样里，耗时大头是「每 chunk 每实例 1 次的 FlatCache buildGrid 触发」+「spline 树遍历」，这些**不集中在 gx=0 列**（FlatCache buildGrid 只在首个角点触发一次，跳过 gx=0 列只是把它移到 gx=1 列触发，省不了）；gx=0 列其余 244 角点是 FlatCache 查表命中（快），省不了多少。
- **边界列复用收益小是真实的**，不是测量误差——它优化了错误的目标（角点采样而非树遍历触发点）。

## 4. 状态

- 代码语义无损：**成立**（SplineDF Hermite 公式逐位等价、边界复用 CELL_X=4 坐标对齐，静态核对通过）。
- 零退化：**成立**（本节 §1 落盘）。
- 状态：**保持 draft**（同意 judge）——spline 扁平化的单线程 -24% 是真实收益，但「多线程膨胀」课题未闭合，需重新定位根因（InterpolatedDF::buildGrid 树遍历的 cache miss 构成）后再评估 DFC/其他方案。
