# 2. 随机数派生（xoroshiro.h）

## 功能目的

MC 世界生成的一切随机都来自单一 seed 的**确定性派生链**。同一 seed + 同一调用序列 → 逐位相同的随机序列，这是 C++ 与 Java 逐位对齐的基石。

## 1.20.1 工作机制

### 根 seed 派生

```
worldSeed
  └─ new XoroshiroRandom(seed)              # 根
      └─ .nextSplitter()                     # impl.next()×2 → Splitter(seedLo, seedHi)
          └─ 即 NoiseConfig.randomDeriverPublic()
```

### Splitter（无状态，纯函数）

| 方法 | 实现 | 线程安全 |
|---|---|---|
| `split(x, y, z)` | `XoroshiroRandom(hashXYZ(x,y,z) ^ seedLo, seedHi)` | ✅ const 纯函数 |
| `split(name)` | `XoroshiroRandom(createXoroshiroSeed(name).split(seedLo, seedHi))` | ✅ const 纯函数 |
| `nextSplitter()` | `impl.next()`×2 → 新 Splitter | ❌ **修改内部状态** |

`split(name)` 的 `XoroshiroSeed.split(lo, hi)` = 把 MD5 派生出的 128bit seed 与 Splitter seed 再混合（`s.split(seedLo, seedHi)`）。

### hashXYZ（版本最敏感！）

```cpp
// 1.20.1: MathHelper.hashCode 是 long 版本
int64_t l = (int64_t)(int32_t)((uint32_t)x * 3129871u)   // ⚠️ int 乘法补码溢出
         ^ ((int64_t)z * 116129781LL)                    // long 乘法
         ^ (int64_t)y;
uint64_t u = (uint64_t)l;
u = u * u * 42317861ULL + u * 11ULL;                     // 平方混合
return (int64_t)u >> 16;                                 // 算术右移（符号扩展）
```

三个易错点：
1. `x * 3129871` 是 **int 溢出**（截断为 32bit），`z * 116129781L` 是 long——两个乘法**精度不同**！
2. `l * l * 42317861L + l * 11L` 在 long 上溢出（补码）——用 `uint64_t` 模拟。
3. `>> 16` 是**算术右移**（负数符号扩展），不是逻辑右移。

### createXoroshiroSeed(name)

```cpp
auto h = md5(name);                       // 16 字节
uint64_t lo = big-endian(h[0..7]);        // Longs.fromBytes = 大端
uint64_t hi = big-endian(h[8..15]);
```

### 使用方（派生链）

| 用途 | 派生 | 位置 |
|---|---|---|
| aquifer | `randomDeriverPublic().split("minecraft:aquifer").nextSplitter()` | worldgen_api.cpp（per chunk） |
| oreVein | `split("minecraft:ore").nextSplitter()` | 同上 |
| 噪声 sampler | `splitter.split("minecraft:<noise 名>")` | density_builder.h getNoiseSampler |
| blob 位置 | `splitter.split(x, y, z)` | aquifer.h getBlockPos |
| surface 噪声 | `splitter.split(name)` + `split(x,y,z)` | surface.h splitterFor / VerticalGradientCond |

## 版本敏感点

- [ ] **hashXYZ 公式**：1.18 是 3 参数 LCG（`x*341873128712 + z*132897987541 + y` 之类），1.20.1 换成 long 版 hashCode。**每个版本都要核对 MathHelper.hashCode 源码**。
- [ ] `split(x,y,z)` 与 `split(name)` 的**调用顺序/次数**：派生链位置变化会整体错位（如 1.18→1.19 引入 noise registry 后 split(name) 的内部混合可能变）。
- [ ] `Xoroshiro128PlusPlusRandom` 的 next 实现（rotl 常量 23/17/26）——一般稳定，但核对 `random.h`。
- [ ] `nextFloat/nextDouble/nextInt(bound)` 的拒绝采样实现（1.20.1 用 `Integer.remainderUnsigned` 变体）。

## 已验证的坑

- **int vs long 乘法**：`x*3129871` 必须 int 溢出，曾导致 surface/verticalGradient 派生错位（surface 坑的根源之一）。
- **算术右移**：`>> 16` 对负数必须符号扩展；无符号实现后转回有符号再右移。
- **nextSplitter 有状态**：多线程时每个线程必须从 `split(name)`（纯函数）重新派生，不能共享 Splitter 的 nextSplitter（见 07 篇）。

