# Knowledge INDEX — 知识库总入口（分析前先查）

> 双轨结构（2026-08-08 确立）：
> - **项目特定知识**（MC 1.20.1 实现结论/对齐状态/排查过程）→ `versions/1.20.1/docs/`（01-09 主题篇 + 10 时间线），管理纪律见 AGENTS.md 三。
> - **通用可复用模式**（跨版本/跨项目：语言惯用法、工具坑、算法指纹）→ 本目录 `knowledge/`，由 core.knowledge skill 管理。

## 错误台账载体（CoreSwap 项目级指定，2026-08-21 对齐框架 P3）

- **载体优先级**：项目级指定载体 > 框架默认 `knowledge/discovered/errors/error-<NNN>-<slug>.md`（后者为回退，未指定时用）。
- **CoreSwap 指定错误台账载体** = **`.investigations/<课题>/<课题>-errors.md`** 独立成篇（不用框架默认 `knowledge/discovered/errors/`）——每个课题一个错误台账文件（如 `rust-density-builder/rust-errors.md`、`perf-rework/gpu-accel-errors.md`），末尾附「错误→根因」速查表，五段式（现象/根因/定位/修复/教训）。
- **为什么**：CoreSwap 错误按课题归口到 `.investigations/`（探查/排查过程就在该课题目录），框架默认 `knowledge/discovered/errors/` 在 CoreSwap 是死目录（从未使用）；按框架 P3 声明"项目级指定 > 默认，两条不并行"，此声明即消除"默认路径成死规则"。
- 错误台账条目的高价值记录（判错经验/签名）仍按框架「错误 > 正确」优先级；**低价值结论不写知识库**（见 AGENTS.md §三.2 记录价值门）。

## 分类入口

