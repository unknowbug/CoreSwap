# c-A（跨 chunk feature 读/写钩子）candidate 交付单（260905-09）

- 代码：commit df6be6c（worldgen-core/src/worldgen_handle.rs + feature.rs）
- judge：.investigations/feature-parity/260905-09-judge-ca.md（APPROVE，3 前置核销见下）
- 门控：WG_CA_MIN（默认关）；WG_CA_LOG（chunk 级诊断计数，零门控成本）

## 验证证据
1. 重构零语义变化：ca0 vs pfix 2193 chunk 逐位一致（.investigations/feature-parity/260905-09-cmd-output-v20-ca-ab.txt）
2. 修复后效应（ca1b vs vanilla，#52 载体同口径）：jungle_l −15,151→−12,027；vine −16,829→+2,134（过冲）；jungle_log −2,453→−2,129；birch −1,964→−1,712；oak_l +29,830→+31,662（方案 §3 预测「oak 不作收敛指标」吻合）
3. 行为化接线证据：单 chunk(37,-16) [CA] out_reads 23→2558、pending_writes 0→2437（260905-09-cmd-output/ca-log*.txt）
4. 性能：+68%（223.7s vs 133.3s，单跑口径）——260905-09-cmd-output-ca-timing.md

## judge 前置核销状态
① index.yaml 登记：本条目（FEA-9）即核销
② 性能计时落盘：cmd-output-ca-timing.md 即核销（含 +68% 声明）
③ 残余差清单：见下

## 残余差清单（candidate 附带）
- IDK-cA1：Java 参照生成时邻 chunk feature 时序未验 → rust 读侧用「post-carver 地形态」近似的残余量未量化
- IDK-cA2：先于 center 生成的邻 chunk 收不到 overlay（dump 顺序扫描不可达方向）
- vine 过冲（−16,829→+2,134）：新调查项，首个动作 = 方案 §5 E-cA2 镜像配对判据
- pending idx 布局 producer↔overlay 一致性：静态推断 + 全 region 输出非空验证，单点 round-trip 微验证未做（judge CONCERN-b，不阻塞）
- patch_grass_forest 空 id generate_nested miss ×31：独立缺陷（random_patch 内嵌对象解析），待登记修复

## 状态
- **confirmed（用户拍板 260905-09，工作块收口时授权）**；原 candidate 记录保留于上
- 翻默认（WG_CA_MIN 常开）另需：E-cA2 + 端到端性能对比 + 用户 confirmed（本次 confirmed 不含翻默认）
