# 知识库草稿 — 260911-01（perf-release-260910-08 遗留 5 条 + 本块 2 条）

> 产出者：knowledge subagent（本稿）；发现时间统一 260911-01（部分素材事实发生于 260910-08 块，按首次成文块标注）。
> 归置总览：
> | 素材 | 归置 | 形态 |
> |---|---|---|
> | E1 自证行硬门禁 | workflow-patterns **发现 #118**（新增，错误优先五段式） | 新条目 |
> | §4 可达性核对再归因 | workflow-patterns **发现 #110 补充案例** | 补充案例 |
> | E5 bench flags vs 出货默认 | workflow-patterns **发现 #66 补充案例** | 补充案例 |
> | E4 WorkingSet 峰值 ≠ 存活集 | workflow-patterns **发现 #119**（新增，错误优先五段式） | 新条目 |
> | E6 公众声明随门控核对 | workflow-patterns **发现 #120**（新增） | 新条目 |
> | 写入侧门控（只写不读缓存） | workflow-patterns **发现 #102 补充案例**（#100 缓存家族延伸） | 补充案例 |
> | 先修再发 + 防陈旧书面化 | workflow-patterns **发现 #121**（新增） | 新条目 |
>
> 去重核对记录（grep `workflow-patterns.md` + `build-tooling.md` 现有条目后判定）：
> - E1 与 #23（内容指纹 vs mtime）、#36（执行体三元组）、build-tooling #23 同族，但「自证行**硬门禁**（mismatch ⇒ 臂判 VOID + 硬返回）」这一**脚本机制**无既有条目承载（#36 只立了「核验」纪律，未立「不判即 VOID」的门禁形态）→ 立新条。
> - §4「优化不可达 ≠ 优化失败」是 #110（不可达证明三件套）的**归因侧对偶**（#110 管「怎么证不可达」，本条管「不可达后怎么归因收益」）→ 挂 #110 补充案例，不另立。
> - E5 与 #66（native bin-diag flags=0 vs vivo stageMask=3，诊断代理与生产执行体结论禁止互迁）**同一机制**（bench 默认值 ≠ 出货默认门控），E5 是第二实例且点名了隐蔽翻转点（`flags` 初值 0）→ 挂 #66 补充案例。
> - E4（内存指标语义）无既有条目（#51/#103 是吞吐噪声带，非内存指标语义）→ 立新条。
> - E6（公众文档漂移）无既有条目 → 立新条。
> - 写入侧门控是 #102（缓存三面：谁写+谁读+clear 波及面）的**第四面**（消费方集合为空 ⇒ 写路径整体短路）→ 挂 #102 补充案例。
> - 「先修再发」发版前置模式无既有条目（#116 等价性门是迁移域；RELEASE-CONTRACT 是 schema 不是模式）→ 立新条，并吸收 judge C2 防陈旧书面化判据。

---

## 发现 #118（最高价值·错误优先）: 替换执行体的实验，自证行必须做成硬门禁——`executed != want` 即判 VOID，只打印不判定 = 没有门禁（260910-08）

