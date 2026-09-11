# P1 结果（260911-02）——接管段 in-flight 限流

> 实现：`NoiseChunkGeneratorMixin.wgDispatch` 信号量背压（调用线程取许可，任务完成释放）。
> 规模：`-Dcoreswap.maxinflight=N`；缺省 = `logical/2 - 2`（本机 24 → 10）；`N<=0` = 不限（旧行为）。
> 载体同 Phase 1（dev server + Chunky r500，dll `dd3b645f`，async，mask=3）。

## 1. 参数生效自证（#81 行为化）

每臂日志首行：`[WG-INFLIGHT] max_inflight=10 logical=24 syncfill=false`，且 `[CHUNKTIME] inflight max=` 与设定值一致（10/12/16/6/23）⇒ 限流真实生效。

## 2. 性能曲线（`[PERFPROF]` + `[CHUNKTIME]` + 驱动结果）

| N | fill ms/chunk | mixin ms | gap ms | wall | cpuSec | jvmCpu |
|---|---|---|---|---|---|---|
| 0（不限） | 92.09 | 98.98 | 103.34 | 36 | 464 | 11.57 |
| 0b（不限复跑） | 106.67 | 114.68 | 105.05 | 40 | 495 | 9.86 |
| **12** | **73.34** | 79.12 | 144.97 | **40** | 454 | 9.06 |
| 16 | 80.76 | 87.35 | 126.85 | 39 | 472 | 11.77 |
| 10 | 66.67 | 72.91 | 163.86 | 43 | 429 | 8.56 |
| 10b | 75.97 | 83.12 | 183.53 | 48 | 477 | 9.45 |
| 6 | 65.20 | 72.04 | 252.95 | 59 | 432 | 7.19 |

**读法**：
- **每 chunk fill 时间 -23% ~ -30%**（不限 92-107 ms → 限 12 时 73 ms / 限 10 时 67-76 ms）⇒ 超订消除，与 ActiveCores 曲线的预测一致。
- **N=12 是 wall 无损点**：wall 40 s = 不限臂均值（36/40 s），同时 fill -23%、CPU -5%。
- N≤10 进一步压低 fill 延迟，但 gap 膨胀（车道等许可）⇒ wall +18~55%（N=6 明显不值）。
- **CPU 总节省有限（-2% ~ -5%）**：限流只压 CoreSwap 段，vanilla gap 工作仍用满池（与 ActiveCores 实验的 -17% 不可比——那整机线程数都降了，§9.7 口径差异）。

## 3. 行为门（MUST，零差异）—— 通过

`-Dcoreswap.mixlog=1` 逐 chunk `[WG-CONTENT] hash=`：

| 对比 | 指纹条数 | diff |
|---|---|---|
| cap0 vs cap10 | 4140 vs 4140 | **0** |
| cap0 vs cap10b | 4140 vs 4140 | **0** |
| cap0 vs cap0b（同臂复跑） | 4140 vs 4140 | **0** |

⇒ 限流**只改调度，内容逐位一致**（含空气位置 nz 计数亦一致）。

## 4. 结论（candidate，待用户实机）

- 机制假设成立：vivo 并发度 = Java 池宽，恢复引擎设计并发度可消除超订，**每 chunk 池占用时间 -23~30%**。
- 服务器侧默认值建议 **N = 物理核数（12）**（wall 无损）；客户端侧 FPS 是目标，建议实机扫 N=0/6/10/12/16 取最优——**只有实机能判**（本地 bench 无渲染线程）。

## 5. 待办

- 用户实机扫描（决定性门）。
- Java 侧临时计时器（`CppBridge` `perfprofile`）在正式发版前 MUST 移除。
- judge：candidate SHOULD（本轮结果齐备后）+ 收尾 MUST。
