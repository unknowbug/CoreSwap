# discovered/compiler-idioms — 语言/编译器惯用法（跨版本通用）

> 从 versions/1.20.1/docs/ 各篇与排查中提炼的可复用模式。写入格式见 knowledge/INDEX.md。

## 发现 #1: Java 整数除法/取模的负坐标语义（floorDiv/floorMod）

**发现时间:** 2026-08-08
**发现者:** worker（负坐标排查）
**来源定位:** MC 1.20.1 源码（负坐标区块定位 / est 4 角插值）
**置信度:** confirmed（用户拍板，-288/3200/8576 逐位对齐）
**module:** re-code

### 观察
Java `Math.floorDiv / Math.floorMod` 与 C++ `/ %`（截断除法）在负坐标下结果不同；`est` 4 角插值、区块偏移、`x * 3129871` 类 int 乘法溢出按补码计算。

### 证据
- `floorDiv(-1, 2) = -1`（C++ `-1/2 = 0`）；`floorMod(-1, 2) = 1`
- 负坐标区块 `(x >> 4)` 与 `Math.floorDiv(x, 16)` 不等价（x 为负时）

### 如何利用
- C++ 还原时用 `floorDiv/floorMod` 等价式：`(a >= 0) ? (a/b) : -((-a + b - 1)/b)`（注意 a/b 除法语义）
- int 乘法溢出用 uint32 计算后转 int32（补码）
- **逐位验证点清单**（AGENTS.md 二、易错点）：负坐标 floorDiv、`x * 3129871` 补码、浮点精度、est 4 角插值、aquifer 邻居随机偏移（split 种子）

## 发现 #2: Java 数学函数语义（MathHelper）

**发现时间:** 2026-08-08
**发现者:** worker（density 对齐）
**来源定位:** MC 源码 MathHelper
**置信度:** confirmed
**module:** re-code

### 观察
`MathHelper.lerp(delta, a, b) = a + (b - a) * delta`（标准线性插值，三线性可交换顺序）；`clamp` 双边界。

### 证据
- C++ 三线性手算需严格按 `d00=lerp(fx,c000,c100); d10=lerp(fx,c010,c110); d01=lerp(fx,c001,c101); d11=lerp(fx,c011,c111); d0=lerp(fy,d00,d10); d1=lerp(fy,d01,d11); rr=lerp(fz,d0,d1)` 顺序（fy 作用于 d00/d10 而非 d00/d01——本 session 手算踩过 fy 错位的坑，差 2 倍）

### 如何利用
- 插值手算/实现前先核对角点索引（c010/c110 是 y 上层的 x 两角点）
- float 精度：Java 内部 double，C++ 用 double；float 中间值（如 684.412f）会造成微差

## 发现 #3: cache 节点的 key 语义（block 级 vs chunk 级）

**发现时间:** 2026-08-08
**发现者:** worker（Cache2DDF 修复）
**来源定位:** MC 源码 ChunkPos.toLong(blockX, blockZ)
**置信度:** confirmed（块状 bug 主因，修复后对齐率大幅提升）
**module:** re-code

### 观察
`cache_2d` / `cache_once` 类节点：Java 的缓存 key 是 **block 级** `ChunkPos.toLong(blockX, blockZ)`，不是 chunk 级。C++ 曾误用 chunk 级 key → 列缓存跨 chunk 错位 → 块状 bug。

### 证据
- 修复后：20000 区域 99.4115% → 99.985%；8576 块状消失

### 如何利用
- 实现任何 cache 节点先确认 Java 语义（key 粒度/生命周期），再写 C++ 缓存
- 多线程下 cache 需 thread_local 或原子（MSVC 铁律：MinGW thread_local 曾退化）

## 发现 #4: MSVC long = 32 位（Windows LLP64）——`long bestCost = INT64_MAX` 截断为 -1

**发现时间:** 2026-08-08
**发现者:** worker（SearchTree 移植 3 版迭代）
**来源定位:** Windows LLP64 ABI（long 4 字节 / long long 8 字节；Linux LP64 下 long 8 字节）+ MultiNoiseUtil.SearchTree 移植
**置信度:** confirmed（crash 复现 + 改 long long 后修复）
**module:** re-code / swe

