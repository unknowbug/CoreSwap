# 知识库追加草稿 — 260904-03（错误台账 LL12/LL13 + workflow-patterns #32 + 10 时间线条目）

> **本文件为草稿**（core.worker 产出），供主会话应用。已过记录价值门：LL12/LL13 与 #32 均为高价值（环境坑/死判别风险/流程反模式，可复用判据）；非低价值一次性结论。

---

## 一、错误台账追加（应用目标：`.investigations/lossless-accel/lossless-accel-errors.md`，追加于速查表之前；速查表加两行）

### LL12. Run1 参照产物被 Run2 同名覆盖（260904-03，P2 Full 回归流程瑕疵）

- **现象**：P2 存档口径 Full 回归中，Run1（vanilla 参照导出）与 Run2（cppReplace）输出**同名文件** `.tmp/p2full/vanilla_8576294172403134396_4_200_200.blocks`——Run1 首跑产物被 Run2 覆盖丢失，只能重跑 vanilla 导出（run1b）补参照。
- **根因**：实验设计阶段未给两臂分配独立输出路径/后缀——A/B 实验的产物命名与臂标识脱钩，覆盖是静默发生（无任何报错）。
- **定位**：采集清单复核时发现两 run 文件名相同 → 判定首跑产物不可信 → run1b 独立重跑 + 参照四要素核对（magic/seed/size/origin/header）确认无数据混入。
- **修复**：本轮以 run1b 重采参照闭合（结论不受影响）；无代码修复。
- **教训**：**A/B 多臂实验的产物路径必须含臂标识（目录或后缀），采集前核对命名清单**——静默覆盖是「无报错的流程错误」，事后只能靠重跑补救；同族：LL2（header 与内容不符——命名/注释不可信，内容实测为准）。
- **状态**：candidate（流程瑕疵已闭合，无残留数据影响）。

### LL13. gradle daemon 复用吞掉客户端 env → 反转臂死判别风险（260904-03，Run3）

- **现象**：P2 Full 回归 Run3 设计为 cppReplace + `WG_EST_SHARED=0/WG_EST_L2=0`（double-0 反转）以独立验证 env 门控生效；但 Run3 走的 gradle daemon 复用了 run1/run2 的 JVM 环境——客户端进程设置的 env 是否透传进 daemon 未验证，Run3 输出与 Run2 SHA256 完全一致（`2FF71249…10BC`）存在「env 根本没生效 → 天然同 hash」的死判别可能。
- **根因**：gradle daemon 是复用 JVM，`gradle runServer -D…`/客户端 env 的传递路径依赖 gradle 配置；**复用 daemon 时新开 shell 设置的 env 不保证进 daemon 子进程**——判别实验的「自变量被改变」前提未被机械验证（workflow-patterns #20 死参数族的环境变量实例）。
- **定位**：本轮仅事后识别（设计评审发现），未做 env 探针验证；env 门控生效的**主要证据**仍为 260903-13 `estopt-ab-defaultflip-260903-13.txt`（专用探针：default 臂 shared=true/l2=true、双 0 臂 stats 归零）——本轮 Run3 只能降级为「与四臂 hash 结论自洽的旁证」，verdict 已如实声明局限。
- **修复**：本轮以「降级 Run3 证据等级 + 显式声明局限」处置；未改流程代码。
- **教训**：**env 门控的 A/B 判别实验：① 判别臂改用专用探针/独立 gradle 调用（`--no-daemon` 或 `-D` 显式透传）；② 结论里「同 hash」必须区分「env 生效且语义一致」与「env 未生效的死同值」——后者无判别力**。env 透传验证的廉价手段：判别臂打印 env 读数（如 L2 stats 归零与否，260903-13 探针即此形态）。
- **状态**：candidate（风险已识别并声明，机械验证待下轮 env 判别实验时落地）。

### 速查表追加行

| 错误 | 根因 |
|---|---|
| LL12 Run1 被同名覆盖 | A/B 多臂产物命名未含臂标识，静默覆盖无报错；采集前须核对命名清单（重跑 run1b + 四要素核对闭合） |
| LL13 daemon 吞 env 死判别 | gradle daemon 复用 JVM，客户端 env 不保证透传；「同 hash」须区分「语义一致」与「env 未生效死同值」，env 判别用专用探针打印读数 |

---

## 二、workflow-patterns 新发现候选（应用目标：`knowledge/discovered/workflow-patterns.md`，追加为发现 #32；INDEX 无需改——该文件无逐条 INDEX 或按主会话惯例同步）

### 发现 #32: gradle daemon 复用吞掉客户端 env——env 门控判别实验的死同值风险（260904-03）

- **发现时间**：260904-03。**发现者**：主会话 + worker（P2 Full 回归设计评审转化）。**置信度**：candidate（单轮风险识别，判别力损失实证为同 hash 无法归因；机制由 gradle daemon 复用模型支撑，待 env 探针复用再升）。**module**：通用方法论（判别实验设计 / 环境坑）。
- **来源定位**：`.investigations/lossless-accel/cmd-output/arm-compare-260904-03.txt` + `.investigations/lossless-accel/knowledge-drafts/260904-03/draft-p2full-verdict.md` 局限①。

