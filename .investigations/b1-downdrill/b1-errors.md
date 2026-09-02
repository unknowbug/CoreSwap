# B1 下钻错误台账（260902-05/06 session）

> 五段式（现象/根因/定位/修复/教训）。来源：facts-260902-05/06.md。判错经验（最高价值）见各条「教训」加粗。

## E-B1-1：后台 job 工作目录 ≠ 会话工作区，相对路径全部落空（两次空跑）

- **现象**：后台命令（gradle runServer 等）两次静默空跑——无产物、无报错输出，任务「成功」结束。
- **根因**：DSH 后台 job 每次起新进程，工作目录不继承会话工作区——脚本内相对路径（run 目录、.tmp/ 输出）解析到错误位置，命令在错误 cwd 下执行了错误目标（或直接找不到目标）。
- **定位**：检查 job 输出发现无任何预期日志行；回溯命令行发现全部用相对路径。
- **修复**：后台命令一律改绝对路径（gradle -p 指定项目目录、输出写绝对 .tmp/ 路径）。
- **教训**：**后台/异步执行环境的 cwd 不可假设继承自会话——跨进程边界的路径一律绝对化**；「命令成功返回但零输出」是 cwd 错位的签名。

## E-B1-2：本工程无 gradlew wrapper，`.\gradlew` 不存在

- **现象**：PowerShell 跑 `.\gradlew` / `.\gradlew.bat` 报「无法识别/不存在」。
- **根因**：Java 探针工程未提交 gradle wrapper（gradlew/gradle/wrapper 缺失），通用模板命令不适用。
- **定位**：`Test-Path gradlew*` 为 false；`Get-Command gradle` 确认 PATH 有全局 gradle.bat（D:\gradle\gradle-8.13）。
- **修复**：直接用 PATH 的 `gradle`（gradle.bat）。
- **教训**：**跑模板命令前先确认工程实际工具布局（wrapper 有无、gradle 版本），勿照抄通用命令**。

## E-B1-3：yarn 1.20.1 `PlacedFeature` 方法实名 `generate` 非 `place`（mixin 静默不命中）

- **现象**：mixin 注入 `PlacedFeature.place` 后无任何命中/无输出。
- **根因**：yarn 1.20.1 中方法实名是 `generate`（且有 3 处重载，mixin 须带方法描述符才能唯一定位）；worker 自检清单①「genSources 核对」在无法执行时被降级为「名字匹配」，踩中映射名差异。
- **定位**：从 loom-cache sources.jar 提取 PlacedFeature 源码核实真实签名。
- **修复**：mixin 改 `generate` + 带描述符指定重载。
- **教训**：**mixin 目标方法名 MUST 从 sources.jar 核实（loom-cache 即可），不得凭记忆/通用映射写**；「自检项无法执行时降级处理」要显式声明降级并另行补验，不能静默跳过。

## E-B1-4：mixin 类禁止非 private 静态成员（@Unique public static 也拒）

- **现象**：mixin 编译期 InvalidMixinException——本版本连 `@Unique public static` 方法都拒绝。
- **根因**：mixin 转换器约束：目标类中非 private 静态成员会泄漏/冲突，本版本（mixin 0.8.x 级）强制拒绝。
- **定位**：异常栈直指静态成员声明行。
- **修复**：调试计数用 `@Unique private` 实例字段 + 反射读取；注意反射 `Class.forName(mixin类)` 会触发自转换失败（读不到计数，无害但要知道）。
- **教训**：**mixin 类内状态一律 @Unique private 实例成员**；反射读取 mixin 类成员走合并后目标类而非 mixin 类名。

## E-B1-5：`RegistryKey.getValue().toString()` 带命名空间，裸路径 equals 过滤恒 false（CSV 空 4 轮真根因）

