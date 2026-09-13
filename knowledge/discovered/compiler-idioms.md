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




## 发现 #11: 诊断门控 flag 在初始化器内唯一置位——「先查 flag 才 init」鸡生蛋死锁（260904-09）

- **现象**：WG_AQDUMP 门控零输出，stderr 连 `[AQDUMP] enabled` 行都没有——门控从未激活，且无任何报错。
- **根因**：worker 草稿把唯一置位点写在初始化器里：`aqdump_hit` 先查 `AQDUMP_ON`（false）才初始化 points/激活门控——flag 永远 false，初始化永远不发生。
- **定位**：stderr 无 enabled 行 → 顺门控生命周期静态走查首次调用路径，发现置位点在死分支内。
- **修复**：改为 env **存在性** `OnceLock` 先判（env 存在 → 门控激活），与初始化解耦。
- **教训**：诊断门控的设计模式——**激活判据（env 存在性）与数据初始化必须分离，激活判据不得依赖被它门控的初始化器**；上机前静态走一遍「首次调用路径」（谁第一个读 flag、flag 在哪置位、可达吗）。与「死参数制造假判别」（workflow-patterns #20）同族：门控没生效 ≠ 机制无差异。

## 发现 #12: mixin 包禁止任何非 mixin 类（含 static nested）——IllegalClassLoadError 的直接判据（260904-09）

- **现象**：Java 启动即 `IllegalClassLoadError: WgCap cannot be referenced directly`。
- **根因**：辅助类 `WgCap` 放在 mixin 包（wg.bench.mixin.*）内——mixin transformer 对包内类做字节码织入处理，非 mixin 目标类（含 static nested）被引用即类加载失败。
- **定位**：混淆栈直接指向 mixin transformer，异常类名即包内类名。
- **修复**：`WgCap` 移到 `wg.bench.AquiferDumpProbe` 公有嵌套类（非 mixin 包）。
- **教训**：mixin 包内**只放 mixin 配置声明的 transformer 类**；任何辅助/工具/数据类（含 static nested、常量类）放普通包——mixin 包边界 = 字节码处理边界，不是普通 Java 包可见性边界。

## 发现 #13: docs 记载的判定口径与一手源码失准——以 docs 口径为修复依据前先 P0 一手源码核对（简条）
- **发现时间**：260904-10　**置信度**：candidate
- **观察**：docs/06 L62/L94 记 surface default 判定「==stone」，一手 SurfaceBuilder.java L181-183 实为「非空非流体」。.b4 候选曾以 docs 口径推出「需改码」，P0 核对证伪——Rust/C++ 本就与一手一致，无需改。
- **判据**：docs/主题篇是二手转述，作为**修复依据**（要动代码）前必须对一手源码核对；作为排查线索使用则无需——判据 =「这个口径要花钱（改码）吗？要则一手核对」。
- **处置**：docs/06 追加修正小节（不覆盖原文）；本条仅记录判据。

## 发现 #14: JSON opacity:-1 经 parse_u8_field 静默 clamp 成 0——负值经无符号 parse 的语义有损姊妹案例（260905-03）

- **发现时间**：260905-03；**发现者**：core.worker 草稿（light-opt 课题顺藤）；**来源定位**：`worldgen-core/src/light/mod.rs:121-125`（18 个 opacity:-1 条目实测 clamp）；**置信度**：candidate（代码位置 + 条目数实证；与光照残差无关，语义有损待修）；**module**：re-code / 数据驱动解析。

### 观察/根因
light_data.json 中 18 个条目 opacity 为 -1（语义：不透明度哨兵/负值语义），经 `parse_u8_field` 的 u8 解析路径被静默 clamp 为 0——解析零告警，但语义有损（-1 的语义与 0 完全不同）。与发现 #8（JSON 布尔字段经 as_f64 恒 false）同家族：**「宽收窄/带符号转无符号的标量 parse API 静默腐蚀语义」**——#8 是布尔收窄，本条是符号性收窄，共同点是解析成功返回值 + 零告警。

### 如何利用
- **签名**：「条件/数值看起来解析成功但下游行为异常、日志零 WARN」→ dump 解析产物核对原始 JSON 值与解析后值逐字段差（#12 姿势：对拍解析产物而非输入原文）。
- **规则**：负值语义字段禁止直接 u8 parse——先 parse i64 再显式映射负值语义（clamp 前必须有「负值是否合法」的显式判定）。
- 交叉引用：#8（布尔恒 false，同族第一例）。

## 发现 #15: 位打包标量域的「部分位判零」陷阱——blocks9 ABI 快路径只判 id 位，吞掉 id=0+luminance>0 的合法光源（260905-04）

- **现象**：光照优化 round2 air 快路径首轮实现只判 `id 位 == 0` 即跳过查表 + col_max 写；合成 golden 用例（id=0 + luminance=15，即打包值 `15<<24`）立即 FAIL——该值是合法光源（ABI = raw id 低 24 位 | luminance 高 8 位），却被快路径当空气吞掉，光照输出静默错。
- **根因**：对**位打包标量**做类别判断时，把「某字段为零」误当「整个字为零」——`id==0` 不蕴含 `v==0`，高位还可能携带其他字段。字段判断与全字判断的语义混淆，且静默（不崩溃、只错输出）。
- **定位**：合成 golden 逐位对照门（workflow-patterns #47）一轮即捕获——合成用例专门构造「id=0 + luminance>0」边界输入，真实数据用例反而可能覆盖不到。
- **修复**：快路径判据改为**判全字 `v==0`**（且 `table[0]==(0,0)`），任何非零位都回退完整查表路径。修复后 4 用例 golden 全等 PASS。
- **教训/判据**：① 对位打包值做类别快路径，**必须判全字为零，或显式声明并验证「其余位必为零」的不变量**。② 与 #8（布尔经 as_f64 恒 false）、#14（负值经 u8 parse 静默 clamp）同家族：**「标量跨域/跨表示的静默语义腐蚀」**——共同点 = 解析/判断「成功」且零告警。③ 合成边界用例（构造字段间取值组合）是 golden 用例集的必备成分，纯真实数据可能永远不触发该分支。
- 交叉引用：#8、#14（同族）。

## 发现 #16: JSON 字段「类型形态枚举不全」静默腐蚀——feature 字段三形态只支持一种，内联对象被吞成空 id（260905-12）

