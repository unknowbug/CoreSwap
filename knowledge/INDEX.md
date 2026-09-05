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
| 语言/编译器惯用法 | [discovered/compiler-idioms.md](discovered/compiler-idioms.md) | Java/MC 代码生成模式、浮点/整数语义、插值公式、MSVC/Windows·JVM 平台坑、跨层 id 域错位 raw block id vs state id（2026-09-01）、锚坐标换算 off-by-one below_top/above_bottom（260901-02）、JSON 布尔字段经 as_f64 读取恒 false——分型标量 API 静默语义腐蚀（发现 #8，260902-03）、跨 session raw id 标注三查——未验证标注当公理继承整链作废（发现 #9，260902-07）、Rust 半开区间 rev().step_by() 复刻 Java 含两端递减 for 的 off-by-one——首点值+下界包含性两独立参数（发现 #10，260903-13）、诊断门控 flag 在初始化器内唯一置位的鸡生蛋死锁——激活判据（env 存在性）与数据初始化必须分离（发现 #11，260904-09）、mixin 包禁止任何非 mixin 类含 static nested——IllegalClassLoadError 直接判据（发现 #12，260904-09） |
| 还原工具误译及修正 | [discovered/f5-bugs.md](discovered/f5-bugs.md) | javap/反编译不可信点、反射缓存污染、修正方法 |
| 构建/工具链坑 | [discovered/build-tooling.md](discovered/build-tooling.md) | gradle daemon/env/参数解析、task UP-TO-DATE 跳过、文件同步、fs::copy 保留 mtime——产物判新旧用内容指纹（发现 #6，260902-01）、GRADLE_USER_HOME 全套状态指工作区绕开 home 权限 + 参照文件名四要素核对（发现 #7，260902-02）、gradle -P→-D 手工映射清单遗漏静默不生效（发现 #8，260902-04）、gradle runServer --nogui 非 CLI 选项 + rustStages 缺映射行静默不生效（发现 #9，260902-09）、参照文件核对四要素不够——文件名不含 stage，SURFACE 参照被当 FULL 用贯穿多轮；判据升级五要素（seed/size/origin/dim/stage 内容指纹：阶段特征 id 有无）（发现 #10，260902-10）、header/文件名本身也可能是错的——chunk 配对以内容实测坐标为准 + 探针恒等式自检（match 差≠分解计数差即假配对）（发现 #11，260903-02）、spv/comp 多产物部分更新——生成器多产物重生成必须整体原子更新，逐位一致哨兵结论须配已知值哨兵点验产物健康（发现 #12，260903-04）、GRADLE_USER_HOME 未指工作区致 native-platform.dll 加载失败——#7 第三表现面（发现 #13，260903-12）、runServer 大 region 预生成触发 watchdog 60s 强杀 + 探针 dump 必须内嵌 seed 头（发现 #14，260903-12） + gradle run 存档口径照抄历史 run 完整参数清单——裁剪属性列表会裁掉历史踩坑后的必带项（-PcppWorldgenDir，资源布局与 marker 路径不一致的绕过项）（发现 #15，260903-14）+ #15 根治复盘：解压死路主因是资源集不完整而非布局（死分支信号）+ bin-diag 旧 exe 假阴性——探针用前必须核产物时间戳（发现 #16，260903-15）+ 导出产物在盘≠本次执行体生成——重导命中旧 world 缓存判别签名 = 每chunk耗时 + pregen 时长（发现 #18，260904-08） |
| 已确认的算法/协议指纹 | [discovered/algorithm-fingerprints.md](discovered/algorithm-fingerprints.md) | MC 密度/噪声算法特征、scale/seed 坑、key 语义、性能指纹（缓存失效/spline 扁平化/边界角点复用）、aquifer est 冷扫描机制指纹——per-chunk 新建 Aquifer × 全价 init 采样 ≈15.4ms/chunk（发现 #16，260903-10） |
| 混淆/反逆向手法 | [discovered/anti-patterns.md](discovered/anti-patterns.md) | （CoreSwap 非二进制逆向，一般空置） |
| 工作流模式 | [discovered/workflow-patterns.md](discovered/workflow-patterns.md) | judge 审查门强制触发点、scout 勘探前置、fan-out 多假设分叉强制触发、块级真相验证法、参照状态三查、FEATURE 独立于地形、getChunk 阶段语义（2026-08-09 更新）、接管单阶段后的后续阶段上下文依赖（2026-08-31）、临时产物唯一隔离区（260901-03）、cppReplace 存档口径三阶段归因法 + 同 dll 重跑非确定容差（发现 #10，260901-03）、嵌套接管管线双跑风险——内层全管线 × 外层分步拦截（发现 #11，260902-01）、静态对拍必须对拍解析产物而非输入原文——假阴性掩盖真 bug（发现 #12，260902-03）、探针坐标 bug 制造 100% 单向假象——探针输出先做 sanity check + one-step decisive probe 逐层收敛（发现 #13，260902-04）+ 测量侧先查三犯（wBiome 坐标/NoiseConfig 维度/pregen 提升 chunk）与 RegistryKey 命名空间过滤恒 false、探针零输出先查过滤/驱动条件（#13 补充案例，260902-05/06）+ 探针指标盲区（指标先从判别证据反推）与行首锚 grep 假零输出（#13 补充案例，260902-07）+ 探针阶段同源性——stageMask 只控本侧阶段，noise-only 判据看存档内容非开关日志（发现 #14，260902-09）+ 假阴性陷阱：空切片/切分散→假 100% 一致，sanity 行强制打长度+common 数（#12 家族案例，260902-09）+ 零面擦边格签名判别法——微差残差先看 |d| 量级，单侧普查只能封闭不能定量（发现 #15，260902-13） + 对照基线归因法——异常差异先跑普通坐标对照分离坐标因素与实现因素，对照更差=差异与X无关的强证据；单对照只分离一个变量，混杂须先 fan-out（发现 #16，260902-14） + 跨探针对比坐标钉死律——打印坐标≠采样坐标时结论无效，oracle 逐点复核识别混列；静态归因须单点隔离复测（发现 #17，260903-06） + 跨 session 基准数字不可直接续推——缺口径标注的历史数字视为不可比（发现 #18，260903-08） + Java bench 前必须删 run\world——世界状态第四查，「快一个量级+min≈0」缓存假象签名（发现 #19，260903-09） + 死参数制造假判别——判别实验必须验证「自变量真被改变」，恒等式自检（发现 #20，260903-09） + 微测基线样本模式决定缓存命中率——单点微测外推热路径必须复刻调用形态（发现 #21，260903-10） + 自由参数凑数反模式——量级核算乘数须独立实测来源，异常贴合是凑数嫌疑（发现 #22，260903-10） + 诊断证据摘要漏行制造 N× 缺口假象——分解数据每行都进证据包（发现 #23，260903-10） + 多臂顺序 bench 顺序效应假交互——差分必须 chunk 粒度交错（发现 #24，260903-10） + 静态调研结论失真两例——差距点必须核生产路径可达性、Java 常量必须追取值源头（发现 #25，260903-11；#21 补充案例 260903-11：working set 失配——e2e 收益超微测上界签名）+ #25 补充案例第三例——静态地图「恰好一致」断言必须显式算术（+15→+12≠+16，260903-12）+ #21 第二次量化实锤——同代码 hot/cold 形态差 ~95×、跨 session 单价稳定性作 fan-out 免触发收敛判据（260903-12） + 预加载/注册表与运行时查询集合同步——新增 expect 型查表调用点必须同步预加载来源，缺失只在低频分支触发，小样本全绿≠无缺失，大 region sweep 是暴露手段（发现 #26，260903-14）+ 启动期断言「运行时引用 key ⊆ 预加载集合」落地（机械收集 + 就近清单 + 盲区诚实声明）——负向测试是断言生效的唯一证明且其假阴性比假阳性更危险（发现 #27，260903-15）+ 并发 bench 互相污染——性能 bench 必须串行、「min 稳定 median 变重」对外源抢占不具区分力、单点离群先复跑再归因（发现 #28，260903-16）+ 一次性 bench 探针必须自带时间戳头注——时序复核从过程重建变日志实录（发现 #29，260903-16）+ 立项级提案 kill 判据前置——一派候选共享前提先做 n 扫描验证摊薄，边际成本全档平坦=整派出局（发现 #30，260904-01）+ 核验证据落盘原始记录——verify-rec 补记写原始 txt 末尾，自证力从转述变原件内证据，与 #29 跑前/跑后互补（发现 #31，260904-01）+ env 判别生效证据行为化（发现 #37）/ env 默认值方向三查（发现 #38）/ 参照自身缺陷辨识——对照基线对 vanilla 校验（发现 #39）（均 260904-06） |
| 预置知识 | [builtin/README.md](builtin/README.md) | 预留；RE-Framework knowledge-builtin 为汇编逆向内容，CoreSwap 不复制 |

