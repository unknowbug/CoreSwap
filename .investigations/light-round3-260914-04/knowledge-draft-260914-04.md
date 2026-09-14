# 知识库草稿 — 光照 round3（260914-04）知识 worker 产出

> 产出者：知识库 subagent（按 SUBAGENT-KNOWLEDGE-GUIDE + core-knowledge 格式）。**全部为草稿，主会话逐节应用**；编号暂拟，应用前主会话 MUST 核对目标文件末条实际编号（compiler-idioms 末条 = #26 → 本稿拟 **#27**；build-tooling 末条 = #150 → 拟 **#151**；workflow-patterns 末条所见 = #146（260913-06），**主会话须再核 260914-02 后是否有更高号** → 拟 **#147/#148**）。
> 价值门自查（逐条）：A=中价值简记（惯用法）；B=高价值（判错签名+假绿掩盖面，五段式）；C1=高价值（可复用 MUST 判据）；C2=高价值（载体灵敏度判据）；一次性数字（7.3→2.94ms 等）只进 record/12 篇，不进 discovered——已按此筛。

---

## A. compiler-idioms.md 新发现草稿（拟 #27，中价值简记）

## 发现 #27 简记: `writePacket` 序列化帧 = 私有 `PalettedContainer` 内部的零 accessor 公有通道——mixin @Accessor 撞私有内部 record 时的替代路径（260914-04）

- **发现时间 / 发现者 / 置信度 / module**：260914-04；主会话（编译实证）+ judge 收尾核对；candidate（编译实证 + 双采集对拍 ALL-MATCH 行为验证；confirmed 留用户）；compiler-idioms / Java·MC 容器序列化面。
- **来源定位**：`.investigations/light-round3-260914-04/record-260914-04.md` §2（候选 B）；帧格式一手锚 = `PalettedContainer.writePacket`（PC.java:383-387，1.20.1 yarn sources）。
- **观察**：要从 Java 侧取出 `PalettedContainer` 的 palette/bits/packed 数据时，直觉路径是 mixin `@Accessor("data")`——但 `data` 字段的类型是**私有内部 record**（`PalettedContainer.Data`），accessor 接口签名无法引用该类型，**编译期即不可行**（编译实证，非猜测）。
- **证据**：候选 B 初版 `@Accessor("data")` 编译失败；改走公有 `writePacket` 后双采集对拍 4 chunk ALL-MATCH（diffTotal=0，probe-r2.log），且经生产 dev loom 环境（remap 链）运行正常。
- **如何利用**：需要私有容器内部数据时，优先找**公有序列化帧**——`writePacket` 帧自带 bits（1B）+ palette 体（按 bits 分派的单值/ID_LIST 形态）+ `VarInt(n)` + packed longs 全信息，逐字段引用 PC.java:383-387 解析即可；零 accessor / 零反射 / **零 remap 风险**（公有方法名经标准映射）。⚠️ 跨版本注意帧格式差（1.20.1 `writeLongArray` 带 VarInt 长度前缀 vs 1.21.6 定长无前缀，#26 家族——本项目 issue #26 已登记）。
- **家族索引**：build-tooling #26（Chunky 载体）/ 本稿 B 条（该帧的长度前缀漏读坑）；workflow-patterns #55（remap 环境识别）——本条是「绕开私有面」正面形态。

---

## B. build-tooling.md 新发现草稿（拟 #151，高价值，五段式）

## 发现 #151: 字节流协议解析「漏读长度前缀」判错签名三件套 + 「部分成功掩盖全流错位」——singular/免位流路径全免疫使对拍抽样假绿（260914-04）

