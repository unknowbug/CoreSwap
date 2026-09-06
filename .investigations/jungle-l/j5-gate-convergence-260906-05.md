# J5 门层收敛记录（260906-05 · 中间产物，主会话写）

> 状态：进行中（Phase 2 收敛 + 残余分叉待 fan-out）。
> 数据：seed 8576294172403134396，chunk (29,-16)；载具 = j5_tree_trace native（bin-diag），5×5 邻域。
> 验证分层：数据层探针（TPFAIL 探针 + WG_CA_MIN A/B），Full 级取证、结论 Partial（残余未闭）。

## 已验证事实（数据层）

> ⚠️ **E10 作废标注（260906-05，§15.4）**：事实 1 的对照对象（Java 树 A）属旧 seed 污染 pass——「同位同高」对齐结论与「.b2 相位候选对该 attempt 不成立」随 J5 基线作废一并失效；残余分歧节（R1/R2 + 判别实验）的分歧前提（Java=2 棵）已被 E10 推翻，待 J5 基线重采后重新评估。事实 2-6（Rust 侧 TPFAIL/-1/ca_min 行为）不依赖 Java 基线，仍有效。

1. **fan-out 首尝试对齐**（⚠️ 对照对象不成立，见上方 E10 标注）：ca_min=off 原始 trace 中，Rust 对 (464,71,-252) 的 mega attempt 与 Java 树 A **同位同高（[TH] 27）**——decorator seed 按 feature 索引派生、与执行序无关，选择层/RNG 相位在该 attempt 上对齐（.b2 相位候选对该 attempt 不成立）。
2. **失败点 = get_top_position 门层**：[TH] 之后无 [MJT0]/[MJTD]——④域守卫过、⑤扫描拒绝。TPFAIL 临时探针（tree.rs :704，env 门控）实锤：mega 扫描壳命中 `(463,71,-253) id=-1`（跨 chunk 不可读）×2 → can_replace 保守拒绝。
3. **-1 来源 = WG_CA_MIN 默认关**（worldgen_handle.rs:924，#53 家族 env 默认值）：ca_min=off 时 block_at 跨 chunk 返回 -1（judge C-3 已登记的已知语义偏差「越界保守拒绝」；Java 读邻 chunk post-carver 实况 → 空气可替换 → 通过）。ca_min=on 重跑：id=-1 TPFAIL 清零。
4. **ca_min A/B 流移动**：ca_min 开关改变前置 attempt 的消费量（失败早退 vs 完整生成），后续 [SQ] 位置整体移动——ca_min=on 流中 (464,71,-252) attempt 不复存在，目标 chunk mega = (478,71,-246)h16（1 棵）；Java = 2 棵（464/479）。
5. **橡树叶非抢占原因**（推翻 .b1-D2 方向的中间解读）：终态 dump 基座 oak_leaves 是 mega 失败**之后**小树放置的结果，非原因。
6. 临时探针保留声明：tree.rs TPFAIL 打点为 env 门控 + 失败路径执行（非热路径每点），随本课题保留；生产 dll 发布前如无必要可移除（无 WG_CA_MIN/WG_TREEDIAG 时零行为变化）。

## 残余分歧（未闭，≥2 互斥候选 → 待 fan-out）

> ⚠️ E10 后本节前提已变：用户实机 11 点对拍（260906-05，vanilla 正确 seed 地面真值 vs Rust native ca_min=off 清单）：
> **严格命中 3/11**（(439,71,-214)、(469,71,-230)、(483,71,-230)，2×2 基座逐位一致）+ **近似命中 2/11**（(436,70,-232) 偏移2、(445,71,-222) 2 格外巨树）+ **未命中 6/11**（(460,72,-229)/(473,71,-217)/(477,71,-234 位置为矮树)/(493,72,-230)/(493,71,-212)/(505,72,-261)）。
> **判定（candidate）**：mega 放置流大方向对齐（严格命中不可为巧合），残余 = 流漂移（前段 attempt 对齐、后段错位/丢失）——与「读取门控差异（-1 保守拒绝/WG_CA_MIN）造成单次消费差 → 后续 [SQ] 整体错位」的相位漂移签名一致，非结构性 placement bug。修复方向候选：WG_CA_MIN 默认翻转（须先过 golden 逐位不变对照 #47）+ 干净 Java 基线重采（第四查 pop 绑定）后量化对齐率。

ca_min=on（读语义≈Java post-carver 邻域）后 Rust 与 Java 的目标 chunk mega 集仍不一致：
- **候选 R1**：Java FEATURES 阶段邻 chunk 读取时序实际**含先期已生成的 feature 产物**（region 内按序生成、邻块已 features）——neighbor_terrain（无 feature）语义仍不完整。
- **候选 R2**：上游 biome 判定 / feature 步序 / placed feature 集差异，导致 attempt 流根本性不同（与读取时序无关）。
- 判别实验设计（待执行）：Java 侧对同 chunk 打 [SQ]/[CNT] 全序列（.b1 §4 探针），与 ca_min=on Rust 流逐 attempt 对齐，首个分叉 attempt 即定位层。

## fan-out 候选文档处置
- .b1（attempt/selector 计数差）：字面计数层静态对齐 ✓；D1（heightmap 静态快照）降级——本块实证主因是 id=-1 保守拒绝（比 heightmap 更上游的读取层）；D2（soil 误拒）证伪（grass_block 排除是 is_soil 语义，非 bug）。
- .b2（RNG 相位差）：主干派生链静态对齐 ✓；对首 mega attempt 相位差被数据排除；D1（退化 uniform 0 消费）保留为低先验静态候选（未判别）。
- 知识库新候选沉淀：①「已知语义偏差 flag 的默认值当公理」#53 家族第三例（WG_CA_MIN）；②「树放置门层 TPFAIL 首失败块探针」判据；③「跨 chunk 读取语义差制造 placement 分叉」机制指纹。待 judge 后由 subagent 草稿。

## 产物索引（260906-05）
- 探针/重跑：.tmp/jungle-l-260906/{j5_gate_probe.rs, j5_gate_probe.exe, j5_tree_trace2.exe, gate_probe.out.log, trace2.err.log, trace3-camin.err.log}
- 候选：.investigations/jungle-l/j5-b1-attempt-count-candidate.md / j5-b2-rng-phase-candidate.md（均 draft）
- 代码改动：worldgen-core/src/tree.rs（TPFAIL 探针，见事实 6）；worldgen-core/src/bin-diag/nether_region_dump.rs（CONCERN-2 工具收编，编译验证 OK）
