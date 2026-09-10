# P4 待应用改动（nether/end 异步化 + 每 chunk 日志门控）

> 状态：**已应用**（2026-09-10 19:0x；以 `java-snapshot/post/` 为准 + `gradle compileJava` BUILD SUCCESSFUL）。改动 1/2/3 全部落地；本文件保留为「改动意图 + 边界声明」记录（judge N5）。
> 目标文件：`runtime/1.21.6/java/src/main/java/wg/bench/mixin/NoiseChunkGeneratorMixin.java`、`runtime/1.21.6/java/src/main/java/wg/bench/CppBridge.java`。
> pre 快照：`.investigations/perf-closeout-260910-05/java-snapshot/pre/`（`NoiseChunkGeneratorMixin.java` sha `5d22dc66d12ad6…`、`CppBridge.java` sha `4cda71a9437305…`）。

## 改动 1 — nether 分支（`:154-160`）→ 与 overworld R3（`:108-150`）同构的异步形态

old（现状，逐字）：
```java
        if (zeroShape && CppBridge.netherActive() && wgSettingsId().equals("minecraft:nether")) {
            if (MIXLOG) System.out.println("[Mixin] populateNoise(nether) intercepted chunk(" + chunk.getPos().x + "," + chunk.getPos().z + ")");
            CppBridge.feedBeardifierNether(chunk, structureAccessor);
            CppBridge.fillChunkNether(chunk);
            cir.setReturnValue(java.util.concurrent.CompletableFuture.completedFuture(chunk));
            return;
        }
```
new：
```java
        if (zeroShape && CppBridge.netherActive() && wgSettingsId().equals("minecraft:nether")) {
            if (MIXLOG) System.out.println("[Mixin] populateNoise(nether) intercepted chunk(" + chunk.getPos().x + "," + chunk.getPos().z + ")");
            // R3 推广（260910-05）：与 overworld 分支同构——重活移出 worldgen 单车道（下界此前同样被串行化）。
            final Chunk target = chunk;
            final StructureAccessor structures = structureAccessor;
            java.util.function.Supplier<Chunk> work = () -> {
                long tt0 = wg.bench.ChunkTiming.ON ? System.nanoTime() : 0L;
                wg.bench.ChunkTiming.enter(tt0);
                wg.bench.ChunkTiming.inflightEnter();
                try {
                    long tb0 = wg.bench.ChunkTiming.ON ? System.nanoTime() : 0L;
                    CppBridge.feedBeardifierNether(target, structures);
                    wg.bench.ChunkTiming.addBeard(System.nanoTime() - tb0);
                    CppBridge.fillChunkNether(target);
                    return target;
                } catch (Throwable t) {
                    System.out.println("[CppBridge] R3 async fill FAILED (nether) chunk("
                            + target.getPos().x + "," + target.getPos().z + "): " + t);
                    t.printStackTrace();
                    throw t;
                } finally {
                    if (wg.bench.ChunkTiming.ON) {
                        long tx = System.nanoTime();
                        wg.bench.ChunkTiming.exit(tx, tx - tt0);
                        wg.bench.ChunkTiming.inflightExit();
                    }
                }
            };
            if (SYNCFILL) {
                cir.setReturnValue(java.util.concurrent.CompletableFuture.completedFuture(work.get()));
            } else {
                cir.setReturnValue(java.util.concurrent.CompletableFuture.supplyAsync(work, WG_FILL_POOL));
            }
            return;
        }
```

## 改动 2 — end 分支（`:162-168`）→ 同上（`feedBeardifierEnd`/`fillChunkEnd`，日志标签 `(end)`）

## 改动 3 — CppBridge 每 chunk `[WG-FILL]` 读回自证日志门控（R1 同族）

现状：`CppBridge.java:446-447`（nether）与 `:490-491`（end）**无条件**每 chunk 一行 `[WG-FILL]`——4225 chunks ⇒ 4225 次 log4j 同步 appender 写入，且 23 线程并发下是串行点（与 260910-04 R1 修的是同一类病理；overworld `fillChunk` 无此行的原因即 R1 已净化）。
动作：新增 `private static final boolean MIXLOG = System.getProperty("coreswap.mixlog") != null;`（与 mixin 同名开关，统一「per-chunk 日志」语义），两处 `[WG-FILL]` 行加 `if (MIXLOG)`；`-Pmixlog=1`（→`-Dcoreswap.mixlog=1`，`build.gradle:130`）可开。
注意：**A/B 双臂对称**（都关）；P4a 载具 sanity 单独用 `-Pmixlog=1` 开这一行做「Rust fill 真跑 + 写回生效」的证据。

## 不改动的项（明确边界）
- 形状判定、`netherActive()/endActive()` 门控、`WG_FILL_POOL` 池与命名（三维修同名 `coreswap_fill_noise`，与 vanilla 跨维度同池同名形一致）。
- Rust 侧零改动（本块不动 `worldgen-core`；故无需重编 dll，执行体与 260910-04 同源）。
- `CppBridge.fillChunk/fillChunkNether/fillChunkEnd` 内 catch 后 print+return 的吞异常点（judge C4 已登记技术债，本块不扩散）。