#### 观察（现象）

P2 存档口径回归 Run3（cppReplace + WG_EST_SHARED=0/WG_EST_L2=0 反转）与 Run2（est 默认开）输出 SHA256 完全一致。预期判别逻辑：「env 反转生效 + 优化语义无关 ⇒ 同 hash」。但 Run3 经 gradle daemon 复用 run1/run2 环境执行——客户端 shell 设置的 env 可能根本没进 daemon 子进程，此时同 hash 是「env 未生效的死同值」，对「env 门控生效」**零判别力**。

#### 根因（机制）

gradle daemon 为复用 JVM，跨调用存活；启动新 client 传 `-D`/env 时，能否到达 `runServer` 子进程取决于 gradle 配置的透传链。判别实验的有效性依赖「自变量真被改变」（#20 原始形态是 CLI 参数死传，本条是 **env 经 daemon 的透传死点**）——同一判别力前提，另一条常见断裂路径。

#### 定位（怎么发现）

设计评审：核对「Run3 结论支不支持核心结论」时发现其判别力依赖未验证的 env 透传前提；检查 260903-13 专用探针（打印 shared/l2 状态 + L2 stats 归零）才是 env 生效的机械证据 → 本轮 Run3 降级为旁证。

#### 修复 / 如何利用（可复用判据）

1. **env 门控 A/B 的判别臂必须自带「env 读数输出」**（如门控计数器归零/置位、启动行打印实际生效值）——同 hash 前先看读数，读数没变即死判别，不得引用该臂。
2. 判别臂用 `--no-daemon`、独立 gradle 调用、或配置内显式透传，避免复用 daemon 的隐式 env 断层。
3. **结论措辞纪律**：凡「A==B 同 hash」型证据，同行声明「env/参数生效的证据是什么」——缺则降级为旁证。
4. 同族：#20（死参数制造假判别——本条为其 env/daemon 实例扩展）、#14（探针阶段同源性——「noise-only」要先验证对侧真静默，同型「前提未机械验证」）。

---

## 三、10 时间线条目（应用目标：`versions/1.20.1/docs/10-timewise-archive.md`，末尾追加）

```markdown
## 260904-03（翻默认后 block_probe 存档口径 Full 回归——遗留闭合 + 既有残差模式化）

> 承接 off-scan-cornerfix-verdict-260903-13.md:31-33 遗留（翻默认当时仅 Partial 声明）。采集 = 三 run（vanilla 参照 / cppReplace 默认开 / double-0 反转）+ 逐位对比。过程产物 `.investigations/lossless-accel/knowledge-drafts/260904-03/` + `cmd-output/cmp_full2-260904-03.txt` + `arm-compare-260904-03.txt`。

### ✅ 主结论（candidate，confirmed 待用户拍板）

- 翻默认后生产 dll 存档口径 **FULL 回归无回归**：default 臂 vs vanilla 参照 99.1526% 一致（13328/1572864 cell，16/16 chunk，aquifer stone↔water 互换族特征）；default==off 同 SHA256 与 260903-13 四臂零语义差自洽。§9.7 载体=WGB2 FULL 存档口径 4×4 单区域、覆盖面=16 chunk×98304 cell、与 260903-13 est 角列口径不可直接比数值。
- 生产 dll sha256 EC4A9AED…C8FC3（晚于全部生产改动）；参照四要素 + seed 三查核对通过。

### 🔍 既有残差登记（独立待查项，非本回归阻塞）

- stone↔water 等互换模式（ref=1→9 n=4165 等 top 对）为**新观察的量化模式**——量级已由 260903-14 记录（99.0107% 带），模式签名未记录过；机制候选：aquifer floodedness/流面高度/表面级联（未验证，下轮残差 y 分布直方图先行）。Run2==Run3 ⇒ 残差与 est 优化无关。

### ⚠️ 流程瑕疵与局限（诚实声明）

- Run1 参照被 Run2 同名覆盖后 run1b 重采（LL12，无数据混入）；Run3 走复用 daemon，env 透传未验证 → 「default==off 同 hash」只作旁证（LL13/#32 族，死判别风险）；两 run 均 [AQF-J] NPE 每 chunk（两臂同现，Java 探针侧既有噪声，非 cppReplace 引入）。

### 📌 记录指引

- 结论 → `.artifacts/lossless-accel/p2full-regression-verdict-260904-03.md`（本稿应用版）+ 260903-13 遗留项闭合指针。
- 残差登记 → 07 篇追加「存档口径残差模式化」小节（补充非取代 260903-14 记录）。
- 通用模式 → workflow-patterns 发现 #32（daemon env 死同值）；错误 → lossless-accel-errors.md LL12/LL13。
```
