# discovered/algorithm-fingerprints.md 发现 #10 修正草稿（2026-08-12 根因定论版）

> **目标路径**：`knowledge/discovered/algorithm-fingerprints.md`
> **范围**：发现 #10（当前 L235-264，UTF-8 编码，无编码转换需求）。
> **应用方式（主会话）**：按下方「修正块」逐项就地修改（每块给出定位 marker → 替换文本）。核心结论（thread_local 单槽 vs 跨线程执行模型失配）**不变**；本次修正 = ① 补充 08-11 vs 08-12 测量口径差异（judge 审查要点 4）② 将「叠加因素」升级为主因机制并补充 **H2 新指纹**（buildGrid 角点越界 → 嵌套 FlatCache 邻居 key 污染 → 递归蔓延）③ 置信度更新（已定论）。
> **草稿状态**：draft（主会话应用；修复验证待 Phase 2 闭环——置信度是否标 confirmed 由主会话按知识库惯例定，本次建议 confirmed 理由已写在块 1）。

---

## 修正块 1：标题 + 元信息 + 头部修正注记

**定位 marker**（L235-241）：
```markdown
## 发现 #10: thread_local 缓存与「每 chunk 跨线程」执行模型冲突 → 缓存命中率归零的性能回归指纹

**发现时间:** 2026-08-11
**发现者:** worker（perf-rework 性能回归调查）
**来源定位:** MC 1.20.1 主世界密度求值缓存（versions/1.20.1/docs/07-block-pipeline.md 2026-08-06 纯算法优化链 FlatCache/Cache2D）+ `.investigations/perf-rework/`（WG_PROFILE 实测 2026-08-11）
**置信度:** candidate（FlatCache 失效机制已 WG_PROFILE 实测坐实：命中率≈0；根因修复未验证）
**module:** perf
```

**替换为**：
```markdown
## 发现 #10: thread_local 缓存与「每 chunk 跨线程」执行模型冲突 → 缓存命中率归零的性能回归指纹

**发现时间:** 2026-08-11（2026-08-12 根因定论确认）
**发现者:** worker（perf-rework 性能回归调查）
**来源定位:** MC 1.20.1 主世界密度求值缓存（versions/1.20.1/docs/07-block-pipeline.md 2026-08-06 纯算法优化链 FlatCache/Cache2D）+ `.investigations/perf-rework/`（WG_PROFILE/WG_SPLINEDEBUG 实测 2026-08-11/08-12）
**置信度:** confirmed（机制已 WG_PROFILE/WG_SPLINEDEBUG 实测坐实；2026-08-12 根因定论经 judge 通过 + 用户拍板；修复验证待 Phase 2 闭环后补数据）
**module:** perf

> **2026-08-12 修正（judge 审查要点 4 + 主因升级）**：08-11 实测（rebuild 438,092 / spline 单次 20,598ns）与 08-12 实测（rebuild 36,252 / 单次 1,714ns）差异巨大——两轮测量口径不同（多线程 thrashing 环境粗计数器 vs 单线程精确统计），见「观察」节口径说明；核心指纹结论不变，叠加因素升级为主因机制（H2），见「主因机制」节。
```

## 修正块 2：「指纹信号」段追加 08-11 vs 08-12 口径说明

**定位 marker**（L249，指纹信号整段）：
```markdown
**指纹信号**：缓存重建/失效计数 ≈ 缓存访问总数（命中率≈0），且原 O(1) 路径变成重建热点（单次成本放大一个量级）；伴随「多线程不加速甚至反降」（并行只放大重建并发，不摊薄重复访问）。本次实测：FlatCache rebuild 438,092 ≈ spline 调用数、Cache2D miss 458,281 次、spline 单次 992ns → 20,598ns、density 阶段 8.5-11.7ms → 670-1000ms/chunk——正是此指纹。
```

**替换为**：
```markdown
**指纹信号**：缓存重建/失效计数 ≈ 缓存访问总数（命中率≈0），且原 O(1) 路径变成重建热点（单次成本放大一个量级）；伴随「多线程不加速甚至反降」（并行只放大重建并发，不摊薄重复访问）。本次实测（2026-08-11，多线程 8/22 线程 thrashing 环境下计数器）：FlatCache rebuild 438,092 ≈ spline 调用数、Cache2D miss 458,281 次、spline 单次 992ns → 20,598ns、density 阶段 8.5-11.7ms → 670-1000ms/chunk——正是此指纹。

**08-11 vs 08-12 口径说明（judge 审查要点 4）**：两个测量口径不同，不构成矛盾——
- **08-11**：多线程（8/22 线程）thrashing 环境下的粗粒度计数器。每 chunk 跨线程迁移 → 单槽缓存全 miss + 多线程重建并发 → rebuild 计数 ≈ spline 调用数（命中率≈0 的表象）、spline 单次被 thrashing 放大到 20,598ns。
- **08-12**：单线程（-threads 1）WG_SPLINEDEBUG 精确统计，剥离 thrashing 后暴露真实主因结构：rebuild 36,252 次 = 每 chunk ~1007（期望 ~6）→ **168×**；rebuild 仅占 spline 调用（4,695,145 = 130,420/chunk）的 **0.77%**；spline 单次 t1 1,714ns（mt 27,155ns，16× thrashing）。
- **核心结论不变**：thread_local 单槽缓存 vs 跨线程执行模型失配。08-12 数据把放大链精确化为「rebuild 168× × 13.36 spline/miss」的级联（而非 08-11 表象的「rebuild ≈ 访问总数」）。
```

