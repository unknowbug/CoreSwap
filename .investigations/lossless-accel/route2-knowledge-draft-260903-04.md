# 路线② FFI 工作包知识库产出草稿（260903-04，subagent 草稿，待主会话应用）

> 依据：SUBAGENT-KNOWLEDGE-GUIDE.md（价值门/载体映射/五段式）。主记录 = `.investigations/lossless-accel/route2-ffi-260903-04.md`；数据 = cmd-output/tri-cut2/3 + gpu-corner-probe2。数字全部来自上述实测记录，无编造。
> 价值门判定：第一部分 = 错误链 + 判据（高价值，必记，进 10 时间线）；第二部分 = 跨课题可复用判据（高价值，进 knowledge/discovered/）。

---

## 第一部分：10-timewise-archive.md 追加条目（追加到文件末尾）

```markdown
## 260903-04（lossless-accel 路线② FFI 工作包：spv 陈旧产物定案——「逐位一致」哨兵结论被陈旧二进制产物击穿）

### ✅ FFI shim + 三事实 + 决定性三路切分
W2 `gpu_ffi.cpp` C-ABI shim（build.ps1 `-Ffi`）+ W3 Rust 角点探针（bin-diag，LoadLibrary 动态加载零新依赖）。三事实实测（gpu-corner-probe-260903-04.txt）：① create ~64-75s，同 seed 第二实例无缓存同价（每次全量编译 pipeline）；② 串行 5.0µs/pt，双线程同 handle Mutex 反而 0.61×（GPU dispatch 异步流水，Mutex 不串行化 GPU 队列，readback 同步保正确性）；③ GPU vs DFC oracle 6144 点 f32_exact 仅 43.26% → 系统性 diff 非纯精度。tri-cut（同程序同坐标 CpuBackend.sample vs GpuDensityEngine.fill）：C++ CPU vs C++ GPU 自己就 major diff（16 点中 5 点，最大 0.502）→ **FFI/Rust 侧无罪，问题在引擎内部**。✅

### ✅ 三 worker fan-out（互斥候选并行）
.bA GPU fill 路径 / .bB CPU 参照单点采样（结论：CPU 参照正确——饱和值 -0.458333343 = DF_SQUEEZE clamp -1 属正常，错误在 GPU 侧；顺带发现 sampleInterpGrid y=320 grid[49] 越界读为独立真实缺陷，另立修复项）/ .bC 历史域考古（历史「逐位一致 maxDiff 3.1e-07」域 = seed …396 × x∈[0,63] × y∈[-64,-49]，新证据域外，结论保留须补域声明）。产物 .artifacts/lossless-accel/route2-tricut.bA/bB/bC.md。✅

### ✅ 决定性双 seed 切分 + 根因 = final_density.spv 陈旧产物（supersedes fan-out 归因方向）
已知值哨兵点 (784,160,-408) 历史验证 seed 下：旧 spv 输出 **0.0453032888 = 时间线 L1386 记录的 D23 修复前错误值**（正确 -0.458333343）——直接复现历史错值签名（tri-cut2）。证据链：① spv mtime 08-15 14:17 **早于** D23 修复提交 cc58e05（08-15 19:21）5 小时，commit 9de661e（19:22）提交的是修复前编译的 spv；② 08-23 `final_density.comp` 与 cpu_backend.h 同批重生成但 spv 未随之重编——生成器多产物部分更新失配。重编（gen_final_density.py → glslc → 部署，旧 spv 备份 .bak-pre-d23）后：**双 seed 23 点 major_diff=0**（tri-cut3），全量 6144 点 max_diff=9.18e-6（f32 ULP 级），rounded6 96.08%。✅ 已结案（根因闭合）

### 📌 记录指引
- 错误链五段式 → `.investigations/lossless-accel/lossless-accel-errors.md`（subagent 草稿应用）。
- 可复用判据（二进制产物无法从内容/时间戳判断新旧 + 逐位一致哨兵须配已知值哨兵点）→ build-tooling 发现（见 route2-knowledge-draft-260903-04.md 第二部分，建议 #12）。
- 状态：candidate（judge 待过）；confirmed 待用户。
```

---

## 第二部分：knowledge/discovered/ 候选条目

**载体评估**：建议写入 **`knowledge/discovered/build-tooling.md`**（发现 #12），理由：① 核心是「构建产物的陈旧性判定/多产物原子更新」——纯构建工具链坑，与该文件 #6（fs::copy 保留 mtime——产物判新旧用内容指纹）、#10（参照五要素）、#11（header 声明 vs 内容实测）构成同一家族递进链；② workflow-patterns 侧已有教训⑧的对账维度覆盖，本条是「部署产物本身陈旧 + 提交时间新鲜度误导」的产物域升级，归 build-tooling 载体更准确。次选 workflow-patterns.md（若主会话认为「哨兵结论必须配已知值哨兵点」的工作流属性更强，可拆两条分别入两文件——不建议，稀释）。

**INDEX.md 分类行增补建议**：「构建/工具链坑」行末尾追加：
`; spv/comp 多产物部分更新——生成器多产物重生成必须整体原子更新，逐位一致哨兵结论须配已知值哨兵点验产物健康（发现 #12，260903-04）`

**条目全文（追加到 build-tooling.md 末尾）**：

