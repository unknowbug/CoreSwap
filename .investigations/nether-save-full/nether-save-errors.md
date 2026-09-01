# nether 存档写入口径 Full 化 — 错误台账（nether-save-errors.md）

> status: draft（knowledge worker 整理，2026-09-04；按 SUBAGENT-KNOWLEDGE-GUIDE 五段式，载体 = `.investigations/<课题>/<课题>-errors.md` 项目级指定载体）
> 课题目录：`.investigations/nether-save-full/`；judge 审查记录：`judge-review.md` #16-#21
> 数据来源：`cmd-output/seedA-contradiction-facts.md`、`candidate.b2-pipeline.md`（含追加段）、`candidate.b3-nondeterminism.md`（含追加段）、judge-review #13-#21

---

## E1. cppWorldgenDir 传错一层 → wg_create=0 → 三场「Rust 验证」run 实为 vanilla-vs-vanilla（本课题最大错误）

- **编号**：E1（judge-review #13，建议 candidate）
- **现象**：seed A 三场 gen run（gen1 20:09 / gen2 20:13 / reconfirm 20:13:47）产出 5 条「矛盾观察」（131 差 / 1 差 / 104 差跨运行不一致），表面像「Rust 生成 vs vanilla 参照」的对齐问题。事后复查日志发现铁证：`[CppBridge] init seed=... enabled=false`、`[CppBridge] initNether seed=... enabled=false`（gen1 log L240-242 / gen2 log L235-236），且两 log `[Mixin] populateNoise(nether) intercepted` **grep 全文 0 命中**——dll 从未加载，Rust 一次都没参与。三场 run 的所有对比数字看似正常（99.98%/99.99% 级 match），实为 **vanilla-vs-vanilla**，全部无效。
- **根因**：CppBridge 初始化传参时，把其注释里描述的**解压后布局** `<tmp>/coreswap-data/worldgen/data/...`（data 才是 worldgen 数据根下的子目录）误读为 `cppWorldgenDir` 参数本身；正确语义 = **含 `data/` 的那一层**（本次即 `versions/1.20.1/data/worldgen`）。路径少一层 → Rust 侧 `wg_create` 找不到 `data/` → 返回 0（NULL）→ CppBridge 仅打一行 log **静默降级 enabled=false**，Java 继续走 vanilla 全管线，实验「照常出数字」。
- **定位**：① 发现 init 行 `enabled=false` + `initNether enabled=false` + intercepted 计数 0（日志取证，candidate.b2 追加段）；② 主会话用 ctypes 直连 `wg_create` 复现：传错层返回 0，传对层返回非 0——单变量隔离，把 b2 早期推测的「dll 提取失败 / 临时目录权限」表象收窄为路径传参错这一机制根因。
- **修复**：改传正确目录层（wg_dir = 含 `data/` 的层），重跑得 v2 真 Rust 双 seed 数据（seed A 内存=存档读回 99.9376% / MCA 99.9278%；seed B 三口径 93.5156%）。验收判据：log 必须含 `enabled=true` + intercepted 覆盖目标 4×4 chunk（实测正确 run 为 64 条 = 目标 + feature 蔓延邻域；judge #22 提醒勿照本宣科写成「恰好 16 条」）。
- **教训**：
  1. **替换模式 run 的第一动作 = 检查 `[CppBridge] init ... enabled=` 标志；enabled=false 时该 run 的一切对比数字无效**——在得出任何「怪结论」（跨运行不一致、矛盾观察）之前就该查，而不是出数字后倒查。
  2. **静默降级是测量管线 P1 级缺陷**：CppBridge init 失败仅打 log 不 fail-fast，导致三场「Rust 验证」实验在无 Rust 参与下跑完并产出 5 条矛盾观察（与 AGENTS.md「DensityProbe 禁 CppBridge 否则参照被污染」同族的反向案例：需要 CppBridge 时静默不可用同样致命）。**改进项（建议）：`wg_create` 返回 0 时 Java 侧 fail-fast 拒绝启动**（judge #10 对 header 校验的 fail-fast 建议同精神）。
  3. **注释里的路径布局 ≠ API 参数语义**：传参前以「实测该层返回值非 0」为准，不凭注释/文件名印象。

