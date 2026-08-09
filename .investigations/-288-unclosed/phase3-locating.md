# -288 未闭合差异 Phase 3 定位结论（draft，待 judge 审查）

> 课题：seed=-8248318472910187742（-8248 世界），-288,-256 4×4 区域 block_probe 95.7376%。
> 范围：judge 审查要求的未闭合 ≈23%——海底边界（water→solid ≈6710）+ gravel（≈4900）+ 表面规则（≈2900）。
> 方法：3 候选 fan-out（core.worker B1/B2/B3）+ 主会话数值验证（splitter 派生复现）。
> 状态：**draft**（candidate 授予前 MUST judge——AGENTS.md 预置触发点）。

---

## 一、结论摘要

| # | 子类 | 块数 | 定位结论 | 范围判定 |
|---|---|---|---|---|
| 1 | **海底边界**（C++ water vs vanilla stone/dirt/sand，y=52-62） | ≈6710 | **aquifer e 值翻转缺失**：C++ e≡0（fl2.y==fl3.y==63 → j=0），vanilla 实心依赖 `density+e>0` 翻转；density/splitter 已证与 Java 逐位一致，差异在**液面网格输入值**（fl2/fl3/fl4 的 y，或 est 邻居值） | **C++ 范围内待修**（aquifer 液面链输入未复刻，B3 部分支持） |
| 2 | 深层含水层（C++ stone vs vanilla water，y=11-23） | 4416+635 | **carvers 阶段产物**（AQF-APPLY 判 solid 与 C++ 一致 + chunk status=carvers 铁证） | ✅ 范围外（FEATURE carvers，已闭合） |
| 3 | gravel（深层 deepslate→gravel + 海底 gravel↔stone） | ≈4900 | 深层 = ore_gravel FEATURE（范围外）；海底 = surface rule（P1 sandstone 层 + gravel noiseThreshold 候选，未细分） | ⚠️ 部分闭合 |
| 4 | 表面规则（sand/sandstone/dirt 互换） | ≈2900 | P1 StoneDepthCond secondaryDepth 映射（Java (int)map 不 clamp vs C++ floor(lerpClamp)）——beach RANGE_6 sandstone 层边界，**P1 可解释块数待量化**（pipeline-map 估 1500-2000，dirt 互换 2119 归属未细分，需 probe 量化收益后再改） | **C++ 范围内待修**（P1，前置量化） |

## 二、fan-out 判定（3 worker，产物 .investigations/-288-unclosed/b1/b2/b3）

### B1 Beardifier（结构密度修正）——**推翻**
- StructureWeightSampler.java L90-105/L140-152：Beardifier 非零区 = **structure piece bbox 外扩 12 格**（x/z 每方向），y 以 ground 为基准 ±12；phase6/7 的「24 格」是 STRUCTURE_WEIGHT_TABLE(24³) 的误读
- -288 区域唯一参与 Beardifier 的结构 = plains 村庄（beard_thin）；沉船/ocean_ruin/矿井 = NONE（不参与）
- **(-244,-256) 距村庄 z 方向 32 格 > 12 → Beardifier=0**——岛 solid 另有机制（指向 B3）
- Beardifier 实现对 -288 修复收益很小（<1000 块），不做主路径

### B2 aquifer pocket 形状场——**推翻**
- AquiferSampler.java L149：`density > 0 → null`（solid）是硬铁律，pocket 形状场只在 density≤0 分支细分
- C++ aquifer.h 已完整实现形状场（floodedness/spread/barrier，L167/L260-273/L290-319 逐项对照一致）
- 矛盾澄清：AQF-APPLY（NOISE 阶段 aquifer 判 solid）vs NOISE-BLK（carvers 后读 water）是**不同阶段**——含水层 water = CaveCarver 挖洞+液面填充，非 aquifer
- 可解释块数 = 0；**不改 aquifer.h**（会破坏逐位对齐）

