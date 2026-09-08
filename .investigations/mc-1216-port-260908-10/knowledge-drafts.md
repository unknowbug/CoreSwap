# 知识库草稿 — 260908-10（MC 1.21.6 移植 Phase 3 后半）

> **性质**：subagent 产出**草稿**，主会话负责应用到 `knowledge/discovered/` 并同步 `knowledge/INDEX.md`。本文件本身不是知识库载体（过程性产物，留在 `.investigations/`）。
> **产出依据**：`knowledge/SUBAGENT-KNOWLEDGE-GUIDE.md`（项目级错误记录规范）+ `knowledge/INDEX.md` + 三个目标文件末尾现有条目格式。
> **编号承接（读文件实测）**：build-tooling 现有最大 **#39**；workflow-patterns 现有最大 **#86**；compiler-idioms 现有最大 **#22**。
> **证据纪律**：下列所有数字/路径均经本 subagent 用 read/grep/glob 逐条核对（未执行任何 shell 命令）；引用的日志行号为本轮实读。

## 候选 → 载体/编号总表

| # | 候选 | 目标文件 | 建议编号 | 价值门 | 置信度 |
|---|---|---|---|---|---|
| 1 | 跨版本枚举新增常量静默落 catch-all | `workflow-patterns.md` | **#87（新）** | **高**（详写） | candidate |
| 2 | mixins.json 与 mixin 类目录失同步 | `build-tooling.md` | **#40（新）** | **高**（详写） | candidate |
| 3 | mixin AP 警告 = 运行时 APPLY FAILED 前兆 | `build-tooling.md` | **#25 补充案例**（不新增编号） | **高**（详写） | candidate |
| 4 | 跨版本工程漏整块 -P→-D 映射 | `build-tooling.md` | **#8 补充案例**（不新增编号） | **高**（详写） | candidate |
| 5 | debug 栈溢出 ≠ 移植缺陷（对照臂） | `workflow-patterns.md` | **#16 补充案例**（不新增编号） | 高（详写，机制单一故篇幅中等） | candidate |
| 6 | dump 副产品文件名 seed ≠ 世界 seed | `build-tooling.md` | **#14 补充案例**（不新增编号） | **高**（详写） | candidate |
| 7 | `Select-Object -First N` 杀上游进程 | `build-tooling.md` | **#41（新）** | **高**（详写） | candidate |

**compiler-idioms**：本轮 7 个候选无一归属该载体（无编译器惯用法/语义指纹类内容）——不做改动。

---

## 条目 1（→ `knowledge/discovered/workflow-patterns.md`，建议 **发现 #87**，高价值，candidate）

