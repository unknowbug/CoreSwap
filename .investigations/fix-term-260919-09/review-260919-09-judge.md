# judge review — verdict-260919-09（B1 主攻包验收，draft / J2 VOID）

reviewer: core.judge（隔离子进程，Anchorlaw §15/§16；只出意见，不改任何 status，confirmed 留人类）
审查对象: .artifacts/fix-term-260919-09/verdict-260919-09.md
三源核对: ① artifacts 快照 ② git HEAD/worktree diff（HEAD=882ad20，工作区未提交批 = 本块实施，见意见 #1）③ 原始产物 .tmp/fix-term-260919-09/（drv-f0/f1/f2.log、o1o3-f1f2.json、b1-*-result.json、regions/ 三目录 18 mca 各臂在案，实测核到）

## 意见清单

**#1 三源核对·实施文件在案 — PASS**
verdict/record 声称的实施面逐项实测核到：coreswap.accesswidener（header v2 + Stage 类/enqueue 4 参加宽行在案，idk 注释在案）；WgLightPost.java（直接公有 enqueue 调用，@Invoker 否决史注释在案）；fabric.mod.json `accessWidener: coreswap.accesswidener`（且 mixins 数组**未**含第二 config——与 record §6.1「accessor/第二 mixin config 撤销」一致）；build.gradle `accessWidenerPath` + 三条新映射（post-finalize/latesubmit/wbprobe，点分族）；check_switch_mapping.py :74 字符类补 `-`（diff 单行，逆登记在注释）；ServerLightingProviderMixin.java:619-636（[LIGHT-WB] pre + WgLightPost.postUpdate + setLightOn 后移）、:892-930（内联路同构）、:774-775（postF 分支 replay future 语义）。git status 未提交批与此完全对应。

**#2 验证记录数字对账 — PASS（附一条读法澄清）**
drv-f1.log 尾 / f1_result.json SELFCERT：wb_pre=wb_post=3703、wb_inline=0（J4 ✓）；resealed="8"、domain_task=464（J5 ✓）；dll_sha8=cc4e39fe ✓；hook_postF=("true","merge") ✓；f2 全同值 ✓；vanilla 臂 lightInit=0/domain_hook=0/wb 全 0（负臂干净 ✓）。o1o3-f1f2.json：jitter=7660、diff_nibbles=4,266,861、band 99.571%/interior 0.429%——与 verdict 表逐项一致。
澄清：SELFCERT `"degraded": 464` 的键义 = `degraded=0 ` 行计数（run_b1.py:143），即「464 task 全部 degraded=0」——verdict「degraded=0/464」读法忠实；键名易误读为「464 次降级」，建议主会话在 record 或脚本注释补一行键义说明（意见级，非缺陷）。

**#3 J1 独立复算 — PASS**
judge 独立读三份 light json：f1=9450/f2=9450/vanilla=9450 keys；f1×f2 common=9450、new=0、gone=0——J1 与 verdict 逐字一致。vanilla 对照：f1-vs-van common=9450、单侧差 0，f1-van 值差 4049（与 verdict J3「4049/4056」的 f1 半边实测吻合）。

**#4 J2 判据语义忠实性（N-1）— CONCERN**
预登记 criteria J2 阈值写的是 **chunk 级 2185**（「d_self ≤ 2185×0.5 / d_self > 2185 = VOID」），而比较器是 section 级 jitter——预登记本身口径混写，字面阈值对 section 度量不可应用。verdict 实际以同口径 6453→7660 替代执行并在 §9.7（verdict:27）声明了不可直比——**方向不是事后挑选**：judge 独立复算 chunk 级 f1×f2 值差 = **2643 > 2185**，VOID 在两种口径下均成立，结论稳健。但 verdict「预登记读法机械执行」的表述过强：实际是「预登记阈值口径混合，按 §9.7 同口径替代基线执行」。**条件**：verdict 应 (a) 补记 chunk 级 d_self=2643（双口径 VOID 同证，反而更强）；(b) 把「机械执行」改为如实表述（替代基线 6453 的选择本身是一次口径裁决，应留痕）。

**#5 失败臂对称摘票默认臂 delta（N-2）— CONCERN**
record §3（:45）已如实登记「无条件落地 + 默认臂行为 delta = 泄漏→不泄漏，登记交 judge」，落盘合规。但 verdict（被审产物）覆盖面/边界节**未复述**该 delta——verdict 是验收面唯一快照，默认臂（postF=off）行为变更应在其覆盖面声明中出现。风险本身 judge 判定可接受：HEAD 该臂 ticket 确定性泄漏、cancel 后 vanilla POST 永不执行，摘票属防御性收窄；但「登记在哪」须闭环到 verdict。**条件**：verdict 覆盖面补一行。

