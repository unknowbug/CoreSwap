# 知识库草稿：fanout-writer-260904-06 轮可复用判据（260904-06）

> 状态：**draft**（subagent 产出草稿，待主会话应用 + 验证；正文 knowledge/ 未改动）。
> 时间锚：git `9157191 2026-09-04 16:46`（真实提交时间戳，非 AI 推断）→ 标签 260904-06。
> 来源：`.investigations/lossless-accel/fanout-writer-260904-06/`（b1-surface-oob-writer.md §8-§10 + R1-R8、review-judge-260904-06-writer.md C1/C2、convergence-260904-06.md）+ `.artifacts/lossless-accel/writer-verdict-260904-06.md`。
> 价值门核对：四条均为「再遇到不想重新想一遍」级判据（判错方法/反模式/可复用判据）→ 高价值，必记。

---

## 载体分配总表

| # | 内容 | 目标载体 | 形态 |
|---|------|---------|------|
| D1 | env 判别实验的生效证据必须用行为证据 | knowledge/discovered/workflow-patterns.md | 新条目 **发现 #37** |
| D2 | 交接结论廉价验证第二例实证（NEXT_SESSION「现役 dll 早于修复」被推翻） | knowledge/discovered/workflow-patterns.md | **#36 补充案例**（非新条目，按既有条目内追加惯例） |
| D3 | worker 判读源码语义前必须核 env 默认值方向（=0 反转） | knowledge/discovered/workflow-patterns.md | 新条目 **发现 #38** |
| D4 | 参照自身缺陷辨识——对照基线要对 vanilla 校验 | knowledge/discovered/workflow-patterns.md | 新条目 **发现 #39**（#16 对照基线归因法的对偶面） |

说明：四条全部落 workflow-patterns.md。D1/D2/D3 是判别实验/残差归因/交接验证类工作流判据，D4 是残差归因方法论对偶面——均非构建工具坑（build-tooling）或语言惯用法（compiler-idioms），不硬塞错载体。

---

## D1 → workflow-patterns.md 新条目（追加在 #36 之后）

```markdown
## 发现 #37: env 判别实验的生效证据必须用行为证据——常规日志行不能当 env 开关生效证据（260904-06）

- **发现时间**：260904-06（锚 git 9157191）。**发现者**：judge（fanout-writer-260904-06 审查 C1）+ worker b1 回填。**置信度**：candidate（单轮实锤 + 与 #14/#20 家族机制同源）。**module**：通用方法论（判别实验设计 / env 门控证据）。
- **来源定位**：`.investigations/lossless-accel/fanout-writer-260904-06/review-judge-260904-06-writer.md` §1.3-C1 + b1-surface-oob-writer.md §8【judge C1 补正】（`[Mixin] buildSurface skipped` 各臂均打，NoiseChunkGeneratorMixin.java L95）。

### 观察（现象）

E1 臂（WG_SKIP_SURFACE=1 跳过 Rust SURFACE）的 skip 生效证据最初引用 log 行 `[Mixin] buildSurface skipped`——经核该行是 mixin 常规 cancel 日志（CppBridge.enabled 即打），E3 各对照臂日志同样满屏此行，与 env 开关是否生效零相关。E1 的 env 激活实际只有行为旁证（列清空 + 243/111 方向翻转）。

### 证据

- 同一行日志在开/关两臂都出现 → 该行对「开关生效」判别力 = 0；
- 真正的生效证据 = 只有该 env 才改变的输出差异：①(195,199) 列 dirt/sand/gravel 全清空；②vs C++ 臂 OOB 族方向翻转（243/111，C++ dirt 留 E1 stone）。

### 根因（机制）

env 开关的常规路径日志（mixin cancel、stage 已跑类输出）描述的是「代码路径存在」，不是「本次运行走了开关分支」——两者在被门控的分支上不可区分。与 #14（noise-only 判据看存档内容非开关日志）、#20（死参数制造假判别）、#32（daemon 吞 env 死同值）同族：判别实验的自变量生效证明必须独立于两侧共有输出。

### 如何利用

1. **env 判别实验的生效证据 = 找只有该 env 才改变的输出差异**（列清空 / 方向翻转 / 数值位移），日志行只作辅助；引用日志作证据前先核「该行在关闭臂是否同样输出」。
2. **预防性改造**：给 env 门控的关键开关分支加独立一次性日志（如 Rust skip 分支加 `[SURFACE-SKIP] handle=… mask=…`，worldgen_handle.rs L609-626），使后续轮的生效证据可直接引用日志而非行为反推。
3. 家族索引：#14（探针阶段同源性）/ #20（死参数）/ #32（daemon 吞 env）/ 本条（生效证据行为化）——判别实验四件套：自变量真变、对侧真静默、env 真达、生效有行为证据。
```

