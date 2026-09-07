# jni-wiring-scout-260907-07（recode.scout 勘探产物）

- 日期标签：260907-07（按任务指定；实际执行以 git 时间戳为准）
- 范围：E:\PYTHON\CoreSwap\runtime\1.20.1\java + worldgen-core（只读）
- 目的：mod 方块 JNI 接线（Java 调 wg_register_block）前置摸底
- 状态：draft（勘探 = Degraded 静态阅读，无运行时验证）
- 纪律遵守：全程未访问 E:\PYTHON\MC（历史目录禁入）

## 1. Java 侧现有 wg_* native 声明与加载模式

| 站点 | 内容 |
|---|---|
| runtime\1.20.1\java\src\main\java\wg\CppWorldgen.java:17-25 | JNI 桥类；static block：`-Dcpp.worldgen.lib` 显式路径 System.load，否则 `extractNativeDll()`（jar 内 native/worldgen.dll 解压到 %TMP%\coreswap-native，CoreSwapFixHelper:65-80 带哈希版本化缓存） |
| CppWorldgen.java:40 | `native long init(long seed, String worldgenDir)` → wg_create |
| CppWorldgen.java:43 | `native long initDim(seed, dir, settingsName, biomeParamsFile, worldHeight)` → 多世界 |
| CppWorldgen.java:46-92 | destroy / fillDensity / fillBlocks / setBeardifier / setFlags / getFlags / light* |
| wg\bench\CppBridge.java:21-28 | handle 持有：static volatile long handle / netherHandle / endHandle（enabled 标志） |
| wg\bench\CppBridge.java:76-96 (init)、119-134 (initNether)、139-156 (initEnd) | 句柄创建点：init 后立即 setFlags(resolveStageMask()) |
| wg\bench\BenchMod.java:15-45 | **时机入口**：Fabric `ServerLifecycleEvents.SERVER_STARTED` 钩子 → `-Dcpp.replace=1` 或默认 active → CppBridge.init(seed) → initNether → initEnd |

**grep 确认**：`wg_register_block|registerBlock|nativeRegister` 在 java 树零命中 → 接线确实未开始（与任务背景一致）。

## 2. mod 方块在 Java 侧的枚举途径（可复用先例）

| 站点 | 先例 |
|---|---|
| BlockProbe.java:946-949 | **全注册表遍历先例**：`for (Block block : Registries.BLOCK) { idToName.put(Registries.BLOCK.getId(block).toString(), getRawId(block)) }` → 导出 blocks.json（Rust 侧数据源正是这个格式） |
| LightDataDump.java:33-34 | 同模式遍历 Registries.BLOCK（Iterable<Block> + getRawId） |
| BlockProbe.java:927 | `out.writeShort(Registries.BLOCK.getRawId(block))`（vanilla raw id 序列化先例） |
| CppBridge.java:30-35, 417-421 | Java 侧反向映射缓存先例：`STATE_BY_ID`（rawId→BlockState，`Registries.BLOCK.get(id)` → getDefaultState），注释明确 raw id ↔ state 正确映射 |
| NoiseChunkGeneratorMixin.java:45-51 | **RegistryEntry key 查询先例**：`getSettings().getKey().map(k -> k.getValue().toString())` —— settings_name 命名空间判定（"minecraft:overworld" 等）；mod 维度 id（"aether:*"）也是此形态 |
| DensityProbe.java:475 | `for (var id : dfReg.getIds())` registry ids 遍历（noise 命名空间） |
| worldgen_handle.rs:369-376 | Rust 侧 mod biome 命名空间目录合成先例（data_ns → `data/<ns>/worldgen/biome`）——mod 命名空间处理已有先例 |

结论候选（candidate）：mod 方块枚举 = 遍历 `Registries.BLOCK`，对 raw id 超出 Rust blocks.json 表（动态 id ≥ max+1）或名字不在其中的条目调 `CppWorldgen.registerBlock(handle, "modid:name")`。注意 Java raw id 与 Rust 动态 id **不是同一 id 空间**——Rust `register` 分配 max+1 递增 id（blocks.rs:68-86），与 vanilla raw id 无对应关系；接线时 Rust 返回的 id 只在 Rust 生成块输出内自洽，Java 回写侧已用 `Registries.BLOCK.get(rawId)` 反查（CppBridge:421），需核对 CppBridge 写回路径用的是 Java raw id 还是 Rust id（**矛盾点 ⚠️**，见 §5）。

## 3. Rust 侧 wg_register_block 调用契约（一手 api.rs / worldgen_handle.rs / blocks.rs）

- **签名**（api.rs:89-95）：`wg_register_block(handle: *mut c_void, name: *const c_char) -> c_int`
  - handle 为空或 name 为空 → **-1**；成功 → `h.register_block(&name)` 返回的 id（≥0）
  - 已存在名字 → 返回既有 id（幂等，blocks.rs:70-74 双检）
- **后端**（worldgen_handle.rs:530-531）：`register_block(&self, name) -> i32 { self.blocks.register(name) }`
- **动态 id 分配**（blocks.rs:66-86）：next_id 从 blocks.json max+1 起 AtomicI32 fetch_add 递增；RwLock 双检并发安全；id_to_name 表容量 16384（blocks.rs:36），超容量 stderr 告警（O1）但 id 仍分配、name 查找回 "?"
- **注册失败面**：api.rs 只有 null → -1；无显式"注册失败"码——容量超限不报错只告警（⚠️ 接线侧不能只判 -1 认为万事大吉）
- **生命周期约束**：
  - handle = `Box::into_raw(Box::new(WorldgenHandle))`（api.rs:74），wg_destroy 前（api.rs:81-84 drop）必须完成全部注册
  - blocks 注册表 `Box::leak` 后以 `&'static BlockRegistry` 存活在各 sampler（worldgen_handle.rs:383-390）→ 注册通过 `&self`（内部 RwLock）写，**与 fill 并发读理论无写竞争**（注释 blocks.rs:67：约定"生成线程启动前"）
