# 形态审计候选池（form audit candidate pool）

- 工作块：260915-01（P3 汇总，主会话收敛型分析——四份 P2 worker 行集合并不重排序）
- 计划：`.investigations/000-架构设计/架构设计-260914-04b-形态审计.md`（HOOK-1 已批）
- 输入：`.investigations/form-audit-260915-01/`（p1a/p1b scout + p2-w1/w2/w3/w4 行集，均已主会话原文核对）
- 状态：**confirmed（2026-09-15 用户「授权确认」；范围 = 候选池 CP-1..6 排序 + 矩阵等价判定 + D-hm 结案；验证分层 Degraded 静态如实保留——各 CP 修复实施后的行为级结论另走各自验收节点）**（judge PASS-with-conditions C-1..C-7，C-2/C-6/C-7 已应用，C-1/C-3/C-4/C-5 绑定后续工作；HOOK-2「按建议执行序全批」）
- 验证分层：**Degraded（全静态源码对照 + 上限推演；无新运行时采集）**——所有量级数字为推演上限，假设已注明于各 worker 文件
- 覆盖面声明（§9.7）：1.20.1 全接管面（生产 3+1 段；carver/features/biome/序列化让位 vanilla 不在矩阵）；**1.21.6 差异未覆盖**（仓库无一手源，P1b 降级声明）；#107（已修）与 #122 家族历史不在本池重复立项

---

## 1. 汇总矩阵（合并交叉项后）

| 交叉项 | 各 worker 命中 | 结论 |
|---|---|---|
| FIFO 优先级丢失（ticket level 不感知） | W4-R3 ＝ W1-A-②b2 ＝ W3-D-② 随行 | **同一发现三角度独立命中**（互为印证，嫌疑加权） |
| 光照同步内联粘 worldgen 车道 | W2-L1+L2 ＝ W4-R5-light | 同体两面 |
| ×9 全量重算 + 无跨 chunk 缓存 | W2-L5+L6（+L4 附） | 单 worker 但证据链完整（代码事实 + round3 实测拼合） |
| RefCell UB 面 | W2-L3；W4-R5 提示「车道串行化事实上规避」 | **与光照线程放置修复耦合**：若先解除粘线，UB 即暴露，须同步修 |

## 2. 候选池（嫌疑度排序，draft）

### CP-1 光照增量形态对齐（L5+L6+L4）
- 形态：3×3 全量重算（结构总量比 ≥9×，BFS 稀疏性后有效工作量比 ~9×–数十×）→ vanilla 种子化增量 BFS + 跨 chunk 状态共享（heightmap 直填 + 边界种子 + LightStorage 式持久层）。
- 收益：① 性能——算力冗余消除，任何并行/池竞争场景直接乘进占用；② 正确性——**疑似 G3 首载漂移 7× 同源主候选**（时机形态 (i)），一箭双雕。
- 量级：大工程（跨 chunk 状态层是增量化前置）。
- 分辨探针（立项前）：G3 同源 (i) 时机 vs (ii) UB 可用「单线程强制复跑 gate ON 首载」一轮分辨。

### CP-2 光照线程放置解粘（L1+L2，.b2 主候选）
- 形态：同步内联全链（每 chunk 2.94ms 占住 worldgen 车道线程）→ vanilla 形态复刻（异步两段 + 独立逻辑队列，worldgen 车道零占用）。
- 收益：e2e 7% 回退首席归因候选（串行反超 1.9× 与 e2e 慢 7% 签名自洽）。
- 量级：中（Java mixin 侧重构调度形态，Rust 内核不动）。
- ⚠️ 耦合：解除车道串行化后 CP-3 UB 暴露，**两项必须同批**。
- 分辨探针：mixin 内线程 id + 粘线分布 / 池利用率计数（.b1/.b2/.b3 分叉判据已备于 W2 §2.1）。

