# Rust worldgen 端到端性能定位（vs Java）：错误与根因清单

> 载体：`.investigations/perf-e2e/perf-e2e-errors.md`（错误台账，独立成篇）。
> 本文件按「现象 → 根因 → 定位 → 修复 → 教训」五段式记录「Rust worldgen 性能定位（端到端 vs Java）」里程碑中的错误。结论性对齐/性能数据见 07 篇与 10 时间线；本文件只记「错在哪、为什么错、怎么发现、下次怎么避」。
> 背景：Rust 全量重写 worldgen（WorldgenRust/）功能链路闭合后，进入端到端性能优化。三个关键错误链条：①「Interpolated 慢 100× → 放弃」被 judge 推翻（双层污染）；② 诊断代码热路径每点执行污染；③「Java 60ms」错误基准导致「积累性差异」误判。
> 编号：本课题用 **P 系列**（perf-e2e），与 rust-mod-load（M 系列）/ functional-errors（F 系列）/ carver-port（C 系列）区分，避免跨课题编号混淆。
> 数据源：`cmd-output/e2e_java_vs_rust.txt`、`cmd-output/diag_pollution_cleanup.txt`、`cmd-output/base_breakdown.txt`、`cmd-output/aquifer_internal_profile.txt`、`.investigations/rust-mod-load/review-density-candidate.md`（judge 意见全文）。

---

## P1. 「Interpolated 慢 100× → 放弃」被 judge 推翻——双层污染假象

### 现象
- `density_interp_bench` 测量（`cmd-output/density_interp_correction.txt`）：裸 finalDensity 采样 6.4ms/chunk（0.065μs/pt），在最终树外层包 `Interpolated(4×4×8 网格)` 后 **632ms/chunk（约 100× 慢）**，且采样出现 44% 差异（三线性插值近似破坏对齐）。
- 据此得出结论：「Interpolated 优化方向错误（慢 + 破坏正确性），放弃」。
- judge 复核 **推翻** 该放弃结论：单层 Interpolated 包装纯 SplineDF（sloped_cheese，无缓存污染）= **加速 70×（83.74ms → 1.19ms）**；「慢 100×」是**双层包装污染假象**，Interpolated 不应放弃，反是密度优化正解。

### 根因（机制）
- **双层 Interpolated**：`overworld.json` 的 `minecraft:final_density` 树**内部已含 `minecraft:interpolated` 节点**。探针却在这棵 raw 树**外层再包一层** `Interpolated(new(raw,...))` → 形成**双层插值**。
- 外层 Interpolated 的 build_grid 网格点**跨多个 chunk**：外层网格点（如 x=16k、z=16m 边界）会反复清空**内层 chunk 网格缓存** → 内层 mesh 跨 chunk **雪崩重建**。
- 诊断探针（`density_interp_diag`，judge 新增）量化：内层 Interpolated 裸树每 chunk grid 采样 6029 次；外层再包后膨胀到**每 chunk 175 万次（112M/64，291×）**——这就是「632ms / 100×」的真相。
- 「44% 采样差异」同理是双层二次插值叠加 + 对插值语义的误解：MC 密度本就该插值（final_density 内部即 interpolated），单层插值误差是固有精度产物，非「不可用」。

### 定位（诊断方法）
- judge 审查（`.investigations/rust-mod-load/review-density-candidate.md`）用**新增单层对照诊断探针（density_interp_single + density_interp_diag）**复现：
  - 单层 Interpolated 包纯 SplineDF 子树 → 加速 70×——证明「100×」来自包装方式，非方向本身。
  - 网格采样计数对拍 → 6029 → 291× = 175 万次——证明内层网格跨 chunk 雪崩重建。
- **测量基准审查**（judge 三源核对：data 记录 ↔ 探针源码 ↔ 实际复现）是发现污染的入口——数值可复现（6.7/628ms 与记录 6.4/632 一致）**不代表基准正确**。

### 修复
- **撤销「Interpolated 放弃」结论**。Interpolated（或 DFC 直排）反是密度优化正解，方向与 AGENTS.md 铁律（SplineDF 树遍历 = 慢根源 → C2ME/DFC 直排 + 网格缓存）一致。
- 评估 Interpolated 性能时**用单层包装**（包纯 SplineDF 子树），不要包已含 interpolated 的 final_density 外层。
- 真正该放弃的是「在 final_density 外层再包一层 Interpolated」这一**本就冗余的双层插值写法**。

