# 审查意见：density 性能优化提交 aae119d（spline 扁平化 + InterpolatedDF 边界列复用）

> 角色：core.judge（只出意见，不改 status，不改代码）· 日期：2026-08-13
> 审查对象：`aae119d` perf(density)（代码 2 文件）+ `ae9a3b9` docs(perf-rework) phase0-2 调查产物（11 文件，同一轮交付）
> 三源核对：① 应用版代码（git HEAD=ae9a3b9 的 density.h / density_builder.h，逐行比对 aae119d diff）② git 元数据（aae119d --stat/--name-only、regress/bench 文件提交归属）③ 落盘数据（cmd-output/phase0|1|2_*_8x8.txt + regress_8576/3200_*.txt + analyze_stagetimer.py 现场复跑聚合）
> 推荐状态：**保持 draft**（代码语义无损成立，但「零退化」证据链不闭合 + 产物契约缺口 + 多线程根因论断不精确；补齐后可再议 candidate）

---

## 一、逐项结论（审查清单）

| # | 检查项 | 结论 | 依据 |
|---|---|---|---|
| 1 | 代码正确性：SplineDF 扁平化无损 | ✅ 通过 | 逐行核对 Hermite 公式等价（见要点 1） |
| 2 | 代码正确性：边界列复用无损 | ✅ 通过 | CELL_X=4 坐标对齐成立（见要点 2） |
| 3 | 声称② -23.7%（61.7→47.1ms） | ✅ 数字吻合 | 现场复跑 analyze_stagetimer，[A] threads=1 median 精确匹配 |
| 4 | 声称③ -1.7%（47.1→46.3ms） | ⚠️ 噪声级/选择性报告 | density 降 0.8ms，但总 wall 71.68→72.06ms 反升（见要点 3） |
| 5 | 声称① 零退化（99.9994%/99.9997%） | ❌ 证据链不闭合 | 无针对 aae119d 的 block_probe 回归落盘（见要点 4） |
| 6 | 声称④ 多线程根因 | ⚠️ 术语不精确 + 预估未兑现 | 「FlatCache buildGrid」混淆 InterpolatedDF::buildGrid（见要点 5） |
| 7 | 证据落盘（spec §1.3） | ❌ 缺口 | 无 regression-record.md；scan invalid=0 无运行记录 |
| 8 | 产物契约（core.artifact） | ❌ 缺口 | phase0-* 系列 11 文件未登记 .artifacts/index.yaml |
| 9 | 置信度合法 | ✅ 通过 | 无越权 confirmed；requirements-doc confirmed 为用户 08-11 合法授予 |
| 10 | @anchor 合法性（Anchorlaw §5.5） | ✅ 通过 | 本提交未新增 anchor；保留的 source=probe:block_probe! 为 trace 类 |
| 11 | 噪声卡历史（Anchorlaw §3） | ⚠️ 无法核对 | 工作区无 .artifacts/noise_cards.json（上轮已指出，未补） |
| 12 | retry cap（spec §5.3） | ⚠️ 缺口 | 逆向假设验证轮次未声明；bench 为工程修复迭代不计，但无记录段 |

---

## 要点 1：SplineDF 扁平化语义无损（逐行核对）

diff 将递归 `shared_ptr<SplineDF>` 树改为扁平 `nodes[]/locations[]/derivatives[]/subIdx[]` + 整数索引，`sampleImpl` → `sampleNode`。逐项比对：

- **叶子判定**：原 `isLeaf` → 新 `Node.n==0`（density.h:839 `if (nd.n == 0) return fixedValue`）✅ 等价。
- **二分查找**：原 `apply` 的 `lo/hi` 二分（`if (f < locations[mid]) hi=mid; else lo=mid+1`）与新 `sampleNode` 完全相同 ✅。
- **i<0 分支**：原 `sampleOutsideRange(f,pos,0)` → `idx=0, d=derivatives[0], base=subSplines[0]`；新 `d=ders[0], base=sampleNode(subs[0]), r=base+d*(f-locs[0])` ✅ 逐位同。
- **i==n-1 分支**：原 `idx=n-1`；新 `idx=n-1` ✅ 同。
- **中间 Hermite 插值**：`p=l*(h-g)-(ov-nv); q=-m*(h-g)+(ov-nv); r=lerp(kd,nv,ov)+kd(1-kd)lerp(kd,p,q)` 原文照搬 ✅。
- **n==1 边界**：原 `if (n==1) return sampleOutsideRange(f,pos,0)` 提前返回；新代码走通用二分，n==1 时 i∈{−1,0} 均落入 i<0 / i==n−1 分支，结果同为 `base+ders[0]*(f-locs[0])` ✅。
- **min/max**：原 `computeMin/Max` 遍历 subSplines；新 `nodeMin/Max` 递归遍历子节点，n==0 返 fixedValue ✅。

