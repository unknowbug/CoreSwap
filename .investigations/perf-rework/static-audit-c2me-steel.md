# 静态审计：C2ME / SteelMC 参考方案的本地对照（density/aquifer/noise 层）

> 日期：2026-08-12（session 续）· 类型：静态审计产物（.investigations/ 中间记录）
> 目的：不盲猜优化方向——对照 C2ME（RelativityMC/C2ME-fabric ver/1.20.1）与 SteelMC（Steel-Foundation/SteelMC）在 density/aquifer/noise 相同位置的优化方案，逐项核对本地实现是否已覆盖、有无可抄遗漏。
> 结论摘要：**三个静态可查的低垂果实全部已覆盖（aquifer 缓存 / Perlin 形态 / interpolated 语义），无「我们比 Java 多算」的语义差异**。剩余优化必须走实测构成分解 + 结构性改造（SIMD 批量角点 / DF 树扁平化）。

---

## 一、外部项目调研（方案清单）

### C2ME ver/1.20.1（与 density 计算相关的模块）

| 模块 | 优化 | 本质 |
|---|---|---|
| c2me-opts-math | PerlinNoiseSampler scalar 重写（commit a7c17a4，~25%） | 消除重复索引链：8 次 `GRADIENTS[perm & 15][idx]` 嵌套 → 先取 `grad[]` 引用再索引（JVM 边界检查 + JIT 友好） |
| c2me-opts-worldgen-vanilla | MixinAquiferSamplerImpl（20KB 重写） | ① `waterLevels[]` FluidLevel 结果缓存 ② `blockPositions[]` 3×3×3 邻居随机偏移缓存（消除重复 split+nextInt）③ `barrierNoise` 采样结果复用（mutableDouble + NaN 哨兵）④ 共享 Random 实例替代重量级 split() ⑤ 位运算内联 ⑥ apply 拆方法 JIT 友好 |
| c2me-opts-worldgen-general | AtomicSimpleRandom 替换 RNG | 省 split 开销 |
| c2me-threading-worldgen | 线程化 worldgen | 本地已做 ✅ |

**注意**：C2ME 1.20.1 **没有** SIMD、**没有** density function compiler（DFC 是 1.21.3+ 引入，interfaces 调用 overhead 在 vanilla datapack ~30%、Tectonic 类复杂 datapack ~90%）。

### SteelMC（Rust 服务器，10,201 chunk / 3.98s 中位数）

| 优化 | 本质 |
|---|---|
| Rust trait 静态分派 | density function 树无虚调用，编译器整体内联（= C2ME DFC 的天然版） |
| 多通道插值（MAX_INTERP=16，Overworld 8 通道） | terrain + 4 noodle caves + 3 veins 一次角点遍历同时算，共享噪声求值 |
| fill_slice 批量 Y 列 SIMD 噪声（compute_noise_column） | 一次 SIMD 算整列 Y 噪声（lane 独立） |
| steel-math SIMD primitives | lerp/smoothstep/grad_dot 向量化，opt-level=3 常开 |
| 确定性 per-sampler biome 缓存 | 替代 vanilla ThreadLocal |

---

## 二、本地静态审计结果（逐项对照）

### 2.1 aquifer.h（对照 C2ME MixinAquiferSamplerImpl 重写清单）

**结论：已全部覆盖，无遗漏**（aquifer.h 397 行）：

| C2ME 优化 | 本地实现 | 判定 |
|---|---|---|
| blockPositions 缓存 | aquifer.h L175 + L220-230（`blockPositions[aa]` 命中即返，未命中 split+nextInt 后缓存） | ✅ 同款 |
| waterLevels 缓存 | aquifer.h L176 + L279-287（`getWaterLevelAt` 命中即返） | ✅ 同款 |
| barrierNoise 复用 | aquifer.h L120 + L183-189 + L261-269（MutableDouble NaN 哨兵） | ✅ 同款 |
| estimateSurfaceHeight 缓存 | aquifer.h L144-164 + L179-181（**flat 数组**，注释明确「原 std::map 红黑树是 aquifer 瓶颈——每块 13 次邻居查询」） | ✅ 更优（C2ME 仍是 HashMap） |
| 方块 id 预取 | aquifer.h L61-64（airId/waterId/lavaId 构造时查一次） | ✅ 同款 |
| 随机实例复用 | 本地 `splitter.split(x,y,z)` 每次创建 XoroshiroRandom（C++ 轻量值类型，无 Java split 重量级问题） | ✅ 等价 |

**遗留观察（非 C2ME 差距）**：L114-117 `getFluidLevel(blockX, blockY-1, blockZ)` 每次 apply 全路径调用（Java 同构，C2ME 也未优化）——列内重复噪声采样，但改它风险高收益低，不列为方向。

