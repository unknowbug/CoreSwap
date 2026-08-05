# 5. 矿脉（ore_vein.h）

## 功能目的

生成矿脉状岩石（granite/diorite/andesite/tuff）+ 铜/铁矿块，取代旧版「散点矿石」。
1.18+ 引入；veinToggle/veinRidged/veinGap 是 noise_router 的三个分量。

## 1.20.1 工作机制

### 三个分量（DensityFunctions.java 动态构造，非纯 JSON）

```java
// vein_toggle = interpolated(rangeChoice(Y, -60, 52, noise(ORE_VEININESS, 1.5, 1.5), constant(0)))
// vein_ridged = add(-0.08, max(abs(rangeChoice(Y,-60,52,noise(ORE_VEIN_A,4,4),0)),
//                               abs(rangeChoice(Y,-60,52,noise(ORE_VEIN_B,4,4),0))))
// vein_gap    = rangeChoice(Y, -60, 52, noise(ORE_GAP, 4, 4), constant(10))
```

- Y 范围 `[-60, 51]`（VeinType 的 minY/maxY 扩展）。
- 每个 `rangeChoice` 都带 `interpolated` 包装（块级插值，见 03 篇）。
- 噪声 scale：veininess 1.5，ridged A/B 4.0，gap 4.0。

### 决策链（apply）

```java
double d = veinToggle.sample(pos);
VeinType t = d > 0 ? COPPER : IRON;        // copper: y[0,50], stone=granite/andesite, ore=copper_ore/raw_copper
                                           // iron:   y[-60,-8], stone=tuff/diorite, ore=deepslate_iron_ore/raw_iron
double e = abs(d);
int j = t.maxY - i, k = i - t.minY;        // i = blockY
if (k >= 0 && j >= 0) {
    int l = min(j, k);
    double f = clampedMap(l, 0, 20, -0.2, 0);
    if (e + f < 0.4) return null;          // ⚠️ 0.4 阈值
    if (random.nextFloat() > 0.7) return null;      // ⚠️ 30% 概率通过
    double g = clampedMap(e, 0.4, 0.6, 0.1, 0.3);
    if (random.nextFloat() < g && veinGap.sample(pos) > -0.3) {
        return random.nextFloat() < 0.9 ? t.ore : t.stone;   // 90% 矿石
    }
}
return null;   // 保持默认
```

**注意**：`veinRidged >= 0 → return null` 的条件在 apply 开头
（Java: `if (veinRidged.sample(pos) >= 0.0) return blockState;`）——C++ 实现时容易漏。

### 随机派生

```cpp
XoroshiroRandom oreRnd = randomDeriverPublic().split("minecraft:ore");
OreVeinSampler(…, oreRnd.nextSplitter(), …);   // per chunk
```

### 接入（ChainedBlockSource）

```cpp
// aquifer.apply 返回 -1（null）时才尝试 oreVein
int block = aquifer.apply(x, y, z, density);
if (block < 0) block = oreVein.apply(x, y, z);
```

## 版本敏感点

- [ ] **1.17 无矿脉**（ORE_VEIN 1.18+）：迭代 1.17 需删 ore_vein 接入与分量。
- [ ] VeinType 的 minY/maxY（copper [0,50]、iron [-60,-8]）与随机阈值（0.4/0.7/0.9/0.3/-0.3）——直接 diff OreVeinSampler.java。
- [ ] `split("minecraft:ore")` 派生（版本间 split 内部混合可能变）。
- [ ] noise 参数（ore_veininess/a/b/gap 的 firstOctave/amplitudes）→ 数据包 noise/*.json。

## 已验证的坑

- **「矿脉零输出」曾是假象**：16:09 的 vanilla 参照被旧 world chunk 缓存污染（含旧世界矿脉），
  对比出假差异。重新导出 vanilla 前必须删 `run/world/region/` 相关 .mca（08 篇）。
- **验证方法**：`ore_probe.exe`（C++ 采样 veinToggle 三件套 + apply 决策）与 Java RouterProbe 对照；
  VeinDiag 驱动真实 ChunkNoiseSampler 拿块级插值 result 对照（逐位一致 = 0.162342 vs 0.160928 同列）。
- vein_toggle 的 wrapped 在 apply 后是 BlendDensity 包装（surface slides 结构）——识别 Interpolator 时容易拿错（挑 RangeChoice 特征 `min=-60` + `xz=1.5`）。
