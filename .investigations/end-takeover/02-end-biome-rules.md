# 02 — TheEndBiomeSource 生物群系判定规则（Java 1.20.1 → Rust 移植依据）

> 状态：**draft**（验证分层：**Partial** —— 纯静态源码审查，无运行时 probe 对拍）
> 分析角色：core.worker subagent；日期标签以主会话 Get-Date 记录为准
> Java 参照来源：`E:\PYTHON\CoreSwap\versions\1.20.1\data\mc_src_extract\`（yarn mappings 1.20.1 源码提取，含 `net.minecraft.registry` 包名 → 确认 1.19.3+ yarn 命名，工程版本 1.20.1）；噪声 JSON 参照：`versions\1.20.1\data\worldgen\data\minecraft\worldgen\noise_settings\end.json`（vanilla 报告提取）
> 以下所有行号均相对上列参照文件，已逐条读源核实。

---

## 1. 判定算法完整描述（伪代码级）

### 1.1 输入与坐标语义

`getBiome(int x, int y, int z, MultiNoiseSampler noise)`（TheEndBiomeSource.java L67-87）输入是 **biome 坐标**（4 块 = 1 biome 坐标），先转回块坐标：

```
i = x << 2   // BiomeCoords.toBlock，BiomeCoords.java L45-47
j = y << 2
k = z << 2
sectionX = i >> 4   // ChunkSectionPos.getSectionCoord，ChunkSectionPos.java L98-100
sectionZ = k >> 4
```

### 1.2 判定流程

```python
def get_biome(bx, by, bz, erosion_sampler):   # 输入 = biome 坐标
    i, j, k = bx<<2, by<<2, bz<<2             # 块坐标
    sx, sz = i>>4, k>>4                       # section 坐标（int 除 16，floor）
    if (long)sx*sx + (long)sz*sz <= 4096L:    # 中心判定，L73
        return the_end                        # 中心 Biome，半径 64 section（1024 块）
    # 采样点 = section 中心块坐标（L76-77）
    n = (sx*2 + 1) * 8        # 该 section 的中心块 X
    o = (sz*2 + 1) * 8        # 该 section 的中心块 Z
    d = erosion.sample(UnblendedNoisePos(n, j, o))   # L78（y=j 传入但见 §2.3 被忽略）
    if d > 0.25:       return end_highlands        # L79-80（严格大于）
    elif d >= -0.0625: return end_midlands         # L81-82
    elif d < -0.21875: return small_end_islands    # L84
    else:              return end_barrens
```

### 1.3 常量精确值（含类型）

| 常量 | 值 | 类型 | 出处 |
|---|---|---|---|
| 中心区平方距离阈值 | `4096L`（`<=`） | long（防溢出 cast：`(long)l*l + (long)m*m`） | TheEndBiomeSource.java L73 |
| highlands 下界（严格大于） | `0.25` | double 字面量 | L79 |
| midlands 下界（`>=`） | `-0.0625` | double | L81 |
| small islands 上界（`<`） | `-0.21875` | double | L84 |
| section 中心块公式 | `(sectionCoord*2 + 1) * 8` | int | L76-77 |

注意边界归属：`d == 0.25` → 不是 highlands 而是 midlands；`d == -0.0625` → midlands；`d == -0.21875` → barrens（不是 small islands）。

### 1.4 biome 选项的注册表查找方式

- 类 CODEC 用 `RegistryOps.getEntryCodec(BiomeKeys.THE_END / END_HIGHLANDS / END_MIDLANDS / SMALL_END_ISLANDS / END_BARRENS)`（L16-25）——五个 RegistryEntry<Biome> 作为可序列化字段。
- vanilla 实例：`createVanilla(biomeLookup)` → `biomeLookup.getOrThrow(...)` 五个 key（L32-40）。
- Rust 复刻：固定五个 biome id（`minecraft:the_end` / `end_highlands` / `end_midlands` / `small_end_islands` / `end_barrens`），从注册表按字符串 id 解析即可，无需 CODEC 通用性。

---

## 2. 依赖清单

### 2.1 唯一噪声依赖：end_islands（经 noise_router.erosion 通道）

`getBiome` 只调用 `noise.erosion()`（L78）。End 维度 noise_settings（end.json L21-26）：

```json
"erosion": { "type": "minecraft:cache_2d", "argument": { "type": "minecraft:end_islands" } }
```

即 erosion 通道 = **cache_2d 包裹的 end_islands 密度函数**（不是 any DoublePerlinNoise）。

### 2.2 end_islands 实现（DensityFunctionTypes.java L626-682）

- 状态：单个 `SimplexNoiseSampler`（L629）。
- **Random 派生（种子混入方式）**（L631-635）：

```java
Random random = new CheckedRandom(seed);   // seed = 世界种子（见 2.4）
random.skip(17292);                        // 精确值：17292 次 next
this.sampler = new SimplexNoiseSampler(random);
```

- 采样 `sample(sampler, x, z)`（L637-661，x/z = 块坐标/8，L665）：
  - `i = x/2, j = z/2`（Java int 除法，截断向零；k = x%2, l = z%2）
  - `f = 100.0F - sqrt(x*x + z*z) * 8.0F`，clamp 到 `[-100.0F, 80.0F]`（float 运算）
  - 邻域循环 `m,n ∈ [-12,12]`：若 `(i+m)² + (j+n)² > 4096L`（long 域）且 `sampler.sample(i+m, j+n) < -0.9F`（field_37677 = -0.9F，L628）：
    - `g = (|o|*3439.0F + |p|*147.0F) % 13.0F + 9.0F`（float）
    - `h = k - m*2; q = l - n*2`（float）
    - `r = 100.0F - sqrt(h*h + q*q) * g`，clamp `[-100.0F, 80.0F]`
    - `f = max(f, r)`
  - 最终输出（L665）：`(sample(...) - 8.0) / 128.0`（double）
  - min/max（L670/L675）：`-0.84375` / `0.5625`（与三阈值范围自洽：-0.84375 < -0.21875 < -0.0625 < 0.25 < 0.5625）
- 注意：**阈值判定用的是经 /128 归一后的值**，`sample()` 内部是 float 域——移植时 float/double 边界必须逐行对齐。

### 2.3 输入维度：cache_2d → y 无关

erosion 是 `cache_2d` 包裹 → 只依赖 x/z；`EndIslands.sample` 也只读 `pos.blockX()/8`、`pos.blockZ()/8`（L665）。`getBiome` 传入的 y（j）实际被忽略 ⇒ **end biome 判定是纯二维、按 section 粒度**（16×16 块一列一个 biome 值）。同一 section 内所有 y 的 biome 相同。

### 2.4 种子：world seed 直接传入

NoiseConfig.java L105（LegacyNoiseDensityFunctionVisitor，end.json `legacy_random_source: true` 命中此 visitor，NoiseConfig L115）：

```java
densityFunction instanceof DensityFunctionTypes.EndIslands
    ? new DensityFunctionTypes.EndIslands(seed) : densityFunction
