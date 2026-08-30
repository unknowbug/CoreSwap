# 草稿：docs/09 追加第八节（subagent 产出，主会话应用）

> **应用位置**：`versions/1.20.1/docs/09-multi-dimension.md`——追加到「七、Rust 世界参数化（2026-08-29，对齐 C++ wg_create 多世界方向）」之后（文件末尾）。追加不覆盖。

---

## 八、Rust 多世界落地（2026-08-30 Phase A/B/C，commit 1102f58 + 9a3f7fa）

> status: **candidate**。Phase A = Rust 引擎 nether 块级验证 + 两修复；Phase B/C = MOD 游戏内接线。
> 错误台账：`.investigations/multiworld-port/multiworld-errors.md`（M1-M5，含速查表）。

### Phase A：nether 块级验证 + 双高度/确定性两修复

**探针**：`WorldgenRust/src/bin/multiworld_nether_blocks.rs`（fill_chunk_blocks vs vanilla nether 参照 WGB2 4×4@0,0 h256）。

**修复 1（双高度，M3）**：`worldgen_handle.rs` 存 `noise_height`（settings noise.height，nether 128）≠ world height（256）；`terrain.rs fill_chunk` 加 noise_height 参数（y≥noise_top 留 Air）；宏观采样器网格只铺噪声高度；13 个探针 bin 调用点同步补参。
- **nether match 23.77% → 73.77% → 74.04%**（y≥128 四带 0% → 100%），**超 C++ 时代 71.97%**；overworld 基线 95.40% 零回归。

**修复 2（确定性，M4）**：`biome.rs` BiomeClassifier features/carvers HashMap → BTreeMap——原 `all_features_lists()` 每进程随机迭代序 → PlacedFeatureIndexer 编号随机 → nether features 放置运行间漂移 2796 块。修后两次运行逐位一致。

**遗留差距（记录不修）**：熔岩海带 y=32..63（7.9%，流体填充缺失——C++ 时代也未解）；底部基岩错位（VerticalGradient 反锚序，C++ 有修复未移植）。证据：`.investigations/multiworld-port/cmd-output/nether_blocks_match_v{1,2_noiseheight}.txt`。

### Phase B/C：MOD 游戏内接线

- **JNI 层**：`jni_bridge.rs` initDim（5 参 wg_create 映射）；`CppWorldgen.java` initDim 声明。
- **CppBridge**：netherHandle + initNether + fillChunkNether（16×16×256 buffer）+ feedBeardifier 泛化 handle 参数 + writeChunk 泛化维度高度。
- **Mixin 按维度分派**：下界拦截分支（min_y=0/h=256 且 netherActive）+ **末地保护**（End 与下界同形状，靠 biomeSource 反射区分——@Shadow 够不到父类字段是坑，用缓存反射）+ buildSurface 从全局 cancel 收紧为**按维度**（修掉「末地表层会被误 skip」的现存隐患）。
- **构建接线（M5）**：`build.gradle processResources inputs.file dll`（根因修复：UP-TO-DATE 跳过 doFirst 同步 → resources 里旧 dll → initDim UnsatisfiedLinkError）。
- **实证**：`initNether enabled=true` + `[Mixin] populateNoise(nether) intercepted chunk(-1,-1)`（rust_nether_test4.log，摘录 `.investigations/multiworld-port/cmd-output/nether_ingame_intercept_20260830.txt`）。

### 遗留课题
- lava 流体填充（Rust 与 C++ 同未解，Phase A 遗留差距同源）；
- 底部基岩 VerticalGradient 反锚序移植；
- **末地引擎未启动**（Mixin 保护已就位，Rust/C++ 末地生成都未做）。