```markdown
## 发现 #87（最高价值）: 跨版本枚举「新增常量」静默落 catch-all——Java 枚举类必须逐版本 diff 常量集合，映射函数禁以「无操作」兜底（260908-10）

- **时间/置信度/module**：260908-10；candidate（行为级双臂对拍闭环 99.8114%→99.9993% + 一手枚举 diff + 全数据集唯一使用者实锤）；workflow-patterns / 跨版本数据驱动移植（**#56「写了但没生效」家族的数据驱动形态**）。
- **来源定位**：`.investigations/mc-1216-port-260908-10/progress-260908-10.md` §P7 根因定位 2-4；`versions/1.21.6/data/mc_src_extract/net/minecraft/world/gen/StructureTerrainAdaptation.java:7-11` vs 1.20.1 同文件 L5-9；`worldgen-core/src/beardifier.rs`（修前 ordinal 映射 `_ => None`；修后 L19-25 枚举 / L210-211 映射）；`.tmp/p7-diff-result-260908-10.txt` vs `.tmp/p7-diff-rust2-260908-10.txt`。
- **现象**：MC 1.21.6 给 `StructureTerrainAdaptation` 新增 `ENCAPSULATE("encapsulate")`（序数 4），1.20.1 只到 `BEARD_BOX`(3)。Rust `beardifier.rs` 的 ordinal→语义映射旧实现写 `_ => None` → 新常量**静默归零**：trial_chambers 结构处密度局部偏低 ≈1.4-2.2，整簇 **11,865 格错位**（64 chunk 区域 6,291,456 格，占 0.19%；27 chunk 有差，前 10 个占 87%）。**编译绿 + 单测全绿**——旧常量 0..3 的覆盖全部通过，缺口不在测试集里。修复后同 seed/同区域重跑 R 臂：**41/6,291,456 = 99.9993%**，原 trial chamber 簇完全消失（残差降为无簇零散近地表点）。
- **根因（机制层面）**：数据驱动移植里，Java 枚举的常量集合本质是**版本数据**，但 Rust 侧把它固化成了「match arm 集合 + catch-all」。catch-all 的语义被选成「无操作」（None / 权重 0）——新常量进入后，行为上等于「结构密度修正被悄悄关掉」，全程无报错、无日志、无告警。这是 #56 的**数据驱动形态**：不是新 arm 落在 catch-all 之后（序问题），而是**新数据值**落进 catch-all 的盲区（集合覆盖问题）——旧 arm 集合覆盖不到新值，且「编译过 + 告警不新增」同样成立。
- **定位（怎么发现的）**：① 双臂 BlockProbe 残差签名 = 簇状 `deepslate→air/water`，机制约束先排除 carver/feature（R 臂 `stageMask=3`，Java carver 两臂同码同 seed）；② 预置 fan-out 双否（b1「1.21.6 AquiferSampler 版本语义变化」REFUTES / b2「Rust aquifer 实现偏离」REFUTES）后回数据层；③ 世界 NBT 直读残差簇 = `minecraft:trial_chambers`（chunk(11,16)，249 children）；④ **一手枚举 diff** 抓到 `ENCAPSULATE`（序数 4）+ 全数据集唯一使用者 `trial_chambers.json`（`"terrain_adaptation": "encapsulate"`，该文件 L145）；⑤ 同点双 dump 68/68 density 符号翻转（vanilla +0.004..+2.215 → stone；Rust −0.0002..−0.05 → 进 aquifer）直证「结构权重归零 → density 被压负」。
- **修复**：`beardifier.rs` 补 `Encapsulate = 4` + q 分支（`min_y/max_y`，不含 `groundLevelDelta`）+ 权重分支 `getMagnitudeWeight(m/2,q/2,n/2)*0.8` + beard file ordinal 4 映射；`get_magnitude_weight` 改 f64 入参（Java 三种调用形态）。语义权威 = 1.21.6 `StructureWeightSampler.java:99/106`（一手实读：L99 `case ENCAPSULATE -> Math.max(0, Math.max(blockBox.getMinY() - j, j - blockBox.getMaxY()))`；L106 `case ENCAPSULATE -> getMagnitudeWeight(m / 2.0, q / 2.0, n / 2.0) * 0.8`）。
- **判据/教训（MUST）**：
  1. **跨版本移植时，Java 枚举类必须逐版本 diff 常量集合**（名字 + 序数 + `asString` 值），把「新增项」当 parity 缺口清单逐项核——枚举/常量表/注册表的新增条目是跨版本缺口的最高发面。
  2. **映射函数的 catch-all 禁止兜底成「无操作」语义**（None/0/忽略）——未识别序数 MUST 告警/断言/panic，把静默变成可观测；若必须兜底，兜底分支必须打一次性日志（带序数与来源）。本条修复后 catch-all 仍在（`_ => TerrainAdaptation::None`，L211），遗留风险已诚实记录，建议加告警。
  3. **「编译绿 + 单测绿」在跨版本枚举面上不构成证据**——旧常量覆盖全过是结构性保证，不是新常量被覆盖的证据。
  4. **数据驱动的「数据」不止 JSON**——代码里硬编码的枚举/常量表同样是版本数据，升级时一并 diff。
  5. 这类缺口的观测签名 = **结构/特性局部区域成簇偏差**（而非随机散点）——见簇先怀疑「某类结构/特性的修正在新版本常量上被静默关掉」。
- **证据**：上述源码/日志路径；`.tmp/p7-diff-result-260908-10.txt`（99.8114%、11865 mismatch、TOP PAIRS deepslate→air 6370 / deepslate→water 3744）；`.tmp/p7-diff-rust2-260908-10.txt`（99.9993%、41 mismatch）；`worldgen-core/src/beardifier.rs` L11-25 头注 + L255-284 三条单测。
```