- **发现时间 / 发现者 / 置信度 / module**：260914-04；主会话（W1 五段式，record §5）+ judge 收尾三源核对（review-final-judge §6 判「四条中知识价值最高」）；candidate（confirmed 留用户）；build-tooling / 协议解析验证有效性（#48 双采集对拍家族的**覆盖面盲区**形态）。
- **来源定位**：`.investigations/light-round3-260914-04/record-260914-04.md` §3 W1 + §5；定位链原始产物 = bin-diag `light_packed_test.rs` 合成帧单测 + Java 侧原始帧 dump（w0 十六进制）。
- **五段式（错误优先）**：
  - **现象**：packed 收集初版解码值恒撞常数（idx=15 v=8）+ 流尾残留 bufLeft=2 + nativeAvg 反升 4.36 伪装「packed 更慢」；多数 chunk rc=-2 静默回退。
  - **根因**：`writeLongArray` 帧含 `VarInt(n)` 长度前缀——B 路（Java 位流解码）有 `readVarInt`，C 路（packed 直传）算了 n 但**没读前缀** → 位流整体错位 2 字节。残留字节即前缀本体（w0 前两字节 `0x80 0x02` = VarInt(256)）。
  - **定位**：bin-diag 合成帧单测排除 Rust 解码本体（全绿）→ 分层 rc 错误码（-6/-2）定层到 Java 收集侧 → Java 原始帧 dump 十六进制一锤定音。
  - **修复**：补 `readVarInt` + 校验 `nLen==n`。
  - **教训**：见判据 2。
- **判据（可复用）**：
  1. **判错签名三件套**（字节流协议解析错位）：① 解码值**系统性撞同一常数**（错位后固定位模式重复命中）；② **流尾残留字节**（读少了，bufLeft>0）；③ **残留字节即跳过结构本体**（十六进制 dump 认出 0x80 0x02 = VarInt）。三件齐 = 长度前缀/结构头漏读，先 dump 帧头再查逻辑。
  2. **「部分成功」比全红更危险**：帧内部分记录（如 singular 非空节）**不经位流、天然免疫错位** → 全 singular 的 chunk 解析成功、恰逢此类 chunk 的对拍抽样即 **ALL-MATCH 假绿**（本例 DUALP 4 chunk 假绿、多数 chunk rc=-2 静默回退、性能读数反向伪装）。**协议解析改动的验证用例 MUST 含位流负载路径**——只含免疫路径的成功不构成帧解析证据。
  3. 静默回退 + 性能均值组合的伪装面：回退 chunk 付全价使均值反升，可被误读为「新路径更慢」——回退计数 = 0 是读均值的前提（W4 同族）。
- **家族索引**：#48（双采集对拍——本条为其**抽样覆盖面盲区**：对拍样本须覆盖非免疫路径）；build-tooling #26（#26 帧格式版本差——本条为 1.20.1 前缀形态的漏读坑）；#118（自证行硬门禁——rc 分层定层是其错误码形态应用）；workflow-patterns #81（假绿/死参数家族的协议解析形态）。

---

## C. workflow-patterns.md 新发现草稿（两条，拟 #147/#148，高价值判据）

## 发现 #147: 减法归因口径（e2e 差值剩余 = 某段 ≈0）作优化排序依据前 MUST 有分段直接计时——「余项小」可能是「余项没测」（260914-04）

- **发现时间 / 发现者 / 置信度 / module**：260914-04；worker（probe-verdict）+ 主会话探针实测 + judge §15.4 取代指示；candidate；workflow-patterns / 性能归因方法（#134/#102 家族延伸）。
- **来源定位**：`.investigations/light-round3-260914-04/probe-verdict-260914-04.md` §1 + record §1；被证伪结论 = 12-lighting.md L94（round2 减法口径「Java 段≈0.15ms/chunk」，§15.4 取代）。
- **现象/根因**：round2 由 e2e 减法推出「Java 收集/JNI 侧开销已基本消除」（3.5s ≈ 内核份额预测 ⇒ 余项记 ≈0.15ms/chunk）；round3 分段探针直接计时 = **collect 4.5ms/chunk，30×**——减法余项被「预测吻合」锁死，实际上内核份额预测本身有偏差、余项被吸收进「吻合」里。对价模型重算（ON 7.3 = collect 4.5 + native 2.8 vs vanilla ~5.6，gap 1.73ms×2025≈3.5s）双锚自洽。
- **判据（可复用）**：**减法余项（e2e 总量 − 已计量段）在用作优化排序/裁剪依据（「X 段已消除，下一头在 Y」）之前，MUST 对该段做一次分段直接计时探针**——减法的「吻合」只约束总量，不约束分账；「预测吻合 ⇒ 余项≈0」是把分辨率不足当结论（#102 量级核算 + #134 减法归因家族的**授权门槛**形态）。成本 = 一个分段计时探针（本例 ThreadLocal 累计/均值即可），远小于按错误分账排错的优化序。
- **家族索引**：#134（减法归因口径）/ #102（量级核算排除法）——本条为其 MUST 前置条件；#22（自由参数凑数——「吻合」作证据的反面形态）；#128（口径纪律：串行 per-chunk vs e2e wall）。

