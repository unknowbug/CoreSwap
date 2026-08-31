# 草稿（knowledge subagent 产出，主会话应用）：multiworld-errors.md 追加 M12

> 应用位置：`multiworld-errors.md` 的 M11 小节之后、「附：错误 → 根因 速查表」之前；速查表行插表末。
> status：**candidate**（有 S8 try-seed sweep 逐位一致证据 + 修复后对拍残差 f32 精度级证据，未过 judge/confirmed）。
> 证据文件：`.investigations/multiworld-port/cmd-output/build_s8.txt`（legacy_calibrate sweep 编译记录，含 `LegacyRandom::new(ws7)` 种子参数化 bin）。

---

## M12. legacy temperature 噪声种子源定案（try-seed sweep）：**worldSeed** 而非 sources 字面 0L；t 残差 0.094 → 0.0005

### 现象
- M11 seed 修正后同 seed（-2032795982907864146）对比：
  - t(12,1,0)：Java **+0.0775** vs Rust **+0.171**（残差 0.094）；
  - h：Java **-0.1533** vs Rust **-0.187**（残差 0.034）。
- 此时 shift 两边已实测恒 0（OFFSET 特例生效）、派生链 split 已对拍逐位一致、参数表一致——**残差来源待定**（M11 遗留第 4 条：noise_params 表 vs 注册表 / shift_a 细节，均被后续排除）。

### 根因（机制）
- yarn sources 的 `NoiseConfig` 源码字面写 `createLegacy(this.createRandom(0L), ...)`——但**运行时行为实测种子源 = worldSeed**。即：**sources 字面与运行行为不符**（真实装配链的种子传递路径与 sources 所示不同——具体传递机制待查，见遗留第 3 条）。
- Rust 之前**按 sources 字面 0L 实现** legacy temperature 特例 → 种子源错 → 采样值错 → t 残差 0.094。

### 定位（三层递进，每层有数据）
1. **ShiftedNoise 递归树 dump**（Biome6Probe 反射，参数类型 **NoisePos**——yarn 1.20.1 接口是 NoisePos 非 FunctionContext，反射时别按旧名找）：shiftX/Y/Z 全恒 0（OFFSET 特例生效 ✓）；temperature noise 是 ShiftedNoise(RegistryEntryHolder 包装) 内的 `DensityFunction.Noise` record。
2. **Noise record 反射（关键判读点）**：`noiseData` 字段显示注册表参数 **(-10, [1.5,0,1,0,0,0])**，**但实际 firstSampler 只有 2 个非空 octave = (-7, [1,1]) 特例形状** → record 的 noiseData 保留原引用、sampler 已被 visitor 替换为特例——「**params 字段 ≠ sampler 实际来源**」。
3. **S8 try-seed sweep（定案手段）**：Rust 构造 `createLegacy(LegacyRandom(seed), -7, [1,1])`，seed ∈ {0, worldSeed, worldSeed×2} 三候选对照——**seed=worldSeed 的 origins [21.877382, 5.365246, 138.552887] / [47.402641, 28.663731, 67.151535] 与 Java router 实测逐位一致** → 种子源 = worldSeed，定案。

### 修复
- `density_builder.rs get_noise_sampler`：legacy 下 `minecraft:temperature` → `DoublePerlinNoiseSampler::new_legacy(RsRandom::Legacy(LegacyRandom::new(world_seed)), -7, &[1.0, 1.0])`（种子源改 worldSeed）。
- 验证：t(12,1,0) Rust **+0.078** vs Java **+0.0775**（残差 **0.0005**，f32 精度级）；nether **82.72%** 中性；overworld **95.40%** 零回归。

### 教训（⚠️ 重点）
- **「同模式类推」翻车**：vegetation 套用 worldSeed 同款 → h=+0.078 与 t 完全相同（**同种子同参数输出必然相同**），而 Java t≠h → 立即暴露 vegetation 种子源不同 → **已回滚**（vegetation 保持派生，残差 0.034 更小）。**种子源必须逐维 try-seed 实测定案，不能类推。**
- **record 字段与实际来源可以不一致**：Noise record 的 noiseData（注册表参数）≠ sampler 实际构造来源（visitor 特例）——「打印字段值」可能误导，**「字段形状 + try-seed 行为对照」才是定案手段**。
- **sources 字面 ≠ 运行时行为**：yarn sources 与真实装配链存在版本/路径差——**实测行为优先级高于源码字面**（与「禁止直接信任 javap」同族的源码层告诫，多一条对照手段：try-seed）。

### 遗留（下轮开工点）
1. **vegetation 种子源 sweep**：Java 打印 vegetation firstSampler origins（同 ABL 模式换 vegetation）→ Rust 候选种子对照（worldSeed/2/1L*2/...）→ h 收敛。
2. h 收敛后 **soul_sand 残差重估**（biome 判定边界变化）。
3. 「sources 0L vs 运行时 worldSeed」**机制待查**（NoiseConfig 真实运行版本 vs yarn sources build.10 v2 的差异点）。

---

## 速查表追加 1 行（插表末）

| 现象/信号 | 根因 | 判错要点 |
|---|---|---|
| M11 修正后 t 残差 0.094（shift/派生/参数表全对齐仍不收敛）（M12） | legacy temperature 特例种子源按 yarn sources 字面 0L 实现，而运行时实测为 **worldSeed**（S8 try-seed sweep：seed=worldSeed origins 与 Java router 逐位一致；seed=0/×2 不符）；修后 t 残差 0.094→0.0005（f32 级），nether 82.72% 中性、overworld 零回归 | **种子源必须逐维 try-seed 实测定案，不能同模式类推**（vegetation 套用 worldSeed 立即翻车：h=+0.078 与 t 全同而 Java t≠h，已回滚）；**record 字段 ≠ sampler 实际来源**（noiseData 显示注册表参数、firstSampler 已是 visitor 特例——字段形状+行为对照才是定案手段）；**sources 字面 ≠ 运行时行为**（实测优先于源码字面） |