- **现象**：探针 CSV 连续 4 轮零数据行，无报错。
- **根因**：`RegistryKey.getValue().toString()` 返回带命名空间全名（`minecraft:the_nether`），与裸路径 `the_nether` 的 equals 过滤恒 false——过滤条件把全部行静默滤掉。属「测量侧先查三犯」之过滤条件类（workflow-patterns 发现 #13 同族）。
- **定位**：排除驱动侧后，打印实际 getValue().toString() 值与过滤串对比，一眼可见命名空间前缀。
- **修复**：过滤改 `getValue().getPath()` 或带命名空间比对（本 session 用后者：`minecraft:the_nether`）。
- **教训**：**Registry/Identifier 字符串化必带命名空间——过滤/比较用 getPath() 或全名对全名**；探针零输出先查过滤条件再查数据源（发现 #13 补充案例已沉淀）。

## E-B1-6：BufferedWriter 未 flush 即 `server.stop(false)` → 小数据量全丢

- **现象**：探针输出文件空/截断，数据量越小丢得越干净。
- **根因**：BufferedWriter 缓冲未满即不落盘；`server.stop(false)` 快速停服不触发关闭钩子里的 flush/close，缓冲内数据全丢。
- **定位**：小数据集必丢、大数据集部分保留的「缓冲量级」特征。
- **修复**：改逐行 append（Files.write APPEND 或每行后 flush）。
- **教训**：**探针写文件不用依赖进程优雅退出的缓冲写——逐行落盘或显式 flush 点**；「数据量越小丢得越彻底」是缓冲丢失签名。

## E-B1-7：cppReplace 轮 CppBridge.extractWorldgenDir 失败——resources 布局缺 data/ 层

- **现象**：cppReplace run 报 extractWorldgenDir 失败（IllegalStateException: worldgen-data not found in mod resources），Rust 侧拿不到 worldgen 数据，server 崩。
- **根因**：resources 内 worldgen-data 布局缺 `data/` 层（minecraft/ 直接在根），与 CppBridge marker 期望的 `data/minecraft/worldgen/noise_settings/overworld.json` 目录结构不符。
- **定位**：异常栈指向 extract marker 检查；对照历史 session 完整命令行发现 `-PcppWorldgenDir` 显式参数。
- **修复**：显式 `-PcppWorldgenDir=E:/PYTHON/CoreSwap/versions/1.20.1/data/worldgen`（历史 session 同法）。
- **教训**：**探针工程的非默认参数（数据目录映射）每轮显式带上，勿依赖 resources 默认布局**；历史可跑通的完整命令行是第一手模板。

## E-B1-8：BlobProbe 单独跑无 driver 不生成 nether chunk（COLPROF 空跑一轮）

- **现象**：COLPROF 单独运行零输出（空跑一轮）。
- **根因**：探针 mixin 只在 chunk 生成时触发；BlobProbe 的 bench driver 才会驱动 nether chunk 生成——单独跑 colprof 没有任何 driver，chunk 根本不生成，探针自然零数据。属「探针零输出先查驱动条件」类。
- **定位**：确认 log 无 chunk 生成活动；检查 driver 参数拼装逻辑。
- **修复**：colprof 必须搭配 `-PblobProbe`（driver 驱动生成）。
- **教训**：**探针零输出两类根因：过滤条件不匹配（E-B1-5）/ 无驱动条件不生成数据（本条）——先问「chunk 有没有被生成」再问「生成了有没有读到」**（发现 #13 补充案例已沉淀）。


## E-B1-9：raw id 标注未经映射核实即当真值、跨 session 当公理继承——整条 H1 机制链建立在错误标注上（重大）

