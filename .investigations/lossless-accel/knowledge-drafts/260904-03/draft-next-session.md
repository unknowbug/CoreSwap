# NEXT_SESSION 开工点更新建议段 — 260904-03（草稿）

> **本文件为草稿**（core.worker 产出）。应用目标：`NEXT_SESSION.md` §「下轮开工点」（仅换 session 前由主会话更新，本稿供其引用）。以本稿为建议，主会话按最新 session 状态核对后应用。

## 建议追加/修订内容

### 「当前状态 · 本 session 完成了什么」追加

1. **P2 翻默认后 block_probe 存档口径 Full 回归完成（candidate 待用户 confirmed）**：default 臂 vs vanilla 99.1526% 一致（WGB2 FULL 存档口径，seed 8576 @200,200 4×4），default==off 同 SHA256 与四臂零语义差自洽 → **260903-13 遗留（off-scan-cornerfix-verdict:31-33）闭合**。verdict 草稿：`.investigations/lossless-accel/knowledge-drafts/260904-03/draft-p2full-verdict.md`。
2. **既有残差登记为独立待查项**（🔍，非阻塞）：aquifer stone↔water 互换族（99.15%，est 无关——Run2==Run3），量级与 260903-14 记录同族带，模式签名新量化。

### 「下轮开工点」修订建议

- **原第 1 项（P2 Full 回归）→ 已完成，删除或标✅**（verdict candidate 应用 + confirmed 拍板后彻底闭合）。
- **新候选开工点（按优先级）**：
  1. **残差 y 分布直方图**（最廉价先行探针，cmp 脚本加 per-y 计数）→ 分流 aquifer floodedness / 流面高度 / surface 级联三候选；分叉 ≥2 互斥候选时按 fan-out 纪律并行（见 `draft-aquifer-residual.md` 探针建议）。
  2. P7 拍板项累积：panic-505 + preload-check + 本轮 p2full verdict 均 candidate 待用户 confirmed。
  3. idk 三项沿用（未动）。
- **纪律要点追加**：LL13/#32——env 门控判别臂必须自带 env 读数输出（或 --no-daemon），「同 hash」结论同行声明「env 生效证据」，缺则降级旁证；LL12——多臂产物命名含臂标识。

### 「遗留环境状态」核对提醒

- `server.properties` level-seed 仍为 8576294172403134396、max-tick-time=-1、`run\world` 存在——Java 测量前照旧删 world / 改回 60000。
- 本轮新增原始数据：`.tmp/p2full/`（三 run blocks + cmp_full2.py）；`.investigations/lossless-accel/cmd-output/`（cmp_full2 / arm-compare / 三 run 日志）。