---

## 条目 2（→ `knowledge/discovered/build-tooling.md`，建议 **发现 #40**，高价值，candidate）

```markdown
## 发现 #40: mixin 配置文件与 mixin 类文件失同步——`required=true` 下编译期零提示、运行时装载失败（「编译绿 ≠ apply 绿」第二形态，260908-10）

- **时间/置信度/module**：260908-10；candidate（脚本集合对拍 + 1.20.1 侧 23/23 全等反证；修后 22/22 实测）；build-tooling / mixin 工程（**#25/#55 mixin 家族，「编译绿 ≠ apply 绿」第二形态**）。
- **来源定位**：`.investigations/mc-1216-port-260908-10/progress-260908-10.md` §T0-1；`runtime/1.21.6/java/src/main/resources/coreswap.mixins.json`（修前 23 项 vs 目录 22 个类）；`runtime/1.20.1/java/src/main/resources/coreswap.mixins.json` 23 项 vs 目录 23 个类（反证）。
- **现象**：1.21.6 侧 `coreswap.mixins.json` 的 `mixins` 数组列 23 项，实际 mixin 类文件 22 个——P4 迁移随 light ticket 删除了 `ThreadedAnvilChunkStorageAccessor`，json 未同步。`"required": true` 下缺类 = mixin 装载失败 = 服务器起不来；而 `gradle compileJava` 与单测**全绿**（json 不是 javac 的编译输入，无任何提示）。
- **根因**：mixin 的配置清单（json 数组）与类文件目录是**两份独立维护的事实**，构建链没有任何一致性校验。删/改/重命名 mixin 类时只改代码不改 json（或反之）都不报错——缺类只在**运行时 mixin 装载阶段**暴露，`required=true` 把「清单指向不存在的类」升级为硬失败。
- **定位**：4 行脚本做**集合对拍**——json 数组项 vs `mixin/` 目录 `*.java` 文件名，双向差集为空才算对齐；再用 1.20.1 侧 23/23 全等作反证，排除「脚本误报」。
- **修复/判据**：修复 = json 移除 stale 条目（修后 22 项 vs 22 类对齐）。判据（MUST）：**mixin 类增删后 MUST 脚本对拍 `mixins.json` 列表 vs `mixin/` 目录文件名集合**，并把它做成构建前置门禁；「编译绿」对 mixin 装载零判别力。
- **教训**：「编译绿 ≠ apply 绿」现有两形态：① 描述符/目标签名失配（#25/#55 家族，AP 警告或生产 APPLY FAILED）；② **清单与类文件失同步**（本条，`required=true` 运行时装载失败）。共同点 = **mixin 的有效性不在 javac 的检查域内**，必须用 mixin 自己的三重门禁兜底：json↔目录对拍 + AP `Cannot find target method` grep + 生产 apply 日志。
- **证据**：progress §T0-1；两个 json 与两个目录的集合对拍记录（本轮实读：1.21.6 json 22 项 / 目录 22 文件；1.20.1 json 23 项 / 目录 23 文件）。
```

---

## 条目 3（→ `knowledge/discovered/build-tooling.md`，建议 **#25 补充案例**，高价值，candidate）

> 说明：本条与 **#25（mixin 门控/注入 build 层）** 及 **#55 判据 4（refmap 错误只在生产 APPLY FAILED）** 重叠，按项目规则写补充案例、不新增编号。