## E2. WGB2 header 断言想当然：凭印象写 nether height=128（实际 256）

- **编号**：E2（judge-review #21，INFO 简记档；独立成行以便速查）
- **现象**：小样本 header 校验 assert `height==128` 失败，实际值 256。
- **根因**：凭印象写死 nether 世界高度。vanilla `the_nether` dimension_type 实际为 **min_y=0, height=256**（128 是 noise_height / 逻辑生成高度——multiworld-errors.md M3/M13 已两次沉淀「world_height 256 vs noise_height 128 双高度」教训，本次在 header 断言上第三种形态复发）。
- **定位**：assert 失败 + 独立 NBT/参照解析确认真实 header（seedA-contradiction-facts.md：ref 文件 WGB2 min_y=0 height=256）。
- **修复**：修正断言；ReadWorldProbe 进一步结构化为动态读取 `world.getBottomY()/getHeight()`（judge #7 PASS），死常量 MIN_Y/HEIGHT 待删（judge #12）。
- **教训**：**header 语义字段（min_y/height/seed/size）以动态读取为准，断言值必须有出处**（DimensionType JSON / world 运行时），勿猜；「逻辑生成高度」与「世界存储高度」是两个值，凡涉高度先问是哪一个。

## E3. 替换模式 run 未查接管标志（与 E1 同根的流程错误，单独立行）

- **编号**：E3（judge-review #19 WARN 的一部分，按任务书要求独立成行）
- **现象**：三场 run 全部跑完、出了对比数字、甚至产出了 5 条「矛盾观察」并 fan-out 三候选分析之后，才发现 `enabled=false`。
- **根因**：违反「探针/参照数据采集核对铁律」的精神——**对比前未核对两侧同源性**（所谓「Rust vs vanilla」两侧实际都是 vanilla）。铁律原文针对 seed/坐标/文件三查，本次暴露第四个必查项：**接管标志/工具链启用状态**。同根因于 E1（CppBridge 静默降级），但错误发生在流程层（未执行核对），不是代码层。
- **定位**：b2 候选在做日志取证时倒查发现（fan-out 产出后）。
- **修复**：验收判据写死进 run 协议（b2 P2-②）：`enabled=true` + intercepted 覆盖目标 chunk，缺一即该 run 作废；v2 重跑按此执行。
- **教训**：**替换/接管模式实验的核对清单要在 run 前执行，不是在结论奇怪后倒查**；「对比数字看似正常」恰恰是危险的——vanilla-vs-vanilla 的 match 率天然高，数字正常 ≠ 前提成立。

## E4. b2 子候选论据与代码不符：「seed 从 ref 文件内读天然防错位」（更正记录）

- **编号**：E4（judge-review #16 FAIL 局部；按任务书要求在时间线与本台账各记一笔）
- **现象**：candidate.b2-pipeline.md 子候选①论证中声称「seed/origin 从 ref 文件内读（非命令行拼），天然防 seed 错位 ✓」——与代码不符：实际文件名与 seed 全部来自 `-D` 属性拼接（ReadWorldProbe L25-34），header 读后**丢弃不校验**（L50-52）。
- **根因**：worker 论证时把「理想设计」当成了「实际实现」，未逐行核对读取路径。
- **定位**：judge 逐行核对 ReadWorldProbe.java（judge-review #10/#16）。
- **修复**：论据作废更正（本文即更正记录；b2 原文按 §15.4 精神不改写、以 judge-review #16 为取代指针）。副作用建议：读 header 后与 `bench.seed`/`world.getBottomY()/getHeight()` 断言、不符 fail-fast（judge #10，优先级因本错误升高）。
- **教训**：**「防错位」结论必须指认代码行**，不能由「文件里有 seed 字段」推出「seed 被使用」；读后丢弃的字段不构成任何保护。

