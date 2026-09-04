# Judge 审查意见 — shared 臂开销课题根因定论（candidate）（260903-16）

> 审查角色: core.judge / anchor-judge（subagent，隔离）
> 审查对象: 「shared 臂（WG_EST_SHARED 翻默认）无固有开销；37.67−27.50 的 +10ms 根因 = 测量污染（default 首跑与 gpu_mt_wall_retest 后台并发）」
> 结论: **PASS（附 4 项 CONCERN，均不推翻根因结论本身）** — 建议 status 保持 **candidate**，confirmed 留给人类。
> 本意见不改任何 status。

## 逐项意见

### 1. 产物快照核对 — 通过
- 已读全部五份 cmd-output（p05-cpu-e2e-bench / gpu-mt-wall-retest / qpd1-stage-l2 / cpu-e2e-l2only / cpu-e2e-default-clean）、p05-worker-解读、shared-arm-errors.md、scout-管线地图。
- 三组干净数据齐备且口径声明（§9.7）同行：l2-only 27.50（256ch）、default 干净复跑 28.46（256ch）、qpd1 FULL default 25.87（64ch，口径差异已声明）。37.67 判为孤例离群，证据落盘完整。
- **CONCERN-1（产物契约）**：该 candidate 根因结论未登记 `.artifacts/`（`.artifacts/index.yaml` 无 shared-arm/vulkan-gpu-operator 条目）——仅存在于 .investigations/。建议补一条 index 条目（id 形如 `re-code:vulkan-gpu-operator:shared-arm-rootcause-260903-16`，status: candidate）。

### 2. 证据链自洽 — 通过（判据使用已纠正；遗留注记问题见 CONCERN-2）
- **三组干净数据充分性**：充分。同机同日、同 seed/region、两个 256-chunk 独立臂（l2-only 27.50 / default 28.46 差 <1ms）+ 第三口径 64-chunk 25.87 同水平 → default 与 l2-only 无臂间差异；37.67 是唯一与 GPU bench 同时段的读数。归因唯一性成立。
- **「min 不变 median 变重」判据**：早前被单向解读为 H1（实现尾部开销）证据——这是判据误用：间歇性外源抢占同样产生「地板不变、尾部变重」签名，判据本身对 H1/污染不具区分力。shared-arm-errors.md 教训 2 已正确改写（「归因前先排除并发/环境因素」），纠正到位。
- **worker 早前 H1 判读（27.69 历史吻合）的重新解释**：**不是巧合，仍有解释力，且方向反转为支持新结论**——l2-only 是干净跑（未与 GPU bench 重叠），27.50≈27.69 跨日复现说明 l2-only 臂数值稳定可复现；这恰恰把 37.67 孤立为唯一离群（也是唯一与 GPU bench 共存 wall 的读数）。建议在补记中显式写明这一反转逻辑，避免后人再把「27.50≈27.69 吻合」读回 H1。
- **CONCERN-2（§15.4 supersedes 注记缺失）**：`p05-worker-解读-260903-16.md` §1 与 `p05-cpu-e2e-l2only-260903-16.txt` 判读行仍保留 H1 定论（「本轮基线采用 37.67」「H1 成立」「机器漂移排除」）而无 supersedes 双指针。按结论取代链纪律，原结论不删不改是对的，但**必须有显式取代指针**——目前取代关系只隐含在 default-clean 判读与 errors 台账里。建议在两文件头部各加一行「> ⚠️ supersedes: 本文 §1 H1 判读已被 shared-arm-errors.md E1 + p05-cpu-e2e-default-clean-260903-16 推翻（测量污染），260903-16」。l2only cmd-output 属原始输出，可在头注区加注（不改正文 RESULT 行）。