- **发现时间**：260905-12；**发现者**：core.worker 草稿（feature-parity patch_grass 内联修复课题）+ 主会话应用；**来源定位**：`.investigations/feature-parity/260905-12-patch-grass-inline-fix.md` + `worldgen-core/src/placement.rs`（PlacedFeature::parse_inline）/ `feature_loader.rs`（generate_configured/generate_nested）git diff；**置信度**：candidate（修复前后基线对照实锤，judge/confirmed 待走）；**module**：re-code / swe（数据驱动解析器 / 跨语言 JSON 语义）。

### 观察/根因
patch_grass 系 **24 个内联点位**（18 个 patch_*.json + 6 个 flower*.json random_patch/flower 系 configured feature，patch_berry_bush/patch_cactus/patch_grass_jungle/patch_large_fern 等）的 `feature` 字段是**内联 configured feature 对象**（Java 语义 = `Holder.direct`，无 id 直发），不是 id 字符串。`parse_inline` 只按字符串形态解析（`as_str().unwrap_or("")`）→ 内联对象静默变**空串 id** → 运行时 `generate_nested("")` → placed/configured cache 双 miss → 空 id 告警 ×N（该参照区 miss=8766）。解析期零告警——与 #8（布尔 as_f64 恒 false）、#14（负值 parse_u8 clamp）同家族：**JSON 字段类型形态枚举不全 = 静默语义腐蚀家族**。#8 是布尔收窄、#14 是符号性收窄，本条是「字段值可能是复合对象而 parse 只支持标量形态」的**结构性**收窄；共同点 = 解析「成功」返回 + 零告警，语义已损失。

### 证据
- 修复前 features_probe release × recheck 参照（6×6）miss=8766 / match 94.99%；修复后 miss=0 / match 94.78%（stash/pop 单变量基线对照，同 seed 同参照同二进制载体）。
- 修复 = `PlacedFeature` 增 `inline_configured: Option<Box<ConfiguredFeature>>`（Box 断 CF→PlacedFeature→CF 递归环）；parse_inline 对 object 形态直接 `ConfiguredFeature::parse("", obj)` 持有实体；generate 侧 inline Some 时直发、优先于 id 查 cache（对齐 Java Holder.direct 直发非查表语义）。

### 如何利用
- **判据（形态枚举全覆盖）**：数据驱动解析器对「id 引用型」字段（placement/configured 的 `feature`、`config` 内嵌等）必须显式枚举**全部三形态**——① id 字符串（查表）② 内联对象（直接解析持有，Holder.direct 语义）③ 缺失（才允许告警/缺省）。只写 `as_str().unwrap_or("")` 一种读法 = 隐含假设「永远是字符串」，移植 Java 时先核对 `Holder` 的 direct/reference 双形态。
- **判据（空 id = 解析层缺陷签名）**：运行时 miss 告警必须带 id 内容检查——**空 id/空串 key 的 miss 是解析层缺陷签名，不是运行时层问题**（运行时层缺 key 应是有名 id）；见空串 miss 先回解析层 dump 产物，不查 cache 实现。
- 交叉引用：#8（布尔恒 false，同族第一例）、#14（负值 clamp，同族第二例）、workflow-patterns #12（对拍解析产物而非 JSON 原文——本例假阴性同样被该缺口掩盖）。


## 发现 #17: Fabric 反射字符串不重映射——双名解析三规则（BUG-001 修复模式沉淀）（260905-13）

- **时间/置信度/module**：260905-13；candidate（mappings.tiny build.10 直查实证 + dev 编译绿；**生产 intermediary 运行时未实测，Degraded 声明**——名字对来自权威 mappings，机制层为 Fabric 铁律）；compiler-idioms / Fabric·mixin 反射域（#12 mixin 包纪律的姊妹面）。

### 根因（机制）
Fabric tiny-remapper 只重映射 .class 常量池里的**引用**，**字符串字面量不映射**——`getDeclaredField("pieceIterator")` 类反射名字开发环境（Yarn）命中、生产环境（intermediary）必 NoSuchFieldException。字段、方法、**类名三层的反射字符串全部中招**。

### 三规则（缺一不可）
1. **字段/方法名都带双名**：`(yarnName, intermediaryName)` 顺序 try——如 `("pieceIterator","field_28744")`、`("box","comp_682")`、`("getMinX","method_35415")`。**双名对照表从 mappings.tiny 直查，禁止凭记忆**（本例 yarn-1.20.1+build.10：field_28744/28745、comp_682/683/684、method_35415..35420、method_16609..16611）。
2. **禁止 `Class.forName(Yarn名)`**：生产 intermediary 下**类名也不同**（`class_5817$class_7301` 等），双名救不了 forName——类引用必须编译期直引（remapper 重写）或运行时 `getClass()`（方法查找用实际类即可）。
3. **静态缓存 + 日志节流**：反射结果存 static（Field volatile / Method ConcurrentHashMap），失败日志「同 key 首次全打 + 每 1000 次汇总 1 条」——反射热路径首错全打刷爆日志，全不打则静默降级（BUG-001 事故形态：降级路径必须留可观测痕迹）。

### 如何利用
任何 Fabric/mod 环境反射访问 mapped 类：按三规则套模板（wgField/wgMethod 双 try + 缓存 + 节流），写后必须在**生产（intermediary）运行时**验证一次——开发环境全绿不构成证据。

### 证据
`runtime/1.20.1/java/src/main/java/wg/bench/CppBridge.java:149-243`（BUG-001 修复实现）；`.investigations/jungle-l/judge-verdict-260905-13.md`（judge APPROVE-WITH-CONDITIONS，修复代码保留）。




## 发现 #18: 复刻 Java long 算术一律 wrapping_*——Rust debug 溢出 panic 是免费的 Java 回绕审计器（260906-04）

- **发现时间**：260906-04；**置信度**：candidate（panic 行号定位 + 修复后 debug/release 同语义实证）；**module**：re-code / Rust↔Java 算术语义。

### 观察/根因
fill 大坐标 chunk（100,100）时 panic `attempt to multiply with overflow` @ `chunkrandom.rs` 的 `set_population_seed`/`set_carver_seed`。Java long 乘法天然回绕（JLS 语义，静默取低位）；Rust debug profile 算术溢出即 panic。`next_long()` 可取满 i64 域，`chunk_x * l` 超域是必然事件——此前 overworld/nether 未炸只因测试 chunk 坐标小。release profile 静默回绕，语义碰巧与 Java 一致，缺陷被掩盖。