### 观察
`long bestCost = INT64_MAX` 在 MSVC（Windows LLP64）下 `long` 是 **32 位**，`INT64_MAX` 截断为 -1 → 后续 `bestCost > cost` 恒 false → 分支选择逻辑全错（bestBatches 恒空 → makeBranch throw → 崩溃）。Java `long` / Linux 代码里 long 常被当 64 位，直接搬到 MSVC 就会踩。

### 证据
- SearchTree 移植 v1 空指针崩溃、v2 异常崩溃（makeBranch throw），v3 定位 `long bestCost = INT64_MAX` 为根因
- 改 `long long`（64 位）后 (812,73,-337) forest→badlands 修复、8576 24→22 mismatch

### 如何利用
- **MSVC 下 64 位整数一律用 `long long` / `int64_t`，不用 `long`**（`int64_t` 在 MSVC 就是 long long）
- 移植 Java long / Linux 源码时 grep `INT64_MAX`、`INT64_MIN`、`0x7FFFFFFFFFFFFFFF` 赋值给 long 的代码
- Java `long` → C++ `int64_t`/`long long`（不是 long）

## 发现 #5: AddVectoredExceptionHandler（VEH）在 JVM 进程（jvm.dll 已加载）不可用

**发现时间:** 2026-08-08 晚
**发现者:** worker（spawn 崩溃 DEBUG）
**来源定位:** worldgen_api.cpp wg_create 崩溃日志 handler（AddVectoredExceptionHandler + StackWalk64）
**置信度:** confirmed（对照实验：注释 installCrashHandler → 不崩；修复后 >5 分钟稳定运行）
**module:** re-code / swe（Windows 原生 + JVM 混合进程）

### 观察
`AddVectoredExceptionHandler` 注册的 VEH 在**所有 SEH 之前执行**（异常处理链第一顺位）。JVM 大量用「预期异常」做正常控制流：JIT null-check、GC guard page、写屏障都是 SEH 异常。VEH 里若做重活（StackWalk64/打印/内存扫描）会破坏 JVM 堆/栈 → 连锁崩溃。

### 证据
- CoreSwap 崩溃日志 handler（VEH + StackWalk64）在 gradle runServer（JVM 进程）下 spawn 预生成后 ~2s native 崩溃：崩溃线程 = JVM "Server thread"、RIP 指向 JVM metadata、RAX 是 Java Object[] oop、栈被 0xDEADDEAF 覆盖、jvm.dll 连锁崩溃
- 二分链逐步排除：线程数（❌）→ 攒批（❌）→ fillChunk 计数（✅ 0 次调用，与 C++ 生成无关）→ wg_create 阶段（✅ 全 OK）→ 对照实验 BenchMod active=replace 不崩 → **注释 installCrashHandler 不崩** → 根因 = VEH
- 独立原生进程（block_probe/got_export）不崩——无 JVM 异常模式；用户机器 D:\MC 的 0x34001 崩溃 = 同根因（客户端 C++ 接管 + VEH）

### 如何利用
- **JVM 进程（jvm.dll 已加载）不装 VEH 崩溃日志 handler**；检测 `GetModuleHandleA("jvm.dll")` 非空则跳过
- JVM 侧崩溃交给 JVM 自带 hs_err（含 native 栈 dll 偏移）兜底——仍满足「崩溃可定位」
- 独立原生进程可安全使用 VEH + StackWalk64

## 发现 #6: 跨层 id 域错位（raw block id vs global state id）——Minecraft mod 写入存档的判据

