# 草稿：multiworld-errors.md 追加 M10 条目（应用位置：M9 之后、「附：错误 → 根因 速查表」之前；速查表行插表末）

> status: candidate（有 S1-S4 四层对拍实证，Octave createLegacy 语义静态核对未完成，待下轮闭环 + judge）
> 证据文件：`.investigations/multiworld-port/cmd-output/legacy_calibrate_rust_v2.txt`（Rust 侧 S1-S5）、`legacy_calibrate_java_oct.log`（Java 参照探针侧，octave origin + blended 逐点）、`nether_biome_dump_summary.md`（BIOME6 6 维直采 + soul_sand 残差诊断）
> 应用方式：主会话将下方 M10 段追加进 `.investigations/multiworld-port/multiworld-errors.md`（M9 之后、速查表之前），速查表行插表末。

---

## M10. 三层对拍校准：LCG/blended 全对齐，缺口隔离到 OctavePerlin createLegacy（humidity≈0 vs Java -0.16）

### 现象
- M9 门控回退后（默认 82.72%），soul_sand 残差（biome 判定 nether_wastes vs vanilla soul_sand_valley）根因未闭环。
- BIOME6（Java router 直采，yarn NoiseRouter）@ mismatch 坐标 y=1：**t=+0.077~+0.119（正）、h=-0.149~-0.175、c/e/d/w=0**。
- Rust 同坐标（无特例）：**t=-0.115（负）、h=-0.092**——temperature 符号相反 + humidity 幅度差 → biome 判定错（Rust h=-0.092 落 nether_wastes 盒，Java h≈-0.16 落 soul_sand_valley 盒）。

### 根因（三层对拍定位，各层独立验证）
- **S1 层（LCG 裸输出）**：LegacyRandom(0) 的 next(32)×8 / nextLong×4 / nextDouble×3 与 Java CheckedRandom(0) **逐位一致** → LCG 实现无错，排除。
- **S2 层（blended Octave 构造）**：new_legacy(-15,[1×16])×2 + (-7,[1×8]) 的 16+2 个 Octave origin 与 Java createLegacy **一致**（Java 打印序为反转方向，数值按随机消耗序对齐；尾数 ~4e-6 在 f32 打印噪声级）→ Octave legacy 构造的随机消耗序无错，排除。
- **S3 层（blended 采样）**：DoublePerlin legacy @ y=1/y=52 六个 mismatch 列与 Java **一致到 ~6e-6**（f32 噪声级）→ blended（old_blended_noise）不是 nether 密度形状差的来源。⚠️ 注意口径：S3 对拍的是 climate DoublePerlin(-7,[1,1])，blended Octave(-15) 的**采样**未单独对拍（见遗留）。
- **S4 层（router 组装/消融）**：WG_LEGACY_CLIMATE=1（特例启用）时 Rust t=+0.127~+0.150（**符号已对**——确认 legacy climate visitor 就是 biome 输入的真实机制）但 **humidity ≈-0.01 vs Java -0.16**；同时总分 82.72→77.01（y32..63 暴跌 65.8→22.4）。
- **缺口定位**：humidity ≈0 vs -0.16 → `DoublePerlinNoiseSampler::new_legacy`/`OctavePerlinNoiseSampler::new_legacy`(-7,[1,1]) 的**构造/采样语义**与 Java createLegacy（yarn `OctavePerlinNoiseSampler.java`，已入档 `.investigations/multiworld-port/OctavePerlinNoiseSampler.java`）有未对齐细节；temperature 同源 +0.05 偏差同因。四层排除后唯一不一致层即缺口所在——**Octave createLegacy 的采样语义**（非构造、非 LCG）。