---

## D2 → workflow-patterns.md #36 条目内追加补充案例（按条目内「### 补充案例」惯例）

```markdown
### 补充案例（260904-06，第二例实证：交接结论廉价验证）

fanout-writer-260904-06 轮：NEXT_SESSION 交接前提「现役 dll 早于 stage-skip 修复」被一轮廉价验证推翻——
`dumpbin /exports`（dll 含 wg_set_flags = 修复在线）+ git 提交时间戳（修复 b611fcb 09-02 01:17 < dll 构建 09-03 23:47）。
即三元组判据第 ③ 维（构建时间 vs 修复时间）在交接验证场景的再次命中：交接文档的方向性前提（§16.3 宿主交接验证 / AGENTS 交接结论验证纪律）用「工具查询 + 时间戳对账」一轮即可证伪，且证伪后果是避免了整轮基于错误前提的实验设计。
来源：convergence-260904-06.md「NEXT_SESSION 前提推翻」+ writer-verdict-260904-06.md 结论 2。
```

---

## D3 → workflow-patterns.md 新条目

```markdown
## 发现 #38: worker 判读源码语义前必须核 env 默认值方向——「=0 反转」型 env 按「默认关」读会造死参数假判别（260904-06）

- **发现时间**：260904-06（锚 git 9157191）。**发现者**：fan-out worker b1 误读 + judge R1 取代链捕获（§15.4）。**置信度**：candidate（单轮误读实证，E2' 实测零差佐证修正后口径）。**module**：通用方法论（源码判读纪律 / 判别实验设计；#20 家族）。
- **来源定位**：`.investigations/lossless-accel/fanout-writer-260904-06/b1-surface-oob-writer.md` §9.0-R1（原 §2.1「WG_EST_SHARED 默认关」→ 实际默认开，`=0` 才关；worldgen_handle.rs L559 注释 + L563）+ convergence-260904-06.md。

### 观察（现象）

b1 worker 依据「变量名含 shared、默认初始化分支在独立重扫路径」推断 WG_EST_SHARED 默认关，E2 实验方案按「=1 反转开启」设计。实际该 env 默认开（env_enabled 语义），`=0` 才关——若按原方案跑 E2，实验臂与默认臂同值，产出「est 路径无差异」的死参数假判别（#20 家族），且方向恰好被主会话执行前核对拦下（R1 取代记录）。

### 证据

- E2' 按「=0 关」口径重跑：关/开两臂导出 sha256 逐字节相同（2ff71249…）——两路径语义同一为真结论，但默认方向若读反，同一实验会得出「开了也无效」的错误表述并污染后续 est 嫌疑链。

### 根因（机制）

Rust 侧 `env_enabled("WG_X")` 类开关默认值 = 未设即开，「默认关」直觉来自 CLI flag/feature-gate 惯例；worker 静态判读时以命名直觉替代了对 env 读取 API 语义 + 默认值字面量的核对。#20 判的是「自变量真被改变」，本条补其上游一步：**实验设计者对自变量语义（方向/默认值/反转点）的判读本身要先验证**。

### 如何利用

1. **A/B env 实验设计前三查（grep 源码，一轮完成）**：① env 读取 API 语义（env_enabled=默认开 / env_var+parse=默认看代码）；② 默认值方向（未设时走哪条路径）；③ 源码注释中的「=0 反转」「legacy, default on」类标注（如 worldgen_handle.rs L511-515 注释模式）。
2. worker 产出含 env 实验模板时，主会话执行前 MUST 核对默认值方向（本轮 R1 即此核对救命）——命令委托契约的「只执行不解读」不含「不核对实验前提」。
3. 家族索引：#20（死参数假判别）/ #32（daemon 吞 env）/ #25（常量追取值源头）/ 本条（env 默认方向判读三查）。
```

---

## D4 → workflow-patterns.md 新条目

