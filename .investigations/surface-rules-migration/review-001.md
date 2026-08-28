# 审查意见 review-001 — ② Beardifier + ③ 块管线阶段 A + ④ 交叉验证

> 审查角色：core.judge（subagent 隔离）
> 审查对象：本 session 交付（ad887e6 / 9d6960c / 7a39c10 / 7891bf4 / e11a21d / 3da23b7 / c9f417c / 76935e9）
> 三源核对：① git 提交 ② 代码文件 ③ 验证记录 + 错误台账
> 日期：2026-08-28
> 结论：**只出意见，不改 status、不改代码**

---

## 一、逐项审查结论

### ② Beardifier（ad887e6 / 9d6960c / 7a39c10）

| 项 | 结论 | 理由 + 证据 |
|---|---|---|
| 结构对齐 C++ beardifier.h | **通过** | Rust `beardifier.rs` 与 C++ `beardifier.h` 逐行一致：TerrainAdaptation 枚举序数、BeardPiece/BeardJunction 字段、24^3 权重表索引（`arr[i*576+j*24+k]` / sample `table[k*576+i*24+j]`）、fast_inv_sqrt 位操作（`6910469410427058090 - (l>>1)` + Newton 一步）、clamped_map 链、sample 四分支（BURY→getMagnitudeWeight、THIN/BOX→getStructureWeight*0.8、junction→*0.4）、empty()。逐行对拍一致。 |
| powf 逐位 | **存疑（未验证，Partial）** | Rust `(2.718281828459045_f64).powf(-d/16.0)` 与 C++ `std::pow` 均依赖系统 libm，Java `Math.pow` 用 fdlibm——三者最后 ULP 未必一致。`beard_probe` 只做手算尺度自检（0.1667 / -0.1117），**未做真实 Java dump 逐位对拍**。`.artifacts/beardifier-port/index.yaml` 已诚实标注「Partial / powf 逐位未验证」。**Rust beardifier.rs 无任何 @anchor.test 注解**（C++ 参考有 `BEARD244#005` 锚点），锚点缺失。 |
| BEARD_THIN/BEARD_BOX 分支 | **有问题（覆盖弱）** | 7a39c10 补了 thin/box 样例，但 `beard_probe.rs` 只 `assert!(thin != 0.0)` / `assert!(bbox != 0.0)`——**只验证「分支执行且非零」，未断言具体值**。commit message 声称「-0.000575 / -0.556886 符合手算尺度」，但代码未 assert 这些值（仅打印）。比 commit message 暗示的验证弱。 |
| 集成 fill_chunk | **通过** | `terrain.rs` L71 `if let Some(b) = beard { d += b.sample(x,y,z) }`，对齐 worldgen_api.cpp L912→915→916（add(finalDensity, Beardifier) CellCache 语义）。3 调用点均传 None。 |

### ③ surface_rules（7891bf4 / e11a21d / 3da23b7）

| 项 | 结论 | 理由 + 证据 |
|---|---|---|
| 4 个 C++ bug 修正对照 Java | **通过（全部 4 条验证正确）** | ① HoleCond：Java `NegativeRunDepthPredicate` L537 = `runDepth<=0`，Java runDepth = sampleRunDepth（L459）→ Rust `surface_depth<=0` 正确；C++ L251 用 stoneDepthAbove 是 bug。② surfaceNoiseThreshold：Java VanillaSurfaceRules L391-392 = `min/8.25` → Rust 全部除 8.25 正确；C++ L541 直接用 1.75 是 bug。③ SteepCond：Java SteepSlopePredicate L548-562 读 `sampleHeightmap(i, k)`（i=x,k=z±1），heightmap 填充 `z*16+x`（worldgen_api.cpp L1045）→ Rust `hm[(z±1)*16+x]` 正确；C++ L254 读 `x*16+z` 转置是 bug。④ apply_material_rule_single：Java initHorizontalContext 设 runDepth=sampleRunDepth → Rust `surface_depth=sample_run_depth(x,z)` 正确；C++ L437-450 留 0 是 bug。 |
| build_surface 接入 fill 管线 | **通过（但 commit message 略夸大）** | `surface_probe`/`grass_probe`/`blocks_cmp` 均走 fill_chunk→BlockColumn→build_surface 端到端。但 `terrain.rs fill_chunk` **本身不调用 build_surface**（只产宏观 BlockKind），build_surface 由探针在 fill_chunk 之后单独调用。commit message「wire build_surface into fill pipeline」是探针级接入，非 fill_chunk 级。 |
| 规则树完整性 | **通过（结构级）** | mr1-mr9 + bedrock_floor + surface + deepslate 齐全，biome 温度表完整。 |

### ④ 交叉验证（c9f417c / 76935e9）

| 项 | 结论 | 理由 + 证据 |
|---|---|---|
| blocks_cmp 97.80% 合理性 | **存疑（宽松判据，未充分说明）** | 参照文件 header 已验证正确（magic=0x57474232、seed=-2032795982907864146、size=4、origin=(0,0)、minY=-64、height=384）。97.80% 相对宏观 98.06% 只降 0.26%，合理。**但 97.80% 是「全列含 air」的匹配率，被 bulk 地形（air + 深部 stone）主导，surface rules 只影响表层薄层——该数字对 surface rules 的验证强度有限**。真正验证 surface rules 的是 grass_probe（col(0,0) top=98 grass_block(8)+dirt(9)，block id 已核实）。 |
| 验证缺口 | **有问题（简化未枚举）** | blocks_cmp/grass_probe/surface_probe 三处共用 4 个简化：① `biome_temp = |_id| 0.5`（TempCond `biome_temp<0.15` 恒 false，frozen_ocean 的 ice 规则失效）② `surface_heights4` 用宏观 surface_height 4 角（Java 用 estimateSurfaceHeight 从 initial_density_without_jaggedness>0.390625 扫描，两者不同量）③ `biome_at` 用 `(x>>2)<<2` floor 对齐（Java 用实际块坐标 8 邻域选点）④ `fill_chunk(..., None)` 不含 Beardifier。commit message 只说「loose criterion」，**未枚举这 4 个具体简化**。 |