## 写入规则（core.knowledge，2026-08-21 对齐记录价值门）

- **先过记录价值门**（AGENTS.md §三.2）：高价值（错误链/判据/坑/反模式）→ 详写；中价值（算法指纹/惯用法）→ 简写；低价值（一次性结论/对齐状态快照）→ **不写知识库**（只留 .investigations/ 过程或直接不落盘）。
- 发现可复用模式 → 立即写入 discovered/ 对应文件（不拖）
- 每条格式：`## 发现 #N: 标题` + 发现时间/发现者/来源定位/置信度/module + 观察/证据/如何利用
- 写入后**同步更新本 INDEX**（对应分类加一行链接）
- 与 docs/ 边界：docs/ 记「对 1.20.1 的验证结论」，本目录记「可复用的通用规律」（1.18/1.19 迁移时直接查这里）
- **无复用价值结论不写 docs**（主题篇/时间线也不是知识库核心资产，见框架 §6 价值门）——别为一次性结论派 subagent。


> 260904-03 追加：workflow-patterns 新增**发现 #32**（gradle daemon 复用吞掉客户端 env——env 门控判别实验的死同值风险；#20 的 env/daemon 实例扩展）。

> 260904-05 追加：workflow-patterns 新增**发现 #33/#34/#35**（载具可比性——跨载具残差禁止互引 §9.7 具体化；feature 消融经谓词耦合——归零式 A/B 对 feature 课题无效；量级核算排除法定位写者）；build-tooling 新增**发现 #17**（block_probe 调用契约三坑：argv[3] vanilla 参照 / wgDir data\minecraft\ 层 / -save 无 biome 段格式）。

