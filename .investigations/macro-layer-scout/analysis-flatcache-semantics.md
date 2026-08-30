# flat_cache 节点语义判定：vanilla Java 1.20.1 = 4×4 量化网格（A 成立，transpiler 侧为语义 bug）

- **结论**：三选一 → **A 成立**。vanilla `minecraft:flat_cache` = per-chunk 5×5 网格预计算 + **4×4 量化查表**（cell 内任意点取左下角格点值，格点 y=0），不是精确 (x,z) 键缓存。
- **归属**：CoreSwap 运行时（Rust FlatCacheData / C++ FlatCacheDF）与 Java 同构（正确侧）；**transpiler 生成侧把 flat_cache 降级为精确键 `transpiler_cache_2d` 是语义 bug**（仅暴露在「精确点诊断采样」路径）。
- **置信度**：**confirmed**（用户 2026-08-30 MOD 实跑授予；judge review-m13-flatcache-jni.md 3×PASS 在前）。
- **验证分层**：Degraded（静态审查）+ 复用主会话已落盘 runtime 探针解读（探针采集/执行非本 worker，见 cmd-output 落盘引用）。
- **生产影响**：确认**不受影响**（98304 点 diff=0.000000 与本判定自洽），附两个前置条件，见「四、修复方向」末尾评估。
- 产出者：core-worker subagent（隔离，2026-08-30 会话）；只读分析，未修改任何源码。

---

## 一、现象（已实测事实重述）

1. **治理对象**：build-time transpiler ch0（terrain channel）精确点 vs 运行时（macrolize channels[0]）精确点。cell 内部点最大 diff 0.068（@y=60），4 个 cell corner **逐位 0**。
2. **分解实验（`transpiler_ch0_decompose.txt`）**：
   - 4 corner 逐位一致（0.000000×4）→ 双边在 corner 处无差。
   - 内部点 (3,3)：transpiler=0.133102 ≈ 自身 corner 双线性 0.137183（差 0.004081，自洽）；runtime=0.068001，偏离自身 corner 双线性 **−0.069182（不自洽）**——runtime 内部点被某个非平滑项拉离插值曲面，transpiler 没有。这是**量化签名**（quantization footprint）。
   - y 密扫：diff 从 y≈52 启动（7.3e-5），y=60-64 峰值 0.068，之后随 y 近似线性（~0.000488/y 至 y=172）。
   - z=3、x 跨 cell/块扫描：**x%4==0 的点 diff 也非零**（如 local 0 → 0.0201）——量化只在 x、z **同时**取格点角时才退化为一致，z 固定内部时 x 过 corner 仍有差，与 A 语义完全相容。
3. **census（`transpiler_ch0_census.txt`）**：channels[0] Interpolated 残留=0；含 **FlatCache=363**、Cache2D=9、ShiftDF=708（ShiftDF=shift_a/shift_b 708 = 354 个 shifted_noise × 2 个 shift 参数，已核实双边精确键、非混淆源，见定位 6）。
4. **生产路径**：TranspilerDensity vs DensityMacroSampler（双边 cell-corner 采样+插值）98304 点 diff=0.000000。

## 二、根因

### 判定对象

vanilla Java 1.20.1 `minecraft:flat_cache` 节点语义：
- **(A)** 4×4 格点量化缓存（per-chunk 一张 5×5 网格，cell 内匿名点取 cell 左下角格点值）；
- **(B)** 精确 (x,z) 每列缓存。

### 判定：A 成立（Java 反编译源实锤）

**决定性证据** — 本仓库反编译权威源 `versions/1.20.1/data/mc_src_extract/net/minecraft/world/gen/chunk/ChunkNoiseSampler.java` L836-881（`FlatCache` 内部类）：