### 3. 代码侧佐证 — 通过
- `worldgen_handle.rs:555-585`：shared 臂 est_at = 4 次角调用 → `aquifer.rs:343-377 estimate_surface_height`。机制核查：
  - 命中路径：per-chunk `surface_cache` 命中即 O(1) 返回（L348）；miss 先查跨 chunk L2（L350-359），再 miss 才做扫描——扫描循环与非 shared 臂（worldgen_handle.rs:571-577）**逐行等价**（同 min_y+height→min_y、步 8、阈值 0.390625）。即 shared 臂最坏情形 = 非 shared 臂成本，不存在额外扫描量。
  - aquifer 阶段两臂同样跑 `estimate_surface_height`（get_fluid_level 13 offsets）→ SURFACE 阶段 4 角调用大概率已全 cache 命中；增量成本 ≈ 4 次数组读 + ≤2 次 Mutex lock/chunk，量级纳秒。
  - **结论：机制上不可能产生 +10ms/chunk**，与「污染」归因互洽。代码侧佐证成立。

### 4. 遗留风险 / 记录一致性 — 通过（两项卫生项）
- 37.67 污染扩散核查：grep 全工作区，37.67 仅出现在本课题 5 份产物内（原始输出 + errors 台账 + worker 解读）；**NEXT_SESSION.md 无 37.67**（仍写 27.69 历史基线，与新基线 27.5-28.5 兼容，不必改）。未污染其他记录。✅
- GPU (b) 重估基线更正：架构计划-260903-16-vulkan-gpu-operator.md 前置数字 27.69 与更正后 ~27.5-28.5 在同一区间，方向变更记录（状态行指向 shared-arm-cost 计划）自洽。✅
- **CONCERN-3**：`架构计划-260903-16-shared-arm-cost.md` 状态仍为「待批准」，其前提（+10.2ms 真实存在）已被本结论推翻——应比照 GPU 计划加状态行（如「前提被推翻/结案：根因=测量污染，见 shared-arm-errors.md E1」），否则该文件是悬空的活性错误前提。

### 5. 时序疑点复核 — 结论不依赖于此时序（CONCERN-4）
- 量级核算：gpu_mt_wall_retest 全程 ≈ 管线编译 70-100s + n=8 轮 ~2s + **n=6145 轮 5×(serial+parallel)≈2×28.7s×5≈287s** ≈ 6.5-7 分钟；pc_e2e total 9.5s——两者并发窗口在物理上完全可能，「default 首跑落在 GPU bench wall 内」成立。
- **CONCERN-4（时序为重建非实录）**：两份 cmd-output 均无时间戳，「同时启动」是从 background job 使用过程重建的，无落盘实录。此疑点不削弱结论（干净复跑 28.46 是独立证据，归因不单靠时序），但按证据链标准建议：① 在 errors 台账 E1 定位段标注「时序为过程重建，非日志实录」；② 未来 bench 采集文件头注强制带 `[start HH:MM:SS]`/`[end]`，使并发重叠可事后审计（可并入「bench 一律串行」教训的操作化）。

## 需要补的验证
不阻塞 candidate 维持；均为记录卫生项，建议本轮顺手完成：
1. `.artifacts/index.yaml` 补本结论条目（CONCERN-1）。
2. worker 解读 + l2only 输出头注加 supersedes 指针（CONCERN-2）。
3. shared-arm-cost 架构计划加结案状态行（CONCERN-3）。
4. E1 定位段补「时序为重建」声明 + 未来 bench 带时间戳纪律（CONCERN-4）。

## 推荐状态
- 根因定论：**建议 candidate 维持**（三源核对通过：产物快照 ✅ / 证据链三组干净数据 ✅ / 代码机制佐证 ✅；验证执行者=主会话，解读=worker，分层标注与实际相符）。
- confirmed：留给人类拍板。若拍板 confirmed，建议同时把「性能 bench 一律串行 + 归因前排除并发」沉淀为 knowledge/discovered 条目（判据「min 不变 median 变重」对外源负载不具区分力——这是可复用判错经验，符合知识库记录价值门）。
