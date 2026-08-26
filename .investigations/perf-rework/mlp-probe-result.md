# MLP 假说验证 — 软流（K 路交错）vs 顺序，单线程吞吐（微基准）

> 角色：主会话 | 日期：2026-08-24 后续 | 状态：draft（机制验证，非 production 保证）
> 背景：11× 争用归因 = 长串行依赖链 + latency QoS（每级 load 结果喂下一级，8 线程灌入 → 延迟膨胀）。
> 修复方向 = 提升 MLP（打破依赖链），但需先验证「软流打断依赖链提升吞吐」这个前提。
> 产物：`mlp_probe.cpp`（微基准）+ `mlp_probe.exe`（编译）。

## 目的
验证 MLP 假说：**软流（K 个独立点的链交错，多点 load 交叠）能否打断依赖链、提升单线程吞吐**。
若成立 → 值得做 production 完整 MLP（数据驱动 op 表 + 软流）。

## 微基准设计
- 模拟 production 的**访存依赖链**：`d = load(idx[base])`，然后 L=15 层 `d = op(d, load(idx[base+l*17]))`（每级读内存→算→下一级）。
- 数组 4MB（超 L2）+ **伪随机索引**（cache miss，贴近生产访存延迟放大）。
- **顺序**：每点一个 15 层链（串行）。
- **软流 K 路**：K 个点的链**交错**（K 点同一层交错，K 个独立 load 在飞行）。
- 测 **per-point 吞吐**（wall/N，单线程）。

## 结果（N=600000，L=15 层）

| K（软流路数）| per-point | vs seq |
|---|---|---|
| 1（seq 顺序）| 44.91μs | — |
| 4 | 47.30μs | ≈0（略高） |
| 8 | 32.51μs | **-28%** |
| **16** | **28.67μs** | **-36%** |

（另一组 N=400000 复现：seq 48.07 / soft4 48.98 / soft8 33.07 / soft16 27.46，一致。）

## 结论（candidate）

1. **MLP 假说成立**——软流（K 路交错）打断依赖链，提升单线程吞吐（K=16 **-36%**）。
2. **需足量路数**——K=4 无效（4 个点的 load 交叠不足以隐藏延迟），**K≥8 显著**，K=16 更好。生产软流需 ≥8 路。
3. **代表性边界**——微基准是**理想化访存依赖链**（L=15，读数组）；production 依赖链更复杂（noodle range_choice 分支/虚调用/grid trilinear），实际增益幅取决于依赖链形态，**需 production 完整 MLP 实测确认**。

## 意义 / 下一步
- 这是「软流打断依赖链」的**机制验证**（方向对），支撑 production 完整 MLP。
- **production 完整 MLP** = **数据驱动 op 表（无 split）+ 软流（≥8 路多点交错 op 段）**——需把 finalDensity->sample 拆成 op 记录才能交错多点 op（黑盒函数无法软流）。这**本质是「无 split 直排 + 软流」**，大工程（改采样路径 + 软流循环）。
- **另立项**（大工程）：production 完整 MLP，预期降 11×（latency QoS），但需实测确认。

## 引用
- `mlp_probe.cpp` / `mlp_probe.exe`（微基准）
- 11× 归因：`.investigations/worldgen-mt-scaling/11x-contention-investigation-log.md`