### 验证分层诚实性

| 项 | 结论 | 理由 + 证据 |
|---|---|---|
| Beardifier 分层 | **诚实（Partial）** | `.artifacts/beardifier-port/index.yaml` 明确标 candidate（结构级）+ Partial（静态结构对齐 + 自洽自检，未做逐位 Full 对拍），并列出 MUST-verify 待办。 |
| surface_rules 分层 | **有问题（未显式声明）** | 无独立 regression-record.md；`surface_rules.rs` 头部注释「未编译验证：本文件由 worker 产出」**已过期**（后续 surface_probe/grass_probe/blocks_cmp 已编译运行）。 |
| blocks_cmp 分层 | **有问题（宽松判据未量化）** | 97.80% 是宽松判据结果，但简化项未枚举（见上）。 |

### 风险

| 项 | 结论 | 理由 + 证据 |
|---|---|---|
| biome_temp=0.5 简化 | **中风险** | 使 TempCond 恒 false，frozen_ocean 的 ice 规则失效。spawn 区（chunk 0,0）为低地，frozen ocean 可能不在，影响小；但若验证扩展到寒带 biome 会显著偏差。 |
| surface_heights4 简化 | **中风险** | 用宏观 surface_height 替代 estimateSurfaceHeight，两者不同量。SurfaceCondC（above_preliminary_surface）是 surface 规则应用的门控，若高度错，grass/sand 会放错高度。grass_probe 显示部分列正确，但未系统验证。 |
| Beardifier powf 逐位未验证 | **低-中风险** | 结构对齐已通过，但 powf 最后 ULP 未对拍。经 as f32 边界条目可能差 1 ULP。对宏观地形影响小，对逐位对齐目标影响大。 |

---

## 二、整体交付确认等级建议

**建议：candidate（不升 confirmed）**

理由：
- 结构级对齐（Beardifier 结构、surface_rules 4 bug 修正、build_surface 接入）均通过，证据充分。
- 但存在 3 个阻止 confirmed 的硬缺口：
  1. **Beardifier powf 逐位未验证**（需真实 Java dump 对拍，index.yaml 已自认）。
  2. **交叉验证 4 个简化未枚举**（biome_temp=0.5 / surface_heights4 / biome_at floor / 无 Beardifier），97.80% 是宽松判据，未量化简化影响。
  3. **验证记录未落盘**：无 regression-record.md；错误台账（rust-errors.md R6/R7、surface-rules-errors.md）与 `.artifacts/beardifier-port/` **均未提交**（仅工作区）。

---

## 三、必须修复 / 需补验证 / 可接受

### 必须修复
1. **错误台账与 artifacts 未提交**：`.investigations/surface-rules-migration/`、`.investigations/rust-density-builder/rust-errors.md`（R6/R7 新增）、`.artifacts/beardifier-port/` 均未 commit。AGENTS.md 明确「错误台账是最高优先级资产」，必须提交。
2. **验证记录落盘**：为 beard_probe / grass_probe / blocks_cmp / surface_probe 补 regression-record.md（命令 + 输出摘要），满足 judge 基线三源核对。

### 需补验证
3. **Beardifier powf 逐位对拍**（MUST，index.yaml 已列）：真实含 terrain_adaptation 的 Java 探针 dump 对拍，确认 powf 最后 ULP。
4. **交叉验证简化影响量化**：至少评估 biome_temp=0.5 与 surface_heights4 简化对 97.80% 的影响（如用真实温度表 / 真实 estimateSurfaceHeight 重跑对比）。
5. **BEARD_THIN/BEARD_BOX 分支断言具体值**：当前只 assert 非零，应断言手算值（-0.000575 / -0.556886）。

### 可接受
6. **Rust beardifier.rs 无 @anchor.test**：结构对齐已通过，锚点缺失可接受（但建议补，与 C++ 参考对齐）。
7. **build_surface 探针级接入**（非 fill_chunk 级）：设计合理（宏观/具体分离），但 commit message 措辞应修正。
8. **surface_rules.rs「未编译验证」过期注释**：应更新为已编译验证。

---

## 四、审查清单核对

- 证据完整性（@anchor.test source）：Rust 侧 Beardifier/surface_rules **无 @anchor.test**（C++ 参考有）。→ 意见标注。
- 证据落盘（regression-record.md）：**缺失**。→ 意见标注「证据链不完整」。
- 三源核对：git 提交 ✓ / 代码 ✓ / 验证记录（commit message 报告数值 + 参照文件 header 已核实）✓，但无独立 run record。
- 置信度合法：`.artifacts/beardifier-port/index.yaml` 标 candidate（非 confirmed），合法。无 AI 自授 confirmed。
- 产物契约：`.artifacts/beardifier-port/index.yaml` 已落盘但**未提交**；无根 index.yaml 更新记录。
- 噪声卡历史：未发现未解决噪声卡。
- retry cap：本 session 各假设轮次未见超限（R1-R7 均有新数据层证据）。
- 模块边界：未发现跨模块 skill 正文引用。

---

*本意见为建议，非命令；confirmed 由宿主人类拍板。*
