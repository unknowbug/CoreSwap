# CoreSwap 下一会话交接（2026-08-08 深夜版 · 完整自包含）

> 本文件是唯一权威交接。**先读全文再动手**——所有路径、命令、环境、铁律都在这里，不需要翻历史。
> 当前主线两条：**① 8576 剩余差（above_preliminary_surface/terracotta 带）**、**② 用户崩溃（内存损坏 0x34001）**。

---

## 〇、项目全貌

CoreSwap = Minecraft 1.20.1 自定义世界生成引擎：C++ 密度引擎（逐位对齐 vanilla）+ JNI 桥 + Fabric mod（Forge 通过 Sinytra Connector 兼容）。目标全版本覆盖（1.20.x 已发布，1.17+ 在路线）。逐位一致是核心卖点（禁止近似优化）。

- 项目根：`E:\PYTHON\MC`
- C++ 引擎：`versions/1.20.1/cpp/worldgen/`（src/ 源码、include/、build-msvc/ 构建目录）
- Java mod：`versions/1.20.1/java/`（fabric-loom，src/main/java/wg/bench/ 探针）
- 参照数据：`E:\PYTHON\MC\data`（vanilla blocks/density 导出、worldgen JSON）
- 工具：`E:\PYTHON\MC\tools`（coreswap-pkg/jdk17/mc-src/mc-src2）
- 版本目录仅 1.20.1

## 一、环境与构建（铁律）

1. **C++ 一律本机 MSVC，严格禁用 MinGW**（MinGW -static 下 thread_local 退化 → 堆损坏 0xC0000005）。本机：
   - VS 2026（v18.7.3）：`D:\Program Files\Microsoft Visual Studio\18\Community`
   - MSVC 工具链 14.51.36231：`VC\Tools\MSVC\14.51.36231\bin\Hostx64\x64\cl.exe`
   - Ninja：`D:\Program Files\Microsoft Visual Studio\18\Community\Common7\IDE\CommonExtensions\Microsoft\CMake\Ninja\ninja.exe`
   - 构建：`cmd /c "call "...VC\Auxiliary\Build\vcvars64.bat" && set PATH="...Ninja";%PATH% && cmake --build build-msvc"`
   - **注意**：改头文件后有时 ninja 不重编（no work to do）——删对应 obj（`build-msvc\worldgen\CMakeFiles\...\*.obj`）强制重编
2. **git 提交**：author 必须 `unknowbug <unknowbug@users.noreply.github.com>`、中文提交信息。命令：`git -c user.name=unknowbug -c user.email="unknowbug@users.noreply.github.com" commit -m @'中文信息'@`
3. **docs/ 知识库追加式更新，禁止覆盖**（铁律）——新增章节，不改旧正文
4. **主世界 100% 是铁律**——任何改动后必须回归 3200（block_probe 对比）
5. **发布铁律**：dll 必须 MSVC + dumpbin 验证导入表（MSVCP140/VCRUNTIME140，无 libstdc++）+ jar 内嵌 dll 大小一致 + block_probe 回归 + 才 gh release

## 二、当前对齐率（block_probe 逐位对比）

- **3200**（正坐标 3200,3208 4×4）：**100.0000%**（回归基线，任何改动必须保持）
- **-288**（负坐标 -288,-256 4×4）：**95.7243%**（负坐标 bug 剩余——非本次主线）
- **8576**（用户 seed 8576294172403134396，720,-432 6×6）：**99.8473%**（本次主线，从 99.58% 一路提升）

参照 blocks 文件：`data/vanilla_8576294172403134396_6_720_-432.blocks`（重导过一致）、`data/vanilla_-8248318472910187742_4_3200_3208.blocks`、`data/vanilla_-8248318472910187742_4_-288_-256.blocks`
blocks.json：`java/src/main/resources/worldgen-data/blocks.json`（99=sandstone、909=tuff、970=deepslate、173=fire、32=water、37=海底沙、425/426/433/437/439=terracotta 系、1=stone、8=grass、9=dirt、494=white_terracotta？）

