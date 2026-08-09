# 4. 含水层（aquifer.h）

## 功能目的

决定 density < 0 区域（地下负密度区）的流体/空洞布局：水袋、熔岩袋、空洞。
MC 用「blob 影响场」生成不规则含水层——不是简单的液面填充。

## 1.20.1 工作机制

### blob 网格与候选

```
每个块：l = floorDiv(blockX-5, 16)，m = floorDiv(blockY+1, 12)，n = floorDiv(blockZ-5, 16)
候选 blob：3×3×2 = 18 个（u∈{0,1}, v∈{-1,0,1}, w∈{0,1} 偏移）
取最近的 3 个（o<p<q）→ r/s/t
```

blob 间距：**x/z 16、y 12、offset -5/+1**——版本敏感。

### blob 位置派生（getBlockPos，lazy）

```cpp
// 每个候选位置 (x,y,z)：确定性随机
XoroshiroRandom rnd = splitter.split(x, y, z);       // hashXYZ 派生
double dd = initialDensity.sample(x*4, y*12, z*4);   // 初始密度
if (dd < 0.39) { … }                                 // 决定 blob 存在/是否 barrier
int r = rnd.nextInt(3);                              // 半径抖动
pos = pack(x*4 + rnd.nextInt(4), y*12 + rnd.nextInt(8), z*4 + rnd.nextInt(4));
```

- **lazy 缓存**：`blockPositions[index]` 数组（INT64_MAX 标记未算），每 chunk 实例一次。
- barrier：blob 间 barrier 噪声（`minecraft:aquifer_barrier`）决定水袋是否连通。

### apply 决策链

```
density > 0                       → -1（石头，交给 oreVein/默认）
fluidBlock：y < -54 lava；否则 water（fluidY=63）   # 主世界
  y < -54 → 直接返回 lava
18 候选 → 最近 3 blob（o/p/q）→ r/s/t
d = maxDistance(o, p)             # 到最近两个 blob 的密度距离
fl2 = getFluidLevel(r)            # 最近 blob 的液面
d <= 0 → fl2.getBlockState(blockY)
否则用 e/f/g（blob 液面/密度修正）判断 water / -1
```

### FluidLevel.getBlockState

```cpp
return blockY >= y ? AIR : block;   // y=液面，block=流体方块
```

### ⚠️ 无效液面常量 = -32512（曾经 99.78%→99.96% 的关键修复）

```cpp
// Java: DimensionType.field_35479 = -32512（yarn: INVALID_AQUIFER_LEVEL）
return -32512;   // 不是 INT32_MAX！
```

`getFluidLevel` 找不到液面时返回 `FluidLevel(-32512, …)`。`blockY >= -32512` **恒真 → AIR**。
若误用 INT32_MAX，`blockY >= INT32_MAX` 恒假 → **深地全返回 water**（air→water 2691 块假差异）。

### estimateSurfaceHeight（getFluidLevel 的 13 邻居扫描用）

```cpp
// initialDensityWithoutJaggedness > 0.390625 的最高 8 格点
for (y = 320; y >= -64; y -= 8)
    if (initialDensity.sample(x, y, z) > 0.390625) return y;
// 4 格对齐：(x>>2)<<2（BiomeCoords.toBlock(fromBlock)）
// 列缓存：per-chunk map（Java: ChunkNoiseSampler.surfaceHeightEstimateCache）
```

**性能关键**：无缓存时每块 13 邻居 × 最多 49 次采样 ≈ 3200 万次/chunk（aquifer 占 88% 耗时）；
缓存后每 chunk ~240 列各 1 次（~2700 倍降幅）。列缓存 key = `((x>>2)<<2, (z>>2)<<2)` 打包。

### getFluidBlockState（HANDOFF 修复 4）

直接用 `defaultFluidLevel.block`（Java 用 defaultFluidLevel.state 不经 getBlockState 判断）。

## 版本敏感点

- [ ] **blob 间距常量**：x/z 16、y 12、offset (blockX-5, blockY+1, blockZ-5)——1.17 可能不同，diff AquiferSampler.Impl 构造。
- [ ] **无效液面常量** `field_35479`（1.20.1 = -32512）——新版本反射/查 DimensionType 确认。
- [ ] **estimateSurfaceHeight 阈值 0.390625** 与步长 8、BiomeCoords 4 格对齐。
- [ ] **fluidLevelSampler**：`y < -54 lava else water`（主世界）；新版可能随维度类型变化。
- [ ] barrier 噪声接入方式（1.18 前 aquifer 更简单，1.19+ 有 floodedness 参数重构）。
- [ ] `getFluidLevel` 的 13 邻居偏移模式（surface 扫描范围）。

## 已验证的坑