```java
class FlatCache implements DensityFunctionTypes.Wrapping, ChunkNoiseSampler.ParentedNoiseType {
    private final DensityFunction delegate;
    final double[][] cache;

    FlatCache(DensityFunction delegate, boolean sample) {
        this.delegate = delegate;
        // horizontalBiomeEnd = BiomeCoords.fromBlock(horizontalCellCount(4) * horizontalCellBlockCount(4)) = 4（L139）→ [5][5]
        this.cache = new double[ChunkNoiseSampler.this.horizontalBiomeEnd + 1][ChunkNoiseSampler.this.horizontalBiomeEnd + 1];
        if (sample) {
            for (int i = 0; i <= horizontalBiomeEnd; i++) {          // 5 个格点
                int k = BiomeCoords.toBlock(startBiomeX + i);        // = (startBiomeX + i) * 4，4 块间隔
                for (int l = 0; l <= horizontalBiomeEnd; l++) {
                    int n = BiomeCoords.toBlock(startBiomeZ + l);
                    this.cache[i][l] = delegate.sample(new DensityFunction.UnblendedNoisePos(k, 0, n));  // y=0 采样
                }
            }
        }
    }

    @Override
    public double sample(DensityFunction.NoisePos pos) {
        int i = BiomeCoords.fromBlock(pos.blockX());      // = blockX >> 2（含负数 floor！）——量化
        int j = BiomeCoords.fromBlock(pos.blockZ());      // = blockZ >> 2
        int k = i - startBiomeX; int l = j - startBiomeZ;
        int m = this.cache.length;                        // 5
        return k >= 0 && l >= 0 && k < m && l < m ? this.cache[k][l] : this.delegate.sample(pos);
    }
}
```

- 网格 = 每采样器一张 **5×5**（horizontalBiomeEnd+1），格点间隔 4 块（biome 坐标系），**网格值在 y=0 预采样**；
- `sample()` 对匿名点做 `(x>>2)−startBiomeX` **量化索引**取格点值——**cell 内部点不重算 delegate，直接共享 cell 左下角格点值**；网格外（±越界）才 `delegate.sample(pos)` 精确直算。
- 这就是 (A)。(B) 被源码直接否定。

**同文件对照（内部三类缓存语义各不相同，transpiler 却合并处理）**：

| Java 类 | 位置 | key/取值语义 |
|---|---|---|
| `Cache2D` L557-579 | Cache2D 节点 | 单槽，key=`ChunkPos.toLong(blockX, blockZ)` **精确列**（block 级） |
| `FlatCache` L836-881 | flat_cache 节点 | **5×5 网格、4×4 量化查表**，y=0 预采样，越界直算 |
| `CellCache` L652-681 | cache_all_in_cell 节点 | per-cell 内 4×8×4 偏移索引缓存（与逐点精确缓存在纯函数下值等价） |

**CoreSwap 双侧独立复刻交叉验证（三重一致）**：

1. **Java 权威**（上，源码级）。
2. **CoreSwap C++（block_probe Full 级已验证）**：C++ `FlatCacheDF`（5×5 预计算 + `k/l=(pos>>2)−startBiomeX` 界内查表 + 越界 `delegate.sample(pos)` 直算）六维逐条对齐 Java（07 篇 L126-127 + review-fix-delivery.md 审查要点 1，judge 通过、用户拍板 2026-08-12），vanilla 块级对齐 8576 **99.9994%** / 3200 **99.9997%**。C++ 侧从未对 (A) 提出异议。
3. **CoreSwap Rust 运行时**：`WorldgenRust/src/density.rs` L428-487 `FlatCacheData`（“POC 用 pos>>4；生产 fill 设 g_cur_chunk 对齐 Java startBiomeX”）逐位复刻同一结构（build 的 25 格点 y=0 L483 = Java L851；匿名量化索引 L470-473 = Java L858-864；越界直算 L475 = Java L864）。**运行时是 Java 语义的忠实侧，不是简化 bug**。

**行为探针一致性（复述根因可解释的全部实测）**：

