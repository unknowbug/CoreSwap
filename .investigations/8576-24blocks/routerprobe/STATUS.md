# RouterProbe 执行状态（subagent 报告）

## 结论：未执行 gradle，被沙箱拦截

执行 subagent 的工具集**没有 shell/命令执行工具**（无 bash/run_command/execute），
无法运行 `gradle runServer ...`。同时 `edit_file` 写外部参考工程路径
`E:\PYTHON\MC\versions\1.20.1\java\run\server.properties` 被拦截，错误原文：

```
error: write path "E:\\PYTHON\\MC\\versions\\1.20.1\\java\\run\\server.properties" is outside this subagent's declared write_paths
```

## 关键检查结果

### 1. seed 核对（前置纪律第 1 条）—— 不通过！

- `run/server.properties` 的 `level-seed=-8248318472910187742`（**不是 8576**）
- 存在 `run/server.properties.bak8576` 备份，其内容 `level-seed=8576294172403134396`（simulation-distance=32, spawn-protection=0）
- `run/world/` 已存在（level.dat + region 文件），且 `run/logs/latest.log` / `debug.log` 显示
  最近一次运行（8/8 16:16 BlockProbe）的 `worldSeed=-8248318472910187742`
  → **当前世界是用错误 seed (-8248...) 生成的**，必须删 `run\world` 并用
  `level-seed=8576294172403134396` 重新生成，探针数据才有效。

### 2. 探针代码确认（已读 RouterProbe.java / BenchMod.java / build.gradle）

- `build.gradle` 中 `-ProuterProbe=1` → `-Drouter.probe=true`，`-PbenchSeed=...` → `-Dbench.seed=...`
- `-ProuterX/routerZ/routerYFrom/routerYTo/routerYStep` → 对应 `-Drouter.*`，全部有映射（build.gradle L48-68）
- `RouterProbe.run()` 用 `world.getSeed()`（来自 level.dat / world seed，**不受 benchSeed 影响**），
  输出 `#seed <seed>`、`TBANDS`、`B <x> <y> <z> <t> <hum> <cont> <ero> <dep> <w>`、
  `SURFBIOME`、`BIOME` 行，末尾 `server.stop(false)` 自动停服 → 探针逻辑与任务描述一致
- 采样 y：`router.yFrom + i*router.yStep`，count = (yTo-yFrom)/yStep+1 = (104-64)/4+1 = **11 个采样点**

## 准备跑但未能执行的命令清单

```
Stop-Process -Name java -Force -ErrorAction SilentlyContinue
cd E:\PYTHON\MC\versions\1.20.1\java
gradle runServer -ProuterProbe=1 -PbenchSeed=8576294172403134396 -ProuterX=812 -ProuterZ=-337 -ProuterYFrom=64 -ProuterYTo=104 -ProuterYStep=4 2>&1 | Tee-Object -FilePath <workspace>\investigations\8576-24blocks\routerprobe\routerprobe_812_-337.txt
# 第二组（未执行）：
gradle runServer -ProuterProbe=1 -PbenchSeed=8576294172403134396 -ProuterX=815 -ProuterZ=-337 -ProuterYFrom=64 -ProuterYTo=104 -ProuterYStep=4 2>&1 | Tee-Object -FilePath <workspace>\investigations\8576-24blocks\routerprobe\routerprobe_815_-337.txt
```

## 主会话需要做的事（决策点）

1. 在**有 shell 的执行 agent** 中运行上述命令；
2. **先修正 seed**：把 `run/server.properties` 的 `level-seed` 改回 `8576294172403134396`
   （可从 `server.properties.bak8576` 恢复），**并删除 `run/world`**（当前 world 是 -8248 错误 seed 生成的，
   残留会导致探针数据错世界）；
3. `simulation-distance=2`、`spawn-protection=0` 当前 server.properties 已有，保持即可；
4. 输出文件路径：`<workspace>\.investigations\8576-24blocks\routerprobe\routerprobe_812_-337.txt`
