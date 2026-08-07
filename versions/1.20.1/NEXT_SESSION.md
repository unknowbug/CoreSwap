# CoreSwap 下一会话交接（2026-08-06 深夜版 · 完整自包含）

> 本文件是唯一权威交接。**先读全文再动手**——所有路径、命令、环境、铁律都在这里，不需要翻历史。
> 当前唯一主主线：**负坐标 bug**（见「当前任务」节）。

---

## 一、项目全貌

**仓库**：github.com/unknowbug/CoreSwap（public）。本地根目录 `E:\python\MC`（大小写不敏感，也可能显示 E:\PYTHON\MC）。git 分支 master。

**目标**：Minecraft 1.20.1 区块生成 C++ 化（性能核心替换）——Fabric mod「coreswap」。C++ 求值 density 树（JNI 调用），与 vanilla 逐位一致，生成速度碾压 Java。

**目录结构**：
```
E:\python\MC\
├─ versions\1.20.1\
│  ├─ cpp\                      C++ 世界生成引擎
│  │  ├─ worldgen\src\          源码：worldgen_api.cpp（主入口/API）
│  │  │                         density.h（InterpolatedDF/FlatCacheDF/Cache2DDF/Perlin/b3d）
│  │  │                         density_builder.h（JSON→树）、surface.h、aquifer.h、ore_vein.h
│  │  ├─ worldgen\include\worldgen_api.h（C API 声明）
│  │  ├─ build-msvc\             MSVC 构建目录（产物 bin\worldgen.dll / block_probe.exe / got_export.exe）
│  │  ├─ build\                  ⚠️ 旧 MinGW 构建目录（已废弃，勿用）
│  │  └─ CMakeLists.txt
│  ├─ java\                     Fabric mod（Java）
│  │  ├─ src\main\java\wg\
│  │  │  ├─ CppWorldgen.java    JNI 桥（dll 加载/解压/哈希校验）
│  │  │  ├─ CppBridge.java      fillChunk：JNI 批量 + PalettedContainer 直写 + heightmap 补齐
│  │  │  ├─ CoreSwapFixHelper.java（dll 缓存版本化）
│  │  │  └─ bench\              探针们（见「探针大全」）
│  │  ├─ src\main\java\wg\bench\mixin\NoiseChunkGeneratorMixin.java（拦截 populateNoise/buildSurface）
│  │  ├─ src\main\resources\worldgen-data\  运行时数据（blocks.json、biome_params*.json、noise_params.json、
│  │  │                                     data\minecraft\worldgen\density_function\{overworld,nether}\...）
│  │  ├─ src\main\resources\native\worldgen.dll（打包进 jar 的 MSVC dll——发布时必须同步最新）
│  │  ├─ build.gradle           版本号在这里改（当前 1.20.1-1.0.8）；所有 -Pxxx 探针 vmArg 开关也在这
│  │  ├─ gradle.properties      org.gradle.java.home 已固定 JDK17
│  │  └─ run\                   ⚠️ 测试端在这！world 存档/saves/logs/latest.log/hs_err
│  ├─ docs\                     知识库 01-09（追加式更新，禁止覆盖）
│  ├─ HANDOFF.md / NEXT_SESSION.md（本文件）
│  └─ README.md / promo\（promo 已 .gitignore，勿提交）
├─ data\                        数据/参照/工具
│  ├─ vanilla_*.blocks          vanilla 参照导出（BlockProbe 格式，见「数据格式」）
│  ├─ blocks.json               block id→name 表
│  ├─ worldgen\                 C++ 用的 worldgen 数据目录（含 blocks.json、density_function JSON）
│  ├─ cpp_neg8248.blocks        got_export 导出的 C++ 负坐标区域（int 格式）
│  ├─ player_pos.blocks         got_export 导出的玩家区域（int 格式）
│  ├─ c2me-fabric\              ⚠️ C2ME 开源源码（已 clone，参考用）
│  ├─ read_nbt.py / read_mca2.py / surface_compare.py / scan_diff.py（存档解析脚本）
│  └─ configure_msvc.bat / clean_*.py / fix_*.py（辅助，不提交）
└─ tools\
   ├─ jdk17\jdk-17.0.20+8\      ⚠️ 唯一可用的 JDK（JAVA_HOME，gradle.properties 已固定）
   └─ mingw\                    ⚠️ 已禁用！绝不用于 C++ 编译（thread_local 退化→假 bug）
```

---

## 二、环境（必须）

