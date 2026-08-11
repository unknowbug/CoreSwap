# 07-block-pipeline.md 性能回归小节「修复闭环」更新草稿（2026-08-12）

> **目标路径**：`versions/1.20.1/docs/07-block-pipeline.md`
> **范围**：`## 性能回归实测（2026-08-11 发现 → 2026-08-12 根因定论）` 小节（当前 L64-121）
> **应用方式（主会话就地编辑）**：按下方修正块逐项操作（每块给出定位 marker → 替换/追加的新文本）。状态标注：`🔍 修复中（Phase 2 已启动）` → `✅ 修复闭环（2026-08-12 用户验收）`。
> **草稿状态**：draft（主会话应用 + 用户已验收；数字与 `.investigations/perf-rework/cmd-output/` 落盘文件核对一致）。

---

## 修正块 1：小节头部状态行更新（L67）

**定位 marker**（L67 整行）：
```markdown
> **2026-08-12 根因定论**：主因（H2）= FlatCacheDF **单槽 thread_local 缓存** + buildGrid 角点 `i=4`/`j=4` 越界 → 嵌套 spline 的 FlatCache 收到**邻居 chunk key** → 单槽被污染 → 邻居网格重建**递归蔓延 112 chunk**（rebuild 36,252 = **168×** → spline 调用 **20×**）；放大器（H3）= 多线程 thread_local thrashing（单次 ×16）。结论已过 judge 审查（`.investigations/perf-rework/review-rootcause.md`）并经**用户拍板确认**。状态：🔍 **修复中（Phase 2 已启动）**。
```

**替换为**：
```markdown
> **2026-08-12 根因定论**：主因（H2）= FlatCacheDF **单槽 thread_local 缓存** + buildGrid 角点 `i=4`/`j=4` 越界 → 嵌套 spline 的 FlatCache 收到**邻居 chunk key** → 单槽被污染 → 邻居网格重建**递归蔓延 112 chunk**（rebuild 36,252 = **168×** → spline 调用 **20×**）；放大器（H3）= 多线程 thread_local thrashing（单次 ×16）。结论已过 judge 审查（`.investigations/perf-rework/review-rootcause.md`）并经**用户拍板确认**。状态：✅ **修复闭环（2026-08-12 用户验收）**——修复方案（当前 chunk 上下文绑定，与 Java per-chunk 语义对齐）与验证数据见下方「修复方案（已实施并闭环）」/「修复闭环验证」小节；judge 审查：`.investigations/perf-rework/review-fix-delivery.md`（主结论通过，4 项修正已闭环）。
```

## 修正块 2：「修复方向（2026-08-12 定论）」小节 → 替换为「修复方案（已实施并闭环）」+「修复闭环验证」两小节（L112-116）

**定位 marker**（L112-116 整段）：
```markdown
### 修复方向（2026-08-12 定论）

- **主修复：per-chunk 多槽缓存**（FlatCacheDF/Cache2DDF 单槽 → 最近 4-8 chunk 的网格/值 map，key 不变）。低风险：纯缓存，采样值逐位不变，**不破坏 BK-001 对齐**（8576/3200 零退化铁律）；需保留 k=4 边界命中语义（density.h L700-702）。预期 rebuild 回落 ~6/chunk → spline 回 ~1,000/chunk。
- **后续：线程亲和恢复**（每 chunk 固定线程 / per-thread 缓存绑定 chunk 生命周期）→ 消除 H3 thrashing（单次 27,155ns → ~1,714ns）。
- **改循环顺序无效且不推荐**（H1 非主因：块级不触发 spline；且 aquifer/oreVein 同序读取 densityBuf，改动有未验证的对齐风险）。
```

