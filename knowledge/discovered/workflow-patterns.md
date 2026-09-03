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

- **发现时间**：260901-03；**发现者**：1.0.22 发版 session；**来源定位**：`WorldgenRust/src/bin/` 过时诊断 bin 阻塞发版事件 + AGENTS.md 八.13「临时文件唯一区纪律」；**置信度**：candidate（用户已拍板纪律并落地修复，本发现未走 judge）；**module**：workflow / 构建工程。
- **观察**：cargo 只自动编译 `src/bin/` 下所有文件（npm 侧 src 扫描、CMake glob 同族），该目录里积累的一次性诊断 bin 过时后无人维护，却仍参与每次全量构建。1.0.22 发版时 `cargo build --release` 因其中 23 个 bin 仍用旧 `RsSplitter` API（`Aquifer::new` 签名已换 `XoroshiroSplitter`）报 E0308 阻塞发版——而发版产物只需 cdylib（dll），这些 bin 根本不在发版产物内。致灾 = 两因素叠加：①「临时产物位置无约束」（写进明面约定目录）②「构建范围 = 全目录」（约定目录全量自动编译）。
- **证据**：E0308 全部指向 bin 内旧 API 调用；分两批迁出（先 5 个，重建后又暴露一批 → 共 23 个 → `WorldgenRust/src/bin-diag/`，cargo 不扫描该目录）后全量构建 exit 0，发版产物（cdylib）不受任何影响——证明这批 bin 与发版零依赖，纯属「位置无约束 × 全目录构建」的结构性问题。**第二轮才见底**也是证据：首次报错列表只是「先编译失败的那批」，cargo 不会一次报完所有坏 bin——清此类债必须迭代到全绿。
- **如何利用**：
  - **判据一**：任何构建系统的「约定目录 = 自动构建范围」（cargo `src/bin/`、npm 打包器 src 扫描、CMake `file(GLOB)`）都**不要放一次性脚本/诊断程序**——放进去即等于「永久参与构建 + 永久欠维护债」。
  - **判据二**：临时产物必须有唯一隔离区，**靠纪律不如靠机制**——最可靠的隔离是目录选择本身（放构建器不扫的目录），而不是「记得别乱放」；CoreSwap 落地：Python/一次性脚本 → 仓库根 `.tmp/`，Rust 诊断 bin → `WorldgenRust/src/bin-diag/`（要用时 rustc 单编或临时挪入）。`src/bin/` 只保留「随库维护、全量编译必须绿」的正式 bin。
  - **判据三**：发版只需库产物时 `cargo build --release --lib` 可救急，但**全量绿仍保留为发版前检查**——全量构建是过时产物的探测器，全量红即暴露「有产物已脱离维护」。
  - 反模式警示：诊断 bin 与生产代码共享 crate 时，API 演进不会「顺带」修好 bin（无 CI 强制时更不会）——过时是默认轨迹，隔离是唯一低成本对策。
- **同族已知坑**：gradle 换机器/新 shell 直接 `gradle` 不带 `GRADLE_USER_HOME` → native-platform.dll 加载失败（build-tooling.md 发现 #4）——环境类「约定不生效」同族：工具的 home/缓存必须显式指到工作区内。





## 发现 #10: cppReplace 存档口径残差的三阶段归因法——先分阶段再定位机制

- **发现时间**：260901-03；**发现者**：nether-save-full session（B1 定论轮）；**来源定位**：`.investigations/nether-save-full/`（residual-interpretation + .b1/.b2 + judge-review）；**置信度**：candidate（三方实验数据实锤，模式层面复用性待下一案例）；**module**：workflow / Minecraft modding / 验证方法。
- **观察**：替换模式（如 cppReplace 只接管 noise+surface，Java carvers/features 仍在替换后地形上运行）下的存档口径残差是**多阶段混合产物**——直接把残差对到单一层（如 surface rule 条件链）会得出错误归因（错误 E6：B1 52k 块大宗互换一度被归因 surface rule 条件链，实为 feature 阶段产物在两种基底地形上的命中/形态差）。正确做法是**三阶段归因**：
  1. **阶段分解**：先分清 noise/surface（替换方 = Rust）与 carvers/features（存续方 = Java）各自贡献，再定位机制；
  2. **消融判别**：`WG_SKIP_SURFACE=1` 重跑——surface 关掉后残差从 93.55% 掉到 55.18% 证明 surface 是实心块主来源；且 blobs 不触发（stone 基底非 netherrack → blackstone=0）反向证明 blobs 是 feature 阶段、依赖 netherrack 基底；
  3. **纯替换方基线**：ctypes 直连 dll（或 rlib 直跑）取得纯 Rust 输出 vs 参照（本轮 77.43%，与存档口径 93.55% 载体不同不可比，§9.7）——分离「替换方自身缺口」与「存续阶段叠加产物」。
