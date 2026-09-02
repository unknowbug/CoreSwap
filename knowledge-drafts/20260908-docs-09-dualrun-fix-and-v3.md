# 草稿：docs/09-multi-dimension.md 追加两小节（双跑修复 + V3 结构对拍，subagent 产出，主会话应用）

> **应用位置**：`versions/1.20.1/docs/09-multi-dimension.md`——「soul sand valley 归因三签名（B2 定稿，2026-09-07）」节之后（文件末尾）追加。追加不覆盖。
> **前置核对**：09 篇当前至「soul 三签名」节，其「下一步」第 1 项即 V3（本稿第一小节闭合该项）；双跑修复承接「矿石归因定论」节 judge CONCERN（env 门全局 → 句柄级 flag），承接关系自洽。
> **状态纪律**：只写草稿，不改 docs 正文，不标 confirmed。

---

## 句柄级 wg_set_flags 修复 cppReplace 存档链路 Rust features/carver 双跑（candidate，2026-09-08；judge PASS）

> 承接「矿石归因定论」节结论 4 的 judge CONCERN（`WG_SKIP_*` 为进程全局 env 门控，勿全局默认翻转）。修复验证分层 **Partial**（存档口径端到端 + ore per-id 消融值佐证，非逐位 Full）。§9.7：94.4241% 为存档口径，与 SURFACE/纯 Rust 口径不可比。

### 修复内容

- **worldgen_handle.rs**：`AtomicU32 flags` 句柄级标志位——bit0=SKIP_CARVER、bit1=SKIP_FEATURES、bit2=SKIP_SURFACE；**OR-env 语义**（句柄 flag 与 `WG_SKIP_*` env 任一置位即生效）；flags=0 时回落 env 兼容行为（存量调用方零影响）。
- **api.rs**：新增 `wg_set_flags(handle, mask)` / `wg_get_flags(handle)`。
- **jni_bridge + Java**：`CppWorldgen` / `CppBridge` 透传；存档链路默认 **mask=0b011**（SKIP_CARVER|SKIP_FEATURES，即存档链路不再双跑），可用 `-Dcoreswap.rust.stages` 系统属性覆盖。

### 回归验证（seed B = 8576294172403134396，nether 4×4 @3200,3208，FULL 参照，ReadWorldProbe 存档口径）

| 项 | 数字 | 判读 |
|---|---|---|
| 修复前（消融轮） | 93.8988% | 存档 = Rust+Java features 双跑基线 |
| 修复后（3 轮全新 run） | **全部 94.4241%**（990108/1048576） | 与消融轮 SKIP_FEATURES 值逐位一致 = 双跑通道闭合 |
| ore per-id 直接佐证 | quartz 4478→2125 / gold 1525→739 / magma 3814→1979 | **= SKIP_FEATURES 消融值**（ref 邻域 1992/728/1533），三族矿石全部落回 ref 邻域 |

- 判据：修复后值 94.4241% 与此前手工 `WG_SKIP_FEATURES=1` 消融值**逐位相同**，且 3 轮全新 run 无散布——比「区间不重叠」判据更强（重复了消融实验的因果链）。
- 日志：`.investigations/nether-save-full/cmd-output/flags-regression-run4/5/6.log`。

### 设计与审查记录

- 设计文档：`.investigations/nether-save-full/design-wg-set-flags-20260908.md`。
- judge 意见：`.artifacts/.c2-p2-ore-attribution/review-judge-20260908.md`（PASS，建议 candidate）。
- confirmed 留用户拍板。

### 口径声明（§9.7 三要素）

- 载体：MCA 存档直解（ReadWorldProbe 口径）vs vanilla FULL 参照；覆盖面：4×4 chunk 全高度，seed B。
- 可比性：与消融轮 94.4241%（SKIP_FEATURES）/ 基线 93.8988% 同口径可比；与 SURFACE 口径 77.49%、纯 Rust 口径 77.43% 不可比，分列。

### 状态

- **candidate（judge PASS）**；过程 → 10 时间线 2026-09-08 条。

---

## V3 结构对拍：nether surface_rule 解析器全节点一致，「分支缺失」假说否定（draft，Degraded，2026-09-08）

> 承接「soul 三签名」节下一步第 1 项（V3 结构对拍，零成本最高优先）。验证分层 **Degraded**（静态结构对拍，无运行时证据），MUST 声明降级。

### 对拍结果（排除式论证）

- **解析器能力面**：nether.json surface_rule 的全 **10 种节点类型** Rust 解析器全部支持；**7 个顶层分支逐节点一致**（节点类型/参数/嵌套结构与 JSON 语义等价）。
- **排除结论**：
  - ❌ **签名 B（soul_soil 子分支失效）**与 **签名 C（floor 侧 soul_sand_layer「分支缺失」）**的**结构差解释不成立**——结构层逐节点一致，不存在「分支没解析出来」。
  - → 签名 C 的「分支缺失」假说被否定；签名 B 的机制必须到**运行时输入**找。
- **归因指向（候选，未验证）**：
  1. **运行时输入差**（V4：生产链路 soul 分支 ctx dump vs probe 输入对差）——probe 的 V2 采样路径与生产链路输入可能不同源；
  2. **biome 分类层**（签名 A 同源，V5 边界带对比）。

### 口径声明（§9.7 三要素）

- 载体：静态结构对拍（nether.json surface_rule vs Rust 解析规则树）；覆盖面：7 顶层分支 × 10 节点类型全量。
- 可比性：结构层一致**不构成**运行时行为一致的证据（Degraded）——V2 三签名的运行时现象不被本节解释，仅排除结构差候选。

### 状态

- **draft（Degraded）**：排除结论（结构差不存在）数据直读可信；归因指向两候选均未验证。下一步 V4（ctx dump 对差）/ V5（biome 边界带）。
- 产物：`.artifacts/.b2-soul/v3-structure-diff.md`。过程 → 10 时间线 2026-09-08 条。