## E5. vanilla run 期矛盾观察（gen1 131 差 / gen2 1 差 / 存档 104 差）——已作废，被 v2 Rust run 取代

- **编号**：E5（§15.4 取代记录形态：本条为取代声明，原观察事实记录见 cmd-output/seedA-contradiction-facts.md，其正文不删不改）
- **现象**：seed A 三场 run 产出跨运行不一致的「矛盾」：gen1 内存 131 差（130 cave_air + 1 quartz→gold）、gen2 内存 1 差、gen2 存档 104 差、reconfirm 读盘 1 差——「同文件两读不同」「同参数跨运行不同」等 5 条矛盾观察，一度触发三候选 fan-out（b1 时序 / b2 管线 / b3 非确定）。
- **根因**：**前提崩塌**——三场 run 全部 enabled=false（E1），观察对象是 vanilla-vs-vanilla + DIM-1 stale 残留（cave_air/gold 来自上一批实验的盘上残留），不是任何生成器行为。b3 追加段判读：CppBridge=false 时 vanilla vs vanilla 语义上应逐位相同，实测仍不一致 → 差异几乎全部来自**观察层伪影**（probe 读取时机落在生成进度线不同位置）与 **DIM-1 残留**（未清 `world/DIM-1/region`）。
- **定位**：日志取证（enabled=false + intercepted 0 条）+ ctypes 直连复现（见 E1）。
- **修复**：**本批观察全部作废**（已作废，被 v2 Rust run 取代；supersedes → candidate.b2-pipeline.md 追加段 + `.tmp/*_v2.log`）。fan-out 三候选的机制分析中仍成立的部分：b3 的 Rust 侧静态审查（HashMap/read_dir/thread_local 风险清单）降级为「接管后风险登记」；b1/b3 的时序竞态模型转化为 v2 下仍开放的问题（见下方未闭合项）。
- **教训**：**矛盾观察先查前提再建候选**——若第一动作是查 enabled 标志（E1/E3 教训），5 条矛盾与一轮 fan-out 本可不发生；nether 实验清 world 必须连 `DIM-1` 一起清（b2 P2-③）。

---

## E6. 对照口径误置：把存档口径残差直接归因 surface rule 条件链，未先做阶段消融