## 发现 #148: 「boot Done」作 e2e 判据载体的噪声下限——OFF 臂 n=6 极差 15% + 跨批漂移 2s，缺口 1s 级效应不可判；灵敏度需求 <2s 时 MUST 换载体（260914-04）

- **发现时间 / 发现者 / 置信度 / module**：260914-04；主会话（六对交错实测）+ judge §7 遗留风险 3；candidate；workflow-patterns / 测量载体选择（#51/#103 噪声带家族）。
- **来源定位**：record §4（六对交错 21:00-21:12 批）+ review-final-judge §2/§7；e2e-on/off 原始日志 12 件（cmd-output）。
- **现象**：runServer boot Done 作 e2e 判据时：OFF 臂 n=6 极差 2.6s（15.69-18.31，**15%**）；整批相对前批系统性漂升 ~2s（机器负载，跨批绝对值不可比）；待判缺口 ~1.0-1.3s 中位 / 单对噪声 ±1.5s——**1s 级真实效应在该载体上不可判**（n=2 批「不可判」、n=6 批 FAIL 1.072 均如实）。
- **判据（可复用）**：**灵敏度需求（预期效应量）与载体噪声下限同阶或更低时，MUST 换更灵敏载体再判**——boot Done 级全进程 wall 的噪声下限实测 ~2.6s 极差/批 + 2s 跨批漂移，缺口 <2s 的效应不可判；替代载体 = Chunky region 级计时 / 光照阶段计时（阶段口径，隔离 worldgen 主噪声）。判据线（如 1.05× 中位比）的可达性评估 MUST 先测载体噪声带（#51 前置噪声基线的**载体选型**形态）——否则进入「判据物理不可达 → C-gate 3 轮 halt」的浪费链（本例实证：B 期 1.07 / n=2 / n=6 1.07 三轮触发 C-gate，用户裁决收尾）。
- **家族索引**：#51（跨 run 噪声基线前置）/ #103（机器噪声带 ±10%）——本条为**载体灵敏度下限**判据；#128（串行 vs wall 口径不可换算——换载体时的口径声明依据）；build-tooling #150 判据 5（配对交错的必要性与本条互补：交错解决顺序效应，换载体解决灵敏度）。

---

## D. 12-lighting.md round3 小节草稿（主会话追加；含 L94 行尾 supersedes 双指针、L101 补强，按 judge §5 指示）

**L94 行尾追加（原文不改，supersedes 双指针）**：

```
〔260914-04 round3 supersedes 本行后半「→ Java 收集/JNI 侧开销已基本消除」及前半「≈内核份额预测吻合」归因：round2 减法口径把 Java 段记 ≈0.15ms/chunk，被 round3 分段探针实测证伪——collect 串行 4.5ms/chunk（30×）；见本篇 round3 小节 + .investigations/light-round3-260914-04/probe-verdict-260914-04.md §1〕
```

**L101 行尾追加（补强非推翻）**：

```
〔260914-04 补强：Java/JNI 侧收益已补单独微基准（round3 collect 4.527→0.288 / native 2.770→2.651ms，见 round3 小节）——本行降级声明由 round3 撤销〕
```

**新增小节（追加于文末）**：

