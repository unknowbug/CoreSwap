# 8576-24blocks forest terracotta + river 3 块 mismatch 根因分析

> 项目：CoreSwap（MC 1.20.1 世界生成 C++ 复刻，逐位对齐 vanilla）
> seed=8576294172403134396，区域 720,-432 6×6 chunks（chunk x 45..50, z -27..-22）
> 参照：`versions/1.20.1/data/vanilla_8576294172403134396_6_720_-432.blocks`（SURFACE 状态）
> 范围：#23 (812,73,-337) stone vs terracotta、#24 (815,89,-337) grass vs terracotta、#15 (733,26,-382) stone vs water
> 角色：anchor.worker 精确分析（只读，不改代码/参照）　日期：2026-08-09　状态：**draft**

---

## 0. 结论先行

- **biome 段索引确认（前置）**：blocks 文件 biome 段**正确索引 = z*16+x**（外层 z、内层 x）。参照真实 biome：**(812,-337)=badlands**、(815,-337)=forest、(733,-382)=river（均 y=100 采样）。
- **#23（真 bug）**：C++ 在 (812,73,-337) 判 **forest**，vanilla 判 **badlands** → badlands 深层 terracotta 带规则不触发 → stone。参照列 y=73 terracotta 带是 badlands 表面规则产物，铁证。
- **#24（真 bug）**：C++ 在 (815,89,-337) 判 **forest**（grass 顶替 terracotta），vanilla 判 **badlands** → terracotta 带顶差 1。带公式（lround(offset*4)+floorMod(y,192)）两版一致，差异在 **biome 判定翻转边界**：vanilla badlands→forest 翻转在 y=90，C++ 在 y=89（早 1 格）。
- **#15（非 biome 问题，引用 workerB）**：biome 判定（river）一致，河床/水底高 1 = aquifer 液面/InterpolatedDF 插值精度边界翻转，见 `aquifer-wateredge/analysis.md`。
- **共同根因方向（#23/#24）**：C++ 六维 biome 分量（temperature/vegetation/continents/erosion/depth/ridges）在某 y 边界的采样与 Java 有差，导致 badlands↔forest 最近 hypercube 翻转。**需运行时六维分量对比定位具体分量**（见 §6）。

---

## 1. biome 段索引确认（前置）

### 1.1 blocks 文件布局（Java 写序，源码证据）

`E:\PYTHON\MC\versions\1.20.1\java\src\main\java\wg\bench\BlockProbe.java`：

- **blocks 段（L773-780）**：`for y { for z { for x { out.writeShort(blockId) } } }` → 块索引 `i = y*256 + z*16 + x`（x 最内层，z 中层，y 外层）。与 `read_col2.py` 解析（`lx=i%16; lz=(i//16)%16; ly=i//256`）一致 ✓
- **biome 段（L782-788）**：`for z { for x { out.writeUTF(biome) } }` → 索引 **`i = z*16 + x`（z 外层，x 内层）**

### 1.2 read_biome2.py 双索引判定

```python
for idx, desc in [((lx*16+lz), "x*16+z"), ((lz*16+lx), "z*16+x")]:
```
- `x*16+z`（lx*16+lz）= **错误**索引
- `z*16+x`（lz*16+lx）= **正确**索引（与 Java 写序一致）

### 1.3 各列正确 biome（y=100）

| 列 | x*16+z（错） | z*16+x（对） | 参照真实 biome |
|---|---|---|---|
| (812,-337) | forest | **badlands** | badlands |
| (815,-337) | forest | **forest** | forest（y=100） |
| (733,-382) | savanna | **river** | river |

> scout 风险提示中的「双索引不一致」（812: forest/badlands、733: savanna/river）已解决：**z*16+x 为正确索引**。

---

## 2. #23 (812,73,-337) got=1 stone vanilla=494 terracotta —— 真 bug（biome 判定差异）

### 2.1 形态（scout column-profiles）

参照列 (812,-337)：y=60..72 stone、**y=73 terracotta**、74..82 stone、83..85 dirt、86 grass。C++：y=60..82 stone（**73 缺失**）、83..85 dirt、86 grass。地表完全一致，仅深层 terracotta 带整层缺失。

### 2.2 参照 biome 判定

- blocks 文件 biome 段 (812,-337) z*16+x = **badlands**（y=100）
- 参照列 y=73 terracotta：badlands 深层 terracotta 带是 `biome(BADLANDS, ERODED_BADLANDS, WOODED_BADLANDS)` 段专属（VanillaSurfaceRules.java:207 起 `MaterialRules.terracottaBands()`）→ **vanilla 在 y=73 判 badlands**（否则不会生成 terracotta 带）

### 2.3 C++ 判定

