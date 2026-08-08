# Phase 6 — aquifer density 输入语义差假设验证：CellCache vs Beardifier

> 角色：recode.scout（勘探）
> 任务：验证「aquifer 的 density 输入语义差：Java 用 CellCache 缓存值（cell 角点/恒定），C++ 用块级插值」——**结论：假设被推翻，真正根因是 Beardifier（StructureWeightSampler）缺失**
> 种子：seed=-8248318472910187742；列 (-244,-256)；NOISE 阶段岛 y=58-61
> 日期：2026-08-08（与 density.h L468 注释同日确认）

---

## 0. 结论摘要（TL;DR）

| # | 结论 | 置信度 |
|---|------|--------|
| 1 | **CellCache 缓存/返回的是「cell 内块位置的三线性插值（finalDensity + Beardifier）」，不是角点值、不是恒定值** | 【确定】（源码行号支撑） |
| 2 | Java aquifer 输入 = `CellCache(Add(InterpolatedDF(finalDensity), StructureWeightSampler)).sample`——**块生成路径 Beardifier 不是 0** | 【确定】 |
| 3 | **假设（CellCache 角点/恒定 vs 块级插值）被推翻**：Java 与 C++ 的插值语义一致，差异在 **Beardifier（结构密度修正）缺失** | 【确定】 |
| 4 | phase5「Beardifier 恒 0（贡献=0）」**结论错误**：误读 NoiseChunkGenerator.java:205（高度图路径），块生成路径是 StructureWeightSampler（L102-111） | 【确定】 |
| 5 | 03 篇旧结论「Java CellCache 缓存同 pos 同值，C++ 纯委托等价（无损）」**正确，无需推翻**；但需补充：aquifer 输入链缺 Beardifier 项 | 【确定】 |
| 6 | -288 岛（13000 块水体边界）= **Beardifier 缺失**；8576/20000 边界翻转（21+18 块）= **插值精度边界翻转**，**不同机制**（任务「同机制」猜想不成立） | 【确定】机制不同 / 【推测】8576 与结构无交集 |

---

## 1. CellCache 语义定论【确定】

### 1.1 CellCache 类结构（ChunkNoiseSampler.java L652-701）

```
L652  class CellCache implements DensityFunctionTypes.Wrapper, ParentedNoiseType {
L653    final DensityFunction delegate;
L654    final double[] cache;                              // ← 缓存数组
L656    CellCache(DensityFunction delegate) {
L658      this.cache = new double[horizontalCellBlockCount
L659        * horizontalCellBlockCount * verticalCellBlockCount];   // 4×4×8 = 128（主世界）
L661      ChunkNoiseSampler.this.caches.add(this);
L665    public double sample(NoisePos pos) {
L666      if (pos != ChunkNoiseSampler.this) return this.delegate.sample(pos);  // 非本 chunk → 直算
L668      else if (!isInInterpolationLoop) throw IllegalStateException(...);
L670      else {
L671        int i = cellBlockX, j = cellBlockY, k = cellBlockZ;   // 当前 cell 内块位置
L680        return this.cache[((verticalCellBlockCount - 1 - j) * horizontalCellBlockCount + i)
L682                              * horizontalCellBlockCount + k];                // ← 按 cell 内位置索引缓存
L683        ...
```

- cache 大小 = 一个 cell 内的块数（主世界 4×4×8 = 128）
- `sample` 在插值循环内、pos == this 时，用**当前 cell 内块位置**（cellBlockX/Y/Z）索引 cache 数组返回

### 1.2 cache 填充时机与值来源（ChunkNoiseSampler.java L342-355 + L313-328）

```
L342  public void onSampledCellCorners(int cellY, int cellZ) {   // cell 角点采样完成后调用
L343    this.interpolators.forEach(interpolator -> interpolator.onSampledCellCorners(cellY, cellZ)); // 拷贝 8 角点
L344    this.isSamplingForCaches = true;
L349    for (CellCache cellCache : this.caches) {
L350      cellCache.delegate.fill(cellCache.cache, this);          // ← 对 cell 内每个块位置采样
L351    }
L354    this.isSamplingForCaches = false;
```

```
L313  public void fill(double[] densities, DensityFunction densityFunction) {
L314    this.index = 0;
L316    for (int i = verticalCellBlockCount - 1; i >= 0; i--) {   // Y 顶→底
L319      for (int j = 0; j < horizontalCellBlockCount; j++) {    // X
L322        for (int k = 0; k < horizontalCellBlockCount; k++) {  // Z
L324          densities[this.index++] = densityFunction.sample(this);   // 逐块位置采样
```

