# scout-1c：1.21.6 接入触点清单 + 项目结构审计

> 角色：recode.scout（只读勘探）。置信度：**draft**。日期锚：Get-Date = 2026-09-08 13:20（260908）。
> 范围：A = 新增 versions\1.21.6\rust 薄壳 + 1.21.6 数据的必触碰点；B = 项目结构事实盘点（只列事实与问题，不给修复方案）。

---

## A. 新版本接入触点清单（假设：新增 versions\1.21.6\rust + 1.21.6 数据）

### A.0 前置底册
已通读 `worldgen-core/data-driven-boundary.md` 全文（55 行）。要点：block id / density / biome / feature / carver tag 展开均已数据驱动（JSON 优先 + fallback 兜底）；多世界参数化入口 `WorldgenHandle::create_for_dim(seed, wg_dir, settings_name, biome_params_file, world_height)` 已存在；维度参数（min_y/height/sea_level/aquifers）从 settings JSON 读取，非硬编码。

### A.1 触碰点清单表

| # | 文件/位置 | 改动类型 | 是否数据驱动边界内 | 说明 |
|---|---|---|---|---|
| 1 | `Cargo.toml`（仓库根，workspace members） | 增行 `versions/1.21.6/rust` | 边界外（构建配置，必须手改） | 现仅 worldgen-core + versions/1.20.1/rust 两条 |
| 2 | `versions/1.21.6/rust/Cargo.toml` | 新建（复制 1.20.1 薄壳） | 边界外（薄壳本身） | 包名 `worldgen` → 产物 `worldgen.dll`；依赖 path 指向 `../../../worldgen-core`；edition 2024 |
| 3 | `versions/1.21.6/rust/src/lib.rs` / `src/jni_bridge.rs` | 新建（复制 + 适配） | **边界外（硬编码风险点）** | jni_bridge.rs 内 `const BLOCK_COUNT: usize = 16*16*384`（overworld 高度硬编码）；`Java_wg_CppWorldgen_init` 注释明确 world_height=384 传参；若 1.21.6 维度高度不变则可原样复制，变了必须改。薄壳 JNI ABI（`Java_wg_CppWorldgen_*` 6 方法）与版本无关 |
| 4 | `runtime/1.20.1/java/build.gradle`（Java mod 侧 processResources） | 新版本需对应新 runtime 工程（或改造现有） | 边界外 | 现为 fabric loom `com.mojang:minecraft:1.20.1` + yarn 1.20.1 + fabric-api 0.92.0+1.20.1；dll 硬编码绝对路径 `E:/PYTHON/CoreSwap/target/release/worldgen.dll`（workspace 根 target 共享 → **多版本薄壳并存时同名 dll 会互相覆盖**，processResources inputs.file 也指向同一路径）；`benchOut` 默认硬编码 `versions/1.20.1/data`；ErrorFile 路径硬编码 |
| 5 | `versions/1.21.6/data/worldgen/data/minecraft/worldgen/**` | 新建数据集（server jar 重新提取） | **边界内（纯数据，换文件即可）** | 参照 1.20.1 布局：noise_settings / density_function（按维度子目录 overworld/nether/end）/ noise / biome / configured_feature / placed_feature / configured_carver / tags/blocks / structure* 等 |
| 6 | `versions/1.21.6/data/blocks.json` | 新建（server jar 提取） | 边界内 | block id 单一事实源 |
| 7 | `versions/1.21.6/data/biome_params.json`（+ 可选 `biome_params_nether.json`） | 新建 | 边界内 | biome 气候参数 |
| 8 | `versions/1.21.6/data/noise_params.json` | 新建 | 边界内 | 噪声参数 |
| 9 | tags 提取通道：`.investigations/tag-datadriven-260907-04/extract_tags.py` | 重跑（指向 1.21.6 jar） | 边界内 | carver replaceable / feature tag 的 JSON 数据源生成脚本（脚本本身版本无关） |
| 10 | `versions/1.20.1/docs/08-version-migration.md` | 参照文档（流程清单） | — | 记录了通用迁移流程；其中引用的 `data/mcsrc_<ver>/`、`data/diag_full.py`、`build-msvc` 等路径已部分过时（见 B.5） |

### A.2 worldgen-core 内版本相关硬编码点（grep 实证）

**生产代码（影响 1.21.6 与否取决于 MC 数据变化）：**

