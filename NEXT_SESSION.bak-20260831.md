# CoreSwap 下一会话交接（2026-08-30 晚 · M13 confirmed + Rust 多世界落地）

> 本文件是唯一权威交接。**先读全文再动手**。工作区：**`E:\PYTHON\CoreSwap`** = 唯一主工作区。
> **本交接主线**：① transpiler 主线 **confirmed**（用户授予，游戏实跑验证）② Rust JNI 桥 + MOD 接入（C++→Rust 切换落地）③ **多世界 Phase A/B/C 全部完成**（nether 块级 74.04% + 游戏内下界接管）。

---

## 🔴 当前状态（直接接此）

### 0. git 基线
最新 HEAD 见 `git log -1`。本 session 提交链：
- `6dd9a5c` judge 复审 + 措辞修正（「逐位」→「<5e-7 浮点残差」）+ should-fix 清理
- `6e57100` 扩样探针 4 个 + 修前证据
- `ae3a3ad` **M13 flat_cache 量化修复** + 修后 6 份证据 + 台账/docs/07/index.yaml
- `411cae3` **Rust JNI 桥 + MOD 接入**（build.gradle 切 Rust dll）
- `be308e2` judge 审查（3×PASS）
- `26c1503` **M13 confirmed（用户授予）+ fillBlocks 防御校验 + JNI 端到端冒烟 PASS**
- `1102f58` **多世界 Phase A**：nether 双高度修复（23.77%→74.04%）+ 确定性修复（BTreeMap，漂移 2796 块归零）
- `9a3f7fa` **多世界 Phase B/C**：initDim JNI + CppBridge/Mixin 下界接管 + 末地保护 + processResources 根修
- （最新）多世界知识库落盘（docs/09 第八节 + multiworld-errors M3-M5 + index.yaml 4 条）

