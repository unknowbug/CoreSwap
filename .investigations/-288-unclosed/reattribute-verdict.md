# -288 差异重归因（用户洞察驱动，2026-08-09）

> 背景：用户指出「冰山在无陆地时也生成 = FEATURE 独立生成实心块」，质疑海底边界「岛」是否 FEATURE 产物。
> 决定性验证：NOISE-BLK 铁证（status=noise 打印验证）(-244,-256) y=58-61 NOISE 阶段已是 stone + Java cns 权威密度负 → **e 值翻转（B3）成立，岛非 FEATURE**；AQF-J densFn +0.037 = CellCache 反射垃圾（phase5 L750 铁律）。
> 但重归因揭示：-288 的 67042 块 FULL 差异中 **FEATURE 占 74.2%**，海底边界真核心候选 17251 块（其中 e 翻转 ~7250 + surface 规则 ~9979，且部分可能混村庄地基）。

## 一、用户拍板（ask 工具）

**FEATURE 实施范围 = 扩展：carvers + 岩石替换 + 装饰层（树草/矿石/团块）**——放弃原「只做 carvers+岩石替换、暂缓装饰层」决策。
理由：-288 差异 74.2% + 300515 差异 94.1% 来自 FEATURE；含装饰层才能闭合 300515 实机差异。

## 二、-288 差异重归因（67042 块 FULL 差异，reattribute_m288.py）

| 类别 | 块数 | 占比 | 来源 |
|---|---|---|---|
| FEATURE 直接产物 | 38056 | 56.8% | 岩石替换 33k + 矿石 3k + 村庄方块 ~1k + 紫晶洞 138 |
| carvers 洞穴（deepslate→air） | 6684 | 10.0% | CaveCarver 雕刻 |
| carvers 含水层（stone→water） | 5051 | 7.5% | 挖洞+液面填水（已闭合项） |
| **FEATURE 合计** | **49791** | **74.2%** | |
| 真核心：water→solid（e 翻转候选） | ~7250 | 10.8% | water→stone 3117/dirt 2539/sand 723/grass 540/sandstone 198/gravel 133 |
| 真核心：terrain→terrain（surface 规则） | ~9979 | 14.9% | stone↔gravel 2135+1802+746、stone↔dirt 2119+655、sand↔sandstone 427+638+273 |
| **真核心合计** | **17251** | **25.8%** | 需 NOISE 状态进一步拆分（部分可能混村庄地基 FEATURE） |

## 三、决定性验证结论（vs 用户假设）

| 问题 | 结论 |
|---|---|
| (-244,-256) 岛是否 FEATURE？ | **否**——NOISE-BLK 铁证（status=noise）NOISE 阶段已 stone，FEATURE 在 NOISE 后不可能产生 |
| Java 密度是否 C++ 少算 0.11？ | **否**——AQF-J densFn +0.037 是 CellCache 反射垃圾值（phase5 L750 铁律），Java cns 权威密度为负 |
| 岛 solid 机制？ | **aquifer e 值翻转（B3 成立）**——密度负但判 solid 唯一自洽 |
| 用户洞察价值？ | **部分正确**——-288 差异确混 FEATURE（村庄 dirt_path 160/紫晶洞 903），且 FEATURE 占比 74.2% 远超之前认知；「先做 FEATURE」方向正确 |

## 四、对 e 值翻转课题的影响

- e 值翻转（B3）仍成立，但**范围缩小**：真核心 water→solid ~7250 块（10.8%），且需 NOISE 状态拆分村庄地基混入
- 修复点不变：Java 真实遍历中间量 dump 判别 (b) 液面网格输入值
- 优先级下调：FEATURE 实施（74-94% 收益）> e 值翻转（10.8%）> P1 surface（14.9% 中部分）

## 五、产物

- `reattribute_m288.py` + 输出（重归因量化）
- `extract_noiseblk2.py` / `cmp_surf_full.py`（决定性验证）
- `seed300515-verdict.md`（300515 判定，FEATURE 94%）
- MC 工程 BlockProbe `-PblockProbeFull`（78b615b，FULL 参照导出能力）
