# c-A 性能计时记录（260905-09，judge CONCERN-d 核销件）
# 说明：计时来自 260905-09 会话后台作业（idk7_dump stderr 尾行），会话内未即时落盘，本文件为事后如实转录（非重跑）；
# 需要可复现计时请重跑同命令。载体=#52 确定性 region dump（seed 8576294172403134396，2193 chunks，cx[-2,40] cz[-26,24]）。

- run ca0（WG_CA_MIN=0，门关=重构零变化基线）: 133.3s  （dump done 2193 chunks nonair=69943011）
- run ca1（WG_CA_MIN=1，octx 接线 bug 版本，行为=ca0）: 241.4s （nonair=69943011，与 ca0 逐位一致）
- run ca1b（WG_CA_MIN=1，octx block_at_ext 修复版）: 223.7s （nonair=69967252，+24241 块）

# 声明：c-A 开态 dump 耗时 ≈ +68%（223.7 vs 133.3）——邻 chunk 地形列缓存摊销后仍每 chunk 约 +1 次地形成本
#（诊断路径可接受；生产翻默认前 MUST 补端到端性能对比，AGENTS.md「端到端性能对比铁律」）。
# ca1 的 241.4s 含与 ca0 并发抢核（两后台作业同跑），不可作单跑口径；ca1b 为单跑。
