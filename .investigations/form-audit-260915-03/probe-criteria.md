# 预验证探针 1-4 判据预登记（260915-03）

- 状态：**预登记**（与插桩/采集脚本同批定稿——#150 判据纪律；本文先于任何采集数据存在）
- 计划：`.investigations/000-架构设计/架构计划-260915-03-预验证探针.md`（已批）
- 输入分叉判据源：`.investigations/form-audit-260915-01/p2-w4-executor-rows.md` §R3、`p2-w2-light-rows.md` §2.1/§2.3
- 日期锚：Get-Date 2026-09-15 16:53（块 260915-03）

## 0. 载体与自证门（全部臂通用，缺一臂判 VOID）

- 诊断插桩：`wg.bench.FormProbe`（`-Dcoreswap.formprobe=1` 事件行 / `-Dcoreswap.formdrive=1` 虚拟玩家票据驱动）；开关映射 -Pformprobe/-Pformdrive/-Pexecpool/-Pfjp1（build.gradle 新增映射行）。
- **自证行硬门禁（#118/#141）**：开启后必见 `[FP-ON]` 行（含 commonPool parallelism、exec 池宽、drive 状态）；驱动臂必见 `[FP-DRV] ev=move seq=0..5` 六行；light 臂必见 `[LightRust] lightInit ok` + `[LightRust] fallback=0`（或 fallback 计数行）；vanilla 臂必见 `[CppBridge] init ... enabled=false`。任一缺失 → 该臂 VOID，禁止人工挑臂。
- **执行体三元组（#36）**：每臂记录 loaded dll sha（`synced Rust dll` 行 + target 产物 sha 对照）。
- **seed/文件三查**：server.properties `level-seed` 备份后设置；run\world 每臂删除；日志内 seed 行与目标一致。
- **日志归档（#144）**：每臂 stdout 全量落 `.investigations/form-audit-260915-03/cmd-output/<臂标签>.log`；复跑必须先归档或换标签。
- 虚拟玩家驱动（替代真实客户端，一手源核对：ChunkTicketType.create 公有 / ChunkTicketManager.addTicketWithLevel:173 / removeTicketWithLevel:177 / ServerChunkManager.ticketManager 私有字段经 Accessor）：自建 `formprobe` 票据类型，虚拟玩家 chunk = (200+24·seq, 200)，seq 0..5，首服 tick 后 25s 起每 25s 跳一格，ticket level 22（= NEARBY_PLAYER_TICKET_LEVEL 同源），梯度由 ticket level 传播天然形成（与玩家 watch 同构：近=level 低=优先）。

## 1. 探针① —— fill 完成序 × 距虚拟玩家距离（裁 R3 (a) 队深延迟 vs (b) 动态重排缺失 → CP-4 归因）

- 臂：**A1** = CS 接管默认（exec 开、light 关）+ formprobe+formdrive；**A2** = vanilla 对照（`-PcppVanilla=1`）同驱动。同 seed、各自全新 world。
- 数据：`[FP-FILL] ev=sub|end` 行（t、cx、cz）+ `[FP-DRV] ev=move` 行。分析窗口 = 驱动窗口 seq0..5（各 25s）；boot spawn 预生成段不进判据（预登记排除）。
- **判据 (a)（首载瞬间优先级倒置）**：每窗口内按提交时刻与虚拟玩家 chunk 的 Chebyshev 距离分桶，near = d≤4、far = d≥8；W_near/W_far = 各桶 sub→end 中位等待。信号 S = W_near / W_far。
  - **S_A1 ≥ 2.0 且 S_A1 ≥ 1.5 × S_A2 → (a) 成立**（近玩家 chunk 被 FIFO 压队，vanilla 无此现象）。
  - **S_A1 ≤ 1.5 → (a) 否定**。
  - 中间带 → (a) 存疑，与 (b) 结果合并交 judge；不允许事后挑选桶宽（d≤4/d≥8 唯一，禁换口径）。
- **判据 (b)（移动中过时任务占用）**：每次 hop 后 10s 内，「过时 chunk」（距**当前**虚拟玩家 >14 chunk 的在飞/排队 fill）的 fill 时间占比 O。
  - **O_A1 ≥ 1.5 × O_A2 且 O_A1 ≥ 20% → (b) 成立**。
  - **O_A1 ≤ 1.2 × O_A2 → (b) 否定**。
- **CP-4 裁决映射**：(a) 成立 → 首载队深延迟实锤 → CP-4 升首候选；(b) 成立 → 重排/可取消缺失主导 → CP-4 修复形态改为「可重排优先」；两者均否定 → R3 嫌疑降级，CP-4 降后（G3 归因交给探针②）。

## 2. 探针③ —— light 车道粘线 + 池利用率 + 回退率（裁 .b1/.b2/.b3 → CP-2 归因）

