# B6-3 light 接管面读写站点地图（scout 勘探，260918-06）

- 状态：**draft**（静态勘探，Degraded——全部为源码静态读取，未运行任何探针验证）
- 范围声明：本图是「light 接管面 reads/writes 声明表」的前置地图，只列站点与门控，不下结论性判定。
- ⚠️ 范围更正（诚实声明）：任务指定 `java-core\src` 重点看 ServerLightingProviderMixin，但该 mixin 实际在
  `versions\1.20.1\java\src\main\java\wg\bench\mixin\`（已知锚点 :77 / :545-550 / :718-724 行号与该文件
  逐行吻合）。`java-core\src` 只含共享核（BulkWb / CppWorldgen / BenchMod 等）。本图按实际位置登记。

---

## 一、Java 读面（light 接管面）

| 站点 | 读/写 | 数据域 | 门控 sysprop（默认值方向） | 备注 |
|---|---|---|---|---|
| versions/1.20.1/java/src/main/java/wg/bench/mixin/ServerLightingProviderMixin.java:248-299 `wgLightCollectBlocks` | 读 | chunk 方块（palette/storage 经 writePacket 帧或 getBlockState 直采）→ blocks9 int[]（rawId\|lum<<24） | 依赖 LIGHT_RUST；子路受 LIGHT_OLD_COLLECT（默认关→快路径 B 优先） | 空 section 填 0（=AIR）；收集失败→回退 vanilla |
| 同上 :303-318 `wgLightNeighbor` | 读 | ChunkProvider 邻 chunk 引用 + ChunkStatus（≥FEATURES 检查） | LIGHT_RUST | 只读检查，不写 chunk 状态 |
| 同上 :263-266 | 读 | ChunkSection[] 活引用（getSectionArray） | LIGHT_RUST | 长度 <24 → 回退 |
| 同上 :433-491 `wgLightFillSectionFast` | 读 | PalettedContainer.writePacket 帧（bits/palette/longs，LSB-first 解码） | 默认路（oldcollect 关） | 帧意外→回旧路；bits 域 4..16 防御 |
| 同上 :398-414 `wgLightFillSectionOld` | 读 | section.getBlockState 逐格（rawId + luminance） | LIGHT_OLD_COLLECT 开 或 快路径失败 | 回退基准路，对拍已移除（#48 惯例） |
| 同上 :326-394 `wgLightCollectPacked` | 读 | packed 三段 meta/pal/sto（writePacket 帧，bits 域 4..14，global 节→整 chunk 回退） | **!LIGHT_BLOCKABI** 时执行（见分叉 F3） | global（bits 0 之外 ≥15）回退 blocks9 |
| 同上 :419-424 `wgLightEncState` | 读 | Block.STATE_IDS（state id→rawId+lum 编码） | 随收集路 | 未知 id→MIN_VALUE→回退 |
| 同上 :200-203 `wgLightHeightOk` + :251 | 读 | 世界高度参数（bottomSection=-4, span=24） | LIGHT_RUST | 不符→vanilla，blocks9 布局域前提 |
| 同上 :810-816 `wgLightNibble` | 读 | out 缓冲 + outFlags（0=数据/1=全0/2=全15） | 随写回 | 构造 ChunkNibbleArray 的输入读取 |
| 同上 :77-133 各静态 final | 读 | sysprop 门本身 | —（即门） | 一手消费点直读，全部 `!= null` 判定（betadump/data 为 String 型门） |
| LightPalDump.java:24-41（versions/1.20.1/java/.../bench/） | 读 | PalettedContainer.Data 私有 record（反射 getDeclaredField("data")） | LIGHT_PALDUMP > 0（`coreswap.light.paldump`，Integer.getInteger 默认 **0=关**） | #55 反射重映射家族：仅 dev loom/yarn 名域可运行；mixin :279-281 / :757-759 两处消费 |
| LightDomainBatch.java:21（versions/1.20.1/java/.../bench/） | 读 | sysprop（gracems） | `coreswap.light.domainbatch.gracems`，Integer.getInteger 默认 **10_000** | 宽限期上限；负/0=无宽限 |

## 二、Java 写面（light 接管面）

| 站点 | 读/写 | 数据域 | 门控 sysprop（默认值方向） | 备注 |
|---|---|---|---|---|
| ServerLightingProviderMixin.java:743-747 `wgLightLegacyTakeover` 写回段 | 写 | **blocklight + skylight** nibble（enqueueSectionData × 24 section × 2 通道） | LIGHT_RUST（:77，默认**关=vanilla**） | 单一总门；flag 1/2 走 defaultValue 构造（非 null） |
| 同上 :750-751 | 写 | chunk.setLightOn(true) + TACS releaseLightTicket（light ticket 状态域） | LIGHT_RUST | 复刻原 light() 尾部 POST 语义 |
| 同上 :566-573 `wgLightDomainWriteBack` | 写 | 同上双通道 + setLightOn + 摘票 | LIGHT_DOMAIN（:79，默认**关**）| 域批任务线程执行；POST 严格在对应 chunk 算完后 |
| 同上 :545 `Arrays.copyOf(b9, …)`（快照） | 读→写（内存快照） | blocks9 | LIGHT_DOMAIN | 提交线程侧快照，域任务零容器读（260916-01 崩溃修复） |
| 同上 :231-245 `wgBetaDump` | 写 | **文件系统**：<betadump>/<cx>_<cz>.bin（packed 三段原始字节） | LIGHT_BETADUMP（`coreswap.light.betadump`，null=关）**且须 LIGHT_BETAPROBE 同开**（:698 判据） | 采集副作用面，声明表须登记此写域 |
| 同上 :762-766 native-throw 置位 | 写 | lightNativeDead 原子标志 | LIGHT_RUST | 置位后永久回退 vanilla |
| LightDomainBatch.java:52-93 | 写 | DOMAINS/futures/chunks/blocks9s 内存登记表 | LIGHT_DOMAIN | seal 幂等（remove(key,st)）；未注册回调 loud fail |
| BulkWb.java（java-core/src/main/java/wg/bench/）:163-311 | 写 | chunk section **方块** PalettedContainer 原地替换 + 三派生计数（nonEmptyBlock/randomTick/nonEmptyFluid） | `coreswap.bulkwb`（WgCompat.flag，fallback=BULKWB_ON=**1.20.1 默认开**；"0" 显式关） | ⚠️ 非 light 面，但同 chunk section 数据域——light 读面（getSectionArray）与 bulk 写面共享 section 数组；sentinel（bulkwbsentinel，默认关）覆盖并发冲突。是否入 light 声明表由主会话裁决（跨面触点） |
| WgCompat.java:14-17 `flag()` | 读 | sysprop 包装器 | — | 「非 "0" 即真」语义 = light.* 家族 `!= null` 语义**不同**（#19 家族风险注记）；light 面无直接消费点，bulk 面消费（BulkWb:90） |

## 三、Rust 读面（light_compute，worldgen-core）

| 站点 | 读/写 | 数据域 | 门控 | 备注 |
|---|---|---|---|---|
| worldgen-core/src/light/mod.rs:106-109 `from_json_file` | 读 | **文件系统**：light_data.json（corewap-light-1 格式） | 无 sysprop（经 lightInit 路径参数；路径由 Java `coreswap.light.data` 决定，默认 E:/…/light_data.json） | light 面唯一 Rust 文件读 |
| mod.rs:112-139 `from_json_str` + :164-174 `parse_u8_field` | 读 | opacity/emission 查找表（raw id 索引，动态扩容） | 无 | 负 opacity clamp 0（260905-05，无 VoxelShape 有损边界） |
| mod.rs:143-148 `lookup` | 读 | 表（缺 id/负 id→default） | 无 | air_fast 前提 table[0]==(0,0) |
| mod.rs:182-192 `light_compute` / :308-338 `light_compute_domain` / :340+ `light_compute_inner` | 读 | blocks9/blocks25 输入数组（方块 rawId 域） | 无 Rust 侧门控（门在 Java 侧） | 纯函数 + per-call Scratch；无世界/文件访问 |
| mod.rs:199-201（packed 解码，`packed_to_blocks9`，行 199-283 区段） | 读 | packed meta/pal/sto（Java 预拆帧） | 无 | 解码失败 PackedDecode→JNI rc=-2 |
| versions/1.20.1/rust/src/jni_bridge.rs:296-311 thread_local 缓冲 | 读/写（内部 scratch） | b9/ob/os/of/pm/pp/ps | 无 | 每次全量覆写，无跨调用脏数据（:289-291 声明） |
| jni_bridge.rs:315-327 `lightInit` | 读 | 文件路径字符串→LightEngine | 无 | 失败返 0→Java 回退 vanilla |
| jni_bridge.rs:346-…/424-…/475-… lightCompute/Domain/Packed | 读 | JNI 数组（blocks9/blocks25/meta/pal/sto） | 无 | catch_unwind 兜底 rc=-3，panic 不跨 FFI |

## 四、Rust 写面（light_compute）

| 站点 | 读/写 | 数据域 | 门控 | 备注 |
|---|---|---|---|---|
| mod.rs `light_compute_inner` export 段（:340+，经 :191/:293/:335 调用） | 写 | out_block/out_sky（24×2048 B 双通道）+ out_flags（48 B：0/1/2 均质旗标） | 无（Java 侧 LIGHT_RUST/LIGHT_DOMAIN 决定是否触达） | 写出目标 = JNI 缓冲→Java out 数组→enqueueSectionData；**不直接写世界** |
| jni_bridge.rs:331-340 `lightDestroy` | 写（内存） | Box 回收 | 无 | handle=0 no-op |
| bin-diag（light_bench*/light_golden_dump/light_probe_260914，worldgen-core/src/bin-diag/） | 写 | 诊断 stdout/golden 文件 | 无 sysprop（独立 bin，不参与默认构建） | §八.13 隔离区；不进生产触达面 |

- 声明修正注记：任务提示「blocks.json 读取」——light 模块**未发现** blocks.json 读取；其唯一数据文件输入是
  light_data.json（blocks.json 属 worldgen 方块注册域，另一条管线）。已在分叉/待查清单登记。

## 五、门控总表（sysprop 一手消费点直读）

| sysprop | 消费点 file:line | 形态 | 默认方向 | 备注 |
|---|---|---|---|---|
| coreswap.light.rust | Mixin:77 | `!= null` | **关（vanilla）** | 单一总门（已知锚点） |
| coreswap.light.domainbatch | Mixin:79 | `!= null` | 关 | 域批 A/B 臂 |
| coreswap.light.domainbatch.gracems | LightDomainBatch:21 | Integer.getInteger | 10_000 | 非「存在性门」，数值门 |
| coreswap.light.data | Mixin:82-83 | String | 默认路径 hardcode | light_data.json 位置 |
| coreswap.light.timing | Mixin:104 | `!= null` | 关 | chunk 级计时（不进每格热路径） |
| coreswap.light.paldump | Mixin:105 | Integer.getInteger | 0=关 | 消费点二处（mixin:279/:757）+ LightPalDump 反射面 |
| coreswap.light.betaprobe | Mixin:115 | `!= null` | 关 | P-β/P-path 探针 |
| coreswap.light.betadump | Mixin:120 | String | null=关 | **须与 betaprobe 同开才生效**（:698）；文件写域 |
| coreswap.light.oldcollect | Mixin:126 | `!= null` | 关 | 强制旧收集路 |
| coreswap.light.blockabi | Mixin:133 | `!= null` | 关 | ⚠️ 语义疑似反转，见分叉 F3 |
| coreswap.bulkwb | BulkWb:90 经 WgCompat.flag | 「非"0"即真」 | **1.20.1 默认开** | 跨面触点（bulk 写 section / light 读 section） |
| coreswap.bulkwblog / wbcontent / bulkwbtest / bulkwbsentinel | BulkWb:91-97 | `!= null` | 关 | bulk 面诊断门（跨面注记） |

⚠️ #53 警示落点：以上默认值全部为本次一手消费点直读（grep `System.getProperty|Integer.getInteger` 全量
17 处命中，见站点行），非交接文档转述；但「build.gradle -P 映射表侧」未在本普查范围内核对（属 B6-1
check_switch_mapping.py 门禁职责），标注 Degraded。

## 六、触达形态潜在分叉（≥2 互斥候选 → 触发 fan-out 建议）

- **F1 写回触达形态二分**：legacy per-chunk 写回（Mixin:743-751，light 调用线程）vs 域批写回
  （Mixin:566-573，Util.getMainWorkerExecutor 线程）。互斥（同一 chunk 只走其一），数据域相同、
  执行线程/时序不同 → 声明表「写站点」须按两形态分别验证触达。预检不过还会第三形态：vanilla 原生
  写回（不 cancel）——即**三**互斥触达形态候选（domain / legacy-rust / vanilla-fallback）。
- **F2 输入收集形态三分**：blocks9 快路径 B（writePacket 位流解码）/ blocks9 旧路（getBlockState 直采）/
  packed ABI（meta/pal/sto 直传）。同一 chunk 收集时 B 与 packed 互斥分流（:684）、失败互为回退——
  实际触达取决于 chunk 内容（global 节/帧意外），声明表须覆盖「任一 chunk 内三种收集形态均可出现」。
- **F3 blockabi 语义疑似反转（建议优先 fan-out 验证）**：注释区（:132 候选 C「packed ABI（writePacket
  帧直传，Rust 侧解码）」）暗示 blockabi 开启 = 走 packed；但消费点 :684 `if (!LIGHT_BLOCKABI)` 才执行
  `wgLightCollectPacked` ——即 **blockabi=开 实际跳过 packed、强制 blocks9**。两个互斥解读：
  (a) 消费点取反是笔误/历史演化残留，packed 实为默认常开路；
  (b) 语义已反转且注释过期，blockabi 现为「禁用 packed 回 blocks9」的回退门。
  静态无法裁决，须 fan-out（消费 git 历史判 (a)/(b) + 运行探针核触达）。
- **F4 数据域候选澄清**：light 输入数据文件是 light_data.json（一手消费点 mod.rs:106），blocks.json 与
  light 面无关——若上游声明草案含 blocks.json，须先消歧（声明域收窄 vs 我漏读；本普查 grep 未见）。

## 七、诚实声明（Degraded 全单）

1. **全部结论为静态源码读取，零运行验证**（未跑 server/探针/JNI）——整图标 Degraded。
2. **未读面**：`LightPalDump.java` 全文（只 grep 门控与反射头）、`jni_bridge.rs` :380-566（domain/packed
   JNI 体只 grep 关键行，未逐行读）、`FormProbe.java`、`BenchMod.java`（lightDataDump 入口钩子），
   versions/1.21.6 侧全部（本图仅 1.20.1 面）、java-core 其余共享核文件、`knowledge/INDEX.md`（本 scout
   快速普查未走 STEP 1 全量知识库核对——相关 discovered 条目 #53/#55/#102/#118 均自源码注释二手引用，
   未回知识库原文核对）。
3. **#102 消费点集合是否为空**：本普查未识别 #102 具体指涉，无法断言（未读知识库原文）——登记为待查。
4. **LightStorage/持久化 NBT 触点**：未发现 CoreSwap 对 LightStorage/ChunkSerializer 的直接 mixin；
   light 写回经 vanilla `enqueueSectionData` 间接进入 LightStorage 与存档 NBT——间接触达未做 xref 级
   摸底，标待深入。heightmap：light 接管面未发现直接 heightmap 读写（sky 用自算 col_max，mod.rs:87）。
5. 门控默认值方向 = 静态直读；**运行时实际生效值未验证**（sysprop 可被启动参数覆盖，属 #53 默认值当
   公理的反向面——本图只声明「缺省方向」）。