- **如何利用**：
  - **判别手段一（消融）**：单阶段开关（如 WG_SKIP_SURFACE）A/B 重跑——残差量级变化 + 依赖块（如 blackstone 依赖 netherrack 基底）是否消失，直接指认产物所属阶段；
  - **判别手段二（直连基线）**：ctypes/FFI 直连替换方库跑同区域——排除 Java 存续阶段干扰，取得替换方独立口径；与 rlib 直跑对拍可顺带验证 FFI 层确定性（本轮 cell 级 0 差异）；
  - **biome/来源分桶**：残差按 biome 列分桶——若差异 100% 落在 vanilla 某列（本轮 basalt_deltas），排除「源分配差」，收窄到「同源产物在不同基底上的表现差」；
  - **同 dll 重跑非确定性容差判据**：存续阶段（Java feature 邻块写入调度）本身非确定——同 dll 两次完整 run 相差 369 块（93.5156%→93.5508%）。**存档口径对齐指标必须声明该容差**：同 dll 重跑块级差 ≤ 百分级（千分位级百分比波动）属调度噪声，不构成实现回归判据；跨口径比较（探针 vs 存档 vs 纯 Rust）一律 §9.7 三要素声明。
  - **实例更新（260902-04）**：basalt per-id −1736（260902-03 run2）vs −1050（本轮 run）——量级一致、数值漂移 ~686 块，落在非确定带宽内，再次佐证本判据：per-id 单值不做回归判据，只看量级/家族归属（B1 家族），跨 run 比较须声明容差。
  - **先消融后归因**：任何「残差 → 某层机制」的归因结论，出手前必须已有至少一个阶段消融或直连基线证据，否则降为 draft（反模式见 E6）。
- **同族模式**：发现 #4（参照数据状态三查 SURFACE vs FULL——参照阶段决定差异构成）是「对照侧」的前置；本发现是「被测侧」（替换模式运行时残差）的阶段归因，两者合用构成替换模式验证的完整口径纪律。

## 发现 #11: 嵌套接管管线复查——「内层全管线 vs 外层分步拦截」的双跑风险（260902-01）

- **发现时间**：260902-01（nether-save-full 课题 P2 矿石归因；judge PASS 建议 candidate）
- **置信度**：candidate（消融数据层证据，单 region 单 seed）
- **module**：workflow / cppReplace 架构
- **观察**：原生层接管宿主管线时，`wg_fill_blocks_multi` 内部自己实现了完整管线（noise+surface+carver+feature），而外层 mixin 只拦截 populateNoise + cancel buildSurface——未拦截的 Java CARVER/FEATURES 步骤照跑 → 同阶段双跑，矿石族计数 ~2.2×（quartz 4478 vs ref 1992）。
- **证据**：消融 WG_SKIP_FEATURES=1 → match +5508，quartz 4478→2125 / gold 1525→739 / magma 3814→1979，全部落回 ref 邻域（09 篇「矿石归因定论」）。
- **如何利用**：①「某块族计数 ≈2× ref」是双跑签名，≈1× 落回即证实；②「同 dll 存档基线 + env 逐阶段消融」两步定位，单步差值直接归因；③接管边界审计（谁拦哪些步骤 × 内层实现哪些阶段）先于条件链归因（与发现 #10 同族、方向互补）；④修复勿用进程全局 env 默认翻转，用句柄/调用级显式 flag。

## 发现 #12: 静态对拍必须对拍解析产物而非输入原文——「参数全对拍」假阴性掩盖真 bug（260902-03）

- **发现时间**：260902-03（soul-v4v5 课题 V3→V4；judge PASS 建议 candidate）；**module**：workflow / 验证方法 / 数据驱动解析。
- **观察**：V3 静态结构对拍把「解析器产物树 vs JSON」的对拍做成「肉眼核对 JSON 原文参数」——节点结构逐项一致、参数「全对拍」通过，结论「结构差不存在」。但**中间层（解析器）本身是嫌疑对象**：布尔字段被解析器读成 false，JSON 原文上是 true，「肉眼对拍 JSON」天然查不出——8 处假阴性中多处 JSON 原值恰为 false，进一步掩护。真 bug（as_f64 读布尔恒 false，见 compiler-idioms 发现 #8）被假阴性压制一轮（V3 draft）才在 V4 由解析产物树 dump 锁定。
- **证据**：修复前解析产物树 dump（soul-tree-repro）实测 8 处 `asd=false`，与 nether.json 原文行号逐项对拍——3 处 JSON=`true`（真阳性）、多处 JSON=false（假阴性掩护）；「JSON 原文 / 解析产物树 / 运行时行为」三方对拍闭合后单轮定位根因（`.artifacts/.b2-soul/v4-eval-conflict.md` §1 表）。
- **如何利用**：
  - **规则**：凡对拍「解析器/转换器/中间层」的正确性，对拍物必须是**parse 产物树 dump**，不是输入原文——原文对拍只能证「输入长什么样」，证不了「中间层把它变成了什么」。
  - **工具化**：把 parse 产物树 dump 固化为 bin-diag 常备诊断（本例 `soul_tree_repro`），带 JSON 行号列，逐节点对拍；「参数全对拍」类结论必须注明对拍对象是原文还是产物。
  - **三方纪律**：probe「复算一致」只证 probe 与生产同源，不证与 JSON 规范同源——JSON 规范 / 解析产物 / 运行时三方对拍缺一不可（本例三方各自自洽、互相矛盾，矛盾点即中间层 bug）。