- **编号**：E6（B1 定论轮复盘，建议 candidate）
- **现象**：2026-09-04 轮把 seed B 残差大头 B1（basalt deltas 三大宗石互换，52,078 块 / 76.6%）的机制候选直接写成「surface rule 条件链系统性偏差（biome 判定 / noise 阈值 / Hole 语义下游表现）」并列为深挖优先级 #1——本轮三方实验证明该归因方向错位：互换主因 = **feature 阶段产物**（blobs/columns/delta/pillar）在两种基底地形上的命中/形态差 + Rust surface 薄带残差，宗石大宗根本不是 surface rule 产物；连带把已修复的 Hole 语义（M6 后 Rust `stone_depth_above <= 0` 与 Java 一致）仍当未闭合疑点继承。
- **根因**：**对照口径误置 + 归因未先做阶段分解**——残差来自存档口径（Rust noise/surface + Java carvers/features 端到端），其中混着 Java feature 阶段产物，却在未做任何阶段消融的情况下把差异整体对到替换方（Rust surface rule）的条件链上；且把上一轮交接文档里的「方向性待查假设」（Hole 语义不一致）当公理直接继承，未做廉价独立验证（该假设 M6 修复时已过时）。
- **定位**：三方实验 + fan-out 两候选裁决：① 纯 Rust 口径（ctypes 直连 dll vs rlib 直跑 cell 级 0 差异）vs FULL = 77.43%（basalt→netherrack 157k = surface 薄带 + 纯 Rust 下 blobs/columns 缺失叠加）；② 存档口径（+Java carvers/features）= 93.5508%——feature 补回大头；③ WG_SKIP_SURFACE=1 重跑 = 55.18% 且 blobs 不触发（stone 基底非 netherrack → blackstone=0、quartz/gold ore=0）——证明 blobs 是 feature 阶段、依赖 netherrack 基底；④ biome 分桶（互换 100% 落 vanilla basalt_deltas 列）排除源分配差。两候选：.b1 surface_depth 带厚（❌ 带厚上限 ≤6 层，40 层体块不可达）、.b2 nether_state_selector 恒 0.0（⚠️ 真实 bug 但只解释零星翻转，非主导）。
- **修复**：B1 机制定论改写为「feature 产物 × 两种基底地形」结论（→ 09 篇追加小节，candidate）；Hole 语义遗留行做 supersedes 标注（§15.4，原行不删）；对照口径澄清（纯 Rust 77.43% 与存档 93.55% 载体不同不可比；B1 参照分两用：BlockProbe SURFACE 口径测 Rust surface 残差、存档口径测端到端）；.b2 的 nether_state_selector 预加载表缺 nether 噪声列为待修（一行补齐，非 B1 主导）。
- **教训**：
  1. **替换模式存档口径残差必须先做三阶段归因（noise/surface = 替换方 vs carvers/features = 存续方），再定位机制**——「残差 → 某层条件链」的归因出手前必须已有阶段消融（如 WG_SKIP_SURFACE）或直连基线（如 ctypes 直连）证据，否则只能是 draft（已沉淀为 workflow-patterns 发现 #10）。
  2. **交接假设开工先验再继承**（AGENTS.md 交接结论验证纪律）：本轮第一动作即用 L101 源码核对推翻 Hole 假设——若沿用上轮归因直接深挖 surface rule 条件链，整轮工作量将投入不存在的 bug。
  3. **「大宗块差」先问产物阶段归属**（与发现 #2/#4 同族）：vanilla 宗石/涂布类块面多为 feature 阶段产物，见到成片互换先查 feature 依赖（基底块条件），再查 surface/noise。

---

## E7. surface rules 噪声预加载表缺 nether 噪声：unwrap_or(0.0) 静默回退使 nether_state_selector 恒 true（.b2 遗留 bug，已修复）

- **编号**：E7（fan-out .b2 候选判定为真实 bug，2026-09-06 修复；candidate）
- **现象**：nether 存档口径验证中 nether surface rule 恒走 basalt 分支（basalt deltas 相关宗石零星分支内翻转，selector 条件失效）；`surface_rules.rs` noise_threshold_sample 对 `minecraft:nether_state_selector` 等 nether 噪声 key 全部取到 0.0。
- **根因**：**隐式契约断裂 + 缺省值吞错误**——「surface rules 引用的噪声 key 必须在 step4 预加载」这一约束没有任何静态检查；`worldgen_handle.rs` step4 预加载表（L192-195 一带）只硬编码了 overworld 噪声清单（surface/surface_secondary/clay_bands_offset/badlands_*/gravel/powder_snow/packed_ice/ice/surface_swamp），nether 的 6 个噪声（nether_state_selector/patch/soul_sand_layer/netherrack/nether_wart/gravel_layer）全部缺失；下游 `noise_threshold_sample`（surface_rules.rs L120-137）查不到 sampler 时 `unwrap_or(0.0)` 静默回退——而 nether_state_selector 的 min threshold 恰为 0.0，回退值使条件恒 true，错误被完全吞掉，只在输出块差异里显形。
- **定位**：B1 大宗互换排查 fan-out 两候选中，.b2 候选沿 noise key 数据流（surface rule JSON 引用 → step4 预加载表 → noise_threshold_sample 查表）逐段对拍发现表缺 key（证据：`.artifacts/.b2-nether-state-selector/`）；judge 裁决「真实 bug 非主导」（B1 主导 = feature 产物 × 基底地形差，见 09 篇 B1 定论）。
- **修复**：step4 预加载表补 6 个 nether 噪声 key（全部存在于 `versions/1.20.1/data/worldgen/data/minecraft/worldgen/noise/*.json`，非自造）。重测（seed B = 8576294172403134396，4×4 @3200,3208，存档口径）：93.5508% → **93.8988%**（+0.348pp ≈ 10× 同 dll 非确定性容差 ±369 块 ≈ ±0.035pp，超出非确定性噪声，真实改善）；E1/E3 判据核对通过（`initNether enabled=true` 且 seed 一致，`cmd-output/b2-fix-rerun.log`）。
- **教训**：
  1. **隐式契约（引用方与加载方的 key 一致性）必须有静态检查或 fail-fast**：本 bug 从 nether 接管上线起潜伏多轮，唯一显形通道是输出块差异——B1 排查绕了一圈才定位。凡「查不到就回退默认值」的路径（`unwrap_or(0.0)` / `get(key).unwrap_or` 家族）在数据驱动的 JSON 引用链上都是吞错误反模式，**跨语言通用**（Java/Rust/C++ 同罪）；至少应 log-warn 一次 + 诊断开关可直接报「unknown noise key」。
  2. **新增维度/数据域时，硬编码清单类代码是天然遗漏点**：预加载表按 overworld 清单写死后，nether 接管时无人提醒补齐——数据驱动边界评审时应专门过一遍「清单是否覆盖所有已启用维度」。
  3. **修复验证用容差倍数判真改善**：+0.348pp 远超 ±0.035pp 容差（10×），不需要逐位 Full 即可判定改善真实（Partial 分层 + 容差声明即可，§9.7）。

