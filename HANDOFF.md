# CoreSwap 会话交接（2026-08-05 晚）

> 本文件是给下一个会话的现场快照。接班 AI 先读本文件 + `git status`，再开始干活。

## 一、项目与环境

- **项目**：CoreSwap —— Minecraft 1.20.1 worldgen C++ 复刻（JNI 桥 + 方块层），目标与 Java 逐位一致。
- **仓库**：https://github.com/unknowbug/CoreSwap（PUBLIC，master，gh 账号 unknowbug，git author 严禁改）。
- **代码**：`versions/1.20.1/cpp/`（worldgen 模块 + ai 占位）；Java 侧 `versions/1.20.1/java/`。
- **环境**：Windows；MinGW gcc 16.1.0（`E:\PYTHON\MC\tools\mingw\mingw64\bin\c++.exe`）；JDK 17（`E:\python\MC\tools\jdk17\jdk-17.0.20+8`）；CMake 4.4.1。
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

## 四、未解之谜（第一优先级）

### 谜 A：99.78%（bash-46）vs 97.73%（bash-64 之后）——代码 diff 干净却结果不同

- 时间线：bash-46 ≈ 15:05 跑出 99.78%；bash-64/70/71 ≈ 16:0x 跑出 97.73%（稳定复现）。
- 期间代码：git diff 相对 HEAD 只有上述 4 项修复（+ ore_vein.h 调试打印 + BlockProbe 诊断代码，均不影响逻辑）。
- vanilla 文件：seed 从未变（有符号 -8248318472910187742）。文件在 14:58（bash-33）和 16:09（bash-69）被重新导出过，内容是否一致**未验证**。
- 97.73% 的差异构成（diag_full.py）：**vanilla 有大量矿脉**（granite 4636 / tuff 3668 / diorite 4042 / andesite 4609 / copper_ore 306，集中在 chunk x=200/201），**C++ 无任何矿脉**；另有 air→deepslate 2938、air→water 2688。
- **假设 1**：14:58 导出的 vanilla 无矿脉（旧 world？）→ bash-46 的 99.78% 是"无矿脉可比"的假象，97.73% 才是真实水平。
- **假设 2**：bash-46 期间代码另有差异。
- **建议第一步**：用当前 vanilla（16:09，含矿脉）确认 97.73% 是当前真实基线；然后集中修 OreVein（见下）。

### 谜 B：OreVein（矿脉）C++ 零输出

- `ore_vein.h` apply 逻辑对照 Java 正确；`fillFromNoise` 中 `if (block<0) block = oreVein.apply(...)` 已接。
- veinToggle 采样验证：y=-64 时 0.0（正常，矿脉范围 y∈[-60,51] 外）；**y=-60 时 -0.1572（非 0，组件工作正常）**。
- 但 y=-60 是边缘（e=0.1572, f=-0.2, e+f<0.4 → 返回 -1 正常）。**矿脉深度中部（如 y=-30）的 toggle 值、veinRidged/veinGap/random 判断链**尚未验证。
- ore_vein.h 里残留 `[ov]` 调试打印（前 5 次 y∈[-60,51] 采样）——接班后先看输出再决定删/留。
- Java 参照：`OreVeinSampler.java`（data/mcsrc），veinToggle/veinRidged/veinGap = router 分量。

### 谜 C：含水层残余矛盾（次要，待矿脉修完再回头）

- 当时调试（位置 3215,-26,3200 等）：r/s/t blob 的 floodedness/erosion/depth 与 Java **逐位一致**（CppCmpS 验证），但 C++ 液面全 INT_MAX → e=0 → 返回 WATER，vanilla 却是 air。
- 该结论基于当时（可能无矿脉）的对比；**矿脉修好后需重验**（可能 vanilla 差异源其实主要是矿脉而非含水层）。

## 五、待清理

- `ore_vein.h`：`[ov]` 调试打印。
- `BlockProbe.java`：`[VgDiag]`、`[CppCmp]/[CppCmpS]`、`[AqApply]`（反射失败分支）、`worldSeed` 打印——这些是诊断代码，可删或保留（保留则跑一次 runServer 慢 ~40s）。

## 六、纪律（上一个会话的教训，务必遵守）

- **思维链禁噪声**：推理用编号短句/自然语言；禁止 `！！！`、`！！`、`——` 等连续重复符号（上一个会话因卡壳反复输出这种符号，被用户手动停止）。
- **卡壳熔断**：同一问题推理 ≤2 轮无进展 → 立即改用工具验证 / 向用户明确汇报卡点，禁止原地绕第三圈。
- **数据说话**：拿不准就先跑工具（diag、探针、git diff），别纯推理。

## 七、下一个会话的第一个动作

1. `git status` + `git diff --stat` 确认工作区（4 项修复在、ore_vein.h 调试打印在）。
2. 读 ore_vein.h 当前 `[ov]` 打印需要重跑 got_export 才能看到 → 先直接加个更全的调试（或直接跑一次 got_export 看 toggle 中部值）。
3. 用 `diag_full.py` 确认当前基线（应为 ~97.7%，含矿脉差异）。
4. 修 OreVein → 目标先把矿脉差异清零 → 再回头含水层 → 100% 后性能优化（紧凑数组+索引）。
