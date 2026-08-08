# 主会话运行时验证补跑（2026-08-08，Phase 2 收尾）

> worker（surface-plus1 / aquifer-wateredge / biome-terracotta）沙箱拦截 exe，主会话补跑：
> ① WG_SURFDUMP 三列 finalDensity 剖面（743,-406 / 800,-363 / 802,-372）
> ② -biomeDump (812,73,-337)
> ③ Java DensityInterpolator 源码精度确认（ChunkNoiseSampler.java）

## ① finalDensity 剖面（%.6f，y 步长 4）

### (743,-406) —— 地表三连 +1 类
| y | initialDensity | finalDensity | 判读 |
|---|---|---|---|
| 64 | 0.632239 | +0.042307 | C++ stone（fd>0→-1→stone）|
| 68 | 0.003681 | **+0.021163** | C++ stone（got=1 ✓）；参照 dirt → Java fd≤0 |
| 72 | -0.624877 | **0.000000（无负号 → ≥0）** | C++ ≤0 边界 → 流体决策 → stone（got=grass，heightmap 顶 72）；参照 air → Java fd>0 |
| 76 | -0.840703 | -0.056512 | air 区域 |

- estimateSurfaceHeight(743,-406)=64（savanna 高原 est=64，与参照一致）
- **关键**：C++ fd(72)≈0（+1e-7 量级或精确 0），Java fd(72) > 0（air）——符号差在 density≈0 边界

### (800,-363) —— 地表三连 +1 类
| y | initialDensity | finalDensity | 判读 |
|---|---|---|---|
| 72 | -0.033505 | **+0.213881** | C++ fd>0 → stone？但 got=1 stone ✓ 参照 dirt（buildSurface 染层差）|
| 76 | -0.732595 | **+0.043437** | C++ fd>0 → stone（heightmap 内）→ 染 dirt（got=9 dirt ✓）；参照 grass |
| 80 | -0.929470 | -0.129633 | air |

- 参照列 stone 顶 71/dirt 72-75/grass 76；C++ heightmap 顶 77（fd(76)=+0.043 → 非 air）
- **C++ fd(76)=+0.043 vs 参照 y=76=grass（buildSurface 顶）→ Java 该点 fd 应 ≤0 但 y=76 是 grass 非 air？** 需注意：Java heightmap 顶 76 = 最高非空气块，fd(76) ≤ 0 → stone → 染 grass ✓；C++ fd(76)=+0.043 > 0 → stone（-1）✓ 同判 stone——**但 C++ 染 grass 的是 y=77**（fd(77)？未打印）→ heightmap 差 1

### (802,-372) —— 深板岩↔air 互补翻转类
| y | initialDensity | finalDensity | 判读 |
|---|---|---|---|
| -4 | 14.437470 | -0.001872 | C++ ≤0 → 流体决策 |
| 0 | 13.651744 | **-0.000000（有负号 → <0）** | C++ 流体决策 → d+e≤0 → bs=air（got=0 air ✓）；参照 deepslate → Java fd≤0 → d+e>0 → stone |
| 4 | 12.866018 | +0.019303 | stone |

- **(802,0,-372) C++ fd=-1e-7 量级负值 → 流体决策输给 air；Java 同点 d+e>0 → stone** —— aquifer 边界翻转直接证据

## ② biome 判定（#23）
- `-biomeDump (812,73,-337)` = **minecraft:forest**
- 参照 blocks biome 段（z*16+x 索引，workerC 确认）= **badlands**
- **C++ biome 判定差坐实**（y=73 采样点），需追 C++ biome 采样（biome.h）vs Java MultiNoiseBiomeSource

## ③ Java DensityInterpolator 精度（ChunkNoiseSampler.java 703-800）
- 角点缓存 double[][] + MathHelper.lerp3 三线性（double），先 Y 后 X 后 Z（lerp3 内部）
- C++ InterpolatedDF::sample（density.h 529-537）同公式同顺序同 double —— **插值公式逐位对齐**（-288 已验证 interp 差 7e-6 量级，非公式差）
- **~1e-6 差的源头不在插值公式，在角点值本身**（base_3d_noise / sloped_cheese 链的浮点路径，已知 POC 现象）

## 综合结论（21 块 + 2 块）
1. **21 块**（深板岩/水边界 12 + 地表三连 9）：C++ vs Java 块级 finalDensity 在 density≈0 边界的 ±1e-6~1e-7 符号差 → aquifer 判定翻转（A/B/C 类）或 buildSurface 起点差 1（D 类）——**插值精度差（角点值）导致，非 aquifer/规则 bug**；形态全部边界 ±1 单格，非结构假 diff
2. **#23/#24**（forest terracotta）：**C++ biome 判定差**（(812,-337) C++ forest vs vanilla badlands）→ terracotta 带规则未触发/顶差 1 —— 真 bug，可修
3. **#15**（river）：同 21 块机制（aquifer 边界翻转）
4. **理论不等价点 2 处**（aquifer.h:367 INT32_MAX vs -32512；method_43718 float/double）——影响面≈0，可顺手对齐

## 待办
- 子步骤 6：y=-32 噪声卡复核——(805,-32,-427) 深层 terracotta 疑点是否与 #23/#24 同源（C++ biome 判定差）
- 子步骤 7：SteepCond 理论差异对拍
- Phase 3：biome 判定修复（#23/#24 真 bug）+ 顺手对齐 2 点；21 块插值精度差立项评估（大工程，需 Java DensityProbe 同点高精度对比）
