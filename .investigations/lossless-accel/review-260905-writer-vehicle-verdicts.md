# judge 审查意见 — vehicle-verdict-260905 / writer-verdict-260905（260905 · 只出意见不改 status）

审查方法：三源核对（.artifacts 快照 / 源码抽查 / 探针与 fanout 证据文件）+ 代码逐点抽查。
抽查过的代码证据（本 judge 直接读源确认）：
- NoiseChunkGeneratorMixin.java：确认仅 populateNoise/buildSurface 两个 @HEAD cancellable 拦截（另含 nether 分支，verdict 未提，属范围小缺口）。
- WorldgenRust/src/ore_vein.rs：L31-32 块表（copper/iron）确认无 dirt/sand/gravel；L46 `y < -60 || y > 50 → -1` 硬门确认。b1 引证行号全部对得上。
- WorldgenRust/src/worldgen_handle.rs：L98-109 flags（bit0 carver/bit1 features/bit2 surface）+ L608-626 管线阶段（surface→carver→features，flag 或 env 任一命中即 skip）+ L793+ apply_features/OCEAN_FLOOR 注释，确认 b2 E5 属实。
- runtime/1.20.1/java/build.gradle：L27-47 确认硬编码拷贝 `WorldgenRust/target/release/WorldgenRust.dll → rename worldgen.dll`（Rust 主线）；L75/L110 cppReplace 属性确认。
- CppBridge.java（实际路径 wg/bench/CppBridge.java）：L63-79 resolveStageMask 默认 0b011 + 注释「Rust 双跑 = confirmed 缺陷」确认。⚠️ vehicle-verdict 写的证据路径含糊（未写 bench/ 子包），且 verdict 行号引 L64，实际注释在 L63-64，可用。
- .tmp/p2full/probe_ns_cells_260905.py：seed assert、16 chunk 解析、12 样本点、(195,199) 全列统计逻辑与 probe-round 所述一致。

---

## 一、逐项审查意见

### ① 置信度状态机合规 — 通过
- 两产物均自标 candidate / 待 judge，未发现任何 AI 越权 confirmed。CppBridge L64 注释中的「Rust 双跑 = confirmed 缺陷」指向既有已确认记录，非本次新授予，不违规。

### ② 证据链完整性 — 基本完整，两处缺口
- writer-verdict 三源（probe-round / b1 / b2）文件均存在，b1/b2 引用的文件:行号抽查全部吻合；260904-04 phase2 记录存在且与「残差族不同载具」叙述一致（22653/98.56% vs 13328/99.15%）。E7（10 篇 L2389：8137 条 [FEATURE] 行）为存档链路 Rust features 确实运行的运行时旁证——这是「双跑」最硬的一条证据。
- 缺口 A（写者 verdict）：**现役 dll（EC4A9AED）是否缺 wg_set_flags/修复从未运行时核实**（b2 自己列的 dumpbin 建议未执行）；stage-skip 重导消融也未跑。即「旧 dll features 全量运行 → 出界 dirt」这最后一环是静态推断+旁证，非定点实证。
- 缺口 B（写者 verdict）：probe-round 的 12 样本「全 stone」排除的是 **C++ block_probe** 的 NOISE/SURFACE，而 mod 载具 NOISE/SURFACE 实际执行体是 **Rust**——探针对真正的嫌疑执行体（Rust NOISE/SURFACE）没有数据层排除，该排除完全依赖 b1 的 Degraded 静态分析。结论链闭合，但 header「探针实测 + 双 worker 静态闭环」的表述让人误以为探针打在了嫌疑载具上，应显式注明「探针为跨载具排除，Rust NOISE/SURFACE 排除仅静态级」。

### ③ 载具可比性（§9.7）— writer 通过（有保留），vehicle 部分失效
- writer-verdict §9.7 口径行声明了 seed/覆盖面/ns 探针载体，且明确记录「探针载体与 mod 载具 dll 载体的可比性正是待裁决项」→ 合规。
- vehicle-verdict 未做 §9.7 三要素声明（其主体是静态读码，属可豁免边缘，但其 STEP-2 推论跨载具引用了 [ORIGIN] 审计并自行纠正，勉强闭环）。

### ④ 结论跳跃 — 两处，均集中在 writer-verdict
- 「双跑」是否被 260904-04 数据直接证实：**否，是最佳解释**。260904-04 直接证实的是「block_probe FULL ≠ mod 路径 feature 行为（残差族完全不同）」；「mod 世界导出时 Rust features 同时运行」的实证是 8137 条 [FEATURE] 日志（另一次运行的旁证，非本次残差数据集的定点归因）。writer-verdict 把裁决行写成事实句（「写者 = Rust FEATURES 阶段产物」）而把最佳解释性质藏进 b2 原文——应在 verdict 顶部显式标注「归因为最佳解释（静态闭环+旁证），定点运行时确证见遗留项」。
- 「Rust feature origin 分叉（OCEAN_FLOOR top-Y / height provider 与 Java 分叉）」：**零验证的机制假设**。无 Rust↔Java origin 对拍，无任何 Rust feature 写出界 dirt 的定点证据。verdict 将其放在「机制解释（candidate）」小节算已部分对冲，但它与裁决行捆绑呈现，读者易当成已证机制。建议降格标注为「未验证机制假设（working hypothesis）」，与遗留项第 2 条挂钩。

