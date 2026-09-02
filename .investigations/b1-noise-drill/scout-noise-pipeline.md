# B1 NOISE 阶段管线地图（scout 勘探产物）

- status: draft（只读勘探，不承担解读结论；机制方向供 worker 定位用）
- 课题: B1 nether NOISE 层 13 列单元格级 air 缘 ±1 微差（seed 8576294172403134396, chunk 3200..3211, basalt_deltas）
- 日期标签: 260902-08 勘探轮（实际时刻以 git/主会话为准）

## 0. 结论速览（nether NOISE 判定链，三侧对照）

nether `final_density = squeeze(mul(0.64, interpolated(blend_density(add(2.5, mul(grad_bottom, add(-2.5, add(0.9375, mul(grad_top, add(-0.9375, base_3d_noise))))))))))`，
`base_3d_noise = old_blended_noise(xz_factor 80, y_factor 60, xz_scale 0.25, y_scale 0.375, smear 8.0)`（nether.json L25-74 + base_3d_noise.json）。
air/solid 判定只有一条链：**块级 finalDensity > 0 → netherrack（NOISE 层 rust 统一 id=1）；d ≤ 0 且 y ≥ 32 → air；d ≤ 0 且 y < 32 → lava**（aquifers_enabled=false → Java `AquiferSampler.seaLevel` 路径，无噪声参与；nether.json aquifers_enabled=false / sea_level=32）。
所以 **13 列 air 缘 ±1 = 该块三线性插值后的 finalDensity 在 0 附近的符号翻转**——差异只可能来自「插值前角点值」或「插值本身」的位级差。

## 1. Rust 侧管线（生产链）

| 步骤 | 内容 | 位置 |
|---|---|---|
| 入口 | `WorldgenHandle::fill_chunk_blocks(cx,cz)` | worldgen_handle.rs L380 |
| density 源 | `DensityMacroSampler`（macrolize 后 cell-grid 采样）；WG_TRANSPILER 时 `TranspilerDensity` | worldgen_handle.rs L401-406；terrain.rs L18-103 / L289-373 |
| macrolize | `macrolize_channels(tree)`：DFS 把所有 `Interpolated` 内层抽成 channel（nether 只有 1 个：blend_density 内层），外层非线性和 squeeze/mul/blend 留在 combine 每块求值 | density.rs L690-760 |
| 角点网格 | 5×17×5（cell 4×8×4）角点采样 channel；nether 只铺 noise_height=128（gy=17, y 0..128） | terrain.rs L34-51（build_slices）；worldgen_handle.rs L160-161 双高度注释 |
| 块级插值 | 三线性，**轴序 X→Y→Z**（先 x 后 y 后 z），逐块全量重算 8 角点 | terrain.rs L53-77（sample_interp_impl；d00=lerp(fx)→lerp(fy)→lerp(fz)） |
| block 分类 | `VanillaAquifer.classify`：`d>0→Rock`；`!enabled` → `y<sea_level(32)? Lava: Air`（严格 <） | terrain.rs L214-227（含 Java seaLevel 对齐注释 L217-219） |
| NOISE 层写 | kind→id（air=0/stone=1/water/lava），rock 处 ore_vein（nether ore_veins_enabled=false，apply 恒 -1） | worldgen_handle.rs L408-430 |
| Beardifier | `self.beardifiers` map，nether 无结构输入时 None（+0） | worldgen_handle.rs L400, terrain.rs L267-269 |
| 噪声底层 | Perlin/Octave/Double：noise.rs L101-149（sample_ys + sample_section，fade y=原始 h），L248-261（Octave sample：maintain_precision(x*e)+amp*pers），L316-321（DoublePerlin：first(g)+second(g*DOMAIN_SCALE) × amp） | noise.rs |
| old_blended_noise | `InterpolatedNoiseData::sample`：interpolation 8 oct `/o` 求和 → q=(n/10+1)/2 → lower/upper 16 oct（bl2/bl3 门控）→ (l/512 + q(m/512-l/512))/128 | density.rs L177-223 |
| legacy 种子 | nether legacy_random_source=true → `RsRandom::Legacy`；old_blended 实际种子 = worldSeed（S8 定案），lower/upper new_legacy(-15,amp_l)、interp new_legacy(-7,amp_i) | density_builder.rs L368-387；legacy_random.rs |

## 2. Java 参照链（yarn 源，versions/1.20.1/data/mc_src_extract）