**构建阶段**（density_builder.h:192-211 `buildSplineNode`）：先递归填子节点、再 `addNode` 登记本节点、最后 `addPoint`——保证 `locBegin/subBegin` 连续。一个**非语义但需记录**的差异：原代码 `locationFunction=buildNode(*coord)` 在遍历 points **之前**构建，新代码在 `addNode` 时（遍历 points **之后**）才构建 → **cacheId 分配顺序改变**（InterpolatedDF/FlatCacheDF 构造顺序）。因每个实例用独立 cacheId 索引独立 thread_local slot，采样值不受影响，**无损成立**，但值得在交付说明中标注「cacheId 顺序非确定性，不影响对齐」。

## 要点 2：边界列复用语义无损（坐标对齐核对）

density.h:485 `CELL_X=4` → `GX=16/4+1=5`，GX−1=4。左邻 chunk 的 gx=4 列坐标 `(chunkX−1)*16 + 4*CELL_X = chunkX*16`，等于当前 chunk gx=0 列坐标（density.h:604 `p.x = chunkX*16 + gx*CELL_X`，gx=0）✅。y/z 对齐由同实例固定 minY/height + 同 chunkZ 保证 ✅。

- 复用条件严格：density.h:596 `reuseLeft = (slot.edgeCX == chunkX−1 && slot.edgeCZ == chunkZ)`，edgeCol 始终存「最近一次 buildGrid 的 gx=4 列」，故条件命中时 edgeCol 必为左邻 gx=4 列 = 当前 gx=0 列，采样纯函数（density 树无状态）→ **逐位无损**。
- 多线程正确性：edgeCol 存于 `tlSlots()`（thread_local），跨线程隔离 → 并行时复用失效（性能损失）而非错误复用 ✅。
- **实现范围 vs 设计差距**：phase1-design.md 预估「-36% buildGrid 采样」含 x/z 双向边界，但实现**只做 x 方向左邻列**（gx=0，上限 245/1225=20%），未做 z 方向、未做右邻。这是 -1.7% 远低于预估 -28% 的直接原因之一。

## 要点 3：声称③「-1.7%」是噪声级，且端到端总 wall 反升

现场复跑 analyze_stagetimer.py（[A] threads=1 段，n=128=64chunk×2rep）：

| 文件 | density median | density mean | density min–max | 总 wall [A] t=1 |
|---|---|---|---|---|
| phase0_baseline | 61.7 | 59.3 | 40.7–79.3 | 92.08 ms/chunk |
| phase1_splineflat | 47.1 | 48.0 | 40.0–76.3 | 71.68 ms/chunk |
| phase2_edgereuse | 46.3 | 47.6 | 39.4–78.0 | **72.06 ms/chunk** |

- density 阶段 47.1→46.3（−0.8ms）在 min–max 波动（±38ms）内，**不足以证明有效果**；且**总 wall 从 71.68 升到 72.06ms（+0.5%）**，density 之外的阶段（aquifer/surface）同轮噪声完全抵消了 −1.7%。
- commit message 声称「-1.7%」只取 density 阶段 median，未披露总 wall 反升 +0.5%——**选择性报告**，建议在交付说明补总 wall 口径。

## 要点 4：声称①「零退化」证据链不闭合（重大）

