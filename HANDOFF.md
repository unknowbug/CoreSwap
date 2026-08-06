# CoreSwap 会话交接（2026-08-06 晚）

> 本文件是给下一个会话的现场快照。接班 AI 先读本文件 + `git status`，再开始干活。

## 〇、最新里程碑（2026-08-06）

- **客户端实机可玩**：修复了「地形全空气」连环 bug（stateById 预填 AIR + 直写 container.set 不更新 nonEmptyBlockCount → isEmpty 误判全空气；commit dbbfd47/28aa90c），玩家走出几千格都有地形，结构不再悬浮。根因与验证方法详见 `docs/07`「Java 侧写入路径的坑」。
- **批量 fillBlocks**（919b559）：fillChunk 攒批（BATCH=16/2ms 超时），一次 JNI 并行生成 16 chunk，探索顺滑。
- **推荐组合实测（2026-08-06 用户确认）**：CoreSwap + Sodium 0.5.13 + Iris 1.7.6 + BSL 10.1.3，**笔记本 RTX 4060 + 最大渲染距离（32 视距）全程不卡**（探索加载由 CoreSwap 批量并行扛住、帧率由 Sodium/显卡扛住）。组合方案：Sodium 管渲染、CoreSwap 管生成，互补不冲突（见记忆 core-swap-sodium-combo）。光影包在 `run/shaderpacks/`，Sodium/Iris 在 `run/mods/`。
- **C++ 一律 MSVC**（用户严禁 MinGW/gcc）：`build-msvc/bin/worldgen.dll`，配置见 `configure_msvc.bat`。
- **JDK17 固定**（`gradle.properties` org.gradle.java.home），runClient/runServer 均用 JDK17（JDK24 会崩）。
- **大项目目标（务必记住，勿漏）**：CoreSwap = 「把 Java 版性能核心 C++ 化、保留 MOD 生态」。区块生成是第一刀（已可玩）；**AI 层（实体 Brain / GoalSelector / Pathfinding 寻路的 C++ 化）是规划中的后续刀**（`versions/1.20.1/cpp/ai` 空占位目录）。AI 层一致性标准 = 行为级一致（非逐位，AI 是随机决策）。
- **待排期**：LIGHT 光照 C++ 化（用户定为最后，价值最低）、打包发布、**全版本适配（用户真实目标：含 1.17 及更早，1.20.x 优先，docs/08 流程）**。**FEATURES（矿物/装饰）绝不做 C++ 化**（版本更新必动的大头，全版本适配忙不过来——见记忆 core-swap-full-version-goal）。

## 一、项目与环境

- **项目**：CoreSwap —— Minecraft 1.20.1 worldgen C++ 复刻（JNI 桥 + 方块层），目标与 Java 逐位一致。
- **仓库**：https://github.com/unknowbug/CoreSwap（PUBLIC，master，gh 账号 unknowbug，git author 严禁改）。
- **代码**：`versions/1.20.1/cpp/`（worldgen 模块 + ai 占位）；Java 侧 `versions/1.20.1/java/`。
- **环境**：Windows；**C++ 必须 MSVC**（VS 2026 Community `D:\Program Files\Microsoft Visual Studio\18\Community`，vcvars64 + Ninja，禁用 MinGW/gcc——用户铁律）；JDK 17（`E:\python\MC\tools\jdk17\jdk-17.0.20+8`）；CMake 4.4.1。
- **数据根**：`E:\python\MC\data\`（blocks.json、biome_params.json、vanilla_*.blocks、got.bin、diag_*.py）。
- **测试常量**：seed `-8248318472910187742`；区块 x=3200–3203, z=3208–3211（4×4=16 chunks）；`minY=-64, height=384, seaLevel=63`。
- **对比基准**：`ChunkStatus.SURFACE`（= NOISE+SURFACE，含矿脉 OreVein，不含 features）。

## 二、常用命令

```powershell
# C++ 编译（Release，产物在 cpp\build\bin\）
$env:Path = 'E:\python\MC\tools\mingw\mingw64\bin;' + $env:Path
cmake --build 'versions\1.20.1\cpp\build' --config Release

