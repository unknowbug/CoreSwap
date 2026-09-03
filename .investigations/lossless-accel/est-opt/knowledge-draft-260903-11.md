# knowledge-draft-260903-11 —— est 查表化优化包（260903-11，commit 0949402）知识库更新草稿

> core-worker 产出（只读调研 + 本草稿），主会话稍后应用。**未修改任何目标文件。**
> 价值门判定：§一 高价值（两例调研误读，必记，五段式）；§二 中价值（#21 补充案例，追加简记）；§三/§四 时间线与 INDEX（归口必做）。
> 剔除项（过价值门筛除，不写知识库）：L2 命中率 84.9%/inserts=1914 等一次性实测数值、256-chunk e2e 各数字、est_l2/est_shared 的实现细节清单（属结果快照 candidate，留 .artifacts/est-opt-result-260903-11.md）、b2 fan-out 塌缩细节（一次性选型过程）——均不进 discovered/。

---

## 一、目标载体：`knowledge/discovered/workflow-patterns.md` —— 新增「发现 #25」（追加到文件末尾，不覆盖）

**载体判断**：两例同属「静态调研结论失真」这一工作流反模式（一例引用行号错位、一例常量取值源头未追），复用判据跨课题通用，归 workflow-patterns 最合适；不新建独立条目文件（项目已有 #19-#24 同族全在此文件）。若主会话希望错误台账同步，可另在 `.investigations/lossless-accel/lossless-accel-errors.md` 加两条目（本文附五段式全文，可直接复用）。

以下为可直接粘贴到 workflow-patterns.md 末尾的文本：

```markdown
## 发现 #25: 静态调研结论失真两例——差距点必须核「生产路径可达性」，常量必须追「取值源头」（260903-11）

- **时间/置信度/module**：260903-11，candidate（judge 两次审查通过），通用方法论。
- **来源定位**：est-opt 包 P1 调研阶段 subagent 结论（G5）与 K3 复核；裁决记录 `.investigations/lossless-accel/est-opt/k3-k2-verdict-260903-11.md`；两次 judge `.investigations/lossless-accel/review-estopt-260903-11.md`。
- **案例①（G5 引用错位）**
  - **现象**：P1 调研 subagent 结论「fill/carver 各自 `Aquifer::new` 不共享」，主会话据此把「跨实例共享」列为优化差距点，差点直接投入实现。
  - **根因（机制）**：调研引用 `worldgen_handle.rs:547` 处 `Aquifer::new` 作证据，但 :547 实为诊断 API `diag_pre_surface_column`（非生产路径）；生产路径 :446 唯一 `Aquifer::new`，:520 carver 复用 `&mut va.aq`——引用行号存在但语义错位，结论在真实调用图上不成立。设计阶段 worker 代码核对推翻。
  - **定位**：设计 worker 对差距点逐条做「生产路径可达性」代码核对（不是重新读一遍结论，而是顺着调用图验证差距点真实可达）。
  - **修复**：原 P1 表述不可改写（§15.4），以 k3-k2-verdict 的 G5 supersedes 记录取代；实现方案按「唯一构造点共享」重定（b1-a est_at 共享）。
  - **教训**：**调研结论的每个「差距点」必须带生产路径可达性核对**——引用行号/调用点存在 ≠ 该点在生产调用图上可达；诊断 API、探针专用路径、死代码都可能是错位来源。下开销实现前先核对，比实现后返工便宜一个量级。
- **案例②（K3 常量记忆错）**
  - **现象**：调研称「Java est 扫描步长 4」（`l -= verticalCellBlockCount`），与 Rust `l -= 8`（aquifer.rs:295）疑似不一致，触发 K3 疑点裁决。
  - **根因（机制）**：凭常见值记忆填写常量——`verticalCellBlockCount = 4 × size_vertical`（GenerationShapeConfig.java:46-48），overworld.json `size_vertical: 2` → **实际步长 8**；「4」是 size_vertical=1 的常见值（如部分 mod 维度），不是 overworld 值。
  - **定位**：裁决沿取值链逐环核对：调用点（ChunkNoiseSampler.java:233）→ 计算式（GenerationShapeConfig.java:46-48）→ 数据源头（overworld.json:18）。三环核对后疑点解除（一致，P1 文档「4 步进」为笔误）。
  - **修复**：P1 文档笔误不改原文，由 K3 裁决记录取代标注。
  - **教训**：**引用 Java 常量必须追到取值源头（config/JSON 派生链），禁止凭记忆/常见值填写**；数据驱动的 MC 常量尤其如此——派生链上任何一环（计算式 × JSON 值）换版本即变，「常见值」是最不可靠的引用方式。
- **同族**：#9（跨 session 未验证标注当公理继承）、#17（打印坐标≠采样坐标）——本条补「调研产出消费前」维度：#9 管跨 session 交接，本条管同 session 内调研 subagent → 主会话的交接，判据同为「引用先验证再消费」。
```