- **发现时间**：2026-09-01；**发现者**：worker（multiworld-port M16）；**来源定位**：`.investigations/multiworld-port/multiworld-errors.md` M16 + snapshot-CppBridge-m16fix.java；**置信度**：candidate（闭环判据实锤；confirmed 待用户拍板）；**module**：re-code / swe（JNI/FFI 跨语言边界）。
- **观察**：MC 存在 raw block id（`getRawId`，注册表序）与 global state id（`STATE_IDS`，blockstate 展平序）两套域；跨层传 id 中间某跳换域而无声明 → 低 id 区（经典块）恰好命中、高 id 区（nether/新块）全面错位，信号是「不相干方块成片」而非崩溃。
- **证据**：nether 存档 oak_leaves×3150+sapling+note_block、重生成精确复现；3×3 biome dump 纯 nether 排除 feature 污染、Status 包装层门控排除调度；修复闭环（改 raw id 解码后存档级 Partial 验证 nether 82.16% / overworld 87.75%）。
- **如何利用**：
  - 每跳「域声明」：JNI/FFI 传 id 每一跳显式声明域；参照导出域与写入解码域同源核对（seed/坐标三查的 id 域版本）。
  - **判据**：① 块名直方图签名（橡树叶+多 sapling+note_block = 错位解码签名，非 feature 签名——feature 不会以 note_block 成片混入）；② 同代码重生成数量精确复现 = 写入层确定性错误，排除下游随机性；③ 排查顺序：写入路径 id 域 → 下游阶段上下文 → 判定算法。
  - 交叉引用：接管类 mod 下游阶段审计清单见 workflow-patterns 发现 #8（M16 本例根因不是它，#8 仍是有效检查清单）。

## 发现 #7: 锚坐标换算 off-by-one（below_top 类顶块相对锚 = min_y+height-1-v）——数据驱动规则锚公式的维度覆盖判据

**发现时间:** 260901-02
**发现者:** worker（multiworld-port M17）
**来源定位:** `.investigations/multiworld-port/multiworld-errors.md` M17 + `.investigations/multiworld-port/m17-bedrock-band-summary.md`（修复位置 `WorldgenRust/src/surface_rules.rs` L944）
**置信度:** candidate（修复后逐位吻合：van_only=rust_only=0；TOTAL 96.0568%→96.4428%，同工具同区域同 seed 前后可比；confirmed 待用户拍板）
**module:** re-code / swe（数据驱动规则解析 / 坐标域换算）

### 观察
MC worldgen 的相对锚（`above_bottom(N)` / `below_top(N)`）换算：**顶块 y = min_y+height-1（闭区间端点），不是 min_y+height**——`below_top(v)` 正确公式为 `min_y + height - 1 - v`。写漏 `-1` 会使整条 vertical_gradient 判定的 y 基准整体平移一层，随机带概率层全部错位。此类 bug 会被「全绝对锚的维度」长期掩盖（overworld deepslate 用 absolute 锚，`below_top` 路径从未被执行），直到第一个依赖相对锚的维度（nether bedrock roof）才暴露。

### 证据
- vanilla nether bedrock roof 概率序列（4×4@0,0 seed -8248，诊断 bin `nether_bedrock_band.rs` per-y 计数）：[123]=0.2、[124]=0.4、[125]=0.6、[126]=0.8、[127]=1.0；Rust 修复前同形状**整体 +1 层**（[123]=0…[127]=0.8）——确定性平移签名。
- 修复（`min_y+height-v` → `min_y+height-1-v`）后逐位吻合（每层 van_only=rust_only=0）→ splitter 种子派生正确，纯锚换算 bug；全量回归 TOTAL 96.0568%→96.4428%、y96..127 94.0→97.12%。

### 如何利用
- **公式**：`above_bottom(v) = min_y + v`；`below_top(v) = min_y + height - 1 - v`——凡「从顶/底数第 N 层」的换算，先把端点语义（inclusive/exclusive）与 Java 源码核对再写（与 M3「锚 height 用逻辑生成高度不混用 world_height」同族：锚换算两个独立坑 = 高度基准 + 端点 off-by-one）。
- **签名**：per-y 概率/计数序列形状一致但整体平移 = 锚 y 基准错位；形状破坏才是随机流/种子错——诊断 bin 按层统计即可单轮定位。
- **覆盖判据**：数据驱动 JSON 规则的每个锚类型（absolute/above_bottom/below_top）至少要有一条**非绝对锚维度**的实测用例——单维度（全绝对锚）验证通过 ≠ 换算正确，只是未被覆盖。

## 发现 #8: JSON 布尔字段经 as_f64 读取恒 false——分型标量 API 下的「静默语义腐蚀」签名

