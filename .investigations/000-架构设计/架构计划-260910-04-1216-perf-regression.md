---
编号: 000
任务: 1.21.6 性能回归定责（vanilla 52s vs coreswap 268s / 4225 chunks = 5.15×）——三分解 + 并发结构判别
任务类型: 性能定责（perf attribution）+ swe 优化候选
模式档位: 重量
状态: 待批准
日期锚: 260910-04（实际 2026-09-10 17:0x，Get-Date 确认）
模块路由: swe（本项目自有 Rust/Java 代码的性能与并发，非逆向）
上游: NEXT_SESSION.md「下轮开工点 1」（用户拍板立项）+ vivo-freeze-260910-03/record.md 遗留节
---

## 0. 前置工作（本 session 已完成：交接结论廉价独立验证，§16.3 / AGENTS.md 交接验证纪律）

> 以下 4 条是**开工前**已核实的**已验证事实**（区别于未验证假设），直接作为本计划输入。

| # | 核验对象 | 方法 | 结果 |
|---|---|---|---|
| V1 | 执行体三元组 | 读 `.tmp/hang-repro-260910/arms/server-coreswap.log` + target 产物 | ✅ coreswap 臂加载 sha256 `5e30187a…`（= 260910-02 vivo confirmed 同一 dll）；`target/release/worldgen1216.dll` mtime 14:22 < 臂运行 15:55 → 三元组自洽 |
| V2 | 5.15× 基线口径 | 读两臂日志 + 驱动脚本 `diffarms1216_260910.ps1` | ✅ 两臂同 seed `417950215108767439`、同 region center(-48,-11) chunkradius=56（4225 chunks）、同 Chunky 驱动、串行执行（#28 合规）；**但发现结构性口径污染（见 D1）** |
| V3 | WG_CA_MIN 默认值 | `worldgen_handle.rs:619/1031` 直读 | ✅ `env::var("WG_CA_MIN").map(\|v\| v != "0").unwrap_or(true)` = **默认开**，NEXT_SESSION 继承结论成立（#53 抵赖线过） |
| V4 | 线程/缓存结构 | `api.rs:24-41,146-169` + `jni_bridge.rs:194-224` + `worldgen_handle.rs:141` | ✅ 机制在场（未量化）：每 chunk count=1 → `adaptive_threads` 返回 10 → `scope` **spawn 10 线程（9 个空转即退）**；JNI 每 chunk 393KB 零初始化 + 393KB 回写；CA_MIN `terrain_cache` = **全局 `Mutex<HashMap>`** |

### 0.1 本 session 新发现（改变基线可信度，MUST 进计划）

**D1（口径污染·高价值）：5.15× 基线的处理臂携带对照臂没有的 per-chunk 同步日志。**
- 证据：`server-coreswap.log` 9682 行，其中 **9556 行**是 `[Mixin] populateNoise intercepted chunk(x,z)` / `[Mixin] buildSurface skipped chunk(x,z)`（**每 chunk 2 条，共 8450 条**，来自 `NoiseChunkGeneratorMixin.java:91,102,110,139` 的**无条件 `System.out.println`**，1.20.1 侧同样存在）；vanilla 臂日志 137 行、零此类行。
- 机制：`System.out` 被 MC 重定向进 log4j（同步 appender），23 个 Worker-Main 线程并发写 → 串行化点。
- 判据（与既有铁律同族）：**「测量/探针污染铁律」+ #83 口径分场景**——处理臂与对照臂的结构开销不对等，则 5.15× 不是干净读数。
- 附带意义：这两行 println 是**生产 jar 里的真实成本**（vivo 用户同样付），不只为测量污染 → 本身就是一个零风险修复项。

**D2（并发结构候选·未验证）：CA_MIN 读路径在 24 线程下疑 mutex convoy。**
- 串行 bench（260909-04/06，`.investigations/camin-perf/`）：ca_min **on 177.9ms/chunk vs off 118.9–128.5ms**（+35~50%），E4b 记录 on 臂 **853k 读/chunk**（off 7.5k，114×）。
- 结构：`terrain_cache: Mutex<HashMap<(i32,i32),CaTerrainEntry>>`（全局单锁）；`est_l2: OnceLock<Option<Arc<Mutex<EstL2>>>>`（260903-13 翻默认开）同为全局锁。
- 推论（**假设，非结论**）：串行 bench 量到的 +35~50% 是**无争用值**；in-game 24 线程并发时同一条读路径的锁代价可能量级放大 → 可解释「并发吞吐塌到 ~1/5」，而这一点在串行 bench 里**结构性不可见**（#83「同 session 跨口径」+ #100/#102 缓存读路径家族）。
- 反证义务：若 C 组矩阵显示 WG_CA_MIN=0 无显著改善，本假设即被否。

