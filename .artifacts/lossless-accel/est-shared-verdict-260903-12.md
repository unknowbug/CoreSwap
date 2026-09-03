# est shared 臂 Java 逐位裁决（P1，260903-12）

- id: `re-code:lossless-accel:est-shared-verdict-260903-12`
- session: 260903-12（实际 2026-09-03 晚）
- status: **candidate**（judge SHOULD 审查通过后待用户 confirmed）
- 验证分层: **Full**（Java vanilla 运行时逐值 dump vs Rust 生产探针逐值 dump，同 seed 同 region）
- §9.7 可比性三要素: 载体 = Java `ChunkNoiseSampler.estimateSurfaceHeight` 运行时返回值（mixin RETURN dump）vs Rust `WG_EST_DUMP` 角值 dump；覆盖面 = seed 8576294172403134396、chunk region (200,200) 8×8=64 chunks、每 chunk 4 角列（量化归并后共同列对比）；历史口径 = 与 260903-11 四臂 hash A/B 同 region 同 seed，可直接比。

## 裁决结论

**Q：WG_EST_SHARED 开启后的输出变化（hash `74f5dfc4…` → `8bff4087…`）是「修正既有 surface est 错位」还是「引入新偏差」？**

**A：是修正。** shared 臂的 est 函数语义与 Java vanilla **一致**；off 臂（现默认）的 est 与 Java **系统性不一致**。证据：

1. **共同列全量对比（P1.3，est-compare-p13-260903-12.txt）**：
   - chunk 原点角列（两臂与 Java 共同覆盖的量化列，64 chunk 的 c0 原点角）：**shared 64/64 与 Java 逐值一致；off 0/64（64/64 全部偏离）**。
   - Java 表 11877 条（含 SURFACE 4 角 + aquifer 9 邻域全部调用），量化列无冲突（conflicts=0，纯函数性自洽）。
2. **角列定向对比（est-compare-p13b-260903-12.txt）**：Java 11 角（(i+1)<<4 列）vs Rust shared 11 角（+15→量化 +12 列）：63/64 同值——本 region est 值对 4 格量化不敏感（仅 3/64 chunk 的 x0 列与 x0+16 列值不同）；唯一敏感 chunk (201,200)：java@+16=56，shared@+12=48，off=55。

## 新发现（独立于裁决，两臂共有）

**角参数 +15 vs Java +16**：Java SURFACE 预取 4 角在 `(i,j)<<4` 与 `(i+1,j+1)<<4`（= +16，量化后仍 +16）；Rust 两臂 heights4 参数为 `cx*16+15`（量化后 +12）。scout 地图 #7 曾称「+15 量化后恰与 Java (i+1) 角一致」——**静态推断失真（#25 家族实锤）**：+15 量化 = +12 ≠ +16。影响面：est 值量化敏感的 chunk 约 1.6%~4.7%（本 region 1/64 角值差、3/64 x0/x0+16 列值差）；完全对齐 Java 需把 heights4 参数改为 +16（两臂，独立小包）。

## supersedes

- supersedes 260903-11 快照「shared 臂疑似修正既有 surface 错位 bug（未裁决假设）」→ 本条以 Full 层运行时证据升级为 candidate 结论；原条目不删。

## 交接验证纪律执行（P0）

- 四臂 hash 现象复现 PASS（estopt-ab-arms-p0-260903-12.txt）：off `74f5dfc4eede8ef4`、shared `8bff408735f1560d`、l2 与 off 逐位一致 + 命中 84.9%，与 260903-11 逐项一致 → 交接结论可继承。
- 环境四查：run\world 删除、seed 三处一致（8576294172403134396）、WG_* 默认关、dump 门控不影响 hash（复跑验证）。

## 附带发现（另立待查，不阻塞本裁决）

1. **off 臂扫描网格 −1 偏移（judge CONCERN-A1，生产 bug 线索）**：judge 复算发现 off 臂 c0 列值恒为 java−1（64/64 delta=−1），D1/D3 在 c0 均 no-op 无法解释；定位线索 = `worldgen_handle.rs` off 臂 `(min_y..min_y+noise_height).rev().step_by(8)` 为**半开区间 rev，首采样点 = min_y+noise_height−1 = 319**，Java 从 k+height=320 起扫（320,312,… vs 319,311,…）——off 是当前默认臂，活的生产 bug，另立验证修复。
2. `fill_chunk_blocks` 在 64×64 大 region sweep 至 ~2304-2560 chunk 处 panic：`surface_rules.rs:505 missing noise sampler`（estopt-sweep-260903-12.txt 尾部）。疑似预加载噪声表缺项在特定 biome/区域触发——生产稳定性问题，另立课题。

## 建议后续

1. shared 臂翻默认（est 语义修正）→ 需先落角参数 +16 修正（小 diff）再翻默认，一次到位对齐 Java。
2. 角参数 +16 修正后复跑四臂 hash + Java est 对比（应 64/64 全一致含敏感 chunk）。
3. off 臂 −1 偏移 bug（CONCERN-A1）与 +16 修正一并验证；探针 dump 头建议回显 seed（judge C1）。
