# BlockProbe 最小 patch：反射 StructureWeightSampler（Beardifier）采样含水层列 (-278,15..19,-240)

> 产出角色：recode.scout（只读勘探 + patch 描述，主会话负责应用）
> 日期：本次分析
> 目标：验证「含水层判水 = ocean_ruin Beardifier 负修正」假设
> 唯一可写路径：`.investigations/-288-reopen/beardifier_patch.md`；未修改任何源文件

---

## ⚠️ 结论先行：静态源码分析强烈预示「ocean_ruin 假设大概率不成立」，但 patch 仍值得跑（排除其他 adaptation 结构干扰）

| 判定 | 证据 | 置信度 |
|------|------|--------|
| **ocean_ruin 不进 Beardifier** | `ocean_ruin_cold.json`/`ocean_ruin_warm.json` **无 `terrain_adaptation` 字段**；`Structure.java` L207 默认 `StructureTerrainAdaptation.NONE`；`StructureWeightSampler.createStructureWeightSampler` L41 过滤 `structure.getTerrainAdaptation() != StructureTerrainAdaptation.NONE` | 高（源码直读） |
| shipwreck/buried_treasure 同样不进 | 二者 JSON 亦无 `terrain_adaptation`；只有 ancient_city(beard_box)、nether_fossil/pillager_outpost/villages(beard_thin)、stronghold/trail_ruins(bury) 有 | 高 |
| Beardifier 值域可能为 0 | 若 (-278,-240) 24 格内无上述 adaptation 结构，`sample` 返回 `0.0`（pieceIterator/junctionIterator 均为空迭代器） | 待实测 |
| 假设若被推翻 → 判水根因另寻 | 需转向 aquifer 内部逻辑（fluid level 随机 / blending / ore_vein 之外的 apply 路径） | 待主会话 |

**架构变更建议（显式标注，交主会话裁决）**：任务背景假设「ocean_ruin（纯 stone）在附近 → Beardifier ≠ 0 拉低 density」在 1.20.1 源码层面**不成立**——ocean_ruin 的 `terrain_adaptation=NONE`，根本不会被 `StructureWeightSampler.createStructureWeightSampler` 收集。若本 patch 实测 `[BEARD]` 全 0，应终止 Beardifier 方向，转向 aquifer 内部（`AquiferSampler.Impl.apply` 的 random/fluidLevel 逻辑）排查。

---

## 1. 源码确认：ChunkNoiseSampler 中 StructureWeightSampler 的字段

**字段名**：`beardifying`

**声明**（`data/mc_src_extract/net/minecraft/world/gen/chunk/ChunkNoiseSampler.java` L54）：
```java
private final DensityFunctionTypes.Beardifying beardifying;
```

**类型**：`DensityFunctionTypes.Beardifying`（`DensityFunctionTypes.java` L314：`public interface Beardifying extends DensityFunction.Base`）→ 是 `DensityFunction` 子类型，可调用 `sample(NoisePos)`。

**反射获取**（沿用 BlockProbe 现有 L178-180 已验证写法）：
```java
java.lang.reflect.Field fBeard = net.minecraft.world.gen.chunk.ChunkNoiseSampler.class.getDeclaredField("beardifying");
fBeard.setAccessible(true);
Object beard = fBeard.get(cns);   // 实际类型 StructureWeightSampler
```

**如何注入（phase6 已证 + 本次复核）**：
- `NoiseChunkGenerator.java` L102-106：`createChunkNoiseSampler` 内
  `StructureWeightSampler.createStructureWeightSampler(world, chunk.getPos())` 作为 `beardifying` 实参传入 `ChunkNoiseSampler.create(...)`。
- `ChunkNoiseSampler.java` L177-180（构造器）：
  ```java
  DensityFunction densityFunction = DensityFunctionTypes.cacheAllInCell(
          DensityFunctionTypes.add(noiseRouter2.finalDensity(), DensityFunctionTypes.Beardifier.INSTANCE))
      .apply(this::getActualDensityFunction);
  ```
