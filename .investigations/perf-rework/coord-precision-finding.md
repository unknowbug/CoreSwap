# 坐标精度关键发现：整数/小数拆分（2026-08-13）

> 修正分层方案的一个关键假设错误。探针：`coord_precision_probe.py`。

## 问题

此前 gpu-route-decision.md 假设：「CPU 用 FP64 把大坐标折叠成 [-2^24, 2^24] 的小坐标，GPU 只接收折叠后的小坐标，float 可精确表示」。

**这个假设错误**：`maintainPrecision` 折叠后坐标仍是 ~2^24 量级（如 16777216.5），float 在此范围的 ulp = 2，整体 float 化会**丢掉小数部分**（16777216.5 → 16777216），噪声误差放大到 O(0.1)。

## 实测（identity perm, origin=0，Perlin 噪声）

| 坐标场景 | 整体 float 化误差 | 拆分（int32 整数 + float 小数）误差 |
|---|---|---|
| 近坐标（720 块 = 123194.2） | **2.1e-3** | 5.7e-9 |
| 折叠后（16777216.5） | **2.2e-1** | 1.3e-8 |

- 「整体 float 化」误差 2e-3 ~ 2e-1 → **会翻转方块，不可接受**。
- 「整数/小数拆分」误差 ~1e-8 ~ 1e-9 → 方块零影响。

## 正确方案：坐标拆分

CPU（FP64）：
1. `coord = pos.x × scale`（double 精确）
2. `folded = maintainPrecision(coord)`（折叠，double）
3. `d = folded + origin`；`i = floor(d)`（**int32 精确**）；`g = d - i`（**小数 ∈ [0,1)**）

传输给 GPU：
- `i`（int32，精确 → hash 进 perm 表不丢）
- `g`（float，误差 ~1e-7 → grad/lerp 用，噪声误差 ~1e-8，方块零影响）

GPU（FP32）：Perlin 采样 = hash（int32 i）+ grad/lerp/perlinFade（float g）。

## 为什么这样对

- Perlin 采样对坐标只有两个用途：① 整数部分 `floor(d)` 进 perm 表 hash（**必须精确**，否则 hash 错位噪声全乱）② 小数部分 `d - floor(d)` 进 grad 点积 + fade（**精度 ~1e-4 即可**，float 的 ~1e-7 绰绰有余）。
- 整体 float 化把「整数精度」和「小数精度」绑在一起，被大整数撑爆；拆分让两者各取所需：整数走 int32 无损、小数走 float 够用。

## 对分层方案的影响

- 「坐标折叠 FP64 放 CPU」仍是正确的，但**不能把折叠后坐标整体 float 化传给 GPU**——必须拆成 int32 整数 + float 小数。
- 传输量：每坐标 int32 + float = 8 bytes（vs 之前单 float 4 bytes），翻倍但可接受（88KB → 176KB/chunk 输入，仍远小于 CPU 47ms 的数据流开销 144µs）。
- 更新 gpu-route-decision.md 的「关键机制」与 coreswap-vs-c2me.md 的对应表述。