## 发现 #13: 探针坐标 bug 制造 100% 单向假象——探针输出必须先做 sanity check（260902-04）

- **发现时间**：260902-04（V5 残差排查；实际 2026-09-02 15:45 = 本轮提交簇 git 时间戳锚）；**发现者**：v5-residual session；**来源定位**：`.investigations/v5-residual/finding-mid-260902-04.md`；**置信度**：candidate（数据层证据实锁，judge 有条件 PASS）；**module**：workflow / 验证方法 / 探针工程。

### 现象

nether 存档残差 35426 mismatch 列，world biome 对比显示 **100% = warped_forest**（vanilla 参照侧 94.4% basalt_deltas + 5.6% soul_sand_valley）——「残差区 biome 全被误判成 warped_forest」的单向假象，指向 biome 分类器/存储填充链（fan-out 候选 .b6）。

### 根因

**探针自身坐标 bug**：ReadWorldProbe 的 wBiome 查询误用 **chunk 局部 x,z（0-15）**调 `world.getBiome` → 实际查询落在 chunk(0,0) 区域（恰好是 warped_forest）→ 所有行读到同一个错误 biome；vanilla 参照侧用的是世界坐标，正确。两侧坐标系语义不一致 + 错误区恰好恒定，制造了「100% 单向翻转」的假签名——符号级错误（坐标域错用）是结构错，不是精度/概率错。

### 定位

一步裁决探针逐层收敛（见下简记）：biome 列对比发现 100% warped → 6 维输入对拍（Java BIOME6 vs Rust biome6_dump 逐位一致，两侧数学均判 basalt_deltas，dist 0.119 vs 1.080 非平局）→ storage cell dump（Rust 独立分类器判定与 vanilla 参照一致）→ **整列 storage vs biomeAccess 对比**锁定：读 storage 正确、走 BiomeAccess 的 wBiome 读错误 → 反查 wBiome 坐标来源发现局部/世界坐标混用。

### 修复

wBiome 改用世界坐标查询（mismatch-nether-run6.csv）：96.3% 残差列 biome 完全一致（basalt→basalt 32817 + ssv→ssv 1306）；biome 真差仅 1303 列（3.7%，ssv↔basalt 边界互换 = 签名 A 降级为次要项）。残差真图景变为「**同 biome（basalt_deltas）下表面规则判定差**」（B1 家族本体）。

### 教训

- **「对比前先核坐标语义」铁律（AGENTS.md seed 三查 #2）的延伸**：不仅跨工具对比要核坐标语义，**探针自身的每个坐标参数都要核**——探针 bug 比实现 bug 更危险，因为它披着「测量仪器可信」的外衣。
- **探针输出先做 sanity check 再下结论**：对照组/已知区抽查（如「100% 单一值」「全部落在同一区域」这类完美签名本身就是可疑信号——真实世界残差不会是 100% 单向）；100% 一致性签名优先怀疑测量侧而非被测侧。
- 再次验证「符号级错误是结构错」：35426 列 100% warped 不是概率/精度问题，一眼就应归因结构性坐标/索引错。
- 同族：workflow-patterns 发现 #12（对拍对象错级——对拍原文而非解析产物）——两者都是「测量/对拍工具自身失真污染结论」家族。

### 简记（同轮方法再验证，中价值）：one-step decisive probe 逐层收敛

V5 残差排查中 4 次「一步裁决探针」逐层收敛（biome 列对比 → 6 维输入对拍 → storage cell dump → 整列 storage vs biomeAccess），每步排除一批候选；fan-out worker 静态代码分析与主会话探针数据采集交替（worker 出模板 → 主会话执行不解读 → 原始输出回传）——与发现 #10 三阶段归因法互补：#10 是宏观阶段分解，本条是微观单层内的最小裁决步设计。复用判据：每一步探针设计成「输出只有两种互斥解释」的裁决形态，避免采集后再发散分析。

### 补充案例（260902-05/06，测量侧三犯清单 + 同族新案例）