- `aae119d` 只提交 density.h + density_builder.h（git --name-only 确认），**未附带任何回归证据**。
- 声称的 99.9994%/99.9997% 与 `regress_8576_raii.txt`/`regress_3200_raii.txt` 数字**完全一致**，而这两个文件归 `c0ac286`（08-12 上一轮 FlatCache 修复）——是**复用上轮数字 = BK-001 基线**，非本提交后的对齐回归。
- `ae9a3b9` 归档的 cmd-output 仅 4 个 bench 文件（phase0_baseline_4x4/8x8、phase1_splineflat、phase2_edgereuse），全部是 `bench_chunks.exe`（吞吐基准，bench_chunks.cpp 只调 wg_fill_blocks_multi 测 wall，**无对齐对比**），无任何 `block_probe.exe` 对齐回归。
- 结论：**「spline 扁平化 + 边界列复用后 BK-001 零退化」无本提交的运行时证据支撑**。语义无损成立（要点 1/2）是静态推导，不能替代端到端回归；尤其边界列复用引入了 thread_local 可变状态（edgeCol），属「重构 + 状态新增」，更需 block_probe 回归背书。

## 要点 5：声称④多线程根因——术语不精确 + 预估 vs 实测未闭环

- **实测确认「未解决」属实**：[A] threads=8 density median，phase0=460.8ms、phase1=478.3ms、phase2=449.6ms——扁平化后 8t 膨胀**不降反略升**（478 vs 460），边界复用也未见 8t 回落。commit message 诚实声明「NOT resolved」✅。
- **术语不精确**：phase0-interp-measurement.md 定位的大头是 **InterpolatedDF::buildGrid**（interpGrid 计数器 238 次 × 1225 角点 = 86.5%），而 FlatCacheDF::buildGrid 是 5×5=25 角点。commit message 的「FlatCache buildGrid tree traversal (per-chunk-per-instance)」混淆了两个同名 buildGrid；准确表述应为「InterpolatedDF::buildGrid 触发的 density 树遍历（含嵌套 FlatCacheDF/Cache2DDF 采样）」。
- **预估 vs 实测矛盾未闭环**：phase0-quantify.md 预估「spline 扁平化…10× 膨胀有望大幅回落」，实测 8t 未回落（略升）。归档的 phase0-* 文档保留乐观预估，**未回填实测证伪结论**，与 commit message 的「未解决」表述存在文档内不一致。

## 三源不一致 / 证据缺口清单

| # | 缺口 | 判定 | 建议 |
|---|---|---|---|
| A | 零退化无本提交回归落盘 | 重大 | 补跑 block_probe 8576+3200 对齐回归并落盘 regress 文件 |
| B | phase0-* 11 文件未登记 .artifacts/index.yaml | 契约缺口 | 补登（与上轮 c0ac286 同类，上轮已补，本轮复发） |
| C | scan_cpp_anchors invalid=0 无运行记录（仅 phase1-design L41 / phase0-architecture-design L51 声明） | 证据缺口 | 落盘 scan 运行输出；另 scripts/scan_cpp_anchors.py 未被 git 跟踪（git ls-files 空） |
| D | 无 regression-record.md（spec §1.3；上轮 review-rootcause 已指出，仍缺） | 规范缺口 | 建 perf-rework/regression-record.md |
| E | retry/验证轮次未声明（spec §5.3） | 记录缺口 | 补逆向假设验证轮次段 |
| F | 噪声卡历史无法核对（无 .artifacts/noise_cards.json） | 环境缺口 | 沿用上轮结论，不阻塞 |

---

## 汇总 verdict

- **代码正确性：通过**。SplineDF 扁平化（Hermite 公式逐位等价，含 n==1/i<0/i==n−1/min-max 全边界）与 InterpolatedDF 边界列复用（CELL_X=4 坐标对齐 + 严格复用条件 + thread_local 隔离）**语义无损成立**，静态逐行核对无发现。
- **性能声称：② -23.7% 数字吻合且可信（n=128 median）；③ -1.7% 为噪声级且端到端总 wall 反升 +0.5%，属选择性报告；① 零退化缺本提交回归落盘（复用上轮数字）；④ 多线程根因术语不精确（FlatCache vs Interpolated buildGrid）且 spline 扁平化未兑现多线程预估。**
- **证据链/契约：不闭合**（缺口 A–F）。核心阻塞项为 A（零退化回归落盘）与 B（index.yaml 登记）。
- **推荐状态：保持 draft**。补齐 A/B 后可再议 candidate；建议 commit message 与 phase0-* 文档回填「多线程预估未兑现」与「总 wall 口径」两处修正。
- 未发现越权标 confirmed；本意见为建议，最终拍板权在用户。
