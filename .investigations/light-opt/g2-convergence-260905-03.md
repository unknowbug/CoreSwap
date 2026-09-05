# G2 残差根因收敛记录（260905-03 · fan-out 三候选 + 主会话决定性探针）

## 背景与验证链

- 交接假设（260905-02）：G2 残差根因 = fallback chunk 收不到被接管邻居的 propagateLight 更新 → 修复方向 = mixin 补调 propagateLight。
- 廉价独立验证（本 session 首个动作）：残差解剖（`.tmp/light-g1/g2_residual_map.py` / `g2_residual_anatomy.py`）**不支持该假设**——残差 448/3377 chunks 全在 interior（0 个在 pregen 边界）、face 分布 inner 主导、主导签名 = 整 section sky 0↔15「翻转」→ 判定树分叉 ≥2 互斥候选 → fan-out。
- 脚本缺陷自纠（复用 cmp_light parser 途中发现）：①1.20.1 chunk NBT 无 xPos（按 region+索引推导）；②struct.unpack_from 不推进指针致解析失步；③首轮把 vanilla 隐式全15 fill 规则误用于 rust 侧。

## fan-out 裁决（三 worker 并行，产物 .investigations/light-opt/g2-fanout/）

- **.b1 传播缺失（H1）：DENY**。fallback 只可能在 pregen 边界发生但残差 0/448 在边界；vanilla `propagateLight` 是拉取语义（method_51589 直接读 4 邻共享 LightStorage），「收不到推送」前提不成立。产物 b1-propagation-missing.md。
- **.b3 缺键序列化（H3）：DENY（唯一根因）/ CONFIRM（放大器候选）**。1138 残差 section 仅 360 个 rust 缺键型，778 个双侧有键真差。产物 b3-missing-key-semantics.md。
- **.b2 sky 直落/索引（H2）：DENY（内核缺陷）**，但定位两个关键事实：①360 个「全翻转」全部是 vanilla 有键渐变 vs rust 缺键，rust MCA 有 14307 均质-15 键、0 个均质-0 键 → **rust 缺键=flag1 全 0，对比口径把 rust 缺键按隐式 15 填充制造了 0↔15 假象**；②翻转集中 Y=3..4 地表树冠段、显式差剖面 rust=vanilla−1~2（树冠顶多一档衰减）→ **blocks9 树冠输入差签名**。产物 b2-sky-fallthrough.md。

## 主会话决定性探针（.tmp/light-g1/g2_verdict_probe.py）

- 修正口径后重对比：448 chunks / 781,997 nibble diffs（对比口径修复只消除了 |d|=15 假象质量，残差 chunk 数不变——说明「翻转」全是口径伪差 + 真实光照差同时存在）。
- **worst 6 chunks 逐块 palette 对比**：真实差异 = 树木/藤蔓放置分歧——`jungle_leaves`/`oak_leaves`/`vine`/`jungle_log` 增减各 80-350 处/chunk（另 granite↔andesite 少量）。光照差是这些 blocks 差的忠实后果（叶 opacity=1、vine 遮挡）。
- 附注（诚实声明）：(air,None) ~4000 处为本探针 unpack 的 data==None section 处理伪差，不计证据。

## 收敛结论（confirmed 2026-09-05 用户拍板；judge APPROVE-WITH-CONDITIONS 已并入）

1. **光照内核在「blocks9 输入一致域」无缺陷（confirmed，收窄表述）**——G1 exact 100% 仅覆盖 4×4@200 树冠一致区 + 静态审查，不为树冠差区域背书。
2. **原 propagateLight 修复方向作废**（H1 对 flip 主导签名 DENY；对 prop 型 210 chunk 小差值 = UNCERTAIN-weak，最终排除依赖 `LightStorage.enqueueSectionData` 再传播语义源码核验——@anchor.idk 保持 open，随 G3 round-trip 一起关闭，MUST 跟踪）。
3. **G2 残差真实根因 = worldgen feature/放置分歧（树冠 + 藤蔓 + 矿石/安山岩替换层）**——judge 全量 447 chunk palette 对比：346/447 chunk 有 blocks 差（kind 签名含 stone↔coal_ore、granite↔andesite，不只树叶）。新课题范围 MUST 覆盖矿石/替换层，否则判据改定义后矿石区复发。是否属既有挂起域待用户裁定（不在 260904-15 四项挂起清单内）。
4. 对比口径修正规则：vanilla 缺 SkyLight 键=15（**推断级/Degraded：ChunkSerializer 一手源不在工作区**；vanilla 地下 Y0-2 缺键实为隐式 0 的撞平隐患保留登记）；rust 缺键=flag1 全 0（一手源实证：14307 均质-15 键/0 均质-0 键）。两侧缺键语义不同，不得统一填充。
5. **judge 新发现群体（条件 3 补登）**：101/447 chunk「自身 blocks 零差但光照有差」——最差样本光照差 >50% 落在 chunk 内部列仅 4/101，呈边界面富集，与「邻 chunk 树冠分歧的影子跨边界传入（sky ~15 格衰减）」空间形态相容；**该解释为推断级**，未经 y/柱级直接验证，列为未解释残差候选（建议一轮 y/柱级验证或随新课题覆盖）。
6. 复现口径（§9.7）：残差 chunk 计数对脚本实现敏感（主会话 448/781,997 vs judge 447/1,172,606，填充口径/统计面不同），chunk 级结论稳健；引用数字须带口径。
7. 附带登记：light_data.json 18 个 opacity:-1 条目被 parse_u8_field clamp 成 0（worldgen-core/src/light/mod.rs:121-125，与本残差无关，语义有损待修）。
8. judge 产物：意见全文见会话记录；复核脚本 .tmp/light-g1/judge_counterexample.py / judge_face.py。

## 对 G2 判据的影响（重大转向，待用户决策）

- 原 G2 判据「全 region 消除 |d|≥1」在 feature 放置分歧存在时**不可达**（光照忠实反映 blocks 差；影子传播群体另计）。
- 可选方向：①G2 判据改定义为「blocks 一致域内光照 exact 100%」（G1 已证）+ feature 放置分歧移交新课题（范围含树冠/藤蔓/矿石）；②开 worldgen feature parity 新排查（scout 勘探 feature placement 管线）。
- 不论方向：G3 round-trip 须一并关闭「enqueueSectionData 再传播」idk（固化残差风险）。
