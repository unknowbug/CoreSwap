# scout-c：JNI 通路 / features 接管切换点语义图（1.21.6 侧）

status: draft（静态代码事实，Degraded=静态审查；无运行时验证）
scope: 只读勘探，机制图，不做方案定论。不确定处标 @anchor.idk。

## 1. Java 侧（runtime/1.21.6/java/src/main/java/wg/）

### stageMask 解析（bench/CppBridge.java L66-74）
- `resolveStageMask()`：`-Dcoreswap.rust.stages` — null/空 → 0b011；`"all"` → 0；其他按 int 解析；非法 → 0b011。
- `setFlags` 调用点：`init()` L82 / `initNether()` L128 / `initEnd()` L148 —— 每个句柄创建成功后立即 `CppWorldgen.setFlags(h, mask)`。
- 注释 L66-67：bit0=SKIP_CARVER, bit1=SKIP_FEATURES；默认 0b011 = 存档链路关 Rust carver/features（Java vanilla 执行，Rust 双跑 = confirmed 缺陷）。
- 1.20.1 侧（runtime/1.20.1/java/.../CppBridge.java）：resolveStageMask/setFlags/日志行完全同构（L69/82/128/148 同位置）——mask 机制两侧无差异。

### mod 入口时序（bench/BenchMod.java，Fabric ModInitializer）
- `SERVER_STARTED` 事件内：`active = (cpp.replace 存在 || 无任何探针参数) && !cpp.vanilla` → `CppBridge.init(seed)` → `initNether` → `initEnd` → `registerModBlocks`（-Dcpp.blockRegister 门控，默认关）。
- `SERVER_STOPPING` → `CppBridge.destroy()`（句柄置 0 防使用后释放）。
- 门控 flag：`-Dcpp.replace`（显式替换）、`-Dcpp.vanilla`（纯 vanilla A/B）、十余个 `*-probe` 属性（任一存在 = 探针模式、非 active）。

### mixin 拦截点完整清单（bench/mixin/，22 个类）
**接管类（取消/短路 vanilla）：**
| 类:方法:行为 |
|---|
| NoiseChunkGeneratorMixin : populateNoise(Blender,NoiseConfig,StructureAccessor,Chunk) : @At(HEAD) cancellable；命中接管集（settings id ∈ {minecraft:overworld,nether,end} + 形状匹配 + 对应句柄 active）→ feedBeardifier + fillChunk + `cir.setReturnValue(completedFuture(chunk))` —— **取消 Java NOISE 阶段** |
| NoiseChunkGeneratorMixin : buildSurface(ChunkRegion,...) : HEAD cancellable；同接管集 → `ci.cancel()` —— **取消 Java SURFACE 阶段** |
| ServerLightingProviderMixin : light(Chunk,Z) : HEAD；光照接管路径 cir.setReturnValue（D4 协议，与 features 无关） |

**其余 19 个 mixin 全部是观测/探针（HEAD 观测不取消、RETURN 打日志），不取消任何阶段**：AquiferDumpProbeMixin、BeehiveDecoratorMixin、BiomeSourceLogMixin、BlobFoliagePlacerMixin、BlobProbeMixin、BushFoliagePlacerMixin、ChunkLightProviderAccessor、ChunkRandomSeedLogMixin、ColProfProbeMixin、ConfiguredFeatureProbeMixin、CountPlacementModifierMixin、DiagFeatureBiomeMixin（generateFeatures 只观测）、EstDumpProbeMixin、LightingProviderAccessor、MegaJungleTrunkPlacerMixin、NoiseDumpProbeMixin、SquarePlacementModifierMixin、SurfaceDumpProbeMixin、TreeDecoratorGeneratorMixin、TrunkPlacerMixin。
→ **结论：CARVERS 与 FEATURES 两个 chunk 步骤 Java 侧当前零拦截**（mixin 对 carve/generateFeatures 无任何 cancel），与 mask 无关恒跑 vanilla。
- 1.20.1 差异：仅 populateNoise 首参 Executor（1.21.6 移除，签名迁移 260908-10，mixin 内注释引 NoiseChunkGenerator.java:326）；buildSurface 签名相同；features/carvers 拦截缺失两侧一致。

## 2. Java features 阶段执行点（1.21.6，mc_src_extract）

调用链（代码事实）：
```
ChunkStatus.FEATURES 步骤
  → ChunkGenerating.generateFeatures (ChunkGenerating.java L131-142)
      Heightmap.populateHeightmaps(MOTION_BLOCKING,...) L135-137
      → ChunkGenerator.generateFeatures(chunkRegion, chunk, structureAccessor) L139   [ChunkGenerator.java L333]
      → Blender.tickLeavensAndFluids L140
```
- 「Java 跳过自己的 features 但保留其他阶段」的机制候选点（只画图）：(a) @Mixin(NoiseChunkGenerator).generateFeatures(StructureWorldAccess,Chunk,StructureAccessor)V HEAD cancellable —— 与现有 buildSurface 拦截同型、同接管集判定；(b) @Mixin(ChunkGenerating).generateFeatures 静态步骤级拦截（更靠上，会影响 FEATURES 步骤的 heightmap 预填充 L135-137 与 tickLeavesAndFluids L140 —— 依赖这些副作用的语义差异需后续验证）。@anchor.idk("generateFeatures 的 3×3 邻 chunk 读取(ChunkPos.stream L345-351)在 Rust 接管下需要什么邻域状态，未验证")
- CARVERS 链同理：ChunkGenerating.carve L112-129 → ChunkGenerator.carve（当前两侧都不拦）。