### 0.2 与历史数字的可比性（§9.7，禁止直比）
- 「1.20.1 时代 Rust 快 ~1.2×」= 2026-08-29 P5 旧口径（载体/覆盖面/机器态均不同，#18/#83）→ 本计划**不把它当分母**，1.20.1 侧一律**本轮同载体重测**（B2 臂）。
- 「vanilla 52s」= 260910-03 同会话 Chunky 口径（4225 chunks）→ 可作 1.21.6 分母基线，但须**同批复跑复核**（B1 臂）以绑定当批机器态。

---

## 1. 全局视图

**目标（本块交付）**
1. 把 `coreswap 268s` 逐机制分解到**量化占比**（带 §9.7 口径三要素声明），产出**定责结论（candidate）**；
2. 给出**优化候选清单 + 优先级**（依数据排序，不预判机制）；
3. 若定责明确且用户批准 → 落地**首轮修复**并复测 in-game 端到端（vs 同批 vanilla）。

**范围**
- 纯测量/定责 + 低风险结构修复；载体 = native bench（bin-diag）+ Chunky 双臂 in-game + vanilla 两版本分母。
- 目标判据（验收面）：① 268s 中 ≥80% 的耗时归因到带独立实测证据的机制项；② 首轮修复后 in-game 端到端较修复前有可复现改善，且 `hash` 行为等价门 PASS。

**明确不做（本块排除）**
- 不动 features/carver/light 接管语义（260909-03 D1-D3 维持现状）；
- 不做大架构重写（如 DFC 编译直排、CA_MIN 语义改为 off）——只在定责后提候选，不擅自改语义；
- 不复活旧 C++ 侧（versions/1.20.1/cpp 仅历史参照）；
- 不改 JAVA 侧 Chunk 并行模型（Worker-Main 池是 MC 的）。

## 2. 角色分配

| 角色 | 承担者 | 任务 |
|---|---|---|
| 主会话 | 本会话 | 构建设计/口径定义/命令执行（工具、gradle、bench、Chunky）、hash 门、修复落地（编程闭环）、todo/签核 |
| scout | `subagent`（只读勘探，subagent 无 shell → 只做静态路径/依赖/成本图） | Phase 1：in-game 通路逐阶段成本图 + 每次调用的结构开销清单，产物只写 `.investigations/` |
| worker（fan-out） | `subagent` × N（各验一个互斥机制候选） | Phase 2 后：对残留互斥候选并行产出 `.bN` 候选（各带独立证据） |
| judge | `subagent`（core-judge，三源核对） | candidate 授予 SHOULD + 收尾 MUST（定责结论 + 修复交付） |
| knowledge | `subagent`（草稿）+ 主会话应用 | 结论性 discovered/docs/error 台账草稿 |

## 3. 任务拆解 & 依赖图

```
Phase 0.5 口径净化与载体准备（主会话）
  T0.1 mixin 日志门控化（默认关，-Dcoreswap.mixlog=1 开）——[行为等价，hash 门]
  T0.2 build.gradle 补 env 映射：-PcaMin / -PestL2 → runServer environment()（防 #32/#53 死参数假判别）
  T0.3 驱动脚本固化到唯一临时区 .tmp/perf-reg-260910-04/（含 ABBA 顺序 + 双臂 + 臂参数矩阵）
      ↓
Phase 1 scout 成本分布勘探（subagent 只读，主会话补实测数据）
  T1.1 .investigations/perf-regression-260910-04/pipeline-cost-map.md
      ↓
Phase 2 判别矩阵（主会话执行，数据层）
  A 组 native（串行、无 JNI、reuse camin_bench 载体）
    A1  1.21.6 data：CA_MIN on/off        → 引擎基线 + CA_MIN 无争用成本
    A2  1.20.1 data：CA_MIN on/off        → 引擎/数据跨版本自回归（回答「引擎是否变慢」）
    A3  1.21.6 data：WG_EST_L2 on/off     → est_l2 无争用成本
  B 组 vanilla 分母（Chunky，same seed/region）
    B1  1.21.6 vanilla 复跑（复核 52s，绑当批机器态）
    B2  1.20.1 vanilla 同口径          → 回答「分母是否被 Mojang 优化变快」
  C 组 in-game 方差分解（Chunky，seed 417950215108767439，center(-48,-11)，chunkradius 56）
    C0  现产物复现基线（≈268s，要求复现率入噪声带）
    C1  C0 + 日志门控关闭（净日志）                → D1 量化
    C2  C1 + -PcoreswapThreads=1（消 9/10 空转 spawn）→ V4 线程项量化
    C3  C1 + WG_CA_MIN=0                          → D2 假设主判别
    C4  C1 + WG_EST_L2=0                          → est_l2 锁量化
    顺序：每臂 ≥2 run，ABBA 配对交错（#24/#28/#51），报中位 + 当批噪声带
      ↓
Phase 2.5 验证
  T2.5.1 行为等价门：改造前后同 dll/同 seed region 内容 diff（hash 哨兵，阻断「优化改行为」）
  T2.5.2 工程门：workspace 全量 build 绿 + cargo test 绿
  T2.5.3 §9.7 口径声明 + evidence saturation 计数（连续 3 轮无新数据层证据 → 回数据层/升级）
      ↓
Phase 3 judge（subagent，三源核对：.artifacts 快照 + git diff/worktree + 验证记录）
      ↓
用户 confirmed → 归档
      ↓
Phase 4 首轮修复（用户批准后；编程 = 主会话闭环 + judge 终审）
  R1 mixin 日志门控（已随 T0.1 落地，纳入交付）
  R2 count<nthreads 时不 spawn 空转线程（api.rs，低风险）
  R3 依 C 组数据选：CA_MIN/est_l2 缓存并发结构（分片/thread_local 快照/无锁读）——需独立架构增补
  交付门：修复后 in-game 端到端 + hash 等价 + 全量绿 + 用户 vivo 复核（#105 生产载体不可替代）
```