### ⑤ §15.4 取代链 — 冲突，需补 supersedes 记录
- writer-verdict 实际取代了 vehicle-verdict 的两块内容，但**双方均无 supersedes 双指针**：
  1. vehicle-verdict STEP-2 推论「写者候选收窄为 C++ 侧（NOISE vein/SURFACE）或 Java feature 谓词差」——已被推翻（真写者 = Rust FEATURES，两个候选都不是）。且该推论的前提「mod 载具 FEATURE 由 Java vanilla 执行」对**被分析的残差数据集不成立**（旧 dll 双跑）。
  2. vehicle-verdict 的载具描述「C++ 执行 NOISE+SURFACE」——实际执行体是 Rust WorldgenRust.dll（build.gradle 实锤，b2 已证）。verdict 全文用「C++ 地形/C++ 执行」属CppBridge 时代的口径残留，与 writer-verdict 的「Rust」表述直接冲突。
- 处理方式须遵守 §15.4：vehicle-verdict 原文不改写，补取代记录（一行推翻理由 + 双指针）；或最低限度在 vehicle-verdict 的 .artifacts 副本头部加勘误注记并登记 index（注记不等于改写裁决正文，需用户认可形式）。

### ⑥ 产物契约（core.artifact）— 两处违规
- `.artifacts/lossless-accel/index.yaml` **无任何 260905 条目**——vehicle-verdict 的 .artifacts 副本未登记。
- writer-verdict **只存在于 .investigations/，无 .artifacts 正本**（对照任务说明，只有 vehicle-verdict 双写）。结论级产物按契约应落 .artifacts + index。

### ⑦ 其他小项（INFO 级）
- probe_ns_cells_260905.py 中 `vb==9`/`vc==9` 硬编码 dirt id=9，未与 blocks.json 交叉校验该 id 映射——低风险但违反项目「数据对比三查」精神，建议脚本输出 id→name 校验行。
- 该脚本中 mod 世界导出文件名为 `vanilla_8576294172403134396_4_200_200.blocks`（REF 在 ref/ 子目录，mod 导出在 base 同名）——命名极易混淆，属「参照文件完整性核对」隐患点；本次 seed assert 通过故判有效，但建议重命名。
- vehicle-verdict 称「全 src 无 FEATURE 阶段 hook」：本 judge 确认了主 mixin 文件仅两拦截，但未穷举全部 mixin 文件（信任 verdict 的 src 扫描声明，标注为未独立复核）。
- 两产物日期标签 260905：与任务委托一致，未见漂移信号（未独立核 git 时间戳）。

---

## 二、判定

### candidate 1：vehicle-verdict-260905 — **可保留 candidate，需补正（非重做）**
理由：核心载具结构结论（populateNoise/buildSurface 拦截、无 FEATURE hook、混合载具、两载具残差族禁止互引）经代码抽查成立，且有持久价值。但必须补三件事：① 按 §15.4 补 supersedes/勘误记录，覆盖「STEP-2 写者候选收窄」推论与「C++ 执行」口径（被 writer-verdict 推翻/修正）；② 载体执行体表述补注「实际 dll = Rust WorldgenRust（build.gradle L27-47）」；③ .artifacts index.yaml 登记补齐。其「L64 注释自证」作为证据用于「被分析数据集」时失效（旧 dll 双跑）——该证据只支撑**当前/修复后代码**的载具设计，不影响载具结构结论本身。

### candidate 2：writer-verdict-260905 — **可保留 candidate，需补充证据后才能升级**
理由：排除链（C++ ns 探针排除 + b1 Rust NOISE/SURFACE 静态排除 + b2 dll=Rust 静态实锤 + 8137 [FEATURE] 旁证）多源交叉，无硬伤；降级声明（b1/b2 Degraded）诚实且准确，遗留项已自列。但：① 「双跑」与「Rust FEATURES 为写者」目前是**静态闭环+旁证支撑的最佳解释**，缺两块声明中的运行时确证——现役 dll 导出表核查（wg_set_flags 有无）+ stage-skip dll 重导消融（stone→dirt 族应消失）；② 「Rust feature origin 分叉」机制假设零验证，须降格标注；③ 顶部「探针实测」表述应注明为跨载具排除；④ 补 .artifacts 正本 + index 登记；⑤ 机制假设的 Rust↔Java origin 对拍列独立待查（verdict 已列，维持）。
在 ①两块运行时确证完成前，**不得升 strong candidate / 不得提交 confirmed**；两块完成且残差按预测消失，则可直接推 strong candidate。

### 附带建议
- writer-verdict 的「原 aquifer 残差课题在 mod 载具上结案重定向」属范围决策（judge MUST 触发点，本次审查即履行）；建议该结案亦走 §15.4 取代记录挂到 residual-signature-verdict-260904-04，避免台账双头。
- 若两项补充证据到位，建议主会话把「混合载具 + 双跑伪影 + 残差课题重定向」合并为一条取代记录进 10 时间线（subagent 产出草稿，主会话应用）。

（本意见只出审查结论，不改任何产物 status；confirmed 留给人类。）
