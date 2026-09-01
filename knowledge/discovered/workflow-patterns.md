# Workflow 模式（judge/scout 触发点 + 反模式）

> 跨项目通用工作流规律。每条格式：发现时间/来源/置信度/module + 观察/证据/如何利用。
> 来源：CoreSwap -288 课题复盘（2026-08-09，retrospective-20260809-scout-judge.md）。

## 发现 #1: judge 审查门是强制环节，不是可选项（收敛门 ≠ 自评）

- 发现时间：2026-08-09；来源：CoreSwap -288 课题（14 轮分析全主会话自评，收尾补 judge 抓到 5 项问题）；置信度：确定；module：工作流
- 观察：v0.8 收敛门「主会话直接闭环」被误解为「自评即可」——漏了「+judge 审查门」。自评无法发现自身盲点：-288 的差异构成表不闭合 23%、phase13 归纳失真（「强候选」写成「✅」）、AQF 高位垃圾值未说明、retry cap 超限未记录——全部由补位 judge 抓到
- 如何利用：
  - **confirmed 授予前 MUST judge**（用户提醒才补 = 失败）
  - **重大转向 MUST judge**：结案重开、根因定论（如「C++ 无 bug」）、范围决策
  - **各阶段结论（candidate 授予）也应 judge**
  - **计划阶段就预置 judge 步骤**（todo 列表含 judge 项，不事后补）

## 发现 #2: 「机制未明」类大排查初期 MUST scout 勘探（管线/子系统全景前置）

- 发现时间：2026-08-09；来源：CoreSwap -288 课题（直接跳单点定位，carvers 晚发现 4+ 轮）；置信度：确定；module：工作流
- 观察：「入口已明」（有 mismatch 明细/坐标）≠「机制未明」（不知道差异来自哪个子系统/阶段）。-288 直接做密度层→aquifer→Beardifier→caves 树单点排查，绕了 4 轮（超 retry cap）才发现含水层 water = **CARVERS 阶段**（NOISE→CARVERS→SURFACE→FEATURE 管线摸底本该第一轮做）
- 如何利用：
  - 机制未明类大排查：初期 MUST scout 勘探（管线阶段/子系统依赖摸底），禁止主会话直接跳入单点定位
  - 勘探产物（.investigations/ 管线地图：MC 生成阶段 NOISE→CARVERS→STRUCTURE→FEATURE→SURFACE + 各阶段负责子系统）作为定位前置
  - 反模式警示：「差在 density 层」这类早期定论，若未先排除后续阶段（carvers/FEATURE）的产物差异，会误导整个排查

## 发现 #3: 块级真相必须读最终块，反射中间量不可信

- 发现时间：2026-08-09；来源：CoreSwap -288（AQF-J/CellCache 反射污染 vs NOISE-BLK 直读）；置信度：确定；module：验证方法
- 观察：Java 反射中间量（CellCache 缓存值、blockStateSampler 反射）受缓存污染（同点 3 次值不同），曾误导「Java aquifer 判 solid」等结论；直接读最终块（NOISE 阶段 chunk.getBlockState + chunk status 确认阶段）才是块级真相
- 如何利用：跨实现对比（Java vs C++）时：① 反射值不可信时改用「游戏同构遍历 + 直接调用」（AQF-APPLY：cns 遍历填 cache 后 aquifer.apply 直接调用）② 读块前先确认 chunk status（noise/carvers/surface/features——同一坐标不同阶段块不同）③ 阶段差异（carvers 挖洞/含水层填水）是「阶段产物」非「判定 bug」

## 发现 #4: 参照数据状态三查（SURFACE vs FULL）——阶段不同差异构成天差地别

- 发现时间：2026-08-09；来源：CoreSwap -288 重归因 + 300515 判定 + check_ref_status.py；置信度：确定；module：验证方法
- 观察：vanilla 参照 blocks 若导出到 SURFACE 状态（无 FEATURE）则 C++ 99.9986% 对齐；若导出到 FULL（含 carvers/岩石替换/村庄/树草）则差异 94%+ 全是 FEATURE。**同一 seed/区域，参照阶段不同，「差异构成」完全不同**——-288 参照含岩石替换（FULL）、8576/3200 参照无 FEATURE（SURFACE），直接用差异量对比会得出错误结论
- 如何利用：
  - 参照导出后 MUST 用脚本检查 FEATURE 产物（岩石替换/ore/草方块/村庄 dirt_path/树）判定状态（check_ref_status.py 模板）
  - 判定差异归属前先确认参照状态：SURFACE 差异 = 纯核心（density/aquifer/surface），FULL 差异 = 核心 + FEATURE 混合，必须分类剔除
  - 21 块插值课题（8576/3200 SURFACE 参照）是纯核心差异，不混 FEATURE——与 -288/300515（FULL）不可直接比
  - 教训：-288 海底边界 6710 块曾被当纯核心（e 翻转），实际混村庄地基/紫晶洞等 FEATURE（dirt_path 160、amethyst 903）