同 session 链再证「探针输出先 sanity check」：260902-05「selector 场差 42%/patch 差 64%」整轮假象 = NoiseConfig 维度取错（overworld vs nether）；同轮 BlockProbe FULL 预生成提升 chunk 污染 SURFACE 参照（22.5% 残差假结论，260901-04 结论被取代）；260902-06 CSV 空 4 轮 = RegistryKey 过滤条件恒 false。合并沉淀**测量侧先查三犯清单**（探针零输出/异常完美签名时，先查测量侧再怀疑被测侧）：
1. **wBiome 坐标**：chunk 局部 x,z（0-15）vs 世界坐标混用（#13 原案例）；
2. **NoiseConfig/维度上下文**：探针取的 server 维度对象 ≠ 目标维度（overworld XOROSHIRO vs nether LEGACY）；
3. **pregen 提升 chunk**：预生成邻域导致 getChunk 返回已推进状态，参照口径名不符实（同族：发现 #6 getChunk「至少 N」语义）。

**同族新案例（260902-06）**：`RegistryKey.getValue().toString()` 返回带命名空间全名（`minecraft:the_nether`），与裸路径（`the_nether`）equals 过滤恒 false → CSV 空 4 轮的真根因。同判据扩展：**探针零输出先查两类条件**——①过滤条件（字符串/枚举语义不匹配恒 false）②驱动条件（如 BlobProbe 单独跑无 driver 不生成 nether chunk，COLPROF 空跑一轮——「没数据」先问「chunk 根本没被生成」再问「生成了没读到」）。




### 补充案例（260902-07，指标盲区 + 行首锚假零输出）

同判据家族两例（b1-downdrill lavaAudit 课题，详见 b1-errors.md E-B1-10/11）：

1. **探针指标盲区**：lavaAudit v1 只记 above=lava 转换，恰好测不到判别「熔岩海缺失」所需的 below=lava 面向（air→lava 转换面）——v1 输出「99.4% 一致」对核心命题零判别力，是「测了且一致」的假安心；v2 加记 lavaSurfY 后一轮实锤两侧 air→lava 面向均为零，直接推翻待测现象的存在。**判据扩展**：设计探针指标先写「要判别命题 P，最小充分证据是什么」再检查指标覆盖，指标名对口 ≠ 覆盖证明（与发现 #12 对拍对象错级同族：测量设计与判别目标脱节）。
2. **行首锚 grep 假零输出**：`^\[LAVAAUDIT\]` 对带日志框架前缀（时间戳/线程名）的 log 行恒零命中 → 误判「探针零输出」。与 260902-06 RegistryKey 命名空间前缀（本发现 #13 补充案例）同属「过滤条件把全部行静默滤掉」家族。**判据扩展**：「零命中」先打印一行原文核对行格式，`^` 行首锚对 log 行默认不可用。

## 发现 #14: 探针阶段同源性——stageMask 只控本侧阶段，「noise-only」要先验证对侧是否真静默（260902-08）

- **时间/置信度/module**：260902-08/09，candidate，re-code/swe 通用。
- **现象**：以 -Dcoreswap.rust.stages=7 控制为「noise-only」采对照存档，存档中仍含蘑菇/basalt delta/carver 产物（air pocket 材质对比被污染）。
- **根因**：该属性只控 Rust 内部阶段（bit0/1/2=CARVER/FEATURES/SURFACE，CppBridge.java L63-71 默认 0b011）；cppReplace 下 Java 侧 CARVERS/FEATURES 照跑——stageMask 是「本侧开关」不是「管线状态」。
- **定位**：直接看存档内容 vs stageMask 日志——日志与存档不符即暴露。
- **修复**：对照口径改为按存档实际内容分阶段归因（与发现 #10 阶段归因法合用）；纯阶段对照走替换方独立通道（bin-diag 直连 fill_chunk_blocks + env 全 skip）。
- **教训/如何利用**：任何「只跑某阶段」的对照实验，判定依据必须是产物内容而非开关日志；「对侧阶段静默」须独立验证（对照侧=发现 #4，本条=被测侧开关语义，合用）。

### 补充案例（260902-09，#12 家族：假阴性陷阱 → 假 100% 一致）

对拍脚本自身失真产出「假 100% 一致」两例（本 session 实证，均发生在同一对拍链）：
1. **空切片假一致**：对 128 项 vanilla 序列施加 `[128:]` 切片（本意为切 Rust 侧 256 项）→ 空序列 → `zip` 空 → 0 差异被读成「完全一致 100%」；
2. **切分散假一致**：`mat=` 解析用 `split(',')[3][4:]`，逗号切散后只取到首个数字 → 单元素序列 → 同样假 100%。

