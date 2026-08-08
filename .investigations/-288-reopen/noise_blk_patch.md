# BlockProbe 最小 patch：NOISE 阶段直读 (-244,-256) 列（NOISE-BLK）

> 产出角色：recode.scout（只读勘探 + patch 描述，主会话负责应用）
> 日期：本次分析
> 目标：判别「岛（FULL 参照 y=58=stone）」是 **aquifer 判定产物**（NOISE 阶段即 stone，C++ 判 water 是 bug）
>      还是 **ocean_ruin 结构覆盖**（NOISE 阶段为 water/air，stone 由后续结构生成，C++ 无 bug）

## 0. 场景确认（来自参照列 FULL 状态）

`ref_col_-244_-256.txt`（seed=-8248318472910187742, origin=(-288,-256)）：

| y | FULL raw | 块 |
|---|----------|-----|
| 40..50 | 1 | stone |
| 51..57 | 32 | water |
| **58** | **1** | **stone（岛，判别目标）** |
| 59..61 | 9 | dirt |
| 62 | 32 | water |
| 63..66 | 0 | air |

FULL 阶段 y=58 为 stone 但上下均为 water/dirt/air → 需 NOISE 阶段（populateNoise 完成、无 surface/结构干扰）同列剖面判定来源。

## 1. 插入点描述

**文件**：`src/main/java/wg/bench/BlockProbe.java`

**插入位置**：`run()` 方法主导出循环内，L477 `Chunk chunk = world.getChunk(wx, wz, ChunkStatus.NOISE, true);` 之后，
L478 `if (wx == -16 && wz == -16) {` 块内，**L479（EstDiagN 打印）之后、L480（if 闭合 `}`）之前**。

现状锚点（行号为本文件当前行号）：

```java
477:                    Chunk chunk = world.getChunk(wx, wz, ChunkStatus.NOISE, true);
478:                    if (wx == -16 && wz == -16) {
479:                        System.out.println("[EstDiagN] chunk status=" + chunk.getStatus() + " class=" + chunk.getClass().getSimpleName());
   :                        <=== 在此插入 NOISE-BLK 块（L480 的 `}` 之前，if 块内）
480:                    }
481:                        // EstDiag-288：...（L482-575 大 try-catch，含 AQF-J；本次不触碰）
```

**选择该点的理由**：
- L477 的 `chunk` 变量已生成到 `ChunkStatus.NOISE`，`getBlockState` 返回的即为 populateNoise 填充结果（aquifer apply + oreVein apply + defaultBlock），无 structure/surface 干扰——与任务背景一致。
- 插在 L479 之后保证**只在 chunk(-16,-16) 触发**（现有代码 L481 起的 try-catch 缩进疑似已脱离该 if，为避免纠缠，本 patch 自置于 if 块内，确定只跑一次）。
- 独立 try-catch，不触碰 L481-575 现有 EstDiagN/AQF-J 逻辑。

**坐标换算**：chunk(-16,-16) 世界原点 = (-256,-256)。
目标世界坐标 (-244,-256) → chunk 局部 `(x=12, z=0)`（-244−(−256)=12；−256−(−256)=0）。
`Chunk.getBlockState(BlockPos)` 使用 **chunk 局部坐标**（与 L418/L431 现有用法一致）。

## 2. 完整 Java 代码片段

```java
                        // NOISE-BLK：chunk(-16,-16) NOISE 阶段直读 (-244,-256) 列（局部 (12,y,0)）
                        // 判别「岛(y=58 stone)」是 aquifer 判定产物（NOISE 即 stone，C++ 判 water 是 bug）
                        // 还是 ocean_ruin 结构覆盖（NOISE 为 water/air，stone 由结构后续生成，C++ 无 bug）
                        try {
                            BlockPos.Mutable nbpos = new BlockPos.Mutable();
                            for (int y = 40; y <= 66; y++) {
                                net.minecraft.block.Block bb = chunk.getBlockState(nbpos.set(12, y, 0)).getBlock();
                                int raw = net.minecraft.registry.Registries.BLOCK.getRawId(bb);
                                String nm = net.minecraft.registry.Registries.BLOCK.getId(bb).getPath();
                                System.out.println("[NOISE-BLK] y=" + y + " raw=" + raw + " " + nm);
                            }
                        } catch (Throwable exB) {
                            System.out.println("[NOISE-BLK] ERR " + exB);
                        }
```