- **JDK17**：`E:\python\MC\tools\jdk17\jdk-17.0.20+8`（gradle.properties 已固定 org.gradle.java.home，一般不用手动设；手动跑 gradle 时设 `$env:JAVA_HOME='E:\python\MC\tools\jdk17\jdk-17.0.20+8'` + `$env:Path="$env:JAVA_HOME\bin;"+$env:Path`）
- **MSVC**（禁 MinGW）：VS 2026 `D:\Program Files\Microsoft Visual Studio\18\Community`；cl 在 `VC\Tools\MSVC\14.51.36231\bin\Hostx64\x64\cl.exe`；`VC\Auxiliary\Build\vcvars64.bat`；Ninja 在 `Common7\IDE\CommonExtensions\Microsoft\CMake\Ninja\ninja.exe`
- **代理**（下载/装东西用）：`http://127.0.0.1:9199`

---

## 三、命令大全（最常用的放前面）

### 1. C++ 编译（MSVC）
```powershell
cd E:\python\MC\versions\1.20.1\cpp
cmd /c "call `"D:\Program Files\Microsoft Visual Studio\18\Community\VC\Auxiliary\Build\vcvars64.bat`" >nul 2>&1 && set PATH=`"D:\Program Files\Microsoft Visual Studio\18\Community\Common7\IDE\CommonExtensions\Microsoft\CMake\Ninja`";%PATH% && cmake --build build-msvc"
```
- 产物：`build-msvc\bin\worldgen.dll`（257KB 左右）、block_probe.exe、got_export.exe
- **⚠️ 改 .h 后必须 touch worldgen_api.cpp**（或删 build-msvc 重配）——ninja 不跟踪头依赖
- 全新配置：先 `configure_msvc.bat`（根目录有，vcvars64 + cmake -G Ninja -DCMAKE_BUILD_TYPE=Release）

### 2. 主世界回归（铁律——任何改动后必须跑，必须 100%）
```powershell
& 'E:\python\MC\versions\1.20.1\cpp\build-msvc\bin\block_probe.exe' '-8248318472910187742' 'E:\python\MC\data\worldgen' 'E:\python\MC\data\vanilla_-8248318472910187742_4_3200_3208.blocks'
# 期望：TOTAL: match=1572864/1572864 (100.0000%) nonAir=501082/501082 (100.0000%)
```

### 3. Java 编译
```powershell
cd E:\python\MC\versions\1.20.1\java
gradle classes --no-daemon
```

### 4. 测试端（runServer / runClient）
```powershell
# 每次跑前清理残留：
Get-Process java | Stop-Process -Force
Remove-Item E:\python\MC\versions\1.20.1\java\run\world -Recurse -Force -ErrorAction SilentlyContinue
Remove-Item E:\python\MC\versions\1.20.1\java\run\logs\latest.log -Force -ErrorAction SilentlyContinue

# 客户端（真机测试 C++ 替换——进世界看地形）：
cd E:\python\MC\versions\1.20.1\java
gradle runClient --no-daemon -PcppReplace=true -PcppLib=E:/python/MC/versions/1.20.1/cpp/build-msvc/bin/worldgen.dll
# 首次可能要下 assets（几百 MB）；日志在 run\logs\latest.log

