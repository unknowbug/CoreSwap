# review-judge-20260907 —— C2 P2/P3 归因结论 + 重大转向 + 修复方向审查

- 审查者：core.judge（subagent，只出意见不改 status；confirmed 留人类）
- 审查对象：`.artifacts/.c2-p2-ore-attribution/verdict-ore-attribution.md`（v2，H_A→H_B' 翻转）、`.artifacts/.c2-p3-soul-valley/verdict-soul-valley-family-draft.md`（V1 收敛稿）、修复方向建议（存档链路默认关 Rust features/carver）
- 审查基线：git HEAD=709b006（**注：judge 沙箱无 shell，未跑 git diff；以文件直读为准，主会话应自行确认工作区无未提交漂移**）
- judge 三查：① artifacts 快照（两 verdict + 本审查）② 代码实证直读（worldgen_handle.rs L361-452 / NoiseChunkGeneratorMixin.java 全文 / CppBridge.java grep）③ 验证记录（V1/消融数据由主会话提供，原始输出落盘情况见 WARN-1）

---

## 1. 证据引用一致性核对 —— PASS

- P2 verdict 表格数字（quartz 1992/2381/4478、gold 728/793/1525、magma 1533/2073/3814、basalt 172704/5514）与主会话提供的 V1/消融数据完全一致。
- 代码引用复核（直读）：
  - `worldgen_handle.rs` `fill_chunk_blocks` 第 5 步 `apply_features` 实际位于 **L442-449**（L437-440 是 carver 步）——prompt 写 L437-449 是「carver+feature 阶段」整体，verdict 写 :442-449 / :443 更精确，两处引用与源码一致。WG_SKIP_CARVER(:438)/WG_SKIP_FEATURES(:443)/WG_FEATURELOG(:446) 门控均实证存在。
  - `NoiseChunkGeneratorMixin.java`：populateNoise 对 overworld（L62-69）与 nether（L72-77）均整块拦截（调 Rust fillChunk/fillChunkNether），buildSurface 对两者 cancel（L94-97）；Java CARVER/FEATURES ChunkStatus 步骤未被拦截 → 「存档 = Rust 全管线 fillBlocks + Java carvers/features 照跑」机制链成立。
  - `CppBridge.java` grep：**无任何 WG_SKIP_* / env 设置代码**（grep WG_|initNether|env 仅命中 getEnvironmentType 与 initNether 日志行）——即存档链路当前确实没有关 Rust features 的机制，与 H_B' 前提自洽。
- 消融数据（93.8988%→94.4241%，+5508；quartz 4478→2125、gold 1525→739、magma 3814→1979）与 H_B' 预测（回落 ≈ref）方向一致；gold 几乎精确回落（739 vs ref 728），quartz 残余 +133、magma 残余 +446 与「H_A 次级贡献」的 v2 定位相容。
- 消融 2（+WG_SKIP_CARVER 仅 +370）支持「carver 双跑次要」。

## 2. H_B' 证据等级评估 —— PASS（建议授 candidate，附范围声明）

- 消融实验是**数据层干预证据**（不是纯静态解读）：直接操纵存档链路 env、观测 per-id 回落 + match 提升，因果方向明确，且带内置判别（gold 0.3% 精度回落难以用 H_A 基底差解释——基底差不会因关掉 Rust features 消失）。
- 范围限制必须随 candidate 声明（§9.7 精神）：单 region（4×4 @3200,3208）单 seed（B）；机制解释（Rust features 未关 + Java features 双跑）跨 seed 稳定性未验。扩样建议：另一 seed/region 复跑消融一次即可加固。
- verdict 文件头部 status 行仍写「未做任何消融……不得升 candidate」（v1 时代措辞），与 v2 正文已并入 V1 的现状脱节——judge 不改 status，但主会话登记 index 时应以「v2 正文 + 消融数据」为准，并补齐该头部（见 TODO-4）。
- magma 残余 +446 偏大（ref 1533），H_A 次级 + 争用覆盖非线性项的解释目前是定性自洽、未定量闭合——candidate 附带此未闭合项即可，不必阻塞。

