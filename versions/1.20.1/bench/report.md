# POC 报告：MC Java 版区块生成 C++ 化 — 密度场验证

**日期**：2026-08-05
**目标版本**：Minecraft 1.20.1（Fabric Loom + yarn build.10）
**场景**：seed=-8248318472910187742，区域 4×4=16 chunks @ chunk(200,200)，密度采样网格 4×4×8

## 一、一致性验证（核心结论）

| 层级 | 验证范围 | 结果 |
|---|---|---|
| 噪声原语 | 54 个 noise key × 64 点采样 | **3456/3456 逐位一致** |
| Density 函数树 | 16 chunks × 768 点 = 12288 点 finalDensity | **12288/12288 (100%)，maxErr=0** |
| Router 全分量 | barrier/temperature/vegetation/continents/erosion/depth/ridges/initial_density/final_density/vein_* | **全部一致** |

**结论**：C++ 复刻的密度场与 vanilla **逐位完全一致**（IEEE double 精确），
无需容差。C++ 生成的地形将与 Java 版完全一致。

## 二、性能基线

| 实现 | 12288 点 density 求值 | ms/chunk |
|---|---|---|
| Java vanilla（JIT 后） | 171.96 ms | 10.75 |
| **C++（-O2，未优化）** | **70.65 ms** | **4.42** |
| 加速比 | **2.43×** | |

**注**：此对比仅覆盖 density 场求值（世界生成管线中计算最密集的部分之一）。
C++ 侧为 -O2 基线，未做任何内存优化（无紧凑数组/缓存布局/SIMD）；
用户已确认的后续优化方向（紧凑数组 + 索引 + 缓存友好布局）预计可再提升 2-5×。

## 三、实现范围（C++ 核心）

```
cpp/src/
├── md5.h/cpp        RFC 1321（RandomSeed 字符串派生）
├── random.h         mixStafford13 + createXoroshiroSeed
├── xoroshiro.h      Xoroshiro128PlusPlus + Splitter（含 split(Identifier) 派生）
├── noise.h          PerlinNoiseSampler + OctavePerlinNoiseSampler(modern+legacy) + DoublePerlinNoiseSampler
├── density.h        DensityFunction 框架：constant/binary/unary/clamp/noise/shift/
│                    shifted_noise/range_choice/y_clamped_gradient/weird_scaled_sampler/
│                    blend*/interpolated/spline(1.20.1 Hermite)/old_blended_noise
├── json.h           最小 JSON 解析器
├── density_builder.h 从 vanilla worldgen JSON 装配 density 树（含 registry 循环引用）
└── worldgen.h/cpp   JNI 入口（probe 验证通路）
```

## 四、关键修复（复刻过程中发现的隐蔽坑）

1. **C++ 函数参数求值顺序**：`Splitter(impl.next(), impl.next())` 字段交换 → 显式求值
2. **MD5 Longs.fromBytes 大端序**：seed 派生全部错误 → 修正字节序
3. **legacy 模式 firstOctave 语义**：`createLegacy(rangeClosed(-15,0))` 的 firstOctave=-15
4. **Identifier.toString 带命名空间**：`split("minecraft:terrain")` 而非 `split("terrain")`
5. **Spline 二分边界**：Java `binarySearch-1` 语义（`f==locations[0]` 应在区间内）
6. **JsonParser 悬垂引用**：构造持临时 string 引用 → 按值持有
7. **depth↔offset 循环引用**：两阶段占位注册（Java RegistryEntryHolder 展开语义）

## 五、POC 结论

✅ **架构成立**：Java 版保留 MOD 生态、性能热点下沉 C++ 的路线验证通过——
C++ 核心可**逐位复刻** vanilla 世界生成（比"自创算法"更强的兼容性），
且 baseline 已有 2.4× 加速。

## 六、后续方向（用户已确认）

1. **代码优化**：紧凑数组 + 索引 + 缓存友好布局（density 求值内存优化）
2. **JNI 桥**：worldgen.dll 导出 `generateRegion`（大块数据一次交换）
3. **方块层**：density → 方块状态（surface rules + 区块填充）
4. **性能对比**：完整 chunk 生成端到端对比（含方块填充）

## 2026-08-05 续：方块层首跑（JNI 桥 + density→aquifer→surface rules）

### 里程碑
- **JNI 桥**：worldgen.dll 导出 wg_create/wg_fill_density/wg_fill_blocks（大块数据一次交换），Java CppWorldgen 验证 density 12288/12288 逐位一致（4.49ms/chunk，JNI 开销可忽略）
- **方块层首跑**：C++ 完整区块管线（density 网格 halo + 三线性插值 → AquiferSampler → SurfaceBuilder + VanillaSurfaceRules 翻译 + MultiNoiseBiomeSource 六维查找）
- **对比基准**：BlockProbe 导出 vanilla SURFACE 状态 chunk（NOISE+SURFACE，不含 structures/carvers/features）

### 结果（seed -8248318472910187742, origin (3200,3208), 4×4 chunks）
| 指标 | 数值 |
|---|---|
| 全方块一致 | 1415757/1572864 = **90.01%** |
| 非空气方块一致 | 373771/494029 = **75.66%** |
| 生成耗时 | 60~1450 ms/chunk（含 biome 采样，需优化） |

### 差异归因（vanilla 非空气构成）
- deepslate 208k / stone 125k / water 59k —— 主体一致
- **tuff/andesite/diorite/granite/copper/coal/iron ≈ 68k —— vanilla OreVeinSampler 生成，C++ 未实现（最大差异源）**
- 其余 ~50k：aquifer 含水层细节 + surface 规则细节 + beardifier（结构边缘）

### 关键修复记录
1. **unpackY 符号扩展 bug**（aquifer blob 点坐标）：pack 负数 y 用 & 0xFFF 包装，unpack 需 12 位符号扩展（Java BlockPos.unpackLongY = l << 16 >> 52）——修复前随机崩溃（waterLevels 越界读/写）
2. **externalLoader 悬垂引用**（wg_create 局部变量捕获 → handle 成员）
3. buildSurface 越界（heightmap+1 > 世界高度 → AIR）
4. TerracottaBandsRule 需 getTerracottaBlock(x,y,z)（bands 按 y 索引）

### 待办（按收益排序）
1. **OreVeinSampler**（vein_toggle/vein_ridged/vein_gap 噪声已有，补方块逻辑）——预计可消 ~68k 差异
2. beardifier（结构边缘密度）或对比时排除结构区域
3. aquifer/surface 细节调试（红陶带/steep/含水层边界）
4. 性能优化（biome 采样缓存 + surface 条件缓存）