- `-biomeDump(812,73,-337)` = **forest**（scout 实测）
- C++ 判 forest → badlands 段不触发 → y=73 保持 stone

### 2.4 结论

**C++ 与 vanilla 在 (812,73,-337) 的 biome 判定不同**（C++ forest vs vanilla badlands）。这是真 bug，不是假 diff（参照 terracotta 为表面规则产物，非结构）。

### 2.5 候选根因（需运行时验证）

C++ 六维分量采样 vs Java：

| 分量 | Java 来源 | C++ 来源 | 对拍 |
|---|---|---|---|
| temperature | noiseRouter.temperature()（shifted_noise） | h->router["temperature"]（buildNode shifted_noise） | 结构等价 |
| vegetation | noiseRouter.vegetation() | h->router["vegetation"] | 等价 |
| continents | noiseRouter.continents()（overworld/continents 引用→flat_cache shifted_noise） | h->router["continents"] | 等价 |
| erosion | noiseRouter.erosion() | h->router["erosion"] | 等价 |
| depth | noiseRouter.depth() | h->router["depth"] | 等价 |
| ridges | noiseRouter.ridges() | h->router["ridges"] | 等价 |

- 采样坐标：Java `UnblendedNoisePos((px<<2),(py<<2),(pz<<2))`（BiomeCoords.toBlock）；C++ `p.x=px<<2; p.y=py<<2; p.z=pz<<2`。**一致**。
- find：Java `Entries.getValue`（SearchTree）vs C++ `biomeSource.find`（线性遍历全 entries）。语义等价；tie（距离相等）处理可能不同——搜索树 vs 线性在平局时返回顺序可能有别（`getValueSimple` 用严格 `<`，SearchTree 用节点遍历）。**候选差异点之一**。
- **hashSeed 已排除**：Guava `Hashing.sha256().hashLong(seed).asLong()` 前 8 字节 **little-endian**（官方文档 + 源码 BytesHashCode.padToLong 确认），与 C++ `biomeHashSeed`（`out |= dg[i] << (8*i)`）一致。

---

## 3. #24 (815,89,-337) got=8 grass vanilla=494 terracotta —— 真 bug（biome 翻转边界差 1）

### 3.1 形态

参照列 (815,-337)：y=84 orange_terracotta(426)、85 stone、86 dirt、**87 white(425)+88 terracotta(494)+89 terracotta(494)**、90 air。C++：84 orange(426)、85 stone、86 dirt、87 white(425)、88 terracotta(494)、**89 grass(8)**、90 air。

→ 参照带顶 y=89（terracotta），C++ 带顶 y=88，**y=89 被 grass 顶替**，带短 1 格。

### 3.2 terracotta 带公式对拍（排除带公式 bug）

| 项 | Java SurfaceBuilder | C++ getTerracottaBlock（surface.h:401-406） |
|---|---|---|
| offset 噪声 | `terracottaBandsOffsetNoise.sample(x,0,z) * 4.0` | `clay_bands_offset.sample(x,0,z) * 4.0` |
| 取整 | `(int)Math.round(...)` | `(int)std::lround(...)` |
| 索引 | `floorMod(y+i, 192)` | `((y+i) % n + n) % n`（n=192） |
| 带数组 | 192 元素 terracottaBands | 192 元素 terracottaBands（clay_bands random 同源） |

**带公式一致**。y=87/88 两版同出 white/terracotta 证明带数组与取整一致；差异只在 y=89。

### 3.3 根因

- 参照 y=87..89 terracotta → vanilla 在 y=87..89 判 **badlands**（terracotta 带规则触发）
- C++ y=87/88 terracotta（判 badlands ✓）、y=89 grass（判 forest ✗）
- **C++ badlands→forest 翻转边界 = y=89，vanilla = y=90**（差 1 格）

参照 blocks 文件 biome 段 (815,-337)=forest（y=100）与参照列 terracotta 带并存 → 证明 vanilla 该列 y 方向**确实存在 badlands→forest 翻转**（y=87..89 badlands、y=100 forest）；C++ 翻转早 1 格。

> scout 疑「参照 biome 段=forest 却带 terracotta 带，疑 biome 段记录问题」——非记录问题：biome 是 3D 的（BiomeAccess.getBiome 选点含 py），同列不同 y 可以不同 biome。blocks 文件 biome 段只记录 y=100。

### 3.4 与 #23 的关系

#23 与 #24 同源：**C++ 在 badlands 区域某 y 边界判 forest 比 vanilla 早/多**。#23 更极端（整带缺失），可能因 y=73 已落在 C++ 的 forest 区；#24 只差带顶 1 格。两者都指向 C++ 六维分量在 y 方向（depth 或 temperature 的 y 依赖）采样差异或 find tie。