### CP-3 LightEngine.scratch RefCell 并发 UB（L3）
- 形态：`Mutex<Scratch>`（单次 compute 全程持有，开销可忽略）或 per-thread handle。
- 收益：正确性风险消除（错值/崩溃面），成本近零。
- 独立可先行：即使 CP-1/CP-2 缓做，本项也应即刻做（风险向、近零成本）——除非确认车道串行化长期保留。

### CP-4 自有池接入 ticket 优先级（R3/A-②b2/D-②）
- 形态：LBQ(128) FIFO 一次定序不可重排 → 复刻 ChunkTaskPrioritySystem 语义（按 ticket level 排序 + 入队后重排/可取消），或退而求其次（优先级队列 + 移动时重插）。
- 收益：G3 首载 7× 家族最强候选（FIFO 队深上限 13.8× / ~0.69s，依赖「提交序与优先级序错位」假设未验证）+ 响应延迟（b2 最坏 128×T_fill）。
- 量级：中。
- **前置**：需首载期 fill 完成序 trace（vs chunk 距玩家距离）先闭合归因，再立项修复——静态层无法定夺（W4 明示）。

### CP-5 #122 池宽死参数三处同族（R1）
- 形态：`resolveExecPoolSize` / `resolveMaxInflight` / `adaptive_threads` 共用 logical/2-2 单机 bench 启发式 → 参数化/按物理核自适应。
- 收益：低核机（4C 逻辑 4）fill 钳 1 线程，上限 ~3×；高核机不触发。发行面（玩家机器多样性）价值。
- 量级：小。

### CP-6 D-④-b needsSaving 标脏完整性（W3 主嫌疑）
- 形态：bulk 原地替换不经 setBlockState/标脏链 → 早 unload 存盘场景生成方块可能不落盘（推理级）。
- **前置**：一轮源码核对可闭合（TACR:797-802 门控 + ProtoChunk 脏标记调用方）——正确性向，核对优先于立项。

## 3. 登记不立项（低嫌疑/归属外）

| 项 | 理由 | 归属 |
|---|---|---|
| R2 LBQ 不可取消 + CallerRuns 有界阻塞 | ≤0.64s/突发窗口、~50ms 有界，非倒置（W4 修正直觉误判） | 登记，随 CP-4 顺带评估 |
| D-①-b serialize 20/tick 成新瓶颈 | 仅 >400 chunk/s 生成速率触发 | e2e 候选池登记 |
| A-⑤ ChunkNoiseSampler 让位段全量重建 | 辖区外（vanilla CARVERS 侧行为），只登记事实 | 让位段课题 |
| D-hm heightmap 补 6 型条件等价 | 条件交 judge 抽查（本轮未逐行核） | 本审计 judge 项 |
| A-②b1 稳态吞吐 2.3× | 依赖饱和场景 + T_r≪T_j 时不触发；T_fill 无实测 | 随 CP-5 评估时补 T_fill 实测 |

## 4. 立项前预验证清单（廉价探针，裁互斥分叉）

1. 首载期 fill 完成序 trace × chunk 距玩家距离（裁 R3 分支 (a)/(b)，定 CP-4 归因）
2. 单线程强制复跑 gate ON 首载（裁 G3 同源 (i) 时机 vs (ii) UB，定 CP-1/CP-3 权重）
3. mixin 线程 id + 粘线分布探针（裁 .b1/.b2/.b3，定 CP-2 归因）
4. T_fill 实测（W1 A-② 系列置信度的最大假设依赖）
5. D-④-b 源码核对（TACR needsSaving 门控 + ProtoChunk 标脏调用方）

## 5. 建议执行序（draft，供 judge/HOOK-2）

CP-3（近零成本风险消除）→ CP-6 源码核对（一轮闭合正确性疑点）→ 预验证 1/2/3/4 → 按 probe 结果定 CP-1/CP-2/CP-4 排序 → CP-5（小）随批。光照 round4 纯算力项（解码融合等）继续冻结，待 CP-1/CP-2 立项裁决后让位或并入。