| 位置 | 硬编码 | 数据驱动边界内？ |
|---|---|---|
| `worldgen-core/src/api.rs:11-12` | `const MIN_Y: i32 = -64; const HEIGHT: i32 = 384;`（C ABI 层默认值） | 边界外，但已被 create_for_dim 参数化旁路（settings JSON 优先） |
| `worldgen-core/src/worldgen_handle.rs:211-216` | `min_y = -64 / noise_height = 384` 作为 **settings JSON 缺字段时的 unwrap_or 兜底** | 半边界：settings 正常时数据驱动；缺字段静默落 overworld 值 |
| `worldgen-core/src/worldgen_handle.rs:242` | `density_function/overworld/base_3d_noise.json` 路径含 `overworld` 字面量 | 边界外（df_ns 分支逻辑） |
| `worldgen-core/src/worldgen_handle.rs:341/401` | `if df_ns == "overworld"` → 代码 surface 规则（build_overworld_rule）| 设计决策（overworld 代码规则已验证），非数据驱动 |
| `worldgen-core/src/density_builder.rs:127,221` | df_ns 默认 `"overworld"`；YClampedGradient 示例 -64..320 | 边界外（默认值可被 settings 覆盖） |
| `worldgen-core/src/surface_rules.rs:1059` | `bedrock_floor true_y: -64, false_y: -59`（build_overworld_rule 内） | 边界外（overworld 代码规则；1.21.x 若 bedrock 公式不变则无需改） |
| `worldgen-core/src/surface_rules.rs:1187` | 已有注释「跨版本风险点：mult 硬编码 0」 | 自标注升级点 |
| `worldgen-core/src/carver.rs:198-208` | `overworld_carver_replaceables` tag JSON 优先 + fallback 硬编码表 | 边界内（换 tag JSON 即可；fallback 为 golden 基准） |
| `worldgen-core/src/feature.rs:74` | `"minecraft:base_stone_overworld"` expand_tag fallback | 边界内（同上） |
| `worldgen-core/src/light/mod.rs:16-18` | `WORLD_H: usize = 384`（光照域常量，y 0..384 = 世界 -64..319） | **边界外，纯代码硬编码**——1.21.6 若高度不变无需改；多世界/新高度需参数化 |
| `worldgen-core/src/generated/*`（dfc_cpu_tables.rs / dfc_cpu_split.rs / vanilla_density_functions.rs） | `MIN_Y = -64`、`split_double(..., -64, 8384...)`、生成代码内 -64..320 网格 | **边界外 + 重生成通道耦合**：由 transpiler/dfc 从 overworld density JSON 生成（build.rs / 生成脚本通道），1.21.6 density 函数若有变化需重跑生成——这是底册外最大的隐性硬编码面 |
| `legacy_random.rs:89` | 统一随机源枚举（overworld=Xoroshiro / legacy=Legacy） | 机制层，非数据 |

**诊断 bin（不参与默认构建，但数量庞大）：** `src/bin/`（93 个）与 `src/bin-diag/`（77 个）中大量 `DensityBuilder::new(seed, -64, 384)` + **绝对路径硬编码** `E:\PYTHON\CoreSwap\versions\1.20.1\data\worldgen\...`（如 bin\biome_hot.rs:16、bin-diag\aquifer_probe.rs:15）。接 1.21.6 时这些**不需要全改**（按 #13a 纪律 bin-diag 不参与构建），但新写 1.21.6 对拍探针应参数化 wg_dir。

### A.3 重生成通道小结
- 数据层：server jar → `versions/<ver>/data/worldgen/...` + blocks/biome_params/noise_params JSON + extract_tags.py。
- 生成代码层：`worldgen-core/src/generated/` 三文件 ← transpiler/dfc 生成链（overworld density JSON 驱动），1.21.6 density 变化时必须重生成，生成器代码固定（符合数据驱动定义）。
- 构建：`cargo build --offline -p worldgen --release`（-p worldgen 指薄壳；多版本需 -p 消歧或改包名）。

---

## B. 项目结构审计（事实与问题清单，无修复方案）

### B.1 仓库根散落文件（逐一判定）

