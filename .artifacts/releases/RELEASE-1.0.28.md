---
version: 1.0.28
status: pending
date: 2026-09-11 13:10
---
## 1. 构建产物
- jar: `E:\PYTHON\CoreSwap\versions\1.20.1\java\build\libs\coreswap-1.20.1-1.0.28.jar`
- jar sha256: `0023654788d7db39699ba421ebfbb0cb4e283cfd5d0fb1c1df95206cc079fe99`
- dll sha256: `dd3b645f2c79d2cb54619e0ecb95f9b913b3fe02f30d74eba9ea5ee92886765d`（jar 内 `native/worldgen.dll` = `target/release/worldgen.dll`，主工作区 2026-09-11 13:0x 核验 MATCH）
- 源 commit: `7c46803`（fix(ca): gate terrain_cache backfill on features enabled；上游 `fc4cc7c` = 1.0.28 confirmed 基线）

## 2. 验证记录（判据表 + 证据路径）
- **异步化行为等价门（260910-06，confirmed）**：同 chunk 原生输出 FNV 指纹 **4140/4140 全等**（噪声无关、决定性）；同构建态单变量 A/B `222/227 s → 37/38 s`（5.97-6.00×），`inflight max` 1→23 —— `E:\PYTHON\CoreSwap\.artifacts\perf-reg-260910-06\verdict-260910-06.md` + `judge-verdict-260910-06.md`
- **Java 工程迁移等价门（260910-07，confirmed）**：迁移前后 jar 逐条目 sha256 全等（1.20.1 1077/1077）—— `E:\PYTHON\CoreSwap\.artifacts\perf-reg-260910-07\verdict-260910-07.md` + `judge-verdict-260910-07.md`
- **用户实机测试（260910-07 23:1x，观察级）**：1.0.28 jar 三个世界（主世界/下界/末地）跑过无问题（实机 jar 为 597e12ed 引擎形态；本票引擎为 dd3b645f，差异 = ca_min 门控微修，见下条等价论证）
- **ca_min 门控微修 A/B（260911-01，candidate）**：4 臂交错（post/pre×2，async mask=3，r500）吞吐/内存均在噪声带内**无回归**；行为不变性 = 静态构造性论证（skip 模式 pre 恒走 miss=直算，post 直算同函数）+ `[WG-CONF]` 两臂逐字段一致 —— `E:\PYTHON\CoreSwap\.investigations\perf-release-260911-01\record-ab-260911-01.md` + `patch-ca-min-gate-260911-01.md`
- **性能基准（260910-08）**：1.0.28 对 vanilla ≈1.26-1.34×（r500/r1000，Chunky 大样本）；对 ≤1.0.27 旧形态 5.5-5.7× —— `E:\PYTHON\CoreSwap\.investigations\perf-release-260910-08\MANIFEST.md` §1-§2
- **执行体三元组（本票，2026-09-11）**：jar sha / jar 内 dll sha / target dll sha 三者 MATCH（重编后重算，非沿用上一块转录）。防陈旧证据（#23/E9 家族）：jar 以 `gradle :build --rerun-tasks --console=plain` 重编于 **2026-09-11 13:04**（A/B 窗口 12:54-13:00 之后，源 commit `7c46803` 在 A/B 前已提交），随后解包核验三元组；A/B 各臂已清 `coreswap-native` 解压缓存（驱动 v2 硬门禁）。

## 3. Changelog（中英双语全文，可直接贴 Release notes）

### 中文
- **异步化世界生成接管段（1.20.1 全三维度）**：`populateNoise` 接管段改为提交至工作线程池（对齐原版异步形态），消除单车道串行；批量预生成吞吐相对旧串行形态 **约 5.5-5.7×**（同构建态 A/B：205 s → 36 s / 4225 chunks；同批单变量对照实测 5.97-6.00×），相对原版快 **约 1.26-1.34×**（大样本 Chunky 基准，r500-r1000）。可用 `-Dcoreswap.syncfill=1` 回退同步形态。
- **ca_min 缓存门控微修**：features 关闭（默认 mask=3 出厂形态）时跳过地形缓存回填（该路径只写不读），移除每 chunk 一次 ~393KB 的纯拷贝开销（吞吐差异在噪声带内不可判，无性能承诺）；features 启用路径行为不变。
- **Java 工程迁回版本管理**：mod 源码自 `runtime/` 迁至 `versions/<ver>/java` 入库（迁移前后 jar 逐条目 sha 全等）。
- 行为等价证据：同 chunk 原生输出指纹 4140/4140 全等；实机三世界测试通过。

### English
- **Async worldgen takeover segment (all three 1.20.1 dimensions)**: the `populateNoise` takeover segment now runs on the worker pool (matching vanilla's async shape), removing the single-lane serialization. Bulk pregeneration throughput is **~5.5-5.7x** over the old serial shape (same-build A/B: 205 s → 36 s per 4225 chunks; 5.97-6.00x in the same-batch single-variable comparison) and **~1.26-1.34x faster than vanilla** on large-sample Chunky benchmarks (r500-r1000). Fallback to the synchronous shape via `-Dcoreswap.syncfill=1`.
- **ca_min cache gating microfix**: when features are skipped (factory default mask=3), the terrain-cache backfill is bypassed (it was write-only on that path), removing a pure ~393KB per-chunk copy cost (throughput delta is within the noise band; no performance claim); the features-enabled path is behaviorally unchanged.
- **Java mod source moved back under version control**: `runtime/` → `versions/<ver>/java` (jar entry-level sha256 identical before/after migration).
- Behavioral equivalence evidence: 4140/4140 identical per-chunk native-output fingerprints; verified in-game across all three dimensions.

## 4. 发布动作清单（Maint 执行）
- tag 名：`coreswap-1.20.1-1.0.28`（对齐 1.0.27 约定）
- Release 标题：`CoreSwap 1.0.28 (MC 1.20.1)`；notes 用 §3
- asset 上传名：`coreswap-1.20.1-1.0.28.jar`
- 发布后：关联 bug 卡回填载体版本（如适用）

## 5. 已知边界 / 降级声明（§9.7 语义）
- 性能数字口径：**开发服务端 + Chunky 批量预生成**（无客户端渲染、无交互探索延迟、无混合核调度）；CPU 利用率列为整个 JVM 而非 Rust 池宽度。
- 实机测试为观察级（未量化、未声明对照基线/mod 列表），且基于 597e12ed 引擎；本票引擎 dd3b645f 的差异面（ca_min 门控）以静态构造性论证 + `[WG-CONF]` 行为自证覆盖（Degraded，未做 region 逐位对拍）。
- 引擎 dll 1.0.27→1.0.28 在 1.20.1 上无速度差——收益全部来自 Java 侧异步形态（260910-08 MANIFEST §3-§4）；「内存优化」组件在出厂 mask=3 下不可达。
- ca_min 门控微修仅核 1.20.1（1.21.6 同形态核对显式延后，用户裁决）。
