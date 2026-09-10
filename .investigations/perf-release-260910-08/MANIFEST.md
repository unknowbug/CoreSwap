# perf-release-260910-08 — 1.0.28 性能基准 / 引擎因子复核 / 跨版本报告归因

> 实际时刻：**2026-09-10 23:45 – 2026-09-11 01:15**（本地；块标签沿用起始日 260910-08，命名前已 `Get-Date` 锚定）。
> 角色：主会话（bench 执行 + 落盘 + 代码一手核对）；代码可达性审查 = subagent `ff43da97`（只读 read/grep，无 shell）。
> 上游：`.investigations/perf-reg-260910-06`（R3 异步化）、`perf-reg-260910-07`（迁移）、`.artifacts/perf-reg-260910-0{6,7}`（两份已 confirmed verdict）。
> 错误台账：`errors-260910-08.md`（E1-E6，其中 E1 为本块最高价值）。

---

## 1. 载体与环境（§9.7 可比性声明）

| 要素 | 值 |
|---|---|
| 载体 | Fabric 1.20.1 开发服务端（`gradle :runServer`），**无客户端渲染、无客户端进程** |
| 工作负载 | Chunky 预生成：`chunky quiet 10` / `world minecraft:overworld` / `center -48 -11` / `radius <R>` / `start` |
| 世界 | 每臂全新：删 `runtime/1.20.1/java/run/world` + `run/config/chunky/tasks`；seed `417950215108767439`（写 `server.properties`） |
| 机器 | Ryzen 9 7845HX（12 物理/24 逻辑），Windows 11；`build.gradle` **未设 `-Xmx`** ⇒ JVM 默认 MaxHeap 12.7GB（另见 E4） |
| 执行体自证 | 每臂 `[CppBridge] dll=…sha256=`（**硬门禁**：不匹配即判 VOID）+ `[WG-CONF]`（stderr，运行期开关自证） |
| 覆盖面 | **仅**批量预生成吞吐（含 Java vanilla 侧 carvers/features）；**不含**客户端渲染、交互探索延迟、混合核调度、内存存活集 |

## 2. 吞吐基准（发版可公开数字）

| 配置 | 样本 | 各次 wall | 中位 | 吞吐 | JVM 整体 CPU 利用率 |
|---|---|---|---|---|---|
| vanilla Java | 4,225 chunk（r500）×4 | 50 / 48 / 50 / 47 s | 48.5 s | ≈86 ch/s | 4.4 – 5.2 核 |
| **CoreSwap 1.0.28（async，默认）** | 4,225 chunk ×5 | 36 / 36 / 38 / 36 / 37 s | 36 s | **≈114 ch/s** | **11.5 – 11.8 核** |
| vanilla Java | 16,129 chunk（r1000）×1 | 190 s | — | 84.9 ch/s | 5.0 核 |
| **CoreSwap 1.0.28** | 16,129 chunk ×1 | 151 s | — | **106.8 ch/s** | 11.2 核 |
| vanilla Java / CoreSwap | 1,089 chunk（r250）×2 | 13/14  vs  10/11 s | — | 77.8-83.8 vs 99-108.9 ch/s | 4.4 vs 7.4-7.6 核 |
| CoreSwap 1.0.28 **旧形态**（`-Psyncfill=1`） | 4,225 chunk ×3 | 202 / 205 / 206 s | 205 s | 20.7 ch/s | 1.33 – 1.46 核 |

- **vs vanilla**：r500 ≈ **1.34×**（区间 1.24–1.39×，受单跑摆动影响）、r1000 ≈ **1.26×**、r250 ≈ **1.29×**。
- **1.0.28 vs ≤1.0.27（形态）**：202–206 s → 36 s = **5.5–5.7×**（同构建态单变量 A/B，同 dll sha / seed / 半径 / 背靠背）。
- 注意口径：CPU 列 = **整个 JVM 的 CPU 利用率**，**不是** Rust 池宽度（池宽 = 物理核−2 = 10）。

## 3. 引擎因子 A/B（修正版；取代 E1/E2 的空跑比较）

**覆盖机制**：唯一有效覆盖点 = `target/release/worldgen.dll`（`build.gradle:36-50 processResources.doFirst` 每次构建无条件覆盖 `src/main/resources/native/`，见 E1）。每臂另清 `<tmp>/coreswap-native` 解压缓存。