### 定位（诊断方法——分段对拍设计，可复用）
1. **分段设计**：裸 LCG 输出（S1）→ 单 Octave 构造产物（S2）→ 复合采样（S3）→ router 集成消融（S4），逐层排除——每层一致即排除一层，最后不一致层就是缺口。本例三层全对齐后锁定 Octave createLegacy 采样语义。
2. **「一致性判据」定义精度口径**：~6e-6 的 f32 噪声级一致算「对齐」（两侧都有 f32 乘法路径，打印尾数必然抖动）；超出该量级即真差异。没有口径定义，「0.128817 vs 0.1288179」会被误判为不一致或漏判真差异。
3. **消融开关（WG_LEGACY_CLIMATE）+ 分带混淆对 + 6 维直采（BIOME6）三件套组合**：单轮即把「总分下降」拆到「哪一维（humidity）、哪一段（Octave 采样）、偏差多少（≈0 vs -0.16）」。S4 符号翻转（负→正）还顺带**确认了机制归属**——legacy climate visitor 确实是 biome 判定输入的真实路径。
4. Java 参照探针（BIOME6，yarn NoiseRouter 直采 @ 相同坐标）是校准的权威侧——没有它，每一步都是盲调。

### 修复
- **未修**（Octave createLegacy 语义静态核对为下轮开工点；yarn 权威源码已入档 `.investigations/multiworld-port/OctavePerlinNoiseSampler.java`）。
- 特例保持 **WG_LEGACY_CLIMATE 门控（默认关）**，82.72% 最佳默认态不受影响（M9 处置延续）。

### 教训（可复用判错经验）
- **对拍校准的分段设计**：裸随机源 → 单 Octave 构造产物 → 复合采样 → router 集成，逐层排除——每层一致即排除一层，最后不一致层就是缺口。逐层排除把「一个大差异」拆成「唯一缺口层」，比在整链上盲调快一个量级。
- **「一致性判据」要先定义精度口径**：f32 打印路径两侧都有乘法噪声，~6e-6 级一致算对齐；口径不定义，对拍结论本身不可靠。
- **消融开关 + 分带混淆对 + 6 维直采三件套**，单轮即可把总分下降拆到「哪一维、哪一段、偏差多少」；符号翻转本身就是机制归属的证据（符号都随开关翻转 → 该开关就是该输出的机制）。
- **Java 参照探针是校准权威侧**——没有 Java 侧同坐标直采，Rust 侧一切采样值都没有「对错」参照。

### 遗留（下轮开工点）
1. **OctavePerlin createLegacy 构造/采样逐行对照**（yarn `OctavePerlinNoiseSampler.java` 已入档 vs Rust `new_legacy`/Octave `sample`）——重点：legacy Octave 的 amplitudes 展开、permutation 消耗、sample 的 y smear/lacunarity 语义。
2. 修好后 **WG_LEGACY_CLIMATE 默认开启**，预期 humidity 对齐 → soul_sand 残差解决 → nether 冲 90%+。
3. bedrock roof 缺失（混淆对 `netherrack→bedrock 12195`@y96..）单独排查。
4. S3 层 blended Octave(-15) 的**采样**（非构造）未单独对拍——S3 只证明了 climate DoublePerlin(-7,[1,1]) 采样对齐，blended 密度形状差的排除不完整。

---

## 速查表追加 1 行（插表末）

| 现象/信号 | 根因 | 判错要点 |
|---|---|---|
| legacy climate 启用后 t 符号对但 humidity ≈0 vs Java -0.16，总分 82.72→77.01（M10） | 四层对拍（S1 LCG 逐位一致 / S2 Octave 构造一致 / S3 DoublePerlin 采样 ~6e-6 一致 / S4 router 消融 humidity 缺口）隔离出缺口 = `OctavePerlinNoiseSampler::new_legacy`/`DoublePerlin::new_legacy`(-7,[1,1]) 的**采样语义**与 Java createLegacy 未对齐（构造与 LCG 均无错）；未修，yarn 源码已入档待下轮 | **对拍校准分段设计**：裸随机源→单 Octave 构造→复合采样→router 集成，逐层排除，最后不一致层即缺口；**一致性判据先定义精度口径**（f32 路径 ~6e-6 算对齐）；消融开关+分带混淆对+6 维直采三件套单轮拆解「哪一维/哪一段/偏差多少」；符号随开关翻转 = 机制归属证据；Java 参照探针（BIOME6）是校准权威侧 |