- 填充时机：每个 cell 的 8 个角点密度采样完毕（`onSampledCellCorners`），对 cell 内 4×4×8=128 个块位置逐一采样
- `delegate` = `Add(InterpolatedDF(finalDensity), Beardifier)`（L177-181 构造）

### 1.3 填充时刻 InterpolatedDF 返回什么（ChunkNoiseSampler.java L786-808）

```
L792    return isSamplingForCaches
L793      ? MathHelper.lerp3(
L794          (double)cellBlockX / horizontalCellBlockCount,     // cell 内块位置归一化
L795          (double)cellBlockY / verticalCellBlockCount,
L796          (double)cellBlockZ / horizontalCellBlockCount,
L797          this.x0y0z0, this.x1y0z0, ... this.x1y1z1)         // 8 个 cell 角点
L806      : this.result;                                         // 插值循环中的三线性结果
```

**决定性：`isSamplingForCaches=true` 时 InterpolatedDF.sample 返回 `lerp3(cell 内块位置比例, 8 角点)` = cell 内块位置的三线性插值。**

### 1.4 语义定论

- CellCache 缓存的是 **cell 内每个块位置（4×4×8=128 个）的 finalDensity+Beardifier 三线性插值**
- aquifer 调用时 `densityFunction.sample(pos)`（pos==this）→ 按当前块在 cell 内位置取回**该块位置的插值**
- **这不是角点值、不是 cell 恒定值**——是逐块三线性插值，与 C++ `InterpolatedDF::sample`（density.h L486-547）**语义一致**
- CellCache 唯一作用是「cell 内同位置多次采样免重算」——纯缓存优化，**无损**

---

## 2. Java aquifer 实际输入值【确定 + 推断】

### 2.1 输入链（ChunkNoiseSampler.java L176-181）

```
L176  Builder<BlockStateSampler> builder = ImmutableList.builder();
L177  DensityFunction densityFunction = DensityFunctionTypes.cacheAllInCell(
L178        DensityFunctionTypes.add(noiseRouter2.finalDensity(), DensityFunctionTypes.Beardifier.INSTANCE)
L179      ).apply(this::getActualDensityFunction);
L181  builder.add(pos -> this.aquiferSampler.apply(pos, densityFunction.sample(pos)));
```

- `noiseRouter2.finalDensity()` = 已应用 `getActualDensityFunction` → finalDensity 树的 `interpolated` 节点被替换为 `DensityInterpolator`（L159、L452）
- `Beardifier.INSTANCE` 在 `getActualDensityFunctionImpl` 中被替换为 **`this.beardifying`**（L469-471）
- **关键：`this.beardifying` 在块生成路径是 `StructureWeightSampler`，不是恒 0 的占位符！**

### 2.2 beardifying 的两条构造路径（NoiseChunkGenerator.java）

| 路径 | 传入的 beardifying | 行号 | 用途 |
|------|-------------------|------|------|
| `populateNoise`（块生成） | `StructureWeightSampler.createStructureWeightSampler(world, chunk.getPos())` | L102-111（`createChunkNoiseSampler`），L359-362 调用 | **真实块生成 → aquifer 输入含结构权重** |
| `sampleHeightmap`（高度图） | `DensityFunctionTypes.Beardifier.INSTANCE`（恒 0） | L199-209 | 高度图/方块采样探针，不含结构 |

- `StructureWeightSampler`（net.minecraft.world.gen.StructureWeightSampler）实现 `DensityFunctionTypes.Beardifying`（L21），`sample` 逐 Piece/Junction 计算结构权重（L80-120）
- `DensityFunctionTypes.Beardifier.INSTANCE.sample` 恒返回 0.0（L294-296）——**只是注册表占位符**，真正逻辑由 `getActualDensityFunctionImpl` 替换

### 2.3 (-244,58,-256) 的实际输入【推断·高置信】

```
Java aquifer 输入 = 块级三线性插值(finalDensity) + StructureWeightSampler(58)
                 = -0.0744 + Beardifier(58)          （需 > 0 才判 solid）
```

