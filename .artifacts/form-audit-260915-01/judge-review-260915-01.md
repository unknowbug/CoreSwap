# P4 收尾 judge 审查 — 形态审计 260915-01（candidate pool + 四 worker 行集）

- 工作块：260915-01（P4 judge，收尾交付三源核对）
- 角色：core-judge（subagent，只出审查意见，不改任何 status——confirmed 留人类 HOOK-2）
- 审查对象：`.artifacts/form-audit-260915-01/candidate-pool-260915-01.md`（P3 draft）+ `.investigations/form-audit-260915-01/` p1a/p1b + p2-w1/w2/w3/w4
- 日期锚：宿主时间取自本审查会话（无新文件日期标签需求，除本文件——按工作块标签 260915-01 归属）
- 本 judge 产物验证分层：Degraded（静态源码复核 + 算术独立重算；未跑任何运行时探针）

---

## 0. 三源核对结果

| 源 | 结果 |
|---|---|
| ① 产物快照 | candidate-pool（76 行）+ p1a/p1b scout + w1/w2/w3/w4 四 worker 行集齐备，行号引用密集且可回溯 |
| ② git HEAD + 工作区 diff | `git status` = 仅 `.artifacts/index.yaml`（M，+4 行）+ `.artifacts/form-audit-260915-01/` + `.investigations/form-audit-260915-01/`（untracked）——**零 src 改动，与「本审计零运行时改动」声明一致** ✅ |
| ③ worker 自检/retry 声明 | W1（§4 retry=1 + 假设声明）✅ / W3（retry=1 + 推理级标注）✅ / W4（retry=1 + 闭合建议）✅ / **W2 无显式 retry/自检段** ⚠️（见 C-2） |

**index.yaml 核对**：M 状态的 +4 行为 light-round3 历史条目组（上工作块遗留未提交），**form-audit-260915-01 尚无 index 条目**——候选池仍是 draft 待 HOOK-2，登记时点在立项后属可接受，但收尾归档时 MUST 补（见 C-6）。

---

## 1. 逐项审查

### 1.1 矩阵完整性抽查 —— **通过**

- **接管面全覆盖**：生产 3+1 段（A NOISE 含 A' Beardifier 前置 / B SURFACE / C 光照 / D 写回）+ executor 横切层（W4）各有专属 worker；carver/features/biome/序列化让位段在 P1a §1.3 显式声明不在矩阵、池头覆盖面声明（§9.7 口径）复述 ✅。
- **5 维度每段有裁定**：W1 A 段 6 行 + B 段 5 行、W2 五维度各表、W3 五维度 + 子行、W4 为横切层（池拓扑/取消/优先级/scope 开销/线程归属表）——横切层不逐段套 5 维属设计使然（计划口径），无缺漏。
- **二选一纪律抽查（4 行）**：
  1. W1 B-①（surface 前移时序等价）：论证走 ChunkStatus margin 隔离 + 输入确定性 + 同 chunk 串行，静态可证——**论证合规且质量高** ✅；
  2. W2 4a（首亮 gate 不触碰 checkBlock）：代码事实定位准确（gate 只拦 light() HEAD），等价论证成立，正确性残差正确归入 12 篇遗留而非混入形态 ✅；
  3. W3 D-③（bulk vs 逐块）：等价论证带已闭合验证链（260911-05/260913-03 指纹门 + 计数语义复刻）——**全场证据最硬的等价行，范本定级恰当** ✅；
  4. W4 R4（per-call scope 开销）：20-50µs / 50ms = 0.04-0.1%，算术复核无误 ✅。

### 1.2 上限推演独立复核（防 #22 自由参数凑数）—— **通过（附条件 C-1/C-5）**

- **W2 L5 ×9**：域 48×48×384 = 9 × (16×16×384)，blocks9 = 884,736 int = 48×48×384 逐位复核成立；「同一邻 chunk 中心 1 次 + 邻居参与 8 次 = 出现在 9 个 3×3 域」结构性事实正确；「≥9×、含 BFS 稀疏性后 9×–数 10×」的口径声明（上限 + vanilla 种子化触达 ≪ 全域）诚实，未把上限当实测 ✅。参数（48/384/9）全部来自代码事实，无自由参数。
- **W4 R3 13.8×**：(128+10)/10 = 13.8 算术成立；128 = LBQ 容量、10 = 本机 24 逻辑核池宽（logical/2-2），均为代码事实；50ms/chunk 为声明假设。13.8×50ms = 0.69s 复核无误 ✅。
  - ⚠️ **C-5**：13.8× 依赖本机 N=10——低核机 N 更小则队深比更大（(128+1)/1 = 129× @4C），发行面评估时 MUST 声明池宽依赖，不能把 13.8× 当普适常数。
  - ⚠️ **C-1**：**T_fill 假设跨 worker 不一致**——W1 A-②b2 用 ~10ms（→最坏 1.3s），W4 R3/R2 用 50ms（→0.69s），均未实测且各自声明。比值型结论（13.8×、2.3×）不受影响，但绝对秒数相互矛盾；预验证清单第 4 项（T_fill 实测）执行后 MUST 统一回填两处。