| 文件 | 判定 |
|---|---|
| `Cargo.toml` / `Cargo.lock` | 合理（workspace 根） |
| `AGENTS.md` / `README.md` / `README.zh-CN.md` / `NEXT_SESSION.md` | 合理（根级约定位置；NEXT_SESSION 仅本地不入库） |
| `.gitignore` | 合理 |
| `beard_pow_ref.obj` / `block_probe.obj` / `md5.obj` / `mlp_probe.obj` / `noise_check_cpp.obj` / `rust_ref_check.obj`（共 6 个 .obj，~450KB） | **问题**：C++ 时代编译中间产物残留在仓库根明面（违反「临时文件唯一区」精神；C++ 已归档，这些 obj 无现役用途） |
| `crash-coreswap-20260903-*.txt` ×4、`crash-coreswap-20260904-*.txt` ×2 | **问题**：崩溃日志散落根目录（versions/1.20.1 下还有 3 个同类：20260815 ×3）；按 crash 铁律应有归口目录 |
| `forge-installer.jar.log`（1.2MB） | **问题**：运行日志残留根目录 |
| `reasonix.toml` / `.reasonix/` | **问题**：Reasonix 宿主已归档（AGENTS 明言停止作为运行时源），根目录仍留配置与目录 |
| `__pycache__/` | **问题**：Python 缓存目录在根明面 |
| `target/` / `.gradle/` / `.gradle-home/` / `.github/` / `.dsh/` / `.git/` | 合理（构建/宿主/CI 元数据） |
| 目录：`worldgen-core/` `versions/` `scripts/` `knowledge/` `runtime/` `protocol/` `spec/` `templates/` `tools/` `framework-proposals/` `.artifacts/` `.investigations/` `.tmp/` | 归属大体合理；`protocol/`（1 项）与 `spec/`（2 项）内容极少、职责相邻（验证协议引用），边界模糊待裁决；`.tmp-coreswap-data/`（10 项）与 `.tmp/` 职责区分不明 |
| `docs/`（根） | **问题**：根 `docs/` 为空目录（AGENTS §八.4 曾记录「docs 目录变成文件」的坑史） |

### B.2 .tmp/ 现状（明面残留盘点）
- 47 个子目录 + ~300 个散文件。**部分带日期标签符合唯一区纪律**（a-group-260907-05、grove-powdersnow-260908-07 等），但**大量无日期标签、无任务前缀**（b1_*.py ×40+、amp_*.py、compare_*.txt、javap 系列）——早期堆积未按 260901-03 纪律命名。
- `.tmp/` 内混有 **exe/pdb 二进制**（qaq1_*.exe/pdb ×16+、gpu_*.exe、res12_probe.exe 等）——与「Rust 诊断 bin → bin-diag、一次性脚本 → .tmp」的分工不符（exe 是 C++/rustc 编译产物落错区）。
- `jar-check` / `jar-check22`、`feature-parity-260905-05..10` 连号 5 个目录、`amp-cpp-save` / `amp-cpp-save200` / `amp-cppref` / `amp-cppref200` / `amp-vanref200` 等近似名簇——快照类目录无清理标记。
- `com/`、`net/`、`META-INF/`、`sources.jar`、`jnssrc/`、`mcp/`——jar 解包残渣直接铺在 .tmp 顶层。

### B.3 worldgen-core 正式 bin 清单（#13a 纪律核对）
- `src/bin/` = **93 个**正式 bin（cargo build --release 全量编译必须绿）。
- `src/bin-diag/` = **77 个**诊断 bin（不参与默认构建，符合 #13a 隔离）。
- **问题/事实**：93 个正式 bin 远超「随库维护的最小集」直觉（aquifer_*/bench_*/perf_probe2..5/perf_fresh/transpiler_* 系列等明显是历史排查探针）；发版虽可用 `--lib` 绕开，但「全量绿」发版前检查的成本随 bin 数线性膨胀。正式/诊断边界是否有一次系统性核对，.investigations 未见对应记录（抽查 workspace-split-regression 目录未确认，存疑）。
- `worldgen-core/` 根另有散落：`beard_probe_box.txt` 等 beard_probe_*.txt ×5、`dll_test.c`、`jni_dll_test.c`、`rust_jni_bridge.cpp`——**问题**：C++ 遗留源文件与探针输出残留在 crate 根（rust_jni_bridge.cpp 在 docs 中被引用为 C++ JNI 桥历史参照，但 crates 根非其归属位）。

### B.4 时间线归口抽查（versions/1.20.1/docs vs knowledge/）
- 主题篇现状：01-09 主题篇 + 10 时间线（370KB）+ **11/12/13 三篇新增主题篇**（features-stage / lighting / feature-parity）——编号已超原 01-09 约定，README 是否同步未核（存在即可，但归口约定需确认是否已扩）。
- 07-block-pipeline.md = **130KB**：主题篇内出现时间线式章节（如 L696「Rust worldgen 作为 mod 运行」带日期叙事、L316「build.gradle dll 同步源错误」错误复盘叙事）——与「时间线只进 10」铁律存在张力（07 篇历史上是垃圾桶前科篇，09 曾 78KB 被点名）。
- 09-multi-dimension.md = 73KB，同为膨胀主题篇。
- 10-timewise-archive.md 370KB / 3000+ 行，条目带 ✅/❌ 状态标注、supersedes 链、Get-Date 锚定——**归口纪律执行良好**（抽查 260903-09 → 260907-01 条目，日期锚与 git 时间线一致）。
- knowledge/ = INDEX + SUBAGENT-GUIDE + discovered ×7，通用模式归位正确，未发现项目特定结论混入。

