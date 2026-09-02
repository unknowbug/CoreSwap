# CoreSwap 下一会话交接（260902-07 · 实际 2026-09-02 18:36–19:1x · H1 环1 证伪定案（judge PASS）· 材质/地形差四候选待判别）

> 本文件是唯一权威交接。**先读全文再动手**。工作区：**`E:\PYTHON\CoreSwap`** = 唯一主工作区。
> **本 session（260902-07）主线**：原计划 LAVAAUDIT 定性 → 熔岩海修复 → 回归；实际 **H1 环 1 被证伪——昨日 raw id 标注是错的，整环推翻，修复作废**。LAVAAUDIT 探针新增（lavaAudit 模式 + lavaSurfY + LAUIDMAP）→ v1 指标盲区暴露 → `[LAUIDMAP]` id 映射实锤（19319=blackstone 非 lava、5854=basalt 非 netherrack）→ judge PASS 转向。
> git HEAD = `a59749f`（**lib/dll 零改动**；本 session 改动全在 runtime/ 探针 + .tmp + 知识库/docs/.investigations）。

---

## 🔴 当前状态

### 0. git 基线与环境
- HEAD = `a59749f`；lib（WorldgenRust/src 非 bin-diag）零改动，dll 无需重编。
- **未提交 tracked 改动仍在**（昨日 260902-06 批 + 本 session 新增批）：.artifacts/index.yaml、knowledge/INDEX.md、knowledge/discovered/workflow-patterns.md（本 session 再追加 #13 补充案例）、knowledge/discovered/compiler-idioms.md（新增发现 #9）、versions/1.20.1/docs/09-multi-dimension.md（260902-06 定案节 + 本 session supersedes 注记）、versions/1.20.1/docs/10-timewise-archive.md（追加 260902-06/07 条）、.investigations/b1-downdrill/（facts ×3 + b1-errors.md 11 条 + judge 材料 + drafts）。untracked：架构计划文件、WorldgenRust/src/bin-diag/b1_selector_dump.rs。⚠️ 仓库根可疑 untracked 文件 `4`（疑似误产）仍未处置。
- runtime/（gitignored）累计改动：260902-06 的 4 个探针 mixin + drivers + BenchMod + coreswap.mixins.json；**本 session 新增**：ColProfProbeMixin lavaAudit 模式（v2 格式 `x,z,lavaSurfY,lavaTopY,n` + `[LAUIDMAP]` 一次性 id 映射）+ build.gradle colProfMode/colProfR -P→-D 映射。
- **.tmp 数据文件清单**（seed=8576294172403134396，区域 3200,3208 size=4+外扩环）：.tmp/blob-probe/lavaaudit-{v1,v2,v3}-*.log + colprof-firstsnap-out.txt；脚本 .tmp/b1_lavaaudit_cmp.py / b1_lavaaudit_cmp2.py / b1_colprof_firstsnap.py。

### 1. 权威结论现状
- **H1 环 1「Rust surface 熔岩海缺失（netherrack 实心兜底）」= 被证伪（judge PASS）**：COLPROF 10/25 列 diff 真相 = **V 黑石底（y=99 恒平）vs C 玄武岩底（y=100~104 贴地形）**，两侧均实心；LAVAAUDIT v2 全扫 11,443 公共列 air→lava 面向两侧均为零。09 篇已出 supersedes 注记（§15.4）。
- **环 2~5 作为现象保留**（转换面漂移 → delta origin 漂 → 级联/blob 放大；cfg 独立证据 delta y=111/119/121 vs 99），因果入口需重定位——「第一转换面不同」现指材质差（黑石 vs 玄武岩）及/或其它 surface 差。
- 已排除（维持）：H2 放大、随机序列 chunk 级/全局分叉、biome 过滤差、selector 采样差。修复方案（熔岩流体填充）**作废未执行**。

