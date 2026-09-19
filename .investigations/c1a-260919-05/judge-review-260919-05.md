---
id: c1a:judge-review-260919-05
block: 260919-05
status: draft          # 审查意见；不改任何 status；confirmed 留用户（Anchorlaw v0.22 §15/§16）
role: core.judge（隔离子进程）
reviewed: record-260919-05.md + criteria-preregistered-260919-05.md + cmd-output/* + 架构计划-260919-05-C1a立项实现.md + git HEAD/diff
---

# C-1a 实现轮 judge 审查意见（260919-05）

## 总裁定：**PASS-with-conditions**

正确性面（J1/J2/J3/J5）证据链完整且读法纪律严格；J4 未满足的判读（两配对方向不一致、灰区+FAIL、不作为性能优化计入）严格按预登记执行，无事后挑读法。条件项为 4 条记录/登记补全（N1–N4），均不改变本块判定方向。

## 三源核对（spec §4）

1. **交付快照**：criteria（17:12:17）/ record（17:47:55）/ cmd-output 17 件 + 架构计划（git status 未跟踪新文件）齐备。
2. **git HEAD + 工作区 diff**：HEAD = `1db76a3`（符合预期）；工作区唯一源码改动 = `worldgen-core/src/light/mod.rs`（+165/−69），与 record 声明一致：
   - 全域 fill 一趟：`fill_domain_generic`（泛化 25 chunk 一趟，域成员 81→25）✔
   - 9 中心窗准备 + `propagate_and_export` 抽取（域批路与 per-chunk 路共用）✔
   - seeds 打包 `((em as u32) << 24) | i as u32`：em 为 u8（≤8 位）左移 24 位无溢出；i 为域内索引 < 25×98304 = 2,457,600 < 2^22，**位打包安全 ✔**
   - 双种子位等价测试 `light_compute_domain_bitwise_equivalence`（mod.rs:872-876，LCG 种子 `0x9E37…` / `0xA5A5…` 双采，mod.rs:873）✔
3. **验证记录复算（本人独立重算，非转抄）**：
   - nativeMs P50（n=457/臂，中位数）：off1=2.301 / on1=2.179 / off2=2.208 / on2=2.244 —— 与 record 完全一致；配对比 0.947（灰区）/ 1.016（FAIL）✔
   - reload：`c1a-reload2_changed_set.json` 内 `changed=[]`（0 条）、`verdict=CONVERGENT(floor)`；reload1/2 `light_after` 逐字节相同；chunk key 总数 = 9450 ⇒ changed=0/9450=0.00% ✔
   - SELFCERT：四臂 n=457、fallback=0、domain_task_ok=457、fp_on=1 ✔；残缺率 on1=12/457、on2=13/457（见 N3）

## 逐项结论（M1–M6）

### M1 判据读法纪律（#112）——**PASS**

J4 两配对比值 0.947 / 1.016，分别落预写死的灰区带 (0.90, 0.95] 与 ≥0.95 FAIL 带，record 判「未满足」并把两臂方向不一致、run 间噪声带（off1/off2=1.042）如实呈现——无任何读法替换或择优。J3 按 ≥0.95 门判满足（见 N1 的口径注，但两种读法均 ≥0.95，无归属翻转）。J5 按 ≤0.5% 地板判满足。**无事后挑读法。**

### M2 机制解读——**PASS（qualitative 已声明）**

「共享 fill 串行化关键路径」有 phase 账支撑：on 臂 LIGHTPHASE P50 copyin=1184µs → fill=4108µs（单线程前缀）→ kernel_us=16548 vs wall≈2.2ms（ΣCPU/wall≈7.5，多线程消化自洽）。旧路「9 窗并行、关键路径只含一份窗 fill ~1.3ms」的对照数字未给出处引用（轻注，随 N2 一并补），但 record §4 已把该解读整条标为 qualitative（"phase 账自洽推演，无独立关键路径探针"），符合 §1.3 分级——**未过度外推**。「fill 分块流水」候选也已标「未验证方向，仅登记」。

### M3 等价性证据分层——**PASS**

J1 是合成 LCG 数据上的「共享 fill 路 vs per-chunk 路」等价（E1 同构建态、合成态），测试注释明示「msvc/Java 无关，仅测试内部确定性」——record 未把它说成 Java↔Rust 位等价；实机 2×457 任务零 fallback/degraded 被定位为正确性旁证（§3「保留代码无正确性风险」的支撑面而非位等价证明）。分层合规。

### M4 口径声明（§9.7）——**PASS**

① subcopy 新旧口径不可比：J4 行明示「on 臂 1.82/1.84ms（新窗准备口径，与旧 b9 拷贝不可直接比，§9.7 声明）」；② fill 段口径差（off 臂 9 中心 fillMs 求和 vs on 臂全域 fill_us 一趟）：criteria J4 判据原文 + mod.rs PhaseTimings 注释（"fill_us = 全域共享 fill 一趟（非 9 中心求和）"）双处声明；③ verdict-260919-02 的 fill 13.51ms 全文未被引用（grep 无）。**无跨块数字不当引用。**

### M5 record / preconditions 完整性——**PASS**

J6（RSS）未执行的原因（采集台无 RSS 采样口径、#149 -Xmx 采样配方未建）如实登记，转 idk-C1a-1 并显式声明「不作为采纳决策依据」——合规的 idk 登记而非静默略过。criteria 各判据已带 key/expected/check 前置三元组（§15.1 形态），SELFCERT 硬门全臂绿（dll sha / world 身份 / fallback 计数）可复算。

### M6 采样/驱动纪律——**PASS**

- **时序锚**：criteria 落盘 17:12:17 < 首轮（失败）17:14:27 < 首个有效 result 17:27:36，预登记先于全部采集 ✔
- **失败轮归档**：c1a-off1-aborted-crash.log（17:14:27）换标签独立归档，未覆盖成功轮日志（§9.8 禁自动回退遵守，失败轮证据保留）✔
- **执行体三元组**：全部 7 份日志 in-log `[CppBridge] dll=… sha256=cc4e39fe850b1100…` 一致，与 criteria 预登记全 sha `CC4E39FE…54AE` 前缀吻合 ✔
- reload 双证：同 dll sha in-log（reload1/2 各核对）+ 同 seed ✔

## 新发现问题（条件项）

- **N1（应补，不翻判定）ratioP50 口径未声明**：record J3 写 ratioP50 on1=0.970 / on2=0.963；但两臂 result.json 内 `ratios` 数组的中位数为 on1=0.9723 / on2=0.9700（off1=0.9690 / off2=0.9703），与 record 数字不符。疑为跨臂按任务配对（on[i]/off[i]）的另一种口径，但 record 未声明推导方式（§9.7 口径三要素缺「口径」项）。两种读法均 ≥0.95，J3 判定不变；**条件**：record 补一行 ratioP50 推导口径（配对方式 + 数据源），使数字可复算。
- **N2（应补）J1/J2 运行日志未落盘**：criteria J1/J2 的 check 字段写「本块运行日志」，但 cmd-output/ 内无 cargo test 输出文件——单测与位等价门的运行证据目前只在 record 文字里，证据链不完整（spec §1.3）。**条件**：补落盘 cargo test 输出（或注明宿主会话可引用出处）。随带：M2 旧路 ~1.3ms 窗 fill 的出处一并注明（b9 历史 fillMs 口径文件）。
- **N3（轻注）残缺率声明精度**：实为 on1=12/457、on2=13/457；record 写单一「13/457=2.8%」。应写「最差臂 13/457（on2）」，避免读成总量口径。
- **N4（流程）index.yaml 未登记**：`.artifacts/index.yaml` 尚无本块条目（record/criteria/本审查意见）。与 260918-04 同族缺口，按先例在用户拍板后随 verdict 一并补登 root index。

## 其它 checklist 项

- status 合法性：record/criteria 均 draft，无 confirmed 越权 ✔
- 副作用与逆（§9.8）：失败轮换标签归档（逆=归档件可寻址）；各轮 gradle 临时目录独立 ✔
- halt / retry cap：本轮无 gate C halt，无超限验证轮 ✔
- 模块边界：record 未跨模块引用 skill 正文 ✔

## 推荐状态

保持 **draft**，建议用户按 §3 处置拍板（采纳=保留缺省生效 / 回退=revert mod.rs）；性能面「不作为优化计入」的方向与 verdict-260919-03 C-3 先例一致，本人意见支持该处置。拍板后按 N4 补登 index，conditions N1–N3 在 record 内补全即可闭合。
