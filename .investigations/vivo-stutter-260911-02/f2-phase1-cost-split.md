# F2 Phase 1 测量结论（260911-02）——每 chunk 成本构成与超订曲线

> 载体：dev server + Chunky r500（4225 chunks，seed 7349435828306001495，center -48/-11），dll `dd3b645f`，factory mask=3，async。
> 工具：临时 Java 计时（`-Dcoreswap.perfprofile=1`，本工作块新增，验证后移除）+ `-Dcoreswap.chunktime=1`。
> 全部为 `[证]`（本机实测）。

## 1. mixin 段构成（池宽 23，默认）

```
[PERFPROF] n=4096 fill=95.77 scan=0.09 write=6.39 height=0.42 sum=102.67 ms/chunk
[CHUNKTIME] mixin=102.60  ← 与 sum 吻合 ⇒ 计时覆盖完整段
```

| 环节 | ms/chunk | 占比 |
|---|---|---|
| **`fillBlocks` JNI（native 地形 + JNI 拷贝）** | **95.77** | **93.3%** |
| 逐块写回（98,304 次含空气 setBlockState） | 6.39 | 6.2% |
| `nz` 全量扫描（诊断用，默认也该跳过） | 0.09 | 0.1% |
| 6 张高度图全量重扫 | 0.42 | 0.4% |

**⇒ F2 原计划（跳过空气 / 高度图单遍）靶子错误**：上限合计仅 ~6.5%。写回与高度图**不是**低帧根因，不必作为主线（可作低风险顺手优化）。

## 2. 超订曲线（同一 native fill，仅改池宽；`-XX:ActiveProcessorCount=N` ⇒ 池宽 N-1）

| 池宽 | `fill` ms/chunk | 每 chunk 总 CPU | wall | cpuSec | jvmCpu |
|---|---|---|---|---|---|
| 1 | 49.98 | 78 | 265 s | 331 | 1.22 |
| 7 | 58.96 | 89 | 51 s | 375 | 6.24 |
| 11 | 62.27 | 93 | 41 s | 392 | 7.82 |
| 15 | 69.13 | 100 | 38 s | 423 | 10.55 |
| **23（默认）** | **95.77** | **111** | 38 s | 471 | 11.75 |

**读法**：
- 单线程真值 ≈ **50 ms/chunk**（池宽 1）；池宽 23 时同一份工作膨胀到 **95.77 ms（1.9×）**，每 chunk 总 CPU 78→111 ms（**1.4×**），总 cpuSec 331→471（**+42%**）。
- 曲线**单调**，拐点在 15→23 之间（本机 12 物理核 / 24 逻辑核）⇒ 与 **SMT 超订（23 线程 > 12 物理核）** 一致。
- **wall 在池宽 11 已与 23 持平（41 vs 38 s）**，但 CPU 少 79 s（-17%）⇒ **超订几乎不换吞吐，只烧 CPU**。

## 3. 排除项

- **`est_l2` 全局 Mutex 争用 —— 排除**：`WG_EST_L2=0`（行为自证 `est_l2=false`）⇒ fill **117.07** ms（更差）、cpuSec 553（更差）vs 对照 90.84/461 ⇒ L2 是**净收益**，全局 Mutex 不是主争用源。
- **C1 每 chunk 线程创建风暴 —— 排除**（本文件同目录 `record.md` 进展 1，`CORESWAP_THREADS=1` 单变量 A/B 无差异）。
- **Java 侧写回/高度图 —— 量级排除**（§1，合计 6.5%）。

## 4. 机制结论（candidate，待用户实机判别）

vivo 路径的并发度**不受 Rust 线程参数约束**：Java 池宽 = cores-1 = 23（`inflight max=23` 实测），而引擎设计的 `physical-2=10` 只 spawn 空转线程。⇒ 客户端上 CoreSwap 以 23 路穿过**与网格构建共用的池**，把 12 物理核全部超订，渲染线程被饿死；vanilla 因单车道只用 ~5 核。

**与「CoreSwap 每 chunk CPU ≈2× vanilla」自洽**：超订 + 每 chunk 真值 50 ms 单线程。

## 5. 候选修法（数据支持的优先级，已与 F2 原计划不同）

| 优先级 | 修法 | 预期 | 代价 |
|---|---|---|---|
| **P1** | 限 in-flight（对齐设计值 physical-2 ≈10，客户端可更低） | CPU -17%、每 chunk fill -35%、wall 基本不变（41 vs 38 s） | 改动小（mixin 信号量/自有有界执行器） |
| P2 | 降 native 单线程真值（50 ms/chunk 内部相位待拆） | 直接减总工作量，两边都受益 | 需继续测量定位相位 |
| P3 | Java 写回跳过空气（F2-1） | ~6% | 小，行为需区域对拍 |

## 6. 状态

- Java 临时计时器**仍在工作区**（`CppBridge.java`，`-Dcoreswap.perfprofile=1` 门控，默认关）——**Phase 2.5 前 MUST 移除**。
- 证据文件：`.tmp/vivo-stutter-260911-02/ab-threads/{results.txt,logs/}`（perfprofile / cores2 / cores8 / cores12 / cores16 / estl2-off / estl2-on-ctl / threads-*）。