## 发现 #5: FEATURE 独立于地形生成实心块（冰山/村庄/紫晶洞）——海底差异先排除 FEATURE

- 发现时间：2026-08-09；来源：用户早期 CoreSwap bug 观察（0,0 ±250 格外无陆地时冰山仍生成）+ -288 重归因；置信度：确定；module：领域知识
- 观察：冰山（frozen ocean placed_feature）、村庄房屋/土径、紫晶洞等 FEATURE 在**无 density 支撑**（density<0 判水处）也能放置实心方块——「无陆地也生成」是 FEATURE 的本质。因此「C++ water vs vanilla 实心」的差异不能默认是 aquifer 判定 bug，必须先排除 FEATURE 方块
- 如何利用：
  - 分析 water↔solid 差异时：先按 FEATURE 块清单（岩石替换/ore/dirt_path/紫晶洞/树草/结构方块）分类剔除，剩余才是核心判定差
  - 判定「岛/山」归属：NOISE 阶段（FEATURE 之前）已有 solid = 核心（aquifer/density）；仅 FULL 阶段有 = FEATURE 放置
  - 反模式：phase2-7 把「距村庄 24 格外」当「非结构」——村庄本体方块放置不需要 Beardifier（那是 density 修正），距离只排除 Beardifier 不排除村庄方块本身

## 发现 #6: getChunk 状态语义——「至少 N」而非「恰好 N」，阶段隔离要立即验证

- 发现时间：2026-08-09；来源：CoreSwap -288 SURFACE 参照导出失败（连带推进）；置信度：确定；module：工具坑
- 观察：`world.getChunk(x,z,ChunkStatus.SURFACE,true)` 在主循环中返回的 chunk 可能已被连带推进到 FULL（stat 验证新参照仍含岩石替换和esite 8796）——「SURFACE 参照」方案失效；而 NOISE-BLK（L477 请求 NOISE 后**立即**打印 getStatus() 验证）是可靠的阶段隔离
- 如何利用：
  - 请求指定阶段 chunk 后 MUST 立即验证实际状态（打印 chunk.getStatus()），不能假设返回值恰好是请求阶段
  - 服务器主循环/后台线程会连带推进 chunk——阶段敏感探针（NOISE-BLK/EstDiagN）要在请求后第一时间读，且用 status 打印留证据
  - 想要「无 FEATURE 参照」：不要试图导出 SURFACE 状态 blocks（会被连带 FULL），改用 NOISE-BLK 直读单列/单点

## 发现 #7: 多疑点冲突 MUST 并行 fan-out——分叉即派 worker，禁止主会话逐个自推

- 发现时间：2026-08-09；来源：CoreSwap -288 未闭合课题（B3 (a)/(b) 子候选自推多轮 + 04 篇结论冲突）；置信度：确定；module：工作流
- 观察：当判定树分叉出 ≥2 个互斥机制候选时，主会话「逐个自推」看似收敛（每个候选都可线性验证），实际会陷入深钻循环（-288 的 B3 (b) 子候选：splitter 复现→液面链→est→r/s/t 点多轮才收；期间用户两次提醒「派 worker」「启动 judge」）。并行 fan-out 各派一个 worker 验证一个候选，一轮出全部结论，效率量级提升
- 如何利用：
  - **分叉即 fan-out**：判定树出现 ≥2 互斥候选（含子假设再分叉、知识库结论与新证据冲突）→ MUST 并行派 worker（core.fanout 产 .bN），禁止主会话逐个自推
  - 触发场景三：① 同一现象多机制候选（e 翻转/pocket/Beardifier）② 旧结论 vs 新证据冲突（04 篇「Java e=0」vs NOISE-BLK NOISE 阶段 stone）③ 子候选分叉（(a) splitter 差/(b) 液面输入差）
  - **不因「候选小/看起来简单」自推**——自推的判断成本（上下文连续性）远高于派 worker 的隔离成本
  - 三触发点并列独立：scout（机制未明勘探）→ fan-out（多假设分叉）→ judge（结论审查），任一触发即执行
  - 反模式警示：主会话深钻到「第二轮仍无定论」时自查是否已分叉——是则立即 fan-out（AGENTS.md fan-out 强制触发点 2026-08-09 固化）