### 修复
三处 `*`/`+` 改 `wrapping_mul`/`wrapping_add`——位模式与 Java 完全一致，非行为变更。

### 如何利用
1. **判据（MUST）**：复刻 Java long/int 算术一律 `wrapping_*`——使 debug/release 同语义，不依赖 profile 巧合。判据适用面：种子派生、hash 组合、坐标×大常量（`x * 3129871` 类，AGENTS.md 易错点清单既有条目）。
2. **正向利用（判错经验）**：新维度大坐标/新 seed 派生链首跑**必须先过 debug profile**——debug 溢出断言等价于免费的「Java 回绕点审计器」，逐个 panic 点就是一处需要 wrapping 的位置；release 只会静默吞掉。
3. 交叉引用：compiler-idioms #4（MSVC long=32 位——同一「Java long 语义在系统语言侧不成立」家族的 C++ 面）。

### 证据
`.investigations/end-takeover/260906-04-errors.md` E2；`chunkrandom.rs` set_population_seed/set_carver_seed。

---

## 发现 #19: SimplexNoiseSampler 移植要点——float/double 域边界逐行对齐 + nextDouble 乘法语义（260906-04）

- **发现时间**：260906-04；**置信度**：candidate（静态源码审查 + 同 seed Java↔Rust 置换表/origin/simplex/erosion 全等对拍实证，judge 三源核对通过）；**module**：re-code / 浮点域移植。

### 观察/根因
EndIslands 密度函数是本工程首个 SimplexNoiseSampler 移植点，其数值域构成三层混合，任何一层提前收窄/扩张都会引入与 Java 不同的舍入：
1. **构造消费序列**：`SimplexNoiseSampler.java` L33-36 —— originX/Y/Z = `random.nextDouble() * 256.0`。nextDouble 的语义 = 两个 next 拼接后 **× `1.110223E-16F` float 字面量**（BaseRandom.java L51-56：`long * float` 提升为 float 域相乘再 widening 回 double——Rust 侧 `(l as f32) * 1.110223E-16f32` 逐域对齐，f64 乘法反而错；本工程同 seed 对拍全等实证）。
2. **采样内层是 float 域**：EndIslands `sample` 中 `f = 100.0F - sqrt(x*x + z*z) * 8.0F`、`g = (|o|*3439.0F + |p|*147.0F) % 13.0F + 9.0F`、clamp `[-100.0F, 80.0F]`、邻域阈值 `-0.9F`——全部 float 字面量（DensityFunctionTypes.java L637-661）。Rust 对应 `f32` 逐行对齐，禁止「顺手用 f64 更精确」——精度更高反而是错的。
3. **输出归一是 double 域**：最终 `(sample(...) - 8.0) / 128.0`（L665）与阈值判定（0.25 / -0.0625 / -0.21875，TheEndBiomeSource.java L73-84）发生在 double 域。

### 如何利用
1. **判据**：移植带浮点的 Java 算法，先画「域地图」——逐常量标注 F 后缀（float 域）与无后缀（double 域），Rust 侧 `f32`/`f64` 严格随行；字面量后缀就是域边界的权威标注。nextDouble 的乘数 `1.110223E-16F` 即典型：F 后缀使整个乘法落 float 域。
2. **int 除法注意**：`x/2`、`x%2` 是 Java 截断向零除法/取模（i 域），与负坐标 floorDiv/floorMod 不同域（compiler-idioms #1），本函数内两族并存勿混。
3. **对拍分层隔离**（本轮实证有效）：先对置换表/origin 值（构造域），再对单点 simplex 采样值（采样域），最后对 erosion/biome 阈值边界（应用域）——精度问题可定位到层，不逐位盲对。
4. 交叉引用：workflow-patterns #59（对拍必须数值化比较——本轮对拍方法前提）；#40 家族的「域混淆」同型。

### 证据
`02-end-biome-rules.md` §2.2/§2.4（DensityFunctionTypes.java L626-682、SimplexNoiseSampler.java L33-36、NoiseConfig.java L105 种子直传无 split）；对拍记录：review-002-final.md ①②（seed 7691421705105351955 / 12345 双侧全等）。


## 发现 #20: MC-239059——BaseRandom.nextLong = (next(32)<<32) + next(32)，低位 j 有符号 int 符号扩展相加；漏符号扩展的签名 = 高位似近、低位全错（260906-06）；candidate

- **发现时间**：260906-06；**置信度**：candidate（一手 yarn 源码引证 + Xoroshiro 复刻逐位命中实证）；**module**：re-code / Java 整数语义复刻。
- **来源定位**：BaseRandom.java:33（nextLong 默认实现）+ ChunkRandom.java:31（next(bits) 基类分流）；实证 .tmp/jungle-l-260906/pop_feature_xoroshiro.py。
- **观察/根因**：`BaseRandom.nextLong() = ((long)next(32) << 32) + next(32)`——第二个 `next(32)` 的返回值是 **int**，`+` 前经符号扩展提升为 long：j 为负（最高位 1）时高 32 位全 1。复刻时若按无符号拼接（`(h<<32) | j`），高位方程因 Xoroshiro 混淆仍可能近似命中，但低位恒错。
- **如何利用**：
  1. 复刻任何走 `nextLong()` 的 MC 随机链（含 setPopulationSeed）时：`((h as i64) << 32).wrapping_add(j as i64)`——j 必须 i64 符号扩展，不是 u32 拼接；与 #18（wrapping_* 回绕审计）同族配套。
  2. **判错签名**：复算结果「高位似近、低位全错」→ 首查低位符号扩展（MC-239059），不查随机算法本体。本轮首版复刻即此签名，一轮定位。
  3. 交叉引用：#19（nextDouble 乘法落 float 域）、workflow-patterns #61（双调用点——逐位吻合前先核调用点覆盖面）。
- **证据**：BaseRandom.java:33 引证 + 逐位命中记录 .investigations/jungle-l/260906-06-errors.md E11 定位段。

## 发现 #21: 模拟 mod 方块注册的探针 raw id 必须取 vanilla 表之后（Registries.BLOCK.size()）——vanilla 域 id 是 blocks.json 既有属主，「冲突拒绝正确工作」≠「探针有效」（260907-08）

