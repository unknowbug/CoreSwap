# 载具差异裁决（260905 · STEP 1 · candidate）

## 证据（静态读码，runtime/1.20.1/java）
1. `NoiseChunkGeneratorMixin.java`：cppReplace 路径只拦截两个方法——
   - `populateNoise` @HEAD cancellable → `CppBridge.fillChunk(chunk)`（JNI `fillBlocks`，整块 C++ 生成方块+高度图）
   - `buildSurface` @HEAD cancellable → ci.cancel()（surface 由 C++ 在 wg_fill_blocks 内部完成）
2. **全 src 无 FEATURE 阶段 hook**：generateFeatures 仅有的 mixin 全是探针（DiagFeatureBiome/BlobProbe/ColProfProbe/ConfiguredFeatureProbe），无 cancellable 拦截。
3. `CppBridge.java` L64 注释自证：「存档链路关 Rust carver/features（这两阶段由 Java vanilla 执行）」。
4. `build.gradle` L75/L110：`-PcppReplace` → `-Dcpp.replace=true/1`；`BenchMod` server started 时 init CppBridge。

## 裁决（candidate）
- **mod cppReplace 载具 = 混合载具**：C++ 执行 NOISE（含 noise 级 vein，见 vanilla populateNoise 内 oreVein 步骤）+ SURFACE；**CARVER + FEATURE（装饰阶段全部 ore/disk/树）由 Java vanilla 在 C++ 地形上执行**。
- **block_probe -features 载具 = C++ 全程载具**（NOISE+SURFACE+CARVER+FEATURE 全 C++）。
- **生产语义 = C++ 全程接管**（项目目标）→ block_probe 是目标语义验证载具；mod 路径是过渡期存档链路混合载具。
- 两载具残差族不同是**结构性必然**：mod 路径 ore 由 Java 放置（origin 同 vanilla），block_probe 路径 ore 由 C++ 放置（origin 由 C++ 计算）——前者残差反映「C++ 地形 × Java feature 谓词交互 + C++ NOISE/SURFACE 差异」，后者反映「C++ feature 复刻保真度」。残差课题应分别在各自载具上归因，禁止互引（§9.7）。

## 对 STEP 2（出界 dirt 写者）的直接推论
- mod 路径 y>200 stone→dirt：Java feature origin 与 vanilla 完全一致（同 seed）→ 若写者是 Java ore，vanilla 世界也应出现（vanilla 无此 dirt）→ **写者候选收窄为 C++ 侧（NOISE vein / SURFACE）或 Java feature 谓词对 C++ 地形的差异化响应**。
- 判别探针（decisive）：取出界 dirt cell 坐标，dump C++ NOISE 阶段（feature/surface 前）方块 → 该处 NOISE 已是 dirt ⇒ 写者在 NOISE（vein 代码）；仍 stone ⇒ 写者在 SURFACE 或下游。
- 注：上轮 [ORIGIN] 审计用 C++ FEATURELOG（worldgen_api.cpp），测的是 **block_probe 载具** 的 C++ feature origin——「ore_dirt 零出界」结论只适用 C++ 载具，不适用 mod 载具（mod 载具 ore origin 是 Java 算的，与 vanilla 相同）。

状态：draft→candidate 待 judge。证据源：本文件引用的 4 处静态读码。

## ⚠️ 勘误与取代记录（260905 judge 后补，原文不改写）
1. **执行体口径勘误**：本篇「C++ 执行 NOISE（含 vein）+ SURFACE」对**被分析的残差数据集不成立**——现役 dll（EC4A9603 前后 EC4A9AED，260903-03）实为 **Rust WorldgenRust.dll 改名**（build.gradle L27-47，2026-08-30 C++→Rust 转向），mod 载具 NOISE+SURFACE 执行体 = Rust；且旧 dll 无 stage-skip，其 features 亦全量运行（见 writer-verdict-260905 / b2 证据 E1-E6）。本篇读码结论只对**修复后代码**成立。
2. **STEP 2 推论取代**：本篇「对 STEP 2 的直接推论」（写者候选收窄至 C++ 侧/Java 谓词）被 writer-verdict-260905 取代——写者最佳解释 = Rust features 双跑。
3. **§9.7 三要素补声明**：载体 = Java 存档导出 WGB2 FULL（mod cppReplace，旧 Rust dll）；覆盖面 = 4×4 chunk @ (200,200) 全阶段；可比性 = 仅与同载体 260904-03/04 口径可比，与 C++ block_probe 载体不可比。
