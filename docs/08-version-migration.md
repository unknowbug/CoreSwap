# 8. 版本迁移方法论（迭代 1.17.x 等）

> 目标：迁移到新版本时，不看 C++ 实现细节，只按此流程：
> **diff Java 核心类 → 对照本库「版本敏感点」清单 → 增删改查 → 探针验证**。

## 迭代流程（六步）

### 第 1 步：准备新版本 Java 参照

1. 用与 1.20.1 相同的反编译/解压流程产出 `data/mcsrc_<ver>/`（fabric yarn 源码）。
   - 关键类必须齐全：`MathHelper`、`XoroshiroRandom`、`RandomSplitter`、`DoublePerlinNoiseSampler`、
     `DensityFunctionTypes`、`NoiseConfig`、`DensityFunctions`、`NoiseChunkGenerator`、`ChunkNoiseSampler`、
     `AquiferSampler`、`OreVeinSampler`、`VanillaSurfaceRules`、`MaterialRules`、`SurfaceBuilder`、`DimensionType`。
2. 导出新版本数据包到 `data/worldgen/`（noise_settings / density_function / noise / biome / blocks.json）。
3. **重构 Java 参照 vanilla**（见第 6 步的缓存污染警告）。

### 第 2 步：diff Java 两个版本（老版 vs 新版）

重点 diff 清单（按影响面排序）：

| 文件 | 关注点 | 本库对应篇 |
|---|---|---|
| `MathHelper.hashCode` | hashXYZ 公式 | 02 |
| `RandomSplitter`/`XoroshiroRandom` | split/nextSplitter 语义 | 02 |
| `DensityFunctionTypes` | 节点类型集合、codec 注册 | 03 |
| `DensityFunctions` | noise_router 动态构造（vein_* 等） | 03/05 |
| `NoiseConfig` | randomDeriver 派生链、Legacy visitor | 03 |
| `AquiferSampler` | blob 间距、无效液面、estimateSurfaceHeight | 04 |
| `OreVeinSampler` | 阈值、VeinType 范围 | 05 |
| `VanillaSurfaceRules` | materialRule1..10 分支归属 | 06 |
| `MaterialRules` | StoneDepth/Water/surface 公式 | 06 |
| `SurfaceBuilder` | s/q/vx 语义 | 06 |
| `DimensionType` | field_35479（无效液面常量） | 04 |
| `NoiseChunkGenerator` | 块级流水线、fluidLevelSampler | 07 |

diff 技巧：**逐类 diff 后，先对照「版本敏感点」清单打勾**，把「没变的」划掉，只改「变了的」。
清单是 `[ ]` 形式，改完勾上。

### 第 3 步：按敏感点清单改 C++

逐篇执行 `[ ]` 检查项。大多数改动是常量/公式级（阈值、间距、边界、scale），
少数是结构级（新增/删除节点类型、materialRule 分支增删）。

### 第 4 步：探针验证（逐层对齐）

| 探针 | 用途 | 命令 |
|---|---|---|
| `ore_probe.exe` | veinToggle/veinRidged/veinGap 三件套 + apply 决策 | `ore_probe <seed> data/worldgen` |
| Java RouterProbe | Java 侧无插值分量对照 | `-ProuterProbe=true` |
| Java OreProbe | Java 侧插值复刻对照 | `-PoreProbe=true` |
| `block_probe.exe` | 全方块对比 | 见下 |
| `diag_full.py` | 差异构成（方块对 + y 分布） | `python data/diag_full.py` |
| VeinDiag（BlockProbe.java） | 驱动真实 ChunkNoiseSampler 取块级真值 | 见 `driveCnsTo()` |

验证顺序：**随机（RouterProbe 采样点一致）→ 密度场（noise_probe）→ 方块层（block_probe）**。

### 第 5 步：性能回归

- 迁移后跑 `block_probe`（并行）确认耗时量级；若 aquifer 占比异常，检查列缓存是否丢失（04/07 篇）。
- 线程数：默认自适应，勿写死本机值。

### 第 6 步：⚠️ 参照数据卫生（必做）

**重新导出 vanilla 参照前必须删 `versions/1.20.1/java/run/world/region/` 下测试区域的 .mca 文件**
（r.5.5/5.6/6.5/6.6 等），否则 `world.getChunk()` 复用旧 chunk 缓存，导出污染参照，
对比出**假差异**（历史教训：99.78%→97.73% 假回归，矿脉「零输出」假象）。

## 已知坑速查（索引）

- 随机：int/long 乘法精度、算术右移、nextSplitter 有状态（02）
- 密度：非线性不可先采样后插值、range_choice.fill 特殊、old_blended_noise 的 terrain split（03）
- 含水层：-32512 无效液面、4 格对齐、blob 间距、列缓存（04）
- 矿脉：vanilla 参照污染假象、分量动态构造、interpolator 识别特征（05）
- 表面：STONE_DEPTH `<=1+offset`、mr7/mr8 分支归属、s 判定集合（06）
- 流水线：块级插值顺序、线程默认自适应、多线程一致性验证（07）

## 数据/工具链（E:\python\MC）

- C++ 编译（MSVC，严格禁用 MinGW）：`cmd /c "call "D:\Program Files\Microsoft Visual Studio\18\Community\VC\Auxiliary\Build\vcvars64.bat" && set PATH="D:\Program Files\Microsoft Visual Studio\18\Community\Common7\IDE\CommonExtensions\Microsoft\CMake\Ninja";%PATH% && cmake --build "versions\1.20.1\cpp\build-msvc""`
- block_probe：`block_probe.exe <seed> data\worldgen data\vanilla_*.blocks [-threads N]`
- Java 导出 vanilla：`gradle runServer -PblockProbe=true -PbenchSeed=<seed> -PbenchSize=4 -PbenchOriginX=3200 -PbenchOriginZ=3208`
- Java 探针：`-ProuterProbe=true` / `-PoreProbe=true`
- 提交纪律：author=unknowbug，中文提交信息，进度及时 push
