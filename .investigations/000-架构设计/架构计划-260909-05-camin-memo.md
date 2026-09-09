---
编号: 000
任务: ca_min 残差 +35% 的写失效 memo 化回收——先评审后动手（IDK-p1 独立架构评审）
任务类型: 性能评审 + 判别实验（swe 域）
模式档位: 轻量
状态: 已批准（2026-09-09 18:31 用户批准，工作块 260909-05，git 锚 70c38d8）
---
## 范围（含明确不做什么）
- 只做「评审 + 判别实验 + 裁决」，不直接进入实现。
- 不做：不改生产代码（除非裁决立项且用户批准）、不动 off 臂、不碰 bench vs dump 口径题（IDK-p2 另案）。

## 任务拆解
1. 廉价独立验证（交接纪律）：重读 .investigations/camin-perf/perf-record-260909-04.md，核对 #101 三证（单读≈51ns / 读放大 114× / per-feature seagrass≈53%）在 HEAD 70c38d8 下成立；必要时复跑 camin_bench 采样。
2. 判别实验（决策门）：写侧画像——列重算真实重写率、写失效触发频率/分布、缓存键覆盖率；门控计数诊断（chunk 级判断防热路径污染），串行 bench 口径 + §9.7 声明。
3. 评审裁决：三互斥候选 a) 立项全量写失效 memo / b) 不立项维持 #101 / c) 有条件立项（子集 memo）。分叉 ≥2 互斥 → fan-out .bN 强制。

## 验证方式
- 实现路径硬门：行为 hash 逐位不变（on 6908dbfc9a79c40a / off 115641b86711d9cd）。
- 不立项路径：判别实验数据层证据充分性审查。
- §9.7：bench 串行 region 口径 ≠ 存档 dump 口径。

## judge 预置
- 裁决交付前 MUST judge（三源核对：.artifacts 快照 + git diff + 判别实验记录）；不立项属「根因定论」级 MUST。

## fan-out 预置
- 第 3 步三候选 MUST fan-out .bN 并行（subagent 隔离），禁止主会话自推。

## 知识库更新
- 结论性落盘（#101 维持/取代、或立项条目）→ subagent 产出草稿 + 主会话应用验证。

## 子角色介入点
- scout: 否（机制已明，只做复核）
- worker: fan-out .bN 候选产出（subagent 隔离）
- fan-out: 三候选 .bN 并行
- judge: 裁决前 MUST
- knowledge: 裁决落盘 subagent 产出草稿

## 风险 & 回退
- 写侧重写率高到 memo 失效 → 直接落 b) 不立项。
- #101 三证在当前 HEAD 不成立 → 暂停回 Phase 0 重评。