> 260905 追加：workflow-patterns 新增**发现 #36**（验证探针与生产执行体不同源——C++ block_probe ≠ Rust worldgen.dll，排除结论禁止跨执行体迁移；残差归因前核「执行体三元组」：加载文件/构建产源/构建时间 vs 修复时间）。

> 260904-06 追加：workflow-patterns 新增**发现 #37/#38/#39**（env 判别生效证据必须行为化——常规日志行不作 env 开关证据 + 开关分支加独立一次性日志；worker 判读源码语义前核 env 默认值方向——「=0 反转」型误读造死参数假判别，#20 家族；参照自身缺陷辨识——「mod≠参照」≠「mod 错」，对照基线自身要对 vanilla 校验，#16 对偶面）+ **#36 补充案例**（交接结论廉价验证第二例：NEXT_SESSION「现役 dll 早于 stage-skip 修复」被 dumpbin /exports + git 时间戳一轮推翻）。

> 260904-06 追加（二）：workflow-patterns 新增**发现 #40/#41/#42**（blocks 列读布局纪律——y-major 权威 blocks.h:69 + 已知地形 sanity 自检，幻影列剖面造「幻幕帘」推翻错课题；judge 独立重跑复用同一变换代码 = 复现同一 bug，独立复算必须独立实现；「0<d」式静态机制断言未实测当公理，一轮 dump 证伪——布局事故三连教训，源自 incident-layout-260904-06）；algorithm-fingerprints 新增**发现 #17**（aquifer barrier margin 机制指纹——|d|≈0.02 微负带 = margin stone 高发区，「stone 但 d≤0」≠ bug）。