- **corner 逐位一致**：corner 处 `(x>>2)` 恰命中网格槽位，该槽位值 = delegate 在同一精确角点的 y=0 采样值；而 ch0 内所有 flat_cache包裹链均为 y 无关（climate shifted_noise，生成代码可见 `y*0f64` 与 offset 噪声第 2 参恒 0.0），y=0 与实际 y 无差 → (A) 量化值 ≡ (B) 精确值，逐位相等。探针数据（corner 0 diff）与 A 自洽。
- **(3,3) runtime 偏离自身角双线性 −0.069**：runtime 树内 flat_cache 子值被量化到格点（0.0680），而其余项（base_3d_noise 等）逐点精确 → 合成值系统性偏离「全精确」的角点插值曲面；transpiler 全精确 → 只剩 0.004 的树本征非线性残差。若 (B) 成立（即 Java 全为精确缓存），runtime 的 −0.069 量化签名在 CoreSwap 自身代码中无来源，且与 C++/Java 三方块级对齐的历史矛盾。
- **y 轮廓**（y≈52 启动、60-64 峰值、之后线性）：flat_cache 子值是「per-cell 常数」，经树内 y 相关线性结构（yclamp 梯度、depth 的 4× 因子链）映射后产生与 y 斜率成正比的差异；corner 处差异为 0，故只出现在内部点诊断。机制无需逐 y 归因，量级与形态均可由 A 派生。
- **探针实验 2 的判据注释过强（不影响判定）**：「若 (1,1)==(3,3)==(0,0) 则被量化」——A 只量化 flat_cache 子值，树内其它项仍在 cell 内变化，故三值不等**不构成**对 A 的反证；该注释与被测机制不同构（教训见 §五.4）。

因此 diff 的机制 = **runtime 正确复刻了 vanilla flat_cache 的 4×4 量化值语义，transpiler 把 flat_cache 当 cache_2d（精确列缓存）生成，两者在非 corner 点的 flat_cache 子值上分歧**。363 个 FlatCache 节点经 climate spline 链放大，量级 0.068、表面带（y=60-64）最敏感。

### 源头定位（bug 引入点）

- `WorldgenRust/build/density.rs` L234-241：`"minecraft:flat_cache" | "minecraft:cache_2d"` **合并分支**，统一生成 `crate::density::transpiler_cache_2d(id, x, z, || inner)`（精确键）。
- 注释原文即错误假设的载体：*「flat_cache/cache_2d 是 xz-only，用 (x,z) key 正确」*——把「y 无关（2D）」偷换成「逐 (x,z) 精确」，二者不等价。
- 生成产物实证：`WorldgenRust/src/generated/vanilla_density_functions.rs` L163（ch0）`transpiler_cache_2d(1000032, x, z, || transpiler_cache_2d(1000031, x, z, || ...continentalness...))`（flat_cache+cache_2d 链双双坍缩为精确键）；L150-159（ridges/erosion 坐标同样精确键化）。
- 历史脉络（07 篇 L999）：旧 MVP「缓存节点全部内联 inner、不做缓存」——内联 = 逐点精确求值 = 与 (B) 同值，**该简化在 corner-only 生产域确实值等价**；M11 增加缓存映射（cache_once 判错已由 judge 修为 (x,y,z) 键）时，flat_cache 沿用「精确键」结论，**继承了 MVP 时代「只在 corner 域验证过」的隐含前提**而未被重新审视。
- 附：`transpiler_cache_2d` 的 key 编码 `(x<<32)^z`（i64 符号扩展参与 XOR）与 Java `toLong` 位型不同但**仍单射**，不构成正确性问题。

## 三、定位（诊断路径复盘：为什么是这个结论、怎么找到的）

1. **知识库检索**：03 篇（Cache2D block 级 key 修复、node 表）、07 篇（FlatCache 六维对齐修复定论 + L999 MVP 脉络）、10 时间线（L210 continents=flat_cache(shifted_noise) 纯噪声；L1180「Java FlatCache 是 per-chunk 实例：预计算 25 角点、之后纯查表、越界直算不重建」）、algorithm-fingerprints #10/#12——既有知识全部指向「量化网格」侧，无任何 (B) 支持。
2. **Java 权威源码定位**：glob `**/NoiseChunk.java` 无（1.20.1 中 FlatCache 在 ChunkNoiseSampler 内），`ChunkNoiseSampler.java` L836-881 读到完整实现 → 一锤定音。
3. **同文件横向对照**：L557-579（Cache2D 精确列）、L597-650（CacheOnce）、L652-681（CellCache）——确认 vanilla 内部缓存语义分层，排除「flat_cache 名义上也该精确」的可能。
4. **反方假设逐一排除**：
   - ❌ (B)「Java 精确键、CoreSwap 运行时自行量化」——与 Java 源码 L858-864 直接矛盾；且与 C++ 六维对齐修复（judge/用户拍板）+ 三方块级 99.99% 对齐史矛盾。
   - ❌ 「ShiftDF 是第二处偏差源」——运行时 `ShiftDF` L553-593 核实为**精确 (x,z) 单值缓存**（`ShiftSlot.cx/cz` 字段名误导，实际存精确 pos.x/pos.z）；transpiler shift_a/shift_b 亦逐点内联精确求值，双边一致，非混淆源。
   - ❌ 「runtime 偏离角双线性说明 runtime 有 bug」——该偏离正是 flat_cache 量化 + 树内其它精确项合成的预期签名，方向/量级与 A 相容。
   - ❌ cache_all_in_cell→transpiler_cache_3d 精确键 —— 与 Java CellCache 在纯函数下值等价，无相邻 bug（CellCache 只在插值循环内有效，诊断外采样 Java 直接抛异常，不构成第三差异源）。