- **发现时间**：260907-08；**发现者**：core.worker subagent 草稿（源材料：.artifacts/jni-blockid-fix-260907-08.md + .tmp/blockreg-smoke-260907-08.log）；**置信度**：candidate（实机冒烟 round1 拒绝 + round4 对位成功双实证）；**module**：re-code / 跨层 id 域（「跨层 id 域错位」条目家族）。
- **来源定位**：worldgen-core/src/blocks.rs `register_with_id` 冲突拒绝语义；runtime/1.20.1/java CppBridge.java registerModBlocks 探针；冒烟日志 .tmp/blockreg-smoke-260907-08.log（round1）vs blockreg-smoke4-260907-08.log（round4）。
- **观察/根因**：round1 用真实 vanilla 块 calcite（java_raw=910）模拟 mod 方块注册 → Rust 侧按设计拒绝（-1，910 已属 blocks.json `minecraft:calcite`）。拒绝本身是冲突拒绝语义正确工作的证据，但**探针设计缺陷**：真实 mod 的 raw id 空间在 vanilla 注册表之后，vanilla 域的每个 id 都已被 blocks.json 占用——拿 vanilla id 冒充 mod id 必然撞属主。
- **如何利用（判据）**：
  1. 模拟 mod 方块注册的 raw id MUST 取 `Registries.BLOCK.size()`（本轮 1.20.1 = 1003），即 vanilla 表之后第一个空位——这才是真实 mod 首块的 raw id 位置。
  2. **判错签名**：「显式 id 注册返回 -1」先分两面——被 blocks.json 既有属主拒绝 = 探针位置选错（假失败）；未被拒绝且 id 合理却未注册成功 = 实现问题。冲突拒绝测试须用已知被占 id 显式构造，冒烟探针须用 vanilla 表后位置，两者不能混用。
  3. round1 的「正确拒绝」不是废轮——它顺带实证了冲突拒绝语义；失败轮结论先做「按设计工作 / 实现缺陷 / 探针缺陷」三分再定性。
- **证据**：round1 日志 `[BLOCKS-REG] testmod:aligned_probe java_raw=910 rust_id=-1 writeback=minecraft:calcite`（拒绝 + 属主回写即铁证）；round4 `java_raw=1003 rust_id(overworld=1003 nether=1003 end=1003)` 对位 PASS。

## 发现 #22: 跨语言 id 域可在注册时对齐时，注册时同域化优于运行时映射表——映射表 = 第二真相源（260907-08）

- **发现时间**：260907-08；**发现者**：core.worker subagent 草稿（源材料：.artifacts/jni-blockid-fix-260907-08.md）；**置信度**：candidate（cargo test 7/7 + runServer round4 显式 id 对齐实机 PASS；「映射表必然更差」未做反向对照，泛化边界见如何利用）；**module**：re-code / 跨层 id 域设计（#21 的设计侧姊妹条）。
- **来源定位**：worldgen-core/src/blocks.rs `register_with_id` / api.rs `wg_register_block_id(handle, name, java_raw_id)`；本工作块候选 B 的方案取舍（显式 id 注册取代 Java↔Rust id 映射表）。
- **观察/根因**：原问题 = Rust 内部动态分配 id 与 Java raw id 域错位，写回 `Registries.BLOCK.get(id)` 无对齐保证；映射表方案在运行时维护「rust_id ↔ java_raw」对应，但这引入**第二真相源**（两侧各有一套 id + 一张映射，映射本身成为新的出错面）。显式 id 注册把对齐动作前移到注册时：`wg_register_block_id` 直传 java_raw_id，Rust 内部 id 与 Java raw id **同域**，写回直查即对齐，映射层整个消除。
- **如何利用（判据）**：
  1. 跨语言/跨模块 id 传递的设计取舍顺序：先问「两侧 id 域能否在注册/边界构造时同域化」——能则显式对齐（单真相源），运行时映射表是退而求其次（第二真相源要付同步/漂移/调试成本）。
  2. 泛化边界：前提是「注册时可拿到权威侧 id 且域无碰撞」（本轮靠 #21 的 vanilla 表后位置判据保证）；id 域无法对齐（如两侧密度函数序号各自派生）时映射表仍是合法方案——判据是**消除映射机会优先**，不是「映射表禁用」。
  3. 显式 id 与动态分配并存时的语义配套：id 被占拒绝 / 越界拒绝 / 成功推进 next_id（显式 id 后动态分配不回退碰撞）——三条件缺一即埋新错位。
- **证据**：round4 `[BLOCKS-REG] testmod:aligned_probe java_raw=1003 rust_id(overworld=1003 nether=1003 end=1003)` 三句柄对齐 + `register_with_id_semantics` 测试（对齐/冲突拒绝/越界拒绝/next_id 推进）；.tmp/verification-260907-08.log。

## 发现 #23: MC Java Direction 存在两个不同的 HORIZONTAL 数组——随机方向选取必须取 Type.HORIZONTAL.facingArray，引用错数组即方向分布错位（260908-15）

- **发现时间**：260908-15；**发现者**：core.worker subagent 草稿（源材料：.investigations/mc-1216-port-260908-15/b5b6-scout.md + b5b6-worker-delivery.md）；**置信度**：candidate（一手 yarn 源码 1.21.6 Direction.java 全文引证）；**module**：re-code / Java API 惯用法复刻。
- **来源定位**：1.21.6 一手源码 net/minecraft/util/math/Direction.java：
  - Direction.HORIZONTAL 静态字段（Direction.java:49-52）：序 = N, W, S, E，用于遍历/迭代水平方向。
  - Direction.Type.HORIZONTAL.facingArray（Direction.java:601）：序 = N, E, S, W，仅供随机选取用——Direction.Type.HORIZONTAL.random(random) = Util.getRandom(facingArray) = array[nextInt(4)]（Util.java:863-865）。
