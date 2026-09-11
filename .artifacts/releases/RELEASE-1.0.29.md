---
version: 1.0.29
status: pending
date: 2026-09-11 18:58
---
<!-- 1.0.28 工单已 abort（2026-09-11 用户撤单，未投递 Maint）→ 本票为新版本号 1.0.29（用户拍板 260911-04）。相对 aborted 1.0.28 的增量：exec 模式（自有有界执行器，缺省开）+ perfprofile 临时件移除。 -->

## 1. 构建产物
- jar: `E:\PYTHON\CoreSwap\versions\1.20.1\java\build\libs\coreswap-1.20.1-1.0.29.jar`
- jar sha256: `b057fda216012647a0e0ea6bbe0d1b4967a5458950ddc5f208dc85f48c312239`
- dll sha256: `dd3b645f2c79d2cb54619e0ecb95f9b913b3fe02f30d74eba9ea5ee92886765d`（jar 内 `native/worldgen.dll` = `target/release/worldgen.dll`，主工作区 2026-09-11 18:55 重编后核验 MATCH）
- 源 commit: `cf9fa54`（perfprofile 移除 + version bump；exec 实现链 `0ecac5c`→`d20aac1`→`504193c`；Rust 侧零改动，引擎 = dd3b645f）

## 2. 验证记录（判据表 + 证据路径）
- **exec 模式三臂 A/B（260911-03，candidate→judge PASS-with-conditions 条件已应用）**：同 dll `dd3b645f`、seed 417950215108767439、Chunky r500/4225 chunks——cap0-fresh 40s/488cpu/fill 102.8ms vs **exec-def 35s（−12.5%）/455cpu/fill 70.1ms（−32%）** vs exec-16 35s/492cpu/fill 87.2ms（池宽 10 优于 16）—— `.investigations/vivo-stutter-260911-02/exec-mode-result-260911-03.md`
- **行为门（260911-03）**：三臂 `[WG-CONTENT]` 指纹各 4140 条 sorted diff=0；`[WG-EXEC] mode=own-pool pool=10` 自证三臂命中；零 env sanity boot 命中且 `[WG-INFLIGHT]` 缺席（缺省开 + 短路双向证据，#81）
- **用户实机验证（260911-04，用户直接确认）**：p2-exec preview vs 1.0.28 实机对比，「EXEC 模式完美解决问题」；线程设定确认维持「物理核−2」（现缺省 `logical/2−2` 同源口径，零改动）
- **perfprofile 移除（260911-04，`cf9fa54`）**：纯删除临时计时件（默认关路径），无行为变化面；final jar 重编后三元组重算 MATCH（源码改动与重编同分钟、commit 落盘于构建后约半分钟；sha 与源状态 MATCH 以内容指纹为准，mtime 严格序不可信）
- **继承验证记录（1.0.28 票，仍有效）**：异步化指纹 4140/4140 全等 + 5.97-6.00× 同构建态 A/B（`.artifacts/perf-reg-260910-06/verdict-260910-06.md`）；迁移等价门 1077/1077（`.artifacts/perf-reg-260910-07/`）；性能基准 ≈1.26-1.34× vs vanilla（`.investigations/perf-release-260910-08/MANIFEST.md`）
- **执行体三元组（本票，2026-09-11 18:55）**：jar sha / jar 内 dll sha / target dll sha 三者 MATCH，重算于重编后

## 3. Changelog（中英双语全文，可直接贴 Release notes）

### 中文
- **exec 填充模式（缺省开）**：区块填充（NOISE 阶段接管段）改投 CoreSwap 自有有界执行器（池宽缺省 = 物理核−2），完全退出原版共享工作池——网格构建线程拿回全部共享池容量，实机低帧/卡顿显著缓解（本机实测填充耗时 −32%、批量吞吐 −12.5%）。`-Dcoreswap.exec=0` 回退共享池+信号量形态；`-Dcoreswap.execpool=N` 覆盖池宽；`-Dcoreswap.syncfill=1` 回退同步形态。
- **移除临时诊断计时器**（perfprofile），生产路径零诊断残留。
- 继承 1.0.28 全部内容：异步化世界生成接管段（三维度，对旧串行形态 ~5.5-5.7×，对原版 ~1.26-1.34×）、ca_min 缓存门控微修、Java 工程迁回版本管理。
- 行为等价证据：三臂逐 chunk 内容指纹 4140/4140 全等；异步化指纹 4140/4140 全等；实机三世界验证通过。

### English
- **Exec fill mode (on by default)**: chunk filling (the NOISE-stage takeover segment) now runs on CoreSwap's own bounded executor (default pool size = physical cores − 2), fully leaving the vanilla shared worker pool — chunk-meshing threads regain the full shared pool, noticeably reducing in-game stutter (local measurements: fill time −32%, bulk throughput −12.5%). Fallbacks: `-Dcoreswap.exec=0` (shared pool + semaphore), `-Dcoreswap.execpool=N` (pool width), `-Dcoreswap.syncfill=1` (synchronous shape).
- **Removed the temporary perf-profile timer** — no diagnostic residue on the production path.
- Carries over all of 1.0.28: async worldgen takeover (all three dimensions, ~5.5-5.7x over the old serial shape, ~1.26-1.34x vs vanilla), ca_min cache gating microfix, Java mod source back under version control.
- Behavioral equivalence evidence: 4140/4140 identical per-chunk content fingerprints across all three arms; 4140/4140 async-shape fingerprints; verified in-game across all three dimensions.

## 4. 发布动作清单（Maint 执行）
- tag 名：`coreswap-1.20.1-1.0.29`（对齐 1.0.27/1.0.28 约定）
- Release 标题：`CoreSwap 1.0.29 (MC 1.20.1)`；notes 用 §3
- asset 上传名：`coreswap-1.20.1-1.0.29.jar`
- 关联 bug 卡回填：不适用（本票无 bug 卡来源改动）
- 发布后：状态回写 `status\` 镜像（1.0.27/1.0.28 镜像缺失项一并待补）

## 5. 已知边界 / 降级声明（§9.7 语义 + judge C3）
- **池宽按逻辑核近似**：缺省 `logical/2−2`（SMT2 下 = 物理核−2，与引擎 adaptive_threads 同源）；非 SMT2/混合架构机器上近似可能偏离，可用 `-Dcoreswap.execpool=N` 显式覆盖。
- **多维度共用单池**：overworld/nether/end 接管段共用同一个自有执行器（惰性单例）。
- **内存**：preview 实机观察 maxWs +2.1GB（未固定 -Xmx 口径，#119 峰值≠存活集）；如遇内存压力可回退 `-Dcoreswap.exec=0`。
- 性能数字口径：**开发服务端 + Chunky 批量预生成**（无客户端渲染）；fill −32%/吞吐 −12.5% 为单 run 趋势（C1，机器 run-to-run 离散 ±10%），实机效果以用户确认为准。
- 实机 FPS 对比为观察级（用户主观确认「完美解决」，未逐帧量化）。
- 引擎 dll 与 1.0.28 票相同（dd3b645f）；本票增量全部在 Java 侧。