- **时机约定（一手注释，api.rs:88）**："调用时序约定：生成线程启动前（创建期注册）"

### 时机窗口结论

- Java 侧窗口 = **CppBridge.init()/initNether()/initEnd() 返回 handle 之后、任何 populateNoise 拦截发生之前**。
- 具体挂点候选：BenchMod SERVER_STARTED 钩子里 init 三句柄之后（BenchMod.java:37-42 之后插入注册循环）——此时服务器已起但 chunk 生成尚未被玩家/ Spawn 触发；严格最早期窗口是各 init* 内部（handle 拿到立即注册，BenchMod SERVER_STARTED 与首个 populateNoise 之间由主线程串行保证）。
- ⚠️ 残余风险：SERVER_STARTED 后若有 mod/数据包在 server start 阶段就请求 chunk（spawn chunk 准备），可能先于注册触发生成——Java populateNoise 拦截在 chunk 生成线程，与主线程注册无同步屏障。稳妥做法 = 注册循环放 init* 之后**同一 SERVER_STARTED 回调内同步执行**（回调在主线程，spawn chunk 生成通常在 tick 开始后，但 @anchor.idk：spawn prepare 与 SERVER_STARTED 的精确先后未做运行时验证）。

## 4. 端到端实测现有载体

| 载体 | 站点 | 用法 |
|---|---|---|
| gradle runServer | runtime\1.20.1\java\build.gradle:79,128,199 | `-PcppWorldgenDir=`（绕解压）、`-PcppLib=`（绕 jar 直载 dll）、runServer 任务存在 |
| run_rust_client.ps1:108 | -PcppLib → -Dcpp.worldgen.lib 直载 | 改 dll 后免重打包 |
| JniProbe | wg\bench\JniProbe.java:32-150（-Pjni.probe=true） | **现成 JNI 冒烟**：init/initDim + fillBlocks + destroy，注册接线可先挂这里最小验证 |
| CppBridge 存档链路 | CppBridge.java:76+，BenchMod.java:37-42 | 端到端（-Dcpp.replace=1 或默认 active），stderr `[BLOCKS] registered ...`（blocks.rs:84）可直接在 runServer 日志观察 |
| WorldGenBench | -Dworldgen.bench（BenchMod:70-71） | 大样本生成冒烟 |
| Rust 侧 bin-diag | worldgen-core\src\bin-diag\（blocks_cmp.rs 等读 blocks.json） | 动态 id 侧验证可仿 blocks_cmp 加注册对比（bin-diag 隔离区，非 src/bin） |
| 数据路径/命名空间入口 | CoreSwapFixHelper.java:35-54（worldgenDir 解压 + marker）；worldgen_handle.rs:364-376（data_ns biome 目录合成）、api.rs:67-72（settings_name 默认 overworld.json） | mod 命名空间 noise_settings/biome 当前解析入口；**mod 方块名注册不走文件路径**，直接走 C ABI |

## 5. 接线建议候选（含矛盾点）

**候选 A（最小侵入）**：在 CppBridge 增 `registerModBlocks()`——遍历 `Registries.BLOCK`（BlockProbe:947 模式），名字不在 Rust blocks.json（如何判断：Rust 侧 register 本身幂等，可直接全量注册，重复名返回既有 id 零成本）→ 对每个 `Registries.BLOCK.getId(block).toString()` 调新 `CppWorldgen.registerBlock(handle, name)`；挂 BenchMod SERVER_STARTED init 三句柄后。新增 native 声明一处（CppWorldgen.java）。
- ⚠️ 矛盾点 1（id 空间）：Java 注册全量（含 vanilla）时 Rust 返回的是 blocks.json 既有 id / 动态 id，**与 Java raw id 不同**；CppBridge 写回 Chunk 用的是 Rust 输出的 raw id 直接 setBlockState（`Registries.BLOCK.get(id)`，CppBridge:417-421）——若 Rust 动态 id ≠ Java raw id，mod 方块写回会映射到错误方块。**需核实**：Rust fill 输出 id 的消费约定（blocks.json id == vanilla raw id 是当前隐式前提；mod 方块在 blocks.json 中无条目 → 动态 id 与 Java raw id 必然错位）。**这是接线设计的核心矛盾，须主会话/fan-out 裁决**（候选解：Java 侧按 Java raw id 顺序全量重注册对齐 / Rust register 支持显式 id / Java 侧写回改双表映射）。

**候选 B（显式 id 版）**：扩展 ABI `wg_register_block_id(handle, name, javaRawId)`，Rust 直接登记 Java raw id（不分配动态 id）——消除错位，但改 Rust api.rs（260907-05 交付为 confirmed，改动需走 judge + 用户拍板）。

**候选 C（仅注册、先不解决写回）**：只做候选 A 的注册 + 日志观察（`[BLOCKS] registered`），写回错位问题单独开卡——适合先把「Java→Rust 注册通路」端到端打通（JniProbe 冒烟）。

## 6. 待深入点

1. CppBridge.fillChunk 写回路径完整核对（offset 161-521 未全读）——确认 Rust id→Java state 映射的现行实现，定候选 A/B 裁决依据。
2. SERVER_STARTED 与 spawn chunk prepare 的先后（需运行时验证，探针）。
3. blocks.json 内是否已含全部 vanilla（决定「全量注册」是否只是 mod 子集）。

## 7. 混淆评估

无混淆：Java 侧为 yarn-mapped Fabric 源码（sources 直接可读），Rust 侧一手源码。无需 recode.deobfuscate。
