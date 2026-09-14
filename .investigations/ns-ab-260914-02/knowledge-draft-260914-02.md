# knowledge-draft-260914-02 —— ns/section A/B 配对块知识库草稿（subagent 产出，主会话应用）

> 来源：`.investigations/ns-ab-260914-02/record-260914-02.md`（candidate，judge review-001 PASS-with-conditions，C1/C2 已应用）
> 价值门分流：高价值 = discovered 新发现（build-tooling #150）；中价值 = #103/#24 补充案例（简记，不占新号）+ INDEX 行 + 时间线块；低价值（不入知识库）= 本批具体数值（只留 record）。
> 编号核对：build-tooling.md 当前最高 #149 → 新条 **#150**；workflow-patterns 当前最高 #146 → 本块用**补充案例行**（不占新号）；INDEX 尾 = 260914-01 行 → 本行接后；时间线尾 = 260914-01 块 → 本块接后。

---

## 产出 1：knowledge/discovered/build-tooling.md 追加（新发现 #150）

---

## 发现 #150: 同构建态单变量 A/B 的**自证门 + 流程纪律**判据——运行时 flag 覆盖实现零重编 A/B（`-Dcoreswap.bulkwb=0`，消费点一手核对）+ 正负成对计数 + 三条过程错误（判据未预登记 #112 违例 / 自证脚本统计字段双重重置 / destroy 时序汇总行）；冒烟臂一条暴露两个假设错误（260914-02）

- **发现时间 / 发现者 / 置信度 / module**：260914-02；主会话（A/B 驱动 + 保守读法补救）+ judge subagent（review-001 三源核对，独立复算全部数字 + C1 措辞必改）；**candidate**（judge PASS-with-conditions，C1/C2 已应用，confirmed 留人类）；build-tooling / 测量有效性自证门（#118「自证行硬门禁」的 **A/B 配对 + 流程纪律面**；#149 同族配套判据）。
- **来源定位**：`.investigations/ns-ab-260914-02/record-260914-02.md`（§三自证 / §四过程错误五段式 / §五数据表 / §六判读）+ judge `review-001.md`（数字逐格零偏差 + 算术独立重算）+ 原始 log `.tmp/ns-ab-260914-02/nsab1216-*.log`（7 份，不入库在盘可核）；运行台 `.tmp/ns-ab-260914-02/run_nsab_1216_260914-02.ps1`。
- **装置形态（正面）**：A/B 两臂共用同一 dll（sha256=ABD7D889…2131 全程未变、**零重编**），唯差一项 `-Dcoreswap.bulkwb=0` 运行时覆盖（消费点 `java-core/CppBridge.java:642` + `BulkWb.java:90`，主会话一手核对）⇒ **严格同构建态单变量**；交错序 A,B,B,A,B,A 对消顺序效应（#24）；自证 = `[WG-BULKWB]`/`[WG-PERBLOCK]` 计数**正负成对**（bulk 臂 576/0、perblock 臂 0/576）+ 同 workload 前提核对（bulk 三臂 sections_replaced/air_skipped 完全一致 5143/8681）。
- **五段式（错误优先，三条过程错误）**：
  - **错误 1：判据未预登记（#112 违例，本块最大过程错误）**
    - **现象**：主会话在看到数字前没有写死「读法/离群处理」规则。
    - **根因**：流程纪律缺口——预登记在采集设计中被遗漏，#112 同族违例。
    - **定位**：本块收尾自查发现。
    - **修复**：保守读法补救——全部数据呈现、离群候选（首臂 r1）只标注不剔除、任何剔除只作敏感性附注不作主判据；judge C1 修正读法措辞（不剔 r1 的全量保守下界 ~1.32×，剔 r1 两读法 2.04×/2.11× 支撑 ~2.1×，三读法方向一致）。
    - **教训**：**判据预登记必须发生在采集脚本定稿同一时刻**（与 seed 三查同级列入采集前检查单）；事后补判据 = 判据污染，只能降级为附注。
  - **错误 2：自证脚本 pickHit 字段双重重置恒 0（统计 bug）**
    - **现象**：脚本输出 pickHit 恒 0，与 log 实际（每臂恰 2 条 pickup 行）不符。
    - **根因**：脚本编辑残留使 pickHit 计数后被无条件重置语句覆盖（judge 定位到行 64-70 计数、行 72 无条件 `= 0`）。
    - **定位**：主会话手工逐臂扫 stdout+stderr 补验（pickHit=2/臂）；judge 以盘上 log 独立复核一致。
    - **修复**：数值以手工补验为准；脚本本体 bug 留待修正。
    - **教训**：**自证脚本的统计字段本身也要有自证**（对照原始日志手抽一条）；「自证工具输出为零」先怀疑工具再怀疑装置。
  - **错误 3：destroy 时序——汇总行在 stop 后打印，进程退出前提取恒空/恒旧**
    - **现象**：`[WG-BULKWB]`/`[WG-PERBLOCK]` 汇总行在 rcon stop 之后才打印；若在进程退出前提取则恒空/恒旧。
    - **根因**：汇总行挂在 destroy 钩子上，时序在 stop 之后——提取点早于打印点结构性拿不到数据。
    - **定位**：冒烟臂先行一步暴露（同臂同时暴露 wbcontent 缺失 + 本时序问题，修正后方开正式臂；7/7 臂汇总行完整性逐臂核验通过）。
    - **修复**：提取点移到进程退出后；正式臂全部拿到汇总行。
    - **教训**：**采集脚本对「何时有数据」必须按载体打印时序设计提取点**，冒烟臂是暴露装置时序/内容缺失类问题的最低成本手段。
  - **附带 open（瑕疵 3，不阻塞）**：rcon stop ConnectionRefused ×3，judge 补充线索 = 脚本双 stop 结构，机制候选高度可疑但未查（open 登记）；教训（暂定）= 采集块应记录重试次数与退避间隔，防静默截断。
