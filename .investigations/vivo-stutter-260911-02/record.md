# vivo-stutter-260911-02 — 实机 VD32 卡顿（1.0.28 jar 00236547 / dll dd3b645f）

> 实际 2026-09-11 14:33 起。触发：用户 confirmed ca_min 门控微修 + 撤回 RELEASE-1.0.28 工单（实机测试「还是有问题」——性能/卡顿）。

## 用户日志判读（latest.log，144 行，附件副本 sha256 f2bc8500…）

- 载体：**集成服（单机客户端）**，Fabric 1.20.1，59 mods（coreswap 1.0.28 + fabric-api），**Java 24**，机器路径 `D:\Games\Minecraft\coreswap`，用户 Valkyrozen。
- 执行体自证：`[CppBridge] dll=…\Temp\coreswap-native\worldgen.dll sha256=dd3b645f…`（= 本块修复版，测的确实是最新产物）+ `stageMask=3` 三维度 enabled。
- **seed `7349435828306001495`** = 260910-08 §6 朋友报告日志同一 seed ⇒ 可能同一台机器/同人，两次报告同源（待确认）。
- 视距 10→**32**、模拟距离 32；13:42:40 起旁观模式飞行 ~2 分钟；13:44:15 `Received passengers for unknown entity`（孤立 WARN，暂无因果）；13:44:32 退出。
- `[CppBridge] init` 在 spawn region（8.8s）Done 之后 ⇒ 出生区恒 vanilla 生成（已知边界）。
- 卡顿窗口：推测 = 13:42:40-13:44:15 飞行探索段（VD32 高负载）。

## 候选假设（互斥）

- **H1 客户端共享工作池争用**：async worldgen 与客户端区块网格构建共用 `Util.getMainWorkerExecutor()`，集成服同进程争用——专用 dev server 载体结构性测不到（260910-08 §6 候选 2）。成立前提：CoreSwap 的池占用形态与 vanilla 可辨差（vanilla 用同一池）。
- **H2 对照缺失假象**：VD32 + 高速飞行本身极端负载，同机 vanilla VD32 可能同样卡（#64/#65：无对照实机观察不可归因）。
- **H3 接管尾延迟形态差**：突发区块请求下 async 提交/回调链尾延迟 ≠ vanilla（#83 延迟 vs 吞吐分母）。

## 待用户补充（判别信息）

1. 同机 vanilla + VD32 + 同样飞行：卡不卡（H2 一票判据）
2. 卡顿形态：持续低帧 vs 周期顿挫；静止是否卡
3. 机器规格（CPU/GPU/内存）
4. （可选）spark profiler 30s 链接

## 纪律注记

- 本块 A/B（dev server + Chunky）「无回归」结论**禁止外推**到集成服 VD32 场景（§9.7 载体差；verdict-260911-01 §3 已加⚠️）。
- 机制未明 ⇒ 开工先 scout 勘探（管线/线程面地图）；分叉 ≥2 互斥 ⇒ fan-out。

---

## 进展 1（2026-09-11 15:0x）：vanilla 侧核对 + C1 排除

### 已证事实（javap 直读，详见 `vanilla-facts-javap.md`）

1. `Util.getMainWorkerExecutor` = official `ac.f()`；池 = **ForkJoinPool，宽 ≈ cores-1**（`max.bg.threads` 缺省 255，非 7）。
2. **客户端网格构建与 worldgen 共用同一池**（`WorldRenderer`(fjv) 构造 `ChunkBuilder`(fmp) 传 `ac.f()`；`MinecraftServer` 亦取 `ac.f()`）⇒ **H1「共用池」不是差异**。
3. vanilla `populateNoise` 也是裸 `CompletableFuture.supplyAsync(supplier, executor)`（`dhn` 字节码）⇒ **提交形态同构**。
4. ⇒ 差异只能在「每 chunk 附加成本/占用时长」，或在池外。

### C1（每 chunk 线程创建爆炸）——**实测排除**

- 机制真实存在：Java 侧 `THREADS` 客户端 = -2（`CppBridge.java:40-62`）→ Rust `adaptive_threads(-2, count=1)` 不 clamp（`api.rs:38-40`）→ `std::thread::scope` spawn 物理核-2 个线程，仅 t=0 干活（`api.rs:146-169`）；Java 侧 count 恒 1（`CppBridge.java:398-399`）。
- **单变量 A/B（本地，4 臂交错，`CORESWAP_THREADS=1` vs 默认，同 dll dd3b645f/seed/region）**：
  - 参数生效已行为化自证：`[WG-CONF] coreswap_threads=1` vs `(unset)`（#81）。
  - 结果：auto 469/484 cpuSec、wall 37/44 s；threads-1 486/487 cpuSec、wall 40/42 s ⇒ **无差异（噪声带内）**。
  - 判定：线程创建浪费真实但量级可忽略（<1% CPU），**不是**低帧根因。数据 `.tmp/vivo-stutter-260911-02/ab-threads/results.txt`。
  - 附注：仍值得顺手修（一行 clamp），但不作为本课题主线。

### 新主线嫌疑（C2/C3，量级未测）

- **C2 逐块加锁写回**：`CppBridge.writeChunk`（`CppBridge.java:518-549`）对**全部 98,304 格（含空气）**调 3 参 `sec.setBlockState` ⇒ 逐次 `PalettedContainer.swap` 加锁；overworld 384 高下空气占比大（估 60-75%）⇒ 其中大部分是无操作写。vanilla 对照：外层段锁 + `swapUnsafe` + 空气短路（scout §3）。
- **C3 分配/拷贝链**：JNI `int[98304]`(384KiB) + Rust `local` 零化 Vec(384KiB, `jni_bridge.rs:205`) + `col.data().to_vec()`(384KiB, `worldgen_handle.rs:682`) + `BlockColumn::new`(384KiB, `blocks.rs:166`) + 6 张高度图全量重扫（`CppBridge.java:552-558`）。
- **量级旁证（既有 bench，非本块实测）**：同区域 4225 chunks，CoreSwap JVM cpuSec ≈469（11.7 核）vs vanilla 臂 ≈233 core-s（4.8 核）⇒ **CoreSwap 每 chunk CPU ≈2× vanilla**；客户端上这会与渲染线程争 CPU（vanilla 因单车道而只用少数核）。
- **机制假设（待用户侧臂 B/E 判别）**：客户端低帧 = worldgen 高 CPU 占用饿死渲染/网格构建；若臂 B（重飞已生成区域）FPS 恢复 ⇒ 生成驱动成立，直接进 C2/C3 优化。

### 子角色执行记录

- scout：完成（`scout-map.md`，独立命中 C1/C2/C3 同三条）。
- fan-out b1：完成（`b1-pool-occupancy.md`，H1 存疑→由本轮 javap 事实进一步削弱）。
- fan-out b3：**两次中断失败**（无产物）⇒ 其辖区（JNI/写回稳态成本）**由主会话收敛吸收**（本轮已推进 C2/C3）。
- 用户侧三臂协议：`user-ab-protocol.md`（待用户读数）。
