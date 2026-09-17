# discovered/ 追加草稿（260917-03）——subagent 产出，主会话应用

> 应用说明：① 第一节追加到 `knowledge/discovered/build-tooling.md` 末尾（#42 家族补充案例）；② 第二节追加到 `knowledge/discovered/workflow-patterns.md` 末尾（新发现 #160）；③ 第三节追加到 `knowledge/discovered/workflow-patterns.md`（#13 家族补充案例，置于 #160 之后）。发现编号衔接：workflow-patterns 现末号 #159 → 本块新号 #160；build-tooling 不新增编号（#42 补充案例形态，同 #8/#118 家族补充案例先例）。

---

## 一、追加到 knowledge/discovered/build-tooling.md 末尾

### #42 家族补充案例（260917-03）：DSH 沙箱 TEMP 重定向 → JVM 内资源提取拒访 → lightInit threw → 整臂静默 vanilla fallback——JNA tmpdir 家族的「JVM 侧全量资源提取」新形态

- **发现时间/置信度/module**：260917-03；candidate；build-tooling / gradle·JVM tmpdir 沙箱坑（#42 简记的沙箱会话重形态）。
- **来源定位**：`.investigations/k2-noise-anchor-260917-03/k2-noise-anchor-errors.md` E2 + judge-review §D2（逐行核对 VOID1.log:222/229/240）；判别臂 = K2-D4 首轮 SELFCERT `lightInit_ok=0, fallback=1, hook=0, sha=none`（#118 硬门 VOID）。
- **五段式（错误优先）**：
  - **现象**：K2-D4 SELFCERT 自证全空、GATE VOID；changed=81/2025=4.00% 恰在 legacy 量级，极易被当有效臂；服务端随后 `Encountered an unexpected exception` 崩溃。接管失败是**静默的**（只有一行 `fallback vanilla x1` + 崩溃栈）。
  - **根因（机制）**：DSH 沙箱把 TEMP/TMP 重定向到宿主 `...\Temp\dsh-jks3Sq`，JVM 内 `CoreSwapFixHelper.extractWorldgenDir` 建目录抛 `AccessDeniedException` → `ExceptionInInitializerError` → `wgLightEnsureInit` 捕获 → `lightInitFailed` **永久 fallback vanilla**；同轮 JNA `jnidispatch.dll` 提取同样被拒。与 #42（JNA 单库 tmpdir 拒访）同族但形态更重：不止 JNA，**JVM 内全部解压/建目录类初始化**都撞沙箱 TEMP，且失败被 catch 吞成静默 fallback——产出的是 vanilla 形态数据冒充接管臂。
  - **定位**：grep 日志 `LightRust|fallback` → `lightInit threw` 行 → 堆栈 `AccessDeniedException: ...Temp\dsh-*\coreswap-data`。#118 SELFCERT 硬门在采集完成时即打 VOID，无效结论未外泄（硬门按设计工作——**正面案例**：自证行缺「接管形态」证据时 changed 数会误导，4% 恰在 G17b legacy 量级）。
  - **修复**：脚本内固化 `TEMP/TMP` + `JAVA_TOOL_OPTIONS=-Djava.io.tmpdir=` 指向工作区 `.tmp/<课题>/jtmp`，并先 `gradle --stop`（防旧 daemon 复用吞 env，#32 成对处理）。修复后三连 boot 全 PASS。
  - **教训（判据）**：① **沙箱会话跑 runServer 必须固化 tmpdir 到工作区**（TEMP/TMP 与 java.io.tmpdir 双通道），判别签名 = `AccessDeniedException` 指向 `Temp\dsh-*` 目录；② 自证硬门必须含「接管形态」三件（lightInit/hook/sha），单看 changed 数会误读；③ daemon env 残留与客户端 env 修正**成对**处理（#32）。
- **家族索引**：#42（JNA tmpdir——本条为其 JVM 全量资源提取的沙箱重形态）；#32（daemon 吞 env——修复的成对面）；#118（SELFCERT 硬门——本条正面案例：VOID 在采集完成时被拦，未外泄 vanilla 形态数据）；workflow-patterns #156（形态错位家族——本条为环境侧成因）。

---

## 二、追加到 knowledge/discovered/workflow-patterns.md 末尾（新发现 #160）