## 发现 #8」，该文件当前最大为 #7）；
- `knowledge/INDEX.md` 分类入口「工作流模式」行的说明列末尾追加「、接管单阶段后的后续阶段上下文依赖（2026-08-31）」。

## 发现 #8: 接管世界生成单阶段后的后续阶段上下文依赖（Minecraft modding 通用）

- **发现时间**：2026-08-31；**发现者**：multiworld-port session（M14）；**来源定位**：`.investigations/multiworld-port/multiworld-errors.md` M14；**置信度**：candidate（现象三方对照实锤，机制方向待查）；**module**：workflow / Minecraft modding。
- **观察**：mixin/注入接管世界生成管线的一个阶段（如 populateNoise/NOISE）后，后续阶段（feature 装饰 applyBiomeDecoration / SURFACE / lighting）对被接管阶段的**上下文依赖**会暴露——本例：Rust fill 的下界地形与 vanilla 高度一致，但 vanilla 后续 feature 装饰拿到的 biome/feature 上下文被污染（主世界森林的树 feature 铺满下界 chunk）。
- **证据**：三方对照（vanilla 导出 vs Rust fill 一致 vs 实机存档橡树海洋）锁死错乱块来自 vanilla feature 阶段而非自家 fill；F3 biome 判定正确排除判定算法，锁定上下文传递链。
- **如何利用**：
  - **审计清单**：被接管阶段之后的**每个 vanilla 阶段**，其输入依赖是否仍满足——biome 上下文（chunk biome 属性在 fill 后是否刷新）、NoiseConfig 状态（climate 采样）、chunk Status 推进（**Status 不推进会导致 chunk 永不重生成**）、高度图依赖。
  - mod 接管世界生成的验收不能只验「被替换阶段的输出正确」，必须端到端验收运行时存档（实况含全部后续阶段产物）——单阶段对拍全绿 ≠ 集成正确。
  - 同族风险：任何「替换框架管线一段」的 mod 模式（不只是 worldgen：事件接管、渲染 pass 替换）都要问「下游阶段吃我什么状态」。

> **【2026-09-01 更新】** #8 原始案例（M14）最终根因已由 M16 定案为 JNI 写入路径 id 域错位（见 compiler-idioms.md 发现 #6）——非 feature 阶段上下文问题。「审计清单」价值保留（「下游阶段吃我什么状态」检查方法仍有效）；本例根因见 M16。

## 发现 #9: 一次性诊断产物禁止放「约定目录 = 自动构建范围」——临时产物必须有唯一隔离区