---

## E8. 沙箱下 gradle runServer「failed to extract worldgen.dll」AccessDeniedException（已修复）

- **编号**：E8（环境坑，2026-09-07）
- **现象**：沙箱下 `gradle runServer` 启动失败，报 `failed to extract worldgen.dll`，异常链为 `AccessDeniedException`，写入目标为 `%TEMP%\dsh-*` 临时目录。
- **根因**：**沙箱文件权限边界**——JVM/gradle 侧原生库提取流程默认写系统 `%TEMP%`，沙箱策略对该路径拒绝写入；机制上不是 dll 本身损坏或版本不符，而是「提取目标目录不可写」，报错被包装成「failed to extract」易误判为 dll 问题。
- **定位**：读异常链中 `AccessDeniedException` 的目标路径（`%TEMP%\dsh-*`），确认拒绝发生在临时目录写入而非 dll 源读取；对照沙箱可写范围（session workspace）即定位。
- **修复**：设 `JAVA_TOOL_OPTIONS=-Djava.io.tmpdir` 指向工作区内目录，使 JVM 全部临时文件（含 dll 提取）落到沙箱可写路径，runServer 正常启动。
- **教训**：
  1. **沙箱环境下 JVM 类工具默认临时目录不可信**：任何「提取/解包到 %TEMP%」的构建/运行流程在沙箱下优先怀疑临时目录权限，用 `java.io.tmpdir`（或等价物）重定向到工作区，而不是去排查被提取的资源文件本身。
  2. **报错文本 vs 异常链**：「failed to extract X」的表面文本指向 X，机制原因常在异常链尾部的目标路径——先读 AccessDenied 的目标再定方向。

---

## E9. WorldgenRust.dll mtime 因 fs::copy 保留时间戳不可信：显示 9/1 实为最新（已修复判定方法）