## 3. Rust 侧（versions/1.21.6/rust/src/jni_bridge.rs + worldgen-core/src/worldgen_handle.rs）

- 1.21.6 薄壳只有 jni_bridge.rs + lib.rs；**worldgen_handle.rs 在共享引擎 worldgen-core**（薄壳无自己的 flags 逻辑）。
- JNI 接线：`Java_wg_CppWorldgen_setFlags`（jni_bridge.rs L109-120）→ `wg_set_flags(handle, mask)` → `WorldgenHandle.flags: AtomicU32`（api.rs L113）；`getFlags` L122-132 读回。
- 位定义（worldgen_handle.rs L145-148）：`FLAG_SKIP_CARVER=1<<0`、`FLAG_SKIP_FEATURES=1<<1`、`FLAG_SKIP_SURFACE=1<<2`（**Java resolveStageMask 只会产生 bit0/bit1；bit2 只能由原始 int prop（如 =4）或 env 触达**）。
- 语义 = **OR 组合，非 mask 独占**（skip 判定三处）：
  - features（L609）：`flags & FLAG_SKIP_FEATURES != 0 || env WG_SKIP_FEATURES 存在` → skip
  - surface（L760）：`flags & FLAG_SKIP_SURFACE == 0 && env WG_SKIP_SURFACE 不存在` → 跑
  - carver（L772）：`flags & FLAG_SKIP_CARVER == 0 && env WG_SKIP_CARVER 不存在` → 跑
  → **mask=0 不是「强制跑」**：env WG_SKIP_* 仍可独立关掉该阶段（兼容旧行为）；反向 mask=0b111 也不能覆盖 env（env 只能额外 skip，不能额外跑）。
- features 执行体：fill_chunk_blocks L596-618 步骤5 → apply_features（矿石/disk/spring/freeze_top/underwater_magma）；WG_FEATURELOG env 开 `[FEATURE] chunk(x,z) placed N blocks` 日志（stderr）。

## 4. mask 真值表（接管维度 overworld/nether/end；Java 侧 mixin 对 CARVERS/FEATURES 恒不拦，与 mask 无关）

| mask | Rust carver | Rust features | Java carver | Java features | 结果判定 |
|---|---|---|---|---|---|
| 0b000（=all） | 跑 | 跑 | 跑 | 跑 | **carver+features 双跑**（Java 拦截缺失，非 mask 引起） |
| 0b001（SKIP_CARVER） | 跳 | **跑** | 跑 | 跑 | **features 双跑**（Rust+Java 同 chunk 各放一遍）；carver 单跑（Java） |
| 0b010（SKIP_FEATURES） | **跑** | 跳 | 跑 | 跑 | **carver 双跑**；features 单跑（Java） |
| 0b011（默认） | 跳 | 跳 | 跑 | 跑 | 各单跑（现状 = Java vanilla 装饰器在 Rust 地形上） |

- 双跑的块写入载体不同：Java features 写 ProtoChunk/ChunkRegion（可直接读写邻 chunk），Rust features 写本地 BlockColumn 再整体回填 —— 同 chunk 顺序/覆盖结果 @anchor.idk("mask=0b001 下 Rust features 输出回填与 Java features 落块的相对顺序及最终覆盖结果，未做运行时验证")。
- 若改 0b010 让 Rust 跑 carver：Rust carver 与 Java carver 双跑——结果双份洞穴 @anchor.idk（同上，未验证）。
- surface 位（bit2=4，prop 传 7 才生效）：Rust skip surface + Java buildSurface 被 mixin cancel（cancel 与 mask 无关恒取消）→ 接管维度无表面规则层。prop 传 7 = 机制上必然缺陷，非 idk。

## 5. 现有 gate / 日志哨兵（接管生效判定可用证据点）

| 哨兵 | 位置 | 含义 |
|---|---|---|
| `[CppBridge] init seed=... enabled=true stageMask=N` | CppBridge L84-86 | 接管激活 + 实际生效 mask（getFlags 读回，非解析值）+ `env.CORESWAP_DEFAULT_BLOCK`；附 dll sha256 前 16 位（L88-96，排查旧缓存） |
| `[CppBridge] initNether/initEnd ... stageMask=` | L130/150 | nether/end 句柄激活 |
| `[Mixin] populateNoise intercepted chunk(x,z)` | NoiseChunkGeneratorMixin L91/102/110 | NOISE 阶段 Rust 接管（每 chunk） |
| `[Mixin] buildSurface skipped chunk(x,z)` | L139 | SURFACE 阶段 Java 取消 |
| `[Mixin] release to vanilla: settings=... shape=...` | L57 | 放行维度（接管集外）一次性 |
| `[FEATURE] chunk(x,z) placed N blocks`（stderr，WG_FEATURELOG 门控） | worldgen_handle L612-614 | **Rust features 实跑直接证据**（N>0 即落了块）——features 接管切换后的核心哨兵 |
| `[BenchMod] CoreSwap replace mode: C++ worldgen active` | BenchMod L91 | 默认游玩模式入口 |
| env 门控：WG_SKIP_CARVER / WG_SKIP_FEATURES / WG_SKIP_SURFACE（Rust 侧独立 OR）；WG_FEATURELOG；-Dcoreswap.rust.stages（Java 侧唯一 mask 入口） | | |

## 差异注记（1.20.1 vs 1.21.6）
1. populateNoise 签名（Executor 参数移除）；2. 其余（resolveStageMask、setFlags 接线、mixin 拦截面、Rust flags 语义）两侧同构。