### B3 aquifer 液面/e 值——**部分支持**（机制成立，输入值未闭合）
- C++ trace_aqf_1.txt：(-244,58,-256) density=-0.074424、d=0.64>0 走 e 分支，但 `[AQF-e] e=0.0000`（fl2.y==fl3.y==63 → j=0）→ FLUID
- vanilla y=58-61 stone（NOISE-BLK 铁证）要求 aquifer 判 solid → 唯一路径 `density+e>0` → **Java 的 e 必须非零**（fl2.y≠fl3.y，如 63 vs -32512 无效液面）
- 16 项源码逐行等价（apply 控制流/calculateDensity/13 邻居液面链/est 入口/getFluidBlockY）——**差异在输入值**
- 可解释块数：~4000-6710（保守 60%/乐观 100%），合理中点 ~5000-5500

## 三、主会话补充验证（splitter 派生复现）

### (a) 子候选排除：C++ splitter/随机点派生与 Java 逐位一致
- Python 完整复现 Java 派生链：`createXoroshiroSeed(mixStafford13) → nextSplitter → split("minecraft:aquifer") → nextSplitter → split(x,y,z) → nextInt(10/9/10)`
- **8/8 点 o/p/q 逐位一致**（y=55..62：54/101/126、67/104/126、82/107/109、90/99/115、75/106/118、62/99/136、51/94/149、42/91/142 全对）
- 涉及 md5（"minecraft:aquifer"）、mixStafford13、hashXYZ（int 溢出+算术右移）、Xoroshiro128++ next、nextInt 拒绝采样、floorDiv——全对齐
- **排除**「随机点 o/p/q 派生差」（B3 子候选 a）

### (b) 子候选未闭合：液面网格输入值
- fl2 = getWaterLevelAt(r)，r = 最近随机点。(-244,58,-256) r=(-239,66,-255)（**y=66 > 海平面 63**）、s=(-247,49,-253)（y=49 < 63）、t=(-247,63,-247)
- **r/s/t 三列 y 分布跨海平面**：r 列（y=66 处列）液面应为 63（water），s 列（y=49 处列）液面可能 63 或 -32512（无效，取决于 13 邻居 est）
- C++ 返回 fl2.y==fl3.y==63 → e=0；**Java 若 s 列液面为 -32512（j=|63-(-32512)|≠0 → calculateDensity≠0 → e=d*6.0=3.84 → density+e=+3.77 → solid）**
- 需 Java 侧真实遍历中间量 dump（o/p/q/d/fl2.y/fl3.y/fl4.y/e/g/h）判别——AQF-J 反射不可信（phase5 L750 铁律），AQF-APPLY 只覆盖 density>0 路径（(-278,-240) y=12..23 全正）

## 四、范围判定汇总

- **C++ 范围内待修**（本课题产出）：
  1. aquifer 液面链输入（B3 机制，~5000-6710 块）——待 Java 中间量 dump 确认 fl2/fl3 值后修复
  2. surface.h P1 secondaryDepth 映射（sand/sandstone ~2900 块）
- **范围外已闭合**：含水层 5051（carvers）、深层 gravel（ore_gravel FEATURE）、结构（Beardifier 仅村庄 12 格内 <1000 块）
- **未定位**：海底 gravel surface rule 细分（P1 gravel noiseThreshold 差或 P2 相关，需 carvers 实现后重测）

## 五、下一步（judge 审查后）

1. **Java 探针 dump**（DensityProbe 扩展，真实遍历内打印 (-244,55..62,-256) 的 o/p/q/d/fl2.y/fl3.y/fl4.y/e/g/h，禁反射）→ 判别 (b)：fl.y 不同（修液面链）或 o/p 不同（理论上已排除）
2. **P1 修复**：surface.h StoneDepthCond secondaryDepth 映射对齐 Java（(int)map 不 clamp）
3. **carvers 实现后重测**海底 gravel

## 六、产物

- `.investigations/-288-unclosed/pipeline-map.md`（Phase 1 勘探）
- `.investigations/-288-unclosed/b1-beardifier.md` / `b2-aquifer-pocket.md` / `b3-aquifer-fluid.md`（fan-out 判定）
- `.investigations/-288-unclosed/verify_splitter2.py` + 输出（splitter 派生 8/8 逐位一致）
- `.investigations/-288-unclosed/rst_points.py` + 输出（r/s/t 点坐标）
- `.investigations/-288-unclosed/classify_m288.py` / `colview_m288.py`（子类归类 + 列形态）
