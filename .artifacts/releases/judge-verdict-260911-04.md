# judge-verdict-260911-04 — CoreSwap 1.0.29 发版决策与工单（judge 原文归档）

> 审查执行：judge subagent（只出意见不改文件），260911-04。本文件 = 审查意见落盘归档（条件 N-2 应用）。

## 节点 ①：exec 缺省开转正 + 池宽维持「物理核−2」拍板（重大方向 MUST）

**结论：PASS**

- 置信度合法性 ✓：fps-verdict L29-30 AI 侧标 candidate、confirmed 明示「用户已拍板——AI 侧仅记录，不授予标签」，无代授越权。
- 拍板证据链完整：260911-03 三臂 A/B（exec-def wall −12.5%/fill −32%/cpu −6.8%，指纹 4140×3 diff=0，`[WG-EXEC]` 自证 + 零 env 缺省开短路双向证据）+ C2/C4 已应用（`d20aac1` 在 HEAD 链）+ 用户实机直接确认。C1 已降级为「单 run 趋势」声明。
- 池宽语义核对 ✓：现缺省 `logical/2−2`（SMT2 下 = 物理核−2，与引擎 adaptive_threads 同源）——零代码改动主张与 HEAD 改动面一致，无隐藏改动。
- C3 核销路径正确（转工单 §5）；maxinflight 悬置决策随 exec 转正自然关闭。

## 节点 ②：RELEASE-1.0.29.md 工单出单（MUST）

**结论：PASS-with-conditions**

独立重算：jar sha256 `b057fda2…312239`、jar 内 `native/worldgen.dll` 与 `target\release\worldgen.dll` 均 `dd3b645f…6765d`——**三元组 MATCH**（judge 亲算）。

其余核对：git HEAD=`cf9fa54` ✓；perfprofile 全树 0 残留 ✓；C1-C4 全核销（C3 三项落 §5）✓；时间链自洽（源 commit → 重编 18:55:17 → sha 重算；引擎 dll 12:53:30 早于 Java 改动，与 Rust 零改动一致）✓；撤单 1.0.28 关系清楚 ✓；schema/changelog 与事实一致、无夸大 ✓。

### 问题清单
- **N-1（阻塞投递）**：INDEX.md 未登记 1.0.29 → 补登记行。
- **N-2（证据链）**：fps-verdict 引用的本文件不存在 → judge 意见落盘至此路径。
- **N-3（措辞，不阻塞）**：「重编 18:55 > 删除 commit」严格不成立（jar mtime 18:55:17 早于 commit 18:55:41，24 秒）——改为「同分钟、commit 落盘于构建后」，mtime 不可信以内容指纹为准（契约 §5 先例）。
- **N-4（建议，不阻塞）**：§4 补「关联 bug 卡回填：不适用」声明行。

## 总结论

**PASS-with-conditions**：放行前置 = N-1 + N-2；N-3/N-4 顺手修正。补齐后按契约告知用户转交 Maint 会话（Maint 侧仍有第二重独立 hash 校验 + 人工确认门）。

## 条件应用记录（主会话，260911-04）

- N-1：INDEX.md 已登记 1.0.29 pending 行 ✓
- N-2：本文件即落盘产物 ✓
- N-3：工单 §2 时间链措辞已改「同分钟、commit 落盘于构建后；sha 与源状态 MATCH 以内容指纹为准」✓
- N-4：工单 §4 已补「关联 bug 卡回填：不适用」行 ✓