- **判据（可复用）**：
  1. **A/B 性能对比 MUST 严格同构建态单变量**：同一 dll（sha 全程核对）+ 运行时 flag 覆盖（消费点一手核对）实现零重编 A/B，避免「重建引入第二变量」；**每臂正负成对计数自证**（A 臂 [WG-BULKWB]=N/[WG-PERBLOCK]=0，B 臂反向）+ 同 workload 前提核对（写回分项计数逐臂一致）。
  2. **判据预登记与采集脚本定稿同时完成**：读法/离群规则在看数字前写死；漏登只能保守读法补救（全数据呈现 + 离群只标注），剔除只作敏感性附注。
  3. **自证统计字段对照原始日志抽验**：脚本计数 ≠ 证据本身；「自证输出为零」先查工具。
  4. **汇总行/destroy 类数据提取点按打印时序设计**：stop 后打印的数据在进程退出后提取；**正式采集前必跑冒烟臂**（一条即可暴露多个装置假设错误，不入账）。
  5. **首跑预热 ≫ 机器噪声带时，同批配对交错采集是必要条件**：本批首臂预热偏移 ~36% vs perblock 噪声带 ~5.3%（#103 ±10% 同量级）——单臂数字不可信被实证（→ workflow #103/#24 补充案例）。
- **家族索引**：#118（自证行硬门禁——本条为 A/B 配对 + 流程纪律面）、#149（JVM 参数注入自证门——本块 pickup 行 pickup=stderr 同款、手工补验即其「对照 log 抽验」维）、#112（判据预登记家族——本条为其违例首犯实录与补救范式）、#24（顺序效应——交错序对消）、#103（噪声带——预热效应量化）；workflow-patterns #139（等价结论与自证成对——本条为性能读数面）、#20（自证工具静默失效 = 驱动侧假判别同族）。

---

## 产出 2：knowledge/discovered/workflow-patterns.md 尾部追加（#103/#24 补充案例行，不占新号）

---

> **#103/#24 补充案例（260914-02，首跑预热 ≫ 机器噪声带——交错配对必要性实证）**：同构建态单变量 A/B（同一 dll 零重编 + `-Dcoreswap.bulkwb=0` 运行时覆盖，256 chunk overworld、每臂 3 重复、交错序 A,B,B,A,B,A）下：perblock 3 臂 ns/call 极差 **~5.3%**（#103 ±10% 噪声带同量级偏下 → 单臂不可信的判断被本批实证），bulk 全量极差 **~36%** 完全由首臂 r1 预热主导（JIT/类加载候选，剔后极差 **~1.8%**）——**首跑预热效应大于机器噪声带一个量级**，同批配对 + 交错序不是加分项而是必要条件（#24 顺序效应对消）。判读纪律（judge C1 已应用）：全量保守下界（不剔首臂）与剔首臂敏感性读法**分开标注**，量级结论只引由多读法独立支撑者、不得把敏感性读法称作「保守值」。来源：`.investigations/ns-ab-260914-02/record-260914-02.md`（judge review-001 PASS-with-conditions，C1/C2 已应用；confirmed 留用户）；装置面判据 → build-tooling **#150**。时间线 → `versions/1.21.6/docs/10-timewise-archive.md` 260914-02 块。

---