### 1. M13：transpiler flat_cache 量化语义 bug（**confirmed**，用户 2026-08-30 MOD 实跑授予）
- **bug**：`build/density.rs` 旧代码把 `flat_cache`|`cache_2d` 合并生成精确键 `transpiler_cache_2d`——「y 无关」被偷换成「逐点精确」。Java vanilla flat_cache = **4×4 格点量化缓存**（ChunkNoiseSampler.java L836-881：5×5 网格 y=0 预计算 + `(blockX>>2)` 量化索引）；cache_2d（L557-579）才是精确列键。运行时/C++ 本就正确。
- **为何生产不可见**：corner-only 采样域量化值≡精确值；只在「精确点诊断采样」暴露（judge 建议项 7 扩样抓出）。
- **修复**：拆分两分支，flat_cache 生成量化封装（i64 右移量化 x/z、y 遮蔽置 0、量化键缓存）。
- **修后**：内部点 ch0 diff 0.065101→**0.000000**；对比1 max_diff=0.000000；量化签名复刻（transpiler(1,1)=0.062840=runtime）；生产 98304 点 0.000000 / 块级 99.30% 持平 / **FULL 94.20%→94.27%**。
- **定位方法（可复用）**：corner 双线性自洽性检验（自洽侧=精确实现、不自洽侧=量化侧）+ 量化签名三特征（corner 0 / 内部点偏双线性 / y 线性轮廓）。4 探针链：exactpoint_verify → ch0_decompose → ch0_census → alignment_expanded。
- 证据：`.investigations/macro-layer-scout/`{analysis-flatcache-semantics.md（A 裁决，draft→judge 后可 candidate）, transpiler-errors.md#M13, review-m13-flatcache-jni.md, cmd-output/*_after_flatcachefix.txt×6}。

### 2. C++→Rust MOD 迁移（已落地，实跑验证）
- **Rust JNI 桥**：`WorldgenRust/src/jni_bridge.rs`（jni crate 0.22.4），7 个导出 `Java_wg_CppWorldgen_*`，对齐 C++ jni_bridge.cpp 语义（init 5 参映射 overworld 默认、fillBlocks 本地 buffer 拷回安全模式）。judge 逐方法 PASS。
- **build.gradle**：processResources 同步源已从 C++ dll 切到 `WorldgenRust/target/release/WorldgenRust.dll`（rename worldgen.dll）。
- **实跑**：`gradle runServer`（JAVA_HOME=jdk17）→ Done 3.598s，加载 1701888 字节 Rust dll，spawn region 生成，零崩溃。运行日志 `runtime/1.20.1/java/run/rust_runserver.log`。
- **构建**：`cargo build --release`（沙箱内需提权下载依赖，见下）→ MOD 侧 `gradle build`。

### ⚠️ 下一步（按优先级）
1. ~~confirmed 授予~~ ✅ 用户 2026-08-30 授予（index.yaml 10 条 + docs/07 M12/M13 + 台账 M13 + 判定报告）。
2. ~~JNI 桥端到端冒烟~~ ✅ JniProbe 16 chunks 14.01ms/chunk，95.96% vs SURFACE 参照（`cmd-output/jni_smoke_rustdll_20260830.txt`）。
3. ~~fillBlocks outs.length 校验~~ ✅ 已补。
4. ~~多世界 Phase A/B/C~~ ✅ 完成（见「多世界」节）——用户可进游戏经传送门实测下界。
5. **多世界收尾（可选后续）**：lava 流体填充（C++/Rust 同未解）、底部基岩 VerticalGradient 反锚序移植、末地引擎（Mixin 保护已就位）。
6. **FULL 差距归因**（用户已示意挂起，除非性能回退才重启）：transpiler 94.27% vs 基线 95.40%。
7. 运行脚本：`WorldgenRust/run_rust_client.ps1`（`-Server`/`-Rebuild` 开关，内置 JDK17）。

### 🌋 多世界（2026-08-30 Phase A/B/C 完成）
- **Rust 侧**：`create_for_dim(seed, dir, settings, biomeParams, worldHeight)` 参数化；nether 块级 **74.04%**（超 C++ 71.97%），双高度（noise 128/world 256）+ 确定性（BTreeMap）两修复；overworld 零回归。
- **MOD 侧**：`initDim` JNI 通路 + CppBridge netherHandle/fillChunkNether + Mixin 下界接管（min_y=0/h=256 且 netherActive）+ 末地保护（biomeSource 缓存反射）+ buildSurface 按维度收紧。
- **实证**：`initNether enabled=true` + `populateNoise(nether) intercepted`（rust_nether_test4.log 摘录 multiworld-port/cmd-output/）。
- **遗留**：lava 流体（y=32..63 带 7.9%）、基岩反锚序、末地引擎——详见 docs/09 第八节。
- **台账**：`.investigations/multiworld-port/multiworld-errors.md`（M1-M5）。

### ⚠️ 本 session 新立教训（已入台账，防重走）
- **「缓存≠透明」**：cache 类节点先问「命中值第一次在哪算」——flat_cache 是量化采样器非透明缓存；**「y 无关≠逐点精确」**。
- **corner-only 生产域掩盖缓存语义 bug**：诊断（精确点）与生产（corner-only）域互补。
- **探针判据必须与被测子机制同构**（decompose 实验 2 判据过强差点误判）。
- **fan-out 双 worker 超时的降级路径**：主会话采集原始数据落盘 → core-worker 判读（采集/判读分工保持）。
- **主会话 fan-out 等待期间自推深钻 = deadloop**（用户点名批评，-288 模式复发）——分叉即 fan-out，等待期间只做数据采集不做假设推演。

---

## 一、环境/工具（本 session 实证）
- **cargo 网络被沙箱 TLS 拦**（SEC_E_NO_CREDENTIALS）——下载依赖必须提权（danger-full-access）跑 `cargo add/build`；编译本身 workspace 内也建议提权跑（registry 写入）。
- **gradle**：`D:\gradle\gradle-8.13\bin\gradle.bat`，**必须 JAVA_HOME=jdk17**（`E:\PYTHON\MC\tools\jdk17\jdk-17.0.20+8`），PATH 默认 java 是 24。fabric 依赖已缓存（.gradle）。
- **dumpbin 不可用**——验 PE 导出表用 PowerShell 解析（本 session 有现成脚本模式，见对话记录）。
- MOD 工程：`runtime/1.20.1/java`（fabric-loom 1.10.5 / MC 1.20.1 / yarn）；server.properties level-seed=-2032795982907864146。
- 残留 java 进程：runServer 停止后确认 `Get-Process java`（本次无残留）。

## 二、关键文件索引
- `transpiler-errors.md`（M1-M13 + 速查表）/ `analysis-flatcache-semantics.md`（A 裁决）/ `review-m13-flatcache-jni.md`（judge 3×PASS）/ `review-transpiler-prod-recheck.md`（补证复审）
- 探针：`WorldgenRust/src/bin/transpiler_{alignment_expanded,exactpoint_verify,ch0_decompose,ch0_census}.rs`
- JNI 桥：`WorldgenRust/src/jni_bridge.rs`；`build/density.rs` flat_cache 拆分分支
- MOD：`runtime/1.20.1/java/build.gradle`（Rust dll 同步）+ `run/rust_runserver.log`

## 三、铁律提醒（常驻）
探针/参照三查（seed/坐标/文件）· 性能结论多次运行取均值 · 排障先核对探针前置条件（set_blended_noise）· 结论性 docs 必须 subagent 产出草稿 · confirmed 只有用户授予 · 分叉即 fan-out（禁主会话自推深钻）· commit 前扫描 docs 时间线归口。

## 📦 发布（2026-08-30）
- **coreswap-1.20.1-1.0.19-beta** 已发布（pre-release/beta 通道）：https://github.com/unknowbug/CoreSwap/releases/tag/coreswap-1.20.1-1.0.19-beta
- 资产：coreswap-1.20.1-1.0.19-beta.jar（内含 Rust dll 1710592B）；tag 推送；master 已推
- **runtime/ 已改为本地私有不入库**（.gitignore /runtime/，历史提交仍含旧版本——如需彻底清除需 rewrite history，未做）
- README/README.zh-CN 已重写为 Rust 时代 + 实测数据口径（~1.2× Java、14ms/chunk、95.40/94.27/74%、<5e-7）
- **常设要求（用户拍板）**：以后每版默认 Fabric + Forge 双支持。当前路线 = 单 jar + Sinytra Connector（1.0.19-beta jar 结构与 Connector 实测过的 1.0.18 逐条目一致，兼容结论成立）；原生 Forge jar 为可选独立立项
- **提权纠正（用户指出）**：默认不提权！实测 commit、cargo build、探针运行、gh 读操作免提权；**git push 需提权**（沙箱拦凭据，匿名只读 ls-remote 不算数）；gh 写操作待验证；仅 cargo add/首次下载依赖需提权（cargo TLS 栈 SEC_E_NO_CREDENTIALS）。惯性提权 = 错误泛化一次性网络需求到所有操作。