### 1.3 排序依据核对 —— **通过**

- CP-1（L5+L6）与 W2 排序第 1 一致；CP-2（L2+L1）与 W2 第 2 一致；CP-3 池内第 3 但执行序第 1——理由（风险向 + 近零成本）与 W2「L3 优先级按风险而非性能排」明确对应 ✅。
- CP-4 池内第 4 低于 W4 排序第 1（R3 高嫌疑）——非不一致：池显式给出降序理由（静态层无法闭合归因，须 trace 前置），这是「立项排序 ≠ 嫌疑排序」的合理区分，且 W4 自己明示「静态层无法定夺」 ✅。
- 交叉项合并：FIFO 三角度（W4-R3 = W1-A-②b2 = W3-D-② 随行）三方产物原文均有对应行 ✅；L3 与 CP-2 耦合「解粘线即暴露 UB 须同批」与 W4 R5-light「车道串行化是事实上的锁」的观察严格自洽 ✅——这是本池最重要的一条耦合声明，表达正确。

### 1.4 降级与覆盖面声明 —— **通过**

- Degraded 分层：池头（第 7 行）+ 四 worker 各自头部均声明，量级数字均标「推演上限 + 假设注明」 ✅。
- 1.21.6 未覆盖：池头 + P1b §5（一手源缺位 + genSources 禁跑的理由）双层声明 ✅。
- T_fill 未实测：池 §3 A-②b1 行 + §4 预验证第 4 项显式登记 ✅。
- §9.7 口径：W2 §2.1 引用 260914-04 的串行/e2e 不可换算声明 ✅。

### 1.5 遗漏检查 —— **通过（一处可补强，见 C-3）**

- W2 四项辖区外归属：① enqueueSectionData/G3 正确性 → CP-1「一箭双雕」覆盖 G3 同源 ✅；② INITIALIZE_LIGHT 不立行（无 mixin 无差异，免登记合理）；③ collect palette 0.29ms → 池 §5「round4 纯算力项继续冻结」覆盖 ✅；④ heightmap → W3 §3 D-hm 接收 ✅。
- W3 D-hm 条件项归宿：池 §3 登记表「本审计 judge 项」→ 由本文件处置（见 §2 裁定） ✅。
- W3 其余 open：D-④-b → CP-6（含源码核对前置）✅；D-①-b → e2e 候选池登记 ✅；D-② 随行 → 并入 CP-4 ✅。W4 R2 → 登记 + 随 CP-4 评估 ✅。**无孤儿行**。

### 1.6 执行序合理性 —— **通过**

CP-3 先行 → CP-6 源码核对 → 预验证 1/2/3/4 → 按 probe 定 CP-1/2/4 → CP-5 随批：

- **CP-3 先行 vs「车道串行化保留」前提**：无逻辑漏洞——Mutex 在串行访问下无竞争、开销可忽略（防御性修复与现状兼容）；「须同批」约束的方向是 **CP-2 不得脱离 CP-3 单独做**（解粘暴露 UB），而非 CP-3 不得先做。池 CP-3 的「除非确认车道串行化长期保留」正确表达了「若保留则 CP-3 属防御冗余（仍可做）」的让步语义 ✅。
- CP-6 源码核对先于立项：正确性推理级疑点一轮闭合成本极低，顺序合理 ✅。
- 预验证清单裁分叉先于 CP-1/2/4 立项：与各 worker 给出的「可分辨探针」一一对应（探针 1↔R3 分支、2↔G3 (i)/(ii)、3↔.b1/.b2/.b3、4↔T_fill、5↔D-④-b），无探针缺位 ✅。
- 微瑕：CP-1 分辨探针（单线程复跑）同时也是 CP-3 的权重判据，池已在 CP-1 行标注——执行时可一次采集两用，无需补。

### 1.7 角色契约/置信度合法性 —— **通过**

- 全部产物 status: draft，无越权 confirmed/candidate ✅。
- 验证执行者分离：所有量级均标注「静态推演、未实测」，分层以实际执行为准 ✅。
- 模块边界：worker 均以 core-worker/recode-scout 角色运行，无跨模块 skill 正文引用 ✅。
- retry cap：W1/W3/W4 均 retry=1 声明，无饱和触发 ✅；W2 缺声明（C-2）。

### 1.8 噪声卡历史 —— **未核（环境限制）**

本 judge subagent 环境未跑 anchorlaw 噪声卡查询，此项标注「未核」而非通过——主会话归档前可补一次廉价查询（非阻塞项）。

---

## 2. D-hm 条件项独立抽查（本 judge 执行，Degraded 源码复核）