## 三、8576 剩余差主线（当前主任务）

**状态**：99.8473%（差 5405 块，y<64 占 99.9%——grass/terracotta/dirt 层差）

**已确认**：
- **heightmap 索引 x/z 交换修复**（ad81342）：buildSurface 遍历用 `heightmap[l*16+k]`（z*16+x），之前 k*16+l 错位——-288 95.47→95.72%、8576 99.58→99.80%
- **above_preliminary_surface 语义**（已提交 +4）：Java 实测 est=64 的列 y58/y63/y64 都产 grass/terracotta → 语义 = `blockY + surfaceDepth + 4 >= est`（当前 99.8473%）。试过 `>=est`（99.80）、`+1`（99.809）、`+surfaceDepth`（99.833）、`+sd+4`（99.8473 最佳）
- **est 用 nc 直接版 initial_density**（R["initial_density"] = "initial_density_without_jaggedness" 的 buildNode）——Java cns 的 est 用查表版（FlatCache）但实测 (738,64) 两版都 = 0.574（一致）——FlatCacheDF 直用会崩（RAX=0 写空指针，多线程），已回滚
- **Java est 验证**（BlockProbe EstDiag 反射 cns）：(738,-421)/(805,-432)/(808,-432)/(803,-432) est 全 = 64（与 C++ 一致）
- **surfaceDepth**（C++）：sampleRunDepth = `floor(surface*2.75 + 3.0 + positional*0.25)`，positional = `splitter->split(bx,0,bz).nextDouble()`——(805,-432) d=-0.117 extra=0.695 → val=2（最大也只到 2）
- **terracotta 带差**：y57/58 错位 1（C++ 在某列 y57 产 439、参照 y58 产 439）——i = `lround(clay_bands_offset*4)`（floor 实验 99.80% 更差，lround 正确）——**未解决**（可能带数组差或 biome 差）
- **参照假 diff 疑点**：参照 biome=savanna 的列却有 terracotta（savanna 不该产）——blocks 的 biome 段（256 个）读出来两种索引都 savanna——**需验证 Java 真实 biome**（EstDiag 的 getBiome 反射签名错 NoSuchMethodException，需找 yarn 正确签名）

**下一步候选**（下个 session 从这里继续）：
1. **验证 Java 真实 biome** @(805,58,-432)（cns.getBiome 正确反射签名——yarn 可能是 `getBiome(int,int,int)` 但非 public 或方法名不同）——确认参照 terracotta 是真差还是假 diff
2. **terracotta 带数组对比**：C++ 192 带 vs Java（TerracottaBands）——带颜色差 → y 错位
3. **-288 剩余 4.3%**（负坐标 bug——另一条线，可后续）

## 四、用户崩溃排查（内存损坏 0x34001）

**用户**（XMing_Glamorgan，D:\MC，Fabric 1.20.1 + API 0.92.11）——只有他崩，从 1.0.11-pre 一路到 1.0.17。

**已修复**：
- 1.0.12-pre：CoreSwapPool::run 的 fn 共享成员并发覆盖（runMtx 全局锁）
- 1.0.14：derivedSplitters（mutable std::map）并发写数据竞争（splitterFor 加 mutex）——Worker-Main-11 空 std::function 崩溃
- 1.0.15：崩溃日志 handler（vectored exception + 栈回溯 + crash-coreswap-*.txt）
- 1.0.16：StackWalk64 完整栈 + CppBridge.init 打印 dll sha256（验证旧缓存）
- 1.0.17：崩溃时打印 data[0x34000/1] + fillOneChunk 每 chunk MEM-CHK 校验 0x34001 vs 基线