- `ChunkNoiseSampler.java` L469-470（`getActualDensityFunction`）：`Beardifier.INSTANCE` 占位被替换为 `this.beardifying`（= StructureWeightSampler 实例）。
- **因此 aquifer 输入 = `cacheAllInCell(add(finalDensity, StructureWeightSampler)).sample` 确认无误**（BlockProbe L554-575 `[AQF-J]` 已反射到该 densityFunction 采样）。

**字段可见性**：`private final` → 反射 `getDeclaredField` + `setAccessible(true)`（BlockProbe 全文件同款，无 SecurityManager 障碍）。

---

## 2. 采样方法确认

**StructureWeightSampler 实现 `DensityFunctionTypes.Beardifying`**（`data/mc_src_extract/net/minecraft/world/gen/StructureWeightSampler.java` L21）：
```java
public class StructureWeightSampler implements DensityFunctionTypes.Beardifying
```

**`sample(DensityFunction.NoisePos pos)`**（L79-120）只使用：
```java
int i = pos.blockX();
int j = pos.blockY();
int k = pos.blockZ();
```
→ **`new DensityFunction.UnblendedNoisePos(blockX, blockY, blockZ)` 完全够用**，不需要 cns 状态字段，也**不进入 interpolation loop**（无 `isInInterpolationLoop` 检查），采样安全、可重复（每次 sample 结束 `pieceIterator.back(Integer.MAX_VALUE)`/`junctionIterator.back(...)` 重置迭代器）。

**语义**：
- `sample` 遍历当前 chunk（含 12 格扩展）内 **terrain_adaptation != NONE** 的 structure pieces + jigsaw junctions，累加权重；
- 无任何结构时返回 `0.0`；
- 权重可为正/负（注释：正 y 使权值为负，负 y 使权值为正——即结构上方「掏空」为负修正）。

**在 (-278,15..19,-240) 的调用方式**（与 BlockProbe L181-182 已运行的 `[BeardDiag]` 写法一致）：
```java
var npB = new net.minecraft.world.gen.densityfunction.DensityFunction.UnblendedNoisePos(-278, yB, -240);
double valB = (double) net.minecraft.world.gen.densityfunction.DensityFunction.class
        .getMethod("sample", net.minecraft.world.gen.densityfunction.DensityFunction.NoisePos.class)
        .invoke(beardB, npB);
```

**坐标说明**：世界 (-278,-240) 位于 chunk(-18,-15)（-18*16=-288..-272，-15*16=-240..-224），局部 `(x=10, z=0)`；y=15..19 为 FULL 参照列 `ref_col_-278_-240.txt` 的 water 段（y=15..19 raw=32 water），y=0..14/20..22 stone、y=23 air、y=24..30 stone。

---

## 3. 完整 patch（最小化，零新增 import）

**文件**：`E:\PYTHON\MC\versions\1.20.1\java\src\main\java\wg\bench\BlockProbe.java`

**插入位置**：`run()` 主导出循环内，NOISE-BLK v2 块（L481-503，`int[][] nbCols` + `for (int[] c : nbCols)` 循环）**之后**、`// EstDiag-288`（L504）注释**之前**。

锚点（行号为当前文件行号）：
```java
499:                                } catch (Throwable exB) {
500:                                    System.out.println("[NOISE-BLK] ERR " + exB);
501:                                }
502:                            }
503:                        }
   :                        <=== 在此插入 BEARD 块
504:                        // EstDiag-288：chunk(-16,-16) 4 角 est + 单列 est（对比 C++ 32）
```

**为什么选这里**：
- L477 `Chunk chunk = world.getChunk(wx, wz, ChunkStatus.NOISE, true);` 后 chunk 已到 NOISE 阶段，`Chunk.chunkNoiseSampler` 字段存活（L506-508 现有 EstDiagN 成功获取的先例）。
- 复用 `chunk`、`wx`/`wz` 局部变量；`if (wx == -18 && wz == -15)` 触发，只在目标 chunk 执行一次。
- 独立 try-catch，不触碰现有 nbCols / EstDiagN / AQF-J 逻辑（最小化铁律）。
- 全部用全限定名 → **零新增 import**。

