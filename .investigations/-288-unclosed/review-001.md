# review-001：verdict-04.md 审查意见（-288 未闭合课题 · Phase 1 产物）

> 审查角色：core.judge（隔离 subprocess，只出意见，不改 status）
> 审查对象：
> - `.investigations/-288-unclosed/verdict-04.md`（裁决，draft→candidate 待审）
> - `.investigations/-288-unclosed/phase3-locating.md`（Phase 3 定位 + 第七节修正）
> - `.investigations/-288-unclosed/cmd-output/aqfdump_run1.txt` + `beard244_run1.txt`（原始 dump）
> 日期：2026-08-09
> 状态：**意见**（不修改任何产物 status；confirmed 由宿主用户授予）

---

## 一、逐项结论

| # | 审查项 | 结论 | 说明 |
|---|---|---|---|
| 1 | 证据完整性：fl.y=63 反射可信度 | ✅ **通过（附条件）** | AQF-DUMP 反射真实私有方法 getWaterLevel/calculateDensity/maxDistance（探针 DensityProbe.java L162-169），8 个 y 全测 fl2.y=fl3.y=fl4.y=63；e=0 前提被 Java 侧实测确认，与 04 篇 L108「e=0 两侧一致」前提衔接 |
| 2 | 证据完整性：BEARD-244 y 范围 | ✅ **通过** | 反射 `ChunkNoiseSampler.beardifying` 字段（BlockProbe.java L538-540）= 真实注入的 StructureWeightSampler 实例（非静态 `DensityFunctionTypes.Beardifier.INSTANCE`，后者 sample 恒 0 不参与块生成路径）；y=50..66 覆盖海底边界 y=52..62 并含翻转点 y=58 与转负点 y=63，范围足够 |
| 3 | 证据完整性：CellCache 等式 8/8 闭环 | ✅ **通过（附条件）** | 与原始 dump 逐行核对吻合（y=58: AQF-APPLY density=0.017663 vs C++ -0.074424+Beard +0.092090=0.017666，diff 3e-6；y=55/62 同理）；8/8 ≤3e-6 非巧合，充分证明「Java CellCache 输出 = finalDensity+Beardifier」⇒「C++ 缺 Beardifier 项 ⇒ 判 water」 |
| 4 | 逻辑一致性：四分支自洽 | ✅ **通过** | 分支 1（e≠0）否定、分支 2（stone 另有来源）否定、分支 3（density 输入差）成立、分支 4（ocean ruin 覆盖）推翻——与 NOISE-BLK 铁证（NOISE 阶段已 stone）、AQF-APPLY（Java 判 solid）、density.h L470 @anchor.idk 自认缺失、worldgen_api.cpp L570 注释一致 |
| 5 | 逻辑一致性：与既有结论冲突 | ⚠️ **标注差异源（缺失）** | **docs/03-density-functions.md L94「Beardifier.sample 恒 0.0（结构密度修正在 1.20.1 是空实现）——不是差异」与 10-timewise-archive.md L731「Beardifier.sample = 恒 0.0（源码 290-312 行）——非差异」是错误结论**：只看静态 INSTANCE 的 sample（恒 0），忽略 ChunkNoiseSampler.java L469-470 `getActualDensityFunction` 将 Beardifier.INSTANCE 替换为 this.beardifying（真实 StructureWeightSampler）。verdict-04 用 BEARD-244 实测 + CellCache 等式**事实上推翻了它，但未显式引用/标注/作废该旧结论**——知识库同时存在两条矛盾结论 |
| 6 | 范围判定 | ✅ **恰当** | Beardifier 属「结构」相关（StructureWeightSampler 由结构生成器注入，terrainAdaptation 机制），此前「结构暂缓」范围决策覆盖；判定「需用户拍板」正确（可闭合海底边界 6710 块，但依赖 C++ 无结构系统，实现成本高） |
| 7 | 产物契约：draft 状态标注 | ✅ 通过 | verdict-04.md L5「状态：draft（candidate 授予前 MUST judge）」 |
| 8 | 产物契约：被推翻结论排除清单 | ⚠️ **不完整** | 已列 04 篇 L108 归因、B1、B3 的修正；**缺 docs/03-density-functions.md L94 与 10-timewise-archive.md L731「Beardifier 恒 0」旧结论的作废声明** |
| 9 | 产物契约：原始数据落盘 | ✅ 通过 | cmd-output/aqfdump_run1.txt（50.9KB）+ beard244_run1.txt 已 git 跟踪 |
| 10 | 产物契约：index.yaml 更新 | ❌ **未满足** | `.artifacts/index.yaml`（100 行）登记了 phase1/2/3、b1/b2/b3、verify-splitter 等，**无 verdict-04.md 条目，无 cmd-output evidence 条目**（aqfdump_run1.txt/beard244_run1.txt） |
| 11 | 产物契约：retry 记录 | ❌ **缺失** | verdict-04.md 头部无 retry 字段（b1/b2/b3 均有 retry: 0）；本课题历史存在「绕 4 轮超 retry cap」记录（workflow-patterns L19），spec §5.3 要求产物声明 retry |
| 12 | 证据落盘补充：AQF-APPLY 高位垃圾值 | ⚠️ **未说明（问题复发）** | aqfdump_run1.txt 高位（y=319..256）density 固定 **-0.024995**、中位（y=255..106）固定 -0.458333——正是 04-aquifer.md L112 点名的「CellCache 反射垃圾值（如固定 -0.024995）」特征；而 04 篇 L112 铁律「CellCache 反射污染不可信（勿作密度参照）」与证据 3 用「AQF-APPLY dCC（CellCache.sample 反射值，DensityProbe.java L129-130）」作密度参照存在张力。y=55..62 的 dCC 因与 C++ finalDensity+Beard 独立吻合 8/8 可信，但产物未说明「为何低层可信、高层是垃圾值」（遍历内 cache 已填充 vs 遍历外）。workflow-patterns 发现 #1 补位 judge 已抓过「AQF 高位垃圾值未说明」，本次复发 |
| 13 | 噪声卡（Anchorlaw §3） | ✅ 未发现 | -288 目标未见未解决噪声卡文件 |
| 14 | 模块边界（spec §1.6 / §2.5 R5） | ✅ 通过 | 未引用其他领域模块 skill 正文 |

