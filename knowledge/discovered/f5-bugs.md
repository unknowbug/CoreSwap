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

## 发现 #4: 探针采样坐标语义三套（floor 对齐 / 8 邻域选点 / 原始直采）

**发现时间:** 2026-08-08
**发现者:** worker（#23/#24 forest terracotta 排查）
**来源定位:** RouterProbe / C++ biomeDump / WG_COMPDUMP
**置信度:** confirmed（已写入 AGENTS.md 四·探针/参照数据采集核对铁律）
**module:** re-code

### 观察
同一坐标点在不同探针里的采样口径**不同**，直接对比会得出错误结论：
- **floor 对齐**：RouterProbe `B`/`SURFBIOME` 行 = `(x>>2)<<2`（biome 格对齐；SURFBIOME 打印 bp 对齐坐标，但判定输入是原始 BlockPos）
- **8 邻域选点**：C++ `-biomeDump`/`WG_BIOMEDUMP` = BiomeAccess 8 邻域选点后 `(px<<2,py<<2,pz<<2)`
- **原始直采**：`WG_COMPDUMP` = 原始块坐标直采（无对齐/无选点）

### 证据
- #23/#24 曾因 -337 vs -336/-340 坐标错位误判「湿度差 0.0054」——实际是不同工具坐标语义不同
- 2×2 参照导出曾混入范围外 chunk（`chunk(65515,65515)` int16 溢出坐标）→ TOTAL 异常

### 如何利用
- 跨工具同点对比 MUST 先确认坐标语义（对齐/选点/直采），再比数值
- 参照 blocks 导出后检查 header（magic/seed/size/origin）+ chunk 范围 + TOTAL 合理性；seed 三查（server.properties level-seed 备份 → 删 world → 输出 #seed/worldSeed 核对）

## 发现 #5: 引用 Mojang 反编译/源码树作参照前必须核版本/DataVersion——版本不符的源码与实测行为矛盾时以实测为准（260905-03）

- **发现时间**：260905-03；**发现者**：core.worker 草稿（light-opt round-trip 判据设计）；**置信度**：candidate（实测矛盾 + 源码树版本疑点，judge 待走）；**module**：re-code / 源码参照核验。

### 观察
设计「存档 round-trip 光照不变」判据时，参照 `.tmp/net` Mojang 源树中 isLightOn 的序列化写入逻辑，得出「存档应有 isLightOn 键」的预期。实测 vanilla/rust 双侧存档**全量无 isLightOn 键**（口径：2025 chunk 全量 NBT 键扫描）——预期与实测矛盾；回查发现 `.tmp/net` 源树疑非 1.20.1（其 isLightOn 写入与 1.20.1 实测行为不符）。

### 根因
反编译/源码树是**某一版本**的快照，参照前若不核版本（DataVersion / build 元数据），会把别的版本的序列化行为当成当前版本事实——与「javap 不可直接信任」同族：问题不在源码假，在「拿错版本的真源码」。

### 教训/如何利用
1. **判据**：任何反编译/源码树被引用为行为参照前，第一步核其版本锚（DataVersion、gradle/loom 元数据、路径内版本号），与课题目标版本一致才可引用。
2. **源码 vs 实测矛盾时以实测为准**，并回查源码树版本——这本身就是版本错位的判别签名。
3. 家族索引：本文件 #1（javap 不可信——静态产物需版本锚）、compiler-idioms #13（docs 口径先一手源码核对——本条为「源码先核版本」对偶面）。