- **现象**：260902-06 COLPROF 10/25 列 diff 中「`99|0->19319`」被标注为「air→lava，熔岩海面 y=99」、「`100~104|0->5854`」被标注为「air→netherrack 实心兜底」——由此确立 H1 机制链环 1「Rust surface 熔岩海缺失（netherrack 实心兜底）」candidate，并设计出「熔岩流体填充」修复方案。260902-07 用 `[LAUIDMAP]` 打印 Java STATE_IDS 权威映射实测：`19319 = blackstone`（黑石）、`5854 = basalt`（玄武岩）——air=0 lava=96 water=80 netherrack=5850 basalt=5854 blackstone=19319 soulsand=5851 soulsoil=5852 magma=12402 bedrock=79。两个标注全错：10 列 diff 真相 = **V 黑石底 vs C 玄武岩底，两侧均实心**，无任何熔岩缺失；LAVAAUDIT v2 全扫 11,443 公共列 air→lava 面向两侧均为零——H1 环 1 整环被证伪，修复方案作废（lib/dll 幸而零改动，未产生实际损失）。
- **根因**：机制层面的错误是「**未解释的观测被赋予了解释性标注，随后标注被当作观测本身继承**」：① raw id（19319/5854）首次出现在 COLPROF 输出时，探针没有同时输出 id→方块映射，session 内凭直觉推断（数值大≈新块≈lava/netherrack 熔岩海家族）赋了语义；② 该标注写进 facts-260902-06 和 09 篇定案节后，下一 session 把「19319=lava」当公理直接续推（建机制链、设计修复、排回归判据），从未回头核映射——**标注从「未验证解释」升格为「事实」只发生了在文档传递里，不在数据里**。这与 seed 三犯同构：种子错位（③ 多世界 worldSeed 错位）也是「上一轮的未验证参数被下一轮当真值」。本质是交接纪律缺口：seed/坐标/参照文件都有三查，唯独「id 标注」没有。
- **定位**：本 session 按 judge 要求先做廉价独立验证——给探针加 `[LAUIDMAP]`（启动时从 `Registries.BLOCK` / STATE_IDS 遍历打印 id→方块名一次性映射），vanilla 轮一跑即实锤：19319 在 STATE_IDS 展平序中是 blackstone 而非 lava（lava=96）。验证成本一轮，昨日的机制链全部改写。
- **修复**：① 探针新增 LAUIDMAP 一次性输出（v2+ 格式）；② H1 环 1 按 §15.4 出取代注记（09 篇），修复方案作废；③ 立纪律「**标注三查**」并写入 NEXT_SESSION 开工检查项：探针输出的 raw id/枚举值/魔法数**首次解释前必须先建立 id→语义映射**（探针打印映射，不靠推断）；标注跨 session 传递时 MUST 带「已验证/未验证」标记，未验证标注续用前先做廉价独立验证（≤ 一轮）；「数值大 = 新块 = 熔岩海家族」类直觉推断不构成标注依据。
- **教训**：**标注不是数据——把「解释」写进事实链之前，解释本身也需要一条证据**。可复用判据：① 任何 raw id / 枚举 / 常量的语义主张，第一动作 = 从权威源（注册表/STATE_IDS/映射表）打印核实，禁止凭数值范围直觉命名；② 交接文档里的「未验证解释」要显式标注，续推前独立验证（§16.3 宿主交接验证）；③ 机制链的每一环问一句「这一环的输入标注是谁验证的」——环 1 错则整链作废，越早核实越便宜。与 AGENTS.md「交接结论验证纪律」（M14/M11 复盘）同族，是其在 id 标注域的具体化。

## E-B1-10：探针指标盲区——lavaAudit v1 只记 above=lava 转换，恰好测不到 air→lava 面向

