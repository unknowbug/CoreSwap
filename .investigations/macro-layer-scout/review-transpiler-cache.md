# judge 审查意见：transpiler 补缓存 + 探针污染修复（收尾三源核对）

> 审查角色：core.judge（subagent，隔离执行）。
> 审查对象：2026-08-30 session 对 build-time transpiler 做两件事——① 补缓存（对齐 Java/SteelMC ColumnCache，M11）；② 修复性能探针污染（M10）。
> 审查标准：core-judge 清单（证据完整性/落盘/三源核对/置信度合法/产物契约/噪声卡/retry cap/模块边界）。
> 审查基线：① `.investigations/macro-layer-scout/` 记录（transpiler-errors.md M10/M11 + cmd-output）② git HEAD + 工作区 diff ③ 原始代码（build/density.rs + src/density.rs + 探针）+ 生成代码（vanilla_density_functions.rs）。
> 结论性质：**只出审查意见，不改任何 status。confirmed 由人类授予。**

---

## 一、逐环核对结论

### 环 1 — 缓存语义是否正确（⚠️ 部分通过，发现 2 处正确性隐患）

**三源核对（transpiler_cache_2d vs 运行时 Cache2DData）：**

| 项 | transpiler `transpiler_cache_2d`（src/density.rs L335-362） | 运行时 `Cache2DData::sample`（src/density.rs L380-393） | 一致 |
|---|---|---|---|
| key 语义 | `((x as i64) << 32) ^ (z as i64)`（f64→i64） | `((pos.x as u32) as u64) << 32 ^ (pos.z as u32 as u64)`（i32→u32） | ✅ 整数坐标下 key 位一致 |
| y 无关 | 忽略 y（仅 id,x,z） | 忽略 y | ✅ |
| 缓存容量 | 256 槽 LRU（CACHE2D_CAP） | 256 槽 LRU | ✅ |
| 嵌套借用 | 未命中 drop 借用后重算（L350） | — | ✅ 避免 RefCell panic |

- **key 语义对齐**：transpiler 用 `(x as i64) << 32 ^ (z as i64)`，运行时用 `(pos.x as u32) as u64 << 32 ^ (pos.z as u32 as u64)`。对整数坐标（corner 坐标均为整数），两者产生相同 key 位。**y 无关语义一致**（transpiler 忽略 y，运行时也忽略 y）。✅
- **嵌套借用处理正确**：`transpiler_cache_2d` 先 `C2D_CACHE.with(|m| m.borrow())` 查缓存（借用释放），未命中后 `compute()`（闭包可能嵌套调用同一 RefCell），再 `borrow_mut()` 写缓存。**drop 借用后重算，避免嵌套借用 RefCell panic**——与 M11 描述一致。✅

**⚠️ 隐患 1 — 缓存 id 跨生成函数碰撞（正确性隐患）**：
- `build_final_density`（build/density.rs L20/L22）调用 `build_compute("final_density", ...)` 和 `build_compute("continents", ...)` 两次，每次 `build_compute` 创建 `GenCtx { cache_id: 0 }`（L29）——**cache_id 计数器每次 build_compute 重置为 0**。
- 生成代码实证：`compute_continents` 用 `transpiler_cache_2d(0, x, z, || continentalness)`，`fill_cell_corner_densities_final_density` 用 `transpiler_cache_2d(0, x, z, || 1.0)`——**两个不同缓存节点共用 id 0**，写入同一 `C2D_CACHE[0]` 槽。
- 若两函数在同一线程、同一 (x,z) 被调用，第二个会命中第一个的缓存值 → **错误结果**。当前 `continents_alignment.rs` 只调 `compute_continents`（不调 final_density），故未 manifest；但这是**潜在正确性 bug**（跨生成函数 cache id 不唯一）。
- 生成代码统计：100 处 `transpiler_cache_2d(` 调用，但仅 **99 个唯一 id**（0-98），id 0 重复——正是此碰撞。