```
---

## D3 性能优化 round3（palette 展开收集 + packed 直传）——ON 路径 7.3→2.94ms/chunk，e2e 判据 FAIL 1.07 如实收尾（260914-04）

> 状态：candidate 建议（judge 收尾 PASS：三源核对 8/8、行为门全绿、判据 FAIL 如实归档；confirmed 待用户拍板）。
> 口径声明（§9.7）：探针计时 = light 线程每 chunk 串行耗时；e2e = runServer boot pregen wall（~400-600 chunk 接管，gate `coreswap.light.rust`）；二者不可直接换算（#128）。跨批 e2e 绝对值不可比（整批漂移 ~2s 实测）。

### 探针定线（三源）与 round2 归因取代
- 源 K（内核）/ 源 T（Java 段计时）/ 源 P（palette 直方图：bits 4:95%、5:5%、6:0.4%，singular 与 ID_LIST(bits≥15) 均 0——1.20.1 实测样本内）。
- 对价模型：ON 串行 7.3 = collect 4.5 + native 2.8（内核 1.48 + JNI ~1.3）vs vanilla ~5.6；gap 1.73ms × 2025 ≈ 3.5s 双锚自洽。round2「Java 段≈0」减法口径被证伪（见 L94 取代标记）。

### 措施
- **B（Java palette 级展开收集）**：非空节走 `PalettedContainer.writePacket` 公有序列化帧（PC.java:383-387）→ Java 位流解码填 blocks9；mixin @Accessor 撞私有内部 record 不可行（→ compiler-idioms #27）。collect 4.527→1.257→0.288ms。
- **C（JNI packed 直传）**：Java 拆帧 → sectionMeta/paletteData/storage 直传 → Rust `light_decode_packed` 解码 → 同一 light_compute 内核（输出等价 = 同内核同输入，结构性承载）；游标式偏移免除法。native 2.770→2.651ms。
- 1.21.6 面：仅 Rust 增量导出（Java 零改动）；packed 收集未做（#26 帧格式差：定长无前缀），行为与 round2 一致（声明式，构建绿 + 语义零变化）。

### 验证链（行为门全绿）
golden 4/4 逐位（与 round2 冻结件一致）→ DUAL ALL-MATCH（B 输入层）→ DUALP ALL-MATCH（C 输出层）→ fallback=0（paldump 采样 0 节间接直证；无显式回退计数器——1.21.6 移植时建议补一行 counter）→ 诊断移除后复编 BUILD SUCCESSFUL。对拍抽样 = 4 chunk/轮（须含位流节，W1 教训 → build-tooling #151）；大样本正确性依赖 fallback=0 + 结构性论证，非全量逐位。

### e2e 判据：FAIL 1.072（如实）
判据线 ON/OFF 中位比 <1.05×（HOOK-2b，探针真值解除 judge「物理冲突」保留意见后维持）。B 期 1.070-1.083 FAIL / 四臂 n=2 不可判（OFF 极差 2.14s 与缺口同阶）/ 六对交错 n=6 中位比 **1.072 FAIL**（配对差均值 1.32s，单对噪声 ±1.5s）。**e2e 判据累计 3 轮未满足 → C-gate 触发，用户裁决「接受现状收尾」——判据未达标，非通过**。载体噪声下限（OFF 极差 2.6s/15% + 跨批漂移 2s）使 1s 级缺口在该载体不可判（→ workflow-patterns #148）。

### 净收与剩余
- ON 路径串行 7.3 → **2.94ms/chunk**（collect 0.29 + native 2.65）；e2e 回退 1.25× → **~1.07×**（收窄 ~2/3）。
- 剩余优化面（未实施）：解码-查表单趟融合（~0.6-0.8ms）+ A-② sky_fall 融合（~0.1-0.2ms）+ **e2e 载体更换**（Chunky region / 光照阶段计时）。
- 遗留风险：global palette（ID_LIST bits≥15）整 chunk 回退路径——1.20.1 实测 0 例但未证不可能；静默正确性由 rc=-2 闩回退结构保证，若发生为整 chunk 性能悬崖且生产无计数可见（建议下轮补回退计数日志）。双开关矩阵：`coreswap.light.oldcollect`（B 层）× `coreswap.light.blockabi`（C 层）正交——双开 = 全旧路径，任一关 = 该层走新路径。
- 过程错误 W1-W4 五段式 → record §5（W1 → build-tooling #151；W2 初值-谓词成对核对 #81 家族；W3 PowerShell 输出流即返回值；W4 回退计数=0 前提下读均值）。
```

---

## E. 10-timewise-archive.md 时间线条草稿（versions/1.20.1/docs/，追加 260914-04 块）

