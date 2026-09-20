# discovered-draft — workflow-patterns #215 + build-tooling #162（subagent 草稿，主会话应用）

> 应用方式：a 追加到 `knowledge/discovered/workflow-patterns.md` 末尾（现最大 #214，顺接 #215）；b 追加到 `knowledge/discovered/build-tooling.md` 末尾（现最大编号 #161，顺接 #162）。格式照各文件既有条目。

---

## a. workflow-patterns.md 追加

```markdown
## 发现 #215（高价值·错误优先）: 判据计数实现与判据口径漂移——「事件窗」在采集脚本 hint 里退化成全窗计数 → hint 误报 FAIL；判读权威 = 预登记判据文本，hint 缺陷只登记不改判读（260920-06）

- **发现时间/发现者/置信度/module**：2026-09-20（260920-06）；worker 判读（窗口化脚本按 design 口径）+ judge N4 复核 + knowledge subagent 草稿；**candidate**（confirmed 留用户）；workflow-patterns / 判据预登记程序面（#112/#195 家族，hint 维新形态）。
- **来源定位**：`.investigations/k2a-t5t6-260920-06/record-260920-06.md` §6（hint 偏差登记）+ design-260920-06.md §3.2（判据 N 原文口径）+ judge-review-260920-06.md N4/C3；机械证据 = `.tmp/k2a-t5t6-260920-06/formedge-fe2_result.json` `criteria_hint` 字段（`N_FAIL (T5 falsified ...)`）。
- **现象（五段式）**：
  - **现象**：fe2 采集 SELFCERT 全绿、判读实为 N PASS + P PASS，但同轮采集脚本输出的 CRITERIA-HINT 写 `N_FAIL (T5 falsified -> fan-out .b1/.b2 MANDATORY)`——hint 与判读结论相反，且该字符串固化在 result.json 的 `criteria_hint` 字段里，裸读 json 会得出「T5 已证伪」的错误交接。
  - **根因（机制）**：预登记判据 N 的窗口口径是**事件窗**（「`[WBQ]`/`[FP-LIGHT]` 计数窗 = phase=P 之前的全窗」），采集脚本 hint 实现时把计数退化成**全窗（全 log）计数**——P 窗的 1 条合法 light 行被计入 N 窗账面 ⇒ hint 判 N FAIL。机制 = 「判据文本 → 计数实现」的翻译损耗：判据文本的窗口限定词（相对性、以某事件行为界）实现成本高于全窗计数，实现者不自觉地采用了可计算性更好的近似口径，且近似**静默**（hint 输出未标注口径差）。
  - **定位（怎么发现）**：窗口化脚本 analyze_fe2.py 按 design §3.2 口径（以 probe phase=N 行 t=60598.0 / log 位置为分界归窗）独立计数：N 窗目标 chunk light 行 = 0 → N PASS；与 hint 对照暴露口径分歧——「hint vs 判据文本」逐字对读即定位（判读权威核对属判读前置动作，#112）。
  - **修复**：判读以 design 预登记文本为准（本记录即按窗口化口径得出 N PASS，hint 不进判读）；hint 窗口化修复列未决项（judge C3：须先于下一次使用该脚本的采集，防误触发 fan-out）；建议修复时让失效 hint 带自标识（judge info-3），防裸读 json 误判。
  - **教训（可复用判据）**：
    1. **判读权威 = 预登记判据文本，不是任何脚本输出**——hint 是提示性输出（非判据载体），hint 与文本冲突时只登记偏差、不改判读；重跑 hint 不产生新数据层证据（judge N4）。
    2. **「判据 = 计数实现」必须从判据文本逐字推导**：判据文本中的窗口/分母/边界限定词（事件窗、相切点、判读面）在实现时逐条对应代码，禁止用「全窗/全集」等可计算性更好的近似口径静默替代（#112 读法写死在实现维的形态；对照 #211——门绑判读面不绑数据面，本条为其计数实现维姊妹）。
    3. **hint 缺陷只登记不改变判读，但 MUST 限期修复**：失效 hint 固化在 result json 里是交接污染源（#90 家族新位形——「自家产物自带的错误结论」），修复优先级 = 先于下一次同类采集；无法立即修时，判读记录 MUST 显式声明该字段作废。
- **家族索引**：#112（读法写死/禁事后挑读法——本条为其「采集脚本 hint 实现漂移」位形）/ #195（预登记四件套）/ #211（门绑判读面——本条为计数实现维姊妹）/ #212（引用机械产物逐字核对——本条为其「产物内嵌 hint」延伸）/ #90（转引漂移）。
```

## b. build-tooling.md 追加