| 步骤 | 内容 | 位置 |
|---|---|---|
| 主循环 | `populateNoise`：x-cell 双缓冲（start/end），cell 内 y 从顶向下逐块 `interpolateY(t, s/8)`，x 逐块 `interpolateX`，z 逐块 `interpolateZ` → `sampleBlockState()` | NoiseChunkGenerator.java L359-428（runtime/1.20.1/java/ 同版） |
| blockState 判定 | `blockStateSampler = ChainedBlockSource(aquifer.apply(pos, cacheAllInCell(add(finalDensity,Beardifier))), [oreVein])`；null → default_block(netherrack) | ChunkNoiseSampler.java L176-188, L203-205 |
| 下界 aquifer | `!hasAquifers → AquiferSampler.seaLevel(fluidLevelSampler)`（d≤0 → y<32 lava : air） | ChunkNoiseSampler.java L160-161 |
| 插值器 | `DensityInterpolator`：8 角点缓存 → **增量轴序 Y→X→Z**（interpolateY→interpolateX→interpolateZ 逐块更新中间量），采样走 `this.result`；cache 路径（isSamplingForCaches）走 `MathHelper.lerp3`（另一条独立公式路径） | ChunkNoiseSampler.java L749-808（onSampledCellCorners/interpolateY/X/Z/sample） |
| 角点采样 | `sampleDensity(start/end, cellX)`：对每个 interpolator `fill(ds, interpolationEachApplier)`——z 列 × y 角点缓冲 `startDensityBuffer[z][y]` | ChunkNoiseSampler.java L254-271 |
| 缓存语义 | CellCache / Cache2D / FlatCache / CacheOnce（cacheOnceUniqueIndex 每次 fill +1） | ChunkNoiseSampler.java L557-701, L836-881 |

## 3. C++ 第三参照（versions/1.20.1/cpp/worldgen/src）

- `worldgen_api.cpp` fillOneChunk L777-840：块级直接采样 finalDensity，`InterpolatedDF` 内部 cell 网格懒建插值（density.h L480 注释「vanilla 语义：cell 角点采样+三线性」）；下界无 aquifer 分支 L784-797（hasAquifer=false 跳过 aquifer/oreVein）。
- 噪声底层 noise.h（Rust noise.rs 即从其移植，头部注释声明）。
- C++ 侧历史上 overworld density diff0 已验证；nether 判定链与 Rust 同构（也非 Java 的增量插值）。

## 4. 差异敏感点假设空间（只列候选，不裁决；按证据位置标注）

| # | 候选环节 | 机制 | 证据位置 |
|---|---|---|---|
| A1 | **插值轴序/浮点结合序**：Java Y→X→Z 增量中间量复用 vs Rust/C++ X→Y→Z 全量重算 | 数学等价但 FP 舍入不同 → d≈0 处符号翻转（±1 块 air 缘） | ChunkNoiseSampler.java L763-806 vs terrain.rs L53-77 / density.rs L311-317 |
| A2 | Java cache 路径 `MathHelper.lerp3` 与 loop 路径 `result` 双公式并存 | 若某采样走 lerp3（isSamplingForCaches，cache_all_in_cell 的 fill 时），同一块两点公式不同 | ChunkNoiseSampler.java L786-808, L810-815 |
| A3 | old_blended_noise 内部（lower/upper/interpolation octaves）| createLegacy 种子派生、octave skip(262)、maintainPrecision、`/o` 累加顺序、q 门控（bl2/bl3 严格 ≥/≤）任一 ±1ulp → 角点值差 | density.rs L177-223；density_builder.rs L368-387；noise.rs L172-201 |
| A4 | Perlin 核心（fade/grad/lerp 顺序、AVX 路径 vs 标量路径不一致）| Rust 生产用 `sample_section_avx`（target_feature=avx 时），其 8-dot 展开顺序与标量 `sample_section` 不同；角点采样若命中 AVX/标量差异即翻符号。Java 永远标量序 | noise.rs L51-149（两路径并存），L119-122 分派 |
| A5 | DoublePerlin 合成序：`(first+second)*amp`、DOMAIN_SCALE 缩放位置 | nether base_3d_noise 是 DoublePerlin? 否——old_blended 直接是三 Octave；但 nether_state_selector/patch（surface 层）是 DoublePerlin——对 NOISE 层 air 无关，仅关联材质差 | noise.rs L316-321（B1 air 口径下低相关） |
| A6 | 角点网格对齐/覆盖：gy=17（y0..128）、gx/gz=5；nether 双高度（noise 128 < world 256） | 若角点 y 集/夹持（clamp gx-2 等）与 Java 缓冲（verticalCellCount=floorDiv(128,8)=16）不一致 → 顶部 cell y120..128 插值差 | terrain.rs L31-32, L58；ChunkNoiseSampler.java L131-132 |
| A7 | squeeze/0.64mul/y_clamped_gradient 每块求值序（combine 树）| squeeze 的 clamp 与 `d/2 - d³/24` 实现序；y_clamped_gradient 在角点求值（梯形 y 线性段被插值=精确），但求值点 y 是角点(8 倍数)两侧一致 | density.rs L44（Squeeze），L523+（sample_combine）；nether.json L26-73 |
| A8 | blend_density 恒等路径 | 无邻居混合时 alpha=1；若 rust 实现为乘除 1.0 引入舍入（Java `Blender#blendDensity` alpha≥1 直通）| terrain.rs/density.rs BlendDensity（density.rs L508, L614）|
| A9 | d==0 边界严格性（`d>0` vs `d>=0`）| 两侧均为严格 >（Java aquifer computeSubstance `density>0`→stone；rust 同）；恰为 0.0 时→air，概率极低但列于空间 | terrain.rs L216；docs/09 L156 |
| A10 | Beardifier 输入污染 | nether 该区无结构则 None/0；若 map 意外有 entry（fortress beard 泄漏到邻 chunk）会改 d | worldgen_handle.rs L400；beardifier.rs L133 |