5. **产物**：本文件；引用探针 `transpiler_ch0_decompose.txt` / `transpiler_ch0_census.txt`（主会话落盘，本 worker 未执行）。

## 四、修复方向（仅方案，不改代码；交主会话裁决）

**修复 = transpiler 生成侧（build/density.rs），不是运行时**。A 成立 → 运行时/C++/生成语义基线不动。

1. **首选（语义对齐）**：拆分 build/density.rs L234 合并分支——
   - `minecraft:cache_2d` 保持 `transpiler_cache_2d` 精确键（现状正确）；
   - `minecraft:flat_cache` 生成 **量化封装**，形如：
     ```
     { let qx = ((x as i64) >> 2 << 2) as f64; let qz = ((z as i64) >> 2 << 2) as f64;
       crate::density::transpiler_cache_2d(id, qx, qz, || { let x = qx; let z = qz; let y = 0.0; (inner) }) }
     ```
     变量遮蔽（shadowing x/z/y）替代子树改写，缓存键自动变 cell 角点值，inner 在 y=0 求值（= Java UnblendedNoisePos(k,0,n)）。嵌套结构（如 L163 的 1000032 外层 flat_cache 包 1000031 内层 cache_2d）须保持：外层量化、内层精确。
   - 注意负坐标必须用 i64 算术右移（`>>2` floor 语义），不得用整除。
2. **备选（文档级最小动作）**：不改代码，在 census/IR 中把 FlatCache 节点标注为「量化语义」，规定精确点诊断必须走运行时 FlatCacheData 参照、禁止与精确键 transpiler 直比。长期仍建议做首选（两个渠道值语义不同迟早再咬人）。
3. **已知边界（修后残留，非本次误差异源）**：Java FlatCache 的查表区间是「采样器自己的 chunk」（per-chunk 实例，越界相对**自身** startBiome 则 delegate 直算，永不建邻居网格）；无状态 transpiler 函数按 pos 推导量化角，跨 chunk 直采点与 Java per-chunk 上下文语义可能仍有差（运行时诊断路径 pos>>4 推导同理）。生产 in-chunk 恒界内，不受影响；诊断跨 chunk 抽查时注意此上下文差。
4. **生产影响评估（任务要求确认）**：**确认不受影响**——
   - 生产双边（TranspilerDensity / DensityMacroSampler）都只在 cell corner（x,z ≡ 0 mod 4）评估 ch0 内容再插值；corner 处量化查表值 = delegate 在该角点的精确值、y=0 且 inner 全 y 无关 → (A)(B)(Java) 三方逐位相等 → 与 98304 点 diff=0.000000 自洽。
   - 该结论的两个前置（当前均成立）：① flat_cache inner 保持 y 无关（climate 树）；② 生产采样保持 corner-only。未来若把 y 相关树包进 flat_cache 或开放内部精确点采样到生产，bug 立即暴露——修复应在那之前完成。
   - 03/06 篇遗留「16 格宽地貌同构划线（疑 FlatCache 网格角点值差）」属 C++ 网格**值**差候选，与本 A/B 语义问题不同源，勿混。