> 260904-08 追加：workflow-patterns **#40 补充案例**（y-major 布局经实机 A/B 对拍升 behavior 级证据 + 两个坐标还原反模式新实例——divmod(i,4096) 跨层步长混入 y 造「幻影见证坐标」、「实机对拍零吻合」= 错位一票否决、手写 index→坐标换算先经直读器 3 点抽查）；build-tooling 新增**发现 #18**（导出产物在盘 ≠ 本次执行体生成——重导命中旧 world 缓存，判别签名 = chunk "FULL in 0-1ms" vs pregen ~24s）。

> 260904-09 追加：workflow-patterns 新增**发现 #43**（交接「静态公式逐项零偏离」当公理续推输入/缓存侧——真根因是链路构造参数层一行缺失（aquifer splitter 漏 split_str("minecraft:aquifer")）；判据：双向残差签名优先怀疑随机派生/组合选择层、「逐行对拍」必须附覆盖面声明（不含 splitter/random provider 派生等装配参数）、同文件同构调用是零成本就地对照）；compiler-idioms 新增**发现 #11/#12**（诊断门控鸡生蛋死锁 + mixin 包禁非 mixin 类）。来源：.investigations/residual-1830/（残留 1830 收口，decisive 1830→76）。

> 260904-10 追加：workflow-patterns 新增**发现 #44**（载具间实装分歧按「同一 Java 机制多点接线」核对——C++ 已对齐机制（BiomeAccess zoom）Rust 侧可整体缺失，跨载具课题先 diff 两侧同功能站点接线清单；来源：残留 76 surface biome 缺 BiomeAccess 收口）；compiler-idioms 新增**发现 #13**（docs 口径失准当修复依据须先一手源码核对——docs/06 ==stone vs 一手非空非流体）。来源：.artifacts/lossless-accel/residual76-verdict-260904-10.md（decisive 76→12）。

> 260904-13 追加：algorithm-fingerprints 新增**发现 #18**（MultiNoise 分类器 1e-4 定点 long 域——f64 复刻平局点假严格差分类翻转，残 9 实例）；workflow-patterns 新增**发现 #45**（同输入异分类三源隔离法 + NoiseValuePoint long 定点反射直读 + 探针 quart 直采≠生产块坐标语义）；build-tooling 新增**发现 #19**（-PblockProbe.full 点分名静默不生效，映射实为 blockProbeFull，#8 家族三犯 + A/B 隔离定责法）。来源：.artifacts/lossless-accel/residual9-verdict-260904-13.md（decisive 12→3）。

> 260905-03 追加：workflow-patterns 新增**发现 #46**（跨实现光照/序列化对比缺键语义不对称——vanilla 缺 SkyLight 键=隐式 15、rust 接管侧缺键=flag1 全 0，统一填充制造整 section 0↔15 伪翻转 1.47M 假差异；判据：跨实现对比 MUST 先核两侧序列化省略语义）+ **#36 补充案例第三例**（「fallback 收不到 propagateLight」假设一轮 DENY，真根因 = worldgen feature 放置分歧）；build-tooling 新增**发现 #20**（1.20.1 chunk NBT 无 xPos 键——region 坐标+槽位推导 cx=rx*32+(i&31) + python unpack_from 直读 int64 不推进位置指针解析失步）；f5-bugs 新增**发现 #5**（引用源码树参照前必须核版本/DataVersion——.tmp/net 疑非 1.20.1，isLightOn 写入与实测矛盾）；compiler-idioms 新增**发现 #14**（light_data.json opacity:-1 经 parse_u8_field clamp 成 0，18 条目——负值经无符号 parse 静默语义有损，#8 布尔姊妹案例）。来源：.investigations/light-opt/g2-convergence-260905-03.md + d3-bench-260905-03.md。

> 260905-04 追加：workflow-patterns 新增**发现 #47**（性能优化 golden 逐位不变对照法——pre 冻结 + post 复跑 + hash 等值任何不等即 FAIL；单调松弛 BFS 边界种子等价剔除论证模板——零贡献证明 + 不动点唯一性两支齐备。来源：.investigations/light-opt/d3-opt-round1-260905-04.md）。

