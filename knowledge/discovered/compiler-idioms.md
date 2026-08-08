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