- **观察/根因**：两数组名字都含 "HORIZONTAL"、成员相同仅顺序不同、一处是字段一处是 enum type 成员——按名字 grep 极易取错。顺序差（W/S/E ↔ E/S/W）意味着用迭代序数组做随机选取时 nextInt(4)→方向映射整体错位（0→N 相同，1/2/3 全错）。Rust 复刻侧：HORIZONTAL_FACING = [(0,0,-1),(1,0,0),(0,0,1),(-1,0,0)]（N,E,S,W 序，对应 facingArray），仅用于随机选取；遍历用途另建迭代序常量。
- **如何利用（判据）**：
  1. 用途判据：随机选取方向 → 取 facingArray（N,E,S,W）；遍历/迭代 → 用静态字段（N,W,S,E）。一句话：跟着调用方法走——.random(random) 背后是 facingArray，for-each 背后是静态字段。
  2. 「互相勘误」反模式：两数组注释（或复刻侧引用两数组的两处代码注释）相邻时，顺序不同不是谁写错了——勿把另一处的序当 typo 纠正到同一序上。本轮 judge 审查实例：tree.rs cocoa 注释 vs fallen_tree 注释各引不同数组，序不同是正确的。发现「两处 HORIZONTAL 序不一致」先查各自 Java 出处再定性。
  3. 判错签名：复刻某随机方向机制后，四方向统计分布系统性对不上（尤其两两互换形态）→ 首查引用的是哪个 HORIZONTAL 数组。
- **证据**：Direction.java:49-52 + Direction.java:601 + Util.java:863-865；落盘引证 .investigations/mc-1216-port-260908-15/b5b6-worker-delivery.md §2 idk-②；.artifacts/mc-1216-port-260908-15/b5b6-verdict-260908-15.md §3。

## 发现 #24 简记: MC 1.20.1 `PalettedContainer.swap()` 自带 `lock()/unlock()`（`LockHelper` **非可重入**）——`ChunkSection.lock()` 只与 `setBlockState(..., lock=false)`/`swapUnsafe` 配套（260910-06）

- **发现时间 / 发现者 / 置信度 / module**：260910-06；主会话（实测线程栈 dump + 一手调用链）；**candidate**（dump 决定性证据 + 调用链 file:line；vanilla 对照为源码直读）；compiler-idioms / Java（MC）API 惯用法与锁语义。
- **来源定位（调用链）**：`CppBridge` `writeChunk` → `ChunkSection.setBlockState(x, sy, z, st)`（**4 参重载 ⇒ `lock=true`**）→ `ChunkSection.setBlockState(x,y,z,state,true)` → **`PalettedContainer.swap(x,y,z,value)`**（`:142`：`this.lock(); try{…} finally{ this.unlock(); }`）→ **`LockHelper.lock()`**（`:42`，**非可重入 Semaphore**，`acquire` 阻塞）/ `PalettedContainer.lock(:45)`；vanilla 对照 = `NoiseChunkGenerator.java:337-346`（外层 sections 锁 + 持锁期 `setBlockState(..., lock=false)` 即 `swapUnsafe`，**不加内层锁**）。溯源：错误台账 `.investigations/perf-reg-260910-06/errors-260910-06.md` E1；判决 `.artifacts/perf-reg-260910-06/verdict-260910-06.md` §2/§8。
- **语义（是什么）**：
  1. `ChunkSection.setBlockState(x,y,z,state)` 的 **4 参重载 = `lock=true`**，内部走 `PalettedContainer.swap()`，而 `swap()` **自己负责** `lock()/unlock()`（**写路径自带锁**）。
  2. 该锁实现（`LockHelper`）**不可重入**：同线程二次 `lock()` 不是计数 +1，而是**永久阻塞**（无异常、无日志、CPU≈0）。
  3. 因此 `ChunkSection.lock()`（外层持锁）**只在写路径改用不带内层锁的接口**（`setBlockState(..., lock=false)` / `swapUnsafe`）时才成立——**「外层段锁 + `lock=false` 写」是成对契约**：只加外层锁 = **自锁死**；只用 `lock=false` = **失去互斥**。
- **判据（可复用）**：① 复刻/接管类改动凡要**自行加段级锁**，MUST 先核该段内所有写接口是否**自带锁**（在被调方找 `lock()` / `try{…} finally{ unlock(); }`）；自带则不要在外层再加，或整段改用 `lock=false` 系列（二选一，不可混）；② 排查签名 = 线程栈里**同一把锁出现两次（持锁帧 + acquire 帧）** + 进程 CPU≈0（方法面判据见 workflow-patterns #113）；③ 跨版本/跨实现的搬运核对表把**锁语义**单列一栏（与 API 形态 / 签名 / 参数映射并列）。
- **家族索引**：workflow-patterns #113（主判据与错误链）、compiler-idioms #11（诊断门控在初始化器内唯一置位——同为「初始化/持锁期语义」类简条）、#12（mixin 包约束——同为「平台语义约束」简条）。

---

### 发现 #24 补充案例（260911-05）：批量导入共享容器的**锁粒度与发布原子性**——`readPacket` 自带一对锁，且填私有容器后单次引用发布

- **发现时间 / 发现者 / 置信度 / module**：260911-05；主会话（C 线 R9 并发/锁论证）；**confirmed**（用户授予 2026-09-11 22:58）；compiler-idioms / 锁语义（**#24 主条在「批量写」形态下的量化面**）。
- **来源定位**：`PalettedContainer.java:203-215`（`readPacket` = `lock()` / `try{…} finally{ unlock(); }`，`:204`/`:213`）；`ChunkSection.java:48-54`（lock/unlock 转发）；`NoiseChunkGenerator.java:342-346`（生成 range 内全 section 上锁）/ `:416`（持锁期 `setBlockState(..., lock=false)` = `swapUnsafe`）/ `:351-355`（统一解锁）；`LockHelper.java:20-21`/`:31-71`（`Semaphore(1)` + `ReentrantLock`，争用即 crash「Accessing … from multiple threads」）；实现 `BulkWb`（每 section 一次 `readPacket`）+ `ChunkSectionAccessor`（原地换容器 + 直写三计数）。
- **观察（锁粒度量化）**：
  - 老 CoreSwap 逐块路径：mixin 在 `populateNoise` HEAD cancel ⇒ vanilla 上锁段整体被跳过；随后**每非空气块**一次 `ChunkSection.setBlockState(x,y,z,st)`（4 参 = `lock=true`）⇒ **每非空气块一对 `LockHelper` 操作**（每 chunk 量级估计数万次，非实测计数），且逐块变异**共享的活容器**（存在数万个中间态可见窗口）。
  - bulk 路径：`readPacket` **自带一对锁** ⇒ 每**被替换** section 一对（实测 `sections_replaced` / `calls`：overworld 4856/607 ≈ **8.0**、nether 4815/625 ≈ **7.7**、end 562/625 ≈ **0.9**；上限 = 24/16/8 section per chunk）；且填的是**尚未发布的私有容器**，最后以**单次引用写**发布 ⇒ 并发读者只见「旧（全空气）」或「新（完整）」，**无部分写可见窗口**。
  - 自洽核对：`sections_replaced + air_sections_skipped = 14,568 = 607×24`（overworld：8.0 替换 + 16.0 空气短路 = 24）。