> 260905-04 追加（二）：compiler-idioms 新增**发现 #15**（位打包标量域部分位判零陷阱——blocks9 ABI 快路径只判 id 位吞掉 id=0+luminance>0 合法光源，合成 golden 一轮捕获；判据=判全字或显式声明其余位恒零不变量，#8/#14 静默语义腐蚀家族）；workflow-patterns 新增**发现 #48**（双采集对拍模式——重构收集/预处理循环新旧两路同调用点逐元素对拍（临时诊断+计数门控+MATCH/MISMATCH 显式标记+验证后移除），与 #47 互补：golden 适用于可冻结输入的纯函数内核，双采集适用于无法冻结输入的运行时循环）；build-tooling 新增**发现 #21**（e2e_run 复用脚本外部状态依赖——RCON 开关被上轮收口复原动作重置 off，stop 静默失败强杀世界；判据=复用脚本开工前核外部依赖项在位）。来源：.investigations/light-opt/d3-opt-round2-260905-04.md。

> 260905-05 追加：workflow-patterns 新增**发现 #49**（交接/导出产物「完成度伪差」——同 seed 不同 run 的存档对比装饰完成度随停服时机/init 时序剧烈变化，15.8×「回归」实为伪差，判据=跨 run 对比先同方法论化+零改动正向对照验伪，#18 家族第三形态）+ **发现 #50**（接管 init 晚于 Done → 同一存档内 vanilla/接管两种装饰来源并存，判据=判别实验先核接管生效的 chunk 范围，装饰级日志优先于统计推断）；build-tooling 新增**发现 #22**（Java 侧采集三坑：gradle daemon 吞客户端 env——JAVA_TOOL_OPTIONS 哨兵法验传播、Loom RunConfigSettings 无 environment() 需 task 级 JavaExec、rcon 同步命令可超共享连接 10s 超时需 per-command 独立连接，#19 通道名实不符家族）。来源：.investigations/feature-parity/phase5-interim-260905-05.md（含取代记录）+ .artifacts/feature-parity/judge-verdict-260905-05.md。

> 260905-06 追加：workflow-patterns 新增**发现 #51**（跨 run 受控 A/B 噪声基线与信号同阶——两批纯 vanilla 臂互差 241,080 vs 信号 239,594，跨 run 只能验无回归不能裁量级收敛，判据=噪声基线前置必测）+ **发现 #52**（确定性区域 dump 载体——纯实现生成 + 固定快照 + SHA256 对拍，代码 diff 唯一变量，idk-7 裁决 −30.8% 首次可裁决）；build-tooling 新增**发现 #23**（cargo -p 薄壳依赖 rlib 陈旧假绿——Finished ≠ 依赖重编，判据=rlib mtime 晚于源码 + 字符串哨兵）+ **发现 #24**（gitignore 目录级 prune 使 ! 白名单失效 + data/* 锚定坑，修复=三段链式白名单，判据=git check-ignore -v 双向核验）。来源：.investigations/feature-parity/260905-06-errors.md + .tmp/feature-parity-260905-06/。> 260905-10 追加：workflow-patterns 新增**发现 #53**（env 门控默认值当公理——交接「默认关」实际 env_enabled 默认开，两臂 dump 全等暴露；#37 家族第二犯 → 强制三查升级：消费点直读 / NEXT_SESSION 声明对照 / 行为化哈希哨兵）；build-tooling 新增**发现 #25**（Java 侧采集三坑第二辑：mixin 门控 JAVA_TOOL_OPTIONS -D 是 sysprop 非 env 须走 findProperty→vmArg 映射、非 cancellable 方法 @Inject 禁 setReturnValue 须 @Redirect/peek、runServer SEEDLOG 全量噪声 + spawn 预生成拖慢——整改 mixin chunk 过滤）。来源：.investigations/feature-parity/260905-10-* + .artifacts/feature-parity/candidate-beehive-260905-10.md。