**防范（判据）**：对拍脚本强制打印 sanity 行（两侧序列长度 + common 列数），`common=0` 或长度不符即拒绝出结论——「完美一致」与「完美失败」一样要先怀疑测量侧（与 #12 对拍对象错级、#13 sanity check 同族）。



## 发现 #15: 零面擦边格签名判别法——微差残差先看 |d| 量级，单侧普查只能封闭不能定量（260902-13）

- **时间/置信度/module**：260902-12/13，candidate（判据部分经 B1 confirmed 定案背书；「量级→机制类」映射在二分判据之间为推断），re-code/swe 通用。
- **来源定位**：B1 NOISE 微差下钻（noise-drill-verdict-260902-11.md，confirmed）；判别手段 = WorldgenRust/src/bin-diag/b1_density_probe.rs（逐点 d_exact）+ b1_grazing_census.rs（全区 |d| 普查）。
- **观察**：端到端残余「孤立单格微差」类残差（如 air 孔、单格翻转）不必逐位复刻浮点求值序即可机制分类——**签名三分**：
  1. **|d| ≤ ~1e-5 + 孤立单格翻转 + 方向不系统** → FP 求值序微差类（非结构错误）——B1 实证：13 格全落擦边带，方向 10 rust-air / 3 vanilla-air 不系统；普查擦边集仅 0.016%，与端到端残差同量级即封闭。
  2. **|d| ~ 0.1** → 角点值错（结构性，修公式/索引/坐标）。
  3. **二分判据之间**（B1 实测 1e-6..2e-5）→ 量级归因为**推断**：配对实测前不可定言「仅需 ~1e-6 级重结合差」（最大翻转格 |d|=2.27e-5 要求 Δd≥2.27e-5）。
- **证据**：① 13 差异格 d_exact 全 ≤2.27e-5 且符号/air 归属 13/13 自洽；② 全区 524,288 点普查擦边集 83 格 ⊇ 12/13 差异格（封闭验证）；③ judge C1——全部量化证据为 Rust 单侧，Java 配对 Δd 未实测，「~1e-6 级」是推断；④ 判据显著收窄排查面：B1 由「四候选 fan-out」收敛到「零面擦边」单机制。
- **如何利用**：
  - 遇「微差残差」第一动作 = 逐点 d_exact 探针看 |d| 量级（先分类再决定下钻深度——擦边类直接考虑封顶结案，收窄需逐位复刻求值序，成本高、跨平台脆弱，通常不值）。
  - **封闭验证**：全区 |d| 普查确认差异格 ⊆ 擦边集 + 擦边集占比与端到端残差同量级——排除「还有别的机制」。
  - **边界铁律**：单侧普查只证「擦边+自洽」，两侧差 Δd 定量 MUST 配对采样；普查阈值是后验统计口径，不作判据引用。
  - **同族坑**：census histogram 打印标签偏移（复用先修标签，计数以变量为准）；scout 小样例推断（5 样例「等差贯穿」）消费前必须全量复核。

## 发现 #16: 对照基线归因法——异常差异先跑普通坐标对照，分离坐标因素与实现因素（260902-14）

- **时间/置信度/module**：260902-14，candidate（经 judge 通过的封存定案背书；混杂局限已声明），re-code/swe 通用。
- **来源定位**：极端坐标 FP 微差应力测试（verdict-260902.md，用户拍板封存）；手段 = 普通 chunk 200,200 对照区（同 seed/同载体/同 4×4 口径，唯一变量 = 坐标）。
- **观察**：面对「某条件（如极端坐标）下的差异」命题，先问「该差异在普通条件下是否存在」——对照区一致率 98.5914% **低于全部极限区**（98.85–99.85%），且泥土带失配在对照区同样出现且更大（top 簇 17,754 vs 12k-18k）→ 直接归因：失配主体是既有实现系统差，非坐标极端化引起；极限坐标反而「稀释」了带差占比。
- **证据**：① 五区一致率与簇统计（derive_stats.out.txt，可复现）；② 排除泥土带后极限区只剩 B1 同族 FP 擦边散簇（466 个，最大 522），量级不随坐标爆炸；③ judge 归因项 PASS（同 seed/同载体/唯一变量=坐标的对照设计质量核可）。
- **如何利用**：
  - 「X 条件下有差异」第一动作 = 造一个无 X 的对照（普通坐标/普通参数/同口径），一次运行即可把「X 引起」与「本来就有的系统差」分离——避免在 X 侧深钻不存在的机制。
  - **反直觉判读**：对照一致率比异常区**更低**不是对照失败，恰是「差异与 X 无关」的强证据（异常区占比被稀释）。
  - **边界铁律（judge 声明的局限）**：单对照只能分离「一个变量」；若对照区与异常区天然伴随其他混杂因素（本例坐标↔biome 组成不同，泥土带是否 biome 驱动无法由本矩阵分离），归因措辞只能到「非 X 引起」，不能跳到「已归因 Y」——要归因 Y 须先做 Y 维 fan-out。
  - **同族**：与发现 #10 三阶段归因、#14 阶段同源性同属「对照/归因设计」家族——本条补「空间/条件维度对照」，#10/#14 是「管线阶段维度对照」。