- **发现时间 / 发现者 / 置信度 / module**：260910-08（事实链 2026-09-10 23:45–261-01 间复核）；主会话（错误台账 E1）+ 本稿成文；**candidate**（判 VOID 机制已在 `.tmp/engine-ab2-260910-08/run_engine_ab2.ps1` 落地并在 260911-01 块 A/B 4 臂复用，4/4 PASS；confirmed 留用户）；workflow-patterns / 执行体核验（#36 执行体三元组的**实验驱动侧门禁**形态）。
- **来源定位**：`.investigations/perf-release-260910-08/errors-260910-08.md` E1/E2；`.investigations/perf-release-260910-08/MANIFEST.md` §1（执行体自证行硬门禁声明）/§3（修正版 A/B）；机制 = `versions/1.20.1/java/build.gradle:36-50`（`processResources.doFirst` 无条件覆盖）。
- **现象（错误链，五段式）**：
  - **现象**：引擎 A/B 两臂（`engNew` / `engOld1027`）wall 与 CPU 秒数几乎相同；逐日志核对 `[CppBridge] dll=` 自证行：`engOld1027` 臂脚本 `want = 7a411e01`（1.0.27），实际 `executed = 597e12ed`（**1.0.28**）——旧臂空跑；后续 4 个「引擎臂」同病（7 个日志同一 sha）。
  - **根因**（两层机制）：① **覆盖点错位**——`build.gradle` 的 `processResources.doFirst` **无条件**把 `target/release/worldgen.dll` 拷进资源目录，直接往 `src/main/resources/native` / `build/resources/main/native` 塞待测 dll 会在 `:runServer` 触发 `processResources` 时被撤销 ⇒ **唯一有效覆盖点在构建上游（`target/release/`），不是资源目录**；② **门禁缺失**——脚本**打印**了 `executed vs want` 但**没有判定**，空跑静默通过并被当成有效实验支撑结论（无效结论一度外泄为「引擎差 0.0298% ≈ 噪声」——见 E2，0.0298% 实为 #115 规范读法下的跨形态噪声锚）。
  - **定位**：`Select-String -Path logs\*.log -Pattern '\[CppBridge\] dll='` 一次比对全部臂即暴露（同值 = 两臂同执行体 = 零判别力）。
  - **修复**：驱动脚本 v2 三件：① 覆盖 `target/release/worldgen.dll`（先备份、臂尾还原并**回读 sha 校验**）；② 每臂清 `<tmp>/coreswap-native` 解压缓存（E9 家族防护）；③ **`executed sha != want sha` ⇒ 该臂判 VOID、写 results.txt、硬返回**（不再静默）。
  - **教训**：只打印不判定 = 没有门禁；「差异 ≤ 噪声 ⇒ 等价」类结论必须同时给出噪声锚来源 + 两臂 sha 对照，否则无法与空跑区分。
- **判据（MUST，可复用）**：
  1. 凡替换执行体的 A/B，每臂 MUST 做 `executed == want` 硬门禁：mismatch ⇒ 该臂自动 VOID（写入结果 + 提前退出），**不允许人工事后挑臂**；
  2. 替换点 MUST 选在**所有会覆盖它的构建步之上**——动手前先查构建脚本「谁在写这个文件」（gradle processResources 类任务都是候选覆盖者），往最终产物目录里塞是最常见错位；
  3. 与 #23（mtime/文件名不可信）/E9（解压缓存复用）同一家族：判据 = **内容指纹 + 硬门禁**；谱系存疑时用「同源重链 sha 不变」反证（`cargo build --offline` 对当前源码无需重编 ⇒ 被测 dll = 当前源码）。
- **家族索引**：#36（执行体三元组——本条为其 A/B 驱动侧落地形态）、#23（内容指纹）、#81（行为化分支证据——`[WG-CONF]` 自证配套）、build-tooling #23/#30（陈旧产物家族）、#115（0.0298% 噪声锚——E2 无效结论的正确身份）。

---

## 发现 #110 补充案例（260910-08 → 260911-01 成文）: 「优化不可达 ≠ 优化失败」——归因前必须先做可达性核对，零收益可能本属预期

