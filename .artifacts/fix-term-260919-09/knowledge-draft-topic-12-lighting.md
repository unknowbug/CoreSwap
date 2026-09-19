# 草稿 3/3：versions/1.20.1/docs/12-lighting.md 主题篇追加小节（260919-09）

> 产出角色：core.knowledge/core.worker 起草（subagent）。
> 选题说明：docs/ 目录主题篇中光照相关 = **12-lighting.md**（07-block-pipeline.md 为方块管线，光照结论已归口 12 篇 + 07 篇形态审计节；本块属光照写回终态化通路，归 12 篇）。
> 建议插入位置：`versions/1.20.1/docs/12-lighting.md` 文件末尾——现末节为「## G3 漂移基底归因（260917-01，confirmed）」（含 §15.4 取代声明与证据指针），在其后追加下述整节。追加式不覆盖，原文不改。

---

## 写回终态化通路修复 B1 主攻包验收——520 存在性差 520→0，但 FB-2 补传播使 run 间抖动 +27%（J2 VOID，260919-09）

> 状态：candidate（verdict-260919-09 draft；J2 VOID = 机械信号 PI-1 断链；FB-2 去留属重大方向决策留用户拍板）。
> 载体与口径（§9.7）：E1（同构建态同批三臂），keep-world n=2 单 seed overworld，seed 8576294172403134396 × dll sha8 cc4e39fe × snap/light-json 比较器；section 级 jitter 与 chunk 级 2185 不可直比，比较用同口径（6453 vs 7660 sections）+ judge 独立复算 chunk 级（2643 vs 2185）双口径同证。基线 r3/r4 seed 身份为 log-only 降级档（judge C-3 随行声明）。

### 结论

B1 主攻包（FA-1 ticket 兜底 + FA-3 POST 投递 + FB-2 写回后补 propagateLight + C3 消二次 grace）验收为 **1/2 目标达成**：

- ✅ **存在性差（C3+FA-1 主攻面）已消除**：两 fix 臂 snap 9450/9450 common、new=0 gone=0——260919-07 分账中的 **520 chunk「vanilla lit / 域臂 unlit」存在性差 → 0**（lit 总数恒 9450，VOID 硬线未触）；行为化自证在案（J4 [LIGHT-WB] pre/post 3703/3703 成对、J5 resealed=8、464 task degraded=0、preconditions 全 PASS）。
- ❌ **值差抖动面（FB-2 主攻面）未收敛反扩张**：f1×f2 run 间 jitter 6453 → **7660 sections（+18.7%）**、diff-nibbles 3,354,675 → **4,266,861（+27.2%）**；O1 形态门机械 PASS（band 99.57%/interior 0.43%）但总量反涨。机制：FB-2 使 chunk 边界进入 vanilla 增量传播体系后，**传播消化次序的 run 级非确定**使值差面扩张——「520→0 但抖动 +27%」并存（→ workflow-patterns #204：值收敛 ≠ 过程确定性，更贴 vanilla 机制可系统性放大 run 间方差）。
- 🔍 d_cross 灰区单列：域臂 vs vanilla 4049/4056 chunks（两 run），与基线总账 4223 同量级；恒定面（2038）归属仍未裁决（档③前置未解除）。

### §15.4 取代声明（supersedes 双指针，原文不删不改）

- **取代** 260919-08 fan-out .b2 §2「FB-2 确定性收敛」论证的**预期部分**（推翻理由一行：FB-2 补传播的「vanilla 原生收敛」预期经 B1 keep-world n=2 运行时 A/B 证伪——jitter 6453→7660、chunk 级 2643>2185 双口径同证 VOID，传播消化次序 run 级非确定使值差面扩张而非收敛）；.b2 §2 原文（.investigations/fix-term-260919-08/ 内 b2 产物）不改。
- 连带：260919-08 设计文档 design-260919-08.md 中 FB-2 行「修复效果待 B1 采集验证」的悬置至此解悬为**负向读数**（#201 三级分账：机制存在性 ✅ / 静态针对性被运行时证据反向削弱 / 修复效果 ❌）。
- 取代记录正式文本：`.artifacts/fix-term-260919-09/verdict-260919-09.md` §结论 + `review-260919-09-judge.md` #10。

### 实施备注与遗留

- 方案变更：@Invoker 被编译否决（mapped jar 嵌套 enum `Stage` 对非 nest-mate = private，yarn 文本源包私有 ≠ jar 实际访问控制）→ **Fabric access widener** 替代（remap 安全）；idk：Forge+Connector 载体 AW 应用未验证（loud IllegalAccessError 可判别）。
- 判据口径教训：预登记阈值（chunk 级 2185）× 比较器（section 级）口径混写，替代执行 = 同口径替代基线 + judge 独立复算双口径同证（judge C-2，表述须如实为「同口径替代执行」）。
- 🔍 未决：FB-2 去留三选项（回退半包 / 档③先判别 / 接受抖动换存在性）留用户拍板；2038 恒定面归属（终态化固化 vs 确定性语义差）待档③ E-3a（挂起 key=c1-e3a-verdict）；过程链全文 → 10-timewise-archive 260919-09 块。
