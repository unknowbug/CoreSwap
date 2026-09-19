# judge 审查意见 — T8 值差机制归因 verdict（260919-07）

- 审查对象：.artifacts/t8-attrib-260919-07/verdict-260919-07.md（draft，建议 candidate）
- 审查角色：judge（core.judge / anchor-judge 形态，只出意见不改 status；confirmed 留人类）
- 三源核对：① 产物快照（verdict + scout-map + b1–b5 + 架构计划）② 原始读数（cmd-output/ 八份抽验）③ 验证记录（预登记判据 / VOID 退出码 / world 身份链 / 判据读法守恒）
- 审查结论：**PASS-with-conditions**（建议 candidate 可成立，附条件 C1–C4 登记后交用户拍板）

---

## A. 数字逐项抽验（verdict §1/§3 ↔ 原始读数，#162 qualitative MUST 回日志）

| # | verdict 引用 | 原始读数出处 | 一致？ |
|---|---|---|---|
| A1 | diff = 4223/9450（44.69%）；2 连通域 max=3607、616；singleton=0；中带零 diff | dist-analysis-260919-07.txt:1-3（44.6878%） | ✓ |
| A2 | to=625（三 run 同一）、full=3078、inline=0；fullHit=3078/3078 | b2-burst-extract:1-3 + b2 §8.1 交集复算（diff∩to=625、diff∩full=3078） | ✓ |
| A3 | d_self=2185（23.12%）、d_cross_r2=r3=4223 key 集恒定、d_self 100%⊆center、中带 0 | n2-mechanical-260919-07.txt:1-7（2185/2185、mid-band 0、identical=True） | ✓ |
| A4 | 4223 = 2038 恒定 + 2185 波动；d_cross constant=4223 fluctuating=0 | n2-mechanical + b4-520-intersect:4（constant 4223/fluctuating 0）+ b1 §10 算术 | ✓ |
| A5 | 检验A = 0/1885；to 集三 run 逐 key 同一（to1Δto2=to1Δto3=0） | b2-assignment-test:4,6 | ✓ |
| A6 | 档①：snap 互验 0/9450；520/520 unlit；lit=3703；rc=0 | b5-r3-audit-260919-07.txt:2-5,12 | ✓ |
| A7 | 520 ⊆ d_cross_r2/r3 逐 key | b4-520-intersect:2-3（b4 §8.2 曾声明未复算，本读数已闭合该缺口） | ✓ |

**逐项一致，无转录漂移。** 44.69% = 3703（39.19pp）+ 520（5.50pp）算术闭合；520 = 簇外差 87+433 精确闭合（b4 §1，零自由度声明成立——两条独立恒等式均闭合且经 A6 运行时直证）。

## B. 逐项审查意见

### N1 【PASS】归因主链证据闭合与算术
分账式结构（3703 值差 + 520 存在性差）证据充分：520 面有「路径穷举排除（inline=0/fallback=0/degraded=0/lightInit ok）+ b4 §2.1 强制链 + 档① 520/520 unlit 运行时直证 + 520⊆d_cross 恒定」四重闭合，零自由度声明成立。值差面的 n=2 判别（错集恒定 × 错值漂移的乘性结构）判别实验设计有效。

### N2 【CONCERN】2038 恒定项的机制归因未与「确定性引擎语义差」判别分离（条件 C1）
b5 §3 的三候选判别（①纯终态化 ②纯覆写 ③两面合并）有效排除①②，但存在**第四种未被列举/未被判别的分解**：「恒定 2038 = 确定性 Rust-vs-Java 内核/边界语义差（b1 §2 S-a/S-c/d 面，结构性存在）+ 抖动 2185 = b5a 覆写」——该组合同样满足全部现有观测（d_self≠0 由覆写承担、d_cross 恒定由确定性差承担）。b1 §10「③ 唯一存活形态」的措辞在该分解未被排除前偏强。verdict §1.1 把「写回终态化通路」标为主嫌可以成立（静态代码链完整、判别力最强），但「2038 = 提交时快照 vs 终态世界的差固化」目前是**最强假说而非已证机制**——档② keep-world（O1 形态门）与档③（E-3a 输入 hash）正是判别此点的实验，均未跑。candidate 可授（有界未决已声明），但 verdict 应明示该备选分解存活。

### N3 【CONCERN】排除清单「内核逐位语义差」行的证据指向性（条件 C2）
排除清单引用 mod.rs:866-900 位等价单测——该测试证明的是**域批内核 ≡ Rust per-chunk 内核**（Rust 内部轴），**不**排除「Rust 内核 vs Java vanilla」的语义差（后者正是 b1 §2 S-a/c/d 让渡面，且按 N2 可能承载恒定 2038 的一部分）。行文「内核逐位语义差」二义：按「域批 vs per-chunk」读法排除成立；按「Rust vs Java」读法不成立。建议 verdict 该行限定轴声明，防后续读者误读为引擎差已被排除。

### N4 【CONCERN】排除清单「均有数据层证据」的总体声明对 .b5b 行偏强
.b5b 排除（R1-R5）是**静态代码语义排除**（非数据层），仅 R3 有数据背书（to 集三 run 同一）；R6 已诚实登记为残余。verdict §2 表头「均有数据层证据」的 blanket 声明与该行实际证据层级不符（§9.7 无效声明清单「静态断言当地面真相」边缘形态）。建议改为「除 .b5b 行（静态出清，R6 残余登记）外均有数据层证据」或同表分层标注。