```markdown
## 发现 #25 补充案例（260908-10）：mixin AP 的 `Cannot find target method` 警告 = 运行时 APPLY FAILED 的可预测前兆——`defaultRequire=1` 下该串必须当错误

- **时间/置信度/module**：260908-10；candidate（AP 警告计数修前 2 / 修后 0 + 重编译 BUILD SUCCESSFUL）；build-tooling（#25 mixin 家族；**#55 判据 4「签名/refmap 错误只在生产 APPLY FAILED」的编译期可见形态**）。
- **来源定位**：`.investigations/mc-1216-port-260908-10/progress-260908-10.md` §T0-3；`.tmp/p2b-compile-260908-10.log:390/393`（修前 2 条）；`.tmp/p2b-compile-r3-260908-10.log:25`（修后 `BUILD SUCCESSFUL in 8s`，0 条）；一手签名权威 `versions/1.21.6/data/mc_src_extract/net/minecraft/world/gen/chunk/NoiseChunkGenerator.java:326`。
- **现象**：`gradle compileJava` 通过，但 mixin 注解处理器报 2 条 `警告: Cannot find target method "populateNoise(Ljava/util/concurrent/Executor;...)..."`（`NoiseChunkGeneratorMixin.java:63` / `NoiseDumpProbeMixin.java:46`）——1.21.6 去掉了 `populateNoise` 的首参 `Executor`（1.20.1 五参 → 1.21.6 四参）。`injectors.defaultRequire=1` 下该 mixin 运行时必 APPLY FAILED。修描述符 + handler 形参后重编译，警告计数 **2 → 0**。
- **根因**：mixin AP 在编译期解析 `@Inject.method` 描述符与目标类，找不到只发 **warning**（javac 层面构建成功），而 mixin 的 `defaultRequire=1` 把「目标方法缺失」升级为**运行时硬失败**——**编译器的告警等级与 mixin 的运行时严格等级不一致**，中间没有门禁把它们对齐。
- **定位**：编译日志 grep `Cannot find target method`（修前 2 / 修后 0）；目标方法签名以一手 `NoiseChunkGenerator.java` 为准逐参核对，不靠记忆/上一版本。
- **修复/判据**：修复 = 两处 `@Inject` 描述符 + handler 形参同步去 `Executor`。判据（MUST）：**mixin 项目构建日志中的 `Cannot find target method` 必须当错误处理**——构建脚本/CI 应 grep 该串做门禁，出现即 fail；跨版本升级后 mixin 目标方法签名 MUST 以一手源码逐方法核。
- **教训**：「compileJava 成功」在 mixin 项目里只证明 Java 语法/类型，不证明注入有效；AP 警告是**免费的前置信号**，漏读即把编译期可发现的问题推到运行时（#25 家族共同结论：门控/注入配置层静默不生效，只有显式门禁兜底）。
- **证据**：上述两个编译日志 + 一手源码行。
```

---

## 条目 4（→ `knowledge/discovered/build-tooling.md`，建议 **#8 补充案例**，高价值，candidate）

> 说明：本条与 **#8/#19（-P→-D 映射遗漏）** 同族，但机制从「漏一行」升级为「漏整块」——按项目规则写补充案例、不新增编号。

```markdown
## 发现 #8 补充案例（260908-10）：跨版本新探针工程首建漏掉**整块** `-P`→`-D` 映射——参数静默不生效，「编译过」不构成接线证据（#8/#19 家族第四形态）

- **时间/置信度/module**：260908-10；candidate（两侧 build.gradle 逐项对照 + 修复后 P2b 三跑参数行为化生效）；build-tooling（#8 家族：从「漏一行」升级为「漏整块」）。
- **来源定位**：`.investigations/mc-1216-port-260908-10/progress-260908-10.md` §T0-2 + §P2b 三跑表；`runtime/1.21.6/java/build.gradle`（修前只有 `processResources` + `loom.runs.server`，零映射）vs `runtime/1.20.1/java/build.gradle:56-181`（`benchVmArgs` 映射块，109 处 `findProperty`，约 126 行）。
- **现象**：1.21.6 探针工程首建时 `build.gradle` 未移植 1.20.1 的 `benchVmArgs` 映射块 → `-PbiomeProbe=true` / `-PblockProbe=true` 等参数**静默不生效**（探针不跑、无报错）。修复 = 整块移植（bench 参数 / 各探针 / `cpp.replace|cpp.lib|cpp.worldgen.dir` / `rustStages` / `cpp.blockRegister` / `featureLog`+`defaultBlock` env 通道），并把 `benchOut` 默认值改指 1.21.6 数据目录；修后 P2b 三跑（biome `biomes=7593` / block `DONE` / nether `biomes=5`）参数全部行为化生效。
- **根因**：gradle `-P`（项目属性）与 JVM `-D`（系统属性）是**两个命名域**，桥接靠 `build.gradle` 手工 `findProperty → vmArg` 逐行映射（#8 原始机制）。新版本工程从零起 `build.gradle` 时，映射块**不在编译依赖里、也不在任何模板里**——漏掉整块没有任何编译/运行报错，只是参数进不了 JVM。
- **定位**：两侧 `build.gradle` 逐项对照（`findProperty` 计数 + 属性名集合），而不是「跑一下看有没有输出」（空跑也会走默认行为，看起来正常）。
- **修复/判据**：判据（MUST）：**新版本工程首建时，`-P` 参数清单必须逐项对照上一版本复制，并以行为化日志（探针 banner / 属性回显）核验生效**——「编译过 / 构建成功」不构成接线证据；映射块建议做成可复用片段或前缀批量映射（#8 结构性修法，仍未落地）。
- **教训**：参数传递链上的静默丢弃只有**清单核对 + 行为化证据**能兜底（#8/#19/#25/#32 家族共同结论）。
- **证据**：两侧 `build.gradle`；progress §T0-2 与 §P2b 三跑结果。
```