```markdown
## 发现 #12: 二进制产物（.spv 等）无法从内容判断新旧——生成器多产物重生成必须整体原子更新，「逐位一致」哨兵结论须配已知值哨兵点（260903-04）

- **发现时间**：260903-04（lossless-accel 路线② FFI 工作包）；**置信度**：candidate（根因经双 seed 重编复现闭环，judge 待过）；**module**：swe/build 通用。
- **来源定位**：GPU final_density pipeline 差异排查；证据 = tri-cut2/3 切分输出（.investigations/lossless-accel/cmd-output/）+ git 提交时间戳（cc58e05 08-15 19:21 / 9de661e 19:22）+ spv mtime 08-15 14:17。

### 现象

GPU 密度引擎 vs DFC-CPU oracle 6144 点 f32_exact 仅 43.26%、max_diff 0.5533——系统性 diff 非纯精度；tri-cut 证明 FFI/Rust 侧无罪、C++ CPU 与 GPU 自身 major diff（最大 0.502）。已知值哨兵点 (784,160,-408)（历史验证 seed）GPU 输出 0.0453032888——正是时间线 L1386 记录的 D23 修复**前**错误值（正确 -0.458333343）。而最终 density 源码、cpu_backend.h 均为 D23 修复后版本。

### 根因

`final_density.spv` 是 D23 修复**前**编译的陈旧产物：mtime 08-15 14:17 早于修复提交 cc58e05（08-15 19:21）5 小时，commit 9de661e（19:22）提交的 spv 是修复前编译的；08-23 `final_density.comp` 与 cpu_backend.h 同批重生成，但 **spv 不随之自动重编**（glslc 编译步骤脱节）——生成器多产物（comp / cpu_backend.h / spv）部分更新造成跨产物语义失配。机制层面：① **二进制产物无法从内容判断新旧**——不读内容无法知道它编码的是哪个版本的源；② **mtime 与提交时间新鲜度均具误导性**——mtime 05:14:17 类时间戳看着「新鲜」，提交时间 19:21 比 mtime 晚 5 小时，两处时间各看都对，合起来才是「产物早于修复」；③ 历史教训⑧（对账必须基于当前生成产物）针对 dump 对账域，本案升级为**部署产物本身陈旧**。

### 定位

决定性手段 = **已知值哨兵点**：(784,160,-408) 在历史验证 seed 下应输出 -0.458333343（DF_SQUEEZE clamp -1 饱和值），实测 0.0453032888 与时间线历史错值逐位吻合 → 直接锁定「旧语义产物」而非引擎 bug。辅以 tri-cut 同程序同坐标双路切分（排除 FFI/Rust/坐标/seed 错位）+ git 时间戳与 mtime 交叉（5 小时窗）。重编（gen_final_density.py → glslc → 部署，旧 spv 备份 .bak-pre-d23）后双 seed 23 点 major_diff=0、6144 点 max_diff=9.18e-6——闭环。

### 修复

① 重编 spv 并部署（旧产物备份）；② 判据固化：**生成器多产物（源模板/生成头/spv 二进制）重生成时必须整体原子更新**——改了任何一个生成输入，所有下游产物同批重编，构建脚本应把 spv 编译纳入与 comp/backend.h 同一入口；③ **任何「逐位一致 maxDiff ~e-07」类哨兵结论必须配一个已知值哨兵点做产物健康检查**——哨兵点的值域应含饱和/边界值（如 clamp -1 的 -0.458333343），饱和值丢失 = 产物语义级陈旧的即时签名。

### 教训/如何利用

- **判据**：拿到任何二进制生成产物，先问「它编译于哪次源状态」——mtime/提交时间/内容都答不了；直接跑已知值哨兵点，一测便知。
- **哨兵结论的反模式**：「同引擎 chunk(0,0) 全对 ≤7e-8」这类逐位一致只证明「该域内新旧产物恰好语义相同」（此处域 = 正 chunk 基址），**一致域外产物可能陈旧**——哨兵点必须覆盖历史修过的错误签名域（负 chunk/饱和值）。
- **家族谱系**：教训⑧（dump 对账须基于当前产物）→ 本条 #12（部署产物本身陈旧 + 提交新鲜度误导）；同文件 #6（mtime 不可靠→内容指纹）、#10（声明字段核对）、#11（声明 vs 内容实测）——共同上位原则：**「看起来对」的元数据一律不作产物健康判据，用可复现实测值验**。
- 上游主记录：`.investigations/lossless-accel/route2-ffi-260903-04.md`（根因闭合节）。
```

---

## 自检清单（SUBAGENT-KNOWLEDGE-GUIDE §四）

- [x] 价值门：两部分均高价值（错误链+判据），无低价值一次性结论混入（一次性数值简记带过）
- [x] 时间线条目带 260903-04 标签 + 状态标注（✅/❌→✅/📌），格式与 10 篇末尾现状对齐
- [x] 错误链五段式齐备（现象/根因/定位/修复/教训），根因为机制层（多产物部分更新失配+新鲜度误导），非现象复述
- [x] 定位含可复用诊断方法（已知值哨兵点/tri-cut/时间戳交叉）
- [x] 被排除/被证伪候选保留可见（.bB GPU 侧归因被 tri-cut3 证伪、.bC 域外成立，标注于时间线）
- [x] 载体选择给出评估与理由（build-tooling #12 > workflow-patterns），INDEX 增补行已给
- [x] 数字全部来自主记录/实测输出，无编造、无占位符