**完整 Java 代码**：
```java
                        // BEARD-288：chunk(-18,-15) 反射 cns.beardifying（StructureWeightSampler）采样 (-278,-240) 列
                        // 验证「含水层判水 = ocean_ruin Beardifier 负修正」假设；参考 ref_col_-278_-240.txt（y=15..19 water）
                        if (wx == -18 && wz == -15) {
                            try {
                                java.lang.reflect.Field fCnsB = Chunk.class.getDeclaredField("chunkNoiseSampler");
                                fCnsB.setAccessible(true);
                                Object cnsB = fCnsB.get(chunk);
                                if (cnsB != null) {
                                    java.lang.reflect.Field fBeardB = net.minecraft.world.gen.chunk.ChunkNoiseSampler.class.getDeclaredField("beardifying");
                                    fBeardB.setAccessible(true);
                                    Object beardB = fBeardB.get(cnsB);
                                    if (beardB != null) {
                                        java.lang.reflect.Method mSampB = net.minecraft.world.gen.densityfunction.DensityFunction.class
                                                .getMethod("sample", net.minecraft.world.gen.densityfunction.DensityFunction.NoisePos.class);
                                        for (int yB = 0; yB <= 30; yB++) {
                                            var npB = new net.minecraft.world.gen.densityfunction.DensityFunction.UnblendedNoisePos(-278, yB, -240);
                                            double valB = (double) mSampB.invoke(beardB, npB);
                                            System.out.println(String.format(java.util.Locale.ROOT, "[BEARD] y=%d val=%.6f", yB, valB));
                                        }
                                        System.out.println("[BEARD] beardClass=" + beardB.getClass().getName());
                                    } else {
                                        System.out.println("[BEARD] beardifying null");
                                    }
                                } else {
                                    System.out.println("[BEARD] cns null");
                                }
                            } catch (Throwable exB2) {
                                System.out.println("[BEARD] ERR " + exB2);
                            }
                        }
```

**说明**：
- 循环 `y=0..30` 全列（覆盖任务要求的 15..19 + 上下界参照），输出 `[BEARD] y=<y> val=<double>` 格式与任务约定一致。
- `yB` 为局部变量名，`npB`/`cnsB`/`beardB`/`mSampB`/`fCnsB`/`fBeardB` 均不与现有作用域冲突（现有诊断用 `cns`/`np`/`npb`/`fBeard`/`mSampB` 等在各自独立 try 块内，此处为块内新名，安全）。
- 附带 `beardClass` 行：确认反射拿到的确实是 `StructureWeightSampler`（排除拿到 `Beardifier.INSTANCE` 或其他实现）。
- **零新增 import**：`Chunk`、`String.format`/`java.util.Locale`、全限定名 `net.minecraft.world.gen.chunk.ChunkNoiseSampler` / `net.minecraft.world.gen.densityfunction.DensityFunction` 全部可全限定引用；`Throwable`、`var` 无需 import。

---

## 4. 验证方式

```bash
# 在 E:\PYTHON\MC\versions\1.20.1\java 下执行（build.gradle 已定义 -Pbench* 与 -PblockProbe）
gradle runServer -PblockProbe -PbenchSeed=-8248318472910187742 -PbenchSize=4 -PbenchOriginX=-288 -PbenchOriginZ=-256
```

**必须删 `run\world`** 强制 NOISE 现场生成（否则 chunk 已 FULL，`chunkNoiseSampler` 可能已释放/状态不符）。

**触发条件核对**（与 noise_blk_patch.md L131 同法）：
- `wx = originX/16 + cx = -288/16 + cx = -18 + cx`，cx=0..3 → -18..-15，含 **-18**（cx=0）；
- `wz = -256/16 + cz = -16 + cz`，cz=0..3 → -16..-13，含 **-15**（cz=1）；
- → chunk(-18,-15) 必触发，且 `nbCols` 中已有 `{-18, -15, 10, 0, 0, 30}`（同列直读，可对照）。

