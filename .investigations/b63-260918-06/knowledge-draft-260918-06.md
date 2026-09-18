# 知识库草稿 —— 260918-06 工作块（subagent 产出，只读 knowledge/ 不动；主会话应用 + 验证）

> 价值门自评：候选 A = **高价值（必记，详写）**——错误链 E1 沉淀的可复用判据/反模式（silently-green 门的运行时校验器形态）；候选 B = **中价值（简记）**——MC 引擎池宽旋钮指纹（javap 一手字节码定论）。两条均过门，无「不写」项。
> 日期锚：record-260918-06.md 头注 Get-Date 2026-09-18 19:25（本草稿不新造日期）。

---

## 候选 A：workflow-patterns 发现 #182（高价值·错误优先）

**拟归文件**：`knowledge/discovered/workflow-patterns.md`（追加至文件末尾，#181 之后）

**正文草稿**：

```markdown
## 发现 #182（最高价值·错误优先）: 映射型校验器必须先验值域封闭性——白名单别名/事件→键归约器对「枚举外世界」恒绿，任何「事件→键」归约器的首条判据 = 未知值必须 FAIL（260918-06，#163/#172 的运行时校验器形态）

- **发现时间 / 发现者 / 置信度 / module**：2026-09-18（260918-06）；CoreSwap 主会话 + knowledge subagent 起草；**candidate**（judge S1 确认落盘、N6 定性同家族；confirmed 留用户）；workflow-patterns / 门禁与校验器设计。
- **五段式（源 = `.investigations/b63-260918-06/b63-errors.md` E1）**：
  - **现象**：B6-3 比对器负向自检注入 `[LIGHT-PATH] path=ghost-path` 后仍输出 `[OK] ⊆ 声明`、rc=0——门禁判据「注入未知值必须 FAIL」首轮不成立，比对器对未知事件值结构性失明。
  - **根因（机制）**：比对器把「运行时事件 → 声明键」建模为**白名单别名表**（ALIASES）；未知事件值不命中任何别名 ⇒ 触达键集为空 ⇒ 与声明的差集恒空 ⇒ **恒 PASS**。「A=B 恒等不证分支生效」（#81 家族）的比对器形态：校验器的枚举覆盖面本身就是判据的一部分，值域封闭性未校验 ⇒ 校验器对枚举外的输入世界恒绿。
  - **定位**：按门禁纪律做负向测试（判别实验必须验证「自变量真被改变」，#20）时 rc=0 与预期 FAIL 矛盾；读 actual keys=0 定位到「未知值零键」路径。
  - **修复**：比对器加值域封闭校验——`path=`/`abi=` 提取值 ∉ KNOWN_VALUES 枚举 ⇒ 直接产出 `light:event.value:<unknown>:<值>` 键（必不在声明集内 ⇒ UNDECLARED ⇒ FAIL rc=1）；修复后负向 rc=1 / 正向 rc=0 双向复测通过（judge N2 亲手复跑实测注入 `path=legacymystery` → `[FAIL] UNDECLARED` rc=1）。
  - **教训（可复用判据）**：
    1. **「基于映射的校验器」必须先校验值域封闭性，再做映射判定**——白名单别名/事件→键归约器对枚举外世界恒绿 = **silently-green 门（#163）的运行时校验器形态**；不能失败的 FAIL 判据 = silently-green 门的负测版（#172/#27 家族，judge N6 定性）。
    2. **任何「事件→键」归约器的首条判据 = 未知值必须 FAIL**——负向测试用**枚举外值注入**，不是「已知值缺位」（后者只测漏报不测失明）；判据表述 = `提取的值 ∉ 枚举 ⇒ 产出 <domain>:event.value:<unknown>:<值> 键`。
    3. 归约器的**空集路径是恒绿通道**：归约为空键集时「差集为空 = PASS」逻辑失效，空集必须按「有未知输入」处理而非按「无违规」处理。
- **家族索引**：**#163**（诊断——silently-green 门本体，本条为其运行时校验器实例）；**#172**（开方——门禁有效性注入式负向测试，本条为其在归约器维的注入实例）；**#81**（A=B 恒等不证生效——同根因家族比对器形态）；**#20**（判别实验自变量真被改变——负向测试的触发依据）；**#27**（断言生效唯一证明 = 负向测试且其假阴性比假阳性更危险——直接先例）。
- **来源定位**：`.investigations/b63-260918-06/b63-errors.md` E1 + `.investigations/b63-260918-06/judge-review-260918-06.md` N2/N6/S1 + `.tmp/260918-06/b63_comparator.py`（KNOWN_VALUES 修复形态，:59/:66-67）。
```