- **发现时间 / 发现者 / 置信度 / module**：260910-08（事实）；主会话（代码一手核对 + 运行期自证）；**candidate**；workflow-patterns / 归因纪律（#110 的**归因侧对偶**：#110 管「怎么证明路径不可达」，本条管「不可达之后收益归因怎么读」）。
- **来源定位**：`.investigations/perf-release-260910-08/MANIFEST.md` §3-§4；`.investigations/perf-release-260910-08/errors-260910-08.md` §4 机制链。
- **观察**：R3 的「内存优化」组件（terrain_cache cache-first / CaTerrainEntry / 3×3 Arc 快照 / MEMODIAG）全部落在 features 阶段或只被 features 消费；1.20.1 出厂 `mask=3` 结构性关闭 features（`worldgen_handle.rs:669-670`，`apply_features` 永不执行）⇒ 它们要省的「邻 chunk 二次地形求值」在 1.20.1 上**从来不存在**。引擎 A/B（同构建态单变量，1.0.27 vs 1.0.28，两形态）确认两形态均无速度差 ⇒ **零收益本属预期，不是"优化失败"**；1.21.6 侧 `resolveStageMask()` 逐行相同 ⇒ 同病。
- **判据（可复用）**：
  1. 跨版本共享引擎上做性能改动归因前，MUST 先做**可达性核对**（#110 三件套）再谈「优化有没有用」——优化目标路径被出货门控默认值关闭时，「无收益」是门控的必然结论，不能记成优化无效，也不能拿别的口径（见 #66 补充案例）硬凑出收益；
  2. 可达性核对的最廉价组合 = **运行期自证行**（`[WG-CONF]` flags/mask 字段）+ **行为化负对照**（`WG_CA_MEMODIAG=1` 跑满 4225 chunk ⇒ 目标阶段输出点零输出）+ 代码链（默认值来源 → 门控 → 调用点）；
  3. **反方向也要查**：不可达路径上可能有**反向成本**——本案 `fill_chunk_blocks` 在 `ca_min` 默认开时每 chunk 做 393KB `BlockColumn` clone + insert，而 1.20.1 无任何消费者（量级在噪声带内不可判定，但机制方向明确 ⇒ 产出候选微修，见 #102 补充案例）。
- **家族索引**：#110（主条——证明三件套）、#25（生产路径可达性）、#98（修复须在生产口径下真生效）、#66 补充案例（本块同报的口径面）。

---

## 发现 #66 补充案例（260910-08）: bench 侧「默认值」≠ 产品出货默认值——`flags` 初值 0 是隐蔽的口径翻转点（E5）

- **发现时间 / 发现者 / 置信度 / module**：260910-08（事实）；subagent 只读代码审查（read/grep）定位；**candidate**；workflow-patterns / 口径可比性（#66 主条「诊断代理 vs 生产执行体结论禁止互迁」的 bench 侧第二实例）。
- **来源定位**：`.investigations/perf-release-260910-08/errors-260910-08.md` E5；机制 = `worldgen-core/src/bin-diag/camin_bench.rs` 从不调 `wg_set_flags`（handle `flags` 初值 0，`worldgen_handle.rs:503`）vs 出货 `CppBridge.resolveStageMask()` 默认 `0b011`。
- **观察**：1.21.6 侧「ca_min 内存优化」量化收益（289.1 → 177.9 ms/chunk，−72%）来自 bin-diag bench，其 `flags=0` ⇒ `skip_features=false` ⇒ **features 全跑**；出货默认 `mask=3` ⇒ features 关。两者不是同一条执行路径——该 −72% 不是该项优化的通用收益。
- **判据（可复用）**：
  1. 引用「优化收益」数字前，MUST 与**出货默认门控**同源核对——bench 里的「默认值」往往是**结构体初值**（`flags: 0`），不是产品的出厂门控（`mask=3`）；这是最隐蔽的默认翻转点（handle 初值不看门控解析代码）；
  2. 收益数字引用时同行声明 §9.7 三要素（载体 = bin-diag bench / 覆盖面 = features 全跑 / 与出货口径不可比）。
- **家族索引**：#66（主条——bin-diag vs vivo 出货 mod 同一翻转机制）、#33（载具可比性）、#14/#25（阶段同源 / 可达性）、#110 补充案例（本块同源素材的归因面）。

---

## 发现 #119（最高价值·错误优先）: JVM WorkingSet/PrivateMemory **峰值 ≠ 存活集**——内存回归判定必须固定堆或读 GC live-set，且先测同配置摆动带（260910-08）

