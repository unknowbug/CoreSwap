# J5 trace verdict（j5-trace-verdict-260906-01）

> 角色：core.worker 解读（只读数据，未跑命令）；状态：**draft**。
> 输入：j5-rng-trace-plan-260906-01（T0-T7 判据）+ 三棵树 trace + 两侧原始日志。
> 数据：seed 8576294172403134396，单 chunk (29,-16)（x∈[464,480), z∈[-256,-240)）。
> 验证分层：**Partial（降级声明）**——两侧树位集不相交，只能做 T5 形状级锚定对照，非同树逐消费对拍。

## 1. 采集有效性（T0/T2）

- T0 ✓：Java 侧 2 条 [MJT0]（chunk 过滤生效，原始日志仅此 2 mega：464,71,-252 h=27；479,71,-249 h=27，各以 [MJTX] 收尾）。Rust 侧按坐标过滤核对：目标 chunk 内**恰 1 棵** mega（474,71,-244 h=18；全量日志中相邻 (461,-255)/(479,-216) 均在 chunk 外）。
- T2 ✓（正式核销同树对拍）：Java 起点集 {(464,71,-252),(479,71,-249)} ∩ Rust {(474,71,-244)} = ∅。**分歧在 trunk generate 之前（placement/树位选择层），「同一棵树逐消费对拍」不可行**——本实验按预案转 T5 锚定模式。

## 2. ⚠️ 数据口径注意（实际 tag 与方案不符）

实际三份 trace 均为 `[MJT0/MJTD/MJTT/MJTG/MJTX]` 标签集——**Rust 侧实际用的是 Java 镜像 tag（MJTT/MJTG），方案 §1 的 [MJTL]/[MJTI]/[MJTF]/[MJTB]/[MJTS] 未出现**。影响：
- Rust [MJTT]→[MJTG] 链若与 Java 同层（trySetState/getAndSetState 语义），则两侧口径恰好对齐（对形状判读反而有利）；
- 但 i 初值（MJTI）、角度 bits（MJTF）、逐枝干 ok（MJTB）均不可见 → T4（角度差）与逐 token 相位判定本轮**不可做**；
- 该 tag 集与 prompt 描述（Rust=MJT0/D/L/I/F/B/S/X）也不符，采集注入的实际版本待主会话核对——不排除 Rust 侧实现的是「trySetState 等价层」打点。以下判读在形状级不受影响，逐消费级全部标注不可用。

## 3. T5 形状对照（三棵树）

| 形状指标 | Java A (464,71,-252) h=27 | Java B (479,71,-249) h=27 | Rust (474,71,-244) h=18 |
|---|---|---|---|
| [MJTD] 行数 | 4 ✓ | 4 ✓ | 4 ✓ |
| 柱层结构 | y=71..96 全 4 块 + 顶 y=97 仅 2 块 | 同构（顶 y=97 仅 2） | 同构（顶 y=88 仅 2） |
| 柱 ok 计数 / 拒绝 | 106/106，**柱层拒绝 0** | 110/110，柱层拒绝 0 | 70/70，柱层拒绝 0 |
| 枝干轮数 | 2（每轮恰 5 次 MJTG） | 3（5/轮） | 2（5/轮） |
| 枝干拒绝 | 轮1: 2/5，轮2: 2/5（共4） | 4/4/3（共11） | 轮1: 3/5，轮2: 1/5（共4） |
| [MJTX] nodes | 3 = 轮数+1 ✓ | 4 = 轮数+1 ✓ | 3 = 轮数+1 ✓ |
| 枝干延伸形态 | 同柱方向逐步 + 端点 2-3 ok | 同构 | 同构 |

**结论：三棵树形状全同**（MJTD=4、柱满高全 ok 顶行缺 2、枝干 5 调用/轮、nodes=轮数+1、拒绝只出现在枝干层不出现在柱层）→ 按 T5 判据「形状全同 ⇒ 树内放置机制无结构性差」。

## 4. T6 消费计数（total draws，Java dirt-get 为 residual）

公式 = getHeight(2) + dirt(0..4, 不可见) + 柱ok + 1(ni0) + Σ轮(1角度+枝干ok+1步长)：
- Java A：2 + 106 + 1 + 2×(1+6+1) = **125** (+0..4 dirt residual)
- Java B：2 + 110 + 1 + 3×(1+5+1) = **134** (+0..4)
- Rust：2 + 70 + 1 + 2×(1+6+1) = **89** (+0..4)