- 块级插值 finalDensity(y=58) = **-0.0744**：phase5 用 cns idx0 验算 squeeze(0.64×(-0.233015)) = -0.074427；C++ trace_aqf_1 L7 同 -0.074424（≤3e-6）【确定】
- Java 真实判定 NOISE-BLK y=58-61 = stone（raw=1），y=51-57 water，y=62 water【确定】（noiseblk_blockprobe.txt L44-50）
- aquifer 判定 `density > 0 → null(stone)`（aquifer.h L70-71 对齐 Java）→ Java 在 y=58 输入**必 > 0**
- → **StructureWeightSampler(58) > +0.0744**（把 -0.0744 顶正）；而 y=56（插值 -0.0534）判 water → Beardifier(56) ≤ +0.053【推断·高置信】
- 结构权重在 y=58-61 正贡献（约 +0.08~+0.12），上下边缘衰减——与 StructureWeightSampler 的 BURY 分支（`getMagnitudeWeight`，量级 0~1，L132-135）一致【推测】

### 2.4 探针 densFn 值 ≠ aquifer 输入【确定】

- phase5（L14、L74-86）已裁定：AQF-J 反射调用走 CellCache.sample 的缓存分支，但反射不触发 `onSampledCellCorners` 重填 → 返回**最后遍历 8-cell 的错误缓存值**，densFn 3 次运行不同是污染证据【确定】
- 补充：若探针传外部 `UnblendedNoisePos`，则 CellCache.sample L666 走 `delegate.sample(pos)` → DensityInterpolator.sample L787 走 `delegate.sample(pos)`（原始 finalDensity 直算，**无 cell 网格插值**）→ densFn = 未插值 finalDensity + Beardifier，也非 aquifer 输入【推测】
- **无论哪条路径，densFn 的 +0.037~+0.048 都不能当作 aquifer 输入证据**；真实判定以 NOISE-BLK 块状态为准【确定】

---

## 3. 假设验证：被推翻【确定】

### 3.1 假设内容

> 「Java 用 CellCache 缓存值（cell 角点/恒定），C++ 用块级插值」→ 判 solid 差

### 3.2 推翻证据链

| 环节 | 证据 | 置信度 |
|------|------|--------|
| (a) CellCache 缓存的是 cell 内块位置的三线性插值 | ChunkNoiseSampler L313-328 + L792-806（lerp3） | 【确定】 |
| (b) Java 与 C++ 插值语义一致 | Java DensityInterpolator 8 角点 lerp3（L786-808）↔ C++ InterpolatedDF lerp（density.h L529-537）；phase5 验算 -0.074427 ≈ C++ -0.074424（≤3e-6） | 【确定】 |
| (c) Java 真实判 solid（岛） | NOISE-BLK y=58-61 stone（noiseblk_blockprobe.txt） | 【确定】 |
| (d) Java 插值 finalDensity(y=58) = -0.0744 < 0 | phase5 cns 验算 + C++ trace 一致 | 【确定】 |
| (e) (c)(d) 矛盾 → aquifer 输入必有额外正贡献（唯一候选 = StructureWeightSampler） | ChunkNoiseSampler L177-181（add finalDensity + Beardifier） | 【推断·高置信】 |
| (f) C++ densityBuf = 纯 finalDensity，无 Beardifier | worldgen_api.cpp L619 `densityBuf[...] = h->finalDensity->sample(fpos)`；grep structure/junction 零匹配 | 【确定】 |
| (g) C++ 因此判 water | trace_aqf_1 L4-12（y=55..62 全负）→ aquifer.h L71 → FLUID | 【确定】 |

**结论：差异不在 CellCache 语义，而在 `add(finalDensity, Beardifier)` 的第二个加项——C++ 整个缺失。**

### 3.3 佐证：已有代码注释自认

- density.h L468：`@anchor.idk("结构 Beardifier 密度修正未实现：结构附近 density 差 ~0.12 可翻转 aquifer 判定（-288 岛缺失根因，2026-08-08 确认）")`
- worldgen_api.cpp L569-571 注释自称「与 Java CellCache(add(DensityInterpolator(finalDensity), Beardifier)) 语义一致」——**注释与实现不符**：实现只做了 `finalDensity->sample`，没加 Beardifier 项

### 3.4 phase5「Beardifier 恒 0」为何错误【确定】