### 2.2 noise.h（对照 C2ME a7c17a4 Perlin scalar 优化）

**结论：已是最优形态，无优化空间**（noise.h）：

- `GRADIENTS[16][3]` int32 常量表（L17-22）+ `dot3` inline（L24）+ `perlinFade` inline（L14）+ `map` 单查表（L48）
- `sampleSection`（L82-110）：8 次 map + 8 次 grad + 7 lerp——与 C2ME 优化后形态等价；C2ME 的「grad 数组预取」在 C++ 由内联 + 常量表自动达成
- 微小差异：int32 表 vs C2ME double 表——C++ int32 表更优（L1 友好），无需改
- `OctavePerlinNoiseSampler::sample`（L226-239）标准 octave 循环，`maintainPrecision`（L123-129）已对齐 Java 语义

### 2.3 density 树结构（对照 Java ChunkNoiseSampler / DensityFunctionTypes）

**结论：interpolated 语义与 Java 同构，无「我们比 Java 多算」差异**：

- Java：`noiseRouter.apply(getActualDensityFunction)`（ChunkNoiseSampler L159）把树里 `Wrapping.INTERPOLATED` 节点替换为 `DensityInterpolator`（L452）；**interpolated 来自 data pack JSON 字面量，不是代码无条件包**
- 本地：`density_builder.h` L160-164 `minecraft:interpolated` → `make_shared<InterpolatedDF>`（机械构建无去重）——同构
- **关键**：noise_router JSON（noise_settings/overworld.json）里 aquifer 组件 `barrier`/`fluid_level_floodedness`/`fluid_level_spread`/`lava`/`erosion`/`depth`/`initial_density_without_jaggedness` **均无 interpolated** → Java 和本地都是 raw 树直接采样，**aquifer 采样方式无差异**（推翻了「我们 aquifer 比 Java 贵」的猜测）
- interpolated 分布（字面量）：final_density 1 + caves/noodle 4 + vein_toggle 1 + vein_ridged 2 = 8 = Java cns 的 8 ✅
- 实测 `[BUILD] InterpolatedDF instances=6`（WG_PROFILE + block_probe）——差异为打印时机（worldgen_api.cpp L395-397 打印在 finalDensity 构建后、vein 组件 L402-413 构建前），**非语义差异**；vein 的 3 个在打印后才建

---

## 三、可实施方向排序（静态审计后收敛）

> 静态可查的低垂果实全部排除 → 剩余方向均为结构性改造，**必须先用构成分解（WG_STAGETIMER 细化）确认 44ms 构成再动手**。

| # | 方向 | 依据（外部项目实证） | 本地可行性 | 前置 |
|---|---|---|---|---|
| 1 | **构成分解实测**：density 44ms 拆成 buildGrid 角点采样 / 块级插值 / 噪声链 / spline | 现状只有阶段级计时 | —— | 无（直接做） |
| 2 | **buildGrid 角点采样 SIMD 批量（Y 列）** | SteelMC fill_slice compute_noise_column 实证 | 高：buildGrid 循环 gy→gz→gx，可改按列批量算 4 lane；lane 独立不改变单点运算顺序 → 逐位无风险 | ① 确认角点采样占 44ms 大头 |
| 3 | **DF 树扁平化 / 消除虚调用** | C2ME DFC（+30~90%）；Rust 静态分派 | 中：C++ 虚调用比 Java 接口调用便宜，收益需微基准；NEXT_SESSION 已挂此方向 | ① + 微基准 |
| 4 | 多通道插值（terrain+vein 共享角点遍历） | SteelMC 8 通道 | 低：1.20.1 语义下 vein/aquifer 是后置阶段不参与 density 多通道，仅 vein 3 个 interpolated 可与 final_density 共享（收益待测） | ① |

**不建议**：改 densityBuf 循环顺序（H1 已否决，aquifer 同序读取对齐风险）、CELL 有损（RQ-006 用户拍板不做）、Z-order 重排（噪声表在 L1、densityBuf 写入已线性）。

---

## 四、附：审计中使用的外部资料

- C2ME-fabric ver/1.20.1 分支：c2me-opts-math（a7c17a4）、c2me-opts-worldgen-vanilla（MixinAquiferSamplerImpl 20KB）、c2me-opts-worldgen-general（random_instances）
- SteelMC：README（10,201 chunk / 3.98s）、DeepWiki 5.2 Chunk Noise Generation / 2.9 steel-math / 5.1 Biome Generation
- 本地 Java 参照：`E:\PYTHON\MC\data\mc_src_extract\net\minecraft\world\gen\chunk\ChunkNoiseSampler.java`（L158-200、L442-455）
- 本地 worldgen JSON：`versions\1.20.1\data\worldgen\data\minecraft\worldgen\noise_settings\overworld.json`（noise_router）、`density_function\overworld\caves\noodle.json`