## 3. 遗留矛盾（overworld 双跑 vs 99.9% 对齐）—— CONCERN（不阻塞 nether 结论，但 MUST 列独立待查项）

同意主会话初判「不阻塞」，理由：消融是在 nether 存档链路上的直接干预，因果证据不依赖 overworld 侧解释。
但该矛盾**不是可以悬空的哲学问题**，而是有具体可检验的候选解释，且直接牵动修复设计：
- **候选 X1**：overworld 句柄的 feature_loader 实际**没有装配 overworld features**（feature_loader.rs 支持 ore/scattered_ore/underwater_magma，但 overworld biome/rule 侧是否真的 load 了 feature 集未实证）→ overworld 根本没有 Rust features，无双跑。**最廉价的裁决**：overworld 存档 run 开 WG_FEATURELOG 看 `[FEATURE] placed N`，或代码直查 overworld handle 的 feature 装配路径。**judge 建议把 X1 作为第一个核查点**（成本最低、解释力最强）。
- **候选 X2**：overworld 双跑存在但 ore/feature 块占地比例 <0.1%，被 99.9% 对齐率掩盖（nether 矿石密度高得多所以显形）。可用 overworld 侧 ore per-id 对比一步排除。
- **候选 X3**：历史口径的 overworld 对齐采集时有 env 设置（WG_SKIP_FEATURES 等）——查历史 cmd-output/脚本即可。
- **需独立落盘为待查项**（.artifacts 登记 + 台账 M 条目），不得默认「overworld 走不同代码路径」——mixin 直读证明两维度走**同一条** populateNoise 拦截路径，「不同路径」说法目前无证据支持。

## 4. 修复方向评估 —— CONCERN（方向正确，实现方式必须改；nether 侧可先行，overworld 问题须在「全局默认」动作前回答）

- 方向本身 PASS：存档链路关 Rust features/carver 与 cppReplace 契约（Rust 只接管 NOISE+SURFACE）一致，消融已实证收益（+5508 match，per-id 回落）。
- **WARN：env 门是进程全局的**——overworld 与 nether chunk 在同一 server 进程生成，`initNether` 设 `WG_SKIP_FEATURES` 会同时关掉 overworld 的 Rust features；dll 侧默认翻转同理会影响所有调用方。**若 X1/X2/X3 任何一个导致 overworld 依赖 Rust features（或依赖其关闭），全局翻转都会静默改变 overworld 行为**。因此：
  - **推荐实现**：句柄级/调用级显式 flag（如 `wg_fill_blocks` 增 skip 参数，或 nether handle 创建时配置），而不是进程级 env；env 仅保留给 rlib 直跑诊断工具。
  - **必须先答 overworld 问题的场景**：任何「默认」（dll 默认、bridge 全局 init）动作前 MUST；**可以先行动的场景**：nether 维度限定的显式 flag 修复，可与 overworld 待查项并行。
  - 修复后可复现性：保留 env 诊断门（消融复跑、未来 V1 型探针依赖它），勿删。
- 顺带：修复落地后应回归一次 P3（soul 侧 save 不变预期）+ basalt 大宗石 B1 定论不受影响的旁证，纳入同一验证轮。

## 5. P3 措辞修正审查 —— CONCERN（修正方向正确，但**尚未落实**在 P3 产物中）

- P2 v2 正文 L21 已明确「V1 的 pureRust 不是纯 surface 口径，而是 Rust 全管线（含 Rust feature）」——P2 侧已落实。
- **P3 verdict 未同步**：L61 仍写「纯 Rust populateNoise+buildSurface（无 carvers/features）」、L65 表头「pureRust (SURFACE 阶段)」、L73「缺口在 Rust SURFACE 阶段即已存在」——与 P2 的更正**直接冲突**（同一 V1 数据源：basalt 5514 两文一致，说明确为同一次含 features 的 run）。这是产物间口径矛盾，必须按 §16.3/§15.4 精神修正 P3 措辞（保留原表述可加修订注记，不必删改正文历史节，但定稿口径要以「Rust 管线内缺口（surface 阶段嫌疑最大，features 已排除——soul 不被 feature 放置、且消融后 save soul_soil 仍 1334）」为准）。
- 影响评估：措辞错误**不推翻** P3 的方向性结论——soul_soil pureRust 1363 ≈ save 1334 + 消融后仍 1334，缺口在 Rust 管线内独立成立；但「SURFACE 阶段」的归因精度依赖 Rust carver 是否在该 run 中活跃（无直接证据），故修正确实必要。.b1a/.b1b 待 V2 裁决的结构不受影响。
- 另外 P3 L74「soul_sand +587 = Java feature 阶段净增」的表述在双重 feature 视角下仍成立（存档链路 Java features 照跑），与 P2 不冲突——无需改。