**INDEX 行草稿**（追加到 workflow-patterns 分类行末尾）：

```
+ **#182（最高价值·错误优先）**（映射型校验器必须先验值域封闭性——白名单别名/事件→键归约器未知值零触达⇒差集恒空⇒恒 PASS，silently-green 门（#163）的运行时校验器形态；判据 = 任何「事件→键」归约器首条判据 = 未知值必须 FAIL，负测用枚举外值注入非已知值缺位；#81/#172/#20/#27 家族）（发现 #182，260918-06）
```

---

## 候选 B：algorithm-fingerprints 发现 #27（中价值·简记）

**拟归文件**：`knowledge/discovered/algorithm-fingerprints.md`（追加至文件末尾，#26 之后）

**正文草稿**：

```markdown
## 发现 #27 简记: MC「Main」后台工作池宽指纹——`Util.getMaxBackgroundThreads()` 读 sysprop `max.bg.threads` ∈ [1,255] 缺省 255，喂 `createWorker` 的 `clamp(cores-1, 1, max)`；三个池宽旋钮语义不同不可互引（260918-06）

- **指纹本体**（javap -c 一手字节码，1.20.1 merged jar / yarn 1.20.1+build.10）：`net/minecraft/util/Util.getMaxBackgroundThreads()` 读 sysprop `max.bg.threads`，int ∈ [1,255] 直接返回；越界/非数字 → LOGGER.error + 返回 **255**；未设置 → **255**。唯一调用者 `Util.createWorker(String)`：`clamp(availableProcessors-1, 1, getMaxBackgroundThreads())`；≤0 → direct executor；否则 `new ForkJoinPool(n, factory, uncaughtHandler, async=true)` = MC「Main」后台工作池（Util.java:183 同源，#153）。
- **判定**：`max.bg.threads` **不是死开关**，是 MC 引擎活开关（调低 Main 后台池宽）；B6-1 t4-adjudication §3.3 的 UNRESOLVED 项由此闭合为 LIVE-MC-ENGINE。
- **per-version 事实**：`-PmaxBgThreads` 映射只在 **1.21.6**（build.gradle:144）；**1.20.1 无映射行**（per-version 缺口家族，#168 口径）。
- **池宽旋钮三分不可互引**（判据）：① `max.bg.threads`（MC Main 池上界，[1,255] 缺省 255，clamp 下限 1、目标 cores-1）② `coreswap.execpool`（物理核−2 语义 ≈ logical/2−2 @SMT2，build-tooling #57）③ `ForkJoinPool.common.parallelism`（fjp1，1.20.1 worldgen 主 worker = 专用 FJP，此参数被架空 = 死参数，workflow-patterns #153）——**三者作用于不同池、语义不同，性能归因/调参禁止互相替代**。
- **来源定位**：`.investigations/260918-06/record-260918-06.md` 项 3（反汇编件 `.tmp/260918-06/util_disasm.txt`）。置信度 candidate（一手字节码，Degraded 静态层）。
```

**INDEX 行草稿**（追加到 algorithm-fingerprints 分类行末尾）：

```
+ MC Main 后台池宽指纹——`max.bg.threads` ∈ [1,255] 缺省 255 → createWorker `clamp(cores-1,1,max)`；与 coreswap.execpool（物理核−2）/#153 fjp1 三旋钮语义不同不可互引（发现 #27，260918-06）
```

---

## 价值门判断与定号说明

| 候选 | 价值门 | 拟用发现号 | 备注 |
|---|---|---|---|
| A 映射型校验器值域封闭性 | **高价值（必记）**：错误链 E1 沉淀 + silently-green 门新形态 + 可复用判据 | workflow-patterns **#182**（现末号 #181，INDEX 行 :201 实录） | 五段式完整；judge S1/N6 已点名同家族 |
| B max.bg.threads 池宽指纹 | **中价值（简记）**：算法/引擎指纹，「是什么」从简 | algorithm-fingerprints **#27**（现末号 #26，实测文件头） | build-tooling 侧不需要另立条目——per-version 缺口仅是 #168 既有判据的一个实例，指纹本体归 algorithm-fingerprints；若主会话倾向归 build-tooling（接 #159 后 = **#160**），正文可直接平移，内容不变 |