**修复后验证探针（给主会话，本 worker 只设计不改代码）**：
1. 重跑 `transpiler_ch0_decompose` 三实验：预期 (3,3) transpiler 精确值 → 与 runtime 逐位一致（或 ≤1e-12，仅 y 上下文与嵌套键序余差），corner 仍 0，production 98304 保持 0.000000。注意：修复后 transpiler(间) 也量化，「与 runtime 一致」只证双边互证；对 Java 的最终复证用 2。
2. （可选，Java 复证）DensityProbe 存量 GRID dump 通道（10 时间线已有「continents 网格值 dump」先例）扩展：反射 `actualDensityFunctionCache` 中某个 flat_cache-wrapped climate 坐标 DF，对同 cell 内部点 `(cx*4+3, 64, cz*4+3)` 与左下角点分别 `sample(UnblendedNoisePos(...))`，断言二者**逐位相等**（A）——不等即本报告被证伪，回炉。

## 五、教训（错误优先原则：为什么错、下次怎么避）

1. **「缓存」与「值语义」必须分开判**：flat_cache 不是透明缓存，是**量化采样器**——它改变 4×4 cell 内所有匿名点的返回值；名字里的 cache 诱发了 flat_cache|cache_2d 合并处理。判据：拿到 cache 类节点先问「命中时返回的值是**第一次在哪算的**」——cache_2d 在本点算（透明），flat_cache 在 cell 角、y=0 算（不透明）。
2. **注释里的断言要带证据域**：build/density.rs L236「flat_cache/cache_2d 是 xz-only，用 (x,z) key 正确」把「y 无关」偷换成「逐 (x,z) 精确」——正是本 bug 的直接入口。写语义注释时按「y 无关性 / x-z 精度 / 采样点集」三轴陈述，禁止二轴合并。
3. **MVP 简化的适用域要随代码迁移**：旧 MVP「内联=语义等价」在 corner-only 生产域成立；M11 缓存化复用该结论时隐含前提未随迁（`假设A在域D成立 → 为D'写新代码时静默沿用`）。凡是继承临时简化的重构，必须在注释/产物里显式重立「在哪个采样点集取值域内等价」。
4. **探针判据要与被测机制同构**：实验 2 注释「(1,1)==(3,3)==(0,0) 才是量化」对整通道值过强（树内非 flat_cache 项仍逐点变化），差点造成「runtime 未量化」的误读。设计探针判据前先写明被测子机制的预期签名（本例应查 flat_cache 子值或角点+内部点差分模式）。
5. **命名陷阱**：运行时 `ShiftSlot { cx, cz }` 实存精确 x/z（非 chunk 坐标）——本次分析差点把其判成「第二处 chunk 级量化偏差」。字段名与语义错位是静态审查的高频误判源，审查时以 key 构造行（L566/L581）为准，不以字段名推断。
6. **置信度纪律**：本结论三重证据（Java 反编译源 + C++/Rust 双侧复刻史 + 探针行为签名）已充分，但按契约仅落 draft，候选/确诊由主会话 judge 链裁决；Java 运行时侧复证探针已备（§四.4.2）。

---

### 附：证据文件清单

- 探针数据（主会话落盘）：`E:\PYTHON\CoreSwap\.investigations\macro-layer-scout\cmd-output\transpiler_ch0_decompose.txt`、`transpiler_ch0_census.txt`
- Java 权威源：`E:\PYTHON\CoreSwap\versions\1.20.1\data\mc_src_extract\net\minecraft\world\gen\chunk\ChunkNoiseSampler.java` L836-881（FlatCache）、L557-595（Cache2D）、L652-681（CellCache）、L139（horizontalBiomeEnd=4）
- 运行时：`WorldgenRust/src/density.rs` L428-487（FlatCacheData，A 语义复刻）、L553-593（ShiftDF 精确键核实）、L396-427（Cache2D 精确键）、L335-395（transpiler_cache_2d/3d）
- 生成/映射：`WorldgenRust/build/density.rs` L234-249（bug 引入点）、`WorldgenRust/src/generated/vanilla_density_functions.rs` L150-172（ch0 双缓存坍缩实证）
- 知识库：`versions/1.20.1/docs/03-density-functions.md`（Cache2D key 修复）、`07-block-pipeline.md` L46-50/L126-127/L999（FlatCache 六维对齐 + MVP 简化溯源）、`10-timewise-archive.md` L210/L1180、`knowledge/discovered/algorithm-fingerprints.md` #10/#12