phase5 L104 依据：
- `DensityFunctionTypes.Beardifier.INSTANCE` 恒 0（L294-296）——真，但那是**注册表占位**
- `NoiseChunkGenerator.java:205 传入的就是 INSTANCE`——**误读**：L199-209 是 `sampleHeightmap`（高度图路径，非块生成）

块生成路径 `populateNoise`（L359）通过 `createChunkNoiseSampler`（L102-111）传入 `StructureWeightSampler.createStructureWeightSampler`（L106）。phase5 未覆盖此路径 → 得出「Beardifier 贡献 = 0」错误结论，进而误判「两测 density 输入同符号（负）」。

---

## 4. 影响面评估

### 4.1 -288 岛 / 13000 块水体边界

- 机制 = **Beardifier（StructureWeightSampler）缺失**（本案例，§3）
- 影响范围：结构（有 terrain adaptation 的 structure start + pieces + jigsaw junctions）附近的所有块——C++ 全部缺失
- 这不是 CellCache 语义问题，**修复 CellCache/插值无济于事**（Java 插值本来就与 C++ 一致）

### 4.2 8576 / 20000 边界翻转（21+18 块课题）

任务问「是否同机制（cell 内符号翻转 cell 恒用角点值）」——**不是**：

| 课题 | 机制 | 依据 |
|------|------|------|
| -288 岛 | Beardifier 缺失（结构权重差 ~+0.11 翻转判定） | 本分析 |
| 8576 21 块（深板岩/水边界 12 + 地表三连 9 + river 1） | 块级 finalDensity **插值精度边界翻转**（candidate 待立项） | docs/07-block-pipeline.md L168-179 |
| 20000 18 块（river/taiga 边界） | 与 8576 同族，并入 21 块课题 | docs/07-block-pipeline.md L179 |

- 「cell 恒用角点值」不成立（§1.4 证明 CellCache 是逐块插值），所以任务假设的「同机制」**不成立**
- 8576 21 块区域 (720,-432) 6×6 为普通地形（forest/savanna/river 边缘），无结构交集【推测】——与 Beardifier 无关；但**无法完全排除**少数块恰好落在结构 Beardifier 范围内（需验证）
- 若后续 21 块课题定位到「插值差 ~0.06@y60」（docs/10-timewise L518）仍在符号临界点徘徊，与 -288 的 +0.11 结构差**不同量级、不同来源**

### 4.3 对既有对齐率的影响

- -288 当前 95.7376%（trace_aqf_1 L145）结案基线含大量结构/FEATURE 假 diff；本机制解释其中**结构性固体缺失**（岛）部分
- 8576 99.9994% / 3200 99.9997% / 20000 99.9989%——这些区域若不含结构，Beardifier 贡献恒 0，**不受影响**；含结构则也会缺【推测】

---

## 5. 修复方向（建议，不修代码）

### 5.1 核心：复刻 StructureWeightSampler

C++ 需要实现（对齐 net.minecraft.world.gen.StructureWeightSampler L21-174）：

1. **STRUCTURE_WEIGHT_TABLE**：24³ 静态查表（L24-32、L162-169），`weight = e^(-(x²+(y+0.5)²+z²)/16)`
2. **Piece 贡献**（L86-106）：
   - 对每个 `StructurePiece`（bbox + terrainAdaptation + groundLevelDelta）
   - `m = max(0, max(minX-x, x-maxX))`，`n = max(0, max(minZ-z, z-maxZ))`，`p = j - (minY+delta)`
   - `BURY → getMagnitudeWeight(m, p, n)`（clampedMap(magnitude, 0, 6, 1, 0)，L132-135）
   - `BEARD_THIN/BEARD_BOX → getStructureWeight(m, p, n, p) × 0.8`（L140-152）
3. **JigsawJunction 贡献**（L110-116）：`getStructureWeight(r, l, m, l) × 0.4`
4. 汇总 `d += ...` 返回

### 5.2 注入位置（C++ 代码位置建议）

- **唯一需要改的采样点**：`worldgen_api.cpp L619` densityBuf 填充处
  ```
  densityBuf[by*256 + bz*16 + bx] = h->finalDensity->sample(fpos) + beardifier->sample(fpos);
  ```
  （`hasAquifer` 为 true 时；Beardifier 只进 aquifer 输入，不进 surface/est）