- **判据（可复用）**：① **批量导入共享容器 = 锁粒度更粗 + 发布更原子**（先建私有对象 → 单次引用发布）——这是**两个独立收益**，评估并发安全性时 MUST 分开陈述；② 复刻/接管类改动涉及「整段替换 vs 逐点写」时，锁语义核对表加两栏：**锁次数**（每块 vs 每段）与**发布原子性**（活容器逐点变异 vs 私有对象单次发布）；③ 判「新路径不比老路径弱」时，`readPacket` / `swap` 这类**自带锁**的接口按「一次调用一对锁」计，不得漏算成无锁。
- **残余风险（诚实声明）**：`sections[s]` 的容器引用非 `volatile`，跨线程可见性依赖既有 chunk 发布/同步机制（与老路径相同）；**未做**并发压力下的可见性专项验证（R9-b，登记为未验证项）。
- **家族索引**：compiler-idioms #24（主条——`swap()` 自带锁 + 「外层段锁 + `lock=false` 写」成对契约）、workflow-patterns #113（跨实现搬运锁语义主判据）、algorithm-fingerprints #22（`readPacket` 契约）、workflow-patterns #115（内容指纹门的边界：指纹取在写回之前，不覆盖写回/并发路径）。

---

### 发现 #24 更正（260911-05，judge S1；§15.4 取代记录——**原「#24 补充案例（260911-05）」正文不删不改**）

> **取代指针**：本条 **supersedes** `发现 #24 补充案例（260911-05）：批量导入共享容器的锁粒度与发布原子性` 的**结论部分**（「新路径不比老路径弱 / 在锁粒度与原子性上更强」）。
> **推翻理由（一行）**：**「锁次数更少」不是「并发语义更强」**——被比较的两把锁**保护的对象不同**（新路径锁的是**私有未发布容器**，老路径锁的是**已发布活容器**），且漏记了「老路径的锁兼作并发冲突检测器、该检测能力被移除」这一项。
> **不被取代的部分**：**发布原子性**方向成立（见下「仍成立的子结论」）。

- **发现时间 / 发现者 / 置信度 / module**：260911-05；judge（S1/A12）指出 + 主会话复核源码；**confirmed**（用户授予 2026-09-11 22:58）；compiler-idioms / 锁语义（**`#24` 主条在「批量写」形态下的更正**）。
- **来源定位**：`PalettedContainer.java:28-49`（`:36` `private volatile Data<T> data`、`:44-46` `lock()` → `LockHelper`）、`:141-149`（`swap()` 自带 `lock()/unlock()`）、`:203-215`（`readPacket` 自带 `lock()/unlock()`）；`BulkWb.java:265-267`（**在私有容器上**构造 + `readPacket`）/ `:166`（取返回值）/ `:170-171`（写入 section = **发布**）；`CppBridge.java:596-610`（老路径逐非空气块 4 参 `setBlockState` = `lock=true`，**锁活容器**）；`record-260911-05.md` §6 R9 更正 / §7 遗留 1、2；judge `review-260911-05.md` S1/A12。

**事实部分（judge 逐条复核成立，本稿复核一致）**
- `LockHelper` 是**真锁**（`Semaphore(1)` + `ReentrantLock`），冲突时**阻塞并抛**「Accessing … from multiple threads」crash（`LockHelper.java:31-54/56-71`）。
- `PalettedContainer.swap()` 与 `readPacket` **各自**带一对 `lock()/unlock()`（本稿实读 `:141-149` / `:203-215`）；vanilla `populateNoise` 的生成期上锁段被 mixin（HEAD cancel）整体跳过（本块之前既如此，C 未改变）。
- 老路径**每非空气块**一次 4 参 `setBlockState`（= `lock=true`，`CppBridge.java:607-610`）。

**更正三点**
1. **新路径的锁对共享容器零互斥**：`BulkWb` 在 `buildContainer` 内构造**刚 new、尚未发布、其他线程不可达**的私有容器并调 `readPacket`（`BulkWb.java:265-267`），**之后**才于 `:170-171` 写入 section（= 发布）⇒ 该锁只保护「本线程独占的临时对象」，**不提供任何对共享容器的互斥**；把它与老路径的锁按「次数」比较**不是同一件事的强弱**。
2. **老路径的锁附带「并发冲突检测」，该能力被移除**：老路径锁的是**活容器**，`LockHelper` 在争用时**立即 crash** ⇒ 它是「多线程访问同一容器」的**检测器/断言**；新路径的私有容器锁**永不争用** ⇒ **该检测能力在 C 之后消失**（冲突从「立即 crash」退化为「静默 data race」）。
3. **「每 chunk 24 次锁」是上界，不是实测值**：24 = 24 个 section 全被替换才成立；**实测均值 = 8.0 次/chunk**（`sections_replaced 4856 / calls 607`；nether 4815/625 ≈ 7.7、end 562/625 ≈ 0.9）。用上界与老路径比 = 双重不对等（对象不对等 + 量级取上界）。

**正确表述（取代原结论）**
- 新路径的正确性**完全外移到「每 chunk 单写者」这一外部保证**——老路径**也**依赖同一保证，但其锁**附带检测**；该外部保证本块**未验证**（`record §7 遗留 1` R9-b）。
- `plan §10.3` 自认 MUST 的前置（NOISE→LIGHT 之间是否有消费者缓存 `ChunkSection` **实例/容器**）本块已在 `record §7 遗留 2` 收口：原地换容器 ⇒ 缓存**实例**安全；缓存**容器**会读到**孤儿容器**；残留风险 = modpack / 未来版本新增「缓存容器」消费者时**静默失效**（老路径不会）。

**仍成立的子结论（方向性，不取代）**
- **发布原子性方向成立**：`PalettedContainer.data` 是 `volatile`（`:36`，本稿实读），且 `readPacket` 在发布前完成全部内部写 ⇒ 读者只见「旧（全空气）」或「新（完整）」，**无部分写可见窗口**。
- **但**：section 数组里的**容器引用本身非 volatile**，x86 TSO 下实践安全，**形式上仍是 data race**（原条已诚实声明）。

