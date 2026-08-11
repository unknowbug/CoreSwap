# 审查意见：引擎级性能回归根因定论（root-cause-draft.md）

> 角色：core.judge（只出意见，不改 status）· 日期：2026-08-12
> 审查对象：`.investigations/perf-rework/root-cause-draft.md`（core.worker 产出，状态 draft）
> 三源核对：分析产物 + 原始数据（collect-summary.md / wgprofile_t1 / wgprofile_mt / splinedebug 统计）+ 代码证据（density.h L627-846、worldgen_api.cpp L669-681、ChunkNoiseSampler.java L305-355）
> 推荐状态：**保持 draft**（主结论成立，附 7 项修正/澄清建议，处理后可建议 candidate）

---

## 逐项结论（审查清单）

| # | 检查项 | 结论 | 依据 |
|---|---|---|---|
| 1 | 证据完整性（Anchorlaw §5.5） | ✅ 通过 | source 均为 trace/memory 实测（WG_PROFILE/WG_SPLINEDEBUG 计数器）+ 代码定位，非 static 断言 |
| 2 | 证据落盘（spec §1.3） | ⚠️ 基本通过 | collect-summary.md、wgprofile_t1/mt、splinedebug_8576_t1.txt（537MB）均已落盘；仅 regression-record 未按规范命名（collect-summary.md 充当记录条目） |
| 3 | 三源核对（spec §4） | ⚠️ 发现 4 处不一致 | 见下「三源不一致清单」；root-cause 引用的 density.h/worldgen_api.cpp 行号与当前工作区一致 |
| 4 | 置信度合法 | ✅ 通过 | 全文状态标 draft；grep 确认 confirmed 仅出现在「不得视为 confirmed」声明；§6 诚实声明 4 个不确定点；无越权确认 |
| 5 | 产物契约（core.artifact） | ❌ 缺口 | root-cause-draft.md 已落盘但 **.artifacts/index.yaml 未登记**（仅登记 requirements-doc/static-audit） |
| 6 | 噪声卡历史（Anchorlaw §3） | ⚠️ 无法核对 | 工作区无实际 `.artifacts/noise_cards.json`（仅 templates/noise_cards.json），该目标有无未解决噪声卡无从查证 |
| 7 | retry cap（spec §5.3 / Anchorlaw §9.4） | ❌ 缺口 | root-cause-draft.md 无 retry/验证轮次记录段（H1/H2/H3 单轮验证，未声明轮次，亦未声明「逆向假设验证 ≤3」） |
| 8 | 模块边界（spec §1.6 / §2.5 R5） | ✅ 通过 | 引用均在本模块范围（density.h/worldgen_api.cpp/Java 源码/知识库/07 篇文档），未引用其他领域 skill 正文 |

---

## 审查要点 1：数字闭环校验

逐项复算（算术均成立）：

```
351,536 ÷ 36 = 9,764.9 ≈ 9,765 ✓
351,536 ÷ 25 = 14,061.4 → 14,061 rebuild ✓（14,061 × 25 = 351,525 ≈ 351,536，差 11 = 0.003%）
4,695,145 ÷ 351,536 = 13.355 ≈ 13.36 ✓
4,695,145 ÷ 36 = 130,420.7 ≈ 130,420 ✓
9,765 × 13.36 = 130,460 ≈ 130,420 ✓
4,695,145 ÷ 2,400,550 = 1.9559 ≈ 1.96 ✓（WG_PROFILE 含 leaf / SPLINEDEBUG 非 leaf 口径正确：sampleImpl L775 isLeaf 直接 return 不打印）
36,252 ÷ 36 = 1,007 ✓
```