---

## 条目 5（→ `knowledge/discovered/workflow-patterns.md`，建议 **#16 补充案例**，高价值，candidate）

> 说明：本条是 **#16 对照基线归因法**在「构建 profile 维度」的实例，按项目规则写补充案例、不新增编号。

```markdown
## 发现 #16 补充案例（260908-10）：debug profile 栈溢出不是移植缺陷——对照臂归因先分离「构建配置因素」与「被移植版本因素」

- **时间/置信度/module**：260908-10；candidate（双版本同二进制对照臂 + release 臂通过）；workflow-patterns（#16 对照基线归因法的**构建 profile 维度**）。
- **来源定位**：`.investigations/mc-1216-port-260908-10/progress-260908-10.md` §Rust 引擎 × 1.21.6 数据冒烟；`.tmp/p2b-rust-1216-smoke-260908-10.log:2869-2870`（1.21.6 debug 溢出）；`.tmp/p2b-rust-1201-control-260908-10.log:2835-2836`（1.20.1 同二进制同样溢出）；`.tmp/p2b-rust-1216-smoke-rel-260908-10.log:2868`（release 通过）。
- **现象**：Rust 引擎对 1.21.6 数据 debug 运行 `thread 'main' has overflowed its stack` / `error: process didn't exit successfully ... (exit code: 0xc00000fd, STATUS_STACK_OVERFLOW)`；但**同一二进制对 1.20.1 数据同样溢出** → debug 栈帧膨胀是共同原因，**不是 1.21.6 缺陷**。release 臂通过：16 chunk，`min=127.2ms median=135.6ms avg=140.1ms`。
- **根因**：debug profile 关闭优化、栈帧显著膨胀，density 树/树生成等深递归路径对栈深度敏感。「新版本数据 + 溢出」被误读成「新版本移植缺陷」的诱因，是把**构建 profile** 这个变量和**版本**混在同一个臂里（混杂变量未分离）。
- **定位**：对照臂归因（#16 原法）——同一二进制换 1.20.1 数据重跑，溢出同样出现即排除版本因素；再换 release profile 直证是工具/配置因素。
- **修复/判据**：修复 = 诊断 bin 走深递归路径（density 树/树生成）MUST 用 `--release`。判据 = 遇栈溢出先跑对照臂（旧版本数据 / 另一 profile），别急着立「新版本缺陷」课题；栈溢出类结论的措辞 MUST 带 profile 声明（§9.7 可比性：debug 与 release 口径不可互推）。
- **教训**：「异常差异」第一动作是造无 X 的对照（#16 原判据）；本条补**变量维度**：构建 profile / 编译器配置本身是常被忽略的混杂变量，应与数据版本分开单独扫。
- **证据**：上述三个日志路径。
```

---

## 条目 6（→ `knowledge/discovered/build-tooling.md`，建议 **#14 补充案例**，高价值，candidate）

> 说明：本条是 **#14「探针 dump 文件必须内嵌 seed 头」** 的反向陷阱（自证字段本身可能是另一个 seed），按项目规则写补充案例、不新增编号。

```markdown
## 发现 #14 补充案例（260908-10）：探针 dump 副产品文件名内嵌的是「探针参数 seed」不是「世界 seed」——文件名自证 ≠ seed 自证，误用即伪参照

