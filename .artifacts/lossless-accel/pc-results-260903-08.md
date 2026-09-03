# P-C：端到端验证 + 0.61× 复测（260903-08）

- **课题**：lossless-accel / P-C「端到端验证（WG_GPU_CHANNELS A/B vs Java）+ 0.61× 双线程异常无探针复测」
- **status**：**confirmed（260903-08 用户拍板；judge 已过 review-001，C1/C2/C3 清偿后提交 0ffc4c0/9384125）**——数据层证据：0.61× 双口径 5 轮复测未复现 + sync-check mismatch=0/6144 + 端到端三方 256 chunks。
- **前包**：260903-06（dc865fe）P-A/P-B confirmed；本包基线 8046593。
- **验证分层声明**：**Full（端到端 + 数据层）**——256 chunks 端到端 wall 实测三方 + GPU fill 同步语义污染-重填探针 + 调用计数断言；性能对比非逐位正确性口径（正确性沿用 P-A 通道级 confirmed）。
- **§9.7 验证可比性声明（三要素）**：
  - 载体：Rust `fill_chunk_blocks` 完整管线（**含 features**）vs Java WorldGenBench FULL；GPU 通道级为逐 chunk 小批量 fill 口径
  - 覆盖面：region(200,200) 16×16=256 chunks、单线程、区外预热、median 主判据；0.61× 复测 = n=8/n=6144 双口径 5 轮 S/P 交替
  - 可比性：三方同日同机同 region 可比；与 08-29 历史（Java 55/Rust 45.48 无树花）**不可比**（Rust 侧 stage 覆盖不同）
- **结论（draft）**：
  1. 0.61× 异常未复现（两口径 1.006×/0.989×，计数断言过）→ 判为 260903-04 该次单 shot 测量伪影；Mutex 真串行化成立。
  2. P0「fill 全同步串行」由 Degraded 静态升级为数据层验证（sync-check mismatch=0/6144）。
  3. 端到端：零退化 ✓（OFF=主线）；ON 369ms/chunk（慢 OFF 4.8×）= 每 chunk 小批量 dispatch/readback 同步往返成本（预热仅 ~25%，用户预热假说已检验排除为主因）；真实 GPU 吞吐需批量合并（独立优化包）。
  4. **同日 vs Java：Rust 全管线 72-77ms vs Java FULL 33ms（慢 ~2.2×）**；08-29「反快 1.2×」系无树花口径不可比。开问题 Q-PD1：features/carver 段疑似大头，独立排查。
- **§15.4 supersedes**：无（本包不推翻已 confirmed 结论；对 260903-04 [fact2] 0.61× 的推翻属候选记录内标注，待 judge）。
- **遗留 idk**：Java 55→33 漂移未归因；features 段耗时分布未测。
- 过程：.investigations/lossless-accel/{pc2-retest,pc1-e2e}-260903-08.md + cmd-output/*260903-08*。
