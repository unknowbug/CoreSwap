# 04 篇结论冲突裁决（verdict-04）：Beardifier 缺失 = 海底边界根因（draft→candidate）

> 课题：04 篇 L108「-288 岛区 e=0 → 岛缺失不是 aquifer bug（是 ocean ruin 结构覆盖）」vs NOISE-BLK 铁证（NOISE 阶段 (-244,58..61,-256) 已 stone）矛盾裁决。
> 方法：DensityProbe 扩展 AQF-DUMP（真实 cns 遍历内复刻 apply 控制流 + 反射真实私有方法）+ BlockProbe BEARD-244（反射 StructureWeightSampler 采样目标列）。
> 日期：2026-08-09。状态：**draft**（candidate 授予前 MUST judge）。

---

## 一、裁决结论（四分支）

| 分支 | 内容 | 裁决 | 证据 |
|---|---|---|---|
| 1 | e≠0（fl2.y≠fl3.y → 液面差 → e 翻转判 solid） | ❌ **否定** | AQF-DUMP：(-244,55..62,-256) 的 fl2.y=fl3.y=fl4.y=**63 全部相等**（8 y 全测，反射真实 getWaterLevel）→ j=0 → calculateDensity=0 → e=0 |
| 2 | e=0 判 water → stone 另有来源（NOISE 阶段非 aquifer 产物） | ❌ **否定** | stone 来源 = **Beardifier 抬 density 判 solid**（density+e>0 是唯一 solid 路径，Beardifier 使 density 翻正） |
| 3 | e=0 判 solid → density 输入差 | ✅ **成立（根因修正）** | **C++ 缺失 Beardifier**：AQF-APPLY dCC（CellCache=add(finalDensity,Beardifier)）= C++ finalDensity + Java Beardifier，8/8 点 ≤3e-6 吻合 |
| 4 | 04 篇「ocean ruin 结构覆盖」 | ❌ **推翻归因** | 岛不是「结构方块覆盖」（NOISE 阶段结构方块未放置），是**结构密度修正（Beardifier）抬 density** |

**最终裁决：04 篇 L108 的「e=0 两侧一致」前提 ✓ 成立（e=0 被 Java 实测确认），但「岛缺失不是 aquifer bug」归因错——真正根因 = C++ 未实现 Beardifier（StructureWeightSampler 结构密度修正），属于 NOISE 阶段 density 链（CellCache(add(finalDensity, Beardifier))）缺失项，非 aquifer bug，也非 ocean_ruin 方块覆盖。**

## 二、裁决证据链（seed=-8248318472910187742, (-244,-256) 列）

### 证据 1：AQF-DUMP（DensityProbe 扩展，真实遍历内反射）
- 复刻 `AquiferSampler.Impl.apply` 18 候选循环（反射 blockPositions/randomDeriver 字段 + index 方法）→ r/s/t 三点
- 反射真实私有方法 `getWaterLevel(long)`/`calculateDensity(...)`/`maxDistance(...)` → fl2/fl3/fl4 + d/e/g/h
- **fl2.y=fl3.y=fl4.y=63 全部相等（8 个 y 全测）→ e=0（Java 侧实测，非假设）**
- o/p/q/r/s/t 与 C++ trace_aqf_1.txt 逐位一致（如 y=58: o=90 p=99 q=115 d=0.64 ✓）
- **AQF-APPLY 取值口径（vs 04 篇 L112 CellCache 反射污染铁律）**：dCC 来自 `CellCache.sample(cns)` 反射——该反射在**非遍历/未填充 cell** 返回垃圾值（本 run y≥310 恒 -0.024995，与 L112 记载一致）；但 y=55..62 的 cell 已被真实 cns 遍历填充，dCC 为真实值——**由证据 3 的 8/8 点独立闭环（diff ≤3e-6）证明 y=55..62 段 dCC 可信**；垃圾值段不参与裁决。fl.y/e 判定不依赖 dCC（独立反射 getWaterLevel），不受此口径影响

### 证据 2：BEARD-244（BlockProbe 扩展，反射 StructureWeightSampler）
```
[BEARD-244] y=55 val=+0.011110
[BEARD-244] y=56 val=+0.025864
[BEARD-244] y=57 val=+0.052574
[BEARD-244] y=58 val=+0.092090   ← 翻转点
[BEARD-244] y=59 val=+0.136843
[BEARD-244] y=60 val=+0.166063   ← 峰值
[BEARD-244] y=61 val=+0.147947
[BEARD-244] y=62 val=+0.063334
[BEARD-244] y=63 val=-0.057071
```
- **(-244,-256) 在 Beardifier 非零区**（B1「距村庄 32 格 > 12 → 0」被实测推翻）
- 峰值 y=60、y=63 翻负 → 与 NOISE-BLK 的 stone(y=58-61)/water(y=62) 边界吻合