## 发现 #17: 跨探针对比坐标钉死律——打印坐标≠采样坐标时结论无效，oracle 逐点复核识别混列（260903-06）

- **时间/置信度/module**：260903-05→06，candidate，re-code/swe 通用。
- **来源定位**：lossless-accel P-A ch0 通道级闭合（pa-ch0-closure-260903-06.md）；手段 = density_probe -dfDump C++ CPU oracle + ch0_compare.py 统一坐标重建对拍。
- **观察**：三方 ch0 对拍「macro vs GPU 残差 0.03-0.23」实为 GPU @ (4,80,z=0) 与 macro @ (4,80,z=16) 混列——两点各自与 C++ oracle 精确吻合（≤5e-7/1.8e-6），残差不存在。构建 oracle 后逐点复核：每个"差异值"都能在 oracle 中找到**另一个坐标**的精确对应 → 混列暴露。
- **判据**：① 跨探针数值比对第一动作 = 断言两侧坐标序列逐点一致（打印不算、要断言）；② 「两实现互不相等、差值随点漂移」与坐标错位签名同构，先查对位（seed/坐标三查同族）；③ 对照独立 oracle 逐点复核是识别混列的廉价手段（一轮脚本成本）。
- **同族**：#12 对拍对象错级、#13 sanity check、#15 配对采样边界——本条补「坐标维度对位」。**补充要点（P-B，260903-06）：判别探针前置——静态归因先做单点隔离复测**：bA 的闭包压平归因源自静态括号配平+生成物结构审查，未先做「绕过外层环境直接单点调用生成函数」的隔离复测；单点隔离探针一轮即可证伪结构层归因（本例生成函数全对，错在构造环境）。静态归因（配平/结构审查）只能出候选，候选消费前 MUST 单点隔离复测。

## 发现 #18: 跨 session 基准数字不可直接续推——引用历史基准先核对 stage 覆盖/口径标注，缺标注的历史数字视为不可比（260903-08）

- **时间/置信度/module**：260903-08，candidate（judge 建议通过），re-code/swe 通用。
- **来源定位**：lossless-accel P-C1 端到端三方对比（pc1-e2e-260903-08.md）；判别证据 = fair_comparison_corrected.txt:3-4 自注（08-29 Rust 侧为「无树花」口径）。
- **观察**：08-29 结论「Rust 45.48 < Java 55，反快 1.2×」被后续 session 引用为「Rust 比 Java 快」的默认前提；同日同机同 region 重测（Rust 含 features 完整管线）实为 Rust 72-77ms vs Java 33ms——**慢 2.2×**，方向完全反转。根因不是哪次测错了，而是两次 Rust 侧 stage 覆盖不同（无树花 vs 全管线），历史数字无口径标注或标注未随结论流转，续推时口径失配。
- **判据**：① 引用历史基准结论前第一动作 = 核对 stage 覆盖/口径标注与当前对比口径是否一致（§9.7「与既有口径可比性」的历史方向应用）；② **缺标注的历史数字一律视为不可比**，不得作为方向性前提续推（交接结论验证纪律的基准数字特例：假设不当公理，历史数字不当基线）；③ 采新基准时把口径自注写进产物文件本身——标注随数据走，不随会话记忆走（本例 fair_comparison_corrected.txt 自注救了归因）。
- **同族**：§9.7 三要素、#14（阶段同源性）、#17（坐标维度对位）——本条补「历史结论跨 session 续推维度」。连锁开问题：Q-PD1（Rust features/carver 段 vs Java 差距大头，独立排查）。（📌 260903-09 更新：Q-PD1 已闭合，方向假设被 supersedes——大头实为 aquifer 段，见 .artifacts/lossless-accel/qpd1-attribution-260903-09.md。）

## 发现 #19: Java bench 前必须删 run\world——世界状态第四查；「快一个量级 + min≈0」是缓存假象签名（260903-09）