### 教训（可复用判错经验）
- **测量基准必须先确认无污染，再做优化方向结论**：一个「看起来可复现」的慢点，可能是多层包装 / 缓存雪崩的假象，不代表算法方向本身慢。
- **多层数组/网格 / 树包装需查「缓存雪崩」**：外层网格点跨 chunk 反复清空内层网格 → 灾难性膨胀（291×）。看到「某方向慢 N×」先问：是不是重复分层包装把缓存击穿了？
- **数值可复现 ≠ 基准正确**：复现同一数字只能证明测量稳定，不能证明测量衡量了正确的东西。judge 审查的价值正在于重新审视基准方法论，而非只复核数字。

---

## P2. 诊断代码热路径每点执行污染（27% 退化）

### 现象
- 用户提醒「断点污染」坑后复查：诊断代码放在热路径**每点执行**，导致性能退化 **61.5ms → 44.9ms（27% 退化）**（`cmd-output/diag_pollution_cleanup.txt`，region 200,200 单线程 median）。
- 污染点（每 chunk 98,304 点执行）：

| 污染点 | 位置 | 每点执行 | 清理后 |
|---|---|---|---|
| `AQPROF_ENABLED.load(Relaxed)` | aquifer.apply | 98304 次/chunk atomic load | 移除 |
| `Instant::now()` ×3 | terrain.fill_chunk | 98304 次/chunk | 移除 |
| `std::env::var("WG_SKIP_AQUIFER")` | terrain.classify | 98304 次/chunk env 查询 | 移到 chunk 级 |
| `std::env::var("WG_SKIP_OREVEIN")` | worldgen_handle 矿脉循环 | 每 rock 点 env 查询 | 移到 chunk 级 |

### 根因（机制）
- 诊断代码（env 查询 / atomic load / `Instant::now`）即使**门控默认关闭**，门控判断本身就有开销：`std::env::var` 每次查环境变量表、`AtomicBool::load` 每次原子读取、`Instant::now` 每次时钟采样。
- 这些操作在 **98304 点/ chunk × 16 chunk 批量**下被放大到 `~157 万次/批量`，单点微秒级开销累积成显式 27% 退化。
- 本质 = **「测量/探针污染铁律」同族的另一种形态**：先前铁律关注**计时探针**（WG_PROFILE 每采样 steady_clock + 原子 → 原子竞争）；本次是**诊断/门控代码本身**（env/atomic/now 每点执行）污染数据面，非计时探针竞争。

### 定位（诊断方法）
- **用户主动提醒「断点污染」坑**——经验触发（不是工具发现），指出「每点执行诊断」这个模式要警惕。
- 复查代码路径确认 `fill_chunk`/`classify`/`aquifer.apply`/矿脉循环里的 env/atomic/now 调用点都在内层点循环里。

### 修复
- 迁移到 **chunk 级门控**：env 查询（`WG_SKIP_AQUIFER`/`WG_SKIP_OREVEIN`）在 **chunk 开头判断一次**，读入局部变量传进点循环；`Instant::now` ×3 移出点循环或仅 chunk 级采样；`AQPROF_ENABLED.load` 移除（或编译期 feature gate）。
- 修复后性能从 61.5ms 恢复到 **44.9ms**。

### 教训（可复用判错经验）
- **诊断代码绝不能放热路径每点执行**——即使门控默认关闭，env 查询 / atomic load / `Instant::now` 自身就有开销；点级（98304×/chunk）放大成千上万次。
- **诊断门控必须 chunk 级判断一次，或编译期 feature gate**——把「是否跑诊断」的决定从「每点查」上移到「每 chunk 查一次」。
- 与「测量/探针污染铁律」同族：**性能数据可信度 = 数据面有无被诊断/探针代码污染**。点级诊断既污染计时（探针竞争）又污染真实负载（env/atomic/now 每点执行）。

---

## P3. 「Java 60ms」错误基准导致「积累性差异」误判

