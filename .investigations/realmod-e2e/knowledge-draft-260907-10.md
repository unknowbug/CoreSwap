# knowledge-draft-260907-10 —— Forge 生产 e2e（260907-10）知识库更新草稿

> subagent 产出（core.worker 知识库角色），主会话应用 + 验证。只含过记录价值门的条目。
> 来源产物：`.artifacts/forge-prod-e2e-260907-10.md`（candidate）；日志 `.tmp/forge-prod-v1b-260907-10-boot.log`（PASS 轮）/ `.tmp/forge-prod-v1-260907-10-boot.log`（FAIL 轮对照）。
> 编号核对：build-tooling.md 现至 **#31**，workflow-patterns 现至 **#81**（经 INDEX 260907-09 行）。

---

## 发现 #32 候选：生产/开发口径参数缺失第二实例——`-Dcpp.blockRegister` 门控生产侧无携带，注册链下半断 → 惰性解析 miss 全 AIR（260907-10）

- **建议载体文件**：`knowledge/discovered/build-tooling.md`（追加于发现 #31 之后）
- **时间/置信度/module**：260907-10（2026-09-07 22:15-22:30，Get-Date 锚定）；candidate；build-tooling / 生产口径参数清单（**#28「冒烟口径 ≠ 存档口径」家族第二实例**；#8「-P→-D 映射遗漏」家族跨口径形态）
- **来源定位**：`.artifacts/forge-prod-e2e-260907-10.md` §3；CppBridge.java:168（sysprop 门控默认关）

### 现象

V1 三轮对照：轮1/轮2（`CORESWAP_DEFAULT_BLOCK=testcontent:test_brick` 已设）env override 行为化日志三维命中（`[WGH] env override ... (was minecraft:stone/netherrack/end_stone)`），但 region 扫描 test_brick 恒 **0**；boot 日志 `[BLOCKS] unknown block 'testcontent:test_brick' -> AIR (register via wg_register_block)` ×3 维度，且**全日志无任何 `[BLOCKS-REG]` 行**。轮3（user_jvm_args.txt 追加 `-Dcpp.blockRegister=1`）后 test_brick = 672 命中 / 144 sections，`[BLOCKS-REG] testcontent:test_brick java_raw=1003 rust_id(三域 1003) writeback=...` 在场。

### 根因（为什么错）

`CppBridge.registerModBlocks()` 被 `-Dcpp.blockRegister` sysprop 门控，**默认关**。dev 口径 260907-09 经 gradle `-PblockRegister`→`-D` vmArg 映射自动带入，门开；生产 run.bat 裸 java 口径无该映射环节 → mod 方块从未注册进 Rust registry → #79 惰性解析按名 miss → default_block 整片 AIR。机制层面：**注册是链路独立的下半段**——env override（上半段，解析分支选择）生效 ≠ 方块名可解析（下半段，注册表内容）；两段由不同参数分别门控，只验上半段会误判「链路已通」。

### 定位（怎么发现的）

判据签名 = **两段行为化日志组合判读**：① 上半段 `[WGH] env override` 已命中（分支活着）；② 下半段 `[BLOCKS-REG]` 行缺席 + `[BLOCKS] unknown block 'X' -> AIR (register via wg_register_block)` ×维度数（惰性解析 miss 直证）。只有①无②=「分支开但注册表空」，一轮锁定门控参数缺携带，不查解析实现。注意细节：PASS 轮 boot 早期（世界创建时）也会出现 unknown 行（注册发生在其后），unknown 行**在场不作判别**，`[BLOCKS-REG]` 有无才是判别面。

### 修复

`runtime/forge-server/user_jvm_args.txt` 追加 `-Dcpp.blockRegister=1`（git 外，runtime/ 整树 gitignore）。**生产口径必带项清单 +1**。注：该值 `-Dcpp.blockRegister=1` 的 1 同时解析为注册 limit=1——只注册首个 mod 块（test_lamp 因此恒 0，预期行为）；多 mod 块验证时 limit 须调大。

### 教训/判据

1. **#28 家族泛化**：跨口径（dev gradle vs 生产裸 java）参数清单**按口径分组维护**，新门控参数（sysprop/env/`-P` 均同）加入时 MUST 同步登记**每个口径**的携带方式——dev 自动带入 ≠ 生产自动带入，映射环节本身是口径差异点（#8 家族的 gradle 侧映射遗漏，在生产侧天然复现）。
2. **注册类链路的完整判据 = 上下两段行为化日志组合**：「单段行为化日志命中 ≠ 全链通」——每个参数门控的段都要有自己的在场/缺席证据（#37 家族延伸）。
3. 家族索引：#28（口径反转第一实例）、#8/#19/#25（映射清单遗漏静默不生效）、#79（惰性解析——本条是其 miss 的上游成因）、#81（行为化日志判据）。
- **证据**：`.tmp/forge-prod-v1-260907-10-boot.log`（FAIL 轮：override 行在场 + 零 BLOCKS-REG）vs `.tmp/forge-prod-v1b-260907-10-boot.log`（PASS 轮：L109/198/214 override + L221-223 BLOCKS-REG）；region 扫描 0 vs 672。