- **时间/置信度/module**：260903-09，candidate，swe/re-code 测量通用。
- **来源定位**：`.investigations/lossless-accel/cmd-output/qpd1-java-recheck-260903-09.md`（run A vs run B）；`.investigations/lossless-accel/q-pd1-260903-09.md`。
- **观察（现象）**：Java WorldGenBench 在 `run\world` 残留时跑 region(200,200)：`total=764ms avg=2.98ms min=0`；删 `run\world` 后 fresh 生成：`total=10993ms median≈32ms`——第一次测出「快 ~60×」的假象级数字。
- **根因**：world 残留时 bench 走服务器 chunk 系统，region 内 chunk 已生成 → 从磁盘加载（~1ms/chunk）而非走 worldgen 管线，测的是 IO 缓存不是生成性能。`benchSeed` 只改 bench 参数，不改 level.dat 里的世界状态（与「benchSeed 不改变世界 seed」同源）。
- **定位**：run A 输出 min=0 + total 偏离前日基线一个量级 → 触发复核，fresh 重跑即闭合。
- **修复**：bench 前 `Stop-Process java` + `Remove-Item run\world` 强制 fresh 生成。
- **教训/如何利用**：**采集核对升级为四查**（seed/坐标/文件 + **世界状态**）——任何依赖 `run\world` 的 Java 侧测量，数据用于对比前必须确认 world 目录状态。假象签名：**同 region 二次 bench 快一个量级 + min≈0** → 立即怀疑缓存/残留，不作结论。同族：AGENTS.md「残留 java 进程占 world/端口」（同一 run\world 问题的性能测量面）。

## 发现 #20: 死参数制造假判别——判别实验必须验证「自变量真被改变」（260903-09）

- **时间/置信度/module**：260903-09，candidate，通用方法论。
- **来源定位**：pc_e2e_bench.rs L18 解析 `WG_E2E_SEED` 后 L22 恒用常量 `SEED`；被污染结论 = pc-results-260903-08.md「negseed 判别 <3% → seed 非因素」（已 supersedes）。
- **观察（现象）**：negseed 判别差 <3%，读作「seed 非因素」记入 confirmed artifact；次日本 session 基线复核顺带源码审查发现 env 解析后根本未接到使用点——两次运行同 seed，「差 <3%」是同变量重复测量的必然结果，不是判别结果。
- **根因**：死参数（解析与使用点脱节/常量遮蔽）——实验在机制上未改变自变量；「两结果相近」被误读为「自变量无关」。失效静默：无告警、输出格式完全正常。
- **定位**：交接纪律的「廉价独立验证」顺带 bin 源码审查发现；若运行时打印实际生效 seed 并断言处理组≠对照组也能当轮发现。
- **教训/如何利用**：**判别实验前置自检——先验证「X 真被改变」再读结果**（打印生效值断言两侧不同，或给 X 一个已知强效应值做 sanity）。近零结果（如「差 <3%」）同时兼容「真无影响」与「假判别」，须用恒等式自检排除后者再下结论。同族：#14（stageMask 静默）、#18（口径核对）——本条补「自变量生效性」维度。

## 发现 #21: 微测基线样本模式决定缓存命中率——单点微测外推热路径成本差 40×
- **时间/置信度/module**：260903-10，confirmed（260903-10 用户拍板），通用方法论。
- **来源定位**：Q-AQ1 归因（qaq1-attribution-260903-10.md）；错误基线 = qaq1-initdensity_cost 探针「随机 y + 交替列」实测 initial_density 0.089µs/sample（F5），真实 est 扫描形态 3557ns/sample（qaq1_b1_coldpath A 段）——差 40×，整套「est 只值 0.66ms」推论作废。
- **观察（现象）**：同一 DensityFunction 树、同 seed 同 region，两种采样模式测出 40× 单价差；错误基线与冷态实测 26.65ms 直接矛盾（0.65ms vs 26.65ms），矛盾被正确用作怀疑信号。
- **根因**：微测的遍历方向/列切换模式/缓存冷热/生命周期与热路径不同构——「随机」模式命中 Cache2D/y 无关缓存，est 扫描每 chunk 换列全冷、且 base_3d_noise（old_blended，24 octave）无任何缓存全价执行。
- **定位**：量级矛盾（核算 vs 实测）触发怀疑 → 决定性探针（qaq1_b1_coldpath）按热路径同构形态重测。
- **教训/如何利用**：**单点微测外推热路径成本必须复刻调用形态**（遍历方向/列切换/冷热/生命周期）；「量级核算与实测差 10×+」第一怀疑形态失配而非测量噪声。同族：#17（坐标钉死）、#20（自变量生效）——本条补「采样形态同构」维度。
- **补充案例（260903-11，working set 维度）**：est-opt 包微测复刻了调用形态（est 逐列冷扫描）仍得出 2117ns/iter 上界，生产冷路径实际单价 ≈11µs/iter（差 ~5×）——签名：**e2e 实测收益（−48ms/chunk）超微测上界（15.5ms）**。差异不在调用形态而在 **working set**：生产树共享 Arc + 大缓存集（Cache2D/邻居表/共享 sampler），独立树微测的缓存足迹远小于生产；微测虽「形态同构」但「内存环境不同构」。教训补全：微测外推需复刻**调用形态 + working set（共享结构/缓存集/生命周期）**两者；「e2e 收益 > 微测上界」是 working set 失配的强签名，以生产实测为准、不反推机制定论。