**⚠️ 隐患 2 — y 相关缓存节点用 (x,z) key（正确性 bug）**：
- `entrances.json` 的 `cache_once` 节点（L37/L51）包装 `spaghetti_3d_rarity` noise（`y_scale: 1.0`，**y 相关**）。transpiler 生成 `transpiler_cache_2d(id, x, z, || sample_noise("spaghetti_3d_rarity", x*2, y*1, z*2))`——**用 (x,z) key 缓存 y 相关 inner**。
- 生成代码实证：ids 76/77/91/92 均包装 `spaghetti_3d_rarity`（`y * 1f64`），在 `fill_cell_corner_densities_final_density` 内（channel inner，cell corners 采样）。
- **Java/SteelMC 语义**：`cache_once` 按精确 (x,y,z) 缓存（`lastPos.equals(pos)`），非 (x,z)。transpiler 用 (x,z) key 对 y 相关 inner **返回错误值**——同一 (x,z) 不同 y 时，后续 y 拿到首个 y 的缓存值。
- **cell grid 采样必然触发**：cell 是 4x8x4（CELL_Y=8），同一 (x,z) 有 8 个 y 层。`fill_cell_corner_densities` 对同一 (x,z) 采样 8 个 y → y 相关 cache 节点返回错误值。
- build/density.rs L230 注释承认「cache_once/cache_all_in_cell 用 (x,z) key 近似（对齐 cache_2d）」——但**对 y 相关 inner 这是正确性 bug，不是性能近似**。`flat_cache`/`cache_2d` 的 (x,z) key 正确（xz-only）；`cache_once` 包装 y 相关 inner 时 (x,z) key **错误**。

**结论**：`flat_cache`/`cache_2d` 的 (x,z) key 语义正确；但 `cache_once` 包装 y 相关 inner（spaghetti_3d_rarity）时 (x,z) key 是**正确性 bug**，且 cache id 跨生成函数碰撞是**潜在正确性 bug**。**环 1 部分通过。**

### 环 2 — 缓存正确性（对齐值不变是缓存正确的证据？⚠️ 证据不足）

- **continents 0.000000**：`transpiler_continents_after_shiftfix.txt`（max_diff=0.000000, n=54）是 **after_shiftfix（补缓存前）** 的测量。**无 post-cache continents 复测记录**（cmd-output 无 `transpiler_continents_after_cache.txt`）。
- **final_density 0.43**：`transpiler_finaldensity_after_shiftfix.txt`（max_diff=0.432843, n=54）也是 **after_shiftfix（补缓存前）** 的测量。**无 post-cache final_density 复测记录**。
- **「不变」是推断，非复测**：M11 声称「continents 0.000000（不变，缓存语义正确）、final_density 0.43（基本不变）」——但 cmd-output 只有补缓存前的对齐值，**没有补缓存后的对齐复测**。任务描述「对齐验证：continents 0.000000（不变）、final_density 0.43（基本不变）」**未被 post-cache 测量支撑**。
- **关键**：环 1 隐患 2 的 y 相关 cache bug 在 `transpiler_alignment.rs` 测试中**必然 manifest**（该测试对每个 (x,z) 采样 6 个 y，y 相关 cache 节点返回错误值）。补缓存后 final_density 对齐**应比 0.43 更差**（0.43 + cache bug 误差）。「基本不变」与 y 相关 cache bug 矛盾——**要么补缓存后未复测（证据缺失），要么复测了但结果被误读**。
- **continents 0.000000 不能证明缓存语义正确**：continents 是纯 xz（无 y 相关 cache 节点），0.000000 只证明 `flat_cache`/`cache_2d` 的 (x,z) key 对 xz-only inner 正确，**未覆盖 y 相关 cache 路径**。

**结论**：缓存正确性证据**不完整**——缺 post-cache 对齐复测，且 y 相关 cache bug 未被测试覆盖。**环 2 证据不足。**

### 环 3 — 探针污染修复是否正确（✅ 通过）

- **污染确认**：`transpiler_fill_noise_share.rs` 原坐标 `px = -288*16 + (i%16)*4`、`pz = -256*16 + (i%16)*4` 用同一 `(i%16)` → (px,pz) 仅 16 种组合 → 缓存命中（缓存热）。git diff 确认原代码如此。✅
- **修复正确**：工作区 `transpiler_fill_noise_share.rs` L35-37 改为 `px = -288*16 + (i%1000)*4`、`pz = -256*16 + (i/1000%1000)*4`——每 corner 不同 (x,z)。`transpiler_fill_cold.rs` 同坐标。✅
- **测量可信**：`transpiler_fill_cold_after_cache.txt` = 260785.7ns（260μs），M10 称 263μs（接近）。`transpiler_fill_after_cache.txt` = 13043.9ns（13μs，污染探针=缓存热）。260/13 ≈ 20 倍低估，与 M10 一致。✅
- **注意**：`transpiler_fill_after_cache.txt` 标签「缓存冷, 不同 corner」是**污染探针的假标签**（实际缓存热）——作为 cmd-output 记录保留可接受，但 M11「fill 单次（缓存热）438μs → 13μs」用此污染数字，把「无缓存（438μs）」与「缓存热（13μs）」混比。M10 已澄清，非阻塞。

**结论**：探针污染修复正确，修后测出真缓存冷 260μs 可信。**通过。**

### 环 4 — 产物契约是否满足（✅ 通过）

