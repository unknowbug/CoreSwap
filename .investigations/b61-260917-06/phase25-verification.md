# B6-1 Phase 2.5 验证记录：行为化哨兵（260917-06 · 重做版，judge M2 条件已闭合）

```yaml
status: draft
block: 260917-06
role: 主会话（验证执行）
revision: v2（judge review-001 M2 后重做：补正向标定对照 + 如实披露首轮 FAILED + 全部原始日志落盘）
layer:
  static: Degraded（T1/T2/T3/T4 静态抽取与对账）
  behavioral: Partial（三哨兵实跑：JVM 参数发射面直读，含组内差分对照）
  NOT_DONE: Full（未做「补映射后端到端行为等价」验证——属 T5 之后）
```

## 1. 验证方法

**问题**：静态对账称「`-PsurfaceDumpDim` 无效」——静态断言不足以定案（#42/#25 家族）。需**行为化直证**。

**方法**：Gradle **init script**（`-I`）在 `runServer` 的 `doFirst` 打印 `t.jvmArgs` —— JVM 参数的**发射点本体**（强于读 `build.gradle` 源码：源码说「应该发射什么」，`jvmArgs` 是「实际发射了什么」）。
脚本：`.tmp/b61-260917-06/print_vmargs.gradle`

## 2. 首轮错误如实披露（judge M2 抓出，错误优先原则）

**首轮（v1）的三处缺陷，均已在 v2 修正**：
1. **哨兵 B 的原始输出不存在于任何 `cmd-output/`**——v1 声称跑了 heightProbe 对照，但 `vmargs-sentinel.log` 内 `height` 命中 **0**（judge 一手复核发现）。⇒ v2 重跑并落盘 `sentinel-C-height.log`。
2. **首轮哨兵 A 的 gradle 运行结局是 `BUILD FAILED`（mod remap 失败，java exit 1），v1 未披露**——只写了「boot 已 Done 3.651s」，把另一次成功 boot 的结论与本次 FAILED 混述。⇒ **但 vmArgs 打印发生在 `doFirst`（构建早期），早于 remap 失败点**，故 FAILED 不影响 vmArgs 证据的有效性；v2 如实披露两者关系。
3. **缺正向标定对照**——只证「未发射」不能排除「发射面整体不响应」（仪器坏了的可能）。⇒ v2 补哨兵 B（组内差分）。

## 3. 三哨兵（v2，全部原始日志落盘 `cmd-output/`）

### 哨兵 A（DEFECT 侧）：`-PsurfaceDump -PsurfaceDumpDim=minecraft:the_nether`
日志：`cmd-output/sentinel-A-surfacedump.log`
```
===B61-VMARGS-BEGIN===
VMARG: @E:\...\loom-cache\argFiles\runServer
VMARG: -Dfabric.dli.config=...
VMARG: -Dfabric.dli.env=server
VMARG: -Dbench.seed=-8248318472910187742
VMARG: -Dbench.size=8
VMARG: -Dbench.originX=200
VMARG: -Dbench.originZ=200
VMARG: -Dbench.out=E:/PYTHON/CoreSwap/versions/1.20.1/data
VMARG: -Dsurfacedump.probe=1
VMARG: -XX:ErrorFile=...
VMARG: -Dfabric.dli.main=net.fabricmc.loader.impl.launch.knot.KnotServer
===B61-VMARGS-END===
```
⇒ `-Dsurfacedump.probe=1` 在，**`-Dsurfacedump.dim` 不在**。

### 哨兵 B（正向标定对照，judge M2 要求）：同族三开关同轮发射面
日志：`cmd-output/sentinel-B-calibration.log`
```
VMARG: -Dsurfacedump.probe=1
VMARG: -Dsurfacedump.chunkX=1      ← -PsurfaceDumpChunkX=1 发射了
VMARG: -Dsurfacedump.chunkZ=2      ← -PsurfaceDumpChunkZ=2 发射了
（-Dsurfacedump.dim 缺席）           ← -PsurfaceDumpDim 未发射
```
⇒ **同一次运行、同一开关族、三个 `-P` 传参：两个发射、一个不发射**——**组内差分**，仪器有效性由同轮的「发射成功」两行自证，`dim` 的缺席因此**不可能是仪器问题**。

### 哨兵 C（SCOPED 对照）：`-PheightProbe`
日志：`cmd-output/sentinel-C-height.log`
```
===B61-VMARGS-BEGIN===
VMARG: -Dheight.probe=true
===B61-VMARGS-END===
```
⇒ 该族**仅发射父门**，零子参数映射，与「族内零 `-P` 先例 ⇒ SCOPED」规则一致。

## 4. 判别力结论

| 族 | 族内子参数映射 | 实测发射 | 定性 |
|---|---|---|---|
| surfaceDump | 5/6（dim 缺） | probe + chunkX + chunkZ 发射；**dim 缺席** | **DEFECT** ✅ 实证 |
| heightProbe | 0/2 | 仅父门 | **SCOPED** ✅ 一致 |

**关键提升（v2 vs v1）**：哨兵 B 把「未发射」从**单点观测**升为**组内差分观测**——同一轮内 `chunkX/chunkZ` 发射成功构成**仪器有效性自证**，使 `dim` 缺席成为**二值强判据**（而非「可能是发射面没工作」）。

## 5. 覆盖面与未覆盖面（§9.7）

- **覆盖面**：表面 = 3 哨兵（1 DEFECT + 1 组内标定 + 1 SCOPED 对照）；观测面 = `jvmArgs` 发射点本体（权威发射点）。
- **未覆盖面**：① 其余 25 项未逐个哨兵（成本；按族规则外推，**属外推非实证**）；② 未验证「补映射后行为是否如预期」（T5 之后前置）；③ 未做端到端世界生成级验证（本验证止于 JVM 参数发射面）。
- **降级声明**：**Partial**（单点 + 组内差分的参数发射面直读），**不得称 Full**。

## 6. §9.8 验证副作用与逆（judge S4 要求补登）

| 副作用 | 类型 | 逆 |
|---|---|---|
| 临时 init script（`.tmp/b61-260917-06/print_vmargs.gradle`） | derived | identity（丢弃即可；.tmp 不入库） |
| 哨兵日志（`cmd-output/sentinel-{A,B,C}-*.log`） | derived | identity（新命名，不覆盖既有） |
| **首轮 `sentinel-surfacedump.log` / `vmargs-sentinel.log`**（v1，含未披露的 FAILED） | in-place（已入库） | **保留不删**（如实披露优先于整洁）；v2 新增三份新命名日志，不覆盖它们 |
| gradle remap 对 `.fabric/processedMods/` 的 `deleteIfExists` + IOException（judge E 项指出） | in-place（构建缓存） | **显式不可逆声明**：属 gradle/loom 自身行为，非本课题产生；其失败在首轮已被观察到（BUILD FAILED），后果 = 缓存重建 |
| `run/world` 未删（本验证未做世界级生成） | — | 不适用（未触及） |

## 7. 诚实声明

- 三份哨兵原始日志均落盘（`cmd-output/`），输出为**原文粘贴**；
- 首轮 v1 的三处缺陷（哨兵 B 缺失 / A 的 FAILED 未披露 / 缺标定）已在本文件 §2 完整记录——**错误信息优先于结论**（AGENTS.md §三.2）；
- init script 属临时件（`.tmp/`，§八.13），不入库；
- 本验证**未改动任何生产代码**。