### 证据 3：CellCache 等式（8/8 点闭环）
| y | C++ finalDensity | Java Beardifier | 和 | Java AQF dCC | diff |
|---|---|---|---|---|---|
| 55 | -0.043591 | +0.011110 | -0.032481 | -0.032483 | +0.000002 |
| 56 | -0.053461 | +0.025864 | -0.027597 | -0.027599 | +0.000002 |
| 57 | -0.063950 | +0.052574 | -0.011376 | -0.011378 | +0.000002 |
| 58 | -0.074424 | +0.092090 | **+0.017666** | +0.017663 | +0.000003 |
| 59 | -0.084882 | +0.136843 | **+0.051961** | +0.051959 | +0.000002 |
| 60 | -0.095322 | +0.166063 | **+0.070741** | +0.070739 | +0.000002 |
| 61 | -0.105740 | +0.147947 | **+0.042207** | +0.042205 | +0.000002 |
| 62 | -0.116134 | +0.063334 | -0.052800 | -0.052803 | +0.000003 |

- `AQF-APPLY dCC = CellCache.sample = add(finalDensity, Beardifier)` 与 `C++ finalDensity + Java Beardifier` **8/8 点 ≤3e-6**（float 精度）→ CellCache 反射值实锤 = finalDensity+Beardifier
- y=58-61 和为正（density 翻正）→ aquifer 判 solid → stone ✓ 与 NOISE-BLK 8/8 吻合
- y=55-57/62 和为负 → 判 water ✓ 与 NOISE-BLK 吻合

### 证据 4：C++ 侧自认缺失
- `density.h L470` @anchor.idk：「结构 Beardifier 密度修正未实现：结构附近 density 差 ~0.12 可翻转 aquifer 判定（-288 岛缺失根因，2026-08-08 确认）」
- `worldgen_api.cpp L570`：CellCache 注释「与 Java CellCache(add(DensityInterpolator(finalDensity)), Beardifier) 语义一致」——但实现无 Beardifier

## 三、B1/B3 结论修正（phase3-locating.md 需更新）

- **B1（Beardifier=0）推翻**：(-244,-256) 实测 Beardifier 非零（峰值 +0.166@60）。phase7 的「距村庄 32 格 > 24」距离判定有误——村庄 bbox 位置记录或距离计算口径需复核（待 Phase 3 定位村庄真实位置）。「Beardifier 实现对 -288 修复收益很小（<1000 块）」**低估**——海底边界 6710 块主体可能是 Beardifier 传导（density 翻正 → aquifer 判 solid → 整列 stone）
- **B3（aquifer e 值翻转）否定**：e=0 两侧一致（Java 实测 fl.y 全 63），无液面输入差。phase3 的「C++ 范围内待修：aquifer 液面链输入」**撤销**
- **海底边界 ≈6710 块根因 = Beardifier 缺失**（结构密度修正，NOISE 阶段 density 链缺项）

## 四、范围判定

- **Beardifier 属于「结构」相关**（StructureWeightSampler 由结构生成器注入，terrainAdaptation 机制）——之前 FEATURE 范围决策「结构暂缓」覆盖此项；**是否列入范围内待修需用户拍板**（预计可闭合海底边界 6710 块 + 村庄 12 格内其他差异）
- 海底边界已从「aquifer 液面链」误标中**纠正**——不再是 aquifer bug 候选

## 五、产物与后续

- 探针改动（MC 工程，本地 M）：DensityProbe AQF-DUMP + BlockProbe BEARD-244
- **retry 声明**：本课题（-288 未闭合）历史已有 retry cap 记录（phase10-13 超 3 轮，见 -288-reopen/summary-final §四-5）；本次裁决为**新方向（Beardifier）**，基于 AQF-DUMP/BEARD-244 两个新探针 + 8/8 点闭环，非对失败假设的重复验证——按 spec §5.3 声明，本轮验证计数 1
- 待 judge 审查 → 用户拍板：
  1. 04 篇修订（L108 归因错 → Beardifier 缺失）
  2. B1/B3 结论修正（phase3-locating.md）
  3. Beardifier 实现立项（范围判定：结构相关，需用户确认是否推进）
  4. 海底边界 6710 块重归因（Beardifier 传导，非 aquifer）
- 后续验证：C++ 实现 Beardifier（StructureWeightSampler 移植）→ block_probe -288 海底边界闭合约 6710 块验证