三树互异，计数不可直接对拍；仅自洽性核对通过（各树内部公式吻合，无多计/漏计）——即树内消费序列长度与形状一致，无隐藏多消费信号（注意：BlockStateProvider.get 若为 weighted 型其内部消费已含在「柱ok/枝干ok」项内，两侧同源 JSON，暂排除）。

## 5. T7 / C-R1 核查（Partial，形状级）

C-R1 预期：Rust place_log 缺 canReplaceOrIsLog → 柱层系统性 ok 差（Rust 多放/多消费）。实测：
- Rust 柱 ok 计数 = 尝试数（70/70，**0 拒绝**），Java 两侧同样 0 拒绝；
- 两侧拒绝均只出现在枝干层（几何越界型，形态同构），无柱位 ok=false vs Java 同位拒绝的对子可查（树不同，无法逐位）。
**C-R1 未证实**（形状级无柱层系统性 ok 差；且实际 Rust tag 集表明打点层可能已是 trySetState 等价层，进一步削弱 C-R1 的前提）。保留为候选但降优先级。声明：不同树、缺 [MJTL] 原始口径，此判断为 Partial。

## 6. h 值分布讨论（定性，T3 不可用）

Java 两棵 h=27，Rust h=18——均落在 vanilla mega jungle 高度随机范围内（h=18 非异常值）。T3（同起点 i 差）因无同树不适用。h 由 trunk generate 之前的 height 随机化（消费同一 chunk 装饰随机流）决定，故 h=27 vs 18 本身即**上游流相位差的旁证**：若两侧该 chunk 装饰流相位一致，第一棵 mega 的 h 分布应相同。无法区分「getHeight 实现差」与「上游相位差」——但 getHeight 在 TrunkPlacer.getHeight 内 2 draws，此前 [TH]/[THJ] 已对拍通过（方案 §1 记录），实现差的先验低。**定性结论：偏向上游流相位/位置选择差，非 getHeight 实现差。**

## 7. Verdict（draft，待 judge）

1. **下一主攻方向 = J5-④（RNG 流漂移）**，且定位收窄到 **trunk generate 之前的 placement/装饰流层**（树位选择、树尝试顺序/次数、或装饰 Random 分支/相位）。证据：① T2 树位集不相交 = 分歧先于树内消费；② T5 形状全同 = 树内机制无结构性差；③ h 分布差 = 上游流相位差旁证。
2. **J5-③（mega 放置成功性差）本轮无支持证据**：C-R1 未证实、柱层零拒绝、形状无成功性差异 → 降优先级（不证伪，仅本轮无证据）。
3. **C1（树内相位重排解读）核销状态：证伪并升级**——「同一棵树内 getHeight 前流已漂移的相位重排」表述不成立（T2：根本没有同一棵树）；C1 升级为「**placement 层分歧**（树位集不相交，重排发生在装饰流上游）」。judge C1 待核销项按此结转。
4. 后续建议：主攻装饰流上游——对 chunk (29,-16) 做 feature 放置层打点（每次 tree attempt 的 (pos, random 相位) 序列两侧对拍，不依赖树相同），或直接对拍 decorated feature 调用序/尝试数（J5-①② 的 bush/尝试数候选与此合流）。

## 8. 错误→教训沉淀候选（按 SUBAGENT-KNOWLEDGE-GUIDE 五段式，供主会话挑拣转正式条目）

1. **gradle JavaExec stderr 合并转发 → stderr 采集假空**
   - 现象：按方案 §3 预期 Java stderr 有 [MJT*] 输出（.log.err），实际 stderr 空，全部 [MJT*] 出现在 stdout。
   - 根因：gradle runServer 的 JavaExec 默认 stdio 合并转发，子进程 stderr 被并入 gradle 自身 stdout 流（log 配置 showStandardStreams 对两流统一转发），「stderr 侧文件」从一开始就不存在独立内容。
   - 定位：采集后在 .log.err 中 grep [MJT*] 零命中、stdout 中全量命中。
   - 修复：以 stdout 为唯一采集载体（本轮已是），stderr 文件不再作为有效性判据。
   - 教训：跨进程日志采集先核「流拓扑」（谁合并谁），不要按计划假设 stderr 独立可达；采集有效性判据应写成「内容命中」而非「文件存在」。
