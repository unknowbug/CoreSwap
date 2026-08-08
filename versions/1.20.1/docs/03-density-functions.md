# 3. 密度函数系统（density.h / density_builder.h）

## 功能目的

密度函数是 MC 世界形状的「数学描述」：从 JSON 数据包（noise_router + density_function/*.json）
构建一棵纯函数树，采样任意 (x,y,z) 得到密度值。finalDensity 的符号决定「石头 vs 空气/水」，
negative 区域再交给 aquifer/oreVein 细分。

## 1.20.1 工作机制

### 数据驱动

- `data/minecraft/worldgen/noise_settings/overworld.json` 的 `noise_router`：15 个分量（barrier/continents/erosion/final_density/vein_*…），
  分量内联或引用 `minecraft:overworld/<name>` 密度函数文件。
- `density_function/overworld/*.json`：sloped_cheese（核心地形）、caves/*（洞穴）、final_density 引用链。
- `noise/*.json`：`{firstOctave, amplitudes}` → DoublePerlinNoiseSampler 参数。

### 节点类型（buildNode 分支）

| type | C++ 类 | 语义 |
|---|---|---|
| `minecraft:noise` | NoiseDF | `sampler.sample(x*xz_scale, y*y_scale, z*xz_scale)` |
| `minecraft:old_blended_noise` | InterpolatedNoiseDF | base_3d_noise：**逐块重算 8+16 次 Perlin**（与 Java 一致，无缓存） |
| `minecraft:interpolated` | **InterpolatedDF** | cell 网格（4×4×8）采样 + 三线性插值 |
| `minecraft:range_choice` | RangeChoiceDF | `input ∈ [min,max)` → whenIn / whenOut；**fill 特殊**（先填 input 再逐点重采样） |
| `minecraft:y_clamped_gradient` | YClampedGradientDF | y 线性渐变 |
| `minecraft:add/mul/min/max` | BinaryOpDF | 二元运算 |
| `minecraft:squeeze` / `abs` / `neg` | UnaryOpDF | clamp(-1,1) / abs / -x |
| `minecraft:constant` | ConstantDF | 常量 |
| `minecraft:y` | Y | 返回 blockY |
| `minecraft:shift_*` | ShiftedNoiseDF | 大陆偏移（spline） |
| `minecraft:blend_alpha/offset/density` | Blend* | 旧世界 blend（NoBlending 时恒等） |
| `minecraft:cache_*`/`flat_cache` | WrappingDF | 语义委托（性能缓存，C++ 未实现缓存） |

> ✅ **已更正（2026-08-08）**：C++ 已实现 FlatCacheDF（5×5 网格预计算）与 Cache2DDF（列缓存）——后者缓存 key 曾误用 chunk 级（Java 是 block 级 `ChunkPos.toLong(blockX,blockZ)`），2026-08-08 已修复（块状 bug 主因，见 10 时间线归档）。
| `minecraft:weird_scaled_sampler` | WeirdScaledSampler | 洞穴 noodle 的 rarity 映射 |
| `minecraft:spline` | SplineDF | 三次样条（continents/erosion/ridges） |

### InterpolatedDF（关键语义！）

- 网格：`5×49×5` 角点，x/z 间隔 4、y 间隔 8（cell 4×4×8），角点 = `(chunkX*16+gx*4, -64+gy*8, chunkZ*16+gz*4)`。
- 块级采样：三线性插值 8 角点。
- **缓存**：per-instance `thread_local`（O(1) ID 索引），按 (chunkX,chunkZ) key 懒构建——多线程安全（07 篇）。
- **为什么存在**：高频噪声（noodle/vein）逐块采样会 alias，MC 用 cell 插值平滑；同时省算力。

### ⚠️ 块级插值顺序（曾经 99.78%→100% 的关键修复）

**Java 只对 `interpolated` 节点插值，`min/squeeze/mul` 等非线性在插值之后应用**：

```
Java: density(块) = min(squeeze(mul(0.64, lerp3(blend_density 角点))), noodle(块))
C++ 旧实现(错): 先 min/squeeze/mul 整树角点 → 再三线性插值（非线性不可交换！）
```

正确实现 = 块级直接 `finalDensity.sample(pos)`，让树内 InterpolatedDF 自行插值。
**教训：任何「把整棵树在角点采样再插值」的优化都会破坏对齐。**

### base_3d_noise（InterpolatedNoiseDF）

sloped_cheese 的核心，`random = split("minecraft:terrain")` 派生，xz/y scale + factor 来自 JSON。
**逐块 24 次 Perlin 采样**（8 主 + 16 辅助）——Java 也这样，无缓存。性能热点（~12ms/chunk，07 篇）。

## 版本敏感点

- [ ] **节点类型集合**：1.17 无 `weird_scaled_sampler`/`spline` 部分参数；1.19+ 有 `shifted_noise` 变体。diff `DensityFunctionTypes` 的 codec 注册列表。
- [ ] **NoiseParametersKeys**：每个版本的噪声参数表（`noise/*.json` firstOctave/amplitudes）变化大，直接 diff 数据包。
- [ ] **old_blended_noise 参数**（xz_scale/y_scale/xz_factor/y_factor/smear）：1.18 前是 `base_3d_noise` 内联参数，1.20.1 走 JSON。
- [ ] **noise_router 分量集合**：1.20.1 有 vein_*；1.17 无矿脉分量（ORE_VEIN 1.18+ 才引入）。
- [ ] `interpolated` 网格间隔（4×4×8）在 1.18+ 稳定，1.17 验证。
- [ ] **DensityFunctions.createOverworldNoiseRouter 动态构造**：vein_toggle 等是代码构造的 verticalRangeChoice（Y 边界 [-60,51]），不是纯 JSON——新版本看 DensityFunctions.java 的 vein 部分。

## 已验证的坑

- 非线性函数（min/squeeze/mul）**不可先采样后插值**（见上）。
- `range_choice` 的 `fill` 语义特殊：先填 input 再逐点重采样——复刻时别用默认 fill。
- `old_blended_noise` 的 random 用 `split("minecraft:terrain")`（Identifier.toString 带命名空间），漏命名空间整体错位。
- 常量 `0.390625`（estimateSurfaceHeight 阈值）是 1.20.1 硬编码，版本间可能变（见 04 篇）。

## 2026-08-08 已验证结论（自 10 时间线归档提炼，完整过程见 10-timewise-archive.md）

### ✅ Cache2DDF 缓存 key 修复（块状 bug 主因）
- Java 1.20.1 Cache2D 是 **block 级**单槽缓存：`ChunkPos.toLong(blockX, blockZ)`（javap 反编译：lastSamplingColumnPos，key 是 block 原值）
- C++ 曾用 **chunk 级** key `(x>>4)<<32 ^ (z>>4)` → FlatCache 5×5 角点错误共享 → grid 值错 → factor/offset/erosion/ridges 查表差 → finalDensity 角点差 → 浅层符号翻转 → 块状
- 修复后：20000 99.4115%→**99.9850%**（角点密度 0.127→≤2e-6）；块循环命中率不变（同列连续）→ **性能无损**

### ✅ WeirdScaledSampler rarity 解析修复
- JSON 是 `"type_2"`（**带下划线**）；C++ 曾写 `"type2"`（漏下划线）→ CAVES 的 weird_scaled_sampler 全部误判 TUNNELS（scale 1.5 vs 1.0）
- 链条：spaghetti_2d weird 差 → entrances 差 → when_out_of_range 差 → 8576 块状；修复后 8576 98.67%→**99.60%**，密度角点全部对齐

### ✅ 已确认一致 / 已排除
- **InterpolatedDF 整树插值 = 正确语义**（chunk(-18,-16) 100% 实证）；「噪声插值+非线性后置」重构已实现并回滚（全区域变差）——**勿再尝试**
- InterpolatedDF cell 大小：`verticalCellBlockCount = BiomeCoords.toBlock(2) = 8`，C++ CELL_Y=8 正确（不是 16）
- Java CellCache（cache_all_in_cell）缓存同 pos 同值 → C++ 纯委托等价（无损）；**CellCache 反射有缓存污染不可信**（勿作密度参照）
- **Beardifier.sample 恒 0.0**（结构密度修正在 1.20.1 是空实现）——不是差异
- **cns 反射不可信**：`ChunkNoiseSampler.interpolators` 是 8 个组件插值器，get(0) min=-∞ 非 finalDensity；DensityInterpolator.sample 依赖 cns 遍历状态
- **8 个 DensityInterpolator 映射**：idx0=finalDensity 顶层（BlendDensity）、idx1-4=noodle 的 4 个（noodle 噪声/thickness/ridge_a/ridge_b）、**idx5-7=ore_vein 的**（veininess/vein_a/vein_b，在 OreVeinSampler 不在 finalDensity 树）
- **finalDensity 树结构**：`min(squeeze(0.64×interp(blend)), caves/noodle)`；blend 内嵌 `range_choice(input=sloped_cheese, min=-1e6, max=1.5625, in=min(sloped_cheese, 5×entrances), out=cave 逻辑)`
- **684.412f / maintainPrecision / 浮点精度**：全部排除（见 02 篇）
- **-namedDump 可信**（与 -nbDump 逐位一致）；**dfreg 不可信**（DENSITY_FUNCTION registry 原始树 ≠ 游戏实际）；cache（actualDensityFunctionCache）是游戏实际（可信）
- **base_3d_noise（InterpolatedNoiseSampler）彻底排除**：与 Java 游戏实际 deriver 逐位一致（8 次 interp + 16 次 lower/upper + clampedLerp 逐行一致）

### ❌ 未解（8576 剩余差候选）
- 8576 剩余 826 块 = **terracotta 带边缘**（y=100-108 地表带，C++ 判 air vs Java terracotta）——est/带数组已排除，疑地表带窗口边界
- 16 格宽「地貌同构划线」（1.0.12-pre 实测）——疑 FlatCache 网格角点值特定位置差（biome 相关），与 8576 finalDensity 微差（0.006@洞穴）可能同源
