# record — R9-b light reader × bulk writer SENTINEL 覆盖判定（260918-10）

> 状态：draft（T2 收敛，待 judge）；验证分层：Degraded（全静态一手源对拍 + sha256 对账）。
> 架构：`.investigations/000-架构设计/架构计划-260918-10-R9b-light-reader-sentinel.md`（已批准）。
> 勘探：`scout-map.md`（读者清单 R1-R9 / 时序交叉 W1-W6 / OQ-1~OQ-6）。

## 1. fan-out 候选与收敛

| 候选 | 结论 | 关键证据 |
|---|---|---|
| .b1 已覆盖说 | **成立** | `ProtoChunk.status` = volatile（ProtoChunk.java:42）；setStatus(NOISE) 在 doWork().thenApply 内 = volatile 写 release（ChunkStatus.java:361-362，:361 有 volatile 守卫读）；读者门 `getStatus().isAtLeast(FEATURES)`（Mixin:313）= volatile 读 acquire；门过 ⇒ NOISE 全部 plain 写（含 BulkWb :205-209 换容器+计数）可见；FEATURES 后无 block 容器写者 ⇒ 读面冻结；OQ-5 由 `PalettedContainer.data` volatile（:36）+ status 门双重兜底。 |
| .b2 真缺口说 | **不成立（自证伪）** | 前提「status 普通写且无同步边」被一手源证伪——读者读的是 ProtoChunk volatile 字段而非 ChunkHolder futuresByStatus（读者根本不触碰）；b2 自行改判「HB 闭合」，并给出加固选项清单（读者登记 / future join / volatile 化——最后一项实为无操作）。 |
| .b3 vanilla 同构既有说 | **成立，附一处限缩** | vanilla 写/读顺序承载三要素（volatile status 全序 + 票/light 队列 + 裸引用读者）CoreSwap 全部原样保留；vanilla 读者（ChunkLightProvider.java:72-77）连 status 门都没有，CoreSwap R1/R2 读者侧更严；bulk 写将可见中间态从「逐块渐进」收敛为「旧或新完整容器」= 原子性增强。限缩：R1/R2 是 CoreSwap 新增读者站点，成立形态 =「同构义务的新站点 + 写者等价替换」，其闭合依赖 b1/b2 的 HB 论证（b3 不独立覆盖）。 |

**收敛裁决（T2）**：三候选汇于同一结论——
1. **R1/R2 光照读者 × bulk 写回：无 data race 缺口**（volatile status 门构成 HB；R9-b「单写者不变量」论证经 #143 四件套核对对读者形态同样闭合，且不依赖 future join）。**HB 传递边明示（judge SHOULD-1）**：release/acquire 是逐写逐读配对——读者 acquire 的是 FEATURES 那次 volatile 写，NOISE 写者 → 读者之间的传递性由 **ChunkStatus.java:361 的 volatile 守卫读**（FEATURES 写者写前必先 volatile 读 status ⇒ synchronizes-with NOISE release，继承其全部 plain 写）与 **CompletableFuture 依赖链**双重承载，任一即足；judge 独立推演「旧 section 内容 × 新 status」反序窗口不存在（volatile 读 acquire 封死后续普通读上浮 + 读前无更晚写者）。**残留边界（#143 全称否定断言，judge SHOULD-4）**：「FEATURES 后无 block 容器写者」是对当前 status 链的静态断言，未来 status 链改动（新增写 block 容器的 status）不承担本判定证明义务，届时须重开。
2. **OQ-5 闭合**：门内读者 writePacket 迭代无半构造窗口（发布前完整构造 + 门后无并发写者）。
3. **SENTINEL 读者不登记 = 检测面盲区（OQ-4），非正确性缺口**——加固定性从「义务修复」降级为「SHOULD 级可观测性增强」（选项：读者登记 / future join / volatile 化/无操作）。
4. **OQ-2 大部闭合**：b3 证 vanilla 自身读者比 CoreSwap 更弱（无门），CoreSwap 形态 ∈ vanilla 既有义务类；R4/R5 的 TicketManager 装配细节 b1 未实读（诚实边界保留，结构性同链保护成立）。

## 2. mcsrc 版本疑点消解

- b3 双通道核对：`.tmp\scout-260905-08\mcsrc` 7 关键文件 sha256 与 `versions\1.20.1\data\mc_src_extract` **7/7 逐字节相同**（复算留痕 `cmd-output/mcsrc-sha256.txt`，judge SHOULD-3）；特征符确认 =1.20.x 且 <1.20.2 ⇒ 版本疑点关闭，**本块行号引用不因此追加额外降级**（本课题整体验证分层仍为 Degraded 静态，见文件头，scout §5 疑点消解不改变分层声明——judge SHOULD-2 措辞修正）。
- **文档勘误（§15.4，待应用）**：07 篇 :1524「INITIALIZE_LIGHT 1.21.6 新增」与 1.20.1 源（ChunkStatus.java:158，双树一致）冲突——INITIALIZE_LIGHT 在 1.20.1 已存在；1.21.6 侧真实差异是经 `ChunkGenerationSteps` 注册链。原句不改，补取代注（knowledge 草稿阶段一并处理）。

## 3. 开放问题终态

- OQ-1：闭合（HB 成立）。OQ-2：结构性闭合（TicketManager 细节留诚实边界）。OQ-3：不需双开关矩阵逐臂声明（正确性已闭，与臂无关）。OQ-4：**未闭合——scope 决策交用户**（sentinel 读者登记与否）。OQ-5：闭合。OQ-6：INPLAY 期超本块范围（维持）。

## 4. 证据分级（v0.22 §1.3）

- numeric：mcsrc sha256 7/7 对账；file:line 证据表见 .b1/.b2/.b3 产物。
- qualitative：三候选收敛裁决推理（见 §1）。
- anchor：待 judge 三源核对。

## 5. 待办

- [x] judge MUST（三源核对）：**PASS-with-conditions**（0 MUST / 4 SHOULD / 3 INFO）——SHOULD-1（传递边明示）/ SHOULD-2（Degraded 措辞）/ SHOULD-3（sha256 留痕 `cmd-output/mcsrc-sha256.txt`）/ SHOULD-4（残留边界句）均已应用；核心风险点独立推演窗口不存在。
- [x] OQ-4 scope 决策（用户拍板 260918-10：sentinel 读者登记**不实施**，SHOULD 级可选项）。
- [x] 知识库更新（subagent 草稿 `knowledge-draft-260918-10.md` + 主会话应用：workflow-patterns #187（judge 传递边要点并入判据 1）/ knowledge INDEX 尾注 / 07 篇 §15.4 取代注 + R9-b 追加小节 / 10 篇 260918-10 条目）。
- [ ] 用户 confirmed：本块收敛裁决 + #187（下轮 hook）。