```markdown
## 发现 #162（最高价值·错误优先）: 门控分发寄生无关开关——多驱动共用注入点时放行门枚举不全 = 新分支不可达 + 历史分支寄生放行；互斥开关负自证（Q5 型）是暴露隐性接线的机械手段（260920-06）

- **发现时间 / 发现者 / 置信度 / module**：2026-09-20（260920-06）；core.worker 错误判读 + judge 三源核对（mixin 放行门 1 行 diff 实证）+ knowledge subagent 草稿；**candidate**（fe2 复跑全链 SELFCERT 绿 + 判据面完整收敛；confirmed 留用户）；build-tooling / mixin 门控接线 + 采集台驱动分支（#156/#158 开关覆盖面家族的「门卫谓词」维）。
- **来源定位**：`.investigations/k2a-t5t6-260920-06/record-260920-06.md` §7（fe1 VOID 五段式 + git 历史核清副产物）+ judge-review-260920-06.md §0（git diff：ServerWorldFormProbeMixin 放行门 `DRIVE` → `DRIVE || GRID || EDGE` 1 行）；证据 = `.tmp/k2a-t5t6-260920-06/formedge-fe1.log`（VOID 原名留档）+ `formedge-fe2.log`。
- **五段式**：
  - **现象**：fe1 run `[FP-ON] ... drive=false grid=false edge=true` 打出（`-Pformedge=1` → `-Dcoreswap.formedge` → getProperty 消费链全通），但 `[FP-EDGE]` 行 **0 条**（init/arm/done 全缺，probe 全 null）——「开关声明生效但行为面零输出」矛盾签名。
  - **根因（机制）**：FormProbe 三驱动（corridor/grid/edge）**共用一个 mixin 注入点**（serverTick 票据驱动分发），注入门的放行谓词只判 DRIVE 族开关——新增 edge 分支只加了 init 打印与 getProperty 消费，**未扩放行谓词**，edge 时序码成为不可达路径。属「开关家族扩展漏改门卫」结构错：B6-1 映射门只保证映射行与消费点存在（必要），不保证消费点内部子分支**可达**（非充分）。历史核清副产物：**260920-05 的 grid 臂实际是寄生在 formdrive 开关下运行的**——该 run 采集脚本同时传了 formdrive=1（run_fullcov.py:35,37），放行谓词恰被 formdrive 满足，grid 时序得以走通；上块 grid 数据面的驱动通路实为 formdrive 门放行而非独立 grid 门（同根缺陷的历史已发面，恰好不坏但属巧合性正确）。
  - **定位（可复用）**：SELFCERT 前置集机械捕获——`edge_init:0` 与 `fp_on_edge:1` 并存 = 「开关生效但行为面零输出」签名；grep 全 log 无任何目标行排除打点丢失；对照 fe2（修复后）同签名消失。**互斥开关负自证（Q4 门关臂 [FP-EDGE]=0 + Q5 驱动互斥 [FP-DRV]=0）把「谁在放行」变成可机械判读的问题**——本例正是 Q5 型门的设计语境下矛盾签名才无歧义。
  - **修复**：放行谓词扩展为 `DRIVE || GRID || EDGE`（mixin 1 行，fe2 采集前合入）；260920-05 grid 寄生事实登记为历史核清项，防止「grid 门独立可用」被当公理续推（交接结论验证纪律）。
  - **教训（判据，MUST）**：
    1. **新增共享注入点的驱动分支 MUST 同步扩注入门放行谓词**——审查清单 = getProperty 消费点（B6-1 管）+ **门卫/放行条件枚举**（B6-1 不管）+ 分支可达性，三件齐才算接线完成。
    2. **互斥开关负自证门（断言「其他开关行=0」）是暴露寄生接线的机械手段**——每个驱动分支独立臂跑时断言其余分支行为行为面 = 0，寄生放行（如 grid×formdrive）在互斥臂下必然显形。
    3. **「开关打了 = 功能跑了」不成立**——SELFCERT 前置集（行为面计数 > 0）是功能确实跑了的第一证据（workflow-patterns #215 同块姊妹：判据面计数也是 hint 不可替代的）。
    4. **历史正结果值得核清「靠哪个门放行」**——巧合性正确（寄生放行但结果恰不坏）与设计性正确在数据面上不可区分，只有接线核清才可区分；防下游把巧合当公理。
- **家族索引**：#156（开关覆盖面/盲区——本条为其「放行谓词」维：消费点存在 ≠ 子分支可达）/ #158（对账工具两段式覆盖——门卫枚举可入其注册表形态）/ #40（mixin 装载面——本条为注入后分发面）；workflow-patterns #118/#172（负自证硬门——Q4/Q5 为其门控接线维实例）、#215（同块「hint ≠ 判据」姊妹）。置信度 candidate。
```

## c. 建议的 INDEX.md 追加行（供主会话参考；格式照既有 `> <块标> 追加：…` 行）

```markdown
> 260920-06 追加（T5/T6 运行时单验）：workflow-patterns 新增**发现 #215（高价值·错误优先）**（判据计数实现与判据口径漂移——预登记判据的「事件窗」口径在采集脚本 hint 里退化成全窗计数 → hint 误报 N_FAIL 且固化在 result json；判据 = 判读权威 = 预登记判据文本非脚本 hint + 「判据=计数实现」从文本逐字推导禁静默近似 + hint 缺陷只登记不改判读但限期修复并自标识；#112 实现维/#211 姊妹）+ build-tooling 新增**发现 #162（最高价值·错误优先）**（门控分发寄生无关开关——FormProbe 三驱动共用 mixin 注入点、放行门 DRIVE-only 枚举不全：fe1 edge 分支不可达 VOID，git 历史核清 260920-05 grid 臂实为 formdrive 开关寄生放行（run_fullcov.py:35,37）；判据 = 新增共享注入点驱动分支 MUST 同步扩放行谓词 + 互斥开关负自证门（断言其他开关行=0）暴露寄生接线 + 「开关打了≠功能跑了」SELFCERT 行为面计数第一证据 + 历史正结果核清「靠哪个门放行」）。来源：`.investigations/k2a-t5t6-260920-06/{design,record,judge-review}-260920-06.md`（judge PASS-with-conditions C1-C3；T5 candidate，confirmed 留用户）；时间线 → versions/1.20.1/docs/10-timewise-archive.md 260920-06 块。
```
