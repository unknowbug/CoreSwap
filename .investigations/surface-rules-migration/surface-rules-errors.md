# ③ vanilla 块管线 — surface rules 移植错误台账（阶段 A）

> 载体：`.investigations/surface-rules-migration/surface-rules-errors.md`（错误台账，独立成篇）
> 课题：WorldgenRust/「③ vanilla 块管线」阶段 A（surface rules 移植）
> 来源：本 session 移植 vanilla surface rules（条件/规则/规则树/buildSurface 引擎）时，发现并修正 4 个 C++ 参考实现 bug（`versions/1.20.1/cpp/worldgen/src/surface.h`）。
> 权威：Java `mc_src_extract/net/minecraft/world/gen/surfacebuilder/`（MaterialRules.java / SurfaceBuilder.java / VanillaSurfaceRules.java）。
> 状态：draft（草稿，待主会话应用 + 验证签核）
> 日期：2026-08-28

> **核心判错经验（先读）**：C++ 参考实现**不是权威**——它是从 Java 移植的中间产物，可能带 bug。移植时**条件/谓词的字段语义、便捷方法的内部缩放、数组索引约定、上下文初始化完整性**四类点，都要对照 Java 权威逐项核对，不能照抄 C++。本台账 4 条错误全部属于这四类之一。

---

## 错误1：HoleCond 用错字段（扫描计数器 ≠ 噪声值）

### 现象
- C++ `surface.h` L251 `HoleCond::test` 用 `ctx.stoneDepthAbove <= 0` 判定洞。
- Java `MaterialRules.java` L537 `NegativeRunDepthPredicate.test` 用 `context.runDepth <= 0` 判定洞。
- 两者判定字段不同，C++ 与 Java 语义不一致。

### 根因
- Java `context.runDepth` = `sampleRunDepth(blockX, blockZ)`（`SurfaceBuilder.java` L172-175 = surfaceNoise + random 噪声值，L459 设置一次列级）——是**噪声值**。
- C++ `ctx.surfaceDepth = sampleRunDepth(m, n)`（`surface.h` L749）才对应 Java 的 `runDepth`；而 `ctx.runDepth` 是**死字段**（L750 设 0 从未更新）。
- C++ `HoleCond` 误用 `ctx.stoneDepthAbove`（= q 扫描计数器，逐列扫描时递增的计数）来判定洞，语义与 Java 的「噪声值 runDepth」完全不同。
- 本质：**「扫描计数器」vs「噪声值」是两类不同语义**，混用必然错。

### 定位
- recode.scout 勘探时指出 HoleCond 语义存疑（pipeline-map.md 风险 #1）。
- 读 Java `MaterialRules.java` L537 + `SurfaceBuilder.java` L172-175，确认 `runDepth = sampleRunDepth`（噪声）。
- 对照 C++ `surface.h` L251（HoleCond 用 stoneDepthAbove）与 L749/L750（surfaceDepth=sampleRunDepth、runDepth 死字段），确认用错字段。

### 修复
- Rust 移植 `HoleCond` 改用 `ctx.surface_depth <= 0`（对齐 Java `runDepth = sampleRunDepth` 噪声值）。

### 教训
- 条件/谓词的字段语义要对照 Java 权威**逐字段核对**，不能照抄 C++（C++ 可能有 bug）。
- 「扫描计数器」与「噪声值」是两类不同语义，混用会错；移植前先分清字段属于哪类。
- 死字段（设 0 从未更新）是 C++ 移植 bug 的高发信号——遇到「设了但从不更新」的字段，先查它是否本应承载某语义。

---

## 错误2：surfaceNoiseThreshold 未除 8.25（便捷方法内部缩放被漏）

### 现象
- C++ `surface.h` L519 等 `noiseThresholdNoMax("minecraft:surface", 1.0)` 直接用 1.0。
- Java `VanillaSurfaceRules.java` L391-392 `surfaceNoiseThreshold(min)` = `noiseThreshold(SURFACE, min/8.25, MAX)`——min 先除 8.25。

### 根因
- Java 的 `surfaceNoiseThreshold` 是便捷方法，内部对阈值做了 `min/8.25` 缩放（surface 噪声缩放）。
- C++ 直接照抄「便捷方法调用处的字面值」（1.0 等），**漏掉了便捷方法内部的 /8.25 缩放**，导致阈值偏大 8.25 倍。

### 定位
- worker 交付时指出 surfaceNoiseThreshold 存在 /8.25 分歧。
- 读 Java `VanillaSurfaceRules.java` L391-392 确认 `surfaceNoiseThreshold(min) = noiseThreshold(SURFACE, min/8.25, MAX)`。
- 对照 C++ `surface.h` L519 确认直接用 min 未除 8.25。

