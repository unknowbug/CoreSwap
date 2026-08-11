# 审查意见：性能回归修复收尾提交（Phase 3 交付）

> 角色：core.judge（只出意见，不改 status）· 日期：2026-08-12
> 审查对象：本 session 的 3 个新提交（HEAD~3..HEAD）——
> 1. `6e2c7ea` fix(perf): FlatCache current-chunk context binding - closes rebuild 168x explosion（代码）
> 2. `bc5f5b0` docs(perf): record FlatCache regression root cause + fix closure in knowledge base（知识库）
> 3. `c0ac286` docs(perf-rework): archive investigation artifacts incl. open-issue uncertainties（产物归档）
> 三源核对：① git 历史/工作区（.git/logs/HEAD、refs/heads/master，git 命令因环境权限被拦截，改用 git 元数据 + 本会话历史中的 git 原始输出）② 应用版代码（density.h / worldgen_api.cpp / CMakeLists.txt / bench_chunks.cpp）③ 落盘数据（cmd-output/*.txt）与知识库（07 篇 / 10 时间线 / discovered #10 / index.yaml / reasonix.toml）
> 推荐状态：**主结论通过**（代码提交与已审查通过的实现一致；知识库数据与落盘一致；产物归档完整）；附 3 项轻微建议（非阻塞）

---

## 逐提交 verdict

| 提交 | 范围 | verdict | 依据 |
|---|---|---|---|
| 6e2c7ea（代码） | density.h + worldgen_api.cpp + CMakeLists.txt + bench_chunks.cpp（新增）4 文件 264+/53- | **通过** | 实现与 review-fix-delivery.md 审查通过的版本逐条一致，无遗漏/多余改动（见要点 1） |
| bc5f5b0（知识库） | 07 篇 + 10 时间线 + discovered #10 + index.yaml + reasonix.toml，5 文件 267+/3- | **通过** | 知识库数据与 cmd-output 落盘逐项一致；cold_resume_prune 删除合理（见要点 2） |
| c0ac286（产物归档） | .investigations/perf-rework/（29 文件）+ spec/ + templates/ | **通过** | 调查链完整；剩余课题不确定性已归档；大文件/临时二进制已排除（见要点 3） |

---

## 要点 1：代码提交 6e2c7ea（与已审查实现一致性）

**FlatCacheDF 上下文绑定实现（density.h，逐行核对）**：
- L40-41：`inline thread_local int g_curChunkX/Z = INT32_MIN` ✅ 与审查描述 L35-41 一致
- L724-743 sample：`cx/cz = g_curChunkX/Z（非 INT32_MIN）时用上下文，否则回退 pos>>4`（L729-730，诊断路径回退分支语义保留）✅
- key 组合 L731、slot 命中判断 L732、buildGrid 绑定 slot.cx/cz（L736）✅
- k/l 相对 startBiomeX：`k = (pos.x>>2) - slot.cx*4`（L739-740）✅
- 越界 `return arg->sample(pos)` 直算不重建（L742）✅ **根因消除点**
- 界内查表 `slot.grid[(size_t)l*GRID+k]`（L741）✅
- buildGrid 角点公式 `(chunkX*4+i)*4, y=0`（L774-778）✅ 未改动
- **buildGrid 递归蔓延根除机理**：角点 i=4 → k=4 ∈ [0,5) 命中本网格（上下文绑定，非 pos 推导）→ 不触发邻居 chunk 重建 ✅

**RAII 恢复（worldgen_api.cpp L590-596）**：`struct CurChunkGuard{ 构造设 g_curChunkX/Z；析构恢复 INT32_MIN }`，fillOneChunkCore 入口声明（L596）——审查修正项 ②「RAII 恢复 + 诊断路径回退声明修正」**已闭环** ✅。CoreSwapPool（L939+）为既有改动，本次提交仅新增 fillOneChunkCore 上下文设置（RAII）✅。

**bench_chunks.cpp 完整**：A 段池并行批提交（threads 1/8/22/0）+ B 段模拟实机 JNI（T worker 各调 count=1），含 warmup/median/参数化，5523 字节 ✅。CMakeLists.txt L72-74 注册 `add_executable(bench_chunks src/bench_chunks.cpp)`（本次新增；无 pool_test target 残留）✅。

**结论**：无遗漏、无多余改动；与 review-fix-delivery.md 审查通过的实现（缓存上下文绑定 + 越界直算 + Cache2DDF 16 槽 LRU 保留 L643-701）完全一致。

## 要点 2：知识库提交 bc5f5b0（与落盘数据一致性）

**07 篇「修复闭环验证」表（L122-135）vs 落盘文件**：

| 指标 | 07 篇记录 | 落盘证据 | 一致 |
|---|---|---|---|
| FlatCache rebuild | 216 = 6.0/chunk | collect-summary（36,252 修复前）；终版 216 见 knowledge-drafts/draft-07-block-pipeline-fix-closed（stat_ctx.py 统计 splinedebug_8576_t1_ctx） | ✅ |
| rebuild chunk 覆盖 | 36 | 同上（修复前 112） | ✅ |
| CACHE2D miss | 23,117 | 同上（修复前 351,536） | ✅ |
| SPLINE 非 leaf 口径 | 3,032/chunk | 同上（修复前 66,682） | ✅ |
| spline.sample 全量 | 5,906/chunk（212,622/36） | **wgprofile_8576_t1_ctx.txt L79-80 实测 spline.sample=212622、spline 单次 7,971ns** | ✅ |
| 单线程 wall | 2,910ms | wgprofile_8576_t1_ctx.txt L41 wall=2910.0ms | ✅ |
| bench 单线程 | 62.38ms/chunk（3×） | **bench_8x8_noprof.txt L4 threads=1 62.38ms/chunk** | ✅ |
| 对齐 8576 / 3200 | 99.9994% / 99.9997% | **regress_8576_raii.txt L41 TOTAL=99.9994%**、**regress_3200_raii.txt L21 TOTAL=99.9997%** | ✅ |

**10 时间线 2026-08-12 条目（L1145-1213）**：根因定论 + 修复实施闭环（16 槽 LRU 失败 → 上下文绑定成功）+ 验证数据表 + judge 通过（4 项修正闭环 L1205）+ 用户验收 + 剩余课题 3 项（多线程无加速 / spline 单次 7,971ns / aquifer 4×）——与 07 篇、review-fix-delivery.md 一致 ✅。

**discovered #10（algorithm-fingerprints.md L235-276）**：thread_local 缓存 vs 每 chunk 跨线程执行模型冲突指纹，含 08-11/08-12 数据、主因机制（H2 嵌套递归蔓延）、修复方案（上下文绑定 + 越界直算）、修复后验证（216/36/3,032/62.38ms/99.9994%/99.9997%）、「多槽 LRU 不够」教训 ✅。

**index.yaml**：perf-rework 登记 6 条目（requirements-doc / static-audit / root-cause-draft / review-rootcause / fix-design / review-fix-delivery，kind 与 status 正确）——审查修正项 ① fix-design 登记 **已闭环** ✅。

**reasonix.toml cold_resume_prune 删除**：当前 `[agent]` 段为空（无 cold_resume_prune 键），提交前该键存在且为未使用 agent 选项；删除属文档整洁性修正，合理 ✅（3 deletions 与之吻合）。

**结论**：知识库与验证数据一致，双种子 99.9994%/99.9997%、rebuild 216=6.0/chunk、覆盖 36、SPLINE 3,032/chunk、单线程 3× 全部有落盘背书。

## 要点 3：产物提交 c0ac286（调查链完整性与排除项）

**调查链完整**（.investigations/perf-rework/ 29 文件）：
- 主文档：requirements-doc → static-audit → architecture → root-cause-draft → review-rootcause → fix-design → review-fix-delivery（7 篇，链路闭环）✅
- 剩余课题：**mt-serialization-investigation.md**（见下）✅
- 附证：random-seed-sampling.md（RQ-004 随机种子对拍）✅
- 摘要：cmd-output/collect-summary.md（采集摘要）✅
- 知识草稿：knowledge-drafts/ 10 个 draft（07/10/discovered 的最终版草稿）✅
- cmd-output/ 16 个实测文件（regress_8576/3200_{fixed,raii}、wgprofile_8576_{mt,t1,t1_ctx,t1_fixed}、bench_8x8_noprof、bench_fixed_ctx、wall_t1/t8_noprof、conctest8、collect-summary）✅

**剩余课题不确定性记录（mt-serialization-investigation.md，44 行）**：
- 多线程无加速 **1.25× max**（pool_test T=1 90.60 vs T=4 72.70 ms/chunk，T=8 反降 82.21；bench T=8 62.17 ≈ T=1 62.38 完全无加速）✅
- spline 单次 **8µs vs 992ns**（7,971ns WG_PROFILE t1 vs 旧基线 992ns = 8×）✅
- **git 二分建议**（§4.1：86e4057 之后 → HEAD 定位 spline 退化引入提交；用户已同意）✅
- 已排除假设 5 条（CRT 堆锁 / 池 worker / beardifierMtx / splitterFor 锁 / regionColsMtx）✅
- 待验证假设 H-A（spline 树退化）/ H-B（隐藏全局写）/ H-C（MEM-CHK static 数据竞争=UB）✅

**大文件/临时文件排除**：
- splinedebug（512MB/260MB/203MB 原始输出）：全工作区 glob `**/*splinedebug*` 无匹配 → 未入库 ✅（仅统计摘要 collect-summary + 知识库数字固化）
- pool_test/alloc_test：glob 仅命中 build-msvc 下 .exe/.obj（`.gitignore` L2 `build-msvc/` + L13 `*.exe` + L11 `*.obj` 覆盖），源码 src/pool_test.cpp / alloc_test.cpp 已删除、CMakeLists 无 pool_test target → 未入库 ✅
- .investigations/perf-rework 最大文件 root-cause-draft.md=13,694 字节，无 >10MB 文件 ✅

**结论**：归档完整、排除项正确，未发现误入库。

## 要点 4：工作区干净度

- git 命令在本环境被权限层拦截，无法现场重跑 `git status`；改用三重替代证据：
  1. `.git/refs/heads/master` = `c0ac2864...`，`.git/logs/HEAD` 末条 new hash 同为 c0ac286 → **HEAD 与 master 一致，无漂移** ✅
  2. `.git/index.lock` 不存在 → 索引未被锁 ✅
  3. 本会话（提交完成时的完整步骤验证，历史会话 message 492/494）实跑 `git status --short` **输出为空**（工作区干净）+ `git log --oneline -4` 确认 3 个新提交；origin 未变（本地领先 16 提交）✅
- 结论：工作区干净，无遗漏未提交改动（本次审查期间未修改任何被跟踪文件）。

## 要点 5：提交纪律

| 检查项 | 结果 |
|---|---|
| commit message 英文动词开头 | ✅ fix(perf): / docs(perf): / docs(perf-rework): |
| author | ✅ 三条均 unknowbug <unknowbug@users.noreply.github.com>（.git/logs/HEAD 末 3 行） |
| 分提交粒度 | ✅ 代码（6e2c7ea）→ 知识库（bc5f5b0）→ 产物归档（c0ac286）三分离，语义清晰 |
| 提交范围 | ✅ 6e2c7ea 4 文件、bc5f5b0 5 文件、c0ac286 归档目录，无混合 |

---

## 轻微建议（不阻塞，处理后可继续）

1. **mt-serialization-investigation.md 补一条已排除假设**：「WG_PROFILE 原子计数争用 = 多线程无加速的假象」——noprof 对照已落盘（wall_t1_noprof.txt 94.04ms/chunk vs wall_t8_noprof.txt 87.61ms/chunk 仅 7%、bench_8x8_noprof.txt 无加速），即排除 WG_PROFILE 计数器开销为无加速主因，但该显式排除条目未写入文档（仅 root-cause-draft.md L147 以「原子计数争用未细分」形式记录为不确定性）。建议在 mt-serialization-investigation.md §2 补一行。
2. **splinedebug_8576_t1_ctx.txt 引用缺口**：07 篇 L124 / 10 时间线 L1186 引用 `cmd-output/splinedebug_8576_t1_ctx.txt（stat_ctx.py 统计）`，但该统计文件未随 c0ac286 归档（glob 无匹配；rebuild 216/SPLINE 3,032 等数字已固化进知识库，可追溯性依赖草稿 draft-07-block-pipeline-fix-closed）。建议要么补归档该统计文件（小文本），要么在文档中注明「统计文件未归档、数字已固化」。
3. **c0ac286 未登记 index.yaml**：mt-serialization-investigation.md（analysis, candidate 类）等新归档产物未补登 .artifacts/index.yaml（当前仅 6 个 perf-rework 条目）。产物契约上建议补登（与上次 fix-design 漏登记同类问题）。

## 汇总 verdict

- **主结论：通过。** 三个提交各归其位：代码提交 6e2c7ea 与 review-fix-delivery.md 审查通过的实现逐条一致（thread_local g_curChunkX/Z + RAII 恢复 + 越界 delegate 直算 + Cache2DDF 16 槽 LRU 保留），无遗漏/多余改动；知识库提交 bc5f5b0 的全部性能数字（rebuild 216=6.0/chunk、覆盖 36、SPLINE 3,032/chunk、spline.sample 5,906/chunk、单线程 wall 2,910ms、bench 62.38ms/chunk 3×、双种子 99.9994%/99.9997%）与 cmd-output 落盘文件逐项一致，cold_resume_prune 删除合理，judge 4 项修正项均已闭环；产物提交 c0ac286 调查链（7 主文档 + 剩余课题 + 10 草稿 + 16 实测文件）完整，多线程无加速 1.25× max、spline 单次 8µs、git 二分建议等不确定性已归档，splinedebug 大文件与 pool_test/alloc_test 二进制均未误入库。工作区干净（HEAD 与 master 一致、无 index.lock、提交时 status 为空）。提交纪律合规（英文动词开头 / unknowbug / 三提交粒度合理）。
- 3 项轻微建议见上（补 WG_PROFILE 原子争用排除条目、splinedebug 统计文件引用注明、c0ac286 产物补登 index.yaml）——均为文档完整性类，不影响本次交付正确性。
- 未发现越权标 confirmed；意见为建议，最终拍板权在用户。