排除性提示（供 worker）：B1 已证 vanilla 侧 PRE dump（NOISE 完成态）该列无 air → vanilla d>0；rust-only air → rust d≤0。**首选量化手段 = 两侧对同一差异块 dump finalDensity 数值（Java DensityProbe / Rust sample_density_exact worldgen_handle.rs L347），看符号与 |d| 量级**——若 |d|~1e-9 级 → A1/A4 舍入类；若 |d|~0.1 级 → A3 角点值本身错。

## 5. 13 差异列坐标特征观察（仅 5 样例，人肉推导）

已知样例：(51231,51329), (51222,51365), (51213,51381), (51240,51348), (51204,51361)。区域 x∈[51200,51391), z∈[51328,51391)。

- **x 构成公差 9 的等差序列**：51204, 51213, 51222, 51231, 51240（五点全中，跨 37 块窗口）。9 ≡ 1 (mod 4)，故 x mod 4 覆盖 0,1,2,3 各值——**不是 4 格 cell 对齐模式**；等差 9 在噪声地形中自然出现的概率低，更像同一洞缘/坑沿沿 x 方向扫过的横列（ pocket 边缘一排点），或某种周期 9 的采样错位伪影。
- z 值（51329, 51365, 51381, 51348, 51361）离散分散，z−51328 ∈ {1,37,53,20,33}，无等差；z mod 4 = {1,1,1,0,1}（4/5 为 1，样本太少不下结论）。
- 样例 y=48（rust-only air）为 8 的倍数（cell-Y 边界值），且远高于 lava 线 32 → 是纯密度符号翻转，非流体语义。
- **判读（弱证据，draft）**：差异呈「局部洞缘簇」而非全网格系统偏移——支持「d 在 0 附近窄带内翻转」类候选（A1/A4 舍入、或 A3 局部角点差），不支持坐标/网格级系统性错位（A6）。**待办**：取全部 13 列坐标验证等差-9 是否贯穿（当前仅 5 样例），并对每列 dump rust/vanilla 交界的 d 值量级（§4 末量化手段）。

## 6. 对拍口径引用

- vanilla 参照: `.tmp/surfacedump/vanilla-pre-*.csv`（buildSurface HEAD 前 = NOISE 完成态）
- rust dump: `.tmp/noiseonly-rust-c3200-3211.csv`（SKIP_SURFACE/CARVER/FEATURES）
- 脚本: `.tmp/compare_noiseonly.py`（air 签名 = id==0 的 y 集合，材质不可比——rust NOISE 统一 id=1）
- v0.20 §9.7 声明：载体 = vanilla-PRE vs rust steps1-2；覆盖面 = 4096 列 × y0..127；与 260902-08 被污染的 13.70% 存档口径不可比。
