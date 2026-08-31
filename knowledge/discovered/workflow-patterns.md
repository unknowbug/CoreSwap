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

## INDEX.md 应用后该行变为

| 工作流模式 | [discovered/workflow-patterns.md](discovered/workflow-patterns.md) | judge 审查门强制触发点、scout 勘探前置、fan-out 多假设分叉强制触发、块级真相验证法、参照状态三查、FEATURE 独立于地形、getChunk 阶段语义（2026-08-09 更新）、接管单阶段后的后续阶段上下文依赖（2026-08-31） |

---