# 服务端探针（导出参照/诊断，见「探针大全」）：
gradle runServer --no-daemon -PblockProbe=true -PbenchSeed=-8248318472910187742 -PbenchSize=4 -PbenchOriginX=3200 -PbenchOriginZ=3208
```
- **崩溃后看**：`run\hs_err_*.log`（原生崩溃栈）+ `run\logs\latest.log`
- **残留进程**：runClient 崩后 java.exe 会残留锁文件——`Get-Process java | Stop-Process -Force`
- **存档位置**：`run\saves\新的世界\`（region\、playerdata\、level.dat）；重跑前 `Remove-Item run\saves -Recurse -Force`

### 5. got_export（C++ 生成导出）
```powershell
# 导出 C++ 生成区域（ox/oz 是方块坐标！ox/16 = chunk 坐标）：
got_export <seed> <worldgenDir> <out.blocks> <ox> <oz>
# 例：chunk(-18,-16) → ox=-288, oz=-256；chunk(1,-30) → ox=16, oz=-480
# 诊断模式（都在 got_export.cpp 里）：
got_export <seed> <wgDir> -namedDump <name> cx cz bx bz -dimension 0   # 任意注册函数 dump（主世界）
got_export <seed> <wgDir> -nbDump cx cz bx bz -dimension 0             # base_3d_noise dump
# ⚠️ -densityDump 硬编码下界（nether.json）！主世界别用
```

---

## 四、探针大全（build.gradle 的 -Pxxx → vmArg -Dxxx）

| 探针 | vmArg 开关 | 用途 |
|---|---|---|
| blockProbe | `-PblockProbe=true` + `-PbenchSeed/-PbenchSize/-PbenchOriginX/-PbenchOriginZ` | 导出 vanilla 参照 .blocks 或对比 |
| 维度参照 | 追加 `-PblockProbeDimension=nether` | 导出下界参照 |
| densityProbe | `-PdensityProbe=true` + `-PdensityProbeDimension/-PdensityProbeChunkX/-PdensityProbeChunkZ/-PdensityProbeX/-PdensityProbeZ` + `-PbenchSeed` | 密度/方块对比（**含 cns 反射链**——见「当前任务」） |
| routerProbe | `-ProuterProbe=true` + `-PbenchSeed` + `-ProuterDim/-ProuterX/-ProuterZ/-ProuterB3dDump` | NoiseRouter 采样/构造参照 |
| biomeProbe | `-PbiomeProbe=true` + `-PbiomePreset` | 导出 biome_params.json |
| noiseProbe | `-PnoiseProbe=true` | 导出 noise_params.json（38 keys） |
| heightProbe | `-PheightProbe=true` + `-PbenchSeed` | getHeightOnWorld/getColumnSample（结构高度） |
| jniProbe | `-PjniProbe=true` | JNI 桥验证 |
| readWorldProbe | `-PreadWorldProbe=true` | 读回存档对比 |
| compProbe | `-PcompProbe=true` + 坐标参数 | density 分量对比 |
| **替换模式** | `-PcppReplace=true`（= `-Dcpp.replace=1`） | runClient/runServer 用 C++ 生成 |
| dll 覆盖 | `-PcppLib=路径` | 指定 worldgen.dll（本地调试用） |
| 线程数 | `-PcoreswapThreads=N` | C++ 生成线程数（0=物理核数自适应） |
| cppDebug | `-PcppDebug=true` | C++ 调试输出 |

**探针输出目录**：`E:\python\MC\data\`（vanilla_density_*.txt 等）

---

## 五、数据格式（解析脚本/对比时）

**vanilla_*.blocks**（BlockProbe 导出，主世界 384 高）：
- 头 32B：magic(4B) + seed(8B long) + size(4B) + originX(4B) + originZ(4B) + minY(4B) + height(4B)
- 每 chunk：wx(4B) + wz(4B) + 16*16*height 个 **short（2B，大端）**
- 例：`data\vanilla_-8248318472910187742_4_-288_-256.blocks`（负坐标参照）

**got_export 的 .blocks**（C++ 导出）：头 20B（magic/seed/size/ox/oz）+ 每 chunk 8B pos + 16*16*384 个 **int（4B 小端）**
- 例：`data\cpp_neg8248.blocks`、`data\player_pos.blocks`

**blocks.json**：id→name 映射（`data\blocks.json` 或 `data\worldgen\blocks.json`——路径注意）

**存档解析**：`data\read_mca2.py`（读 region .mca 方块，含 gzip/zlib 处理）、`data\read_nbt.py`（NBT）、`data\surface_compare.py`、`data\scan_diff.py`

---

## 六、当前任务：负坐标 bug（唯一主线）

**现象**：负坐标区域 C++ 生成地形断裂/浮空。正坐标 100% 逐位一致，负坐标才触发。用户验证 seed `8576294172403134396`（玩家降落 731,82,-404）。

### 已确认事实（勿重复排查）
1. **A 方案（cns 游戏实际参照）已跑通**——DensityProbe.java 反射 cns 完整生成链：
   - `sampleStartDensity()` → 循环 `sampleEndDensity(cellX)` → `onSampledCellCorners(cellY,cellZ)` → `interpolateY/X/Z(世界坐标, progress)` → **`DensityInterpolator.sample(cns)`**（反射 `interpolators` 字段 get(0)——字段名是 `interpolators` 不是 `interps`）
   - **不能调 `sampleBlockState`**（aquifer 单 chunk 探针越界 `Index 358`——探针只加载 1 个 chunk 缺周围上下文）
   - cell 尺寸：水平 4、垂直 8；cellHeight=48；minCellY=-8；blockY=(minCellY+cellY)*8+vb（世界 y）；blockX/blockZ 必须世界坐标（chunkStartX + cellX*4 + cbx）
   - 跑法：`gradle runServer --no-daemon -PdensityProbe=true -PdensityProbeDimension=overworld -PdensityProbeChunkX=-18 -PdensityProbeChunkZ=-16 -PdensityProbeX=8 -PdensityProbeZ=8 -PbenchSeed=-8248318472910187742`
   - 输出：`data\vanilla_density_overworld_c-18_-16_b8_8_cns.txt`
2. **同 seed -8248 chunk(-18,-16) 列 (-280,-248) 对比**：
   - cns 游戏实际密度：y 48=+0.213（正）、y 52=-0.010（负）→ **过零 51-52**
   - C++ 方块（cpp_neg8248.blocks）：y 40-51 实心✅、**y 52-60 实心❌（应空气）**、y 61-64 空气✅、**y 65-99 全 stone❌（应空气）**
3. **排除项**（全验证过，别重查）：Perlin 实现（c2me 确认 vanilla 原样）、maintainPrecision（已修）、FlatCache/Cache2D key（负坐标唯一）、InterpolatedDF 插值（gx/gy/cz 非负）、取模/移位/GRADIENTS/deriver

### 下一步（按优先级）
1. **dump C++ 的 densityBuf 原始值**（fillOneChunk 内部 ~worldgen_api.cpp:534 行，`densityBuf[by*256+bz*16+bx] = h->finalDensity->sample(fpos)`）加 WG_DBDEBUG 环境变量条件打印列 (bx,bz)——对比 cns 反射值，**区分「density 错 vs aquifer/surface 错」**
2. 修 got_export `-densityDump`（硬编码下界，忽略 dimension）——主世界 dump 用 `-namedDump final_density ... -dimension 0`（但 namedDump 目前全 0——final_density 的 registry 名不对，需查 builder 注册的 key）
3. WG_SURFDUMP 诊断（worldgen_api.cpp ~547 行已有列剖面 dump）
4. 若 aquifer/surface 错：查 estimateSurfaceHeight / surface 规则遍历的负坐标路径
5. 兜底：noise-in-Java 开关（docs/09 有设计，不优先）

### 参考
- C2ME 源码：`E:\python\MC\data\c2me-fabric`（`MixinNoiseChunkGenerator.java` 有完整 populateNoise 链——A 方案照抄的；`MixinChunkNoiseSampler.java` 有 cacheAllInCell 语义）
- cns 类名：net.minecraft.world.gen.chunk.ChunkNoiseSampler；方法 yarn 名：sampleStartDensity/sampleEndDensity/onSampledCellCorners/interpolateX/Y/Z/sampleBlockState/swapBuffers

---

## 七、已发布版本与发布流程

**已发布**：1.0.4/1.0.5/1.0.6/1.0.7/1.0.8（当前 build.gradle version = '1.20.1-1.0.8'）。1.0.8 = dll 缓存版本化（哈希对比自动替换，修 XuanRikka 更新不替换问题）。

**改代码后要发版**：
1. bump build.gradle version
2. `gradle build --no-daemon`（日志应有 `[coreswap] 已同步 MSVC dll`）
3. 验证：jar 内 dll = build-msvc 的 dll（哈希一致）；block_probe 主世界 100%
4. `gh release` 删旧 tag 建新（tag 名 `coreswap-1.20.1-1.0.x`，标 Pre-release）
5. 发布铁律记忆：core-swap-release-dll-msvc-verification

---

## 八、铁律（违反必被用户骂）

1. **C++ 必须 MSVC 构建**——MinGW 严格禁用（thread_local 退化→跨线程共享缓存→假 bug；用户原话「游戏都用标准 MSVC 是有原因的」）
2. **提交 author 必须 unknowbug**（`git -c user.name=unknowbug -c user.email=unknowbug@users.noreply.github.com commit -m "中文信息"`）
3. **主世界 100% 是铁律**——任何改动后必须 block_probe 回归
4. **知识库 docs/ 追加式更新，禁止覆盖**；已解决项标 ✅/❌ 不删除历史
5. **不在 GitHub Issue 直接回复**（除非用户明确指示）
6. **全版本覆盖是真实目标**（含 1.17+）——对外文档禁止「不计划」措辞
7. **FEATURES（矿物/装饰/结构）绝不做 C++ 化**（版本更新必动，适配忙不过来）
8. **不放弃原则**：除非用户命令停止，否则持续推进
9. **近似优化已被用户否决**——禁止任何破坏逐位一致的近似
10. **PowerShell 拼接写 C++ 源文件会截断**（曾只剩 130 行）——必须用 edit_file
11. **负坐标参照导出前删 run\world\region 测试 .mca**（vanilla 参照污染）