- **发现时间**：2026-09-03；**发现者**：1.0.22 发版 session；**来源定位**：`WorldgenRust/src/bin/` 过时诊断 bin 阻塞发版事件 + AGENTS.md 八.13「临时文件唯一区纪律」；**置信度**：candidate（用户已拍板纪律并落地修复，本发现未走 judge）；**module**：workflow / 构建工程。
- **观察**：cargo 只自动编译 `src/bin/` 下所有文件（npm 侧 src 扫描、CMake glob 同族），该目录里积累的一次性诊断 bin 过时后无人维护，却仍参与每次全量构建。1.0.22 发版时 `cargo build --release` 因其中 23 个 bin 仍用旧 `RsSplitter` API（`Aquifer::new` 签名已换 `XoroshiroSplitter`）报 E0308 阻塞发版——而发版产物只需 cdylib（dll），这些 bin 根本不在发版产物内。致灾 = 两因素叠加：①「临时产物位置无约束」（写进明面约定目录）②「构建范围 = 全目录」（约定目录全量自动编译）。
- **证据**：E0308 全部指向 bin 内旧 API 调用；分两批迁出（先 5 个，重建后又暴露一批 → 共 23 个 → `WorldgenRust/src/bin-diag/`，cargo 不扫描该目录）后全量构建 exit 0，发版产物（cdylib）不受任何影响——证明这批 bin 与发版零依赖，纯属「位置无约束 × 全目录构建」的结构性问题。**第二轮才见底**也是证据：首次报错列表只是「先编译失败的那批」，cargo 不会一次报完所有坏 bin——清此类债必须迭代到全绿。
- **如何利用**：
  - **判据一**：任何构建系统的「约定目录 = 自动构建范围」（cargo `src/bin/`、npm 打包器 src 扫描、CMake `file(GLOB)`）都**不要放一次性脚本/诊断程序**——放进去即等于「永久参与构建 + 永久欠维护债」。
  - **判据二**：临时产物必须有唯一隔离区，**靠纪律不如靠机制**——最可靠的隔离是目录选择本身（放构建器不扫的目录），而不是「记得别乱放」；CoreSwap 落地：Python/一次性脚本 → 仓库根 `.tmp/`，Rust 诊断 bin → `WorldgenRust/src/bin-diag/`（要用时 rustc 单编或临时挪入）。`src/bin/` 只保留「随库维护、全量编译必须绿」的正式 bin。
  - **判据三**：发版只需库产物时 `cargo build --release --lib` 可救急，但**全量绿仍保留为发版前检查**——全量构建是过时产物的探测器，全量红即暴露「有产物已脱离维护」。
  - 反模式警示：诊断 bin 与生产代码共享 crate 时，API 演进不会「顺带」修好 bin（无 CI 强制时更不会）——过时是默认轨迹，隔离是唯一低成本对策。
- **同族已知坑**：gradle 换机器/新 shell 直接 `gradle` 不带 `GRADLE_USER_HOME` → native-platform.dll 加载失败（build-tooling.md 发现 #4）——环境类「约定不生效」同族：工具的 home/缓存必须显式指到工作区内。





## 发现 #10: cppReplace 存档口径残差的三阶段归因法——先分阶段再定位机制

- **发现时间**：2026-09-05；**发现者**：nether-save-full session（B1 定论轮）；**来源定位**：`.investigations/nether-save-full/`（residual-interpretation + .b1/.b2 + judge-review）；**置信度**：candidate（三方实验数据实锤，模式层面复用性待下一案例）；**module**：workflow / Minecraft modding / 验证方法。
- **观察**：替换模式（如 cppReplace 只接管 noise+surface，Java carvers/features 仍在替换后地形上运行）下的存档口径残差是**多阶段混合产物**——直接把残差对到单一层（如 surface rule 条件链）会得出错误归因（错误 E6：B1 52k 块大宗互换一度被归因 surface rule 条件链，实为 feature 阶段产物在两种基底地形上的命中/形态差）。正确做法是**三阶段归因**：
  1. **阶段分解**：先分清 noise/surface（替换方 = Rust）与 carvers/features（存续方 = Java）各自贡献，再定位机制；
  2. **消融判别**：`WG_SKIP_SURFACE=1` 重跑——surface 关掉后残差从 93.55% 掉到 55.18% 证明 surface 是实心块主来源；且 blobs 不触发（stone 基底非 netherrack → blackstone=0）反向证明 blobs 是 feature 阶段、依赖 netherrack 基底；
  3. **纯替换方基线**：ctypes 直连 dll（或 rlib 直跑）取得纯 Rust 输出 vs 参照（本轮 77.43%，与存档口径 93.55% 载体不同不可比，§9.7）——分离「替换方自身缺口」与「存续阶段叠加产物」。
- **如何利用**：
  - **判别手段一（消融）**：单阶段开关（如 WG_SKIP_SURFACE）A/B 重跑——残差量级变化 + 依赖块（如 blackstone 依赖 netherrack 基底）是否消失，直接指认产物所属阶段；
  - **判别手段二（直连基线）**：ctypes/FFI 直连替换方库跑同区域——排除 Java 存续阶段干扰，取得替换方独立口径；与 rlib 直跑对拍可顺带验证 FFI 层确定性（本轮 cell 级 0 差异）；
  - **biome/来源分桶**：残差按 biome 列分桶——若差异 100% 落在 vanilla 某列（本轮 basalt_deltas），排除「源分配差」，收窄到「同源产物在不同基底上的表现差」；
  - **同 dll 重跑非确定性容差判据**：存续阶段（Java feature 邻块写入调度）本身非确定——同 dll 两次完整 run 相差 369 块（93.5156%→93.5508%）。**存档口径对齐指标必须声明该容差**：同 dll 重跑块级差 ≤ 百分级（千分位级百分比波动）属调度噪声，不构成实现回归判据；跨口径比较（探针 vs 存档 vs 纯 Rust）一律 §9.7 三要素声明。
  - **先消融后归因**：任何「残差 → 某层机制」的归因结论，出手前必须已有至少一个阶段消融或直连基线证据，否则降为 draft（反模式见 E6）。