```markdown
## 发现 #39: 参照自身缺陷辨识——「mod≠参照」不等于「mod 错」，对照基线自身要对 vanilla 校验（260904-06）

- **发现时间**：260904-06（锚 git 9157191）。**发现者**：fan-out worker b1（R4→R6 取代链）+ judge。**置信度**：candidate（单轮定点实证：biome 七点同 + vanilla 语义核对）。**module**：通用方法论（残差归因 / 对照基线校验；#16 对照基线归因法的对偶面）。
- **来源定位**：`.investigations/lossless-accel/fanout-writer-260904-06/b1-surface-oob-writer.md` §9.4-R4 → §10.1-R6（C++ -biomeDump (244,-60..-54,244) 七点全 = deep_lukewarm_ocean，与 mod 一致；Java VanillaSurfaceRules 该 biome 地板 = sand）+ convergence-260904-06.md「m3 证伪」。

### 观察（现象）

(244) 列 gravel→sand 单列差异，第一归因是「mod biome_at 分叉写错」（m3）。证伪路径：C++ -biomeDump 七点 biome 与 mod 全同（deep_lukewarm_ocean）→ 分叉不在 biome 输入；再对 Java vanilla 语义核对：deep_lukewarm_ocean 地板规则命中 mr2(sand)，mod 的 sand 是**正确输出**——是 C++ 参照（block_probe surface.h）在同 biome 下未命中 sand 条目、落到 gravel fallback，**参照自身偏离 vanilla**。

### 证据

- biome 输入一致性：cpp-biomedump-244col.txt 七点全同；
- 规则分支：surface_rules.rs L1030-1033（warm-ocean StoneDepth-floor → mr2 sand）在 C++ surface.h 未正确命中（R6，C++ 侧缺陷待查）；
- 「mod 输出 = vanilla 正确值」的最终定案留一个廉价实验：vanilla 列对照（writer-verdict 结论 3 已挂新线首实验捎带）。

### 根因（机制）

对照基线归因法（#16）默认「参照 = 真值」，残差单向归因到 mod。但本项目参照是**自己复刻的 C++ 探针/实现**，它与 mod 一样可能偏离 vanilla——「mod 与参照不同」只证明两实现分叉，方向裁决（谁对）必须引入第三极（Java vanilla 语义/源码）或对参照本身做 vanilla 校验。#16 核「差异是否与 X 相关」，本条核「参照是否有资格当真值」——同一方法的输入端校验。

### 如何利用

1. **单列/定点差异归因前双问**：① 两臂输入（biome/seed/坐标口径）是否真一致（先排除输入差）；② 参照在该点的输出对 vanilla 语义是否正确（Java 源码/参照表核对）——两问都过才有资格说「mod 错」。
2. C++ 探针的 surface/规则类实现与 Rust 主线是两套代码（#36 执行体不同源），参照缺陷要独立立缺陷单，不随 mod 课题销案。
3. 家族索引：#16（对照基线归因法——本条其对偶面）/ #33（载具可比性）/ #36（执行体不同源）/ #3（块级真相）。
```

---

## INDEX.md 对应追加行（追加到文末追加记录区）

```markdown
> 260904-06 追加：workflow-patterns 新增**发现 #37/#38/#39**（env 判别生效证据必须行为化——常规日志行不作 env 开关证据 + 开关分支加独立一次性日志；worker 判读源码语义前核 env 默认值方向——「=0 反转」型误读造死参数假判别，#20 家族；参照自身缺陷辨识——「mod≠参照」≠「mod 错」，对照基线自身要对 vanilla 校验，#16 对偶面）+ **#36 补充案例**（交接结论廉价验证第二例：NEXT_SESSION「现役 dll 早于 stage-skip 修复」被 dumpbin /exports + git 时间戳一轮推翻）。
```

同时 INDEX.md 分类表 workflow-patterns 行的说明列追加：`env 判别生效证据行为化（发现 #37）、env 默认值方向三查（发现 #38）、参照自身缺陷辨识——对照基线对 vanilla 校验（发现 #39）（均 260904-06）`。

---

## 应用顺序建议（主会话执行时）

1. workflow-patterns.md 末尾（#36 之后）依次追加 D1（#37）、D3（#38）、D4（#39）三个新条目；
2. #36 条目「教训 / 可复用判据」小节之后追加 D2 补充案例块（保持原 589 行正文不改）；
3. INDEX.md 文末追加记录区追加上节追加行 + 分类表说明列补句；
4. 应用后跑格式一致性抽查（条目六要素齐全：发现时间/发现者/来源定位/置信度/module/观察-证据-如何利用）。

## 自检清单核对（SUBAGENT-KNOWLEDGE-GUIDE §四）

- [x] 价值门：四条均高价值（判错方法/反模式/可复用判据），无一次性结论混入；
- [x] 数字/行号/哈希全部来自主会话提供的落盘产物（b1/judge/convergence/writer-verdict/cmd-output），无编造；
- [x] 根因为机制层（mixin 日志描述路径存在≠开关分支 / env_enabled 默认开语义 / 参照与真值无传递关系）；
- [x] 被取代假说（m3、默认关读法）保留并标注 R1/R6 取代指针，未删改；
- [x] 载体正确：全部通用可复用 → workflow-patterns.md；无错塞 build-tooling/compiler-idioms；
- [x] 只产出草稿，knowledge/ 正文未改动。