- **INT32_MAX 陷阱**：任何「无效值」常量都必须从 Java 确认实际值（-32512），不能想当然用 INT32_MAX/INT64_MAX。
- estimateSurfaceHeight 的 4 格对齐：`(x>>2)<<2` 是 BiomeCoords 语义，漏了会整列错位。
- blob 的 `initialDensity.sample(x*4, y*12, z*4)`：参数是 blob 网格坐标×间距，不是块坐标。
- **验证方法**：C++ `[aq]` 调试打印 r/s/t 位置与 fl2.y 对照 Java；VeinDiag 驱动真实 ChunkNoiseSampler（08 篇）。

## 2026-08-08 已验证结论（自 10 时间线归档提炼，完整过程见 10-timewise-archive.md）

### ✅ estimateSurfaceHeight（est）两版一致
- **Java cns 查表版 = 无插值版 = C++ 版**：17 点（含负坐标岛区）全 32——est 不是负坐标差异来源
- 扫描语义：`initialDensityWithoutJaggedness > 0.390625` 从顶向下（步长 8，列缓存），`(x>>2)<<2` biome 格对齐
- **est 修复（4 角插值 → 扫描）正确**（Java 语义），但 8576 略降（99.60→99.576）——est 不是 8576 主差
- C++ sh4（aquifer 4 角）与 Java cns 4 角在**同 seed** 下一致（8576 seed：48/56 系；-8248：32 系）——之前「est 不同」是 seed 混淆假象

### ✅ aquifer 判定链与 Java 逐行一致
- getFluidLevel 13 邻居 `OFFSETS` 与 Java `CHUNK_POS_OFFSETS` 逐项相同
- getFluidBlockY / method_43718（erosion<-0.225 && depth>0.9）/ getNoiseBasedFluidLevel / clampedMap / map2 全部一致
- apply 邻居选择（18 候选 2×3×2）与 Java 逐行一致（含 pack/unpack 负坐标符号扩展）
- -288 岛区 e=0（fl2/fl3 液面全 63）→ 两侧判定一致——**岛缺失不是 aquifer bug**（是 ocean ruin 结构覆盖，见 06 篇/10 时间线归档）
  - **⚠️ 2026-08-09 重审**：本结论的「e=0 两侧一致」中 **Java 侧 e 值从未实测**（trace_aqf_1.txt 仅 C++ e=0.0000；Java fl2.y/fl3.y 是假设）。且 NOISE-BLK 铁证（status=noise 验证）(-244,-256) y=58-61 **NOISE 阶段已 stone**（FEATURE 之前）与「Java aquifer 判 water」矛盾。B3 (b) 子候选（Java 液面网格输入 ≠ C++）未验证。**裁决点**：Java 真实遍历内 dump e 值（DensityProbe 扩展，禁反射）——详见 10 时间线「知识库冲突裁决记录 2026-08-09」；04 篇此结论在裁决前视为 **draft（重审中）**
  - **✅ 2026-08-09 裁决（verdict-04.md，judge 审查 candidate）**：AQF-DUMP 实测 (-244,55..62,-256) `fl2.y=fl3.y=fl4.y=63` 全部相等（8 y 全测，反射真实 getWaterLevel 私有方法）→ **e=0 两侧一致成立（Java 实测确认）**。但「岛缺失不是 aquifer bug」的**归因错误**——真正根因 = **C++ 缺失 Beardifier**（StructureWeightSampler 结构密度修正）：AQF-APPLY dCC（CellCache=add(finalDensity,Beardifier)）= C++ finalDensity + Java Beardifier，8/8 点 ≤3e-6 闭环；(-244,58..61,-256) Beardifier 非零（+0.092~+0.166）使 density 翻正 → aquifer 判 solid → NOISE-BLK stone ✓。**海底边界 ≈6710 块从「aquifer 液面链待修」纠正为「Beardifier 缺失」**（结构相关，范围判定见 verdict-04 §四）

### ⚠️ 坑
- **CellCache 反射污染**：blockStateSampler.sample / CellCache.sample 在非真实遍历状态返回缓存垃圾值（如固定 -0.024995）——**勿以反射作密度参照**；必须用 DensityProbe 的完整 cns 链（sampleStartDensity→interpolateY/X/Z）在真实遍历内取值
- fluid_level_floodedness = `{"type":"minecraft:noise", y_scale:0.67}`（非 cache 包装）——直接采样，RouterProbe 与 C++ 一致（0.0191 @(-244,58,-256)）

---

## 2026-08-08 已验证结论（追加 2）：顺手对齐（Java float 常量提升 + field_35479 无效液面常量）

- aquifer.h `method_43718`（erosion<-0.225 && depth>0.9）阈值：`-0.225`→`-0.225f`、`0.9`→`0.9f`——Java 源码是 **float 常量比较**（float 提升），C++ 原用 double 字面量等价但类型未对齐；语义零变化，仅对齐 Java 类型。
- `fluidLevel != INT32_MAX` → `!= -32512`——Java `field_35479` = -32512 是**无效液面常量**（「未初始化」哨兵）；C++ 原用 INT32_MAX 魔法数，值等价但语义/可读性差，对齐 Java 常量名。
- 两处均为 judge 建议的顺手对齐（与 06 篇追加 3 同批），8576/3200 回归零退化。