**教训（可复用判错经验）**：**「次数更少的锁」不是「并发语义更强」**——评估锁变更 MUST 问三件事：
1. 锁保护的**对象**是否**共享/已发布**？（**私有未发布对象上的锁 = 零互斥**）
2. 原有的**冲突检测/断言**是否随之消失？（真锁 + 争用即 crash = 检测器；锁到私有对象上 = **检测器被移除**）
3. 正确性依赖的**外部前提**（本例「每 chunk 单写者」）是否被**显式声明并验证**？（未声明的前提 = 未验证假设）

- **家族索引**：compiler-idioms #24（主条——`swap()` 自带锁 + 「外层段锁 + `lock=false` 写」成对契约；**本条为其批量形态的更正**）、workflow-patterns #113（跨实现搬运锁语义必须成对核对）、#42（静态机制断言未实测当公理——「不比老路径弱」即此类断言）、#25（静态调研结论失真）、#118（自证行必须做成硬门禁——「检测器被移除」= 检测能力从硬门禁退化为无门禁）。

---

### 发现 #24 补充案例（260913-02，检测器移除面的一般化）: 「冲突检测器随优化移除」是**永久回归面**——适用于任意共享数据结构的写入路径替换，不止锁

- **发现时间 / 发现者 / 置信度 / module**：260913-02；主会话（R9-b 专项对 `#24 更正` 第 2 条的复核与一般化）；**candidate**；compiler-idioms / 并发检测面（**`#24` 更正教训第 2 条的一般化**——原条问「锁」，本条问「任意共享结构」）。
- **来源定位**：`.investigations/r9b-260913-02/record-260913-02.md` §3.2；`judge-260913-02.md`（回归登记核对项）；承接 `compiler-idioms #24 更正（260911-05）`。
- **一般化判据（可复用，MUST 问句）**：评估**替换共享数据结构的写入路径**（换锁 → 换无锁批量写 → 换容器 → 换发布时机，均适用）时，MUST 问：「**老路径自带的并发冲突检测/断言是否随之消失？消失后，破坏前提的未来改动其失败模式如何退化？**」——本案：老路径 per-write `LockHelper` 锁活容器 = 「多线程访问同一容器」立即 crash 的检测器；bulk 路径锁私有未发布容器 = 零互斥且**永不争用** ⇒ 检测能力永久消失，未来引入同 chunk 双任务时失败模式从「立即 crash」退化为「静默 data race」。**该退化必须作为回归项登记**（不是正确性缺陷，是可观测性损失），且登记载体要能被未来改动者检索到（本块：record §3.2 + 本条）。
- **家族索引**：compiler-idioms #24 更正（主体三问——锁对象共享性 / 检测器消失 / 外部前提显式化，本条为第 2 问的一般化）、#118（检测能力从硬门禁退化为无门禁）、workflow-patterns #143（单写者论证模板——检测器消失的回归面与其配套）。

---

### 发现 #24 补充案例（260913-03，回归面收口）：已登记的检测器回归面补回的落地形态——门控 sentinel 模式四要件

- **发现时间 / 发现者 / 置信度 / module**：260913-03；主会话（swe 收敛实现）+ judge（隔离 subagent，PASS-with-conditions：0 MUST / 2 SHOULD / 4 INFO，条件已应用）；**candidate**（confirmed 留用户）；compiler-idioms / 并发检测面（**260913-02 补充案例登记的「补回可观测性」未来动作的收口**）。
- **来源定位**：实现 = `java-core\src\main\java\wg\bench\BulkWb.java`（`-Dcoreswap.bulkwbsentinel` 门 + `SENTINEL_ACTIVE` + `sentinelEnter/Exit/Crash` + `writeSections` 包裹层）；执行记录 = `.investigations/000-架构设计/架构计划-260913-03-R9b并发加固.md`；一手源对照 = `.tmp\scout-260905-08\mcsrc\net\minecraft\util\thread\LockHelper.java`（`tryAcquire` 失败即 crash，含同线程重入 + 双方线程 dump）。承接 #24 更正（260911-05）+ #24 补充案例（260913-02）。
- **落地形态（检测器补回的四个要件，可复用）**：
  1. **默认关门控 + 单点求值**：`static final boolean SENTINEL = System.getProperty(...) != null`（`<clinit>` 单次赋值），门判断 chunk 级一次（`sentinelEnter` 首行 return），不落每 section 每点（#98 诊断门控纪律）。⚠️ 这不是编译期常量消除——见发现 #26（260913-03）。
  2. **行为化自证行**：`[WG-BULKWB-SENTINEL] armed` 一次性打印（`AtomicBoolean` CAS 保证恰一行）——门开/门关两臂各出「armed 出现 / 不出现」正负成对自证（#81 判据），防「门没生效的空验证」（#139① 同族）。
  3. **违例路径同构复刻 + 检测域语义显式声明**：`sentinelCrash` 与 `LockHelper.crash` 同形态（`CrashException` + 双方线程 dump 写 crash report + 不吞异常，崩溃捕获铁律）；检测域 = **在飞重叠 + 非可重入**（`putIfAbsent` 命中即 crash，含同线程重入），忠实复刻不扩大不缩小——不抓顺序双写。语义边界 MUST 显式写进类注释，防后人误当广义互斥锁。
  4. **粒度差异显式声明**：原检测器 per-write（每块一次），sentinel 为 chunk 级（每 chunk 一次 map 操作）——per-chunk 单写者不变量（R9-b confirmed）下的**有意升级**，非遗漏；写入类注释。
- **验证形态（可复用样板）**：正对照（门开 + 正常单写者生成 522 chunk 零误报）+ 门关臂零介入自证 + 违例路径无法注入时**降级声明**（Degraded 局部：门控行为 Full 运行时证据、违例路径静态论证承载），不假装验证过。
- **判据（可复用）**：评估「检测器随优化移除」的回归面补回时，四要件齐备才算收口——①默认关且门判断不在热路径 ②armed/未-armed 正负自证行 ③违例路径与原检测器同构（含检测域语义声明）④粒度/语义差异显式登记。缺任一件 = 未收口（缺②则门生效性只是假设，缺③则失败模式仍退化）。
- **家族索引**：#24 主条、#24 更正（260911-05）、#24 补充案例（260913-02）、#26（门控编译期常量误称）、workflow-patterns #81/#143。

---

