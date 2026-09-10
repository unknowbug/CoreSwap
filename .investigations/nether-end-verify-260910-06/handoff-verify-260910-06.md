# 交接结论廉价复核（260910-06 开工前，2026-09-10 20:1x）

> 依据：AGENTS.md §三.1「交接结论验证纪律」（NEXT/台账里的机制方向/根因方向/待查假设类结论，开工前 MUST 先做一次廉价独立验证）。
> 复核对象 = `NEXT_SESSION.md` 下轮开工点 3（重复接管调用）与 260910-05 登记后续项 1/2 的候选成因。
> 证据载体 = 只读归档 `.investigations/perf-closeout-260910-05/cmd-output/`（14 臂日志 + 11 份对拍 + region 快照 + 驱动脚本），**未重跑任何 bench**（零新开销）。
> 复核者 = 主会话（会话角色：执行 + 采集；裁定均为 **candidate**，未 confirmed）。

## V1 — 「重复接管调用（4761 > 4225）」= **REFUTED**

**被核断言**（NEXT 下轮开工点 3 / verdict-nether-end §2 ⚠️ / record §6.1-2）：
> 4761 > 4225 表明存在**重复接管调用**（同 chunk 被 `populateNoise` 多次进入）。

**方法**：从归档日志逐行抽取 `[Mixin] populateNoise(nether|end) intercepted chunk(x,z)` 的坐标，统计 **行数 vs 去重坐标数**。

```
regex = populateNoise\(nether\) intercepted chunk\((-?\d+),(-?\d+)\)
```

**结果**：

| 臂 | 维度 | 行数 | 去重坐标数 | 重复坐标组 | 坐标包围盒 |
|---|---|---|---|---|---|
| `naS-r1` | nether | 4761 | **4761** | **0** | x -37..31 × z -35..33 = **69×69 = 4761** |
| `eaS-r1` | end | 4761 | **4761** | **0** | x -37..31 × z -35..33 = **69×69 = 4761** |

**裁定**：同一 chunk 被多次进入 = **否**。4761 是 **4761 个互不相同的 chunk 各命中一次**，恰好填满一个 69×69 方块（中心 = (`-3`,`-1`) chunk = Chunky `center -48 -11` 的块坐标对应 chunk）。
超出 Chunky 计数（4225 = 65×65）的 **536 = 69² − 65²**，来源 = **额外不同 chunk**（NOISE 之上的邻域依赖环），**不是重复调用**。

**覆盖面声明（§9.7）**：载体 = 260910-05 两个 sanity 臂（nether/end 各 1，均开 `-Pmixlog=1`）；覆盖面 = 该臂**全量** mixin 日志（无抽样）；可比性 = 与 260910-05 verdict 所用同一日志文件、同一计数口径（`Select-String` 行计数）扩为坐标级；**未覆盖** = 3809 关日志的 A/B 臂（日志门控关闭，无坐标可查）与 overworld 臂（该块 overworld 未开 mixlog）。

**影响**：verdict-nether-end-260910-05 §2 的 ⚠️ 后续项与 record §6.1-2 的成因前提**被推翻**（该条目为 confirmed 正文 ⇒ 按 §15.4 走取代记录，原文不删不改，见本块计划 D3）。同时**移除**了「同 chunk 多次接管 ⇒ 双重写回/与 `feedBeardifier` 交错」这一风险面（NEXT 下轮开工点 3 可关闭）。

## V2 — 「region 目录含非本臂生成的 chunk」= **REFUTED**

**被核断言**（record §6.1-1 候选成因 / NEXT 下轮开工点 2 候选）：
> nether 噪声高（0.14% vs overworld 0.0092%）的候选成因之一 = region 目录含非本臂生成 chunk（`common` 7542/7567 > 臂内 4225）。

**方法**：直读 region 文件头（每 chunk 槽位 4 字节写入时间戳，偏移 `4096 + i*4`，big-endian u32），按臂比对时间戳是否落在该臂自身运行窗口内。

**结果**（`slots` = 该 region 目录内非空槽位总数）：

| 臂 | 槽位 | 时间戳最早 | 时间戳最晚 | 全部落于本臂窗口 |
|---|---|---|---|---|
| `naS-r1` | 7569 | 09-10 19:08:30 | 09-10 19:08:50 | ✅（Chunky 任务 19:08:28 起，21s） |
| `na-r1` | 7569 | 09-10 19:15:25 | 09-10 19:15:46 | ✅ |
| `na-r2` | 7569 | 09-10 19:16:52 | 09-10 19:17:12 | ✅ |
| `ns-r1` | 7569 | 09-10 19:13:19 | 09-10 19:14:22 | ✅ |
| `r3-r2`（overworld） | 8004 | 18:53:13 | 18:54:18 | ✅ |
| `eaS-r1`（end） | 7569 | 19:36:13 | 19:36:19 | ✅ |

