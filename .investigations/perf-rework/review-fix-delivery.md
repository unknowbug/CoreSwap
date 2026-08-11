# 审查意见：性能回归修复交付（Phase 2，FlatCache/Cache2D per-chunk 上下文绑定）

> 角色：core.judge（只出意见，不改 status）· 日期：2026-08-12
> 审查对象：`.investigations/perf-rework/fix-design.md`（修复设计）+ 应用代码（density.h / worldgen_api.cpp）+ 回归数据（cmd-output/*_fixed.txt）
> 三源核对：① 交付快照（fix-design.md / root-cause-draft.md / collect-summary.md）② 应用版代码（density.h L35-41、L636-786；worldgen_api.cpp L586-592；ChunkNoiseSampler.java L557-595、L836-881）③ 回归记录（cmd-output/regress_8576_fixed.txt、regress_3200_fixed.txt、wgprofile_8576_t1_fixed.txt、bench_fixed_ctx.txt）
> 推荐状态：**保持 draft**（主结论通过：修复机制正确、对齐零退化、证据已落盘；附 4 项修正/澄清建议，处理后可建议 candidate）

---

## 逐项结论（审查清单）

| # | 检查项 | 结论 | 依据 |
|---|---|---|---|
| 1 | 证据完整性（Anchorlaw §5.5） | ✅ 通过 | FlatCacheDF/Cache2DDF 的 @anchor.test source="probe:block_probe!FLATCACHE#004"（trace 实测探针）+ WG_PROFILE/WG_SPLINEDEBUG 计数器，非 static 断言 |
| 2 | 证据落盘（spec §1.3） | ⚠️ 基本通过 | regress_8576_fixed.txt / regress_3200_fixed.txt / wgprofile_8576_t1_fixed.txt / bench_fixed_ctx.txt 均已落盘 cmd-output/；但无 regression-record.md 规范命名条目（沿用 collect-summary.md 命名惯例，同上次审查） |
| 3 | 三源核对（spec §4） | ⚠️ 发现 2 处不一致 | 见下「三源不一致清单」（设计文档 vs 最终实现；任务摘要性能数字 vs 落盘文件） |
| 4 | 置信度合法 | ✅ 通过 | 无越权 confirmed；index.yaml 中 root-cause-draft/review-rootcause 均标 candidate；修复交付无 status 篡改 |
| 5 | 产物契约（core.artifact） | ❌ 缺口 | **fix-design.md 未登记 .artifacts/index.yaml**（仅登记 requirements-doc/static-audit/root-cause-draft/review-rootcause）；本次 review-fix-delivery.md 亦需补登 |
| 6 | 噪声卡历史（Anchorlaw §3） | ⚠️ 无法核对 | 工作区无 .artifacts/noise_cards.json（仅 templates/），该目标有无未解决噪声卡无从查证（同上次审查） |
| 7 | retry cap（spec §5.3 / Anchorlaw §9.4） | ⚠️ 缺失声明 | fix-design.md / 交付记录无 retry/验证轮次段；本次属工程修复迭代（不计数逆向假设），修复验证单轮完成，标注缺失即可 |
| 8 | 模块边界（spec §1.6 / §2.5 R5） | ✅ 通过 | 引用均在本模块（density.h/worldgen_api.cpp/Java 源码/知识库），未引用其他领域 skill 正文 |

---

## 审查要点 1：机制正确性（FlatCacheDF 上下文绑定 vs Java 语义）

与 Java ChunkNoiseSampler.java 逐条核对：

| 维度 | Java FlatCache（L836-881） | C++ 修复后（density.h L703-786） | 对齐 |
|---|---|---|---|
| 实例绑定 | per-chunk 实例，startBiomeX 固定 | thread_local g_curChunkX/Z（fillOneChunkCore 设置），单槽 | ✅ 等价模拟 |
| 网格构建 | 构造时一次性 25 角点（y=0） | 首次 sample 该 chunk 时 buildGrid（角点公式不变：(chunkX*4+i)*4, y=0） | ✅ 公式/时机一致 |
| k/l 计算 | k = fromBlock(blockX) - startBiomeX = (x>>2) - chunkX*4 | k = (pos.x>>2) - slot.cx*4 | ✅ 逐位一致 |
| 界内查表 | k,l ∈ [0,5) → cache[k][l] | k,l ∈ [0,5) → grid[l*5+k] | ✅ |
| 越界处理 | delegate.sample 直算（不重建） | arg->sample(pos) 直算（不重建） | ✅ **根因消除点** |
| 边界共享 | x=chunkX*16+16 → k=4 ∈ [0,5) 命中本网格 | x=cx*16+16 → k=4 ∈ [0,5) 命中本网格 | ✅ 语义保留 |

**buildGrid 角点 i=4 不再触发邻居网格重建的机理（核实成立）**：
- buildGrid(chunkX,chunkZ) 角点 i=4 → p.x = (chunkX*4+4)*4 = (chunkX+1)*16（下一 chunk 首列）。
- 嵌套 spline 的 locationFunction FlatCache 采样该 pos 时：cx = g_curChunkX = chunkX（上下文绑定，非 pos 推导）→ k = ((chunkX+1)*16 >> 2) - chunkX*4 = (chunkX*4+4) - chunkX*4 = **4 ∈ [0,5)** → 命中本网格 → **不触发邻居 chunk buildGrid**。
- 更远越界（如对角邻居 (chunkX+2)*16 → k=8）→ arg->sample 直算，亦不重建。
- 结论：**蔓延根除机理成立**，与实测 rebuild 216（= 36 chunk × 6 实例，恰每实例每 chunk 1 次）、chunk 覆盖 36（修复前 112）吻合。

**边界 k=4 命中语义**（x=cx*16+16 → k=4）：保留 ✅。且经推演，FlatCache 为纯函数缓存——任意界内 pos 的返回值为 `arg->sample((pos.x>>2)*4, 0, (pos.z>>2)*4)`（同一 biome 网格点），与绑定哪个 chunk 无关 → 缓存改造零退化在数学上成立。

**Cache2DDF 16 槽 LRU**（L636-701）：key=block 级 (x,z)，miss 时替换 LRU 槽 + arg->sample，纯函数缓存 → 值正确；跨 chunk 复用 16 槽（Java per-chunk 每 chunk 重建）仅影响命中率不影响值 → 对齐安全 ✅。注：任务摘要中「FlatCache/Cache2D per-chunk 上下文绑定」与 fix-design.md 设计（两处均 16 槽 LRU）**不完全一致**——最终实现为混合方案（FlatCacheDF=上下文绑定单槽；Cache2DDF=16 槽 LRU），方向更贴合 Java 语义，但设计文档未同步更新（见三源不一致 #1）。

## 审查要点 2：对齐风险（双种子 99.999x% 保持是否可信）

**可信**，理由：
1. 修复为**纯缓存路径改造**：SplineDF（L789-865）、InterpolatedDF（L482-592）、buildGrid 角点公式均未改动（git 子代理核实：density.h 86 行改动全部在本次范围内，无采样/插值公式变更）。
2. 任何坐标的返回值 = 同一坐标的 arg 采样（biome 网格点查表）或 arg 直算 → 纯函数 → 与旧实现逐位一致（同 root-cause §3.3 同坐标网格值逐位相同实证逻辑）。
3. **落盘验证**：regress_8576_fixed.txt TOTAL match=3538922/3538944 (**99.9994%**) 与修复前 collect-summary 的 99.9994% 一致；regress_3200_fixed.txt (**99.9997%**) 逐位核对无退化。剩余 0.0006%/0.0003% 为历史已有（beardifier 相关，非本次引入，per-chunk 几乎全 100%）。

**诊断路径回退**（wg_sample_density/wg_sample_named 直接采样 finalDensity）：
- 代码 L729-730：g_curChunk==INT32_MIN 时回退 pos 推导 key，与旧实现一致 ✅。
- ⚠️ **声明不完整**：fillOneChunkCore（L591-592）设置 g_curChunkX/Z 后**从不恢复 INT32_MIN**。同线程在 fill 之后调用诊断路径时，g_curChunk 残留为"上次生成 chunk"（非 INT32_MIN）→ 走上下文分支而非回退分支。经推演，FlatCache 纯函数性使**采样值仍正确**（界内查表值 = biome 网格点 arg 值，界外直算；均与绑定 chunk 无关），但**行为/性能与旧实现不同**（不按 pos 重建）。建议 fillOneChunkCore 末尾恢复 g_curChunkX/Z=INT32_MIN（RAII），并修正注释「未设置时回退」→「未设置或已恢复时回退」，避免诊断路径语义漂移。

## 审查要点 3：线程安全

- g_curChunkX/Z 为 `thread_local`（density.h L40-41）→ 每线程独立，无跨线程共享 ✅。
- fillOneChunkCore 每 chunk 完整处理：设置上下文 → density → aquifer → surface → 输出全部在函数内单线程完成（L586-892），无跨线程迁移 ✅。
- CoreSwapPool 多 worker（L1054-1064）：每个 worker 线程处理不同 chunk，各自 thread_local 上下文独立 ✅；单线程模式连续 fill 36 chunks → 每 chunk 重设上下文 → 216 rebuild 恰为期望值 ✅。
- 结论：线程安全成立，无跨线程上下文污染风险。

## 审查要点 4：遗留问题（多线程无加速 / spline 单次成本）

- **多线程无加速**（bench_fixed_ctx.txt：threads=1 79.91ms/chunk vs threads=8 70.99ms/chunk，仅 11%）：属本次修复范围外的新课题。本次修复目标（消除 H2 rebuild 爆炸）已达成（216 vs 1007）；多线程剩余瓶颈看落盘 wgprofile_fixed 为 aquifer+oreVein（20-52ms/chunk，远超 spline 贡献），系独立课题（root-cause 方案 2 线程亲和 / aquifer 并行化），**不是本次修复引入的副作用** → judge 认可记录为独立课题。
- **spline 单次**：落盘 wgprofile_8576_t1_fixed.txt 显示 spline 单次 1,737ns（修复前 1,714ns）→ 修复未引入单次劣化；「旧基线 992ns」为更早口径（07篇/8-06），与本次无关。任务摘要的「7,971ns」未在落盘文件找到，疑为另一轮运行/bench 场景，**需主会话澄清出处**（见三源不一致 #2）。
- 结论：遗留问题记录方向合理，属于范围外；仅需统一数字口径。

## 审查要点 5：产物契约

- **fix-design.md 未登记 .artifacts/index.yaml**（已读 index.yaml 确认，仅 requirements-doc/static-audit/root-cause-draft/review-rootcause）→ 违反 core.artifact，需补登。
- 本次 review-fix-delivery.md 落盘后亦应补登（kind: review）。

---

## 三源不一致清单

| # | 不一致 | 判定 | 建议 |
|---|---|---|---|
| 1 | **fix-design.md 设计 vs 最终实现**：设计 §3.3 描述 FlatCacheDF/Cache2DDF 均为 16 槽 LRU 多槽；最终实现为 FlatCacheDF=上下文绑定单槽 + Cache2DDF=16 槽 LRU 混合方案 | 实现更贴合 Java per-chunk 语义（每 chunk 每实例恰 1 次 buildGrid），方向更优；但设计文档未同步更新 | 更新 fix-design.md 记录最终实现（或补"实施变更说明"段） |
| 2 | **性能数字口径**：任务摘要（SPLINE 3,032/chunk、wall 2910ms、bench 62.38ms/chunk、spline 单次 7,971ns）与落盘 wgprofile_8576_t1_fixed.txt（spline.sample=1,028,986=28,583/chunk、wall=3469.2ms、单次 1,737ns）及 bench_fixed_ctx.txt（79.91ms/chunk）不完全一致 | 方向一致（均大幅改善），具体数值存在多轮运行/口径差异（SPLINEDEBUG 非 leaf 口径 vs WG_PROFILE 全量口径） | 以落盘文件为准；主会话在交付摘要中注明所用口径与文件 |

## 汇总 verdict

- **主结论：通过**。修复机制（FlatCacheDF 当前 chunk 上下文绑定 + 越界直算不重建、Cache2DDF 16 槽 LRU）与 Java per-chunk 实例语义逐条对齐，边界 k=4 语义保留，buildGrid 角点 i=4 不再触发邻居网格重建（机理经代码路径推演成立，实测 rebuild 216/覆盖 36 吻合）；纯缓存路径改造零退化在数学上成立，双种子 99.9994%/99.9997% 落盘验证与修复前一致；thread_local + fillOneChunkCore 单线程完整处理保证线程安全。性能大幅改善方向有落盘证据支持（wall 3469 vs 6533ms ≈ 1.9x、bench 79.91 vs 181 ≈ 2.3x）。
- **需修正/澄清项**（不阻塞 draft，处理后可建议 candidate）：
  1. fix-design.md 未登记 index.yaml，且设计与最终实现不一致需更新（产物契约 + 文档同步）
  2. fillOneChunkCore 设置 g_curChunkX/Z 后不恢复 → 诊断路径回退声明不完整（建议 RAII 恢复 + 修正注释；不产生错误采样值，中低风险）
  3. 性能数字口径统一：任务摘要数字 vs 落盘文件（以落盘为准，注明 SPLINEDEBUG/WG_PROFILE 口径）
  4. retry 轮次记录缺失（spec §5.3）+ 噪声卡无法核对（文件缺失）
- 未发现越权标 confirmed；意见为建议，最终拍板权在用户。