## 修正块 3：「叠加因素」段升级为主因机制 + H2 新指纹

**定位 marker**（L251，叠加因素整段）：
```markdown
叠加因素：FlatCache 网格构建含**嵌套采样递归**（边界点 x=cx*16+16 命中本 chunk 网格 k=4 才不重建，失配时触发相邻 chunk 网格重建递归）——缓存 miss 时单次重建成本被递归放大，进一步恶化。
```

**替换为**：
```markdown
**主因机制（2026-08-12 定论，H2 新指纹）**：FlatCache 网格构建含**嵌套采样递归**。buildGrid 角点 `i=4`/`j=4` 时 `p.x=(chunkX*4+4)*4=(chunkX+1)*16` 指向**下一 chunk 首列** → 嵌套 spline（continents/erosion/ridges 的 locationFunction FlatCache）收到**邻居 chunk key**（key=(x>>4,z>>4) chunk 级）→ 单槽缓存被污染 → 重建邻居网格 → 递归蔓延（实测 112 chunk = 36 生成 + 76 邻居，含左下对角 (44,-28)）→ rebuild 36,252 = 每 chunk ~1007 vs 期望 ~6（**168×**）→ spline 调用 20× 爆炸（130,420/chunk vs 旧 6,250）。

**H2 指纹信号**：缓存 key 由采样坐标派生（chunk 级），而采样点存在**越出当前上下文范围的角点**（buildGrid i=4/j=4）时，单槽缓存必然收到非本 chunk key → miss + 重建 → 递归蔓延。排查特征 = 重建计数的 chunk 覆盖**超出生成范围**（112 = 36+76 邻居）。修复方向 = per-chunk 多槽缓存（采样值逐位不变，不破坏 BK-001 对齐）或显式传入当前 chunk 键；**改循环顺序无效**（块级不触发 spline，H1 非主因）。
```

## 修正块 4：「证据」段追加 08-12 数据

**定位 marker**（L254，08-11 WG_PROFILE 证据行）：
```markdown
- WG_PROFILE（2026-08-11，density 阶段）：spline 单次 992ns → 20,598ns；spline.sample 338 万次；FlatCache rebuild 438,092 次 ≈ spline 调用数；Cache2D miss 458,281 次；density 阶段 670-1000ms/chunk（旧 8.5-11.7ms）
```

**替换为**（保留原行 + 追加 08-12 行）：
```markdown
- WG_PROFILE（2026-08-11，density 阶段，多线程 thrashing 环境）：spline 单次 992ns → 20,598ns；spline.sample 338 万次；FlatCache rebuild 438,092 次 ≈ spline 调用数；Cache2D miss 458,281 次；density 阶段 670-1000ms/chunk（旧 8.5-11.7ms）
- WG_PROFILE/WG_SPLINEDEBUG（2026-08-12，单线程 -threads 1 精确统计）：spline 4,695,145 次（130,420/chunk，旧 6,250 → **20×**）；FlatCache rebuild **36,252** 次 / 112 chunk（每 chunk ~1007，期望 ~6 → **168×**）；CACHE2D miss 351,536 次（4 个 cacheId，= 14,061 rebuild × 25 角点 ✓）；spline 单次 t1 **1,714ns** / mt **27,155ns**（**16×** thrashing）；放大链 = rebuild 168× × 13.36 spline/miss ✓
```

## 修正块 5：「如何利用」段追加 H2 越界角点指纹用法

**定位 marker**（L263-264，「如何利用」末两行）：
```markdown
- **git 二分定位引入点**：stash/checkout 旧提交对照（本次 8s 级退化用 stash 实验证明非本次引入，具体引入提交待二分）
- 跨版本/跨项目通用：任何「局部缓存 + 并行执行」组合都适用此检查
```

**替换为**：
```markdown
- **git 二分定位引入点**：stash/checkout 旧提交对照（本次 8s 级退化用 stash 实验证明非本次引入，具体引入提交待二分）
- **越界角点指纹（H2，2026-08-12 新增）**：缓存 key 由采样坐标派生（如 `(x>>4,z>>4)` chunk 级）且采样点可能越出当前上下文范围（buildGrid 角点 i=4 → 下一 chunk 首列）时，单槽缓存必然被邻居 key 污染 → 递归重建蔓延。排查 = 检查重建计数的 chunk 覆盖是否超出生成范围（本次 112 chunk = 36 生成 + 76 邻居实锤）；修复 = per-chunk 多槽缓存（采样值逐位不变，对齐零退化）或显式传入当前 chunk 键
- 跨版本/跨项目通用：任何「局部缓存 + 并行执行」组合都适用此检查
```