2. **sysprop ≠ env 两个命名域（再犯确认）**
   - 现象：同一诊断开关（WG_TREEDIAG / wg.treediag）在 Java 侧需 sysprop（-D）与 env 两条路径都覆盖，混用其一即静默不生效（mixin 侧 `Boolean.getBoolean(...) || getenv(...)!=null` 双检才能兜住）。
   - 根因：gradle daemon/JavaExec 进程的 env 与 JVM sysprop 是两个独立命名域，gradle 不自动把 env 映射为 sysprop；注入点不同（JAVA_TOOL_OPTIONS / -Pxxx / runServer jvmArgs）落点不同。
   - 定位：开关不生效时先打印两域实际值比对。
   - 修复：诊断开关一律双域读取（本轮 mixin 写法即此）；采集命令模板中显式同时设置。
   - 教训：诊断开关「不生效」第一查命名域归属，不是先查注入代码——本项目第二次命中此坑，应升级为采集模板固定项。
3. **mixin LocalCapture 前缀 capture LVT incompatible → 回退最低保障集**
   - 现象：[MJTI]/[MJTF]（locals capture i/f）两轮注入失败（运行期注入错误），Java 侧最终无 i/角度值，仅剩 [MJT0/MJTD/MJTT/MJTG/MJTX]。
   - 根因：@LocalCapture/@At INVOKE + 局部槽序匹配对编译器局部变量表（LVT）布局敏感，Loom 重映射/编译器差异导致槽序与源码序不一致 → 运行期 incompatible 而非编译期报错；失败模式在运行期才暴露，成本高。
   - 定位：注入失败在 runServer 启动/首次命中时抛 mixin 异常；按预案 §5 风险 2 回退纯 HEAD/RETURN 最低保障集。
   - 修复：本轮按最低保障集交付（形状级判据不受影响）；locals capture 改走「下一次调用前 capture 参数」或其他低风险注入点再评估。
   - 教训：mixin locals capture 是高风险注入，任何 trace 方案必须预登记「最低保障集」并预先写好回退判据（本案预案已做，教训=确认该做法为标准动作）；编译通过 ≠ 注入可运行。
4. （观察项，非错误）**实际打点 tag 集与方案不符**（§2）：Rust 侧落地为 Java 镜像 tag 而非方案 §1 的 [MJTL/I/F/B/S] 集——采集注入版本与方案脱节，导致逐 token 判据（T3/T4）整轮不可用。教训：采集脚本/注入的最终落地版本必须回写方案文件（一行「实际 tag 集」注记），否则解读侧只能盲猜口径。

## 9. 自检清单

- [x] 数字全部来自本轮 trace 文件实测，无编造；Java dirt-get residual 已注明
- [x] 降级声明：Partial（形状级，非同树对拍）
- [x] C-R1 未证实但未删除（候选降优先级）
- [x] C1 处置 = 升级为 placement 层分歧（取代表述，原假设不成立已注明）

---

## 附录（260906-01 主会话核实补记，数据层事实）

1. **载具更正**：本文 §1-§7 所引「Rust 侧」trace（j5-rust-tree-474-71-244.txt 等）实为 **Java mixin 产物**——该 run（j5-rust-mjt.log）中全部 MJT0/MJTD/MJTX 行都是 Java 格式（soil=? 计 1952，Rust 格式 soil=true/false 计 0；Rust 独有标签 [MJTL]/[MJTI]/[MJTF]/[MJTB]/[MJTS] 全 0）。§2-§6 中「Rust 形状」结论**作废**，T5 对照需以新数据重做。
2. **新 Rust 载具**：worldgen-core/src/bin-diag/j5_tree_trace.rs（WorldgenHandle::create + fill_chunk_blocks，5×5 邻域），native run 日志 .tmp/jungle-l-260905-13/j5-rust-native-trace.log（WG_TREEDIAG=1，stderr）。结果：5×5 邻域 11 棵 mega（(436,70,-232)h17 … (505,72,-261)h30），**目标 chunk (29,-16) 零 mega**（Java 同 chunk 有 2 棵）→ placement 层分歧在 chunk 级即成立，J5-④ 方向获数据层支持。
3. **载具警告（#36）**：b2b3-rust log（260905-13）是 pre-stageMask 链路（20260908 提交前 Rust 特征在存档链内执行），其目标 chunk 树位与 native 载具不一致——**b2b3 Rust 历史数据与 native/server 现行载具三者互不可直接续推**。
4. 本块错误台账：.investigations/jungle-l/260906-01-errors.md（E6 sysprop≠env 二犯 / E7 stageMask 默认当公理 / E8 gradle stderr 转发 / E9 LocalCapture 回退）。