**口径问题（需 worker 澄清，不推翻主结论）**：
- collect-summary §2 记录 **CACHE2D miss 有 4 个 cacheId**；root-cause §2/§6 说「含 cache_2d 实例 3 个：factor/jaggedness/offset」。第 4 个 cache_2d 实例来源未解释（可能为某 spline 的 locationFunction——SplineDF::sampleImpl L779-780 支持 FlatCache/Cache2D 检测）。若第 4 个 cache_2d 的 miss 也计入 351,536，则「含 cache_2d 的 rebuild 14,061 = miss/25」反推值偏高，连带 22,191（无 cache_2d rebuild）= 36,252−14,061 失真。root-cause §6 不确定点 2 已诚实声明反推，但**未指出 4 个 cacheId 的事实**。建议补入不确定点。

## 审查要点 2：机制逻辑校验（H2 代码路径）

与 density.h 代码逐条核对：

- **L734-735**：`p.x = (chunkX*4+i)*4`，i=4 → `(chunkX+1)*16` = 下一 chunk 首列 ✓（root-cause 表述正确）
- **L687**：FlatCache key = `(pos.x>>4, pos.z>>4)` chunk 级 ✓
- **L693**：rebuild 条件 = `slot.key==INT64_MIN || kc<0 || lc<0 || kc>=GRID || lc>=GRID`——root-cause §3.1「slot.key 不匹配且 kc/lc 越界 → buildGrid」与代码一致 ✓
- **L700-702**：k/l 界内（含边界 k=4/l=4）返回网格值，不 rebuild ✓
- **L684-686 / L718-721**：thread_local 单槽确认（每实例 1 槽，vector 按 cacheId 索引）✓

**「边界 k=4 命中」vs「角点 i=4 触发邻居 rebuild」不矛盾**（关键分辨）：
- L700-702 的边界命中作用于**同一 FlatCache 实例的后续采样**：只要 pos 坐标落在该实例 slot 已建网格覆盖范围（含边界列/行，kc/lc ∈ [0,4]），即返回网格值——这是防同实例重复 rebuild 的设计。
- buildGrid 角点 i=4 的邻居坐标作用于**嵌套层级**：外层 FlatCache buildGrid 角点 i=4 → 坐标传入内层 cache_2d（arg）→ miss → 内层 spline 树 → 内层 spline 的 locationFunction（另一 FlatCache）收到**邻居 chunk 坐标** → 该实例的 slot 若缓存 chunk 与坐标 chunk 不相邻（kc/lc 越界）→ rebuild 邻居网格 → 递归。
- 因此「角点 i=4 必然触发邻居 rebuild」并不恒真——仅当**嵌套 FlatCache 的 slot 与当前坐标 chunk 失配且越界**时触发（root-cause §3.2「若其 slot 已被污染」已隐含此条件）。root-cause 表述略简化但方向正确，与实测蔓延 112 chunk（含左下对角邻居 (44,-28)）一致。
- 结论：**机制成立**，与代码一致；建议在 3.2 补一句「同 x/z 相邻的边界命中可挡掉部分方向蔓延，越界（对角/隔远 chunk）才 rebuild」，使表述更精确。

## 审查要点 3：置信度合法性

- 状态 draft、无越权 confirmed、4 项不确定点诚实声明 —— **通过**。
- 补位意见：§6 不确定点可追加「CACHE2D 第 4 个 cacheId 来源」与「08-11 vs 08-12 数据差异」（见下）。

## 审查要点 4：知识库一致性

- **发现 #10**（thread_local 缓存 vs 跨线程执行模型，candidate）：root-cause H3（thrashing 放大器 16×）与之一致 ✓；H2（buildGrid 嵌套递归叠加因素）亦与之呼应并将该因素从「叠加」升为主因，属合理扩展 ✓。
- **07 篇 candidate 1/2/3**：candidate1（thread_local 冲突）= H3 放大器；candidate2（buildGrid 递归）= H2 主因；candidate3（计数器口径）= 旧基线推演说明。对齐 ✓。
- **⚠️ 未讨论的数据差异**：知识库/07 篇的 **08-11 实测**（rebuild 438,092 ≈ spline 调用数、Cache2D miss 458,281、spline 单次 20,598ns、spline 338 万次）与 **08-12 新采集**（rebuild 36,252 = spline 0.77%、miss 351,536、单次 t1 1,714ns/mt 27,155ns、spline 470 万次）差异巨大，root-cause 全文未提及。08-12 数据下「rebuild ≈ 访问总数（命中率≈0）」指纹不再成立（rebuild 仅占 spline 0.77%，实际是「rebuild 168× × 13.36 spline/miss」的级联放大）。**需 worker 说明两轮数据口径/环境差异（可能 08-11 为早期更大范围或多线程全量环境），并据 08-12 修正发现 #10 的指纹描述**——否则知识库与本次定论存在量级矛盾。

