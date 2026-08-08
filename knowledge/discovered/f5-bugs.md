# discovered/f5-bugs — 还原工具误译及修正（跨版本通用）

> 从 CoreSwap 排查中提炼：工具/反射给出的值不可信的模式与修正方法。

## 发现 #1: javap 反编译不可直接信任

**发现时间:** 2026-08-08
**发现者:** worker（CoreSwap 逆向方法论）
**来源定位:** AGENTS.md 二、逆向方法论
**置信度:** confirmed
**module:** re-code

### 观察
javap 输出的字节码反编译（及混淆 jar）仅供参考，**Java 源码（yarn mappings + sources jar）才是权威**——混淆/优化会使 javap 输出与真实逻辑偏差（签名、常量折叠、异常表）。

### 证据
- CoreSwap 全程以 yarn mappings + sources jar 为准还原，逐位对齐通过（3200 100%）

### 如何利用
- 还原前先确认权威源（源码/mapping）；javap 只用于交叉验证，不一致时以源码为准

## 发现 #2: cns 反射 / CellCache 缓存污染（固定垃圾值）

**发现时间:** 2026-08-08
**发现者:** worker（density 排查，多次踩坑）
**来源定位:** DensityProbe 反射 ChunkNoiseSampler
**置信度:** confirmed（9 篇时间线多次记录）
**module:** re-code

### 观察
反射 `blockStateSampler.sample` / `CellCache.sample` 在非真实遍历状态下返回**缓存垃圾值**（如固定 `-0.024995`）；cns 的 interpolator 逐层值若遍历顺序与实际生成不一致也会错位（X/Z 顺序敏感）。

### 证据
- 9 篇时间线：「CellCache 反射污染：blockStateSampler.sample / CellCache.sample 在非真实遍历状态返回缓存垃圾值（如固定 -0.024995）——勿以反射作密度参照」
- 本 session：(810,76,-411) 参照=terracotta（真实方块）但 CellCache 反射=-0.038（air）——反射与真实生成矛盾

### 如何利用
- 密度参照以**真实导出**（BlockProbe 干净 world）为准，不以反射为准
- 必须反射时用完整 cns 链（sampleStartDensity→interpolateY/X/Z）在真实遍历内取值，且与真实生成交叉验证
- 参照导出铁律：删 run/world 后重导，防旧 chunk 复用

## 发现 #3: 密度无插值 vs 插值后口径混淆

**发现时间:** 2026-08-08
**发现者:** worker（8576 排查）
**来源定位:** DensityProbe（UnblendedNoisePos 直算）vs 游戏实际（cns 网格角点插值）
**置信度:** candidate
**module:** re-code

### 观察
`finalDensity().sample(UnblendedNoisePos)` = 无插值（Interpolated 节点直通）；游戏实际方块判定 = 网格角点缓存 + 三线性插值。两者在 range_choice 分支切换的陡峭区域差异可达 0.04+（角点 when_out 大正值拉高插值）。

### 证据
- (810,-411)：无插值 Java=-0.0397 ≈ C++ -0.038（一致），但参照（真实生成插值后）y=76 是方块——无插值/插值后口径必须分清

### 如何利用
- 对比时先声明口径（无插值直算 vs 游戏实际插值后）；定位插值差用 GRID 角点 dump（InterpolatedDF 网格值）