- **现象**：lavaAudit v1 全扫输出「99.4% 列一致」，看似支持「熔岩海两侧基本一致」；但该指标同时也没有发现任何熔岩海异常——而 v2 加记 below=lava 面向后，直接实锤「两侧 air→lava 面向均为零」，推翻了 H1 环 1 的存在本身。v1 的「一致」对「熔岩海缺失」这一核心问题**零判别力**。
- **根因**：设计探针指标时凭「测熔岩」的直觉命名（lavaAudit → 记 lava 转换），没有先从「结论所需的最小判别证据」反推：要判定「熔岩海缺失」，需要观察的是**实心→熔岩的面向转换（below=lava）**，即 air→lava 的转换面；v1 只记 above=lava 的转换，恰好落在盲区—— lavaTopY 看得到流动熔岩的位置，看不到「本该是熔岩面的地方变成了实心」。指标名与判别目标错位，产生了「测了且一致」的假安心。
- **定位**：judge 审查指出 v1 指标与待判命题不匹配（「叫 lavaAudit 但测不了熔岩海」）；v2 补 lavaSurfY（below=lava）字段后一轮实锤。
- **修复**：探针 v2+ 格式 `x,z,lavaSurfY,lavaTopY,n`（加记 below=lava 面向）；v1 数据废弃（99.4% 一致率不构成世界一致证据）。
- **教训**：**设计探针指标先从判别证据反推，再命名**——动手前先写一行「要判别命题 P，最小充分证据是什么」，再检查指标是否恰好覆盖该证据；「指标名听起来对口」不是覆盖证明。与发现 #12（对拍对象错级）同族：都是测量设计与判别目标脱节。

## E-B1-11：grep 行首锚 `^\[LAVAAUDIT\]` 对带日志前缀的 log 恒零命中——假「零输出」误警

- **现象**：对 server stdout log 用 `^\[LAVAAUDIT\]` 过滤，零命中——误判「探针没有输出」，险些触发一轮无谓的重跑排查。
- **根因**：log 文件里的行带日志框架前缀（时间戳/线程名等），`[LAVAAUDIT]` 不在行首，行首锚 `^` 使匹配恒 false——又是「过滤条件把全部行静默滤掉」家族（E-B1-5 同族：命名空间前缀 / 本条：日志前缀）。
- **定位**：肉眼抽查 log 原文一行，发现 `[LAVAAUDIT]` 前有前缀；去掉 `^` 锚即命中。先查自己的过滤条件，再怀疑数据源（发现 #13 判据再次生效）。
- **修复**：改用不带行首锚的 `[LAVAAUDIT]` 或按子串匹配。
- **教训**：**「零命中」先查过滤正则的锚定假设（行首/行尾/全行匹配）是否与实际行格式一致**——log 行几乎总有前缀，`^` 锚默认不可用；与 E-B1-5 合并记为「恒零命中 = 先打印一行原文对格式」。

---

## 错误→根因速查表

| # | 错误 | 根因类 | 一句话教训 |
|---|------|--------|-----------|
| E-B1-1 | 后台 job 两次空跑 | 环境假设（cwd 继承） | 跨进程路径一律绝对化 |
| E-B1-2 | gradlew 不存在 | 环境假设（wrapper 存在） | 先查工程工具布局再跑模板命令 |
| E-B1-3 | mixin 不命中 | 映射名错（generate≠place）+重载 | mixin 目标名从 sources.jar 核实 |
| E-B1-4 | InvalidMixinException | mixin 静态成员约束 | mixin 状态一律 @Unique private |
| E-B1-5 | CSV 空 4 轮 | 过滤条件恒 false（命名空间） | Registry 字符串比较用 getPath() |
| E-B1-6 | 输出文件空/截断 | 缓冲未 flush + 快速停服 | 探针写文件逐行落盘 |
| E-B1-7 | extractWorldgenDir 失败 | resources 布局不符 | 非默认参数每轮显式带 |
| E-B1-8 | COLPROF 空跑 | 无 driver 不生成 chunk | 零输出先查驱动条件 |
| E-B1-9 | H1 环1 整环证伪 | 未验证标注当公理继承（id→方块映射从未核实） | raw id 首次解释前先建映射；标注三查 |
| E-B1-10 | lavaAudit v1 指标盲区 | 指标设计与判别目标脱节 | 指标先从「结论所需最小判别证据」反推 |
| E-B1-11 | grep 零命中假警 | 行首锚 vs 日志前缀 | 零命中先打印一行原文核格式 |