## 二、三源核对（spec §4）

- ① 交付快照：`.investigations/-288-unclosed/` 产物齐全（verdict/phase3/cmd-output/探针脚本均 git 跟踪）。
- ② git HEAD + 工作区：HEAD `1a91937 docs(-288): verdict-04...`，工作区无未提交变更；**但 index.yaml 未随 verdict-04 提交更新**。
- ③ regression/验证记录：AQF-DUMP/BEARD-244 输出与 `trace_aqf_1.txt`、`verify_splitter2.txt`（o/p/q 8/8 逐位一致）、NOISE-BLK 交叉一致 ✓；**差异源**：docs/03-density-functions.md L94 + 10-timewise-archive.md L731「Beardifier 恒 0」旧结论未标注（见上 #5）。

## 三、推荐状态

**建议 candidate（有条件）**。核心根因结论「C++ 缺失 Beardifier（StructureWeightSampler 结构密度修正）= -288 海底边界根因，非 aquifer bug、非 ocean_ruin 方块覆盖」证据链强（反射真实实例实测 + CellCache 8/8 闭环 + C++ 侧自认缺失 + NOISE-BLK 独立铁证），方向可信。但授予 candidate 前应补齐：

1. **作废旧结论**：在 verdict-04 §三 或独立条目显式声明 docs/03-density-functions.md L94、10-timewise-archive.md L731「Beardifier.sample 恒 0.0」为错误结论（忽略 ChunkNoiseSampler L469-470 替换注入），并更新 docs/03 相应行。
2. **说明 AQF-APPLY 取值口径**：解释 dCC 来自 CellCache.sample 反射（04 篇 L112 警告过）、为何 y=55..62 可信（遍历内 cache 已填充）而高位 -0.024995/-0.458333 是垃圾值；或将证据 3 的数据源标注为「遍历内 CellCache 反射 + 与 C++/BEARD 独立吻合」。
3. **更新 .artifacts/index.yaml**：登记 `re-code:-288-unclosed:verdict-04`（draft）与 cmd-output evidence 条目。
4. **补 retry 声明**：verdict-04 头部补 retry 字段（本裁决轮次），并说明与历史超 cap 记录的关系。

**confirmed 由用户拍板**（judge 不授予 confirmed）。Beardifier 实现立项也需用户确认（结构相关，范围决策）。

## 四、产物引用

- `.investigations/-288-unclosed/verdict-04.md`（L5 状态、L13-18 四分支、L44-55 CellCache 等式、L60-61 density.h/worldgen_api.cpp 引用、L69-72 范围判定）
- `.investigations/-288-unclosed/cmd-output/aqfdump_run1.txt`（L259-281 AQF-DUMP 8 点；L1-64 高位 -0.024995 垃圾值）
- `.investigations/-288-unclosed/cmd-output/beard244_run1.txt`（y=50..66 全列）
- `.investigations/-288-unclosed/phase3-locating.md`（第七节 L77-83 修正）
- `versions/1.20.1/docs/03-density-functions.md`（L94 旧结论「Beardifier 恒 0」——需作废）
- `versions/1.20.1/docs/10-timewise-archive.md`（L731/L861/L898 同款旧结论）
- `versions/1.20.1/docs/04-aquifer.md`（L108-109 被裁决原文 + L112 CellCache 反射污染铁律）
- `versions/1.20.1/data/mc_src_extract/.../ChunkNoiseSampler.java`（L177-181 注入、L469-470 替换）
- `versions/1.20.1/data/mc_src_extract/.../DensityFunctionTypes.java`（L290-312 静态 INSTANCE 恒 0）
- `versions/1.20.1/cpp/worldgen/src/density.h`（L470 @anchor.idk 自认缺失）
- `versions/1.20.1/cpp/worldgen/src/worldgen_api.cpp`（L570 CellCache 注释无 Beardifier 实现）
- `E:\PYTHON\MC\...\wg\bench\DensityProbe.java`（L99-138 AQF-APPLY、L142-225 AQF-DUMP）
- `E:\PYTHON\MC\...\wg\bench\BlockProbe.java`（L530-554 BEARD-244）
- `.artifacts/index.yaml`（100 行，缺 verdict-04 条目）