## 6. 其他发现

- **WARN-1（落盘契约）**：消融 1/消融 2 的 match/per-id 数据目前只在会话记录与主会话转述中，未见 `.artifacts/.c2-p2-ore-attribution/` 下对应原始输出文件（v2b-ore-per-id.json 等 verdict §0.1 预定的产物名未确认存在——judge 未列目录权限受限，主会话请确认）。**结论不得只留在 chat**：消融原始输出须落盘并登记 index.yaml，否则 candidate 证据链不可引用。
- PASS：P2 的「先消融后归因」纪律执行良好（v1 明确禁止升 candidate，消融后才翻转）；P3 的上轮交接假设经廉价独立验证证伪（§16.3 合规）且证伪链保留。
- PASS：两产物均带 §9.7 可比性声明、Degraded/数据分层标注、@anchor.idk 式诚实残余点。
- 意见限制声明：judge 沙箱无 shell——git diff、目录列举、WG_FEATURELOG 复跑均未执行，相应项以「主会话待办」移交。

---

## 审查结论摘要

| 项 | 判定 | 一句话理由 |
|---|---|---|
| 1. 证据引用一致性 | **PASS** | 数字/代码引用与源码及数据全对上；CppBridge 无 env 设置与 H_B' 前提自洽 |
| 2. H_B' 授 candidate | **PASS（建议授）** | 消融=数据层干预证据，gold 回落 0.3% 精度难由 H_A 解释；附单 region 单 seed 范围声明 + magma 残余未闭合项 |
| 3. overworld 矛盾 | **CONCERN（不阻塞）** | nether 因果证据独立成立；但必须落盘独立待查项，首选候选 X1（overworld 未装配 Rust features，FEATURELOG 一步裁决） |
| 4. 修复方向 | **CONCERN** | 方向对；但 env 是进程全局，禁用全局默认翻转先答 overworld；nether 限定显式 flag（句柄/调用级参数）可先行，保留 env 诊断门 |
| 5. P3 措辞修正 | **CONCERN（未落实）** | P2 已更正、P3 仍写「SURFACE 阶段/无 features」，同源数据口径冲突须修；不推翻 P3 方向性结论 |
| 6. 落盘契约 | **WARN-1** | 消融原始输出未见 artifacts 落盘确认；结论不得只留 chat |

## 待办清单（主会话执行）

1. **TODO-1（裁决 X1，最廉价）**：overworld 存档 run 开 WG_FEATURELOG（或直查 overworld handle feature 装配代码），裁决 overworld 是否双跑；结果落盘 `.artifacts/.c2-p2-ore-attribution/`。
2. **TODO-2**：确认消融 1/2 原始输出已落盘 artifacts + index.yaml 登记；未落盘则补落。
3. **TODO-3**：修 P3 verdict 措辞（pureRust = Rust 全管线；缺口 = Rust 管线内，surface 嫌疑最大、features 已排除），加修订注记不删历史。
4. **TODO-4**：P2 verdict 头部 status/验证分层行按 v2 现状更新（消融已做）；index.yaml 对应条目同步。
5. **TODO-5**：修复实现走 nether 限定句柄/调用级 flag，不依赖进程全局 env；与 TODO-1 并行；落地后回归 P3 soul 侧 + basalt 旁证。
6. **TODO-6**：candidate 授予（H_B' 主机制 + P3 .b1）由用户拍板；批准后按流程交 knowledge subagent 产出 docs/discovered 草稿（含「同源 feature 双跑→消融一步判别」可复用判法，验过 P2 §五建议）。