```

`seed` 即 NoiseConfig 构造入参的世界种子 —— **直接用 worldSeed 构造 CheckedRandom，无 randomDeriver.split**（对比 terrain 等 split 派生，L102）。skip(17292) 后喂给 SimplexNoiseSampler（SimplexNoiseSampler.java L33-36：originX/Y/Z = random.nextDouble()*256.0，随后置换表）。

### 2.5 MultiNoise 完全不参与（假设验证 ✅）

- `getBiome` 签名接收 `MultiNoiseUtil.MultiNoiseSampler` 但**只访问 `.erosion()`**（L78），六个参数中五个不用。
- TheEndBiomeSource **覆写了 `getBiome`**（L66-87），不走 `BiomeSource` 默认的 MultiNoise SearchTree 最近邻路径 —— MultiNoise 参数匹配/SearchTree 在 end 维度完全不执行。
- ✅ 任务假设「位置判定非 MultiNoise」成立（静态源码层面确认）。

### 2.6 其他依赖

- `BiomeCoords.toBlock` = `coord << 2`（BiomeCoords.java L45-47）；`ChunkSectionPos.getSectionCoord` = `coord >> 4`（ChunkSectionPos.java L98-100）—— 移植注意负坐标算术位移语义（Rust `<<`/`>>` 对 i32 同义）。

---

## 3. Rust 移植评估

### 3.1 需要新写的组件

| 组件 | 现状 | 工作量 |
|---|---|---|
| `EndBiomeSource` 分类器（纯函数：section 距离 + erosion 值三段阈值） | 无 | 很小，~30 行；输入建议 `(biome_x, biome_y, biome_z)` 或 `(section_x, y_block, section_z)` + erosion sampler 引用 |
| `EndIslands` 密度函数（simplex 邻域 25×25 循环 + float 域公式） | 无 | 中等；核心是 float 精度逐行对齐 + `CheckedRandom(seed)+skip(17292)` 种子链 |
| `SimplexNoiseSampler`（2D/3D simplex） | **worldgen-core 无现成实现**：noise.rs 仅有 GRADIENTS 表（L13-19）供 PerlinNoiseSampler 使用，无 simplex 采样算法 | 中等（~100 行：置换表构造消费 Random 序列 + sample 算法） |
| `CheckedRandom::skip(17292)` | chunkrandom.rs CheckedRandom 已有 LCG；skip 循环调用即可（legacy_random.rs L112 已有 skip 先例） | 极小 |

### 3.2 噪声依赖：worldgen-core 现状

- `CheckedRandom` 已有（chunkrandom.rs L15-）；`RsRandom::skip` 已有（legacy_random.rs L112-113，但那是 RsRandom 层，CheckedRandom 层需补 skip 或用循环）。
- `SimplexNoiseSampler` 缺失——`GRADIENTS` 常量已在 noise.rs L13-19（16 梯度，与 SimplexNoiseSampler.java L7 一致），可复用。
- cache_2d：end 用法下可简化为按 section 列缓存或直接每次算（25×25 邻域 × 邻域内每点 1 次 simplex 采样 ≈ 625 次/sample，性能上建议按 (sx,sz) memoize）。

### 3.3 与现有 BiomeClassifier 的关系

现有 `worldgen-core/src/biome.rs` 的 `BiomeClassifier`（L225-）是 **MultiNoise SearchTree 最近邻**（1e-4 定点 long 域），服务 overworld/nether。End 分类器机制完全不同（section 距离 + 单密度函数阈值），**不建议复用/塞进 BiomeClassifier** —— 应**并列新写**一个轻量 `EndBiomeSource`（如 `biome_end.rs` 或 biome.rs 内独立模块），共享的只有：biome 注册表查找基础设施、`CheckedRandom`、（新写的）`SimplexNoiseSampler`（将来其他模块也可用）。接管入口在 `worldgen_handle.rs` 的维度参数化处按 settingsName 路由到 end 分类器。

### 3.4 对拍验证建议（Phase 2.5）

Java 侧可用 RouterProbe 打 end erosion 值（或直接用 96.06% 存档口径之外的 end 专用 dump）；Rust 侧对拍点：① 同 seed 下 `SimplexNoiseSampler` 置换表/origin 值 ② 同 (n,o) 下 erosion 值 ③ biome 边界（d 阈值附近的 section）。**验证前 MUST 核对两侧 worldSeed 一致（seed 三查纪律）**。

---

## 4. 证据索引

| 结论 | 证据（文件:行） |
|---|---|
| getBiome 全逻辑/三段阈值/中心 4096L | TheEndBiomeSource.java L67-87（阈值 L73/79/81/84） |
| 采样点 = section 中心块 `(sx*2+1)*8` | TheEndBiomeSource.java L76-77 |
| toBlock = <<2 | BiomeCoords.java L45-47 |
| getSectionCoord = >>4 | ChunkSectionPos.java L98-100 |
| erosion = cache_2d(end_islands) | end.json L21-26 |
| EndIslands: CheckedRandom(seed)+skip(17292)+SimplexNoiseSampler | DensityFunctionTypes.java L631-635；-0.9F 常量 L628 |
| EndIslands sample 公式/邻域/输出归一/min-max | DensityFunctionTypes.java L637-661, L665, L670, L675 |
| EndIslands 世界种子直传（无 split） | NoiseConfig.java L105（visitor 应用 L115；end legacy_random_source=true 见 end.json L10） |
| 只用 erosion、MultiNoise 不参与 | TheEndBiomeSource.java L78（唯一 sampler 访问）；getBiome 为覆写 L66 |
| biome 注册表查找（5 key） | TheEndBiomeSource.java L16-25（CODEC）、L32-40（createVanilla） |
| SimplexNoiseSampler 构造消费序列 | SimplexNoiseSampler.java L33-36（originX/Y/Z = nextDouble()*256） |

### 未核实声明（@anchor.idk 式）

- **未核实**：`mc_src_extract` 提取的精确上游（sources jar 具体 yarn build 号）——包名/类容与 1.20.1 官方报告 JSON 一致，判定按 1.20.1，但 jar 元数据未核。
- **未核实**：end.json 是否与 1.20.1 原版 data report 逐字节一致（本仓库 worldgen/data 提取链的来源未回查原版 jar hash）。
- **未核实（运行时）**：上述全部为静态审查结论；erosion 采样数值、阈值边界行为、MultiNoise 排除均无运行时 probe 佐证 —— Phase 2.5 需按 §3.4 对拍后才能升 candidate。
- **未核实**：`SimplexNoiseSampler.sample` 2D 入口的完整实现细节（本次未通读该文件全文，仅核构造消费序列）——移植前需通读 SimplexNoiseSampler.java 全文。

---

## 5. 结论速览（TL;DR）

1. End biome 判定 = 纯二维 section 级规则：`sx²+sz² <= 4096` → the_end；否则按 `erosion` 值三段分：>0.25 highlands、>=-0.0625 midlands、<-0.21875 small_islands、其余 barrens。y 无关（cache_2d）。
2. 唯一噪声 = end_islands（simplex 邻域形态函数），种子 = worldSeed 直传 CheckedRandom + skip(17292)；**MultiNoise/SearchTree 完全不参与（假设已验证）**。
3. Rust：新写 `EndBiomeSource` + `EndIslands` + `SimplexNoiseSampler`（复用已有 GRADIENTS/CheckedRandom）；与 MultiNoise BiomeClassifier 并列，不复用框架。
