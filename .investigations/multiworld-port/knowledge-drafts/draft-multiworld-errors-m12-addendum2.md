# 草稿：multiworld-errors.md 追加「M12 补遗二」+ 速查表 1 行

> 产出者：knowledge 落盘 subagent（2026-08-31）。主会话应用方式：① 下面「A 部分」整段插入
> `multiworld-errors.md` 的 M12 节（`## M12. legacy temperature 噪声种子源定案…`）之后、
> `## 附：错误 → 根因 速查表` 之前；② 「B 部分」表格行追加到速查表末尾。
> status: candidate（数据源：cmd-output/nether_density_rust_same_seed.txt、nether_confusion_v3_postfix.txt）。

---

## A 部分：追加小节正文

### M12 补遗二：同 seed 竖切定案——cell 角点全对齐，「density 残差」大部分是工具语义陷阱（2026-08-31，status: candidate）

#### 定案数据（WG_SEED=server seed 同 seed 竖切：Rust exact 纯函数采样 vs Java DensityProbe 纯函数）
- **y=16 倍数点（cell 角点 = 插值端点）残差全 0**：y=16/24/32/40/48/56/64/72/80/88/96/104/112/120 全部 diff=0（`nether_density_rust_same_seed.txt`）→ **生产插值端点完全对齐，形状主体已对齐**。
- 非 16 倍数点（y mod 16 ∈ {4,12}）：散布 ±0.01~0.08 —— 定性为**插值语义差**而非实现错：Java DensityProbe 是纯 `df.sample` 直采，Rust 竖切走生产插值路径（cell 8 点 est 插值），**两侧采样语义不同，不可直接比**。
- **y≥128 段**：Java 恒 0.291（SQUEEZE 饱和）vs Rust 0.209/0.250 交替 —— 是 Rust 竖切工具的插值伪影（生产路径 y≥128 全 air ✓，v3 后置混淆对 band y128.. 全 100% 已证，`nether_confusion_v3_postfix.txt`）。
- **blended 列对拍彻底排除**：InterpolatedNoiseSampler(LocalRandom(worldSeed)) vs Rust 同构，16 点全一致到 f32 精度 ✓。

#### 被推翻的旧结论（记录价值高，❌ 排除清单）
- ❌「density 残差随 y 增长 0.09-0.11」= **seed 错位 + 工具语义差的复合假象**（v2 竖切：Rust 参照 seed + 派生 blended，两个变量同时错；v3：Rust worldSeed blended 但 Java 参照是 server seed，仍是 seed 错位）——非实现 bug。
- ✅ blended（old_blended_noise）构造与采样**实测完全对齐**（列对拍 16 点）——M10 补遗一的 S4/S5 早期对拍结论**维持成立**。

#### 教训（⚠️ 与「探针采集核对铁律」同族的工具语义陷阱）
- **「纯函数 vs 生产路径」的对比语义陷阱**：interpolated 类节点在纯函数直采（Java DensityProbe）与生产插值路径（Rust fill）下**值不同是语义差异而非 bug**。跨工具对比必须先声明两侧采样语义（纯函数 / 插值 / 网格），语义不同层的数值残差没有判错意义。
- **cell 角点（插值端点）对齐 = 形状主体对齐的充分判据**：端点差才是实现差；中间点差优先怀疑插值/采样语义，不先动实现。
- **残差定位三层递进完整闭环**：① seed 一致（M11 铁律）→ ② 采样语义一致（本次新增层）→ ③ 逐节点公式对照。本次走到第二层即收敛——**先做 ①② 再做 ③，能省掉整个公式层排查**。

#### 当前格局与下一步（剩余块级 gap：82.51% vs Java 100%，三条线）
1. cell 内插值一致性：Java fill 插值 vs Rust fill 插值的行为核对（本次判据下唯一可能的 density 层残差）；
2. features 层：soul_sand/soul_soil 等装饰性放置（v3 混淆对 `air→soul_sand 3052` 的定性，M11 遗留 #2）；
3. bedrock roof 缺失：`netherrack→bedrock 10288@y96..`（v3 混淆对）。
- 已收敛：climate（t/h）f32 级 ✓、blended f32 级 ✓、cell 角点 density ✓。

---

## B 部分：速查表追加行（插表末）

| 同 seed 竖切 density 残差：cell 角点全 0、中间点 ±0.01~0.08、y≥128 恒值不饱和（M12 补遗二） | **工具语义陷阱**：Java DensityProbe 纯 `df.sample` 直采 vs Rust 竖切走生产插值路径——两侧采样语义不同，中间点/y≥128 的「残差」是语义差非实现错；旧「残差随 y 增长」结论 = seed 错位（M11）+ 语义差复合假象，已推翻 | **跨工具对比先声明两侧采样语义（纯函数/插值/网格）**——「⚠️ 探针采集核对铁律」的采样语义扩展：seed 一致 → 采样语义一致 → 逐节点公式，三层递进，前两层收敛就不进第三层；**cell 角点（插值端点）对齐 = 形状主体对齐的充分判据**，中间点差优先怀疑语义 |