- **发现时间 / 发现者 / 置信度 / module**：260910-08（事实，自查拦截未外泄）；主会话（交替复跑自证 + 固定堆判定实验）；**candidate**（机理 + 判定性实验；confirmed 留用户）；workflow-patterns / 性能指标语义（#83 分母分场景的**内存指标维**）。
- **来源定位**：`.investigations/perf-release-260910-08/errors-260910-08.md` E4；`MANIFEST.md` §5（内存探针全表）。
- **现象（错误链，五段式）**：
  - **现象**：单对测量得 1.0.28 `maxWs ≈ 5973 MB` vs 1.0.27 `2713 MB`——看似「1.0.28 多占 3.3 GB」的内存回归。
  - **根因（机制）**：JVM `WorkingSet`/`PrivateMemory` 峰值 = **G1 堆高水位**（未设 `-Xmx` 时 JVM 默认 MaxHeap 12.7GB），由分配节奏与 GC 时机决定，**不是存活集**——同 dll 同配置交替复跑 4 次：`6505 / 4666 / 2984 / 5976 MB`，**同配置自身摆动 1.7×**，单对差值毫无判别力。
  - **定位**：不轻信单对读数，做**交替复跑**（1028/1027/1027/1028）——组内摆动 ≥ 组间差 ⇒ 指标语义存疑；再查 `build.gradle` 未设 `-Xmx` ⇒ 高水位解释成立。
  - **修复/判定**：改**固定堆判定**——`-Xmx2G` 两臂：1.0.28 = 36 s / 469 cpuSec / 峰值 3043 MB，1.0.27 = 35 s / 469 cpuSec / 2223 MB ⇒ 两引擎在玩家常用堆档下均满速跑完，**无阻塞性内存回归**；两臂峰值差仍在已证摆动带内（n=1/臂）**不作等价结论**（只作「无阻塞证据」）。
  - **教训**：拿单对峰值差说事 = 用堆高水位冒充存活集。
- **判据（MUST，可复用）**：
  1. 内存回归判定 MUST 用固定 `-Xmx`（判定「常用堆档下能否满速跑完」）或 GC 日志 live-set，**禁止**直接引用无界堆的 WorkingSet 峰值；
  2. 任何内存指标对比前 MUST 先测**同配置 run-to-run 摆动带**（本机该指标 1.5–1.7×）——组间差被组内摆动覆盖 ⇒ 不作结论（与 #51「噪声基线前置」同构，#119 是其在内存维的实例）；
  3. 「无阻塞证据」与「等价结论」分开写：n=1/臂只能支持前者。
- **家族索引**：#51（噪声基线与信号同阶）、#103（±10% 噪声带——本条为内存维同构）、#83（指标绑场景）、#64（未受控观察降级——单对读数不结论）。

---

## 发现 #120: 面向公众的能力声明必须与「出货默认门控」同源核对——「引擎里有实现」≠「产品做了这件事」（260910-08）

- **发现时间 / 发现者 / 置信度 / module**：260910-08（事实）；主会话（README 双语一手核对）；**candidate**；workflow-patterns / 文档与门控一致性（#66/#110 家族的**公众文档维**）。
- **来源定位**：`.investigations/perf-release-260910-08/errors-260910-08.md` E6；漂移点 = `README.md:27` / `README.zh-CN.md:28` vs 出厂 `mask=3`（`CppBridge.resolveStageMask()`）+ `worldgen_handle.rs:669-670`/`:832`。
- **观察（错误链要点）**：README 声称主世界「density → aquifer → ore veins → surface rules → carvers → features」全原生；实际出厂默认 `mask=3` 切断 `apply_features` 与 carvers ⇒ **carvers/features 实际由 Java vanilla 执行**。该声明在 260909-03 用户决策（矿物/树/装饰层不接管、保 MOD 兼容）后即过时——门控默认值改了，公众文档没跟着核。修复 = README 双语改措辞（"carvers and features stay vanilla Java for mod compatibility"）+ "How It Works" 段补出厂生效范围。
- **判据（MUST，可复用）**：
  1. 能力声明（README / Release notes / 公告）按「**出货默认门控下实际生效的路径**」写，不按「引擎里有实现」写——mask/ca_min/env 默认值是声明的一部分；
  2. **改门控默认值的 commit MUST 触发一次公众文档核对**（同源 grep：被改门控对应的声称语句），否则文档漂移是静默的；
  3. 与发版联动：Release notes 草稿（RELEASE-CONTRACT §3）引用的性能/能力数字，出单前按 E5/#66 判据同源复核一遍（本块 1.0.28 changelog 即按此口径写：收益归 async 形态、ca_min 微修明写「无性能承诺」）。