- **时间/置信度/module**：260908-10；candidate（同一日志内两 seed 并存实读 + P7 修复后三处一致实锤）；build-tooling（#14 dump 自证 seed 的**字段语义陷阱**）。
- **来源定位**：`.investigations/mc-1216-port-260908-10/progress-260908-10.md` §P2b「seed 三查留痕」+ §P7「seed 三查」；`.tmp/p2b-block-260908-10.log:216-217`（`[BlockProbe] seed=-8248318472910187742` vs `[BlockProbe] worldSeed=-5307016484385870680`）+ L1186（产物 `vanilla_-8248318472910187742_8_200_200.blocks`）。
- **现象**：`BlockProbe` 产物文件名内嵌 `bench.seed`，但实际地形由 `worldSeed`（`server.properties` 的 `level-seed`）决定——两者不同时（本例 bench seed `-8248318472910187742` vs 未设 level-seed 随机得到 `-5307016484385870680`），`vanilla_<benchSeed>_..._<origin>.blocks` 是**伪参照**：文件名声称的 seed 与文件内容的地形 seed 不一致，拿它做对拍即 seed 三查违例。
- **根因**：`#14` 要求 dump 文件自证 seed，落地时嵌入的是**工具输入参数**（`bench.seed`）而非**世界生成 seed**（`worldSeed`）——两个 seed 在探针里都存在且都有名字，但只有后者决定地形；文件名模板选了前者，自证字段就变成误导字段（**自证 ≠ 正确自证**）。
- **定位**：同一日志内 `seed=` 与 `worldSeed=` 两行并列对照；修复路径 = 先设 `level-seed` + 删 `run/world` 重导，之后 `worldSeed` = level-seed = bench seed，两臂导出 header 逐字段相同（P7 实锤）。
- **修复/判据**：判据（MUST）：① 导出参照前先设 `level-seed` + 删 `run/world`（否则 worldSeed 随机）；② 对比前核对三处一致——产物 header/文件名内嵌 seed == 命令行/bench seed == 日志 `worldSeed`；③ **dump 文件头应同时落 bench seed 与 worldSeed 两个字段并标注哪个是地形 seed**（#14 字段清单升级），只落一个时按「伪参照」处理。
- **教训**：「文件自证」只有在**自证字段与结论所依赖的语义同一**时才成立——自证字段选错比没有字段更危险（看起来有据可查）。
- **证据**：`p2b-block-260908-10.log` 两行 + 产物文件名行；progress §P7 三处一致记录。
```

---

## 条目 7（→ `knowledge/discovered/build-tooling.md`，建议 **发现 #41**，高价值，candidate）

```markdown
## 发现 #41: PowerShell `Select-Object -First N` 提前关闭管道会杀掉上游进程——长时构建日志被截断成「构建中断」假象（260908-10）