| 臂 | 引擎 dll | 形态 | wall | cps | cpuSec |
|---|---|---|---|---|---|
| eng1028-async | `597e12ed`（1.0.28，09-10 构建） | async | 36 s | 117.4 | 473 |
| eng1027-async | `7a411e01`（**1.0.27 发布**，09-07） | async | 36 s | 117.4 | 465 |
| eng1028-sync | `597e12ed` | sync | 205 s | 20.6 | 280 |
| eng1027-sync | `7a411e01` | sync | 203 s | 20.8 | 303 |
| eng1024-* | `561aff49`（1.0.24 发布） | 两形态 | **启动后崩** | — | — |

- **结论**：1.0.27→1.0.28 的引擎（worldgen.dll）改动在 1.20.1 上**两形态均无速度差**；1.0.28 的全部速度收益来自 **Java 侧异步形态**。
- 1.0.24 臂：`Done (14.550s)` 后 initNether 即 `Task :runServer FAILED` ⇒ **JNI 面不兼容，本载体不可测**（E3，`@anchor.idk`）。
- 谱系核验（排除"测到旧构建"）：`cargo build --offline -p worldgen --release` 对当前源码无需重编，只重链，sha 仍 `597e12ed` ⇒ 被测 dll = 当前源码（含 `a70de20` 及全部 ca_min 工作）；顺带证明构建可复现。
- 证据留存说明：本目录 `cmd-output/*.mca`（32 文件 / 94.5MB）系**修正前那次空跑 A/B** 的两组 region 快照，对"引擎差"零判别力 ⇒ **仅本地保留、不入库**（见同目录 `.gitignore`）。修正版 A/B 只测 wall/CPU，不采 region（其判据是执行体 sha 硬门禁 + 计时）。

## 4. 「为什么引擎无差」——可达性（代码一手核对 + 运行期自证）

1. **运行期自证**（stderr，`[WG-CONF]`，`worldgen_handle.rs:647-661`）：
   `[WG-CONF] ca_min=true est_l2=true ca_cap=2048 flags=3 skip_features=true skip_carver=true skip_surface=false coreswap_threads=(unset)`
2. **行为化负对照**：`WG_CA_MEMODIAG=1` 跑满 4225 chunk ⇒ **`[CA-NT]`/`[CA-MEMO-OFF]` 零输出**（全部输出点在 `apply_features` 体内 :1350-1372）。
3. **代码链**：`worldgen_handle.rs:669-670` `skip_features = flags & FLAG_SKIP_FEATURES != 0 || env WG_SKIP_FEATURES` → `if !skip_features { apply_features(...) }`；mask=3（出厂默认 `CppBridge.resolveStageMask()` = `0b011`）⇒ **`apply_features` 永不执行** ⇒ 其内的 `neighbor_terrain`（:999）、B3a 3×3 快照（:1120-1130）、`ca_min`（:1050）、全部 CA 探针均不可达。carvers 同理（:832）。
4. **判定**：R3 的「内存优化」组件（terrain_cache cache-first / CaTerrainEntry / 3×3 Arc 快照 / MEMODIAG）**全部落在 features 阶段或只被 features 消费**；1.20.1 出厂 mask=3 结构性关掉 features ⇒ 它们要省的「features 触发的邻 chunk 二次地形求值」在 1.20.1 上**从来不存在** ⇒ 零收益**本属预期**，不是"优化失败"。
5. **该路径上唯一可辨的引擎侧变化是反方向的**：`fill_chunk_blocks:619-641` 在 `ca_min` 默认开时每 chunk 做一次 `BlockColumn` 全量 clone（16×16×384 `i32` ≈ 393 KB）+ HashMap insert，而 **1.20.1 上没有任何消费者**（缓存只写不读）。量级在单跑噪声带内（465→473 cpuSec，+1.7%），**不可判定**，但机制方向明确 ⇒ 候选微修：**features 关闭时跳过既有缓存回填**（或按 mask 门控 ca_min 的缓存段）。
6. **1.21.6 侧同病**：`resolveStageMask()` 两版本逐行相同（均默认 `0b011`），`WG_CA_MIN` 默认同源 ⇒ 那批优化在 1.21.6 **出货默认**下同样不可达。

## 5. 内存探针（自我否证；结论：无存活集回归证据）

| 臂 | dll | 堆设置 | wall | cpuSec | 峰值 WorkingSet |
|---|---|---|---|---|---|
| mem-1028-caON | 597e12ed | 默认（MaxHeap 12.7GB） | 40 s | 507 | 5973 MB |
| mem-1028-caOFF（`WG_CA_MIN=0`，`[WG-CONF] ca_min=false` 已自证） | 597e12ed | 默认 | 36 s | 463 | 6010 MB |
| mem-1027-ref | 7a411e01 | 默认 | 36 s | 471 | 2713 MB |
| m2-1028 / m2-1028b | 597e12ed | 默认 | — | — | 6505 / 5976 MB |
| m2-1027 / m2-1027b | 7a411e01 | 默认 | — | — | **4666 / 2984 MB** |
| **xmx2g-1028** | 597e12ed | **`-Xmx2G`** | **36 s** | 469 | 3043 MB |
| **xmx2g-1027** | 7a411e01 | **`-Xmx2G`** | **35 s** | 469 | 2223 MB |

