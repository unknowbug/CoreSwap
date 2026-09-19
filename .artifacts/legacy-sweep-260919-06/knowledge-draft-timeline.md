# versions/1.20.1/docs/10-timewise-archive.md 追加块草稿（260919-06 块；主会话应用）

> 追加位置：文件末尾（260919-05 块之后）。格式对齐既有「## YYMMDD（yyyy-MM-dd）标题 + 状态标注 + ✅/❌/🔍/📌 条目」形态。

---

## 260919-06（2026-09-19）旧遗留清单 8 项 sweep（scout 溯源 → 廉价验证/重算 → judge PASS-with-conditions）——T8 双臂对拍判据 FAIL（值面差异 44.69%）+ 多项挂起拍板收口 🔍 candidate（confirmed 留用户）

> 承 NEXT_SESSION 260919-05 开工点 1；架构 `.investigations/000-架构设计/架构计划-260919-06-旧遗留清单验证.md`（已批准）；
> scout 溯源地图 `.investigations/legacy-sweep-260919-06/backlog-map.md`（8 项逐项溯源到首证原文，禁止转抄——#90 纪律；其中 B3 交接线索未命中，实际溯源为 260911-05 方向变更记录）。

- ✅ **T1 R4 上报跟进**：上游复核 merge_index.py sha 16 位 299598f46ebe1269 一致 + 上游末 commit eeebff9（2026-08-08）→ 缺陷仍活；提案状态 draft → **submitted（转交维护侧）**。
- ✅ **T4 260914-02 open①③**：open① 静态坐实 = 驱动脚本含**编辑残留的第二次 `rcon stop`+WaitForExit 块**（judge C2 候选证实，未运行时复现）；open③ 修复 = 删 pickHit 双重重置 + stop 重试可见化（refusedCount 输出）；修复后对当日 log 复算 pickHit=2 与人工补验一致。§9.8 副作用逆登记：脚本直改 server.properties level-seed（逆 = 常量 seed 重写，未还原，环境未漂移）。judge N3 登记：FAIL 分支 stop 后无 WaitForExit（残留进程风险）+ 修复后未实跑整臂回归（下次采集自然覆盖）。
- ✅ **T5 260915-01 后续**：排序表/池范围转抄零漂移（逐行对照）；CP-5 静态参数表（Degraded）= 三处同族缺省全 `max(1, logical/2−2)`（mixin ×2 + Rust adaptive_threads）；4 核机缺省=1 线程为发行面价值基础，**是否立项留用户，本块不立项**；CP-2/CP-4 无新证据不重开。
- ✅ **T2 Scope B/C 重算（worker，Degraded）→ 用户拍板 HOOK-2：B/C 均继续挂起**。产物 `.artifacts/legacy-sweep-260919-06/scope-bc-reassessment.md`：探针现 **19 个**（原计划 16 已过时）；8 逐字节相同 / 10 纯改名（缝面 4-5 个 WgCompat 方法）/ 1 实质差异（BlobProbe stats 修复不对称）；**范围 B 技术面可行属低成本波次**（唯 BlobProbe 出货树前置）；**范围 C 体量远超原计划**——1.20.1 侧 mixin 已长出 ~200 行 exec/P1/FormProbe 分派机器且在现役演进（C-1a），现在抽取 = 活跃演进代码上做等价重构，返工风险高 → 等 exec 形态收口再评估。
- ✅ **T3 B3 余额重算（worker）→ 用户拍板：继续挂起**。产物 `b3-balance.md`：6+3 项中**仍缺 2**（exec 模式 / P1 maxinflight，均 1.20.1 独有 mixin 面）/ 顺带完成 4（指纹门/StallWatch/ca_min/adaptive_threads）/ 作废 3（A1a/A1b/A1c）。不满足关账条件（真缺口在），触发条件 = 1.21.6 上线/并发压测需求。
- ✅ **T6 260918-03 open①②**：open① 静态闭合到分支级（worker 件 `knot-selftransform-static.md`，Degraded）——root cause = **Sponge MixinProcessor 设计性拒绝 mixin 类直接 classload**（IllegalClassLoadError，字节码偏移 ~415-430），经 KnotClassDelegate.java:422-427 包装、BlobProbe.java:46 吞 cause；一手 = fabric-loader 0.15.11 sources + sponge-mixin javap 字节码；推断等级 = 机制方向（定论需 getCause 打点一次）。open② 复核仍成立 → **用户拍板：不动出货树，登记已知限制**。→ build-tooling **#160 草稿**。
- ✅ **T7 global palette 回退采样**：复用 C-1a 现役台一臂（SELFCERT 全绿，n=457 任务满载）→ `fallback vanilla`/`packed-rc`/legacy-fallback 全 **0 行**（log 实 grep）。边界 = 单 seed 单臂 n=1 非「未证不可能」的证伪。**用户拍板：转关账 + @anchor.idk 90 天线（judge 确认后生效）**。
- ❌ **T8 域批值 vs vanilla 双臂对拍：判据 FAIL（值面差异存在）——本块最重项**。
  - 程序合规：判据预登记（criteria-t8.md mtime 18:53）**先于**驱动与双臂采集（18:59/19:14）；双臂 SELFCERT 全绿 + vanilla 臂**负自证硬门**过；VOID 分支 sys.exit(2)。
  - 结果：common 9450/9450（覆盖面一致），**light-hash diff = 4223 = 44.69% → VERDICT: FAIL**（预登记存在性判据，无阈值带）；judge 独立复算逐位吻合。域批臂自身 run 收敛（G3）≠ 对 vanilla 值对齐——两个不同命题，本轮证伪「值面一致」假设；机制归因/课题化超廉价范围，**交用户决策是否立项**（关联 verdict-260917-01:56 显式遗留）。
  - 比较器外观缺陷（诚实登记）：sample 行对 str key 序列切片显示失真，计数/VERDICT 不受影响 → build-tooling **#161 草稿**；量级句「≫0.8%」E1/E2 档位补声明（judge N2）→ workflow-patterns **#100 追加注记草稿**；判据模板化 → **#195 草稿**。
- ✅ **judge（收尾 MUST）：PASS-with-conditions（N1-N7）**——三源核对（产物快照 + git diff + 验证记录含独立复算）；N1（binding）= 本块产物未登记 .artifacts/index.yaml（主会话补）；N2 = E1/E2 档位声明（已应用）；N3/N4/N5 建议（登记/补登记/一行修复）；N6/N7 登记即可。
- 📌 **知识库**：subagent 草稿三份 `.artifacts/legacy-sweep-260919-06/knowledge-draft-{discovered,index,timeline}.md`（build-tooling #160 最高价值 + #161；workflow-patterns #195 + #100 追加注记）→ 主会话应用。
- 📌 过程产物：`.artifacts/legacy-sweep-260919-06/`（record + scope-bc + b3-balance + judge-review）+ `.investigations/legacy-sweep-260919-06/`（backlog-map + knot-static）+ `.tmp/`（判据/驱动/比较器/diffs 原始证据）。
- 🔍 **未闭合/下一步**：T8 值差剖析是否立项（用户）；N1 index.yaml 补登记；N3 下次采集首跑验证 refusedCount 与进程退出；T7 idk 90 天线随 judge 确认生效；open②（warmup 臂）续挂。
