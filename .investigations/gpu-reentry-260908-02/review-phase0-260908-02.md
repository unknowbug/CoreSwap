# core.judge 审查意见：phase0-architecture-260908-02（aquifer est 冷路径双路线架构提案）（260908-02）

> 审查角色：core.judge（subagent，只出意见不改 status；confirmed 留人类）。
> 审查对象：`.investigations/gpu-reentry-260908-02/phase0-architecture-260908-02.md`（draft）。
> 证据链：同目录 convergence / addendum-denominator / worker-aquifer-input-face / worker-c2me-aquifer-emitter / review-convergence / scout-a/b/c；`.artifacts/perf-rework/gpu-phase1-verdict-260908-01.md`（confirmed）；`.investigations/perf-rework/gpu-accel-errors.md`（A2、D24/D25/D26/D27、G 系列）。
> **裁定：PASS-with-conditions**——场景二分有数据支撑、fp64 gate 设计正确前置、路线 B 收益声明守住「命中率未知」边界、与 verdict-260908-01 无取代冲突、未发现 D26 式直觉预估；但 gate 对比式漏了端到端固定/互斥成本、两处口径待验项未随行继承、冷路径暴露面量化缺位。J1-J5 条件修订后即可交用户 HOOK。

---

## 审查点 1：场景二分的推理支撑与第三场景 —— **基本通过（附 J1/J2）**

- 暖区 1.02ms 与冷态 35-37ms 的两臂各有独立实测来源（addendum §A 四臂表 estopt_mt_bench 差分；qpd1_stage_bench 单 chunk 冷态），且 addendum 注记 2 已做交叉自洽验证（9.70/10 ≈ 0.97 ≈ 1.02，并行扩展良好）——二分本身有数据支撑，非直觉切分。
- **第三场景确实存在**：新区域连片预生成时 est L2 命中率从 0 爬升到 ~90% 的**中间态**（region 首 N chunk 冷、后续渐暖）。注记 2 只声明了两端口径不矛盾，未覆盖爬升带。但核对后果可接受：双路线的优化目标都是「压冷态」——中间态本质是冷态与暖态的按比例混合，不产生第三种优化形态，架构不需要为它新增路线。**J1**：B-1 前置计数探针的采样应覆盖爬升带（region 首 N chunk 的逐 chunk 命中率曲线），不能只测稳态暖/冷两端——否则 B-1 收益上界在「最需要它的连片预生成场景」反而没有数据。
- **J2**：架构文档继承了 convergence 的结论但丢了 addendum §C 的一条前置：「先钉冷路径在真实使用中的暴露面（玩家实际遇到多少冷 chunk）」。冷态 35-37ms 即使压到 0，若真实游玩中冷 chunk 极少，课题总收益仍存疑。建议把「冷路径暴露面量化」列入前置探针清单（可与 B-1 计数探针同轮），或至少在文档 §一声明该项开放。

## 审查点 2：路线 A fp64 gate 与最小切片边界 —— **gate 设计正确前置（附 J3）**

- fp64 必要性推导与 A2 台账一致：/o 结构放大（2^15）→ ~35 位坐标精度 → float 24 位硬上限不可救（A2 明言「坐标拆分救不了」）→ fp64 必要。文档把 GeForce fp64 = 1/32 吞吐**显式写成 gate 而非隐性假设**（§二「前置 micro-bench（gate，非承诺）」），且给了廉价判别式（fp64 单价 × 34×256 列 vs CPU 冷价 ~60µs/列，一轮微测可判；60µs/列 = 15.4ms/256 列，算术自洽）——这正是 D27 教训「摊销曲线实测先于架构投入」的正确继承形态。编译时间风险（G 系列）也按 noodle 44 函数×1.6KB 基准显式控制。
- 最小切片边界与 C2ME 形态一致性：worker-c2me（源码级，vendored HEAD 615baf8）证实 C2ME aquifer 路径绕不开 initial_density 专用 kernel（addendum §B.5），worker-aquifer-input-face 证实 `initial_density_without_jaggedness` 同时服务 est + surface——「只做一棵树」的切片与两份 worker 证据一致，且明确排除 apply 链与整树 final_density（与 D24/D25/D27 负面结论域不重叠）。
- **J3（必改）**：gate 对比式只比「fp64 采样单价 × 列数 vs CPU 冷价」——**漏了端到端固定/互斥成本**：dispatch/readback/上传非零（D27 证明非主项但非零，且该结论绑定角点+解释器形态，对本 kernel 形态不可迁移）、以及与生产 T=10 多线程管线共存的 mutex 串行化开销（D24 P2-4 实锤驱动崩溃 → 串行化「正确性解决但性能更劣化」）。这正是上一轮 judge C3b 已对 convergence 提出的同一缺口，addendum 继承了它但架构文档的 gate 式没带上。修订：gate 判据改为「GPU 侧每 chunk 端到端成本（kernel + dispatch + readback + 与 CPU 线程互斥）< CPU 冷价对应口径」，micro-bench 至少量到 dispatch+readback 往返，不能只量 kernel 内核。
- 轻提醒（不改条件）：kernel 内若含 flat_cache/cache_2d 节点，坐标对齐语义（B2/C1 家族历史失败模式）应在 kernel 设计阶段显式处理——est 列采样恰是 biome 对齐坐标场景。