说明：
- `y=40..66` 全列循环，覆盖任务要求的 55..62 判别带 + 参照列 FULL 剖面（40..66）逐行 diff。
- 局部变量 `nbpos` 自行声明——L415 的 `rpos` 作用域止于 RuleDiag 独立块（L338 `if (rule.diag)` 内），此处不可复用；`pos`（L469）虽在作用域内但不复用，避免与其他导出逻辑耦合。
- 输出格式与 `ref_col_-244_-256.txt` 的 `y=NN raw=X name` 对齐（`getPath()` 去掉 `minecraft:` 前缀），便于直接对比。
- raw id 由 `Registries.BLOCK.getRawId(Block)` 给出（stone=1, water=32, air=0, dirt=9）。

## 3. 编译注意

**现有 import（文件头 L3-25，完整列表）**：

```java
net.minecraft.block.Block
net.minecraft.registry.Registries
net.minecraft.server.MinecraftServer
net.minecraft.server.world.ServerChunkManager
net.minecraft.server.world.ServerWorld
net.minecraft.util.math.BlockPos
net.minecraft.util.math.ChunkPos
net.minecraft.world.biome.Biome
net.minecraft.world.chunk.Chunk
net.minecraft.world.chunk.ChunkStatus
net.minecraft.world.gen.chunk.ChunkGenerator
net.minecraft.world.gen.chunk.NoiseChunkGenerator
java.io.BufferedOutputStream / DataOutputStream / FileOutputStream
java.nio.charset.StandardCharsets / java.nio.file.Files / java.nio.file.Path
java.util.ArrayList / List / Map / TreeMap
```

**结论：patch 不引入任何新 import**。
- `BlockPos`（含内部类 `BlockPos.Mutable`）已有（L8）；
- `Block` 已有（L3）；
- `Registries` 已有（L4），`Registries.BLOCK.getId/getRawId` 用法与 L404-405、L418-421 现有代码完全一致；
- `chunk` 局部变量、`Throwable`（java.lang）均无需 import。

## 4. 验证方式

```bash
gradle runServer -PblockProbe -PbenchSeed=-8248318472910187742 -PbenchSize=4 -PbenchOriginX=-288 -PbenchOriginZ=-256
```

预期输出（stdout，混在现有诊断中）：

```
[NOISE-BLK] y=40 raw=1 stone
...
[NOISE-BLK] y=55 raw=32 water      <- 若 y=58 为 stone：aquifer 已判 solid，岛是 aquifer 产物
[NOISE-BLK] y=56 raw=32 water
[NOISE-BLK] y=57 raw=32 water
[NOISE-BLK] y=58 raw=1 stone       <- 关键判别行
[NOISE-BLK] y=59 raw=9 dirt
[NOISE-BLK] y=60 raw=9 dirt
[NOISE-BLK] y=61 raw=9 dirt
[NOISE-BLK] y=62 raw=32 water
...
```

**判别规则**：
- `y=58` NOISE 阶段为 **stone（raw=1）** → aquifer 在此判 solid，FULL 的 stone 是 aquifer 产物；C++ 判 water 为 **bug**（与 FULL 参照一致：FULL y=58=stone，C++ 若判 water 则错）。
- `y=58` NOISE 阶段为 **water（raw=32）** 或 **air（raw=0）** → aquifer 判水，FULL 的 stone 是 ocean_ruin（或 surface/dirt 等）结构覆盖；C++ 判 water **无 bug**。

**环境确认**（已核对）：
- `build.gradle` L50-58/L77 已定义 `runServer` task 及 `-PbenchSeed/-PbenchSize/-PbenchOriginX/-PbenchOriginZ/-PblockProbe` 全部属性（映射到 `-Dbench.seed` 等）。
- 该命令下 `wx = originX/16 + cx = -288/16 + cx = -18 + cx`（cx=0..3 → -18..-15，含 -16 于 cx=2）；`wz = -256/16 + cz = -16 + cz`（含 -16 于 cz=0）→ chunk(-16,-16) 必触发。
- 对照：`ref_col_-244_-256.txt` 头部注明来源 `vanilla_-8248318472910187742_4_-288_-256.blocks`，与验证命令参数一致。

## 5. 附带观察（不改动）

- L481 起的 `try`（EstDiagN/AQF-J）缩进（24 空格）与其上 L480 `}`（20 空格）不一致，疑似已脱离 `if (wx == -16 && wz == -16)`，会对每个 chunk 执行反射诊断。本次 patch 不触碰；若输出中出现大量重复 `[AQF-J]/[EstDiagN]` 即佐证。如需修正括号请主会话另行裁决。
- 参照列文件存在于 `.investigations/-288-reopen/ref_col_-244_-256.txt`，可与 `[NOISE-BLK]` 输出逐行 diff。
