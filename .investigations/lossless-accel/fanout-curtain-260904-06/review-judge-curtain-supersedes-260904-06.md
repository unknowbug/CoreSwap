# Judge 审查意见：幕帘课题取代裁决（curtain-verdict-260904-06，§15.4 重大转向，260904-06）

> 审查角色：core.judge（隔离 subagent，只出意见不改 status）
> 审查对象：`.artifacts/lossless-accel/curtain-verdict-260904-06.md`（candidate）+ `.investigations/lossless-accel/fanout-curtain-260904-06/p4-reference-check-260904-06.md` + 三臂 b1/b2/b3（candidate-pending）
> 被取代方：`.artifacts/lossless-accel/writer-verdict-260904-06.md` 结论 3（「真根因上移 NOISE：幕帘 = C++/Rust 共有 vanilla 偏离」）
> **总体判定：PASS-with-conditions**（取代裁决本身证据充分、逻辑闭合；5 项补正条件，均不动摇结论方向）

---

## 1. 三源核对（spec §4 / judge 基线）

| 源 | 核对结果 |
|---|---|
| ① 产物快照 | curtain-verdict / p4-reference-check / b1/b2/b3 全部落盘，内容与引用一致 |
| ② git HEAD + 工作区 | **HEAD = 924e934**（非移交单预期的 3f047a2）——924e934 恰为「curtain supersedes verdict」docs 提交，含且仅含 curtain-verdict + p4-reference-check 两文件（+49 行），工作区 clean。快照 = 已提交内容，**无快照滞后**；移交单「3f047a2 + 未提交」预期已过时，非违规，记录在案 |
| ③ 验证记录 | 探针原始产物在 `.tmp/p2full/`（cpp-dbdebug-195-199-260904-06.txt / cpp-fdcol-195-199.txt / ref_check + xcheck 脚本）；judge **独立重跑** ref_check_p4 与 xcheck_p4（只读）复现全部关键数字（见 §3） |

## 2. 推翻链逻辑闭合性

链：d 全负 → stone 只能来自 margin → margin 机制三方零偏离（b3）→ vanilla 必有幕帘 → P4 实测证实。逐环核验：

1. **d 全负（✅ 实测确证）**：cpp-dbdebug-195-199 dump y180-319 全负——y192-255 = −0.458333、y256-318 = −0.024995（−0.02 带）；density_probe 点采 %.17g（cpp-fdcol）= −0.4583333…/−0.024994791… 恒定，与生产一致。**且与 b1 §2.2 纯静态理论预测（−0.458 / −0.025 地板）逐位吻合**——静态推导与生产 dump 互相背书，强信号。「幕帘 = 0<d」旧前提在 C++ 臂确实证伪。
2. **stone 只能来自 margin（✅ 结构性成立）**：b3 §1 论证 `apply` 在 d≤0 时 return 集合 = {流体, bs, margin 三连 -1}，无第三条路；C++ 消费环 block<0 → stone（worldgen_api.cpp:1040）。静态结构穷举，可信。
3. **margin 机制三方零偏离（✅ 静态；不承担独证）**：b3 §2 十七项全 ✓（Degraded 分层，其自身已声明）。**关键：本环不承担独证负担**——b3 同时发现真实分叉（Rust margin→air 丢 barrier stone），但该分叉只影响幕帘内部构成，不影响「vanilla 同构存在 margin 机制」的定性。
4. **P4 实测证实（✅ 独立一手证据，闭环点）**：结论不依赖 3→4 的纯推理——vanilla ref 列 stone 族是**直接实测**的。因此即使 b3 静态对拍有遗漏，裁决依然成立。**无循环论证**：P4 读数独立于被推翻的「vanilla y≥201 无 stone」前提（该前提源自 convergence「新世界图景」对同一 .blocks 的误读——ref_col_check_260904-06.py 佐证：该轮脚本只查 dirt/sand/gravel，从未核对 stone 族，误读属「读了但看错对象」而非解析错误）。

**判定：链闭合，无断链，无循环。** 唯一注意点：b3 的 P-4 预测（「vanilla 同列无 stone」）被本裁决证伪——它是预测非结论，取代记录无需覆盖，但收敛时应把 b3 的 P-4 标 ❌ 一行（条件 C5 提及）。

## 3. 关键数据抽核（judge 独立重跑，非复述）

- **① ref col(195,199) 读法（✅）**：重跑 ref_check_p4 → `stone:99, granite:24, copper_ore:7, iron_ore:1`（y180-319），与 p4 Facts #3 完全一致。idx = 3*(H*16)+7*H+(y−(−64))，chunk(12,12)/local(3,7) 对应世界列 (195,199)，seed 双臂同 8576294172403134396（xcheck 打印核对），blocks.json 抽查 stone=1/granite=2/copper=923 与 b3 引用契约一致。
  ⚠️ **措辞问题（C2）**：p4 Facts #3 与 curtain-verdict 推翻理由行均写「99 个 stone 族块（stone 99 + granite 24 + …）」——括号内合计实为 **131** stone 族块（99 是 stone 单项计数）。「99」是词面歧义非结论错误（哪怕 1 块 y>201 stone 已足杀前提），但应改为「131 stone 族块（其中 stone 99）」。