**预期输出**（stdout，混在现有诊断中）：
```
[BEARD] y=0 val=0.000000
[BEARD] y=1 val=0.000000
...
[BEARD] y=15 val=0.000000
...
[BEARD] y=30 val=0.000000
[BEARD] beardClass=StructureWeightSampler
```

---

## 5. 判别规则

| `[BEARD]` 实测 | 结论 |
|----------------|------|
| **val 全 0**（`|val|<=1e-9`） | **假设推翻**：(-278,-240) 附近无任何 terrain_adaptation 结构，Beardifier 不参与判水 → 判水根因在 aquifer 内部（fluid level 随机 / blend / apply 分支），转主会话决策是否开 aquifer 内部 patch |
| **val 负且 \|val\|>0.074**（任务给定阈值） | 假设闭合：density 正 0.055-0.077 + Beardifier 负修正 ≤0 → Java 判水。但注意——ocean_ruin 自身不可能产生该值；若出现，说明 24 格内另有 adaptation 结构（village/stronghold 等），需打印 `pieceIterator` 内容定位（本次 patch 未含，可在确认非 0 后追加） |
| **val 显著非零但达不到 0.074** | 中间态：需结合 `[AQF-J] densFn`（BlockProbe L569-573 已有，含 Beardifier）与 raw `finalDensity` 差分定位 |
| **beardClass 非 StructureWeightSampler** | 环境异常（mod 替换），记录并交主会话 |

---

## 6. 环境与依赖确认

- **Java 版本**：1.20.1，Yarn 映射（`net.minecraft.world.gen.chunk.ChunkNoiseSampler` 等为 yarn 名，与 BlockProbe 现有反射一致）。
- **mod 依赖**：`build.gradle` L19-20 仅 `fabric-loader:0.15.11` + `fabric-api:0.92.0+1.20.1`，**无 C2ME**；DensityProbe 注释中 c2me 仅为参照说明，实际运行 vanilla 路径（NoiseChunkGenerator 原生 populateNoise）。
- **依赖清单（本次 patch 涉及）**：
  - 内部：`Chunk.chunkNoiseSampler` 字段（BlockProbe L506 已验证）、`ChunkNoiseSampler.beardifying`、`StructureWeightSampler.sample`、`DensityFunction.NoisePos`/`UnblendedNoisePos`。
  - 外部：无新增第三方库。
- **BlockProbe 已有先例**（L176-193 `[BeardDiag]`）：对 `beardifying` 字段用 `UnblendedNoisePos(bx,by,bz)` 采样 `sample` 已在该文件内运行过——本 patch 完全复用其成功路径，仅改为目标列 + 全列循环 + 新输出前缀，风险极低。

## 7. 待深入点清单（若假设推翻）

1. `AquiferSampler.Impl.apply`（`mc_src_extract/.../world/gen/chunk/AquiferSampler.java`）在 (-278,15..19,-240) 的 random 分支：fluid level 采样、`sampleDensity` 的 == 0 / < 0 判定（判水阈值可能不是 density ≤ 0，而是 aquifer 内部 fluid level 判定）。
2. Blender 路径：`ChunkNoiseSampler` L142-143 cachedBlendAlpha/Offset——(-288,-256) 区域是否在旧世界 blend 区（C++ 若未实现 blend 会系统性错判水/陆）。
3. `ore_vein` 之外的 apply 顺序：`ChainedBlockSource` L181 aquifer 先 apply、L183 oreVein 后 apply——确认 (-278,15..19,-240) 不是 vein 覆盖的 stone 吞水。

## 8. 混淆评估

- 目标为开发期 mod 工程源码（非发布 jar），无 ProGuard/R8 混淆；反射字段名 `beardifying`/`chunkNoiseSampler` 为 yarn 映射名，与当前运行环境一致（`[BeardDiag]`/`[AQF-J]` 已在同环境跑通）。**无需 recode.deobfuscate**。