### 修复
- Rust 移植所有 `surfaceNoiseThreshold` 值（windswept_* 系列 1.0 / 1.75 / 2.0 / -1.0 / -0.5 / -0.95）统一除 8.25。

### 教训
- Java 的**便捷方法内部有缩放**（如 surfaceNoiseThreshold 的 /8.25），照抄 C++ 的「直接值」会漏掉缩放。
- 移植时对 Java 便捷方法**要展开看内部实现**，不能只看调用处字面值；凡「方法名带语义（threshold/noise/scale）」的，先查其内部公式。

---

## 错误3：SteepCond 索引转置（heightmap 索引约定读反）

### 现象
- C++ `surface.h` L252-261 `SteepCond::test` 读 `columnHeightmap[i*16+j]`（i=x, j=z）。
- 但 heightmap 填充为 `z*16+x`（`worldgen_api.cpp` L1045 实证）。

### 根因
- heightmap 数组索引约定是 `z*16+x`（z 为外层、x 为内层）。
- C++ `SteepCond` 读 `x*16+z`（i=x 外层、j=z 内层），**索引转置**，导致 steep 判定读错邻居（把 (x,z) 的邻居读成 (z,x) 的邻居）。

### 定位
- worker 交付时指出 SteepCond 索引存疑。
- 对照 Java `SteepSlopePredicate` + `worldgen_api.cpp` L1045，确认 heightmap 填充语义为 `z*16+x`。

### 修复
- Rust 移植 `SteepCond` 用 `hm[(z±1)*16+x]` / `hm[z*16+(x±1)]`（对齐 Java，z 外层 x 内层）。

### 教训
- heightmap/数组的**索引约定（z*16+x vs x*16+z）要对照填充方（worldgen_api.cpp）确认**，不能假设。
- 转置 bug 会导致读错邻居，且通常不报错、只产生「看起来合理但错」的结果——比崩溃更难发现。
- 判错经验：**符号级/结构级错误（索引、坐标、公式）优先于精度级**——先查索引/坐标/公式，别在精度上纠结。

---

## 错误4：apply_material_rule_single 留 surface_depth=0（上下文初始化不完整）

### 现象
- C++ `surface.h` L437-440 `applyMaterialRuleSingle` 构造 SurfaceContext 时 `surfaceDepth` 留 0。
- Java `initHorizontalContext` 会设 `surfaceDepth = sampleRunDepth`。

### 根因
- C++ `applyMaterialRuleSingle` 未设 `surfaceDepth`（留默认 0），导致单点规则应用时 `surfaceDepth` 相关条件（aboveY / water / stoneDepth 的 addSurfaceDepth）用错值（0 而非真实噪声值）。

### 定位
- worker 交付时指出单点规则上下文初始化存疑。
- 对照 Java `initHorizontalContext` 确认其会设 `surfaceDepth = sampleRunDepth`。

### 修复
- Rust 移植 `apply_material_rule_single` 设 `surface_depth = sample_run_depth(x, z)`。

### 教训
- **单点规则应用的上下文初始化要完整**（surfaceDepth 等列级字段不能留默认值）。
- 上下文/上下文构造器是「字段漏设」高发区——构造时对照 Java 的 init 函数逐字段核对，凡 Java 初始化了而 C++ 留默认值的字段，都是潜在 bug。

---

## 「错误 → 根因」速查表

| # | 错误 | 根因（一句话） | 修复要点 |
|---|------|----------------|----------|
| 1 | HoleCond 用 `stoneDepthAbove`（扫描计数器）判定洞 | Java `runDepth` 是噪声值（sampleRunDepth），C++ 误用扫描计数器 q | Rust 改用 `ctx.surface_depth <= 0` |
| 2 | surfaceNoiseThreshold 直接用 1.0 等字面值 | Java 便捷方法内部有 `min/8.25` 缩放，C++ 漏掉 | 所有 surfaceNoiseThreshold 值除 8.25 |
| 3 | SteepCond 读 `x*16+z` | heightmap 填充是 `z*16+x`，索引转置读错邻居 | Rust 用 `hm[(z±1)*16+x]` / `hm[z*16+(x±1)]` |
| 4 | applyMaterialRuleSingle 留 surface_depth=0 | 单点规则上下文未设 surfaceDepth（Java initHorizontalContext 会设） | Rust 设 `surface_depth = sample_run_depth(x,z)` |

**共性根因**：C++ 参考实现是从 Java 移植的中间产物，非权威；四类易错点 = ① 字段语义（扫描计数器 vs 噪声值）② 便捷方法内部缩放 ③ 数组索引约定 ④ 上下文初始化完整性。移植时对 C++ 一律对照 Java 权威逐项核对。