```
## 260914-04（实际 2026-09-14 15:29 起，Get-Date 锚：光照性能 round3——探针定线 + B palette 展开 + C packed 直传）🔍 candidate 建议（judge 收尾 PASS 8/8；e2e 判据 FAIL 1.072 如实、用户裁决收尾；confirmed 待拍板）

> 过程产物 `.investigations/light-round3-260914-04/`（record / probe-verdict / review-final-judge / scout-map / b1-b3 / cmd-output 23 件）+ 计划 `.investigations/000-架构设计/架构设计-260914-04-光照round3.md`。

- 🔍 **探针定线（三源）**：K 内核 1482.6µs（同数据 vs 12 篇 1587µs sanity 过）/ T Java 段 collect 4.5ms、native 2.8ms / P palette bits 直方图（4:95%）；**round2 减法口径「Java 段≈0」证伪 30×**（→ 12 篇 L94 supersedes + workflow #147 判据）。
- ✅ **B（palette 级展开收集）**：writePacket 公有帧路径（@Accessor 撞私有 record 不可行 → compiler #27）；collect 4.527→1.257→0.288ms；DUAL ALL-MATCH。
- ✅ **C（JNI packed 直传）**：三数组直传 + Rust light_decode_packed + 同内核；native 2.770→2.651ms（游标免除法）；DUALP ALL-MATCH、fallback=0（paldump 0 节直证）。
- ⚠️ **W1 漏读长度前缀假绿**（最高价值过程错误）：singular 节免疫位流错位 → 对拍 4 chunk 假绿 + rc=-2 静默回退 + nativeAvg 反升伪装（→ build-tooling #151 三件套签名 + 位流负载用例判据）；W2 初值自关闭 / W3 脚本返回值污染 / W4 回退均值前提（record §5 五段式）。
- ❌ **e2e 判据 FAIL 1.072（n=6 六对交错，如实）**：OFF 极差 2.6s/15% + 跨批漂移 2s → 1s 级缺口不可判（→ workflow #148 载体灵敏度判据）；3 轮未满足触发 C-gate，**HOOK-3 用户裁决接受现状收尾**（非判据通过）。
- ✅ **净收**：ON 串行 7.3→2.94ms/chunk；e2e 1.25×→~1.07×；golden 4/4 逐位。剩余：解码-查表融合 / sky_fall 融合 / e2e 载体更换 / global palette 回退计数。
- 📌 通用模式 → compiler-idioms #27、build-tooling #151、workflow-patterns #147/#148（subagent 草稿 + 主会话应用）。
```

---

## F. .artifacts/index.yaml 登记条目草稿（6 条；含 round2 standing 的 light 课题登记闭环）

```yaml
  # === light-round3（光照性能 round3：B palette 展开 + C packed 直传，260914-04，judge 收尾 PASS）===
  - id: 'swe:light-round3-260914-04:record'
    path: '../.investigations/light-round3-260914-04/record-260914-04.md'
    kind: record
    status: candidate   # judge 收尾 PASS（三源 8/8、判据 FAIL 如实）；confirmed 待用户
    # 260914-04：探针定线证伪 round2 减法口径（collect 4.5ms 30×）→ B writePacket 帧收集 4.527→0.288 +
    #   C packed 直传 native 2.770→2.651；ON 串行 7.3→2.94ms/chunk、e2e 1.25×→~1.07×；e2e 判据 FAIL 1.072
    #   （n=6）3 轮触发 C-gate、用户裁决收尾。W1 漏读前缀假绿（singular 免疫）。§9.7：串行 per-chunk vs e2e
    #   wall 不可换算；1.21.6 仅 Rust 增量（#26 帧格式差，Java 不动）。
  - id: 'swe:light-round3-260914-04:judge'
    path: '../.investigations/light-round3-260914-04/review-final-judge.md'
    kind: review
    status: candidate
    # 收尾 MUST 三源核对 8/8 PASS、diff 面一一对应无未声明改动；推荐 candidate。§5 给出 12 篇 L94 supersedes
    #   双指针指示；§7 遗留（global palette 回退未证不可能 / 双开关矩阵 / e2e 载体噪声）随知识库落盘。
  - id: 'swe:light-round3-260914-04:probe-verdict'
    path: '../.investigations/light-round3-260914-04/probe-verdict-260914-04.md'
    kind: analysis
    status: candidate
    # 探针三源（K/T/P）解读 + 对价模型（ON 7.3 = 4.5+2.8 vs vanilla ~5.6）；round2「Java 段≈0」证伪 30× 的
    #   一手锚；HOOK-2b 判据线维持依据。含 §9.7 先行声明。
  - id: 'swe:light-round3-260914-04:knowledge'
    path: '../.investigations/light-round3-260914-04/'
    kind: knowledge
    status: candidate   # subagent 草稿 + 主会话应用：compiler-idioms #27 + build-tooling #151 + workflow #147/#148
    #   + 12 篇 round3 小节（L94 supersedes）+ 10 时间线 260914-04 块 + INDEX 追加行。
  # —— round2 standing 闭环（review-d3-round2-260905-04 should-fix「light 课题 .artifacts 登记」随本块补齐）——
  - id: 'swe:light-opt-round2-260905-04:standing-closure'
    path: '../versions/1.20.1/docs/12-lighting.md'
    kind: note
    status: candidate   # 12 篇 round2/round3 小节 + 本组 light-round3 条目即 standing 登记闭环载体
    # round2 judge should-fix「light 课题 index.yaml 登记」由本块 light-round3 条目组补齐；round2 结论已被
    #   round3 部分取代（L94 supersedes 双指针，§15.4）。
```

