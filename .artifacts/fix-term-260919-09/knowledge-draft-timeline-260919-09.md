# 草稿 2/3：versions/1.20.1/docs/10-timewise-archive.md 追加块（260919-09）

> 产出角色：core.knowledge/core.worker 起草（subagent）。
> 建议插入位置：`versions/1.20.1/docs/10-timewise-archive.md` 文件末尾——现物理末尾为 260919-08 块的「### 未决」列表（末行「FB-2 修复效果未经运行时验证（机制存在性 ≠ 修复效果，#201）」）之后，直接追加下述整块。

---

## 260919-09（2026-09-19）写回终态化通路修复 B1 主攻包（FA-1+FA-3+FB-2+C3）实施 + keep-world n=2 验收——J1 存在性差 520→0 ✅ / J2 VOID 🔍（抖动面反扩张）/ FB-2 去留留拍板

> 承 260919-08 HOOK-4 选项 A 立项；架构 HOOK-1 已批；实施记录 `.investigations/fix-term-260919-09/record-260919-09.md`；
> verdict `.artifacts/fix-term-260919-09/verdict-260919-09.md`（draft，J2 VOID 如实登记）；judge review `.investigations/fix-term-260919-09/review-260919-09-judge.md`（PASS-with-conditions C1-C5）。

### 过程链

- ✅ **P1 ★复核三项（静态一手源 file:line）**：POST 投递点 = SLP.java:127-129 私有 `enqueue`（4 参重载唯一）；Mixin:540-744 逐行复核 PASS（C3 前提②闭合，size=1 非 timedOut 无需新 mixin）；FB-2 新增保真面 = `excludeBlocks` guard 复刻（SLP:175，超出设计文案，登记待 judge——judge #7 评「语义更贴 vanilla，方向正确，认可待用户」）。
- ❌→✅ **方案变更（编译轮否决）**：@Invoker 方案被编译否决——mapped jar 中嵌套 enum `Stage` 对非 nest-mate = **private**（yarn 文本源显示包私有，与 jar 实际访问控制不符；@Invoker 目标解析失败系同因级联）。**替代 = Fabric access widener**（coreswap.accesswidener：Stage 类 + enqueue 4 参加宽）+ loom accessWidenerPath 注册；WgLightPost 直调公有 enqueue，accessor/第二 mixin config 撤销。已知边界（idk）：Forge+Connector 载体 AW 应用未验证（未加宽 → loud IllegalAccessError，可判别非静默）。
- ✅ **实施落盘**：WgLightPost 新建 + ServerLightingProviderMixin（FB-2 插入 + POST 任务体 setLightOn→releaseTicket→complete + 失败臂对称摘票 + legacy 内联路同构 + hook armed 自证 postF/latesubmit/fb2/wbprobe）+ LightDomainBatch（C3：LATE_SUBMIT=merge + RESEALED 计数 + SEALED_KEYS FIFO cap 65536 + excludeBlocks map）+ build.gradle 三条映射（post-finalize/latesubmit/wbprobe，B6-1 门）。
- ✅ **编译证据**：`gradle :compileJava --rerun-tasks`（GRADLE_USER_HOME=工作区 .gradle-home——#13 家族再现：默认 home 下 native-platform.dll 加载 fail）BUILD SUCCESSFUL rc=0；mixin AP 无 Cannot find target method 警告；Rust 零改动（dll sha cc4e39fe 不变，#202 无漂移连锁）。
- ✅ **工具门缺陷修复**：check_switch_mapping.py RE_VMARG 字符类补连字符（`-D` 名带 `-` 被截断 ORPHAN 误报，首碰撞 post-finalize；:74，逆 = 单行 revert）。门禁报告级 rc=0；1.20.1 缺口 14→13。
- ✅ **验收采集**：keep-world 三臂 vanilla-f0 / fix-domain-f1 / fix-domain-f2（n=2，全臂 SELFCERT PASS，region 18 文件各 ~63MB 归档，dll sha 同）。

### 验收读数（三臂数字）

- ✅ **J1 存在性差 PASS**：两 fix 臂 snap 9450/9450 common、new=0 gone=0——**520 unlit → 0**（C3 消二次 grace + FA-1 ticket 兜底生效；lit 总数恒 9450，VOID 硬线未触）。
- ❌→🔍 **J2 d_self 收敛 VOID**：f1×f2 jitter **7660** sections（基线 6453，**+18.7%**）；diff-nibbles **4,266,861**（基线 3,354,675，**+27.2%**）；O1 形态门机械 PASS（band 99.57%/interior 0.43%）但总量反涨。judge C-2：预登记阈值口径混写（chunk 级 2185 × section 级比较器），**同口径替代执行**；judge 独立复算 **chunk 级 f1×f2 值差 2643 > 2185**——双口径同证 VOID，结论稳健。机制 = FB-2 补传播使边界进入 vanilla 增量传播体系，传播消化次序 run 级非确定使值差面扩张（→ workflow-patterns **#204** 草稿：值收敛 ≠ 过程确定性）。
- 🔍 **J3 d_cross 灰区单列**：域臂 vs vanilla 4049/4056 chunks（两 run），与基线总账 4223 同量级略低——与 #150 预测「520 换形为 lit 值差」一致；恒定面（2038）归属仍未裁决（档③前置未解除）。
- ✅ **J4 S3 窗探针 PASS**（pre/post 3703/3703 成对、inline=0 负自证；vanilla 臂 wb 全 0）✅ **J5 RESEALED PASS**（=8 两臂同值；464 task degraded=0）✅ **preconditions 全 PASS**。

### judge 条件（PASS-with-conditions C1-C5）

- **C-1（MUST）**：`.artifacts/index.yaml` 补 fix-term-260919-09 条目（MUST-FIX，产物未入主索引）。
- **C-2（MUST）**：verdict「预登记读法机械执行」表述改为「预登记阈值口径混合，按 §9.7 同口径替代执行」+ 补记 chunk 级 d_self=2643 双口径同证。
- **C-3（SHOULD）**：verdict 覆盖面补「失败臂对称摘票 = 默认（postF=off）臂行为 delta（泄漏→不泄漏，防御性收窄）」+ 基线 r3/r4 seed-by-log-only 降级档声明随行。
- **C-4（SHOULD）**：record §2 表补第二 mixin config「已撤销，见 §6.1」标注 + SELFCERT `degraded` 键义注释（= 464 行 degraded=0 计数）。
- **C-5（可选）**：派 knowledge subagent 起草 #204（已产出草稿，本块知识库项）。

### 结论与未决

- 主攻包整体 = **1/2 目标达成**：机制存在性成立（J4/J5），520 存在性差消除（J1），但 FB-2 抖动收敛预期被运行时证据反向削弱（J2 VOID，b2 §2「确定性收敛」论证被取代——见 12-lighting.md 本块小节 §15.4 取代声明）。J2 VOID 触发 PI-1 断链，evidence saturation 计数重置（新数据层证据）。
- 🔍 **FB-2 去留留用户拍板**（重大方向决策）：① 回退 FB-2 半包（删行即回退，无开关）② 档③先判别 ③ 接受抖动换存在性。2038 恒定面归属仍未裁决（档③ E-3a 挂起 key=c1-e3a-verdict 未解除）。
- 📌 知识库：workflow-patterns **#204** 草稿 + 时间线本块 + 12-lighting.md 本块小节（subagent 三件草稿 `.artifacts/fix-term-260919-09/knowledge-draft-*.md`）→ 主会话应用。