- **发现时间**：260902-03；**发现者**：core.worker 草稿（soul-v4v5 课题 .b2-soul fan-out 裁决）+ 主会话应用；**来源定位**：`.artifacts/.b2-soul/v4-eval-conflict.md` + `.investigations/soul-v4v5/v4-fix-verification.md`（修复位置 `WorldgenRust/src/surface_rules.rs` parse_surface_cond 三处 + `parse_bool_field`）；**置信度**：candidate（三级数据层证据实锤，confirmed 待用户拍板）；**module**：re-code / swe（数据驱动解析器 / 跨语言 JSON 语义）。
- **观察**：自定义 JSON 包装层若按标量分型提供 API（`as_f64` / `as_bool` / `as_str` 各只对同型返回 Some），则 `x.as_f64().map(|f| f != 0.0).unwrap_or(false)` 读布尔字段**恒得 false 且无任何告警**——不是兼容读取，是静默语义腐蚀。Java `GsonHelper.getAsBoolean(json, key, false)` 是 bool 优先/缺省 false 的类型感知读取，两端 API 语义不等价，直译即错。本例：surface_rule 三处布尔字段（add_surface_depth/add_stone_depth）恒 false → soul 分支条件 `sdb ≤ 1+0+surface_depth` 退化为 `sdb ≤ 1` → 分支该进未进穿透兜底，nether 存档对齐被压 2.20pp（94.42%→96.62% 修复）。
- **证据**：nether.json L293 `"add_surface_depth": true`（布尔）vs 解析产物树 dump 实测 `asd=false`（soul-tree-repro，8 处假阴性中 3 处为真阳性翻转）；定点 3260,1,3200（sdb=2, surface_depth=3）`2 ≤ 1+0+0`=false 复现 applied=256，修复后 `2 ≤ 1+0+3`=true → applied=258 与 V3 语义推演逐位一致；生产 180 点 dump netherrack 103→71；存档 94.4241%→96.6215%/96.5866%（seed B，4×4@3200,3208，存档口径）。
- **如何利用**：
  - **规则**：分型标量 API 下读布尔一律 `as_bool().or_else(|| as_f64().map(|f| f != 0.0)).unwrap_or(false)`（类型感知 + 数字 0/1 兼容 + 缺省 false），禁止「万能 as_f64 转 bool」；移植/翻译 Java 数据驱动解析器时，逐字段核对 Gson getXxx 的类型容忍面与目标语言 API 的分型行为是否等价。
  - **签名**：「条件永远不成立但无任何告警」+ 解析期零 WARN（读取成功返回 false，不是解析跳过）——凡「分支看起来存在却从不进入」先 dump 解析产物核对布尔字段；与发现 #7 同族（都是「单维度/单分支未覆盖即潜伏」的解析器坑）。
  - 交叉引用：对拍方法教训见 workflow-patterns 发现 #12（对拍解析产物而非 JSON 原文——本发现的假阴性正是被 #12 缺口掩盖的）。



## 发现 #9: 跨 session raw id 标注三查——未验证的 id→方块标注当公理继承，整条机制链作废（260902-07）