## 审查点 3：路线 B 收益上界诚实性 —— **通过**

- 「est 冷 15.4 + 冷 miss 6-8 ≈ 21-23ms」明确标注为**收益上界**，且 B-1 命中率显式声明「取决于坐标重复度，需一轮计数探针定命中上界」——守住边界，无过度承诺。B-2 依赖的 L2 容量数字（131072 ≈ 4370 chunk、evictions=0@256）与 addendum 注记 1 一致。B-3「语义零风险」限定在预计算保值语义上，工程风险（线程/生命周期）另行声明，划分恰当。
- 「est 扫描循环不可改（逐位对齐铁律）——只允许缓存/记忆化/预取类保值优化」表述正确，与项目逐位验证纪律（block_probe Full 层）一致；B-1 的 old_blended 记忆化为 pure function 无宿主缓存的前提与 worker-aquifer §B 冷价归因（old_blended 无缓存为主）自洽。

## 审查点 4：与 verdict-260908-01 的范围关系与 D27 判据继承 —— **通过（附 J4/J5）**

- 范围：verdict L5 confirmed 范围 = noise/density 现引擎形态负面；L29 选项③ =「特定子管线/离线预生成」显式留白。本架构 = est 冷路径子管线，落留白③内；文档 §首行有显式范围声明（review-convergence C1 已应用的补强被正确继承）——**无 §15.4 取代冲突**。
- D27 判据继承：fp64 微测 + B-1 计数探针均为「廉价先行判据」，先出数后投入，与 D27 教训 1「摊销曲线实测先于架构投入」同构。**J4**：D27 教训 2（GPU 门槛随 CPU 基线优化同步抬高）只被隐式覆盖——B 系列若先落地压掉冷态大半，A 的回本窗口同步收缩，§四「两路线不互斥」应补一句「B 落地后 A 的 gate 须按新冷态分母重算」，防止 B 做完后 A 按旧 21ms 分母续推。

## 审查点 5：D26 式直觉预估混入检查 —— **通过（附 J5）**

- 逐条扫描：路线 A 全部收益表述挂在 gate 后（「gate，非承诺」）；路线 B 收益挂计数探针后（上界声明）；无「结构性优化应大幅提速」式断言。D26 教训（先 benchmark 钉主导成本）在两个 gate 上均被遵守。
- **J5（轻）**：§一第 8 行「est 冷扫描 ~15.4 为主」未随行继承 review-convergence C2a 的口径注记——15.4 + 6-8 + apply 5.5 ≈ 27-29 ≠ 35-37，残差 ~7-9ms 未归因（两口径不同源）。补一行「构成分解与总量残差未归因」防止读者误做加法。同族：暖区 1.02ms 分母依赖 addendum 注记 1 的 `l2=true` 默认值方向（待 api.rs 直读确认）——该待验项未随行标注。

---

## 条件清单（修订后可交用户 HOOK）

| # | 级别 | 条件 |
|---|---|---|
| J3 | **必改** | 路线 A gate 判据补端到端成本：dispatch/readback/上传 + 与 T=10 管线 mutex 互斥开销纳入对比式；micro-bench 至少量到 dispatch+readback 往返，不能只量 kernel 单价 |
| J1 | **必改** | B-1 计数探针覆盖 L2 命中率爬升带（region 首 N chunk 逐 chunk 曲线），不只测稳态两端 |
| J2 | **必改** | 补「冷路径真实暴露面量化」为前置探针（可与 B-1 同轮）或在 §一显式声明该项开放 |
| J4 | 应改 | §四补「B 落地后 A 的 gate 按新冷态分母重算」（D27 教训 2 显式化） |
| J5 | 应改 | §一补 C2a 残差口径注记 + 暖区分母 l2 默认值待验随行标注 |

## 推荐状态

- **维持 draft**；J1-J5 修订后再走一轮轻量核对即可交用户 HOOK（架构批准点）。
- 方向面肯定：双路线同页对比 + 双探针前置的架构形态，是对 D24/D25/D26/D27 四个负面教训的系统性正确回应；fp64 gate 与「est 循环不可改」纪律表述尤其扎实。

— core.judge（260908-02），只出意见，不改 status。