**INDEX 追加行建议**（追加到 INDEX.md 末尾追加区块）：

> 260907-10 追加：build-tooling 新增**发现 #32**（生产/开发口径参数缺失第二实例——`-Dcpp.blockRegister` 门控 dev 经 gradle -P→-D 映射自动带入、生产裸 java 无携带，mod 方块未注册→惰性解析 miss 全 AIR；判据 = env override 行为化日志命中 + [BLOCKS-REG] 缺席 + unknown→AIR ×维度数组合判读，「单段行为化日志命中 ≠ 全链通」，#28 家族第二实例；修复 = user_jvm_args.txt 追加参数，生产口径必带清单 +1；来源：.artifacts/forge-prod-e2e-260907-10.md）。

---

## 发现 #31 补充案例（简记）：remapped jar 进 Forge+Connector 生产 mods——第三轮复用确认（260907-10）

- **建议载体文件**：`knowledge/discovered/build-tooling.md`（追加于发现 #31 之后，作为其补充案例小节）
- **时间/置信度/module**：260907-10；candidate；build-tooling / loom jar 载体选择（#31 复用记录）
- **正文（观察/如何利用）**：#31「dev jar 不可进生产，须 loom remapJar 产物」判据第三轮成功复用：主线 `coreswap-1.0.26.jar` 与 content-test `content-test-1.0.0.jar` 均以 remapJar 产物进 `runtime/forge-server/mods/`（SRG 载体，boot 日志 `server-...-srg.jar` + mod 自证打印在位），post-Done forceload 观察判据（#80）同轮复用成立（forceload 在 `Done (13.928s)` 之后触发，观察对象 = post-Done 新区块）。观察载体延续：region palette 定点扫描脚本（`.tmp/scan-r11-260907-10.py`）。
- **证据**：`.artifacts/forge-prod-e2e-260907-10.md` §1/§2 V1。

**INDEX 追加行建议**：并入 #32 那条 INDEX 行末尾（同批追加）：「+ build-tooling **#31 补充案例**（remapped jar 进 Forge+Connector 生产 + post-Done forceload 观察判据第三轮复用确认）。」

---

## #33 补充案例（简记）：diag bin 直读数据目录载体不经 Java remap/SRG 管线 → 生产复跑无判别力，§9.7 声明收口（260907-10）

- **建议载体文件**：`knowledge/discovered/workflow-patterns.md`（作为发现 #33「载具可比性」的补充案例追加）
- **时间/置信度/module**：260907-10；candidate；workflow-patterns / 载具可比性（#33 家族）
- **正文（观察/如何利用）**：V2（mod namespace settings AGG 恒等）生产面不重跑、以 §9.7 声明收口——机制：diag bin（diag_modns）是 **Rust 侧直读数据目录**载体，整个验证路径不经 Java remap/SRG 管线，生产（SRG 命名）与 dev（named）对该载具是同一执行语义 → 生产复跑无判别力（不是「判别力弱」，是结构为零）。V3（biome override 分支）同理：该分支属 Rust+数据层，无 SRG 暴露面，生产侧生产数据目录（mod jar 解压）本就不含 testcontent 数据 → vivo 无触发场景。**判据**：动生产复跑前先问「该验证路径是否经过生产特有层（remap/SRG/生产数据目录）」——不经 = 复跑零信息，声明收口 + dev 证据维持，省整轮。#33「跨载具残差禁止互引」的生产侧对偶面：**载具不经生产层 = 生产复跑无意义**。
- **证据**：`.artifacts/forge-prod-e2e-260907-10.md` §2 V2/V3。

**INDEX 追加行建议**：并入同批 INDEX 行：「+ workflow-patterns **#33 补充案例**（diag bin 直读数据目录不经 remap/SRG 管线 → 生产复跑判别力结构为零，V2/V3 §9.7 声明收口；判据 = 验证路径不过生产特有层则不重跑）。」

---

## 自检（按 SUBAGENT-KNOWLEDGE-GUIDE §四）

- [x] 价值门：条1 高价值（错误链 + 判据签名 + 口径坑）；条2/条3 中价值简记（载体复用确认 / 载具可比性补充案例）；无低价值条目
- [x] 条1 五段式齐全，根因为机制层（注册下半段独立门控，非现象复述），定位含可复用判别签名
- [x] 数字来自实测记录（672/144、672 vs 0、java_raw=1003、Done 13.928s），无编造
- [x] 编号核对：build-tooling 现至 #31 → 新条 #32；INDEX 行建议同批合并交付
- [x] 被排除/边界事实保留：unknown 行 PASS 轮亦在场不作判别、limit=1 副作用、test_lamp 恒 0 预期