## 发现 #160（错误优先）: VOID/GATE 裁决必须非零退出断链——裁决只打印不退出，下一 boot 以 VOID 产物为 prev 启动（260917-03，E3）

- **发现时间/发现者/置信度/module**：260917-03；主会话采集侧（E3 五段式已落台账）+ knowledge subagent 草稿；candidate；workflow-patterns / 采集链路设计（#118 自证硬门家族的「裁决执行」面 + #144/#146 复跑覆盖家族）。
- **来源定位**：`.investigations/k2-noise-anchor-260917-03/k2-noise-anchor-errors.md` E3 + record §1 表 K2-D5-VOID2-partial 行；修复实例 = run_k2.py:112-115（GATE 不过即 `sys.exit(4)` 断链，judge §B4 已核对回写）。
- **五段式（错误优先）**：
  - **现象**：boot1 判 VOID 后链未断，boot2 已带上一轮启动；kill 中断时留下**无 [SELFCERT]/[GATE] 行的半截日志**（归档为 `K2-D5-VOID2-partial.log`，#144/#146 换标签纪律）。
  - **根因（机制）**：采集链是「boot → 判 → 下一 boot 以本轮产物为 prev」结构；GATE 只打印 VOID 不 `exit` 非零 → 链式脚本继续执行下一 boot，**VOID 轮的污染产物（vanilla 形态 81 chunk）成为下一轮对比 prev**——即使下一轮被人工叫停，也已产生不可作数的半截日志与 world 状态。
  - **定位**：日志缺 `[SELFCERT]`/`[GATE]` 行 + java 进程清单核对发现 boot2 在跑（kill 发现延迟是链未断的直接后果）。
  - **修复**：GATE 不过即非零退出（VOID = 断链语义）；脚本已回写 `sys.exit(4)`；半截日志换标签归档不计入。
  - **教训（判据）**：**裁决的价值在执行不在打印**——VOID/GATE 类裁决 MUST 以进程退出码表达（非零即断链），使「裁决 → 下一轮是否启动」由结构保证而非人工盯守；任何「打印了但流程继续」的裁决 = 裁决未生效。
- **家族索引**：#118（自证硬门——本条补「硬门触发后的执行后果」维：VOID 必须断链）；#144/#146（复跑覆盖/换标签——本条为其上游成因：链未断才产生待归档的半截轮）；#145（驱动未生效签名——「无裁决行」同属装置未生效签名族）。

---

## 三、追加到 knowledge/discovered/workflow-patterns.md（#13 家族补充案例）

### #13 家族补充案例（260917-03）：零成本日志探针——引用「日志零命中 = 计数为零」前，先核打点节奏覆盖首次发生（P-α 方法）

- **发现时间/置信度/module**：260917-03；candidate；workflow-patterns / 探针零输出判读（#13 家族：探针零输出先查过滤/驱动条件 + 行首锚 grep 假零输出的判读前置维）。
- **来源定位**：`.investigations/k2-noise-anchor-260917-03/p-alpha-result-260917-03.md` + judge-review §A1（打点节奏一手核对成立：`ServerLightingProviderMixin.java:149-159`，`n <= 8 || (n % 256) == 0` 无条件 println，且不受 FormProbe.ON 门控）；应用实例 = P-α 探针（fallback 计数 0 → M-a'-α 排除）。
- **方法（可复用）**：用「日志零命中」作「事件计数为零」的证据前，MUST 核对三件：① **打点节奏覆盖首次发生**——`n<=8 无条件打印`类首发不受节流盲区（若节奏是 `n%256==0` 则首次发生可整段静默，零行 ≠ 零计数）；② **该打点不受其他开关门控**（门控关闭时零行只证门关）；③ **通道覆盖**（stdout/stderr 均并入日志，如采集脚本 `stderr=subprocess.STDOUT`）——三件齐，零行 = 零计数才成立（G17 系采集脚本同参数核对为 judge SHOULD 补档项 A5）。本案 P-α 即凭此以**零新 run 成本**排除 M-a'-α 通道并连带证伪 M-b。
- **家族索引**：#13（探针零输出先查过滤/驱动条件 + grep 假零输出——本条为其「先核打点节奏再引用零命中」判读前置维）；#84（日志级别假阴性——同属「零命中前核打点条件」家族，本条补节流/门控/通道三维）。