- **家族索引**：#66（bench/文档双面的口径同源）、#110（可达性——「有实现」不等于「可达」）、#120 与 RELEASE-CONTRACT §3 联动。

---

## 发现 #102 补充案例（260911-01，#100 缓存家族第四面）: 「只写不读」缓存路径的**写入侧门控**——消费点集合为空 ⇒ 写路径整体短路（连写都不该做）

- **发现时间 / 发现者 / 置信度 / module**：260911-01；主会话（260910-08 可达性核对定位 + 本块 patch/A/B 落地）；**candidate**（A/B 无回归 + 静态构造性论证；judge 意见已入 RELEASE-1.0.28 工单；confirmed 留用户）；workflow-patterns / 缓存三面核对（#100「谁写+谁读」、#102「+clear 波及面」之外的**第四面：写入门控**）。
- **来源定位**：`.investigations/perf-release-260911-01/patch-ca-min-gate-260911-01.md`（patch 三处：flags 提前 load + `skip_features` 同源提取、门控 `ca_min` → `ca_min && !skip_features`、删两处重复声明）；`.investigations/perf-release-260911-01/record-ab-260911-01.md`（4 臂交错 A/B + 判读）；上游 `.investigations/perf-release-260910-08/MANIFEST.md` §4.5。
- **观察**：terrain_cache 的消费点全部在 `apply_features` 邻 chunk 读；出厂 `mask=3` 下 features 关 ⇒ 缓存**只写不读**，每 chunk 纯付 ~393KB `BlockColumn` clone + HashMap insert（cpuSec +1.7%，噪声带内不可判定但机制方向明确）。修复 = 判据化门控：**盘点缓存消费点集合 ⇒ 为空 ⇒ 写路径整体短路**（直算 `fill_terrain_column`，同一函数产物，输出逐位相同——构造性等价）；features 启用路径逐字未动。
- **A/B 判读（口径：同构建态单变量，n=2/臂交错）**：吞吐 post 36.5 s vs pre 35.5 s（~3%，#103 ±10% 带内，不可判差异——本修是「去纯成本」非可测提速，预期内）；maxWs 跨臂摆动（pre 自身 4257 vs 6428 = 1.5×）覆盖组间差，不作结论（#119）；cpuSec 均值 463 vs 464 零差；行为不变性 = 静态构造性论证 + `[WG-CONF]` 两臂逐字段一致（#81），region 逐块对拍未做（Degraded 声明）。
- **判据（可复用）**：
  1. 缓存类路径的三面核对（#100/#102：谁写 / 谁读 / clear 波及面）之后**加第四问：「消费点集合在出货门控下是否为空」**——为空 ⇒ 写侧整体门控短路（`gate = 原开关 && 消费路径可达`），不是只省读；
  2. 门控判定必须与阶段门控**同源提取**（本案 `skip_features` 判法从 `:669` 逐字提取到缓存判定处，消除「缓存按旧 flags 走、features 按新 flags 判」的双判源潜在不一致）；
  3. 等价论证可走构造性路线：skip 模式下 pre-fix 恒走 miss 分支（单次预生成 (cx,cz) 不重复 ⇒ hit 永不触发）= 直算，post-fix 直算同一函数 ⇒ 逐位相同（构造性）；缓存条目本就是同一函数产物（260909-04 E1b 已验证）；
  4. 「无回归门」即可发版的口径见 #121。
- **家族索引**：#100（缓存失效当结论——第一二面）、#102（三面核对——本条加第四面）、#110（可达性核对提供「消费点集合」）、#81（`[WG-CONF]` 行为自证）、#103/#119（吞吐/内存噪声带读法）。

---

## 发现 #121: 「先修再发」发版前置模式——性能疑虑未归因前不出版；发版门是「修后 ≡ 修前（噪声带内）」，不是「修后有提升」；防陈旧证据入单（260911-01）

