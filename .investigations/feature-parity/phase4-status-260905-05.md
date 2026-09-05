# Phase 4 状态快照（260905-05，feature parity 课题，主会话落盘）

## 已完成
1. Phase 0-3 全链：架构批准（tree 解禁 + 矿石规则层边界）→ V1 遗产验证（655/3377 签名复现，Y=section 索引裁决）→ 双 scout（管线地图/随机链 11 站点）→ fan-out b1/b2 → judge 有条件通过（5 MUST 全落实）。
2. S1 一手源：`versions\1.20.1\data\mc_src_extract\`（用户指路：项目内）。TrapezoidHeight = 2×nextBetween（一手坐实）。
3. Phase 4a 交付：s1-semantics（8 idk 裁 6）+ phase4-tree-patch（tree.rs 全文等）。
4. Phase 4b 应用：apply-report 全 hunk applied；主会话编译 **绿**（WorldgenRust rlib + worldgen release dll；仅 1 处 `opposite(&dir)` 借用修复）。

## 改动文件（未提交，git 工作区待 judge 三源核对）
- 新建：worldgen-core/src/tree.rs；lib.rs +pub mod tree;
- 改：feature_loader.rs（4 载荷+cache 参数+preload 递归）、placement.rs（谓词树+modifier+Biome 实装+未知告警+IntProvider 修正）、carver.rs（HeightProvider trapezoid 实装）、worldgen_handle.rs（调用点+anchor_biome）

## ⚠️ 风险移交（judge/worker 登记项）
- §3.5/3.6 修正改变现有 height_range 随机序列 → 必须 palette 回归定界（不止单调改善）
- R-1 遗留 idk：decorator 同 Y 序 = Java HashSet 桶序（近似方案已入码，S5 判据须声明）
- idk-7（selector/patch generate 公式）占位禁入对拍
- fancy_oak = LargeOak placer 对（b2 假设被推翻，已实现，验证口径拍板待定）
- anchor_biome 口径（chunk biome vs posToBiome jitter）已登记

## 下轮开工点（Phase 5 运行时验证）
1. 重建导出管线：G2 pregen 3377 chunk 的复现命令未在文档落单条（light-opt p2-round1 方法描述存在，命令需从 .tmp/light-g1/ 脚本与 cmd-output 考古或重建）——首选载体 = 既有 vanilla 参照不重导（.tmp/light-g1/vanilla-region + vanilla-blocks），只重导 Rust 侧（新 dll）→ 跑 .tmp/feature-parity-260905-05/v1_handoff_verify.py 对比。
2. 判据：树族+规则层差异归零方向量化（R-1/idk-7 已知偏差源声明口径）；S2/S3 回归 = 非 feature 域逐位一致。
3. 若 blob 差不归零 → .b1-1b 序列分离实验（WG_FEATURELOG A/B）。
4. 完成后 judge（MUST）→ 提交 → 知识库 subagent。