- **发现时间**：260902-07；**发现者**：core.worker 草稿（b1-downdrill H1 环1 证伪复盘）+ 主会话应用；**来源定位**：`.investigations/b1-downdrill/b1-errors.md` E-B1-9 + `facts-260902-06/07.md`；**置信度**：candidate（数据层实锤：LAUIDMAP 权威映射一轮推翻定案候选，judge PASS，confirmed 待用户拍板）；**module**：workflow / re-code（MC 注册表 / 交接纪律）。
- **观察**：MC 探针输出 raw block/state id（如 COLPROF `99|0->19319`）时，若首次出现未同时输出 id→方块映射，session 内凭数值直觉赋语义（「19319 大数≈新块≈lava」「5854≈netherrack」）后写入事实链，下一 session 即当公理直接续推——实测 Java STATE_IDS 权威映射：**19319=blackstone、5854=basalt**（lava=96、netherrack=5850），「熔岩海缺失」机制链环 1 整环证伪、修复方案作废。标注从「未验证解释」升格为「事实」只发生在文档传递里，不在数据里。
- **证据**：`[LAUIDMAP]`（探针启动时遍历 Registries.BLOCK/STATE_IDS 打印映射）vanilla 轮一轮实锤；LAVAAUDIT v2 全扫 11,443 公共列 air→lava 面向两侧均为零；COLPROF 10 列 diff 真相 = V 黑石底(y=99) vs C 玄武岩底(y=100~104)，两侧均实心。验证成本一轮，此前整条五环机制链与回归判据全部改写。
- **如何利用**：
  - **标注三查（与 seed 三查同级的开工检查项）**：① 探针输出的 raw id/枚举/魔法数**首次解释前必须先建立 id→语义映射**（探针打印 LAUIDMAP 类映射，禁止数值范围直觉命名）；② 标注跨 session 传递 MUST 带「已验证/未验证」标记，未验证标注续用前先做廉价独立验证（≤ 一轮，§16.3 宿主交接验证）；③ 机制链逐环追问「这一环的输入标注是谁验证的」——环 1 错则整链作废，越早核实越便宜。
  - **判据**：「机制链自洽 + 量级对得上」不能替代输入标注核实——自洽的链条建在错误标注上时全链同样自洽（本例黑石/玄武岩材质差同样能解释转换面漂移现象）。
## 发现 #10: Rust 半开区间 rev().step_by() 复刻 Java 含两端递减 for 循环的 off-by-one

- **发现时间**：260903-13；**发现者**：core.worker 草稿（lossless-accel off-scan+cornerfix 课题）+ 主会话应用；**来源定位**：commit 3e2e67d + `.artifacts/lossless-accel/off-scan-cornerfix-verdict-260903-13.md` + `.investigations/lossless-accel/review-offscan-cornerfix-260903-13.md`（Rust 侧 `WorldgenRust` est 扫描；Java 侧 forge official sources `NoiseChunk.java:174` `computePreliminarySurfaceLevel`）；**置信度**：confirmed（修复后两臂四臂 hash 完全一致 f2b1a3932c6e589e + Java 角列 256/256 0 diff，judge PASS，260903-13 用户拍板）；**module**：re-code / swe（跨语言循环移植）。

### 观察

Java `for(l=top; l>=bottom; l-=step)` 是**含两端**的递减扫描；移植 Rust 时写成 `(bottom..top_exclusive).rev().step_by(step)` 会引入**两个独立的错位**：① 半开区间上端使 rev 首点 = top−1（本例 319 vs Java 320）；② 下端 exclusive 使下界端点语义差（本例 Java 扫到 −64 含端）。首点差与下界包含性是**两个独立参数**，只对齐其一修不完整。

### 证据

- 本例签名：修复前 off 臂 est 角列对 Java **恒差 −1**（64/64 全偏、delta 恒 −1，含 c0 原点角——规整性系统偏移而非随机差）；敏感角 (201,200) 值 55 vs Java 56。
- 修复（扫描对齐「首点值 + 下界包含性」）后：两臂四臂 hash 完全一致（`f2b1a3932c6e589e`）；Java est 角列 off/shared 各 256/256 一致 0 diff。

### 如何利用

- **判据**：跨语言移植递减扫描循环时，必须显式对齐**「首点值」+「下界包含性」两个独立参数**，逐一与 Java 源码核对（`l>=bottom` 含端 vs Rust `..` 半开），禁止凭「看起来等价」直译。
- **签名**：结果相对参照**恒差固定小量（如 −1）且全样本规整偏移** = 扫描/索引 off-by-one 类错位，优先核对循环端点语义，不是精度/随机性问题。
- 等价复刻形态：Java `for(l=top; l>=bottom; l-=step)` → Rust `(bottom..=top).rev().step_by(step)`（含端 RangeInclusive），并核对 `top−bottom` 可被 step 整除时的末点行为。
- 交叉引用：workflow-patterns #25（静态调研/直译结论失真——本例 +15 角参数即其第三例实例）；compiler-idioms 发现 #7（锚换算端点 off-by-one 同族：端点语义 inclusive/exclusive 是跨语言移植的第一易错点）。