- 臂：**B1** = `-PlightRust=1` + formprobe+formdrive，其余默认（并行、Mutex dll）。
- 数据：`[FP-LIGHT] ev=call`（th、durMs、gapMs）+ ev=fall + `[FP-FILL]` + `[FP-ON]` 池宽。
- **判据 .b2（粘线）**：同线程 gap<100ms 连续 light 调用段长 ≥5 的调用占比 R_sticky。
  - **R_sticky ≥ 30% → 粘线证实 → CP-2 升**；R_sticky ≤ 10% → 粘线否定。
- **判据 .b1（池饱和放大面）**：U = fillBusyMs / (池宽 × 驱动段 wall)（FP-SUM + [FP-ON]）。
  - U ≤ 40% → 池未饱和 → **.b1 单独不成立**（与 W2 预判一致）；U ≥ 70% → .b1 放大面生效。
- **判据 .b3（时机差）**：回退率 F = fallN / (fallN + okN)。
  - F ≥ 10% → .b3（邻域成熟门推迟）有实际贡献面；F ≤ 2% → .b3 受限。
- **CP-2 裁决映射**：.b2 证实 = CP-2 主证据；.b1 U≥70% 或 .b3 F≥10% 作为复合权重并入排序建议；三者全否 → e2e 7% 回退归因存疑，CP-2 降后（交 judge 复议）。

## 3. 探针④ —— T_fill 实测（回填 260915-01 judge C-1）

- 数据源：A1 臂 dispatch wrapper `durMs`（busy 口径，不含排队）。
- **预登记口径（§9.7）**：载体 = formprobe dispatch wrapper（生产路径 exec 池、CallerRuns 事件单列）；覆盖面 = 6 驱动窗口 + 本机 24 逻辑核、单 seed；**与本表任何历史数字（~10ms / ~50ms / 2.94ms）不比绝对值，只报实测分布**。
- 产出：P50/P90/max + 「CallerRuns 内联次数」单列行。**C-1 裁决规则**：P50 落在 10ms 量级 → 采信 10ms 系；50ms 量级 → 采信 50ms 系；分布双峰（P90/P50 ≥ 4）→ 两系并存，报分位数不许并单值。W4 R2/R3 推演中引用 50ms 的上游数字在本探针后**禁再引用**，一律以本实测为准。

## 4. 探针② —— G3 首载漂移单线程/并行复跑（裁 (i) 时机形态 vs (ii) UB → CP-1/CP-3 权重）

- 协议：复刻 `.tmp/g3-260905-03/`（snap_light.py / cmp_roundtrip.py / rcon.py，载体 = 存档 section light 签名哈希；§9.7：45×45 spawn 区 2025 chunks，与内存/客户端口径不可比）。seed = 8576294172403134396（server.properties 三查）。每臂：删 run\world → run1（boot spawn 预生成 → RCON stop 优雅存档）→ 重启 run2 → 停 20s → stop → snap before/after → cmp。
- 臂：**G1** = gate ON（`-PlightRust=1`），当前 **Mutex dll**（CP-3 已落地），默认并行。
- 既有基线（260905-03，旧 RefCell dll）：rust 首载漂移 123/2025（6.07%）；vanilla 17/2025（0.84%）。**跨 dll 数字只作带参照，不作逐位预期（#127/#103）**。
- **判据（预登记）**：
  - **drift_G1 ≤ 0.9%（vanilla 带内）→ G3 超额漂移已随 CP-3 消除 → (ii) UB 面主导实锤 → CP-1 的「G3 同源一箭双雕」权重解除**。
  - **drift_G1 ≥ 2.5%（≥3× vanilla 带）→ Mutex 在位仍漂移 → (ii) 排除 → (i) 时机形态主导 → CP-1 权重升级**（追加 G2 臂 `-Pfjp1=1` 单车道旁证：漂移消失→并发时序相关；仍在→纯时机形态——旁证不改变主判据）。
  - 0.9%–2.5% 灰区 → 跑 G2 + 原始 123 chunk 簇空间对照（anatomy 复用），交 judge。
- 自证：run2 日志 `[LightRust] lightInit ok`（重算路径为 rust 接管）+ fallback=0；缺失 → 臂 VOID。

## 5. 执行序与产物约定

- 采集序：构建（--rerun-tasks，dll sha 记录）→ A1 → A2 → B1 →（探针②）G1 →（条件）G2 →（条件）A3 syncfill T_fill 串行对照（④ 灰区时）。
- 原始输出：`.investigations/form-audit-260915-03/cmd-output/`；解析脚本：`.tmp/formprobe-260915-03/parse_fp.py`（产出 JSON 汇总，不写结论）。
- 解读：worker subagent（对照本文预登记判据逐条判定，禁主会话自推）；若解读揭示 ≥2 互斥机制候选新分叉 → fan-out .bN。