- **同族模式**：发现 #4（参照数据状态三查 SURFACE vs FULL——参照阶段决定差异构成）是「对照侧」的前置；本发现是「被测侧」（替换模式运行时残差）的阶段归因，两者合用构成替换模式验证的完整口径纪律。

## 发现 #11: 嵌套接管管线复查——「内层全管线 vs 外层分步拦截」的双跑风险（2026-09-07）

- **发现时间**：2026-09-07（nether-save-full 课题 P2 矿石归因；judge PASS 建议 candidate）
- **置信度**：candidate（消融数据层证据，单 region 单 seed）
- **module**：workflow / cppReplace 架构
- **观察**：原生层接管宿主管线时，`wg_fill_blocks_multi` 内部自己实现了完整管线（noise+surface+carver+feature），而外层 mixin 只拦截 populateNoise + cancel buildSurface——未拦截的 Java CARVER/FEATURES 步骤照跑 → 同阶段双跑，矿石族计数 ~2.2×（quartz 4478 vs ref 1992）。
- **证据**：消融 WG_SKIP_FEATURES=1 → match +5508，quartz 4478→2125 / gold 1525→739 / magma 3814→1979，全部落回 ref 邻域（09 篇「矿石归因定论」）。
- **如何利用**：①「某块族计数 ≈2× ref」是双跑签名，≈1× 落回即证实；②「同 dll 存档基线 + env 逐阶段消融」两步定位，单步差值直接归因；③接管边界审计（谁拦哪些步骤 × 内层实现哪些阶段）先于条件链归因（与发现 #10 同族、方向互补）；④修复勿用进程全局 env 默认翻转，用句柄/调用级显式 flag。

## 发现 #12: 静态对拍必须对拍解析产物而非输入原文——「参数全对拍」假阴性掩盖真 bug（2026-09-09）

- **发现时间**：2026-09-09（soul-v4v5 课题 V3→V4；judge PASS 建议 candidate）；**module**：workflow / 验证方法 / 数据驱动解析。
- **观察**：V3 静态结构对拍把「解析器产物树 vs JSON」的对拍做成「肉眼核对 JSON 原文参数」——节点结构逐项一致、参数「全对拍」通过，结论「结构差不存在」。但**中间层（解析器）本身是嫌疑对象**：布尔字段被解析器读成 false，JSON 原文上是 true，「肉眼对拍 JSON」天然查不出——8 处假阴性中多处 JSON 原值恰为 false，进一步掩护。真 bug（as_f64 读布尔恒 false，见 compiler-idioms 发现 #8）被假阴性压制一轮（V3 draft）才在 V4 由解析产物树 dump 锁定。
- **证据**：修复前解析产物树 dump（soul-tree-repro）实测 8 处 `asd=false`，与 nether.json 原文行号逐项对拍——3 处 JSON=`true`（真阳性）、多处 JSON=false（假阴性掩护）；「JSON 原文 / 解析产物树 / 运行时行为」三方对拍闭合后单轮定位根因（`.artifacts/.b2-soul/v4-eval-conflict.md` §1 表）。
- **如何利用**：
  - **规则**：凡对拍「解析器/转换器/中间层」的正确性，对拍物必须是**parse 产物树 dump**，不是输入原文——原文对拍只能证「输入长什么样」，证不了「中间层把它变成了什么」。
  - **工具化**：把 parse 产物树 dump 固化为 bin-diag 常备诊断（本例 `soul_tree_repro`），带 JSON 行号列，逐节点对拍；「参数全对拍」类结论必须注明对拍对象是原文还是产物。
  - **三方纪律**：probe「复算一致」只证 probe 与生产同源，不证与 JSON 规范同源——JSON 规范 / 解析产物 / 运行时三方对拍缺一不可（本例三方各自自洽、互相矛盾，矛盾点即中间层 bug）。


