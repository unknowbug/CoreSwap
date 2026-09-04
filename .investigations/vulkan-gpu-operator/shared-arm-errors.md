# shared-arm 课题错误台账（260903-16）

## E1: 并发 GPU bench 污染 CPU e2e 基线——37.67 假象造出「shared 臂 +10ms」假课题

- **现象**：pc_e2e_bench default 臂首跑 median=37.67ms，比 l2-only 27.50 高 10.2ms（+37%）；min 几乎不变（24.53 vs 23.84）→ 误判为「shared 臂尾部变重固有开销」（H1），据此立项归因课题。
- **根因**：default 臂首跑与 `gpu_mt_wall_retest`（70-100s Vulkan 管线编译 + 412 次 GPU fill + 持续 CPU split）**同一时刻后台并发**——GPU bench 的 CPU/GPU/内存带宽压力直接抬高同期 CPU bench 读数。违反「测量/探针污染铁律」的变体：不是探针污染测量，是**并发进程互相污染**。
- **定位**：归因开工前的代码阅读未发现任何可产出 +10ms 的机制（shared 臂 est_at 仅 4 次调用、且多为缓存命中）→ 回查测量时序，发现两个 background job 同时启动；随后三组干净数据定案：qpd1 FULL（default 臂）25.87、干净 default 复跑 28.46、l2-only 27.50——三者同水平，37.67 是孤例离群。时序声明：并发重叠为过程重建（pc_e2e total 9.5s 落入 GPU bench ≈6.5-7 分钟窗口），非日志实录——但结论不单靠它，干净复跑是独立证据；judge PASS 带此保留。
- **修复**：课题前提撤销（shared 臂无罪）；default 基线更正为 ~27.5-28.5ms/chunk。无需代码修复。
- **教训**：
  1. **性能 bench 一律串行**——任何 background job 运行期间不得跑另一条 bench（含「看起来只吃 GPU」的负载，Vulkan 管线编译/驱动/JIT 同样吃 CPU）。
  2. 「min 不变 median 变重」不只指向实现尾部开销，同样符合外源负载间歇抢占——**归因前先排除并发/环境因素**（对照基线归因法 #16 的测量侧版本）。
  3. 单点离群 + 机制上找不到解释 = 先复跑再归因（本轮 qpd1 FULL 25.87 这条 default 臂数据就在手边，若先看它可提前一步止损）。

## 错误→根因速查表

| 错误 | 根因 | 判别签名 |
|---|---|---|
| E1 37.67 假基线 | 并发 GPU bench 污染 | min 不变 + 机制找不到解释 + 同臂其他口径（qpd1）不 corroborate → 查测量时序 |