- **自我否证**：同一 dll 同配置两次跑出 **2984 vs 4666 MB（1.7× 摆动）** ⇒ `WorkingSet` 峰值 = G1 堆高水位（未设 `-Xmx`），**不是存活集**；单对差值（"1.0.28 多占 3.3GB"）**不成立**（E4）。
- **判定性实验**：固定 `-Xmx2G` 下两引擎**均满速跑完**（36 s / 35 s，cpuSec 相同）⇒ 玩家常用堆档下**无阻塞性内存回归**；两臂峰值差（3043 vs 2223 MB）在已证摆动带内，**不作结论**（n=1/臂）。

## 6. 跨版本报告（朋友「1.0.28 不如 1.0.24」）归因

- 用户提供两份日志（CoreSwap `2026-09-10-1.log` vs vanilla `latest.log`）分析：vanilla 全程 **VD 32**；CoreSwap **VD 12 持续 26 s 后转 32 约 10 s** ⇒ 需求差 ~7×，**两者不可比**（§9.7）；CoreSwap 日志中 `[CppBridge] init` 出现在 spawn 准备之后（出生区恒为 vanilla 生成，属已知边界）；两日志 **seed 不同**（CoreSwap `7349435828306001495`，vanilla 未记录）⇒ 无同世界可比性。
- 本载体结论：**形态（async）在 4 核与 24 核可见核下都远胜旧形态**（4 核：45.4 vs 20.6 cps = 2.2×；24 核：117.4 vs 20.7 cps = 5.7×）⇒ "异步在少核机器上更差"被否；引擎因子无差（§3）⇒ **本载体无法复现朋友报告**。
- 剩余候选解释（**均未验证**，需朋友机器上的对照）：
  1. **协议不可比**：同世界/同坐标二次传送可能读的是**已生成区块**（= 磁盘+客户端网格，不是世界生成）；同世界不同坐标则地形类型/成本不同。
  2. **客户端共享工作池**：async 占用 `Util.getMainWorkerExecutor()`（核数−1）；1.20.1 客户端区块网格构建也用它 ⇒ 同进程内两者争用（本载体无客户端，测不到）。
  3. **混合核（i5-13600KF 6P+8E/20T）** 调度差异（本载体为全大核 12C/24T）。
  4. 朋友实例**含其他 mod**（至少一个远程传送 mod）⇒ 第三方变量。
- 建议的判别实验（不占用其世界）：**同 jar 单变量 A/B** `-Dcoreswap.syncfill=1`（运行期属性，无需重编；`NoiseChunkGeneratorMixin.java:71` 读 `System.getProperty`），同世界、同视距、**同一批从未去过的坐标**，背靠背各 1 次；若同步形态反而"体感更快"，则 2 成立。

## 7. 边界 / 待办

- **待办 ①（发版前置）**：重编发布 jar（默认 async，**不带** `-Psyncfill=1`，`--rerun-tasks` 规避沙箱内 gradle VFS 假 `UP-TO-DATE`）→ 执行体三元组核验（jar sha / jar 内 dll sha / `target/release/worldgen.dll` sha）→ 填 `RELEASE-1.0.28.md` + INDEX。**本块未做**（被内存探针链占用；工单不可在未核验 sha 的情况下出单）。
- **待办 ②**：候选微修（§4.5）：features 关闭时跳过 terrain_cache 回填（或 mask 门控），并核 1.21.6 是否同样"只写不读"。
- **待办 ③**：知识库草稿（须 subagent 产出，本块未派）：E1「自证行硬门禁」、§4「跨版本共享引擎性能改动必须可达性核对」、E5「bench flags vs 出货默认口径」、E4「WorkingSet 峰值 ≠ 存活集」、E6「公众声明随门控核对」。
- **待办 ④**：1.20.1 `WG_CA_MIN` / `WG_CA_*` 无 gradle `-P` 映射（1.21.6 有）⇒ 用户侧只能靠系统 env；若要做用户可用的回退开关需补映射。
- 既有债务（沿用）：260910-05 数字需按 E5 口径重算；1.20.1 下界/末地完整行为门；`runtime/` 遗留清理；多 gradle home。