---

## 二、目标载体：`knowledge/discovered/workflow-patterns.md` —— 发现 #21 追加补充案例（追加不覆盖）

在 #21 条目末尾（「教训/如何利用」段之后）追加：

```markdown
- **补充案例（260903-11，working set 维度）**：est-opt 包微测复刻了调用形态（est 逐列冷扫描）仍得出 2117ns/iter 上界，生产冷路径实际单价 ≈11µs/iter（差 ~5×）——签名：**e2e 实测收益（−48ms/chunk）超微测上界（15.5ms）**。差异不在调用形态而在 **working set**：生产树共享 Arc + 大缓存集（Cache2D/邻居表/共享 sampler），独立树微测的缓存足迹远小于生产；微测虽「形态同构」但「内存环境不同构」。教训补全：微测外推需复刻**调用形态 + working set（共享结构/缓存集/生命周期）**两者；「e2e 收益 > 微测上界」是 working set 失配的强签名，以生产实测为准、不反推机制定论（本包即止于观察，未逐项归因剩余差）。
```

---

## 三、目标载体：`versions/1.20.1/docs/10-timewise-archive.md` —— 追加 260903-11 节（文件末尾追加）

```markdown

## 260903-11（est 查表化优化包：est_at 共享 + 跨 chunk est L2，candidate @ 0949402）

> 承接 260903-10 Q-AQ1「修复方向（另立优化包）」；过程 `.investigations/lossless-accel/est-opt/`；结果快照 `.artifacts/lossless-accel/est-opt-result-260903-11.md`（candidate）。

- ✅ **P0 交接验证**：复跑 qaq1_surf_probe（新鲜进程），iterations 7342 / avg 34.35 / miss 2782 逐项一致，median 72.84 在方差内 → Q-AQ1 est 冷扫描量级可继承（§15.3 廉价独立验证）。
- ❌→✅ **P1 调研误读两例被核对推翻（高价值，详见 workflow-patterns #25）**：① G5「fill/carver 各自 Aquifer::new 不共享」系 subagent 引用 :547（诊断 API）错位，生产路径 :446 唯一构造——主会话差点按假差距点投入实现，被设计 worker 代码核对推翻（G5 supersedes 入 k3-k2-verdict）；② K3「Java 步长 4」系常量记忆错，实际 4×size_vertical=8，与 Rust 一致（疑点解除）。P1 文档原文不改，裁决记录 supersedes。
- ✅ **P2 fan-out：b2 主形态判死 → 分叉塌缩单线 b1**：b2 粗表逐位一致硬约束下不成立（唯一逐位安全形态 ⊂ b1-b）；K2 blend 旁路等价成立（blend 类 DF 全为 no-blending 常数，density.rs:626-628；原「blending_active 字段」引用系 b1 拟新增字段，judge R3 补正）；新增 D3 扫描域差异（est_at noise_height vs Aquifer height，仅 nether 有差，overworld 同 384）。
- ✅ **实现（commit 0949402，门控默认关）**：b1-b EstL2 精确值缓存（量化列 key / FIFO 131072 上限 / 代际挂 handle / blend 闸门；`WG_EST_L2` 门控）；b1-a est_at 共享（`WG_EST_SHARED`，对齐 Java ChunkNoiseSampler.java:222-226）；探针 bin-diag/estopt_ab.rs（四臂 hash A/B + L2 统计）。
- ✅ **四臂验证**（§9.7：载体=fill_chunk_blocks 全管线；覆盖面=64 chunks A/B + 256 chunks e2e region(200,200) seed …396；历史口径同 pc_e2e 260903-08 可比）：off 臂 == HEAD 基线（64-chunk 聚合 hash 相等 + stash 重建基线）；l2 臂 hash 逐位一致；16 chunks est 迭代 7342→1715（−76.6%）；**256 chunks e2e median 75.94→27.69ms（−63.5%）**。shared 臂 hash 变化（D1 角列量化修正 + D3 扫描域）——默认关，翻默认前 MUST Java 逐位验证。
- ✅ **judge 两次审查 PASS**：P2 选型（有条件）+ P5 交付（4 CONCERN 无 BLOCK，建议 candidate）：C1 L2 stats 口径修正（e2e 行未落盘，外推表述作废）/ C2 零回归证据载体标注（聚合 hash，非 block_probe 全量 diff）/ C3 未执行清单显式声明 / 代码抽查 8 项全过。
- 📝 **观察（不反推机制定论）**：e2e 收益（−48ms/chunk）超 est 微测上界（15.5ms）——生产冷路径 est 实际单价 ≈11µs/iter vs 微测 2117ns（working set 失配，workflow-patterns #21 补充案例）。
- 🔍 **未闭合**：shared 臂疑似修正既有 surface 错位 bug（需 Java 逐位裁决，独立小包）；b1-b 翻默认前置（mt_fill Mutex 基线 + 大 region 淘汰 + e2e l2 stats 落盘）；nether est_at 扫描域（D3）未收敛（生产仅 overworld，显式声明）。

### 📌 记录指引
- 通用模式 → workflow-patterns #25（静态调研结论失真两例）+ #21 补充案例（working set 维度）。
- 产物：.artifacts/lossless-accel/est-opt-result-260903-11.md（candidate + index.yaml 登记）；裁决/验证 k3-k2-verdict / p0-handover-verify / cmd-output/estopt-{ab-arms,perf}-260903-11.txt。
- 状态：candidate（judge 建议授予），confirmed 留用户；翻默认（WG_EST_L2 / WG_EST_SHARED）均不在授予范围。

```

---

## 四、目标载体：`knowledge/INDEX.md` —— 两行更新建议

1. **工作流模式行**（表内「工作流模式」单元格）末尾追加：
   ` + 静态调研结论失真两例——差距点必须核生产路径可达性、Java 常量必须追取值源头（发现 #25，260903-11）`
2. 同一单元格 #21 摘要后补（或并入 #21 摘要句）：
   `（补充案例 260903-11：working set 失配——e2e 收益超微测上界签名）`

---

## 五、自检清单（SUBAGENT-KNOWLEDGE-GUIDE §四）

- [x] 价值门先行：两例误读=高价值必记；#21 补充=中价值简记；一次性数值/实现细节已剔除不写
- [x] 五段式完整（现象/根因/定位/修复/教训），根因为机制层
- [x] 判错经验沉淀（可达性核对 / 取值源头追链 / working set 失配签名）
- [x] 被推翻表述（G5/K3 笔误/外推 stats）保持原文不改 + supersedes 标注指引
- [x] 载体正确（discovered=通用判据；时间线=过程归口；INDEX 同步）
- [x] 数字全部来自四份背景材料，无编造、无占位符
- [x] 格式与 workflow-patterns.md 现有条目（#19-#24）及时间线 260903-10 节对齐