| 分类 | 文件 | 说明 |
|------|------|------|
| 语言/编译器惯用法 | [discovered/compiler-idioms.md](discovered/compiler-idioms.md) | Java/MC 代码生成模式、浮点/整数语义、插值公式、MSVC/Windows·JVM 平台坑、跨层 id 域错位 raw block id vs state id（2026-09-01）、锚坐标换算 off-by-one below_top/above_bottom（260901-02）、JSON 布尔字段经 as_f64 读取恒 false——分型标量 API 静默语义腐蚀（发现 #8，260902-03）、跨 session raw id 标注三查——未验证标注当公理继承整链作废（发现 #9，260902-07）、Rust 半开区间 rev().step_by() 复刻 Java 含两端递减 for 的 off-by-one——首点值+下界包含性两独立参数（发现 #10，260903-13） |
| 还原工具误译及修正 | [discovered/f5-bugs.md](discovered/f5-bugs.md) | javap/反编译不可信点、反射缓存污染、修正方法 |
| 构建/工具链坑 | [discovered/build-tooling.md](discovered/build-tooling.md) | gradle daemon/env/参数解析、task UP-TO-DATE 跳过、文件同步、fs::copy 保留 mtime——产物判新旧用内容指纹（发现 #6，260902-01）、GRADLE_USER_HOME 全套状态指工作区绕开 home 权限 + 参照文件名四要素核对（发现 #7，260902-02）、gradle -P→-D 手工映射清单遗漏静默不生效（发现 #8，260902-04）、gradle runServer --nogui 非 CLI 选项 + rustStages 缺映射行静默不生效（发现 #9，260902-09）、参照文件核对四要素不够——文件名不含 stage，SURFACE 参照被当 FULL 用贯穿多轮；判据升级五要素（seed/size/origin/dim/stage 内容指纹：阶段特征 id 有无）（发现 #10，260902-10）、header/文件名本身也可能是错的——chunk 配对以内容实测坐标为准 + 探针恒等式自检（match 差≠分解计数差即假配对）（发现 #11，260903-02）、spv/comp 多产物部分更新——生成器多产物重生成必须整体原子更新，逐位一致哨兵结论须配已知值哨兵点验产物健康（发现 #12，260903-04）、GRADLE_USER_HOME 未指工作区致 native-platform.dll 加载失败——#7 第三表现面（发现 #13，260903-12）、runServer 大 region 预生成触发 watchdog 60s 强杀 + 探针 dump 必须内嵌 seed 头（发现 #14，260903-12） + gradle run 存档口径照抄历史 run 完整参数清单——裁剪属性列表会裁掉历史踩坑后的必带项（-PcppWorldgenDir，资源布局与 marker 路径不一致的绕过项）（发现 #15，260903-14）+ #15 根治复盘：解压死路主因是资源集不完整而非布局（死分支信号）+ bin-diag 旧 exe 假阴性——探针用前必须核产物时间戳（发现 #16，260903-15） |
| 已确认的算法/协议指纹 | [discovered/algorithm-fingerprints.md](discovered/algorithm-fingerprints.md) | MC 密度/噪声算法特征、scale/seed 坑、key 语义、性能指纹（缓存失效/spline 扁平化/边界角点复用）、aquifer est 冷扫描机制指纹——per-chunk 新建 Aquifer × 全价 init 采样 ≈15.4ms/chunk（发现 #16，260903-10） |
| 混淆/反逆向手法 | [discovered/anti-patterns.md](discovered/anti-patterns.md) | （CoreSwap 非二进制逆向，一般空置） |
| 工作流模式 | [discovered/workflow-patterns.md](discovered/workflow-patterns.md) | judge 审查门强制触发点、scout 勘探前置、fan-out 多假设分叉强制触发、块级真相验证法、参照状态三查、FEATURE 独立于地形、getChunk 阶段语义（2026-08-09 更新）、接管单阶段后的后续阶段上下文依赖（2026-08-31）、临时产物唯一隔离区（260901-03）、cppReplace 存档口径三阶段归因法 + 同 dll 重跑非确定容差（发现 #10，260901-03）、嵌套接管管线双跑风险——内层全管线 × 外层分步拦截（发现 #11，260902-01）、静态对拍必须对拍解析产物而非输入原文——假阴性掩盖真 bug（发现 #12，260902-03）、探针坐标 bug 制造 100% 单向假象——探针输出先做 sanity check + one-step decisive probe 逐层收敛（发现 #13，260902-04）+ 测量侧先查三犯（wBiome 坐标/NoiseConfig 维度/pregen 提升 chunk）与 RegistryKey 命名空间过滤恒 false、探针零输出先查过滤/驱动条件（#13 补充案例，260902-05/06）+ 探针指标盲区（指标先从判别证据反推）与行首锚 grep 假零输出（#13 补充案例，260902-07）+ 探针阶段同源性——stageMask 只控本侧阶段，noise-only 判据看存档内容非开关日志（发现 #14，260902-09）+ 假阴性陷阱：空切片/切分散→假 100% 一致，sanity 行强制打长度+common 数（#12 家族案例，260902-09）+ 零面擦边格签名判别法——微差残差先看 |d| 量级，单侧普查只能封闭不能定量（发现 #15，260902-13） + 对照基线归因法——异常差异先跑普通坐标对照分离坐标因素与实现因素，对照更差=差异与X无关的强证据；单对照只分离一个变量，混杂须先 fan-out（发现 #16，260902-14） + 跨探针对比坐标钉死律——打印坐标≠采样坐标时结论无效，oracle 逐点复核识别混列；静态归因须单点隔离复测（发现 #17，260903-06） + 跨 session 基准数字不可直接续推——缺口径标注的历史数字视为不可比（发现 #18，260903-08） + Java bench 前必须删 run\world——世界状态第四查，「快一个量级+min≈0」缓存假象签名（发现 #19，260903-09） + 死参数制造假判别——判别实验必须验证「自变量真被改变」，恒等式自检（发现 #20，260903-09） + 微测基线样本模式决定缓存命中率——单点微测外推热路径必须复刻调用形态（发现 #21，260903-10） + 自由参数凑数反模式——量级核算乘数须独立实测来源，异常贴合是凑数嫌疑（发现 #22，260903-10） + 诊断证据摘要漏行制造 N× 缺口假象——分解数据每行都进证据包（发现 #23，260903-10） + 多臂顺序 bench 顺序效应假交互——差分必须 chunk 粒度交错（发现 #24，260903-10） + 静态调研结论失真两例——差距点必须核生产路径可达性、Java 常量必须追取值源头（发现 #25，260903-11；#21 补充案例 260903-11：working set 失配——e2e 收益超微测上界签名）+ #25 补充案例第三例——静态地图「恰好一致」断言必须显式算术（+15→+12≠+16，260903-12）+ #21 第二次量化实锤——同代码 hot/cold 形态差 ~95×、跨 session 单价稳定性作 fan-out 免触发收敛判据（260903-12） + 预加载/注册表与运行时查询集合同步——新增 expect 型查表调用点必须同步预加载来源，缺失只在低频分支触发，小样本全绿≠无缺失，大 region sweep 是暴露手段（发现 #26，260903-14）+ 启动期断言「运行时引用 key ⊆ 预加载集合」落地（机械收集 + 就近清单 + 盲区诚实声明）——负向测试是断言生效的唯一证明且其假阴性比假阳性更危险（发现 #27，260903-15） |
| 预置知识 | [builtin/README.md](builtin/README.md) | 预留；RE-Framework knowledge-builtin 为汇编逆向内容，CoreSwap 不复制 |

## 写入规则（core.knowledge，2026-08-21 对齐记录价值门）

- **先过记录价值门**（AGENTS.md §三.2）：高价值（错误链/判据/坑/反模式）→ 详写；中价值（算法指纹/惯用法）→ 简写；低价值（一次性结论/对齐状态快照）→ **不写知识库**（只留 .investigations/ 过程或直接不落盘）。
- 发现可复用模式 → 立即写入 discovered/ 对应文件（不拖）
- 每条格式：`## 发现 #N: 标题` + 发现时间/发现者/来源定位/置信度/module + 观察/证据/如何利用
- 写入后**同步更新本 INDEX**（对应分类加一行链接）
- 与 docs/ 边界：docs/ 记「对 1.20.1 的验证结论」，本目录记「可复用的通用规律」（1.18/1.19 迁移时直接查这里）
- **无复用价值结论不写 docs**（主题篇/时间线也不是知识库核心资产，见框架 §6 价值门）——别为一次性结论派 subagent。