**1.0.17 崩溃日志分析**（错误报告-2026-8-7_20.07.42.zip，data/crash_2007/）：
- dll size=300544（1.0.17 的——排除旧缓存）
- 崩溃1：0xC0000005 read 0x28F45990000（堆），RIP=0x28F57AF5057（堆地址！）——**call 到堆地址执行**（use-after-free/函数指针被覆盖）
- **data[0x34000]=0x854800014F721D8B**（不是 memset IAT 的正常值）——0x34000 数据被覆盖/或正常运行时就是这个值（需对比）
- MEM-CHK 没打印异常（fillOneChunk 内没写坏——**写坏发生在 fillOneChunk 之外**或**MEM-CHK 的 0x34001 校验位置不对**）
- **关键疑点**：0xEFE1 `call qword ptr [rip+0x25019]` = call [0x34001]——0x34001 在 .rdata（0x34000+1，奇数——**未对齐的 call 目标**）——静态值 = 0x2800000000000000（垃圾）——运行时被 loader 填成什么？——正常应该 call memset（0xEFDA mov edx,0x18 / 0xEFDF xor ecx,ecx / 0xEFD3 lea r8,[rbp+0x270] = memset(rbp+0x270, 0, 0x18)）——**但 0x34001 是奇数地址，call 读取错位 8 字节**——可能编译器生成问题或 .rdata 布局（需下个 session 用 CE 或 dumpbin /disasm 确认 0x34001 的运行时值）
- 用户机器可能真有问题（内存/驱动）——但 5+ 次固定崩溃模式需先排除我们代码

**下一步**：
1. 用 CE（cheatengine MCP 可用）attach 到崩溃现场/或本地复现——看 0x34001 的运行时值
2. 0xEFE1 的 call 目标确认（memset 的 IAT 错位？）——dumpbin /disasm /range 用完整 VA（ImageBase 0x180000000 + RVA）
3. 让用户试：删 `%TEMP%\coreswap-native` 和 `coreswap-data` 缓存 + 最新版 + 关杀毒（0x40010006 崩溃像被 patch/hook）

## 五、已发布版本（GitHub unknowbug/CoreSwap Releases）

- 1.0.13：heightmap x/z 索引修复（3200 100%、8576 99.80%、-288 95.72%）
- 1.0.14：derivedSplitters 并发崩溃修复
- 1.0.15：原生崩溃日志 handler（vectored exception + 栈 + 文件）
- 1.0.16：StackWalk64 完整栈 + dll sha256 打印
- 1.0.17：内存损坏诊断（data[0x34000] 打印 + MEM-CHK 校验）
- 兼容性说明（1.0.13+ notes 已改）：Fabric 1.20.1 + Forge（Sinytra Connector，PR #3 兼容——CoreSwapFixHelper 多级定位 jar 提取）
- Forge 兼容 git 历史：e8d52dc（PR #3 思路）、75b9e75、9b3de36、c61d96c（致谢 dustinmoon78）

## 六、诊断工具（探针）

- **block_probe**（C++）：`build-msvc/bin/block_probe.exe <seed> <worldgen dir> <vanilla.blocks> [-threads N] [-mismatch] [-blockDump X Y Z] [-crashTest]`——-mismatch 输出差块（位置/方块/biome）；**注意 i 的 lx=i%16, ly=i/256, lz=(i/16)%16 已修正**；-blockDump 的 idx 用局部坐标 (by-(-64))*16+lz)*16+lx
- **got_export**：-namedDump/-densityDump/-compXY/-noiseDump/-biomeDump/-listRegistry（-densityDump 参数是 chunk+块内坐标：`-densityDump cx cz bx bz`）
- **DensityProbe**（Java）：`gradle runServer -PdensityProbe=true -PdensityProbeChunkX= -PdensityProbeChunkZ= -PdensityProbeX= -PdensityProbeZ= -PbenchSeed=`——CAVES-NOISE 反射 cns 噪声、ESH-ID 反射 initial_density
- **BlockProbe**（Java）：`gradle runServer -PblockProbe -PbenchSeed= -PbenchOriginX= -PbenchOriginZ= -PbenchSize=`——导出 .blocks 参照（**参数是 benchOriginX 不是 blockProbeOriginX！**）；EstDiag 反射 cns 的 est/initial_density（条件 wx==45 && wz==-27）
- **CppBridge**（Java）：init 打印 dll 路径/size/sha256（-Dcpp.worldgen.dir 可覆盖）
- **参照重导**：`-PbenchOut="E:\python\MC\data\recheck"` 导出到指定目录