## 🟢 下轮工作清单（按优先级）
1. **判别探针：SURFACE 前/后逐列 dump**（材质序列 + biome id + 顶面 y，两轮 vanilla/cpp）——四候选判别的共同前置。
2. **四候选 fan-out（judge 设计）**：(a) surface rule 材质分支差（basalt deltas 表层 vanilla 黑石 vs cpp 玄武岩的条件/随机选择差）(b) biome 判定输入差翻转材质分支 (c) surface rule 随机序列/parity 差 (d) **前置地形形状差（NOISE/density 阶段列高度差——judge 指出若成立则 a/b/c 全降次生，先跑 (d)**）。候选非严格互斥，按「判别目标」设计、各自可独立排除；分叉即 fan-out，禁止主会话自推。
3. **LAUIDMAP 补 cpp 轮**（judge D⑤：现只跑 vanilla 轮，cpp 侧映射待核）。
4. **only_v=10/only_c=56 覆盖缺口查明**（judge D②：66 列差异未被现行口径解释）。
5. **用户遗留事项**：① 本批 tracked 改动提交（docs/knowledge/.artifacts/.investigations，commit message 英文动词开头）；② untracked 处置（.gradle-home/、.tmp/、.tmp-coreswap-data/、旧架构计划文件、**可疑文件 `4`**）；③ `.blocks` 双口径命名正式化（SURFACE=02B94092 当前 / FULL 备份在 .tmp-coreswap-data/vanilla_FULL_1DDE3B09.blocks，建议 `.blocks.surface` 类命名）。

## ⚠️ 纪律要点（新增/重申）
- **标注三查（本 session 新立，与 seed 三查并列）**：① raw id/枚举/魔法数首次解释前 MUST 建立映射（探针打印 LAUIDMAP，禁止数值直觉命名）；② 标注跨 session 传递带「已验证/未验证」标记，未验证续用前廉价独立验证；③ 机制链逐环追问「输入标注谁验证的」——**任何 NEXT_SESSION/台账里的方向性结论开工前先独立验证（§16.3），昨日本条纪律的失守正是 H1 整链作废的根因**。
- **judge 六项 CONCERN 备忘**：① lavaTopY 逐列已补（329/11443）② only_v/c 66 列缺口未解释（→工作清单 4）③ 结论限 3200,3208 区域（外推须声明）④ **口径三要素显式声明**：SURFACE 99.9423%（4×4 固体表面顶块口径）vs 内部转换面差（y=99~104，顶块以下）不可比 ⑤ LAUIDMAP cpp 轮待补 ⑥ 标注三查入开工检查项（已入本文件）。
- **跑探针固定命令模板（仍有效）**：用 PATH 的 `gradle`（无 gradlew wrapper）；后台命令一律绝对路径；每轮前三清理（Stop-Process java + 删 run\world + 删 .tmp\java-tmp\coreswap-native）；env GRADLE_USER_HOME/JAVA_TOOL_OPTIONS 同昨日；vanilla 轮 `gradle runServer -PbenchSeed=8576294172403134396 -PbenchOriginX=3200 -PbenchOriginZ=3208 -PbenchSize=4 -PblobProbe=true -PcolProfProbe=true`；cppReplace 轮加 `-PcppReplace=true -PcppWorldgenDir=E:/PYTHON/CoreSwap/versions/1.20.1/data/worldgen`（必须显式）；**colprof 必须搭配 -PblobProbe**（无 driver 不生成 chunk）；本轮新增 `-PcolProfMode=lavaAudit -PcolProfR=64`；seed 三查照旧。
- mixin 三坑（E-B1-3/4）仍有效：目标方法名从 loom-cache sources.jar 核实；类内状态 @Unique private；BufferedWriter 逐行落盘。
- 参照文件多口径并存——用前必查 hash（FULL=1DDE3B09 / SURFACE=02B94092）。
- 日期编号 YYMMDD-##（真实时间锚）；subagent 沙箱无 shell 无写盘：worker 出模板/草稿 → 主会话执行/应用。
- **gradle -P→-D 映射坑**（build-tooling 发现 #8 三犯史）：新增 -P 参数必同步 build.gradle 映射行，否则静默不生效。