## 4. 并行执行计划
- 第一波（可并行）：T0.1/T0.2（主会话代码）+ T1.1（scout subagent 静态勘探）。
- 第二波（串行铁律 #28）：A 组 → B 组 → C 组；**C 组内臂严格串行 + ABBA**（性能 bench 禁并发，机器噪声带 ±10%）。
- 第三波：残留互斥候选 → fan-out `.bN` 并行（若 C 组已把某一机制定量到主导，则收窄为单分支，不无谓 fan-out）。

## 5. 人工决策 HOOK 点
| 节点 | 触发 | 决策 |
|---|---|---|
| H1 架构批准 | 现在 | 本文件批准与否（**未经批准不动手**） |
| H2 口径净化后 | C0 复现结果出来 | 基线是否可信（干净读数 vs 旧 268s 的差量是否已被 D1 解释） |
| H3 C 组矩阵后 | 分解表成型 | 方向选择：单机制主导（直接修） vs 多候选残留（fan-out） vs 证据不足（回数据层） |
| H4 修复实施前 | 定责 candidate + 候选修复清单 | 是否实施 R2/R3（涉及结构与语义风险分级） |
| H5 confirmed 授予 | judge 通过后 | 用户拍板（AI 只建议 candidate） |

## 6. 风险 & 回退
| 风险 | 判据/回退 |
|---|---|
| gradle daemon 吞 env → WG_CA_MIN/EST_L2 死参数假判别（#32/#20/#53） | T0.2 显式 environment() 映射 + **行为化证据**（Rust 启动打印 ca_min/est_l2 实际值一次）+ A/B hash 双向变化（#81） |
| 机器噪声带 ±10% 淹没信号（260909-06 实测） | ABBA 配对交错 + ≥2 run 中位 + 报噪声带；差值 < 噪声带时不得作排除结论（#51） |
| 1.20.1 侧缺 Chunky/驱动 | 复用 260906-09 confirmed 载体（Chunky jar + 同驱动脚本，#26）；缺则降级声明并只做 B1 |
| .tmp 不入库 → 脚本/原始数据丢失 | 驱动脚本 + 关键日志在 Phase 2 结束前复制进 `.investigations/perf-regression-260910-04/cmd-output/` |
| dev 口径 ≠ 生产 vivo 口径（#105 载体偏差 / #36 执行体三元组） | 结论声明口径；最终体感验收以用户 vivo 实测为准（不宣称生产等效） |
| 「诊断代码放热路径」（铁律） | T0.1 门控默认关 + 每 chunk 一次而非每点（#11 鸡生蛋死锁：env 读取与数据初始化分离） |

## 7. judge 步骤预置
- J1 节点：**定责 candidate 授予前** | 级别 SHOULD→本次提 MUST（涉及「跨版本回归」重大定论） | 审查对象：`.artifacts/` 快照 + git worktree diff + A/B/C 组原始验证记录（含噪声带声明）
- J2 节点：**收尾交付**（定责报告 + 首轮修复，若有） | 级别 MUST | 三源核对（快照 + git HEAD/工作区 diff + 验证记录）；含「结论取代链」检查（若推翻既有 #83/V2 类结论，须 §15.4 supersedes 双指针）