W3 交 judge 的条件：「ProtoChunk 写路径对全部 6 型 heightmap 的增量更新覆盖」未逐行核。独立复核（`.tmp/scout-260905-08/mcsrc` 一手源）：

- `ProtoChunk.setBlockState`（ProtoChunk.java:108-155）：按 `getStatus().getHeightmapTypes()` 逐型 `trackUpdate` 增量更新，缺失型自动 `populateHeightmaps` 补建 ✅；
- `ChunkStatus.CARVERS/FEATURES` 携带 `POST_CARVER_HEIGHTMAPS`（ChunkStatus.java:34-35 = OCEAN_FLOOR/WORLD_SURFACE/MOTION_BLOCKING/MOTION_BLOCKING_NO_LEAVES 四正式型）——**carver/features 的写块增量维护 4 正式型** ✅；
- FEATURES 步前 vanilla 还有全量 `populateHeightmaps` 重算（ChunkStatus.java:150-152）作为兜底，即使增量有残差也被重算覆盖 ✅；
- 2 个 WG 型在 carve 后不被增量维护——**但 vanilla 自身同样如此**（PRE_CARVER_HEIGHTMAPS 只挂 NOISE/SURFACE，:33/:115），两侧形态一致，非 CoreSwap 引入的差异。

**裁定意见**：D-hm 等价条件**成立（源码级证据齐）**，建议主会话把上述证据落进 W3 或池登记行后按等价结案（状态提升留人类）。此为 judge 意见，不代替 W3 补记。

---

## 3. 条件清单（PASS-with-conditions）

| # | 条件 | 级别 | 责任点 |
|---|---|---|---|
| C-1 | T_fill 假设跨 worker 不一致（W1 ~10ms vs W4 50ms，均未实测）：预验证 4 执行后统一回填；此前禁止引用绝对秒数（0.69s/1.3s）做立项量化依据（比值 13.8×/2.3×/3× 不受影响） | should-fix | 预验证阶段 |
| C-2 | W2 缺显式 retry/自检声明段（W1/W3/W4 均有）：主会话应用阶段补一行（retry=1，无运行时采集） | should-fix | 主会话应用时 |
| C-3 | CP-4 的 G3 因果链（FIFO 延迟 → 邻域成熟序 → 快照差）为间接假设——「量级足以解释 7×」只在时序轴成立，G3 本体是落盘方块差；trace 前置必须保持绑定，不得以量级吻合跳过归因直上修复 | binding | CP-4 立项门 |
| C-4 | W2 L3/CP-3 的 UB 面为静态推理（未运行时验证）——CP-3 修复落地后应顺带在提交信息/验证记录声明「UB 可达性未实证，修复为防御性」（防将来误引为已证 bug） | should-fix | CP-3 实施时 |
| C-5 | 13.8× 为本机 N=10 口径：发行面/低核机评估 MUST 声明池宽依赖（4C 下 (128+1)/1 = 129×） | should-fix | CP-4/CP-5 量化时 |
| C-6 | form-audit-260915-01 尚未登记 `.artifacts/index.yaml`：HOOK-2 立项裁决后 MUST 补条目（含本 judge 产物） | binding | 归档前 |
| C-7 | 噪声卡历史未核（judge 环境限制）：主会话归档前补一次查询 | may-fix | 归档前 |

---

## 4. 最终裁决

**PASS-with-conditions（C-1..C-7，其中 C-3/C-6 为 binding）**

- 矩阵完整性、二选一纪律、上限推演算术（×9、13.8× 独立重算均成立，参数来源核清）、排序依据、交叉合并（尤其 L3↔CP-2 耦合）、降级/覆盖面声明、遗漏检查、执行序逻辑——全部通过。
- 无 FAIL 级问题：未发现自由参数凑数（128/10/48/9/384 全为代码事实）、未发现孤儿行、未发现 status 越权。
- 候选池推荐状态：**保持 draft，建议待 HOOK-2 人类拍板后升 candidate**（judge 只建议，不授状态）。

---

## 5. 条件应用与补查记录（主会话应用，260915-01）

- **C-2 ✅**：p2-w2-light-rows.md 已补 retry/自检声明行（retry=1，无新运行时采集，引用 260914-04 round3 既有探针）。
- **C-6 ✅**：index.yaml 已登记 form-audit:candidate-pool-260915-01（candidate）+ form-audit:judge-review-260915-01（本文件，candidate）。
- **C-7 ✅ 补查**：工作区 grep 全仓 `noise_cards|噪声卡` ——无 noise_cards.json 在册（与 10 时间线历史 judge ⑥ 留档一致：噪声卡历史无法核对）；本审计零运行时改动、无运行时失败登记面，C-7 闭合。
- **C-1/C-3/C-4/C-5**：非即时项，已作为约束写入 candidate-pool 头部状态行，绑定后续预验证/实施阶段。
- **状态变更**：HOOK-2 用户拍板「按建议执行序全批」（2026-09-15）→ candidate-pool 已升 candidate。