# 对比（读 vanilla 参照）
& 'versions\1.20.1\cpp\build\bin\block_probe.exe' '-8248318472910187742' 'E:\python\MC\data\worldgen' 'E:\python\MC\data\vanilla_-8248318472910187742_4_3200_3208.blocks'

# 导出 C++ 结果 got.bin（慢，~3-5 分钟，后台跑）
& 'versions\1.20.1\cpp\build\bin\got_export.exe' '-8248318472910187742' 'E:\python\MC\data\worldgen' 'E:\python\MC\data\got.bin'

# Java 导出 vanilla 参照（会重新导出 data/vanilla_*.blocks，覆盖！）
cd versions\1.20.1\java; $env:JAVA_HOME='E:\python\MC\tools\jdk17\jdk-17.0.20+8'; $env:Path="$env:JAVA_HOME\bin;"+$env:Path
gradle runServer --no-daemon -PblockProbe=true -PbenchSeed=-8248318472910187742 -PbenchSize=4 -PbenchOriginX=3200 -PbenchOriginZ=3208

# 差异分析
python data\diag_full.py   # 全方块互换 top + y 分布
```

**注意**：
- vanilla 文件 seed 是 **有符号** `-8248318472910187742`；python 用 `>Q` 读会得到 10198425600799363874 —— 两者位模式相同，**不是 seed 变了**！别被坑（本次会话为此白删过 world）。
- `run/server.properties` 的 `level-seed=-8248318472910187742` 已正确；`run/world` 已按此重建（worldSeed 已验证 = -8248318472910187742）。删 world 会强制重建，很慢，非必要别删。

## 三、本次会话已完成的修复（未提交！git status 可查）

1. **hashXYZ（xoroshiro.h）**：1.20.1 的 `MathHelper.hashCode(int x,int y,int z)` 是 **long 版本**：
   `long l = x*3129871 ^ z*116129781L ^ y; l = l*l*42317861L + l*11L; return l>>16;`
   - `x*3129871` 是 **int32 溢出**（补码）；`>>16` 是**算术右移**（符号扩展，之前漏了这个导致 y=3/4 的 rnd 对不上）。
   - 已用 Java 探针逐位验证：VgDiag 输出 y=2..5 的 hash/rnd 与 Java 完全一致。
   - 影响：aquifer blob 位置、verticalGradient（deepslate/bedrock 渐变）、oreVein split 全部依赖它。
2. **surface.h**：mr2（sandstone/sand）、mr3（stone/gravel）、red_sandstone 三处 `stoneDepth(0,false,0,X)` 的 ceiling 参数 `false→true`（Java 是 STONE_DEPTH_CEILING）。
3. **aquifer.h estimateSurfaceHeight**：加 BiomeCoords 对齐 `(blockX>>2)<<2`（Java: BiomeCoords.toBlock(fromBlock(x))）。
4. **aquifer.h getFluidBlockState**：`state = defaultFL.block`（Java 是 `defaultFluidLevel.state`，**不经 getBlockState**；旧代码在液面=63 时返回 air 导致水袋缺失）。

**对齐数据（4 项修复后）**：
- 99.72%（hashXYZ 后）→ 99.93%（ceiling 修复后）→ 99.78%（getFluidBlockState 后，bash-46）
- **但后续 block_probe 复跑出 97.73% —— 见"未解之谜"**

### 里程碑：JNI 桥 + mod 替换接入（2026-08-06，提交 03732db）

**CoreSwap 替换 mod 跑通**：`gradle runServer -PcppReplace=true` → 世界 NOISE+SURFACE 阶段由 C++ 生成。

- **JNI 桥**：`Java_wg_CppWorldgen_fillBlocks`（wg_fill_blocks_multi 包装）；Java 侧 `wg/CppWorldgen.java`；JniProbe 验证 Java→JNI→C++ 往返 100.0000%（对比 vanilla 参照 + got.bin 交叉验证）
- **mixin**：NoiseChunkGeneratorMixin 拦截 `populateNoise`（HEAD，C++ 整块填充 + completedFuture）与 `buildSurface`（跳过）；`-Dcpp.replace=1` 启用
- **验证闭环**：服务器启动 Done 无崩溃，spawn 区 region 落盘；ReadWorldProbe 读回对比——**NOISE+SURFACE 层与 vanilla 逐位一致**（FULL 差异仅 FEATURES 阶段矿物/熔岩湖，由原版 Java 继续处理，预期行为）
- **坑**：mixin 注入目标是公共 `populateNoise(Executor, Blender, NoiseConfig, StructureAccessor, Chunk)`（返回 CompletableFuture）不是私有版；参照文件每 chunk 末尾 256 个 biome writeUTF 需跳过；DataOutputStream 是 big-endian
- **下一步候选**：FEATURES 阶段替换（矿物也 C++ 化）、攒批并行（C++ 16 线程用于批量 chunk）、玩家实跑测试（runClient 已确认能启动到主菜单，进世界需手动）


### 技术知识库（docs/，2026-08-06 建立）

**面向版本迭代（1.17.x 等）**：`docs/` 下 8 篇——架构映射 / 随机派生 / 密度函数 / 含水层 / 矿脉 /
表面规则 / 块级流水线 / **版本迁移方法论**。每篇固定结构：功能目的 → 1.20.1 工作机制（含代码位置）
→ **版本敏感点（`[ ]` 检查清单）** → 已验证的坑。迁移时先读 `docs/08-version-migration.md`。

### 谜 A+B 真相：16:09 的 vanilla 文件被旧 world 缓存污染（假矿脉差异）

- **根因**：BlockProbe 导出用 `world.getChunk(wx, wz, ChunkStatus.SURFACE, true)`。若 run/world 的 region 文件里有旧 chunk（14:58 前或早期 world 生成的），**直接复用缓存，不重新生成**。
- 时间线还原：14:58 导出（99.78%）时 world 是干净重建的（无矿脉）；16:09 导出时 region 里有旧 chunk（含矿脉，可能是旧 world 产物）→ 复用 → vanilla 文件含假矿脉 → block_probe 对比出 97.73% 假差异。
- **验证**：删掉 `run/world/region/r.5.5/5.6/6.5/6.6.mca` 后重新导出 vanilla，(3211,4,3204) 从 andesite 变 deepslate，矿脉消失；block_probe 回到 **99.7782%**。
- **C++ 的 OreVein 一直正确**：VeinDiag 实测 Java 真实 veinToggle 插值 (3211,4,3204)=0.162342，与 C++ 完全一致；Java 在该点也 block=null（不生成矿脉）。
- **教训**：重新导出 vanilla 参照前必须删 region 文件（或删 world），否则 getChunk 复用旧缓存。
- **修复了假参照后无需修 OreVein**（矿脉差异是假象）。

### 谜 C 状态：待重验

- 谜 A/B 解开后剩余差异 ~0.22%（见第五节 diag 结果），含水层残余需基于干净参照重验。

## 五、本次会话新增工具与待清理

### 里程碑：方块层 100% 对齐（2026-08-06）**block_probe：TOTAL 100.0000%，非空气 100.0000%（16/16 chunks 全 100%），75-190ms/chunk。**

### 性能优化：多线程并行 + aquifer 列缓存（2026-08-06，提交 66e05f5）

**16 chunks：串行 1056ms → 并行 109.8ms（24 线程，~9.6× 加速），100% 保持。**

- 热点量化：density 采样 ~12ms/chunk（12%），**aquifer+oreVein 59-562ms/chunk（88%）**。
- **aquifer 根因**：`estimateSurfaceHeight` 无缓存——每块 13 个邻居列 × 最多 49 次 initialDensity 采样 ≈ 3200 万次/chunk。加 **per-chunk 列缓存**（Java `surfaceHeightEstimateCache` 同款，key=(x>>2,z>>2) 对齐列）→ 每 chunk ~240 列各 1 次（~2700 倍降幅）。
- **多线程**：`wg_fill_blocks_multi`（chunk 级 std::thread 池，确定性结果与串行逐位一致）。线程安全前提：
  - `InterpolatedDF` 缓存 → per-instance `thread_local`（O(1) ID 索引 vector，非 std::map）
  - `overworldRule` 预构建到 `wg_create`（消除懒构建竞态）
  - aquifer/SurfaceContext/oreVein 均 per-chunk 局部对象；`split()`/`split(name)` 是 const 纯函数 ✓
- 串行 API `wg_fill_blocks` 保留（JNI 用）；block_probe 改用并行入口。
- **注意**：单线程内「base_3d_noise 网格插值」类优化会引入浮点误差破坏 100%——多线程是唯一无损的大优化。

四项修复（提交 d445ae5）：
1. **aquifer 无效液面**：`INT32_MAX → -32512`（Java `DimensionType.field_35479`）。`fl2.getBlockState(blockY)` 用 `blockY >= y ? air : block`，INT32_MAX 导致深地永远返回 water（air→water 2691 块归零）。
2. **finalDensity 块级插值顺序**：C++ 原为「整树在 cell 角点采样 + 手动三线性插值」；Java 是「只对 interpolated 节点插值，min/squeeze/mul 等非线性在插值后应用」。改为块级直接 `finalDensity->sample(pos)`（InterpolatedDF 内部按 cell 网格插值）——数学语义对齐，water→stone 173 块归零。**附带性能提升 40 倍**（4000ms→90ms/chunk）。
3. **surface materialRule7 结尾**：误放 materialRule8 的 taiga/ice_spikes/mushroom/mr 分支，且 fallback 应为 `MANGROVE→MUD + DIRT`。修复地表草皮误生成（dirt→grass_block 200 块归零）。
4. **estimateSurfaceHeight**：去掉 `+ runDepth - 8`（本 seed 下与 Java 实际行为一致）。

### 探针工具（保留，验证用）

- `cpp/worldgen/src/ore_probe.cpp`（C++ 矿脉探针）
- `java/src/main/java/wg/bench/OreProbe.java`（Java 插值复刻，`-PoreProbe=true`）
- `data/locate_ore.py`、`data/locate_diff.py`、`data/locate_diff2.py`
- **BlockProbe.java 的 VeinDiag/driveCnsTo**（驱动真实 ChunkNoiseSampler 插值循环取真实方块/veinToggle/角点——验证阶段极其有用，保留）

### 待清理

- BlockProbe.java 的 `[VgDiag]`/`[CppCmp]`/`[CppCmpS]`/`[DimDiag]`/`[EstDiag]`/`[BioDiag]`/`[BeardDiag]`/`[InterpList]` 诊断打印（不影响逻辑，可删或保留）
- `data/vanilla_1609_backup.blocks`（污染版参照，可删）
- `versions/1.20.1/cpp/build` 里的 build-dbg/build-asan 产物（不提交）

### vanilla 参照

`data/vanilla_*-8248318472910187742_4_3200_3208.blocks` = 干净导出（删 region 后重新生成）。**教训：重新导出 vanilla 前必须删 run/world/region/r.5.5/5.6/6.5/6.6.mca，否则 getChunk 复用旧 chunk 缓存。**

## 六、纪律（上一个会话的教训，务必遵守）

- **思维链禁噪声**：推理用编号短句/自然语言；禁止 `！！！`、`！！`、`——` 等连续重复符号（上一个会话因卡壳反复输出这种符号，被用户手动停止）。
- **卡壳熔断**：同一问题推理 ≤2 轮无进展 → 立即改用工具验证 / 向用户明确汇报卡点，禁止原地绕第三圈。
- **数据说话**：拿不准就先跑工具（diag、探针、git diff），别纯推理。

## 七、下一个会话的第一个动作

1. `git status` + `git diff --stat` 确认工作区（4 项修复在、ore_vein.h 调试打印在）。
2. 读 ore_vein.h 当前 `[ov]` 打印需要重跑 got_export 才能看到 → 先直接加个更全的调试（或直接跑一次 got_export 看 toggle 中部值）。
3. 用 `diag_full.py` 确认当前基线（应为 ~97.7%，含矿脉差异）。
4. 修 OreVein → 目标先把矿脉差异清零 → 再回头含水层 → 100% 后性能优化（紧凑数组+索引）。