## 发现 #25 简记: 给「逐字复制」的类加头注释也会改变该 `.class` 条目 sha——`LineNumberTable` 随源码行号位移（指令完全相同）（260912-01）

- **发现时间 / 发现者 / 置信度 / module**：260912-01；主会话（Wave 2 共享化实施中实测）+ scout-judge（C2 要求逐条声明调试属性类差异）+ 本波 judge（**C4 即本条的正向实例**）；**confirmed**（用户授予 2026-09-12 15:52；judge 已做 = PASS-with-conditions、C1–C9 已响应，record §4.7.7；record §6 已回填）；compiler-idioms / 类文件调试属性与字节锚（**#24 的「类文件结构」邻接面**）。
- **来源定位**：`record-260912-01.md` §4.6（`:161-168` 构建三态表——`post1`（含头注释）→ **`post2` = V1b 判定基线**，隔离差异恰 2 条 = `CoreSwapFixHelper` + `StallWatch`；1.20.1 侧 `:184`「`StallWatch` 逐字复制后条目 sha 全等；**曾因加 1 行头注释导致行号位移 ⇒ 删注释恢复字节锚**」；`:178/180/182/200/201` 各「仅调试属性（`LineNumberTable`）」条目 + `javap -c -p` 输出完全相同的判定）+ **§4.7.7（C4 条 `:305`：注释修补保行数 ⇒ 类字节不变）**；证据 = `evidence/post1-vs-post2-1.20.1.txt`、`evidence/post3-vs-post4-{1.20.1,1.21.6}.txt`。
- **现象（语义/机制）**：把一个类**逐字复制**进共享源后条目 sha 全等；**加 1 行头注释**（如「本文件自 1.20.1 共享化而来」）后该条目 sha **改变**，而 `javap -c -p` 输出**完全相同**（指令集未变）——差异全在调试属性。
- **根因（机制）**：javac 为指令生成 `LineNumberTable`（源码行号 → 字节码偏移），**行插入使其后所有源码行号 +1** ⇒ 每条指令的 start line 整体位移，类文件的属性表字节随之变化（常量池/指令不变）。⇒ 「注释不影响产物」对**需要字节锚**的场景**不成立**。
- **定位（怎么发现的）**：对拍同一类的**条目级 sha** 与 `javap -c -p`（不带 `-l`）输出——sha 变而 javap 输出完全相同 ⇒ 差异只在调试属性（`LineNumberTable`）；`javap -v`/`-l` 可见行号变化。
- **判据（可复用）**：① 需要**字节锚**（V1-strict 等价门 / 跨树条目级 sha 比对）的「逐字复制」文件 **MUST NOT 加头注释、MUST NOT 做纯格式调整**——出处/血统写在 **README + commit message**；② 判「仅调试属性差异」MUST 用 `javap -c -p`（该输出不含 `LineNumberTable`），并在声明里写「仅调试属性」而非「等价」；③ 反向利用：**想故意零字节差异**时，行号是必须冻结的变量之一（与空白、导入顺序并列）；④ 与「注释级改动 jar 逐字节相同」（build-tooling #56 / #116 等价性门的一环）**不矛盾**——那是**不加/删行**的注释改动，本条是**增删行**的注释改动，两者结论不同、MUST 分开表述；⑤ **正向对偶（本波 judge C4 实例）**：若必须改注释，则**保持总行数不变**（逐行替换）⇒ 其后代码行号不变 ⇒ `LineNumberTable` 不变 ⇒ 类字节不变（`ChunkTiming.java` javadoc 修补实测 `post4` ≡ `post3`、差异 0，两版 jar sha 相同 —— `evidence/post3-vs-post4-*.txt`）；⑥ **V1b 判定基线 MUST 声明是否含头注释态**（本波 `post1` 含注释、`post2` 删注释后才是判定基线；判定基线选错会把「行号位移」当成 Java 面差异 —— judge C6）。
- **家族索引**：workflow-patterns #116（全量条目级 sha256 等价门）、workflow-patterns #138（V1b 声明粒度：仅调试属性 vs 指令级）、build-tooling #56（UP-TO-DATE 假绿——注释级改动的另一半）、f5-bugs（javap 不可信点）、build-tooling #61（javap 对拍陷阱）。

---

## 发现 #26 简记: 读 sysprop 的 `static final boolean` 不是编译期常量——javac 不内联不死码消除，「门关零开销」的正确措辞 = `<clinit>` 单次求值 + C2 运行期折叠（260913-03）

- **发现时间 / 发现者 / 置信度 / module**：260913-03；judge（隔离 subagent SHOULD-1）+ 主会话复核；**candidate**（confirmed 留用户）；compiler-idioms / Java 门控与常量语义（诊断门控家族 #11/#98 的措辞纪律面）。
- **来源定位**：`java-core\src\main\java\wg\bench\BulkWb.java`（`SENTINEL = System.getProperty("coreswap.bulkwbsentinel") != null`）；`.investigations/000-架构设计/架构计划-260913-03-R9b并发加固.md` judge SHOULD-1 段。
- **机制（为什么）**：`System.getProperty(...)` 是方法调用，**非常量表达式** ⇒ javac 不内联、该字段不是 JLS 编译期常量 ⇒ `if (!SENTINEL) return;` 之后的代码**不会**被编译期剔除（Java「条件编译」只对 `static final` = 常量表达式字面量成立）。实际保证链 = `<clinit>` 恰一次求值 + C2 JIT 运行期折叠 ⇒ 门关成本 = 每 chunk 一次方法调用 + 一次分支，可忽略但**不是零指令**。
- **判错经验（可复用）**：① 「门关零开销」类性能声明，机制名 MUST 与实际保证链一致——「compile-time 消除」vs「clinit 单赋值 + 运行期折叠」是不同声明，后者才成立；② 确需编译期消除须用 `static final boolean = <常量表达式字面量>`（构建期生成常量类/常量注入），接受其代价；③ 自查：对声称「编译期消除」的门，javap 看 `<clinit>` 与分支是否还在字节码里。
- **家族索引**：#11（诊断门控初始化器唯一置位）、#25（类文件常量面——同属「编译期常量直觉不可靠」）、workflow-patterns #98 家族（诊断门控 chunk 级一次）、#120（公众声明与门控同源核对——同一「措辞与机制对齐」纪律）。**验证分层 = Degraded**（静态 JLS 语义论证 + javap 自查法，未做专门字节码实验）。