---

## G. INDEX.md 追加行草稿（每发现一行，格式对齐现有追加块；主会话确认编号后应用）

```
> 260914-04 追加：光照 round3 收尾（judge 三源 8/8 PASS，e2e 判据 FAIL 1.072 如实、用户裁决收尾）。compiler-idioms 新增**发现 #27 简记**（writePacket 序列化帧 = 私有 PalettedContainer 的零 accessor 公有通道——@Accessor 撞私有内部 record 编译不可行时的替代路径，remap 安全，#26 帧格式版本差警示）；build-tooling 新增**发现 #151**（字节流协议解析漏读长度前缀判错签名三件套——解码值系统性撞常数 + 流尾残留字节 + 残留即 VarInt 本体；**「部分成功掩盖全流错位」**——singular/免位流路径免疫 → 对拍抽样假绿，判据 = 协议解析验证用例 MUST 含位流负载路径，#48 覆盖面盲区形态）；workflow-patterns 新增**发现 #147**（减法归因口径作优化排序依据前 MUST 分段直接计时——round2「Java 段≈0」被探针实测 30×，#134/#102 家族授权门槛形态）+ **发现 #148**（boot Done 载体噪声下限——OFF 极差 15% + 跨批漂移 2s，缺口 <2s 不可判；灵敏度需求低于载体噪声带 MUST 换载体，#51/#103 家族载体选型形态）。来源：.investigations/light-round3-260914-04/（12 篇 round3 小节 + L94 supersedes + 10 时间线 260914-04 块同步落盘）。
```

---

## 自检清单（交付核对）

- [x] 先读 SUBAGENT-KNOWLEDGE-GUIDE（价值门 + 五段式 + 载体映射）
- [x] A=中价值简记（惯用法「是什么+怎么用」）；B=高价值五段式（现象/根因/定位/修复/教训全）；C 两条=高价值判据（MUST 句式 + 家族索引）
- [x] 一次性数字（7.3→2.94、1.072 等净收值）不进 discovered，只进 12 篇/时间线/record 引用
- [x] B 条含定位链（合成帧单测→分层 rc→帧 dump）与假绿掩盖面机制层根因
- [x] D 按 judge §5 指示：L94 行尾双指针原文不改、L101 补强、round3 小节含 FAIL 如实 + 用户裁决措辞（防误读「判据通过」）+ 双开关矩阵一行 + global palette 风险
- [x] F ≤6 条且含 round2 standing 闭环；G 每发现一行
- [x] 编号为主会话暂拟，应用前须核对目标文件末条（workflow-patterns 260914-02 后可能有更高号）
- [x] 所有数字来自 record / judge 意见 / probe-verdict，无编造