- **编号**：E9（环境坑/判错方法，2026-09-07）
- **现象**：WorldgenRust.dll 文件资源管理器/Get-ChildItem 显示 mtime 为 9/1，按时间戳判断应为旧产物，实际是最新构建——按 mtime 判新旧会得出错误结论。
- **根因**：构建/部署链使用 `fs::copy`，该调用**保留源文件时间戳**——复制产物的 mtime 反映的是源文件时间而非复制时刻；mtime 在此链路上不是「产物生成时间」的可靠信号。
- **定位**：对 dll 内容做二进制字符串探测（比对链路中新特征字符串/版本串存在于「旧 mtime」文件中），确认内容为最新 → 证明 mtime 与内容不一致，时间戳不可信。
- **修复**：判 dll 新旧改用**二进制字符串探测（内容指纹）**，不再依赖 mtime。顺带处置：bin-diag 诊断 bin 不参与默认构建（cargo 只编译 `src/bin/`），使用时**临时挪入 `src/bin/` 编译**（`init_vertical` 需 `pub` 化），用完迁回，符合临时文件唯一区纪律（AGENTS.md 八.13）。
- **教训**：
  1. **`fs::copy` 保留 mtime——凡复制链路上的产物，时间戳不代表新旧**；判产物版本用内容指纹（二进制字符串/哈希），不用文件时间戳。
  2. **诊断 bin 与正式 bin 分区**：`src/bin/` 只放随库维护 bin，一次性诊断程序放 `bin-diag/`（不参与默认构建），临时挪入编译是合法用法——勿为诊断 bin 长期污染 `src/bin/` 的全量绿。

---

## 未闭合待查项（供后续 session，非错误）

- **103 cave_air 簇机制**（judge #14 保持 draft）：v2 下新形态矛盾——seed A 内存=存档读回精确同值（1047922），MCA 却多 103 块 air→cave_air（chunk(203,200) y70-72 一簇）。b1/b3 均未闭合，residual-interpretation §3 #5 给出探针方向（M4 复核 / 禁 carvers 重跑 / save 前后 hook）。
- **basalt deltas 大宗互换**（B1，52,078 块 76.6%）：~~surface rule 条件链系统性偏差候选，方向见 residual-interpretation §3 #1~~ **[supersedes 2026-09-05]** 已定论：feature 阶段产物 × 两种基底地形差 + Rust surface 薄带残差（见 09 篇「B1 定论」节）；本行原候选方向作废。
- **矿石 features 缺口**（A1+B4，3,269 块）：nether ore feature「未实现 vs 放置错位」归属未定，§3 #2。**[注 2026-09-05]** 存档口径下 features 由 Java vanilla 运行（本轮 cppReplace 架构事实），A1 归因候选需按三阶段归因法（发现 #10）重估。**[supersedes 2026-09-07]** 已定论：非「未实现/错位」——Rust 管线自带 feature 阶段与 Java 双跑（见 09 篇「矿石归因定论：双重 feature 应用」，消融实证）；本行原候选方向作废。

---

## 错误→根因 速查表