**#6 FB-2 回退成本呈现（N-3）— PASS**
verdict:23 三选项并列（回退 FB-2 半包 / 档③先判别 / 接受抖动换存在性），明确「重大方向决策留用户拍板，不自动重跑」，与设计 §3.1「删行即回退，无开关」一致，未越权推荐单一方向。合规。

**#7 实施与设计偏离登记（N-4）— PASS（附登记面小瑕疵）**
三处偏离均在 record 登记：AW 替代 @Invoker（§6.1，含编译否决证据指针 + Forge/Connector idk）；excludeBlocks 穿透（§1.3「超出设计文案，登记待 judge」——本 judge 意见：语义更贴 vanilla（复刻 SLP:175 guard），方向正确，认可待用户认）；[LIGHT-WB] 探针新增（§2/§3，门控 + 正负成对）。瑕疵：record §2 实施清单表仍列「coreswap-world.mixins.json 新建」，未随 §6.1 撤销更新（§6.1 已写明撤销，靠读者自行合并）——建议主会话在 §2 表加一行「已撤销，见 §6.1」。

**#8 覆盖面声明完整性（N-5）— PASS（附一条遗漏）**
E1/n=2/O3 降级/内联路无独立判据均已声明（verdict:27，与 criteria:37 一致）。遗漏：o1o3-f1f2.json notes 含 `"r3/r4 leveldat absent seed by log only (degraded)"`——**基线 r3/r4 的 seed 身份是 log-only 降级档**，与基线数值（6453/3,354,675/2185）比较时该降级声明未随行进 verdict。§1.3 numeric 档允许继承，但降级注记应显式带到比较声明里。**条件**：verdict §9.7 节补一行基线降级档声明。

**#9 产物契约（core.artifact）— MUST-FIX（机械项）**
`.artifacts/fix-term-260919-09/` 仅有 verdict 一文件，根 `.artifacts/index.yaml` **无 fix-term-260919-09 条目**（实测 grep 0 命中）。产物未入主索引，违反落盘契约。**条件**：补 index.yaml 登记（draft/J2 VOID）。

**#10 知识库候选评估（N-6）— 值得写（judge 意见，不代写）**
「补传播使边界进入传播体系 → 传播消化次序 run 级非确定 → 抖动面扩张（7660>6453、2643>2185 双口径同证）」具备 discovered 资格：可复用判据形态 = **「修复预期须分『值收敛』与『过程确定性』两轴；『更贴 vanilla 机制 ≠ 更收敛』——把更多面纳入非确定排程体系可系统性放大 run 间方差**」。过价值门（再遇同类「贴原版修复反而劣化」时不需重想）。建议主会话按流程派 knowledge subagent 起草（附 SUBAGENT-KNOWLEDGE-GUIDE），本 judge 只证资格。

**#11 其他核对项 — PASS**
retry/evidence saturation：VOID 轮 = 新数据层证据，verdict:23 声明计数重置，合规。PI-1：VOID 未自动重跑/未自动回退，未静默继承为下一轮基底（verdict:3、:23）。§9.8 逆登记：verdict:31 + criteria §9.8 表 + run_b1.py:7-8 头注一致，regions=derived、level-seed .bak 在案（脚本 :49-51）、rmtree 无自动回退显式声明。模块边界：产物未跨模块引用 skill 正文。噪声卡：未发现本目标未解决噪声卡阻塞项。fallback 判据：f1 log `fallback vanilla` 0 行 + `declined` 0 行，与 criteria 前置（计数=0）一致。

## 总评：**PASS-with-conditions**

VOID 判定本身**成立且稳健**（judge 双口径独立复算：section 7660>6453、chunk 2643>2185），verdict 诚实登记失败而非掩盖，PI-1 断链纪律执行正确。维持 **draft**；J2 VOID 读数建议维持，方向决策（FB-2 回退与否）留用户 HOOK。

放行条件（全部机械级，不动判据语义）：
1. **C-1（MUST）**：补 `.artifacts/index.yaml` 的 fix-term-260919-09 条目（意见 #9）。
2. **C-2（MUST）**：verdict 修正 J2「机械执行」表述为「预登记阈值口径混合，按 §9.7 同口径替代执行」，并补记 chunk 级 d_self=2643 双口径同证（意见 #4）。
3. **C-3（SHOULD）**：verdict 覆盖面补失败臂摘票默认臂 delta 一行 + 基线 r3/r4 seed-by-log-only 降级档声明（意见 #5/#8）。
4. **C-4（SHOULD）**：record §2 表补第二 mixin config「已撤销」标注；SELFCERT `degraded` 键义注释（意见 #2/#7）。
5. **C-5（可选）**：用户若采纳，派 knowledge subagent 起草意见 #10 的 discovered 条目。

（核到证据均给出 file:line；.tmp 产物本轮实测在场，若被清以本 review + result.json/o1o3-f1f2.json 引用为准。）