### 现象
- 早期 `fair_java_vs_rust.txt`（`cmd-output/`）测出「Java 60ms/chunk」，据此判定 Rust 与 Java 接近 / 已达标。
- 本次端到端精确对比（`cmd-output/e2e_java_vs_rust.txt`，region 200,200，单线程，2026-08-29）修正：**Java 原版（WorldGenBench FULL 含树花植被，充分预热 JIT）稳定后 chunk 5-24ms，中位数 ~8-9ms/chunk**（排除第 1 个 298ms 冷启动）。
- Rust（fill_chunk_blocks 无树花，清理诊断污染后）：median **44.9ms/chunk** → Rust 比 Java 慢 **~5 倍**。

### 根因（机制）
- **JVM JIT 预热不足**：Java 基准若未充分预热，首个 chunk 的冷启动开销（JIT 编译、类加载、解释执行）会把中位数拉高。早期「Java 60ms」测在 **JIT 未热**状态下，是**错误基准**。
- **「积累性差异」被错误基准掩盖**：逐段优化（`fill_chunk` 内一段 vs 上一段）用「Java 60ms」当锚点，会让 Rust 看似「每段都在进步 / 已达标」——但真实 Java 只要 8-9ms，Rust 远未达标。逐段 vs 上一段的相对优化**不暴露与最终标准的绝对距离**。

### 定位（诊断方法）
- **用户提出「积累性差异」担忧**（AGENTS.md 端到端性能对比铁律的催生者）：点出「逐段优化用错误基准会掩盖 Rust 未达标的真实差距」。
- 端到端公平对比：`e2e_java_vs_rust.txt` 用**同一 region（200,200）、单线程、同测量口径**，且 Java 侧 `WorldGenBench FULL` **充分预热后取稳定中位数**、排除首个冷启动 chunk——修正后暴露真实 5× 差距。

### 修复
- **性能优化必须端到端对比充分预热的 Java 原版**（最终标准 = 比 Java 强），禁止只做「逐段 vs 上一段」。
- Java 基准必须**充分预热**（worldgen bench warmup 后 JIT 热，取稳定中位数，排除首个冷启动 chunk）。
- AGENTS.md 新增「端到端性能对比铁律」（用户拍板）。

### 教训（可复用判错经验）
- **端到端对比必须用充分预热的 Java 基准，禁止逐段 vs 上一段**——逐段相对优化掩盖「积累性差异」，让 Rust 在错误锚点（未热 Java）上误判达标。
- **Java（JVM）基准必须先让 JIT 热起来**：冷启动 chunk（JIT 编译/类加载/解释）会把中位数拉高成假慢/假公平；取稳定中位数 + 排除首个冷启动 chunk。
- **「相对 vs 绝对」陷阱**：优化对比的锚点必须是最终标准（充分预热的 Java），不是「上一段 Rust」。相对进步 ≠ 达标。

---

## 附：错误 → 根因 速查表（一页索引）

| 现象/信号 | 根因 | 判错要点 |
|---|---|---|
| 「Interpolated 慢 100× → 放弃」被 judge 推翻（P1） | **双层 Interpolated 污染**：final_density 树内部已含 interpolated，外层再包一层 → build_grid 网格点跨 chunk 反复清空内层网格 → **291× 雪崩重建**（6029 → 175 万次/chunk）；单层 Interpolated 包纯 SplineDF 实际**加速 70×** | **测量基准先确认无污染再下优化方向结论**；数值可复现 ≠ 基准正确；**多层网格/树包装先查缓存雪崩**（外层点跨 chunk 击穿内层网格） |
| 诊断代码热路径每点执行 → 27% 退化（61.5→44.9ms）（P2） | env 查询/`AtomicBool::load`/`Instant::now` 即使门控关闭，自身每点执行（98304×/chunk）仍放大到 ~157 万次/批量 | **诊断门控必须 chunk 级判断一次或编译期 feature gate**；诊断代码绝不放热路径每点执行；与「测量/探针污染铁律」同族 |
| 「Java 60ms」错误基准 → 误判 Rust 达标（P3） | Java bench **JIT 未预热**，冷启动开销拉高中位数；「积累性差异」被逐段 vs 上一段的相对优化掩盖 | **端到端对比必须用充分预热的 Java 基准**（稳定中位数 + 排除首个冷启动 chunk）；**禁止逐段 vs 上一段**，锚点必须是最终标准 |
