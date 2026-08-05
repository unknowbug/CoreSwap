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