## 发现 #22: 自由参数凑数反模式——量级核算的乘数必须有独立实测来源
- **时间/置信度/module**：260903-10，confirmed（260903-10 用户拍板），通用方法论。
- **来源定位**：qaq1-b1-candidate-260903-10.md §4.2「158 miss × ~4-5 次重建 × 1225 角点 × ~30ns ≈ 26ms」——三处乘数无实测来源，GRID_ARG_SAMPLES=0 反证后整套机制作废（§5 supersedes）。
- **观察（现象）**：凑出的乘积与目标缺口「异常贴合」（26 vs 26.65，残差 <3%）——但机制根本不存在。
- **根因**：把「缺口量级」当约束反推乘数（拟合目标而非解释世界）；每乘数缺独立实测锚点。
- **定位**：决定性计数器（GRID_ARG_SAMPLES 双态增量 0）一击否证。
- **教训/如何利用**：**量级核算每个乘数须有独立实测来源；≥2 个自由参数只算猜想**；「异常贴合」反而是凑数嫌疑信号——机制预测应在自由度小的前提下先出预测后对照。

## 发现 #23: 诊断证据摘要漏行——漏一行制造「N× 缺口」假象
- **时间/置信度/module**：260903-10，confirmed（260903-10 用户拍板），通用方法论。
- **来源定位**：Q-AQ1 证据包 F4 摘要漏收 get_fluid_level(全调用上界): t_fl 一行（qaq1_apply_breakdown 本来就打印了）→ 得出「diag 只解释 ~6ms、生产贵 6×」的 G1 缺口假象；实为部分漏项 + 部分真冷态成本。
- **观察（现象）**：分解探针输出 5 行，摘要只录 4 行；被漏行恰是后续归因主路径。
- **根因**：摘要按「当时判断的相关性」归组删行，判断错了就制造假缺口。
- **定位**：b1 worker 复读原始输出发现漏行。
- **教训/如何利用**：**分解数据的每一行都进证据包**（摘要可归组不可删行）；接手「A 比 B 贵 N×」先回溯原始输出查未入包行。

## 发现 #24: 多臂顺序 bench 顺序效应制造假交互——差分必须 chunk 粒度交错
- **时间/置信度/module**：260903-10，confirmed（260903-10 用户拍板），通用方法论。
- **来源定位**：qaq1-b2-candidate-260903-10.md；v1 顺序执行 aquifer×carver 2×2 四臂，round2 全臂 +8~13% 漂移下出现物理不可能的负交互（C|Aoff 22.97ms > C|Aon 10.62ms——carver-on-Air 每点工作应严格更少）；v2 chunk 粒度交错 ×3 轮 ×median 稳定 ±0.5ms。
- **观察（现象）**：同进程顺序跑多配置，机器漂移/缓存状态系统性偏向后跑的臂；差值中出现违反物理约束的读数。
- **根因**：顺序执行的臂序与时间相关漂移卷积；「负交互」= 漂移伪影。
- **定位**：物理约束违反签名（工作量少的配置反而更贵）→ 怀疑测量设计而非机制。
- **教训/如何利用**：**≥2 臂差分 bench 必须 chunk 粒度交错 + 多轮 + median**；「物理不可能读数」是测量设计 bug 的强签名，先修测量再谈机制。同族：#19（世界状态）、AGENTS 测量污染铁律——本条补「臂序漂移」维度。

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
  - **根因（机制）**：凭常见值记忆填写常量——`verticalCellBlockCount = 4 × size_vertical`（GenerationShapeConfig.java:46-48），overworld.json `size_vertical: 2` → **实际步长 8**；「4」是 size_vertical=1 的常见值，不是 overworld 值。
  - **定位**：裁决沿取值链逐环核对：调用点（ChunkNoiseSampler.java:233）→ 计算式（GenerationShapeConfig.java:46-48）→ 数据源头（overworld.json:18）。三环核对后疑点解除（一致，P1 文档「4 步进」为笔误）。
  - **修复**：P1 文档笔误不改原文，由 K3 裁决记录取代标注。
  - **教训**：**引用 Java 常量必须追到取值源头（config/JSON 派生链），禁止凭记忆/常见值填写**；数据驱动的 MC 常量尤其如此——派生链上任何一环（计算式 × JSON 值）换版本即变，「常见值」是最不可靠的引用方式。
- **同族**：#9（跨 session 未验证标注当公理继承）、#17（打印坐标≠采样坐标）——本条补「调研产出消费前」维度：#9 管跨 session 交接，本条管同 session 内调研 subagent → 主会话的交接，判据同为「引用先验证再消费」。