## 8. fan-out 步骤预置
- 预置分叉点（C 组矩阵后，**若残留 ≥2 互斥候选**）：
  - b1：CA_MIN `terrain_cache` 全局 Mutex 并发争用为主因（证据面：853k 读/chunk × 24 线程 + C3 臂改善量）
  - b2：`est_l2` 全局 Mutex 争用（C4 臂改善量 + 读计数）
  - b3：每 chunk 10 线程 spawn/join 的 OS 级开销与调度超额订阅（C2 臂改善量 + 线程计数）
  - b4：mixins per-chunk log4j 同步写（C1 臂改善量）
  - b5：引擎固有成本（A1/A2 对照 + 单 chunk 理论下界核算）
- 规则：**禁止主会话逐个自推候选**；各 bN 由独立 worker subagent 依分配到的数据层证据产出 `.artifacts/perf-regression-260910-04/candidates/bN-*.md` + `.bN` 标记；主会话只做收敛与量化对账；若 C 组已单点定量到 ≥80%，则**不启动** fan-out（避免形式化 fan-out），改用单分支收敛 + judge。

## 9. 知识库更新（每 Phase 末标注产出者）
- Phase 2.5/3 后：`subagent` 产出草稿 → 主会话应用 + 验证
  - `knowledge/discovered/workflow-patterns.md`：候选条目（D1 类「处理臂结构开销不对等污染对照臂」；D2 类「串行 bench 量不到的并发争用成本」；口径三要素实例）
  - `knowledge/discovered/build-tooling.md` 或 `algorithm-fingerprints.md`：依实际发现定（如 gradle env 映射第四犯、thread-per-call spawn 结构指纹）
  - 错误台账：`.investigations/perf-regression-260910-04/perf-regression-errors.md`（五段式 + 错误→根因速查表；prompt 必含 `先读 E:\PYTHON\CoreSwap\knowledge\SUBAGENT-KNOWLEDGE-GUIDE.md`）
  - 若结论推翻了既有条目 → **§15.4 取代注记**（双指针，原文不改）

## 10. 子角色介入点（全部预置，执行只核对不补排）
- **scout**：节点 T1.1 | 触发「机制未明」大排查初期（**是**：为什么慢 5×，机制未明） | 产物 `.investigations/perf-regression-260910-04/pipeline-cost-map.md` | 只读、subagent 隔离
- **worker**：节点 Phase 4 修复实现（若需上下文隔离） | 触发「代码交付」 | 产物 patch/新文件 + 静态自检清单，主会话编译验证
- **fan-out**：节点 Phase 2 后分叉 | 候选 b1–b5（见 §8） | MUST 并行 `.bN`（仅当残留 ≥2 互斥候选）
- **judge**：节点 J1（candidate 前，提 MUST）/ J2（收尾 MUST）
- **knowledge**：节点 Phase 2.5/3 末 | 结论性落盘 | **subagent 产出草稿 + 主会话应用验证**（禁止主会话直写结论性 docs）

---

## 附：可复现命令骨架（执行时按实际校正）

```powershell
# A 组 native（bin-diag 载体，#30 纪律：链 deps 最新 hash rlib）
rustc -O worldgen-core/src/bin-diag/camin_bench.rs --extern WorldgenRust=target/release/deps/libWorldgenRust-<hash>.rlib -L target/release/deps -o .tmp/perf-reg-260910-04/camin_bench.exe
.tmp/perf-reg-260910-04/camin_bench.exe -8248318472910187742 E:\PYTHON\CoreSwap\versions\1.21.6\data\worldgen 16 0 0
.tmp/perf-reg-260910-04/camin_bench.exe -8248318472910187742 E:\PYTHON\CoreSwap\versions\1.20.1\data\worldgen 16 0 0

# C 组 in-game（每臂：删 world → 起服 → RCON chunky → 记 Total time）
#   C0: -PcppReplace -PcppLib=<dll> -PcppWorldgenDir=versions\1.21.6\data\worldgen
#   C2: 追加 -PcoreswapThreads=1
#   C3: 追加 -PcaMin=0     （T0.2 新增映射 → environment WG_CA_MIN=0）
#   C4: 追加 -PestL2=0     （T0.2 新增映射 → environment WG_EST_L2=0）
# B 组 vanilla 分母：-PcppVanilla=true；1.20.1 复用 260906-09 Chunky 载体
```

> 违反约束提示：**未经用户确认本架构，不得开始 Phase 0.5 及之后任何执行动作**（core-plan 约束）。
