# Scout 勘探地图 — 1.21.6 BlobProbeMixin「Mixin transformation failed」（b61x-260918-03）

角色：recode.scout（只读勘探；本 subagent 无 shell，全部结论基于文件直读，沙箱读不到的文件如实标注——本次无读取失败）。
触发日志：`.investigations/b62-260918-02/cmd-output/sentinel-1216-all12.log:137-143`。

## 0. 一句话形态判定（关键新结论）

**失败时序证明：mixin 注入本身是成功应用的，失败发生在 33 秒后 `BlobProbe.java:39` 的 `Class.forName("wg.bench.mixin.BlobProbeMixin")` 反射自载点。**
- 12:56:07（启动后 ~13s）`handler active worldClass=ChunkRegion` + `dimSeen minecraft:the_nether`（log:138-139）→ 注入已生效，`CALLS`/`WRITTEN` 正在累计。
- 12:56:40 `stats read failed`（log:142）紧跟 `BlobProbe.java:39/42` 的两连 `Class.forName(...).getDeclaredField("CALLS"/"WRITTEN")`。
- Mixin 对目标类织入时**不需要类加载 mixin 类本身**；mixin 类首次被真正类加载（此处 = 反射强制）才走 transformer 的自变换路径并抛 `RuntimeException: Mixin transformation of ... failed`。**「前半正常、后半失败」= 惰性类加载时点，不是两段代码分属两个 mixin。**

## 1. 文件清单

| 项 | 1.21.6 树 | 1.20.1 树 |
|---|---|---|
| BlobProbeMixin | `versions/1.21.6/java/src/main/java/wg/bench/mixin/BlobProbeMixin.java`（99 行） | `versions/1.20.1/java/src/main/java/wg/bench/mixin/BlobProbeMixin.java`（99 行） |
| 反射读取方 | `versions/1.21.6/java/src/main/java/wg/bench/BlobProbe.java`（52 行） | `versions/1.20.1/java/src/main/java/wg/bench/BlobProbe.java`（52 行） |
| mixin json | `versions/1.21.6/java/src/main/resources/coreswap.mixins.json` | `versions/1.20.1/java/src/main/resources/coreswap.mixins.json` |
| build.gradle | `versions/1.21.6/java/build.gradle`（loom 1.10.5, loader 0.16.14, fabric-api 0.127.0, **release=21** :245） | `versions/1.20.1/java/build.gradle`（loom 1.10.5, loader 0.15.11, fabric-api 0.92.0, **release=17** :266） |
| gradle.properties | JDK 21（`org.gradle.java.home=.../jdk-21`） | JDK 17（`.../jdk-17.0.12`） |