**裁定**：每个臂的 region 快照**全部槽位都写于该臂自己的运行窗口**，无跨臂残留 ⇒ 驱动脚本 `run_arms3.ps1:52` 的 `Remove-Item "$run\run\world"` 生效，「陈旧 chunk 混入对照集」**不成立**。

**顺带确定的口径事实（本块新增，供计划使用）**：`common` 口径的 7542/7567/7959 **不是**接管生成集的规模——region 里 7569 = 87×87 槽位中绝大多数（≈2808）是**低状态残缺 chunk**（见 V3）。⇒ 行为门的分母包含非地形 chunk，**有效可对比地形集 ≈ 4761（nether/end）/ 4225+（overworld）**。

**覆盖面声明**：载体 = 260910-05 全部有 region 归档的 6 个臂；覆盖面 = 每臂 16 个 mca **全部** 1024 槽位（无抽样）；可比性 = 时间戳读法直接对齐驱动脚本的 per-arm `Remove-Item` 语义；**未覆盖** = `.tmp/` 下原始世界目录（已按 record §2.1 说明不入库，若需复核须重建）。

## V3 — 「2808 个被写入但未过接管路径的 chunk」= **benign（非覆盖漏洞）**

**动机**：V2 顺带暴露「每臂写入 7569 chunk，但只有 4761 走了 `populateNoise` 接管分支」的 1.6× 缺口。若缺口 chunk 是**地形 chunk 但绕过接管**，则是接管覆盖率漏洞（严重）——必须排除。

**方法**：用 260910-05 对拍工具自身的 loader（`diff_arms.py` 前 95 行 exec 复用，非重新实现）读快照，打印 chunk 的 `Status` 与「非空气 section 数」。

**结果**：

| 臂 | chunk | 位置 | `Status` | 非空气 section |
|---|---|---|---|---|
| `naS-r1` | (-3,-1) | 中心 | `minecraft:full` | 2 |
| `naS-r1` | (-20,-20) | 内环 | `minecraft:full` | 2 |
| `naS-r1` | (-40,-40) | 外环 | **`minecraft:structure_starts`** | 0 |
| `naS-r1` | (36,38) | 外环 | **`minecraft:structure_starts`** | 0 |
| `naS-r1` | (-45,-43) | 最外 | **`minecraft:structure_starts`** | 0 |
| `eaS-r1` | (-3,-1) | 中心 | `minecraft:full` | 6 |
| `eaS-r1` | (-40,-40) | 外环 | **`minecraft:structure_starts`** | 0 |

**裁定**：外环 chunk 是 **`structure_starts` 状态的残缺 chunk（section 全空）**，非地形产出 ⇒ 缺口**不是接管覆盖漏洞**；「未过接管分支」对它们而言是正确行为（它们根本没到 NOISE 状态）。
**残余待查（转入本块 scout）**：为何 Chunky 区域外还会有 9 chunk 宽的 `structure_starts` 环被**写盘**（通常低状态 chunk 不写盘）——机制未明，与噪声课题可能无关，但关系到「行为门分母语义」的完整声明。

**覆盖面声明**：载体 = `naS-r1` / `eaS-r1` 各 1 快照；覆盖面 = **5 点定向抽样**（中心/内环/三处外环），**非普查**；可比性 = 使用与 260910-05 对拍**同一 loader 代码**（同解析口径）；**未覆盖** = 外环全量 2808 chunk 的状态分布（未逐块统计）——若 scout 需要定量，须补全量 `Status` 普查。

## 三项复核的净效果（进入计划的输入）

1. **关闭** NEXT 下轮开工点 3（重复接管调用）——风险面不存在，无需再查成因。
2. **移除** nether 噪声候选成因两支：①（同 chunk 重复调用）②（对照集含陈旧 chunk）。
3. **新增确定口径事实**：行为门分母含 ≈37%（nether/end）低状态残缺 chunk，有效地形集 ≈ 4761；任何后续噪声/等价门结论 MUST 连此口径一并声明。
4. **触发一条 §15.4 取代记录**（confirmed 正文的后续项推断被推翻，原文不改）。

## 复现命令（只读，零重跑）

```powershell
# V1：坐标去重
$c = Get-Content .investigations\perf-closeout-260910-05\cmd-output\logs\naS-r1.log
$rx = [regex]'populateNoise\(nether\) intercepted chunk\((-?\d+),(-?\d+)\)'
$coords = foreach($l in $c){ $m=$rx.Match($l); if($m.Success){ "$($m.Groups[1].Value),$($m.Groups[2].Value)" } }
$d = $coords | Sort-Object -Unique; "$($coords.Count) / $($d.Count)"
# V2/V3：脚本（主会话临时区）
#   .tmp\region_timestamps_260910-06.py   （region 槽位时间戳）
#   .tmp\probe_ring_chunks_260910-06.py   （Status / 非空气 section）
```

> 上述 `.tmp` 脚本为本块一次性诊断，按「临时文件唯一区纪律」（AGENTS §八.13）落在仓库根 `.tmp/`，不入库；V2/V3 的**结论**与**方法**已在本文件内自足记录（无需脚本即可按方法复现）。