- **时间/置信度/module**：260908-10；candidate（同一命令两次运行对照：截断日志 vs 全量落盘日志）；build-tooling / PowerShell 环境坑。
- **来源定位**：`.investigations/mc-1216-port-260908-10/progress-260908-10.md` §T0-3 ⚠️过程坑；`.tmp/p2b-compile-r2-260908-10.log`（截断：104 行、无 `BUILD` 行、末尾停在堆栈中间帧，夹 `java.io.FileNotFoundException: ...mixin-targetdb-*.tmp`）vs `.tmp/p2b-compile-r3-260908-10.log`（全量落盘，含 `BUILD SUCCESSFUL in 8s`）。
- **现象**：`gradle ... | Select-String ... | Select-Object -First 30` 运行时 gradle 被中途终止——日志只剩半截堆栈（末尾停在 `DefaultBuildOperationRunner.execute` 中间帧），被误读为「构建中断/构建失败」。
- **根因**：PowerShell 管道中 `Select-Object -First N` 满足数量后**停止消费并关闭下游管道**，上游进程（gradle/java）收到管道关闭后终止——early-exit 的正常语义，但对「长时间运行、日志即证据」的构建命令等于**中途杀进程**；截断的堆栈与真实失败的堆栈在观测上不可区分。
- **定位**：对照两次运行——截断版 104 行、无 `BUILD` 行、末尾非自然结束；全量版有 `BUILD SUCCESSFUL`；确认是管道早退而非构建错误。
- **修复/判据**：修复 = 长时构建/采集命令一律 `... *> <logfile>` 全量落盘，**再**对落盘文件单独过滤（`Select-String <logfile>` / `Get-Content -Tail`）。判据 = 「日志末尾停在堆栈中间帧 + 无 BUILD 行」是**管道截断签名**，先复跑全量落盘再判构建失败。
- **教训**：日志采集与日志过滤必须**两阶段分离**；任何在管道里做 early-exit 截断的写法都会把「采集侧副作用」伪装成「被测系统故障」（与 #29 RCON 帧解析伪装「服务器挂死」同构：工具层假象优先排除）。
- **证据**：两个日志文件对照（r2 截断 / r3 完整）。
```

---

## 「不写」清单

**候选清单 7 条全部过价值门（高价值）→ 无一列入「不写」。** 逐条理由见上表；判据均为「若再遇到，我不想重新想一遍」（错误链条 / 判错方法 / 环境坑 / 可复用判据）。

**非候选清单、附带声明不写**（本轮 session 内结论，低价值/一次性，按价值门不入知识库，仅留 `.investigations/`）：

| 结论 | 不写理由 |
|---|---|
| P5 ChunkStatus 邻居依赖表「挂起（不做）」 | 一次性课题范围决策（判据 = 本侧无消费点），无跨版本/跨项目复用价值；若未来出现「邻 chunk 状态不足」失败再立项时另记。 |
| 附带发现 1：Rust `aquifer.rs:493-499` water-over-lava 用全链 `get_fluid_level`（y≤−10 深部、只返回 WATER） | 已定位但未立项未修复的**有界已知偏离**，属待办事项不是可复用判据；修复后若要沉淀应进错误台账（`.investigations/<课题>/`）而非 discovered。 |
| 附带发现 3：`worldgen_handle.rs` 给 `Aquifer::new` 传 world_height(256) vs Java height()(128)，nether 下 aquifers_enabled=false 不消费 | 同上：当前不触发的待办偏离，写进知识库会稀释高价值权重。 |
| 产物跨版本核对的具体数字（blocks 1105 条 / biome_params 40 行 diff / nether sha 相同） | 低价值：当前对齐状态快照 + 某次数值（价值门明列不写）；结论性内容已在 `.artifacts`/progress 留档。 |
| P7 对拍量化指标 99.8114% → 99.9993% | 作为条目 1 的**证据**保留在条目内即可，不单独立条（单独成条 = 一次性数值）。 |

---

## 主会话应用提示（非知识库内容，供落盘操作）

1. **新增编号**：`workflow-patterns.md` 追加 **#87**；`build-tooling.md` 追加 **#40 / #41**；`compiler-idioms.md` 不动。
2. **补充案例位置建议**：`build-tooling.md` 的 `#25 补充案例` 紧跟现有 `#25 补充案例（#8/#25 家族，260906-06）` 之后；`#8 补充案例` 可紧随 `#8` 主体或同族补充案例区；`#14 补充案例` 紧随 `#14` 主体；`workflow-patterns.md` 的 `#16 补充案例` 紧随 `#16` 主体（现有 #16 在 L219-229）。
3. **INDEX.md 同步**：build-tooling 行补 #40/#41 摘要 + `#8/#14/#25` 补充案例标注；workflow-patterns 行补 #87 摘要 + `#16` 补充案例标注；并在文末追加一条「260908-10 追加」说明行（与既有追加行格式一致）。
4. **置信度**：7 条全为 **candidate**（有行为级/一手证据）；**不得**由 AI 标 confirmed——confirmed 留用户拍板（本轮 verdict 本身仍是 candidate，见 progress §待办 1）。