**替换为**：
```markdown
### 修复方案（已实施并闭环，2026-08-12）

> 设计文档：`.investigations/perf-rework/fix-design.md`（§0 含实现演进注记，已登记 `.artifacts/index.yaml`）；judge 审查：`.investigations/perf-rework/review-fix-delivery.md`（主结论通过 + 4 项修正闭环）。

- **终版：FlatCacheDF 改为「当前生成 chunk 上下文绑定」（与 Java per-chunk 实例语义完全对齐）**。thread_local `g_curChunkX/Z`（density.h L40-41）在 `fillOneChunkCore` 入口 RAII 设置、函数返回恢复 `INT32_MIN`；网格绑定当前 chunk，k/l 相对 `startBiomeX` 计算（`k=(pos.x>>2)-slot.cx*4`），越界 → `delegate.sample(pos)` **直算不重建**。与 Java `ChunkNoiseSampler.java` L836-881 FlatCache（构造时一次性预计算 25 角点、之后纯查表、**永不构建邻居网格**）六维逐条对齐（实例绑定/网格构建/k-l 计算/界内查表/越界直算/边界共享，见 review-fix-delivery.md 审查要点 1 表）。
- **机理（蔓延根除）**：buildGrid 角点 i=4 的 pos 不再用 pos 推导邻居 key——嵌套 spline 的 FlatCache 采样该 pos 时 `cx=g_curChunkX=当前 chunk` → `k=4 ∈ [0,5)` 命中本网格；更远越界 → arg 直算，亦不重建。
- **Cache2DDF**：保留 **16 槽 LRU**（角点共享列可命中，无蔓延风险；review-fix-delivery.md 已确认对齐安全）。
- **初版 16 槽 LRU 已弃用（关键教训）**：FlatCacheDF 也上 16 槽 LRU 时 rebuild 36,252→7,318（5× 降）**但未消除蔓延**（rebuild 203/chunk vs 期望 6，覆盖仍 112 chunk）——16 槽 LRU 仍会为 **pos 推导的邻居 key** 构建网格：多槽只减少重建频率，**不改变「越界=重建」语义**。上下文绑定从根上消除「越界→重建」，与 Java 语义一致，故终版采用（fix-design.md §0 演进注记）。
- **改循环顺序无效且不推荐**（H1 非主因：块级不触发 spline；且 aquifer/oreVein 同序读取 densityBuf，改动有未验证的对齐风险）。

### 修复闭环验证（2026-08-12，终版 ctx 数据）

> 数据文件：`cmd-output/regress_8576_raii.txt`、`regress_3200_raii.txt`、`wgprofile_8576_t1_ctx.txt`、`splinedebug_8576_t1_ctx.txt`（stat_ctx.py 统计）、`bench_8x8_noprof.txt`。数字口径：SPLINEDEBUG 为 `[SPLINE]` 非 leaf 入口行计数（每 chunk 3,032）；WG_PROFILE `spline.sample` 为全量采样计数（每 chunk 5,906），两口径并存，方向一致。

| 指标 | 修复前（08-12 定论） | 16 槽 LRU 初版 | **终版（上下文绑定）** | 结论 |
|---|---|---|---|---|
| FlatCache rebuild | 36,252（~1007/chunk，168×） | 7,318（203/chunk） | **216 = 6.0/chunk** | 完全达期望 ~6 ✓ |
| rebuild chunk 覆盖 | 112（36 生成 + 76 邻居） | 112（蔓延未除） | **36** | 蔓延根除 ✓ |
| CACHE2D miss | 351,536 | — | **23,117** | ↓15× |
| SPLINE（非 leaf 口径） | 66,682/chunk | 14,772/chunk | **3,032/chunk** | 回旧基线 6,250 水平 ✓ |
| spline.sample（WG_PROFILE 全量） | 130,420/chunk | — | **5,906/chunk**（212,622/36） | ↓22× |
| 单线程 wall | 6,533ms（181ms/chunk） | 3,469ms（wgprofile_8576_t1_fixed） | **2,910ms** | 2.2× ✓ |
| bench_chunks 单线程 | ~181ms/chunk（口径：旧 wall 6533/36） | 79.91ms/chunk（bench_fixed_ctx） | **62.38ms/chunk** | 3× ✓ |
| 对齐 8576 / 3200 | 99.9994% / 99.9997% | 同 | **99.9994% / 99.9997%** | 零退化 ✓ |
```

## 修正块 3：「状态与下一步」小节替换（L118-121）

**定位 marker**（L118-121 整段）：
```markdown
### 状态与下一步

- 🔍 **修复中（Phase 2 已启动）**：根因已定论（H2 主因 + H3 放大器，用户拍板 + judge 通过），per-chunk 多槽缓存修复方案已立项实施，验证待闭环（修复完成后以 08-12 同口径计数器复测 rebuild/spline 回落）。
- 相关：Java 桥并发重写（RQ-001~005，✅ 已实施）+ C++ 池改造（✅ 已实施）见 10 时间线 2026-08-11 条目；根因定论见 10 时间线 2026-08-12 条目；通用指纹见 knowledge/discovered/algorithm-fingerprints.md 发现 #10。
```

**替换为**：
```markdown
### 状态与下一步（修复闭环）

- ✅ **修复闭环（2026-08-12 用户验收）**：终版（FlatCacheDF 当前 chunk 上下文绑定 + Cache2DDF 16 槽 LRU）已实施并验证——rebuild 216 = 6.0/chunk（完全达期望）、覆盖 36（蔓延根除）、CACHE2D miss 23,117、SPLINE 3,032/chunk（回旧基线 6,250 水平）、单线程 wall 2,910ms（2.2×）、bench 62.38ms/chunk（3×）、8576/3200 双种子零退化（99.9994%/99.9997%）。judge 审查主结论通过（review-fix-delivery.md），用户验收，性能回归结案。
- 🔍 **剩余课题（独立于本次修复，待续）**：
  1. **多线程无加速**：bench threads=8 62.17ms/chunk ≈ 单线程 62.38ms（仅 ~0.3% 提升）——spline/cache 瓶颈消除后，**aquifer+oreVein 阶段**（wgprofile_8576_t1_ctx.txt 实测 20-52ms/chunk，远超 spline 贡献）成为主导；需线程亲和（root-cause 方案 2）/ aquifer 并行化。
  2. **spline 单次 7,971ns**（WG_PROFILE ctx 口径，wgprofile_8576_t1_ctx.txt L80）：调用量 ↓22× 后的单次成本，**非本次修复引入的劣化**（review 三源不一致 #2 已注明出处），与修复前 1,714ns 为不同测量口径。
  3. **aquifer 阶段 4× 级**（20-52ms/chunk vs 旧基线 6.5-8.9ms）——独立课题。
- 相关：Java 桥并发重写（RQ-001~005，✅ 已实施）+ C++ 池改造（✅ 已实施）见 10 时间线 2026-08-11 条目；根因定论见 10 时间线 2026-08-12 条目；修复闭环见 10 时间线 2026-08-12（补）条目；通用指纹见 knowledge/discovered/algorithm-fingerprints.md 发现 #10（已补修复方案）。
```