---

## 4. #15 (733,26,-382) got=1 stone vanilla=32 water —— 非 biome 问题（引用 workerB）

- 参照 river 主水体 y=14..26（13 格），C++ y=14..25（12 格），y=26 被 stone 顶替 → **C++ 河床/水底高 1**
- blocks 文件 biome 段 (733,-382) z*16+x = **river**；C++ -biomeDump(733,26,-382)=**river** → **biome 判定一致**
- 结论：非 biome 问题。与深板岩/水边界 12 块同机制（aquifer 液面/InterpolatedDF 插值精度边界翻转），完整分析见 `../aquifer-wateredge/analysis.md`（workerB，draft）。

---

## 5. Diag810 类诊断：真 bug vs 假 diff

| 块 | 判定 | 证据 |
|---|---|---|
| #23 | **真 bug**（C++ biome 判定差异） | 参照 y=73 terracotta = badlands 表面规则产物；C++ forest；blocks biome 段 badlands 佐证 |
| #24 | **真 bug**（C++ biome 翻转边界早 1） | 参照 y=87..89 terracotta 带 vs C++ y=89 grass；带公式两版一致 |
| #15 | **非 biome 假 diff 排除**，aquifer 插值边界 | biome 判定一致（river），河床差 1 与 aquifer 同机制 |

排除项：
- **FEATURE/结构假 diff**：terracotta 带是 surface rule 产物，非结构；参照为 SURFACE 状态（10-timewise L328），无结构方块。
- **hashSeed 字节序**：Guava asLong 为 little-endian，C++ 一致。
- **带公式**：lround+floorMod 一致。
- **blocks 文件 biome 段记录**：非记录问题，是 y=100 采样 vs 实际 y 的 3D biome 差异。

---

## 6. 未验证项与验证命令（主会话补跑）

本 worker 只读沙箱拦截 exe 调用，无法运行时采样。建议主会话补跑：

1. **C++ 六维分量 dump（定位 #23/#24 差异分量）**：
   ```
   WG_COMPDUMP=1 WG_COMPDUMP_X=812 WG_COMPDUMP_Z=-337 block_probe.exe 8576294172403134396 versions\1.20.1\data\worldgen versions\1.20.1\data\vanilla_8576294172403134396_6_720_-432.blocks
   WG_COMPDUMP=1 WG_COMPDUMP_X=815 WG_COMPDUMP_Z=-337 ...
   ```
   对比 Java RouterProbe `-Drouter.x=812 -Drouter.z=-337 -Drouter.yFrom=60 -Drouter.yTo=100 -Drouter.yStep=4` 的 `B` 行（6 维分量）。关注 **depth/temperature 随 y 的翻转点**。

2. **C++ -biomeDump y 扫描**：
   ```
   block_probe.exe ... -biomeDump 812 68 -337 / 812 70 -337 / 812 73 -337 / 812 76 -337 ...
   block_probe.exe ... -biomeDump 815 86 -337 / 815 87 -337 / 815 88 -337 / 815 89 -337 / 815 90 -337 ...
   ```
   确认 C++ 的 badlands→forest 翻转边界与 vanilla 差几格、差在哪个分量。

3. **Java RouterProbe SURFBIOME 对照**（8 邻域选点后实际 biome）：
   ```
   -Drouter.x=812 -Drouter.z=-337 -Drouter.yFrom=60 -Drouter.yTo=100
   -Drouter.x=815 -Drouter.z=-337 -Drouter.yFrom=60 -Drouter.yTo=100
   ```

4. **#15 river**：见 workerB 产物命令模板（WG_SURFDUMP 剖面）。

---

## 7. 修复方向（供主会话/后续 worker 参考，不改代码）

- #23/#24：定位 C++ 六维分量与 Java 的差异点（§6 命令）。若为 find tie：C++ 应复刻 Java SearchTree 遍历序或确认 `dist < bestDist`（严格小于）与 Java `m < l` 一致（C++ biome.h find 已用严格 `<`，与 getValueSimple 一致）；若为分量采样（depth/ridge 的 y 依赖），需对拍对应 density function 实现。
- #15：aquifer 液面/InterpolatedDF，见 workerB 修复方向。

---

## 8. 产物索引（供主会话合并到 .artifacts/8576-24blocks/index.yaml）

```yaml
  - id: 're-code:8576-24blocks:biome-terracotta'
    path: 'biome-terracotta/analysis.md'
    kind: analysis
    status: draft
```

> 根 `.artifacts/8576-24blocks/index.yaml` 因 worker 写权限边界由主会话更新；本子目录另有 `index-entry.yaml` 片段。
