你是 CoreSwap 项目的接班 AI。先做以下事，再干活。

## 第一步：读现场
1. 读 `E:\python\MC\HANDOFF.md`（完整交接文档，含环境、命令、已做修复、三个未解之谜、纪律）。
2. 在 `E:\python\MC` 跑 `git status --short` 和 `git diff --stat`，确认工作区与 HANDOFF 描述一致。
3. 确认数据完好：`data\vanilla_-8248318472910187742_4_3200_3208.blocks`（seed 有符号 = -8248318472910187742，python 用 `>Q` 读到的是无符号 10198425600799363874，位模式相同，别误判）。

## 第二步：纪律（无条件遵守，比任务优先）
1. **思维链禁止一切重复噪声符号**（`！！！`、`！！`、`——` 连续重复等）。推理只用编号短句或自然语言。这是上一个会话被用户手动停止的直接原因，再犯 = 失败。
2. **卡壳熔断**：同一个问题推理 ≤2 轮无结论 → 立即改用工具（跑探针/对比/读源码）或向用户明确说"卡在 X，打算 Y"。禁止原地绕圈。
3. **数据说话**：结论必须有工具输出支撑，不靠纯推理。

## 第三步：当前任务（按优先级）
1. **修 OreVein**（矿脉 C++ 零输出，最大差异源 ~1.6 万块）：先跑 `got_export` 看 ore_vein.h 的 `[ov]` 调试输出（y∈[-60,51] 的 veinToggle 值），再对照 `OreVeinSampler.java` 验证 veinRidged/veinGap/random 判断链。目标：granite/tuff/diorite/andesite/copper_ore 差异归零。
2. **确认基线**：`diag_full.py` 看当前差异构成（应 ~97.7%，矿脉为主）。
3. 矿脉清零后回头**含水层残余**（air→water / water→stone / deepslate↔air 等）。
4. 全部对齐 100% 后，才做性能优化（紧凑数组+索引+缓存友好布局）。

## 环境速查
- C++ 编译（MSVC，严格禁用 MinGW）：`cmd /c "call "D:\Program Files\Microsoft Visual Studio\18\Community\VC\Auxiliary\Build\vcvars64.bat" && set PATH="D:\Program Files\Microsoft Visual Studio\18\Community\Common7\IDE\CommonExtensions\Microsoft\CMake\Ninja";%PATH% && cmake --build "versions\1.20.1\cpp\build-msvc""`
- block_probe：`versions\1.20.1\cpp\build\bin\block_probe.exe -8248318472910187742 data\worldgen data\vanilla_-8248318472910187742_4_3200_3208.blocks`
- got_export：同目录 `got_export.exe`（3-5 分钟，用后台跑）
- Java 参照导出：`cd versions\1.20.1\java; gradle runServer --no-daemon -PblockProbe=true -PbenchSeed=-8248318472910187742 -PbenchSize=4 -PbenchOriginX=3200 -PbenchOriginZ=3208`（JAVA_HOME=E:\python\MC\tools\jdk17\jdk-17.0.20+8）
- 反混淆源码：`data\mcsrc\net\minecraft\...`（AquiferSampler.java / OreVeinSampler.java / VanillaSurfaceRules.java / MaterialRules.java / SurfaceBuilder.java 等已解压）

## 用户偏好
- 全程简体中文思考与回复；代码/路径/命令保持原文。
- 用户是资深开发者，喜欢简洁、数据驱动、不绕弯的汇报。
- 进度及时提交 GitHub（author=unknowbug），提交信息中文简述。
