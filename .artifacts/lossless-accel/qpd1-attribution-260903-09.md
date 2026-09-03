# Q-PD1 归因结论：Rust vs Java ~2.2× 差距分阶段定位（260903-09）

- status: **confirmed**（260903-09 用户拍板；judge review-260903-09 通过 + 口径声明补丁已应用）
- 验证分层：**Full（数据层差分实测，两轮稳定复现）**
- §9.7 口径声明：载体 = Rust `fill_chunk_blocks` WG_SKIP_* 门控差分 + Java WorldGenBench FULL（fresh world）；覆盖面 = Rust 每配置 64 chunks median ×2 轮 / Java 256 chunks；可比性 = 同 seed 8576294172403134396、同 region (200,200)、同预热法，与 260903-08 pc-e2e 口径同族可比；与 08-29 无树花口径不可比。

## 结论

0. **口径声明（judge 260903-09 补丁）**：(a) 本差分 bench FULL median=62.05 与同日 pc_e2e 256-chunk 口径（70.2-73.5）存在 ~13% 差（样本量/批序/测量区不同）——**60% 分母基于 62 口径**，2.2× 基线基于 256-chunk 口径，两口径并存且不可互换；量级方向结论不受影响。(b) 差分归约存在二阶级联局限（aquifer/surface 段的差值可能含下游段的二阶影响），不影响「aquifer 为大头」方向结论。
1. **差距大头 = aquifer 段：~37-38ms/chunk（占 Rust FULL ~60%，62ms 口径）**。density/interp+块填充底座 ~14.4ms（~23%），surface ~5.5-6.7，carver ~5-6.5，orevein/features ≈0（噪声级）。
2. supersedes（推翻，原记录不改）：
   - pc1-e2e-260903-08 Q-PD1 假设「Rust features/carver 段疑似 vs Java 差距大头」——实测 features ≈0、carver ~8%。
   - pc1-e2e-260903-08「seed 判别：negseed 差 <3% → seed 非因素」——证据无效：pc_e2e_bench.rs L18 解析 WG_E2E_SEED 后未使用（L22 恒用常量 SEED），negseed 运行实际未换 seed。
   - 取代自（双指针）：pc-results-260903-08.md 结论 4（Q-PD1 方向假设 + negseed 判别）——该文件文末已加 §15.4 指针小节。
3. 基线复核（交接纪律）：Rust OFF median 70.2/73.5ms（两跑，落在 08 日 71-77 带内）✓；Java FULL fresh-world median≈32ms/total 10993（对 08 日 33/11067）✓ → **2.2× 有效**。
4. 工程读数：GPU density 优化（gpu-batch-merge）端到端天花板 = 62→~47ms（density 底座归零仍慢 Java ~1.4×）；①prefetch ring 优先级进一步下调；**优化主攻转向 aquifer 段机制**（邻居随机偏移 / split / 采样次数；复用 WG_AQUIFERCOUNT/WL/BP 计数器）——新课题待立项。

## 证据链

- 过程记录：`.investigations/lossless-accel/q-pd1-260903-09.md`
- 原始输出：cmd-output/qpd1-stage-bench-260903-09.txt（两轮全段值）/ qpd1-baseline-rustoff-260903-09.txt（256 chunks 全量）/ qpd1-java-recheck-260903-09.md（Java run A 无效 + run B 有效）
- 工具：`WorldgenRust/src/bin-diag/qpd1_stage_bench.rs`（新，bin-diag 隔离区，rustc 单编）
- 自洽检查：段和−FULL=0.0%（构造恒等）；真 sanity = 段值符号/两轮稳定性（aquifer/density/surface/carver ±1ms 级一致；orevein/features |Δ|≤4ms 噪声带，段内小差不下结论）

## 附带新坑（错误台账候选）

- `run\world` 残留 → Java WorldGenBench 走服务器 chunk 系统，已生成 chunk 磁盘加载 ~1ms/chunk（total 764ms vs fresh 10993ms）——**Java bench 前必须删 run\world**（世界状态第四查）。
- pc_e2e_bench WG_E2E_SEED 解析未使用（死代码制造假判别实验）——判别实验必须验证「变量真被改变」（与探针恒等式自检同族）。