- **发现时间 / 发现者 / 置信度 / module**：260911-01；主会话（流程执行：发现 → patch → A/B → 工单）+ judge（RELEASE-1.0.28 工单审）；**candidate**；workflow-patterns / 发布流程纪律（#116 等价性门的**发布域**形态；RELEASE-CONTRACT 的**工单上游前置**）。
- **来源定位**：`.investigations/perf-release-260911-01/{record-ab-260911-01.md, patch-ca-min-gate-260911-01.md}`；`.artifacts/releases/RELEASE-CONTRACT.md`（schema/状态机/双 hash）；`.artifacts/releases/RELEASE-1.0.28.md` §2（验证记录 + 防陈旧书面化：重编 13:04 在 A/B 窗口 12:54-13:00 之后 + 源 commit 先后关系）；上游 `.investigations/perf-release-260910-08/MANIFEST.md` §7 待办①。
- **观察**：260910-08 块发现不可达路径上的纯成本（#102 补充案例）后，**没有**带着未归因疑虑出 1.0.28 工单，而是先修（门控微修）→ 同构建态单变量 A/B 证「修后 ≡ 修前」→ 才填单。发版门读法关键：门判据是**无回归**（吞吐/内存落在既有噪声带内 + 行为不变性），**不要求**修后提升——把「必须有提升」当门会逼出假提升结论；本修的收益是「移除纯成本路径」（结构性正确），吞吐上不可测（预期内，明写）。
- **判据（可复用）**：
  1. **性能疑虑未归因/未处置前不出版**——即使疑虑量级在噪声带内（+1.7% cpuSec），也先走「定位 → 微修 → A/B」闭环再出单；理由：工单是跨 session 契约，发布后的『已知边界』无法追溯补修；
  2. 发版门 = **「修后 ≡ 修前（既有噪声带内）」+ 行为不变性证据**（构造性等价 / `[WG-CONF]` 自证 / 指纹门），不是「修后有提升」；「无性能承诺」的修也要过同一道门（门证的是不变性，不是收益）；
  3. **防陈旧书面化（judge C2）**：工单内的执行体证据 MUST 写**时间先后关系**——最终 jar 重编时刻须晚于 A/B 窗口、且源 commit 提交时刻早于 A/B（本案：重编 2026-09-11 13:04 > A/B 12:54-13:00 > commit `7c46803`）；sha 三元组（jar / jar 内 dll / target dll）**重算于重编之后**，不沿用上一块转录——否则「A/B 验证的构建」与「发布的构建」可能是两个执行体（#23/#36 家族的工单形态）；
  4. A/B 臂 MUST 清解压缓存（E9 家族）+ 硬门禁自证行（#118）——工单上游实验与下游发版用同一套执行体纪律。
- **家族索引**：#116（等价性门——迁移域；本条为发布域）、#118（自证硬门禁——本条的实验前置）、#51/#103/#119（噪声带读法）、#102 补充案例（本模式的首个应用实例）、RELEASE-CONTRACT §5（hash 双重校对——本条管出单**前**，契约管交接**后**）。

---

## INDEX.md 追加段草稿（一行一条，追加到文件末尾）