- **`.artifacts/index.yaml`**：新增 5 条 transpiler 补缓存条目（cache-fix / cache-runtime / cache-alignment / probe-pollution / mvp-simplification），全部 `status: candidate`。✅
- **错误台账 transpiler-errors.md**：M10/M11 五段式完整记录（现象/根因/定位/修复/教训）+ 速查表。✅
- **cmd-output**：`transpiler_grid_after_cache.txt`（16.91ms）/ `transpiler_fill_after_cache.txt`（13μs）/ `transpiler_fill_cold_after_cache.txt`（260μs）落盘。✅
- **docs/07-block-pipeline.md**：末尾追加「2026-08-30 transpiler 补缓存 + 探针污染修复」小节，标注 candidate + DRAFT。✅
- **结论**：产物契约完整满足。

### 环 5 — 置信度标注是否合法（✅ 通过）

- 所有 `.artifacts/index.yaml` 条目、docs/07 小节、错误台账均标 **candidate**（非 confirmed）。✅
- 无任何 AI 自标 confirmed 的违规。✅
- **confirmed 留给人类**：docs/07 明确「confirmed 由人类授予」。✅

### 环 6 — 是否有遗漏（⚠️ 发现 2 处正确性隐患 + 1 处证据缺口）

| 审查点 | 结论 |
|---|---|
| 缓存覆盖所有缓存节点 | ✅ 100 处 `transpiler_cache_2d(` 调用，`grep "unhandled type"` = 0，`grep "unresolved ref"` = 0 |
| cache id 唯一性 | ⚠️ **99 个唯一 id（0-98），id 0 重复**——`compute_continents` 与 `fill_cell_corner_densities_final_density` 共用 id 0（cache_id 每 build_compute 重置） |
| `cache_once`/`cache_all_in_cell` cell 级语义 | ⚠️ **y 相关 cache 节点（ids 76/77/91/92，spaghetti_3d_rarity y_scale=1.0）用 (x,z) key 缓存 = 正确性 bug**；`cache_all_in_cell` 的 cell 级语义未对齐（用 (x,z) 点级近似） |
| post-cache 对齐复测 | ⚠️ **缺**：continents/final_density 对齐值均为补缓存前（after_shiftfix）测量，无补缓存后复测 |

**遗漏结论**：缓存覆盖完整（100 处、无 unhandled），但存在 2 处正确性隐患（cache id 碰撞 + y 相关 cache bug）和 1 处证据缺口（post-cache 对齐复测缺失）。

---

## 二、三源核对表

| 核对项 | ① 记录（.investigations/） | ② git HEAD + 工作区 diff | ③ 原始代码/生成代码/运行时 | 一致 |
|---|---|---|---|---|
| 补缓存代码（build/density.rs） | transpiler-errors.md M11「修复」 | build/density.rs +10 行（L226-234） | build/density.rs L226-234 生成 `transpiler_cache_2d(id,x,z,||inner)` | ✅ |
| 运行时 transpiler_cache_2d | transpiler-errors.md M11「运行时加函数」 | src/density.rs +32 行（L335-362） | src/density.rs L335-362 先查缓存、drop 借用后重算 | ✅ |
| 探针污染修复 | transpiler-errors.md M10「修复」 | transpiler_fill_noise_share.rs 坐标改 (i%1000)/(i/1000%1000) | 工作区 L35-37 每 corner 不同 (x,z) | ✅ |
| 生成代码 100 处调用 | M11「100 处」 | 生成代码 diff | `transpiler_cache_2d(` 计数 = 100；unhandled=0；unresolved=0 | ✅ |
| cache id 唯一性 | —（未记录） | 生成代码 | **99 唯一 id，id 0 重复（compute_continents vs fill_final_density）** | ⚠️ 差异源 |
| y 相关 cache 节点 | —（未记录） | 生成代码 | **ids 76/77/91/92 包装 spaghetti_3d_rarity（y*1f64）用 (x,z) key** | ⚠️ 差异源 |
| continents 0.000000 | M11「不变」 | 无 post-cache 复测记录 | `transpiler_continents_after_shiftfix.txt`（补缓存前） | ⚠️ 证据缺口 |
| final_density 0.43 | M11「基本不变」 | 无 post-cache 复测记录 | `transpiler_finaldensity_after_shiftfix.txt`（补缓存前） | ⚠️ 证据缺口 |
| 性能复测 | M11「443ms→17ms」 | cmd-output | `transpiler_grid_after_shiftfix.txt`=456.54ms → `transpiler_grid_after_cache.txt`=16.91ms | ✅ |
| 产物契约 | index.yaml 5 条 + docs/07 小节 | diff 确认新增 | — | ✅ |

**三源核对发现 2 处差异源（cache id 碰撞、y 相关 cache bug）+ 1 处证据缺口（post-cache 对齐复测缺失）。**