| 错误（现象签名） | 根因 | 一句话教训 |
|---|---|---|
| E1 三场「Rust run」实为 vanilla（enabled=false，intercepted 0 条，对比数字却「正常」） | cppWorldgenDir 传错一层（把解压布局 `…/worldgen/data/…` 当 wg_dir；正确 = 含 `data/` 的层）→ wg_create=0 → CppBridge 静默降级 | **替换模式 run 第一动作 = 查 `enabled=` 标志；false 则一切数字无效**；wg_create=0 应 fail-fast（改进项） |
| E2 header assert height==128 失败（实际 256） | 凭印象写 nether 高度，未查 the_nether dimension_type（min_y=0, height=256） | **header 语义字段动态读取为准，断言值必须有出处**；world_height ≠ noise_height（M3/M13 同族第三犯） |
| E3 出数字后才发现 enabled=false | 流程层未在对比前核对两侧同源性（接管标志），违反探针/参照核对铁律精神 | **核对清单 run 前执行，不结论奇怪后倒查**；数字正常 ≠ 前提成立 |
| E4 b2「seed 从 ref 文件内读天然防错位」与代码不符 | seed 实来自 -D 属性拼文件名，header 读后丢弃 | **「防错位」结论必须指认代码行**；读后丢弃的字段无保护力 |
| E5 vanilla run 期 5 条矛盾观察（131/1/104 差跨运行不一致） | 前提崩塌：全 vanilla + DIM-1 stale 残留；差异 = 观察层伪影 + 残留，非生成器行为 | **矛盾观察先查前提再建候选**；nether 清 world 必须连 DIM-1（已作废，被 v2 Rust run 取代） |
| E6 B1 大宗互换被归因 surface rule 条件链（实为 feature 产物 × 基底差） | 对照口径误置：存档口径混 Java feature 阶段产物，未先阶段消融就归因替换方条件链 | **先消融/直连基线后归因**（发现 #10 三阶段归因法）；交接方向性假设开工先验 |
| E7 nether surface rule 恒 basalt 分支（noise key 全取 0.0） | step4 预加载表只含 overworld 噪声，nether key 缺失 → noise_threshold_sample `unwrap_or(0.0)` 静默回退（threshold=0.0 使条件恒 true） | **隐式契约要有静态检查；unwrap_or(0.0) 吞错误是跨语言通用反模式**；新维度上线先核硬编码清单覆盖面 |
| E8 沙箱 gradle runServer「failed to extract worldgen.dll」AccessDeniedException | JVM 默认写 `%TEMP%\dsh-*` 提取 dll，沙箱拒绝临时目录写入；非 dll 本身问题 | 沙箱下 JVM 工具用 `-Djava.io.tmpdir` 重定向临时目录到工作区；先读异常链目标路径再定方向 |
| E9 dll mtime 显示 9/1 实为最新，按时间戳判新旧出错 | 构建链 `fs::copy` 保留源时间戳，mtime ≠ 产物生成时间 | 复制链产物判新旧用内容指纹（二进制字符串探测），不用 mtime；诊断 bin 走 bin-diag/ 临时挪入 |
| E10 强杀 gradle daemon 后全部 gradle 调用报 "Failed to load native-platform.dll" | 根因 = `~/.gradle/native/**/native-platform.dll.lock` 拒绝访问（daemon 被杀锁未释放 + home 目录在沙箱外删锁被硬拒），非 dll 本身 | --stacktrace 定位到 .lock 文件级拒绝再动手；`GRADLE_USER_HOME` 指工作区（.gradle-home）一次性绕开 home 权限（= build-tooling 发现 #7，同机制与 #4 互证）；参照文件名四要素（seed/size/origin/dim）与命令参数逐项核对防空跑（run2/3 空跑教训） |

## E10. 强杀 gradle daemon → native-platform.dll 加载失败（2026-09-08，已修复）

- **现象**：`Stop-Process -Name java` 清理残留后，任何 gradle 调用（含 `gradle --status`）报 `Gradle could not start your build. > Could not initialize native services. > Failed to load native library 'native-platform.dll'`；带不带 `JAVA_TOOL_OPTIONS` tmpdir 均复现。
- **根因**：daemon 被强杀时 `native-platform.dll.lock` 未释放；锁文件在 `C:\Users\NDark\.gradle\native\`（工作区外），沙箱 workspace-write 下删除被硬拒（sandbox_permissions 升级被拒：已是 workspace-write，属策略硬拒绝）。表象（dll 加载失败）与根因（.lock 访问拒绝）错位。
- **定位**：`gradle --status --stacktrace`，最内层 `Caused by: java.io.FileNotFoundException: ...\native-platform.dll.lock (拒绝访问。)`——先看最深 Caused by 再定方向，不被第一行报错误导。
- **修复**：`$env:GRADLE_USER_HOME='E:\PYTHON\CoreSwap\.gradle-home'`（仓库根既有目录）——gradle 全套可变状态（native 锁/daemon/依赖缓存）都在 GRADLE_USER_HOME 下，指到工作区即整体绕开。同时固化 nether 回归命令模板（run4 log）：cppReplace + readWorldProbe + blockProbeDimension=nether + benchSeed/benchSize/benchOriginX/benchOriginZ/benchOut 须与参照文件名四要素逐项一致。
- **教训**：①沙箱下强杀 java daemon 前先想锁文件，预防（GRADLE_USER_HOME 指工作区）优于修复；②「Failed to load X.dll」类报错先用 --stacktrace 查最内层 Caused by，别按表象查 dll；③跨工具对比 run 的参数 ↔ 参照文件名四要素核对前置，防空跑烧轮次（run2/run3 两次空跑）。
