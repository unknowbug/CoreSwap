# aquifer 输入面代码级分解 + biome 补测（260908-02，主会话收敛记录，draft）

> 对应 todo 2/4（GPU 重勘探 260908-02）。判定材料：代码直读 + 既有探针复跑 + 新微测。

## A. aquifer 输入面（J1/J5 判据，代码级事实）

- `Aquifer` 7 个输入全是 `Arc<DensityFunction>` 坐标参数化共享实例（aquifer.rs:291-297）：barrier / fluid_floodedness / fluid_spread / fluid_type / erosion / depth / initial_density——**无 split 数组、无角点分组，D25 结构障碍不适用（J1/J5 结构域通过）**。
- **est（estimate_surface_height，aquifer.rs:531）**：输入仅 `(bx,bz)` quart 列坐标；自顶向下 step 8 扫 `initial_density.sample() > 0.390625`；结果可记忆化（per-chunk surface_cache + 跨 chunk EstL2 已在）→ 天然网格批量形态（J4 通过）。
  - est kernel 需要评估的树 = **`initial_density_without_jaggedness`**（比 final_density 便宜，无 jagged）。
- **apply（apply_inner，aquifer.rs:463）**：每块 3×3×3 邻域 27 次 get_block_pos（纯 RNG，廉）；r/s/t 三次 get_water_level_at → get_fluid_level（13 offset 的 fluid 噪声采样）+ barrier 采样（每点 1 次，MutableDouble 缓存）——全部坐标参数化（J1 通过）。
- **关键交叉**：`initial_density_without_jaggedness` 同时被 **surface rules** 消费（worldgen_handle.rs:756/848，apply_material_rule 的 initial_density_at）→ 专用 GPU kernel 一份可同时服务 aquifer est + surface 两阶段（占比 35-37 + 5.5-6.7 ≈ 42ms/chunk 的潜在服务面）。

## B. 单价数据（口径随行）

- **initial_density 热路径单价**（qaq1_initdensity_cost 复跑，260908-02，seed 8576294172403134396）：**0.0850µs/sample**（73420 样本 6.24ms；探针采样坐标热缓存）。
- ⚠️ **矛盾待 Phase 0 分解**：热价 85ns × est 合理采样数推不出 est ~15.4ms/chunk——est 成本主要在**冷路径采样价**（qaq1_b1_coldpath_probe 归因：old_blended_noise 无缓存为主）。含义：GPU 无宿主缓存可吃，**GPU 可行性对比应以冷价而非热价为分母**（对 GPU 有利方向，但需冷/热分解实测钉死）。
- **biome 全量 quart 图单价**（biome_cost_bench 新微测，260908-02，同 seed/region，串行单线程，含 String 分配，直采 1536 quart/chunk × 64 chunks）：**8.383ms/chunk（5.458µs/采样）** ≈ FULL 62ms 的 13.5%。生产 fill 若有列缓存/复用会低于此（覆盖面声明：这是「全量重算」上界口径）。biome 侧 J8 有数据了。

## C. 对 Phase 0 的含义（供架构文档，非结论）

1. aquifer 的 est + surface 共享 initial_density kernel——B2 的 kernel 投入与 B1（density 专用 kernel）同源，**可做「initial_density 专用 kernel」最小切片先行**，服务面 42ms/chunk。
2. 冷/热价分解是 J6 核算前置（todo 3 多线程分母 + 冷价探针）。
3. biome 8.4ms/chunk 使 B3（biome multinoise kernel，C2ME 有 BIOME_MULTINOISE 先例）从「无数据」升为「占比 ~13.5%、值得纳入 Phase 0 对照」。