---

## 三、审查清单结论（core-judge 8 项）

| # | 清单项 | 结论 |
|---|---|---|
| 1 | 证据完整性（@anchor.test source） | ✅ 探针可复现（seed + 坐标 + cmd-output 落盘）；验证分层 = Partial（探针，非 @anchor.test，docs/07 已声明） |
| 2 | 证据落盘 | ✅ cmd-output 验证记录 + 错误台账 + docs/07 均有可引用落盘 |
| 3 | 三源核对 | ⚠️ 发现 2 处差异源（cache id 碰撞、y 相关 cache bug）+ 1 处证据缺口（post-cache 对齐复测缺失） |
| 4 | 置信度合法 | ✅ 全部 candidate，无 AI 自标 confirmed |
| 5 | 产物契约 | ✅ index.yaml 5 条 + docs/07 小节 + 错误台账更新 |
| 6 | 噪声卡历史 | ✅ 目标（transpiler 性能/对齐）无未解决噪声卡记录（该 session 为性能定位，非运行时失败累积） |
| 7 | retry cap | ✅ 本次为工程修复（补缓存 + 修探针）+ 重测，不消耗 evidence saturation 计数；无超限未声明 |
| 8 | 模块边界 | ✅ 未引用其他领域模块 skill 正文 |

---

## 四、审查意见汇总（各环节推荐状态）

| 环节 | 推荐状态 | 理由 |
|---|---|---|
| 补缓存代码（build/density.rs + src/density.rs） | **建议 candidate（附正确性隐患）** | `flat_cache`/`cache_2d` 的 (x,z) key 语义正确、嵌套借用处理正确；但 cache id 跨生成函数碰撞 + y 相关 cache bug 需修复 |
| 探针污染修复（M10） | **建议 candidate** | 坐标修复正确，修后测出真缓存冷 260μs 可信 |
| 缓存正确性（continents 0.000000 / final_density 0.43 不变） | **保持 draft** | 缺 post-cache 对齐复测；y 相关 cache bug 未被测试覆盖；「不变」是推断非复测 |
| 产物契约（index.yaml + docs/07 + 错误台账） | **建议 candidate** | 完整满足 |
| cache id 碰撞（compute_continents vs fill_final_density 共用 id 0） | **保持 draft（需修复）** | 潜在正确性 bug，当前未 manifest（两函数不同线程），但设计缺陷 |
| y 相关 cache 节点（cache_once 包装 y 相关 inner 用 (x,z) key） | **保持 draft（需修复）** | 正确性 bug，cell grid 采样必然触发，final_density 0.43 含此误差 |

**整体：本次补缓存 + 探针污染修复收尾建议 candidate（confirmed 由人类授予），但需先处理 2 处正确性隐患 + 补 post-cache 对齐复测。**

---

## 五、下一步建议（给主会话/人类）

1. **（必做）补 post-cache 对齐复测**：补缓存后重测 continents 与 final_density 对齐（`transpiler_continents_after_cache.txt` / `transpiler_finaldensity_after_cache.txt`）。当前「不变」是推断非复测，且 y 相关 cache bug 在 `transpiler_alignment.rs`（每 (x,z) 采样 6 y）必然 manifest——复测很可能显示 final_density 对齐变差。
2. **（必做）修 y 相关 cache bug**：`cache_once` 包装 y 相关 inner（spaghetti_3d_rarity y_scale=1.0）时，(x,z) key 返回错误值。需区分缓存节点类型：`flat_cache`/`cache_2d` 用 (x,z) key（xz-only 正确）；`cache_once` 用 (x,y,z) key 或按精确位置缓存；`cache_all_in_cell` 对齐 cell 级语义。
3. **（必做）修 cache id 碰撞**：`cache_id` 每 `build_compute` 重置为 0，导致 `compute_continents` 与 `fill_cell_corner_densities_final_density` 共用 id 0。需用全局递增 id（跨 build_compute 共享计数器）或按函数命名空间隔离。
4. **（建议）验证 y 相关 cache bug 影响**：用独立探针对比「y 相关 cache 节点」在 cell grid 采样下的 transpiler vs 运行时值，量化 cache bug 对 final_density 0.43 的贡献。
5. **（建议）扩大对齐样本**：覆盖 cell 内部任意点 / chunk 边界 clamp / 负 Y 极端，把「transpiler 核心正确」从局部充分提升为更全局的证明。
6. **（必做）confirmed 授予**：以上 1-3 为必做项，处理后再授予 confirmed；若人类认可本次补缓存 + 探针修复方向，可先授予 candidate。

> 本意见为建议非命令；用户是最终拍板者。confirmed 由人类授予。
