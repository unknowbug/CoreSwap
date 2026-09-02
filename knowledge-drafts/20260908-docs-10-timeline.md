# 草稿：docs/10-timewise-archive.md 追加 2026-09-08 时间线条目（subagent 产出，主会话应用）

> **应用位置**：`versions/1.20.1/docs/10-timewise-archive.md`——「### ✅ 拍板回填（2026-09-07）」之后（文件末尾）追加。追加不覆盖，每条带状态标注。

---

## 2026-09-08（nether-save-full 课题续：双跑修复 + soul V3）

### ✅ 一、句柄级 wg_set_flags 修复双跑（judge PASS，candidate）

- 承接 09-07 P2 矿石归因 judge CONCERN（`WG_SKIP_*` env 门控进程全局，勿全局默认翻转）。
- 改动：`worldgen_handle.rs` AtomicU32 flags（bit0=SKIP_CARVER bit1=SKIP_FEATURES bit2=SKIP_SURFACE，**OR-env 语义**，0=回落 env 兼容）+ `api.rs` wg_set_flags/wg_get_flags + jni_bridge + Java CppWorldgen/CppBridge（**默认 mask=0b011**，`-Dcoreswap.rust.stages` 可覆盖）。
- 回归：3 轮全新 run 全部 **94.4241%**（990108/1048576，seed B，nether 4×4@3200,3208，FULL 参照，ReadWorldProbe 存档口径；修复前 93.8988%）；ore per-id quartz 4478→2125 / gold 1525→739 / magma 3814→1979 = **SKIP_FEATURES 消融值**（ref 邻域 1992/728/1533）——与消融实验因果链重复，比区间判据更强。
- 设计：`.investigations/nether-save-full/design-wg-set-flags-20260908.md`；judge：`.artifacts/.c2-p2-ore-attribution/review-judge-20260908.md`；日志：cmd-output/flags-regression-run4/5/6.log。

### ✅ 二、V3 结构对拍（draft，Degraded）

- nether.json surface_rule 全 10 种节点类型 Rust 解析器全支持、7 顶层分支逐节点一致。
- ❌ 签名 B（soul_soil 子分支失效）/ C（floor 侧 soul_sand_layer「分支缺失」）的**结构差解释不成立**——「分支缺失」假说被否定。
- 归因指向：①运行时输入差（V4：生产链路 soul 分支 ctx dump vs probe 输入对差）；②biome 分类层（签名 A 同源，V5）。产物：`.artifacts/.b2-soul/v3-structure-diff.md`。🔍 V4/V5 未做。

### ⚠️ 三、环境坑 E10（详录 `.investigations/nether-save-full/nether-save-errors.md`）

- 强杀 gradle daemon 后所有 gradle 调用报 `Failed to load native library 'native-platform.dll'`——根因 = `C:\Users\NDark\.gradle\native\**\native-platform.dll.lock` 拒绝访问（非 dll 本身，--stacktrace 定位到 .lock 文件级拒绝）；删锁被沙箱硬拒（工作区外，升级亦被拒）；最终修复 = **GRADLE_USER_HOME 指向工作区 `E:\PYTHON\CoreSwap\.gradle-home`**。

### 🔍 四、参数试错过程（run1-3 空跑教训）

- run2/run3 两次因 **bench 参照文件名四要素不一致**空跑（cppReplace + readWorldProbe + blockProbeDimension=nether + bench 参数须与 ref 文件名四要素一致）——完整命令模板以 flags-regression-run4.log 对应调用为准固化。

### 📌 记录指引

- 结论 → 09 篇追加「句柄级 wg_set_flags 修复双跑（candidate）」+「V3 结构对拍（draft，Degraded）」两小节，草稿 `knowledge-drafts/20260908-docs-09-dualrun-fix-and-v3.md`。
- 通用模式 → `knowledge/discovered/build-tooling.md` 发现 #7（E10：GRADLE_USER_HOME 沙箱策略 + 参照四要素核对），草稿 `knowledge-drafts/20260908-build-tooling-faxian7.md`。
- 错误 E10 五段式 → `.investigations/nether-save-full/nether-save-errors.md` 追加 + 速查表加行。
- 状态：双跑修复 candidate（judge PASS）；V3 draft（Degraded）；confirmed 留用户。