- **② WG_DBDEBUG 负值读数（✅）**：见 §2.1；y180-319 无一正值，dump 与点采两载体交叉一致。
- **③ xcheck 32 diffs（✅）**：重跑 xcheck → `y180-319 diffs = 32`，且 **32 = 24 granite + 7 copper + 1 iron，精确等于 ref 带内 ore 族总数**——即全部 diff 都是 `C++ stone ← vanilla granite/copper/iron_ore`，无一例外（展示的前 20 条逐一核对同型）。verdict §3「32 块全部同型」确证，y 范围 185-229 ⊂ 所述 185-231。

## 4. 被取代结论波及面

- **writer-verdict 结论 1（写者 = Rust SURFACE）：不受影响**——依据是 E1 运行时消融（WG_SKIP_SURFACE 后列清空）+ 结构消去链，与「vanilla 无 stone」前提无关。
- **结论 2（双跑伪影结案）：不受影响**——dll/git 时间戳证据，独立。
- **结论 3：取代正确且范围恰当**——其幕帘段（0<d≤0.39 描述、移交「共有偏离」线）全部失效；(244) 列 heightmap 判据分叉（m2）属独立机制，未被本次证据动摇，verdict 未波及它，正确。
- **260905 载具裁决：不受影响**——其判据是 C++/mod 臂间对比与执行体归属，不依赖 vanilla 参照 stone 解读。
- **残留敞口（C3）**：p4 下一步 #2 自认「此前以错误参照前提归因的差异需重算」——目前只有 convergence「新世界图景」一处引用被定位；主会话应做一次 grep 级波及扫描（「vanilla 无 stone / y≥201」表述在 docs/台账的其他出现点）并逐点标注，方能关闭被取代结论的全部下游引用。

## 5. 取代记录格式（§15.4 合规性）

- curtain-verdict 头部：supersedes 指针指向 writer-verdict-260904-06 **结论 3**（粒度正确，非全篇取代）+ 一行推翻理由 + 原文不删不改（已核对 writer-verdict 正文未被改动）✅
- **C1（反向指针缺失）**：writer-verdict-260904-06 头部仍是 `superseded-by: （无）`——§15.4 要求双指针，被取代方应补 `superseded-by: curtain-verdict-260904-06.md#结论3`（只加指针，不动正文）。
- **C4（index.yaml 未登记）**：`.artifacts/lossless-accel/index.yaml` 无 curtain-verdict、p4-reference-check、review、b1/b2/b3 条目（b1/b2/b3 随 3f047a2 提交但同样未登记）。产物契约（core.artifact）要求登记，应补。

## 6. idk 诚实性

- grass/water 交替（y 67-214，两臂逐位一致）✅ 已登记（#8 raw id 家族嫌疑）；ref y180-319 带内也有 5 个 grass_block，同一 idk 覆盖，可接受。
- granite@y217-229 写者（超 oreVein y≤50 与 feature 上界 128/112）✅ 已登记为新 idk。
- C++ margin stone 占比未定量 ✅ 已登记。
- b1 §6 明列「vanilla 无 stone 引用自 convergence，本臂未独立复验」——事后看正是这一条 idk 命中真 bug，诚实声明纪律起到了应有的风险标记作用 ✅。
- 未查项（声明）：噪声卡历史未核（本会话未初始化 anchorlaw 噪声卡基础设施）；retry cap 无问题（P4 为新数据层证据轮，计数重置）。

## 7. 推荐状态

- **curtain-verdict-260904-06：建议维持 candidate**（维持，非降级；C1-C5 补正后可请人类拍板 confirmed）。
- b1/b2/b3：b1 密度抬升候选随课题对象消失而失去靶子（verdict §2 已裁）——建议收敛时在 b1 头部加一行「对象消失，停于 candidate-pending 不再推进」；b2/b3 的排除性结论与 margin 分叉发现**独立于参照误读，维持有效**。
- confirmed 留人类，本意见不授予任何状态。

## 8. 条件清单（PASS-with-conditions）

| # | 条件 | 级别 |
|---|---|---|
| C1 | writer-verdict-260904-06 补 `superseded-by` 反向指针（只加不改） | MUST（§15.4 双指针） |
| C2 | 「99 stone 族块」措辞改正为「131 stone 族块（stone 99 + granite 24 + copper 7 + iron 1）」（p4 + verdict 两处） | SHOULD |
| C3 | 被取代前提的下游引用扫描（「vanilla y≥201 无 stone」类表述全库 grep + 逐点标注） | MUST（confirmed 前） |
| C4 | index.yaml 补登记 curtain-verdict / review / p4 / b1-b3 条目 | MUST（产物契约） |
| C5 | 收敛时 b3 P-4 预测标 ❌、b1 加「对象消失」停推一行 | SHOULD |