```markdown
> 260911-01 追加：workflow-patterns 新增**发现 #118（最高价值·错误优先）**（替换执行体的实验，自证行必须做成**硬门禁**——260910-08 引擎 A/B 两臂空跑：`build.gradle` `processResources.doFirst` 无条件覆盖资源目录 dll ⇒ 唯一有效覆盖点 = 构建上游 `target/release/`，直接塞资源目录被构建撤销；脚本打印了 `executed vs want` 却不判定，空跑静默通过并外泄「0.0298% ≈ 噪声 ⇒ 引擎等价」无效结论；修复 = 驱动 v2：覆盖 target dll + 备份回读校验 + 每臂清解压缓存 + `executed != want` ⇒ 臂判 VOID 硬返回；判据 = 自证行 mismatch 即 VOID 不许人工挑臂 + 替换点须在覆盖它的构建步之上 + 与 #23/E9 同族判内容指纹）+ **发现 #119（最高价值·错误优先）**（JVM WorkingSet/PrivateMemory **峰值 ≠ 存活集**——单对测量「1.0.28 多占 3.3GB」被交替复跑自证：同 dll 同配置 2984 vs 4666 MB = **1.7× 摆动**（未设 -Xmx 时峰值 = G1 堆高水位，由分配节奏/GC 时机决定）；判据 = 内存回归 MUST 固定 `-Xmx` 或读 GC live-set + 先测同配置摆动带 + 「无阻塞证据」与「等价结论」分开写；-Xmx2G 判定实验两引擎均满速跑完）+ **发现 #120**（公众能力声明（README/Release notes）MUST 与**出货默认门控**同源核对——README 声称全管线原生而出厂 mask=3 切断 carvers/features（实际 Java vanilla 执行），260909-03 门控决策后文档静默漂移；判据 = 按「出货默认下实际生效路径」写 + 改门控默认值的 commit MUST 触发公众文档核对 + Release notes 数字出单前同源复核）+ **发现 #121**（「先修再发」发版前置模式——性能疑虑未归因前不出版：ca_min 只写不读缓存发现后先微修再出 1.0.28 工单；发版门 = 同构建态单变量 A/B 证「修后 ≡ 修前（噪声带内）」+ 行为不变性，**不是**「修后有提升」；防陈旧书面化（judge C2）= 工单写时间先后关系：重编时刻 > A/B 窗口 > 源 commit，sha 三元组重算于重编后不沿用转录）+ **发现 #110 补充案例**（「优化不可达 ≠ 优化失败」——R3 内存优化组件全落 features 阶段、1.20.1 出厂 mask=3 结构性关闭 ⇒ 要省的成本从来不存在，零收益本属预期；判据 = 归因前先可达性核对（自证行 + 行为化负对照 + 代码链）+ 不可达路径上反向查纯成本）+ **发现 #66 补充案例**（bench「默认值」≠ 产品出货默认——bin-diag camin_bench 从不调 wg_set_flags，handle `flags` 初值 0 ⇒ features 全跑，出货 mask=3 features 关，−72% 收益不是通用收益；判据 = 收益数字与出货门控同源核对 + 结构体初值是隐蔽翻转点）+ **发现 #102 补充案例（#100 缓存家族第四面）**（「只写不读」缓存路径的**写入侧门控**——terrain_cache 消费点全在 features 邻读、出厂下集合为空 ⇒ `gate = ca_min && !skip_features` 写路径整体短路，直算同函数构造性等价；A/B n=2 噪声带内无回归；判据 = 三面核对后加第四问「消费点集合在出货门控下是否为空」+ 门控判定与阶段门控同源提取）。来源：`.investigations/perf-release-260910-08/{MANIFEST.md,errors-260910-08.md}` + `.investigations/perf-release-260911-01/{knowledge-draft-260911-01.md,record-ab-260911-01.md,patch-ca-min-gate-260911-01.md}` + `.artifacts/releases/RELEASE-1.0.28.md`。
```

---

## 自检清单（SUBAGENT-KNOWLEDGE-GUIDE §四）

- [x] 价值门：7 条全部高/中价值（错误链 2、可复用判据 5），无一次性结论
- [x] E1/E4 按**五段式**（现象/根因/定位/修复/教训），根因为机制层非现象复述
- [x] 定位含诊断方法/工具（Select-String 全臂比对、交替复跑、-Xmx2G 判定实验、行为化负对照）
- [x] 被排除/降级标注保留（#102 A/B region 对拍未做 = Degraded 声明；#119 n=1/臂不作等价结论）
- [x] 载体：全部归 `knowledge/discovered/workflow-patterns.md`（归置判断含去重核对记录）
- [x] 数字全部来自主会话/工单一手记录（errors-260910-08.md / MANIFEST.md / record-ab-260911-01.md / RELEASE-1.0.28.md），无编造
- [x] 格式与 workflow-patterns.md 末尾现状对齐（发现时间/发现者/置信度/module + 来源定位 + 观察 + 判据 + 家族索引）
- [x] 编号顺延：新增 #118/#119/#120/#121（现有最大 #117）；补充案例挂 #110/#66/#102 不占新号
- [x] 置信度全部 candidate，confirmed 留用户（AI 不授予 confirmed）