注：`.tmp\w2\pre-wt\` 下另有两份历史副本（工作区临时区，非现役，未读）。

## 2. 两树 diff 表

| 维度 | 1.21.6 BlobProbeMixin | 1.20.1 BlobProbeMixin | 差异 |
|---|---|---|---|
| @Mixin 目标 | `PlacedFeature.class`（:34） | 同 | 无差 |
| 注入点 | `@Inject(method="generate(Lnet/minecraft/world/StructureWorldAccess;Lnet/minecraft/world/gen/chunk/ChunkGenerator;Lnet/minecraft/util/math/random/Random;Lnet/minecraft/util/math/BlockPos;)Z", at=@At("HEAD"), require=1)`（:45-48） | 同 | 无差 |
| @Unique 静态字段 | `CALLS`/`WRITTEN`/`DIMS_SEEN` + 非 @Unique `BLOB_PROBE`/`BLOB_OUT`（:37-41） | 同 | 无差 |
| 嵌套类 / static nested | **无** | **无** | 无差（KB #12 排除，见 §5-C） |
| 唯一源码差 | :81 `world.getRegistryManager().getOrThrow(...)` | :81 `.get(...)` | **仅此一行**（yarn API 差异），与变换无关 |
| BlobProbe.java | 两树逐行同构（含 :37 `calls=-1, written=-1` 哨兵初值、:39-46 反射块） | 同 | 无差 |
| mixins.json | `required:true`、`package:wg.bench.mixin`、`compatibilityLevel:JAVA_17`、27 条含 `BlobProbeMixin`（:21）、`defaultRequire:1` | 同构，26 条 | 条目数不同但 BlobProbeMixin 均注册且名字精确匹配类文件 |
| **编译目标** | `options.release = 21`（build.gradle:245）→ mixin 类字节码 **v65** | `options.release = 17`（:266）→ **v61** | **★ 两树唯一实质层差异（字节码版本）** |
| **loader** | fabric-loader **0.16.14** | **0.15.11** | ★ 差异（捆绑的 mixin 版本不同） |

## 3. 关键代码摘录（file:line）

- 抛"stats read failed"：`versions/1.21.6/java/src/main/java/wg/bench/BlobProbe.java:46`（catch 打印 `+ t`，**只打顶层 RuntimeException 消息，cause 链被吞**——底层真实异常不可见）。
- 反射自载点：`BlobProbe.java:39,42` `Class.forName("wg.bench.mixin.BlobProbeMixin").getDeclaredField("CALLS"/"WRITTEN")`；`mixinCalls=-1 written=-1` 的 -1 = `BlobProbe.java:37` 哨兵初值（异常路径未更新）。
- `Mixin transformation of %s failed` 字符串出处：**不在本工作区源码内**（grep 全 workspace 仅命中日志/文档）——属 fabric-loader 0.16.14 捆绑的 Mixin transformer 内部包装文案，源码在依赖 jar 内（本次无 shell 无法解包核对，标注 Degraded）。
- 计数写入侧：`BlobProbeMixin.java:53`（CALLS++）、:93（WRITTEN++）——异常发生前已正常累计（log:138-139 佐证），**因此 1.21.6 内那轮 run 的 origins.csv 数据本身大概率有效，丢的只是收尾统计行**。

## 4. 互斥根因候选（fan-out 输入，各附证据）

**候选 A（推荐主候选）：mixin 类反射自载触发 transformer 自变换路径，在 1.21.6 工具链组合下失败——探针「反射读 mixin 类私有静态字段」的设计在新链上不可用。**
- 支持：失败时点与 Class.forName 精确重合（log:142 vs BlobProbe.java:39）；注入已成功证明类字节码可被 mixin AP/织入器消费；两树源码同构 → 差异只能在工具链/字节码版本。
- 反对：无法解释为何 transformer 恰好对该类失败（底层 cause 被吞）；未测「同树另一 mixin 反射自载是否同样失败」。

**候选 B：类版本 / compatibilityLevel 失配——1.21.6 编译 v65（release=21）而 mixins.json 声明 `JAVA_17`（json:5），mixin 自变换路径对 mixin 类自身做版本校验/重写时超界。**
- 支持：两树唯一字节码层差异就是 65 vs 61（build.gradle:245 vs 1.20.1:266）；compatibilityLevel JAVA_17 两树同值，1.20.1 恰好一致、1.21.6 恰好失配——与「只在 1.21.6 失败」的观测吻合（1.20.1 侧成功与否未实测，见 §6）。
- 反对：其他 26 个 mixin 同为 v65 却正常应用——但注意它们**从未被类加载**（只被织入），不构成反证；需实验判别。

**候选 C（KB #40 家族：json 与类失同步）——已被削弱，倾向排除。**
- 支持：形态上同为「编译绿 ≠ 运行时绿」。
- 反对：json 明确注册 `BlobProbeMixin`（json:21）、类名精确、`require=1` 未触发 APPLY FAILED、注入实际已生效（log:138）——#40 是装载期失败，本例是**后置的自变换失败**，非同机制。

**候选 D（KB #12：mixin 包含 static nested）——排除。**
- 反对（决定性）：BlobProbeMixin 无任何嵌套类/第二顶级类（全文 99 行直读核对）；且 #12 的签名是启动期 IllegalClassLoadError，不是延迟 33s 的变换失败。

## 5. 「前半正常后半失败」形态解释（勘探目标 5，结论）

- 不是嵌套类（§4-D）、不是反射加载另一个含 mixin 注解的普通类（反射目标是 mixin 类自身）。
- 真实形态 = **mixin 类惰性类加载**：织入不需要加载 mixin 类本体；运行期唯一加载点就是 BlobProbe 收尾统计的反射（BlobProbe.java:39），该加载走了 mixin transformer 的类变换入口并在其中抛错。transformer 失败发生在收尾，不影响此前已织入目标类的 handler 代码继续运行—— hence 前半段日志全正常。

## 6. 建议下一步（交主会话裁决 / fan-out）

1. **廉价对照（一轮）**：查 1.20.1 树同参数 run 的哨兵/服务器日志是否出现 `stats read failed` 或 `mixinCalls>=0`——直接判定「1.20.1 也失败」还是「仅 1.21.6 失败」，一票定 A/B 走向。
2. **拿到底层 cause**：BlobProbe.java:46 改打完整 cause 链（`t.getCause()` 或直接传 t 给 logger 带堆栈）重跑一次最小 run——顶层 RuntimeException 是包装，真实异常（ClassFormatError/VersionException/…）决定 A vs B。
3. **判别实验（B 候选）**：1.21.6 mixins.json 的 compatibilityLevel 升 `JAVA_21`（或 options.release 临时降 17）单变量重跑——变化即坐实类版本面。
4. **若 A 坐实（自载机制不可用）**：修法 = 把 CALLS/WRITTEN 移到非 mixin 包的普通类（如 `wg.bench.BlobStats`，mixin 持引用写入），反射只碰普通类——同时天然消除「反射进 mixin 类」这一平台语义灰区（与 compiler-idioms #12 mixin 包纪律同族）。
5. 修复落点均在 java 源码/配置，属收敛工程修复（不消耗 evidence saturation 计数）；根因定论前按 #40 判据纪律不标 candidate 以上。

置信度：§0 时序判定 / §2 diff / §5 形态解释 = 文件直读一手（candidate 级）；候选 A/B 机制归因 = Degraded（依赖 jar 内 mixin 源码未核、底层 cause 未采集）。