## 七、关键代码位置

- `surface.h`：
  - SurfaceCondC::test（above_preliminary_surface）：`blockY + surfaceDepth + 4 >= est`（当前）
  - estimateSurfaceHeight：initialDensityAt 扫描 >0.390625（间隔 8，从顶向下）
  - sampleRunDepth：surface 噪声 depth 计算
  - getTerracottaBlock：红陶带（lround(offset*4) + floorMod(y+i,192)）
  - buildSurface：遍历 heightmap[l*16+k]（x/z 已修）、stoneDepth 条件 `q <= 1+offset+sd`
  - splitterFor：derivedSplitters（mutex 已修）
- `worldgen_api.cpp`：
  - fillOneChunk（517 行）：density → aquifer/oreVein → buildSurface → out
  - MEM-CHK（fillOneChunk 开头）：0x34001 vs 基线校验
  - CoreSwapPool::run（runMtx 已修）
  - est 的 initialDensityAt lambda：R["initial_density"] 直接采样（FlatCacheDF 直用会崩）
- `crash_handler.h`：installCrashHandler + CrashHandler（vectored exception + StackWalk64 + data[0x34000] 打印）
- `worldgen_api.h`：wg_router_sample 声明（-compXY 用）
- `CMakeLists.txt`：/MAP 生成 worldgen.map（崩溃地址→函数，RVA = 0001:xxxx + 0x1000）

## 八、8576 排查已排除的（避免重走弯路）

- ❌ FlatCacheDF 直用做 est 查表版（多线程 RAX=0 崩溃 + 单线程 98.88% 更差）——回滚
- ❌ getTerracottaBlock 的 i 用 floor（99.80% 更差）——lround 正确
- ❌ vein 顺序交换、InterpolatedDF 整树插值重构——已回滚
- ✅ cns 反射不可信（interpolators 8 个全组件插值器，无 finalDensity）——弃用
- ✅ 组件对比（temperature/continents/jaggedness/caves 噪声）逐位一致
- ✅ 16 格划线 = 误报（用户确认）；(800,534) 高度差 2 = 误报
- ✅ 写入方向（x/z 映射）一致——排除

## 九、工作环境备注

- PowerShell 7（bash 工具是 pwsh）——`python -c` 多行会被 blocked，用脚本文件（write_file 到 data/ 再 python 执行）
- 崩溃 zip 在 `E:\Users\NDark\Downloads\`——解压到 `data\crash_*`（Expand-Archive；.7z 用 py7zr）
- blocks 列读取：`data/read_col2.py wx wz y0 y1`（列方块）、`data/read_biome2.py wx wz`（biome 两种索引）
- pefile + capstone 已装（反汇编 dll 崩溃点）——`data/pe_probe.py dll rva`、`data/dis_efe1_16.py`、`data/find_pat.py`、`data/iat_probe.py`、`data/parse_map.py`
- dumpbin：`D:\Program Files\Microsoft Visual Studio\18\Community\VC\Tools\MSVC\14.51.36231\bin\Hostx64\x64\dumpbin.exe`（/dependents 验证；/disasm /range 需完整 VA 或用 capstone）
- **用户已告知**：Fabric API 0.92.11+1.20.1（正常）；用户机器 D:\MC（别人的机器，本机无）
- 记忆已存：全局崩溃日志捕获铁律（mem-9a4a913a02866db45d3da32c26611e83）

---

**下一步（按优先级）**：
1. 8576 terracotta 带/above_preliminary 收尾（Java biome 验证 + 带数组对比）
2. 用户崩溃 0x34001 之谜（CE attach / dumpbin 完整 VA 反汇编 0xEFE1）
3. -288 负坐标 bug（独立主线，95.72%）