- 或更贴近 Java 结构：在 `density_builder.h` 构建 `final_density` 树时包 `add(finalDensity, Beardifier)`——但 C++ 的 `InterpolatedDF` 只包 `interpolated` 节点，Beardifier 在 interpolated 外（Java 同），**直接改 L619 更简单且等价**
- `estimateSurfaceHeight` / surface 阶段**不用改**：Java 高度图路径 Beardifier = 0（sampleHeightmap L199-209）【确定】

### 5.3 前置依赖（工作量大，需立项）

- C++ 目前**无任何结构生成代码**（grep structure/junction 零匹配）——StructureWeightSampler 需要结构 start + pieces + junctions 数据
- 短期验证路径：**注入式 Beardifier**——从 Java/参照提取 (-288) 区域结构 piece 列表（bbox + terrainAdaptation + groundLevelDelta + junctions）硬编码/查表，验证 y=58-61 是否翻 solid（预期 +0.08~+0.12）
- 长期：实现 StructureSet → StructureStart → pieces 生成（依赖完整结构系统，超出本课题）

### 5.4 验证判据

- 注入 Beardifier 后：(-244,58..61,-256) 应从 FLUID → SOLID（densityBuf + B > 0），NOISE-BLK 对齐 y=58-61 stone
- 交叉验证：densFn 探针的「y=52/58/60 3 次变化 vs y=48/56/64 恒定」应可复现——Beardifier 在结构范围（y≈52-60）非零、cell 底角点（y=48/56/64）为零【推测】

---

## 6. 旧结论验证

### 6.1 03 篇「Java CellCache（cache_all_in_cell）缓存同 pos 同值，C++ 纯委托等价（无损）」【确定·保留】

- docs/03-density-functions.md:93 原文，density_builder.h L155-157 实现
- **正确**：
  - 「缓存同 pos 同值」= cell 内同块位置多次采样同值（§1）✓
  - 「C++ 纯委托等价（无损）」= 无 Beardifier 时 C++ 块级插值 = Java cache 值（§1.3/§3.2-b）✓
  - 「CellCache 反射污染不可信」= phase5 复现 ✓
- **需补充（非推翻）**：该等价只对 density 函数本身成立；**aquifer 输入链的第二个加项 StructureWeightSampler 缺失**，C++ 不等价于完整 `add(finalDensity, Beardifier)`

### 6.2 phase5「Beardifier 恒 0（贡献=0）」【确定·推翻】

- 依据 NoiseChunkGenerator.java:205（高度图路径）——块生成路径是 StructureWeightSampler（L102-111）
- phase5 L170 表格「Java aquifer density 输入 = CellCache(add(finalDensity, Beardifier=0)).sample」→ 改为 `add(finalDensity, StructureWeightSampler)`
- phase5「Java 噪声阶段 density(58) = -0.0744（负）→ 若判定链与 C++ 完全一致，Java 应判 water」的推理本身对，但前提（判定链完全一致）**因 Beardifier 不成立**——Java 判定链多一个 StructureWeightSampler 项

---

## 7. 探针 densFn 异常（3 次变化 + y=48/56/64 恒定）【推测】

数据（noiseblk_blockprobe.txt / aqfj_blockprobe.txt）：
- y=48 / 56 / 64（cell 底角点）：3 次全 = 0.037482（恒定）
- y=52 / 58 / 60（cell 内）：3 次不同（0.036956/0.041260/0.051064 等）

解释候选：
1. **Beardifier 存在性佐证**：若 Beardifier 恒 0（phase5 结论），densFn 应全恒定（纯确定性函数）——但 y=52/58/60 变化，说明这些高度有非零结构权重、且 3 次运行 piece 集不同；y=48/56/64 恒定说明这些高度无结构权重【推测】
2. 反射污染（phase5 主裁定）：返回最后遍历 8-cell 缓存值——但该解释难解释「y 依赖的恒定/变化分组」【推测】

结论：densFn 值不可信（phase5 裁定【确定】），但其「y 分组变化」模式与 Beardifier 空间分布**不矛盾且倾向支持**（注：此条仅佐证，非核心证据）。

---

## 8. 下一步建议（交主会话裁决）

1. 【架构变更建议·需裁决】实现注入式 StructureWeightSampler（§5.3 短期路径），验证 -288 岛
2. 【需裁决】8576/20000 21+18 块课题维持「插值精度边界翻转」方向（与 -288 不同源），可并行验证其区域是否有结构（排除 Beardifier 混入）
3. 【需裁决】phase5「Beardifier=0」结论作废，更新 03 篇补充项
