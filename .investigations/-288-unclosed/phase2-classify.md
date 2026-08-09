# -288 未闭合差异 Phase 2 归类 + 量化（draft）

> 课题：seed=-8248318472910187742，-288,-256 4×4 区域，block_probe 95.7376%。
> 数据源：`.investigations/-288-reopen/m288_natural_rows.txt`（55318 行 natural 类 MISMATCH，带块名）+ `m288_run1.txt`（全量）+ vanilla blocks 参照。
> 脚本：`classify_m288.py`（子类归类）、`colview_m288.py`（列形态叠加）、`compare_cols_m288.py`（blocks 对比）。
> 状态：draft。

---

## 一、三大未闭合子类提取（natural 类，55318 行）

| 子类 | 块数 | 判定口径 |
|---|---|---|
| **seabed（water↔solid 双向）** | 11135 | 含水层 stone→water 4416（**已闭合 carvers**）+ 海底边界 water→solid **6710** |
| **gravel** | 4881 | 深层 deepslate→gravel 1802（ore_gravel FEATURE）+ 海底/浅层 gravel↔stone 2881 |
| **surface_rules** | 4675 | sand/sandstone/dirt 互换，beach biome 1876 集中 |
| rest（浅层岩脉/矿脉/洞穴等已闭合类） | 34627 | 岩石替换/矿脉/carvers 洞穴（范围外） |

**seabed 细分**（关键：含水层与海底边界方向相反）：
- `stone→water` 4416（+deepslate→water 635）：C++ 实心、vanilla 水——**carvers 阶段产物（已闭合，AQF-APPLY + chunk status 铁证）**
- `water→stone/dirt/sand/sandstone/gravel` 6710：C++ 水、vanilla 实心——**海底边界（本课题主目标）**

## 二、海底边界 y 分布（6710 块）

- 主峰 **y=52-62**（y=55:346、56:632、57:878、58:1017、59:1103、60:1157、61:1116、62:218）
- biome：cold_ocean 6109、plains 3349、beach 1197、river 480（部分块多 biome 计数）
- chunk 分布：chunk(-17,-14) 2523 最大、(-17,-13) 1479、(-16,-14) 1034 等

## 三、样本列形态（C++ vs vanilla 叠加）

| 列 | vanilla | C++ 差 | 结论 |
|---|---|---|---|
| (-264,-215) | y=53-62 stone/dirt/dirt_path 实心 | y=53-62 全 water | C++ 海底低 ~10 格 |
| (-241,-256) | y=57-61 stone/dirt 实心 | y=56-62 water | C++ 海底低 ~6 格 |
| (-244,-256)（NOISE-BLK 铁证） | y=40-50 stone / 51-57 water / **58-61 stone 岛** / 62 water / 63+ air | C++ 全 water | 4 格厚悬浮岛 = e 翻转候选 |

**关键判定：C++ 海底系统性低 4-10 格（非 ±1e-6 边界翻转）**——独立机制，不并入 8576 21 块课题。

## 四、机制候选收敛（3 互斥）

1. **B1 Beardifier 结构抬升**（(-244,-256) 岛）→ fan-out 推翻（距村庄 32 > 12 格，Beardifier=0）
2. **B2 aquifer pocket 形状场** → fan-out 推翻（density>0→solid 硬铁律；C++ 已实现；含水层=carvers）
3. **B3 aquifer 液面/e 值** → fan-out 部分支持（C++ e≡0 判水直接原因；~(b) 液面输入未闭合）

## 五、产物

- `classify_m288.py` / `colview_m288.py` / `compare_cols_m288.py`
- 输出见会话记录 + `phase3-locating.md` §三