## 产出 3：knowledge/INDEX.md 追加行（接 260914-01 行后）

---

> 260914-02 追加：build-tooling 新增**发现 #150（错误优先）**（同构建态单变量 A/B 自证门 + 流程纪律——同一 dll 零重编 + 运行时 flag 覆盖（消费点一手核对）+ 每臂正负成对计数自证 + 同 workload 前提核对；三条过程错误五段式：**判据未预登记**（#112 违例首犯，预登记必须与采集脚本定稿同时完成，事后只能保守读法补救）/ **自证脚本 pickHit 双重重置恒 0**（统计字段须对照原始日志抽验，「自证输出为零」先查工具）/ **destroy 时序**（汇总行 stop 后打印，提取点须在进程退出后；冒烟臂一条暴露 wbcontent 缺失 + 时序两个假设错误））；workflow-patterns **#103/#24 补充案例**（首跑预热 ~36% ≫ 噪声带 ~5.3%，交错配对必要性实证；全量保守下界与剔首臂敏感性读法分开标注）。来源：`.investigations/ns-ab-260914-02/`（record + judge review-001 PASS-with-conditions，C1/C2 已应用；confirmed 留用户）。时间线 → `versions/1.21.6/docs/10-timewise-archive.md` 260914-02 块。

---

## 产出 4：versions/1.21.6/docs/10-timewise-archive.md 260914-02 时间线块（接 260914-01 块后）

---

## 260914-02（实际 2026-09-14，Get-Date 锚开工时点）：bulk vs perblock 写回分项 ns/section A/B 配对采集（bulk-writeback-260911-05 §9.7 遗留落地）—— **candidate**（judge review-001 PASS-with-conditions，C1/C2 已应用；confirmed 待用户授予）

> 过程产物 `.investigations/ns-ab-260914-02/`（`record-260914-02.md` 主记录 + `review-001.md` judge 三源核对 + 本知识库草稿）+ `.tmp/ns-ab-260914-02/`（7 份 log/err + 运行台，不入库在盘可核）。上游：bulk-writeback-260911-05 §9.7（A/B 配对 + 噪声带控制缺失遗留）+ #103/#24（噪声带/顺序效应）。通用模式 → build-tooling **#150** + workflow-patterns **#103/#24 补充案例**（subagent 草稿 → 主会话应用）。

- ✅ **核心结论（candidate）**：同构建态单变量（同一 dll `ABD7D889…2131` 零重编 + `-Dcoreswap.bulkwb=0` 运行时覆盖，消费点 `CppBridge.java:642`/`BulkWb.java:90`）、256 chunk overworld（seed=417950215108767439，region 2048,2048→2303,2303，forceload post-Done，wbLines=576/臂）口径下，bulk 写回每 chunk 写回分项（ns/call(total)，同仪器）**≈ perblock 的 1/2（~2.1× 快）**——剔首臂两读法独立支撑（最差臂对比 2.04× / 均值 2.11×），全量保守下界 1.32×；绝对值不外推。
- ✅ **自证链**：`[WG-BULKWB]`/`[WG-PERBLOCK]` 正负成对计数（bulk 臂 576/0、perblock 臂 0/576）+ 同 workload 前提（bulk 三臂 sections_replaced/air_skipped 完全一致 5143/8681）；交错序 A,B,B,A,B,A 对消顺序效应。
- ❌→修正 **过程错误（→ #150 五段式）**：① 判据未预登记（#112 违例首犯）→ 保守读法补救（全数据呈现 + 离群只标注）；② 驱动脚本 pickHit 双重重置恒 0 → 手工补验 pickHit=2/臂；③ destroy 时序（汇总行 stop 后打印）→ 提取点移到进程退出后；冒烟臂一条同时暴露 wbcontent 缺失 + 时序两个假设错误（不入账）。
- ⚠️ **§9.7**：载体 = 1.21.6 worldgen1216.dll（零重编）+ destroy 汇总行同仪器；覆盖面 = 单 seed × 256 chunk overworld region × 每臂 3 重复交错；可比性 = 仅同批配对臂可比；calls=576 vs 260913-06 的 625 分母语义差（#132）并列不连线，260913-06 旧单样本**弃用**不进任何对比。
- 🔍 **open**：① rcon stop ConnectionRefused ×3（judge 线索 = 脚本双 stop 结构，未查）；② 首跑预热假设未独立验证（可选 warmup 臂）；③ pickHit 脚本本体未修。
- 状态：**candidate**（judge PASS-with-conditions，C1 读法措辞 + C2 脚本注记已应用；C3 index 登记随升 candidate 由主会话处理），confirmed 待用户授予。