### N5 【PASS】写回终态化两面合并表述 vs I 级参照的诚实性
两面合并（Mixin:594-595 一行代码对）静态链完整；终态化面证据 = F 代码 + :593 显式复刻声明注释 + LegacyTakeover 同款镜像（F）；覆写面 = **I 级参照**（yarn LightStorage/增量传播语义不在工作区），verdict §4 已显式登记 I 级 + O1 门未跑，§16.3/#162 口径合规。轻微保留：verdict §1.1 正文以陈述语气写覆写机制（「就地改写」），限定语只在 §4——候选措辞可再收半档（条件 C3：正文 I 级面加内联限定），不阻塞 candidate。

### N6 【PASS】未决/降级声明完整性
O1 门未跑 ✓、R6 ✓、档②③ ✓、E1/E2 档位与 n=2 不外推 ✓、520 跨 run 恒定前提依赖声明 ✓、b2 §9.4 灰区判据作废重订 ✓、b3 Degraded 分层声明 ✓。b4 §8.2 自报的「520⊆d_cross 未逐 key 复算」缺口已被 b4-520-intersect 读数闭合（A7）——追踪链完整。

### N7 【PASS】预登记判据时序与 VOID/退出码纪律
b2 §5.2 判据文本内嵌于 worker 交付、先于 n=2 执行（§9 引用同一文本 + §9.4 灰区后重订判据亦为预登记形态）；b5 §4.2 档① preconditions（key/expected/check 五项）先于 §4.3 模板，实测 rc=0 且退出码区分 VOID(1)/FAIL(2/3)/OK(0) 已机械落实。SELFCERT 全绿 exit 0（n2 记录）。world 身份链（#161：seed_in_leveldat / region mtime>rmtree / keep-world 副作用逆登记预告 b5 §4 档②）齐备。

### N8 【PASS】判据读法守恒
全产物一致遵守 criteria-t8.md 存在性判据不回改（scout §4.2 / b2 §5.2 / b4 §5 / verdict 均显式引用）；n=2 归因实验未触碰主判据；44.69% verdict 未被翻案，只做机制分解。

### N9 【PASS/备忘】worker 间冲突与让渡链
让渡链闭合：b1 引擎差让渡 → b4 剥离中带 E-a 证据 + 520 分账；b2 §10 关闭原辖区让位抖动候选；b5 合并 .b1 终态化假说（术语「写回终态化通路」统一）。无未裁决冲突。两处备忘（不扣分）：① b2 §10.3 提议的候选名「.b4a/.b4b」实际落盘为 .b5a/.b5b（.b4 号已被 outside-chunks worker 占用）——编号漂移可追溯，建议台账登记一行更正；② b1 §11（更新二）的部分判读（终态化为主因）在 b4 §3 剥离 E-a 证据后措辞已被 b5 §3 取代，取代链在 b1 内部（§7→§9→§11）与跨产物（b5 §3）均有登记，合规。

### N10 【CONCERN】.artifacts index.yaml 未登记（条件 C4）
.artifacts/t8-attrib-260919-07/ 下无 index.yaml，根 .artifacts/index.yaml 亦无 t8-attrib 条目——core.artifact 落盘契约的索引面缺失（verdict 正文本身在位）。candidate 授予前应补登记（本条为流程性缺口，不涉证据质量）。

### N11 【PASS】retry cap / 副作用 / 模块边界
证据饱和未触线（每轮均有新数据层读数：dist → burst → n2 → assignment → r3-audit）；run_t8 rmtree 重建为实验设计内声明动作、keep-world 副作用逆已预告（b5 §4 档②）；无跨模块 skill 正文引用；各 worker 辖区边界清晰（#73 让渡清单制执行到位）。

## C. 条件清单（candidate 授予建议附带）

- **C1**：verdict 增补一段「备选分解登记」——「恒定 2038 = 确定性引擎/内核语义差 + 抖动 = 覆写」的组合未被现有数据排除，判别实验 = 档②O1 + 档③E-3a；在此登记前「写回终态化通路」读作**最强假说主嫌**而非已证唯一机制（收敛 N2）。
- **C2**：排除清单「内核逐位语义差」行补轴限定（域批 vs per-chunk 内部轴；不排除 Rust-vs-Java 引擎语义差）（收敛 N3）。
- **C3**：verdict §1.1 覆写子面正文加 I 级参照内联限定（与 §4 呼应）；§2 表头「均有数据层证据」对 .b5b 行改为分层表述（收敛 N4/N5）。
- **C4**：补 .artifacts index.yaml 登记（收敛 N10）。

## D. 总评

**PASS-with-conditions。** 数字链逐项与原始读数一致、520 存在性差四重闭合且经档①运行时直证、预登记/VOID/world 链/判据守恒纪律全程在位——**支持升级 candidate**；条件 C1–C4 为措辞收敛与登记补全，均为非阻塞项，建议随 candidate 授予一并处理，档②/档③ 判别实验（终态化 vs 确定性语义差的分离）列为 candidate → confirmed 前的优先验证项。confirmed 留用户拍板。

---
*judge 只出意见，未改动任何产物 status（verdict-260919-07.md 保持 draft）。*