### B.5 docs 相对路径悬空引用抽查（5 例）

| 文档:行 | 引用路径 | 存在？ |
|---|---|---|
| 08-version-migration.md:52 | `data/diag_full.py` | **悬空**（versions/1.20.1/data/ 下无此文件） |
| 08-version-migration.md:79 | `E:\PYTHON\CoreSwap\build-msvc`（CMake 构建） | **悬空**（根下无 build-msvc；C++ 已归档，属历史口径未标注） |
| 01-architecture.md:31-46 | `cpp/worldgen/src/*.h/.cpp` ×11 处 | 存在（versions/1.20.1/cpp/worldgen/src/），但 01 篇通篇仍以 C++ 为映射主体，Rust 主线后未重写（陈旧而非悬空） |
| 04-aquifer.md:136 | `.investigations/-288-unclosed/beardifier-verdict.md` | 存在 |
| 05-ore-vein.md:88 | `.investigations/-288-reopen/summary-final.md` | 存在 |

结论：**真正悬空 2 处（均在 08 篇，C++ 时代口径），陈旧口径 1 篇（01 篇），锚定的 .investigations 引用抽查 2/2 有效**。

### B.6 .investigations/ 顶层目录盘点（46 个）
- **命名一致性差**：三种风格并存——① 日期标签式 `*-260902-10` / `gpu-reentry-260908-02` / `tag-datadriven-260907-04`（新纪律）；② 无标签任务名 `carver-port` / `perf-rework` / `residual12`；③ 异常名 **`-288-reopen` / `-288-unclosed`**（以连字符开头，起源于 chunk 坐标 -288，PowerShell/Test-Path 下极易踩路径坑——B.5 抽查已实证可被引用）。
- 近义/成对目录：`-288-reopen` vs `-288-unclosed`、`residual-1830` / `residual12` / `residual13` / `v5-residual` / `8576-24blocks`（数字系残留课题链）、`feature-parity` / `features-port` / `feature-pipeline` 三近名。
- `reference/`、`000-架构设计/`（中文命名，唯一一个）——风格离群。

### B.7 versions/1.20.1 顶层杂项
- `data/` 目录名义上是「数据集」，实际混入：**C2ME-fabric 完整 gradle 工程**、`mc_src_extract`、`crash_0831/crash1/crash2...` 崩溃目录、**~300 个对照 dump/bin/日志/py 脚本平铺在 data/ 顶层**（vanilla_*.blocks 参照文件与 javap_*.txt 反汇编混放）——数据集与排查现场未分离。
- 顶层 3 个 crash-coreswap-20260815-*.txt + `.investigations-out/`（investigations 输出到了 versions 层，与根 .investigations 并存，归口分裂）。

### B.8 探针工程对版本的耦合（补充 A 节）
- Java 探针/载体 = `runtime/1.20.1/java/`（fabric loom 工程，rootProject name 'worldgen-bench'）：Minecraft/yarn/fabric-api 三坐标硬编码 1.20.1；processResources dll 绝对路径 + benchOut 指向 versions/1.20.1/data（见 A.1 #4）。
- `CoreSwapFixHelper.java` 在 runtime/1.20.1/java/src/main/java/wg/bench/（现役）+ versions/1.20.1/data/CoreSwapFixHelper.java.bak（**问题**：.bak 副本留在 data 顶层）。
- `.investigations/multiworld-port/MCP-Reborn/` 是另一个完整 gradle 工程藏在 investigations 里（历史参照，位置离群）。

---

## 待深入点（交主会话裁决）
1. `src/generated/` 重生成通道的具体脚本/命令入口未在本次勘探内逐一定位（build.rs 存在，生成链文档化程度待查）——1.21.6 接入的最大隐性工作量所在。
2. 多版本薄壳并存时 `target/release/worldgen.dll` 同名冲突——是构建布局问题，另立项。
3. 07 篇 130KB / 09 篇 73KB 的主题篇瘦身归口（知识库压实，另立项）。
