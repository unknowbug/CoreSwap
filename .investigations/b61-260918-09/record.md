# b61-260918-09 — bulkwb/skipair 发射面实测（SCOPED 判据加固 + #184 dissent 闭合）

日期锚：2026-09-18 21:04-21:1x（Get-Date 实取，本块 260918-09）。HEAD 起点 `ea31c32`。

## §0 目的

闭合 260918-08 遗留两缺口：
1. **#184 dissent**——本体豁免①（A/B 实验专用）依赖「扩展读法」，dissent 在案（260918-08 record §5.3）；若按字面严格读本体可能落 DEFECT。
2. **judge S1**——发射面实测列为可选加固，未做时 §9.7 口径缺口在案。

## §1 方法（复用 #170 发射面直证法）

- init script：`.tmp/260918-09/print_vmargs_stop.gradle`（derived）——`runServer` `doFirst` 打印 `t.jvmArgs`（发射点本体）后抛 `StopExecutionException` 跳过服务器启动（本块新形态，省去 kill java；打印观测点与 260917-06 各哨兵同位，已验证有效）。
- 命令：`gradle :runServer -I <init> -Pbulkwblog=1 -Pbulkwb=1 -Pskipair=1`（workdir `versions/1.20.1/java`，GRADLE_USER_HOME 指工作区）。12s，BUILD SUCCESSFUL，exit 0。
- 日志：`.tmp/260918-09/emission-bulkwb.log`（derived）。

## §2 结果（单轮 E1，组内标定）

| 臂 | `-P` 输入 | 发射面观测 | 判定 |
|---|---|---|---|
| 标定（仪器消费点） | `-Pbulkwblog=1` | `VMARG: -Dcoreswap.bulkwblog=1` **发射** | 仪器本轮有效（同轮自证） |
| 目标本体 | `-Pbulkwb=1` | `-Dcoreswap.bulkwb` **缺席** | `-P` 通道不存在 |
| 目标本体 | `-Pskipair=1` | `-Dcoreswap.skipair` **缺席** | `-P` 通道不存在 |

- argfile 复核：`build/loom-cache/argFiles/runServer` 仅含 classpath 条目，无隐藏 `-D`（排除「藏进 argfile」分支）。
- 静态面对照：`build.gradle:173-179` 仅仪器 4 项有 `findProperty` 映射（bulkwblog/wbcontent/bulkwbtest/bulkwbsentinel）；`bulkwb`/`skipair` 零映射（gate M2=138 wrapper 捕获面一致）。

## §3 裁决

**维持 260918-04 confirmed（bulkwb/skipair = SCOPED-DIRECT-D），无 §15.4 取代；#184 dissent 关闭。**

理由：发射面证明「`-P` 通道对本体消费点**结构性不存在**」（同轮标定排除仪器失效解释）。按 DEFECT 严格读法补 `-P` 映射，等于把「缺省权威 flag」（javadoc 未宣传、直传 `-D` 通道）改造成 `-P` 通道，恰是 260918-04 定性所排除的方向；#184 边界栏预设的「字面严格读 → DEFECT」分支被行为证据封死——**先例不可跨消费点借用的判据（#184）不是读法分歧，而是发射面事实**。S1 口径缺口随之闭合（§4）。

## §4 §9.7 可比性声明

- 等价档位：**E1**（同构建态、同轮单变量组内差分）；共享观测 key 集 **S = `t.jvmArgs` 发射面**。
- 载体：1.20.1 `:runServer`（gradle 8.13，GRADLE_USER_HOME=工作区）。
- 覆盖面边界（诚实声明）：止于**发射面**（Partial），未跑 fork JVM 端到端消费行为（消费点 `BulkWb.java:90`/`CppBridge.java:642` 直读 `System.getProperty`，1.20.1 一手源已核，无端到端新信息可采）；**1.21.6 侧未实测**，其静态零映射由 `check_switch_mapping.py` M2=138 wrapper 捕获面覆盖，未跑面在此声明。
- 预登记判据（本轮执行前写死于 §2 表）：标定 emitted=true 且目标两项 emitted=false ⇒ 裁决如 §3；任一不满足 ⇒ 裁决改写并回 fan-out。**全部满足，无灰区。**

## §5 §9.8 副作用与逆

| 副作用 | 类型 | 逆 |
|---|---|---|
| init script + 日志（.tmp/260918-09/） | derived（新命名） | 天然合规 |
| `StopExecutionException` 跳过 runServer action | 无执行 | 无 world/端口副作用 |
| gradle/loom 构建缓存触碰 | in-place | **显式不可逆声明**：gradle 自身缓存行为，非本课题产物（同 260917-06 §90 先例），失败后果仅缓存重建 |

## §6 状态

本块定性：**candidate**（发射面单轮实证 + 同轮标定；confirmed 留用户）。judge 待收尾执行。