## 2026-08-08 已验证结论（自 10 时间线归档提炼，完整过程见 10-timewise-archive.md）

### ✅ 已确认一致
- **XoroshiroRandom(seed) 单参数构造** = RandomSeed.createXoroshiroSeed（SHA-256 混合，random.h:46）与 Java 一致
- **Xoroshiro128PlusPlusRandom.nextInt(bound)** = Lemire 乘法（`l*bound` 高 32 位 + 拒绝采样），C++ 逐行一致
- **legacy 构造 random 消费顺序**：firstPN + kx 循环 + skipCalls=262 一致
- **Perlin 负坐标差异 = 假象**：b3d（InterpolatedNoiseSampler）在负坐标与 Java 游戏实际 deriver 逐位一致（3e-5 级，含 -8248 负坐标多列）；「负坐标 Perlin 差坐实」是 RouterProbe rd2 漂移假象，勿再查
- **Java 1.20.1 PerlinNoiseSampler.sample 无 512 归一化**（1.18 前的旧版才有），C++ 直接 floorD 一致

### ✅ maintainPrecision（已修复）
- Java 1.20.1：`(long)(v/33554432.0 + 0.5)`（+0.5 后向零截断）——C++ 曾误写成纯向零截断
- **只在 |坐标×scaledXz|×2^r > 3.35e7 时触发**（|x| > ~19.6 万）——玩家小坐标不触发，但正坐标超阈值区域（20000/30012000）会触发

### ✅ nextDouble float 精度（已修复）
- Java `Xoroshiro128PlusPlusRandom.nextDouble() = next(53) * 1.110223E-16F`——**float 常量**（53 位舍入到 ~24 位）
- C++ 原实现用 double 常量（53 位全保留）→ base_3d_noise 差 ~7e-6——已改 float 对齐 Java
- 影响：PerlinNoiseSampler originX/Y/Z（nextDouble()*256）差 ~5e-7，在 maintainPrecision 折叠边界可能放大

### ❌ 已排除
- **684.412f 精度**：模拟 Java `(double)(float)684.412` 后主世界 100% 无变化

## 2026-08-29 CheckedRandom / ChunkRandom（CARVERS 阶段，Rust 移植 chunkrandom.rs）

> CARVERS 阶段用 `ChunkRandom`（基类 `CheckedRandom`，48 位 LCG），与 FEATURES 阶段（基类 `Xoroshiro128PlusPlus`）不同。Rust 移植 `WorldgenRust/src/chunkrandom.rs`。

- **CheckedRandom（48 位 LCG）**：`java.util.Random` 算法。`seed = (seed * 25214903917 + 11) & (1<<48)-1`；`next(bits) = (int)(seed >> (48-bits))`。
- **ChunkRandom.next(bits) 按基类分派**：基类 CheckedRandom → `lcg.next(bits)`（LCG）；基类 Xoroshiro → `(int)(baseRandom.nextLong() >>> 64-bits)`（高 bits 位）。
- **setCarverSeed(worldSeed, chunkX, chunkZ)**：`setSeed(worldSeed); l=nextLong(); m=nextLong(); n=chunkX*l ^ chunkZ*m ^ worldSeed; setSeed(n)`。
- **nextLong() 有符号拼接（MC-239059）**：`(long)next(32) << 32 + next(32)`——i/j 都是 int 符号扩展后做有符号加法，j<0 时高 32 位被 0xFFFFFFFF 填充，**非无符号位拼接**。
- **nextInt(bound)**：幂 2 用 `(int)((long)bound * next(31) >> 31)`；非幂 2 用 do-while 拒绝采样 `i % bound`（Java int 回绕，无符号模拟防 UB）。
- **nextDouble()**：`((long)next(26) << 27 + next(27)) * 1.110223E-16F`——long * float 是 float 乘法（精度截断），结果提升回 double，用 float 模拟。
- **可复用判据**：MC 里 `Random.create(seed)` 默认实现是 `new CheckedRandom(seed)`（48 位 LCG），**不是** Xoroshiro——凡看到 `Random.create(...)` 派生内部随机源，先确认是 LCG 而非 Xoroshiro（carveTunnels/carveRavine 内部递归即此）。