## 审查要点 5：修复方向风险评估（BK-001 零退化）

- **方案 1（per-chunk 多槽缓存，推荐）**：**逐位不变成立**。理由：FlatCache/Cache2D 均为纯函数缓存，同输入坐标采样值确定（root-cause §3.3 以 splinedebug L133 vs L62484 同 chunk 网格值逐位相同实证）。多槽仅改变「哪些 chunk 网格被缓存」，任一坐标的采样值要么由 buildGrid 在同一坐标重算、要么取自缓存（值即该坐标 buildGrid 结果）→ 逐位一致。实现风险点在于**必须保留 k=4 边界命中语义（L700-702）**，否则边界坐标改走 arg->sample 直接采样路径——因 arg 亦为纯函数/缓存链，值仍不变，但需验证不引入路径分歧。root-cause §5 备注已识别此点 ✓。
- **方案 4（改循环顺序）**：root-cause 自评中高对齐风险且不推荐——judge 认可，与 BK-001 铁律及 aquifer 同序读取约束一致。
- 结论：修复方向风险评估**合理**。

## 三源不一致清单（审查要点 3 明细）

| # | 不一致 | 判定 | 建议 |
|---|---|---|---|
| A | **wall 时间**：原始 wgprofile_t1 `wall=6448.0ms (179.11ms/chunk)` vs collect-summary/root-cause `6533.3ms (181.48ms/chunk)` | 次要偏差（可能多次运行/口径），不推翻结论 | root-cause 可注明取自 collect-summary 而非原始文件 |
| B | **Java 循环顺序**：collect-summary §3「Java ChunkNoiseSampler 为 x→z→y 列主序」vs root-cause §4.1「y 外层（y→x→z）」 | **Java 源码 L313-328 证实 root-cause 正确**（verticalCellBlockCount 外层 → horizontalCellBlockCount x 中层 → z 内层） | collect-summary 断言有误需修正；root-cause 已独立核对并纠正，但未显式标注与数据源矛盾 |
| C | **CACHE2D cacheId 数**：collect-summary 4 个 vs root-cause 3 个含 cache_2d 实例 | 未解释第 4 个来源 | 见审查要点 1 |
| D | **08-11 vs 08-12 实测数据**：知识库/07 篇 vs 本次 | 差异巨大未讨论 | 见审查要点 4 |

## 汇总 verdict

- **主结论：通过**。H2 主因（FlatCacheDF 单槽缓存 + buildGrid 角点 i=4 越界 → 嵌套递归蔓延 112 chunk，rebuild 168×）、H3 放大器（thread_local thrashing 16×）、H1 非主因（y 主序注释矛盾已实证不触发 spline），机制与代码一致，数字闭环可复核，置信度标注合法，修复方向（per-chunk 多槽）不破坏 BK-001。
- **需修正/澄清项**（不阻塞 draft，处理后可建议 candidate）：
  1. CACHE2D 第 4 个 cacheId 来源（含 cache_2d rebuild 拆分反推的脆弱点）
  2. 08-11 vs 08-12 数据差异 + 发现 #10 指纹描述修正
  3. collect-summary 的 Java 循环顺序断言修正
  4. index.yaml 未登记 root-cause-draft.md（产物契约）
  5. retry 记录缺失（spec §5.3）
  6. 噪声卡历史无法核对（文件缺失）
  7. wall 时间 6448.0 vs 6533.3 来源注明（次要）
- 未发现越权标 confirmed；意见为建议，最终拍板权在用户。
