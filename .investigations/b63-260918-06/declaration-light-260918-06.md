# light 接管面读写声明表（B6-3 试点，260918-06）

- 状态：**draft**（Degraded——静态源码读取 + F3/F4 消歧事实继承；零运行验证；judge review-260918-06 PASS-with-conditions S1-S5 已应用）
- §9.7 等价档位（judge S5 补行）：**E1 同构建态单变量**（比对证据 = pbeta05c.log，单 seed × 2025-chunk × 3 boot × legacy 臂 packed 路，n=1 载具）；声明本体为静态读面（Degraded），运行时触达子集仅由该证据源背书，**不外推**其他臂/维度/ABI 形态。
- 范围：light 接管面 = `ServerLightingProviderMixin.java`（1.20.1）+ `LightPalDump` / `LightDomainBatch` / `BulkWb`（跨面触点标注）+ Rust `worldgen-core/src/light/` + `versions/1.20.1/rust/src/jni_bridge.rs`。仅 1.20.1 面（1.21.6 未普查，见诚实声明）。
- 行号置信度：本表所有 file:line 均为本 worker 一手逐行核对（2026-09-18 本轮读源），非 scout 图转述；「未核」字样 = 未读到。
- 「生产门控 mask」定义（本表口径）：**全部 sysprop 缺省**（即 `LIGHT_RUST=off`、`LIGHT_DOMAIN=off`、探针/paldump/oldcollect/betaprobe/betadump 全关、`coreswap.bulkwb` 缺省=开）。

---

## 一、Java 读面

| 站点 file:line | 读/写 | 数据域 | 门控 sysprop（默认方向） | 前提条件（生产 mask 可达性） | inverse 状态 | 备注 |
|---|---|---|---|---|---|---|
| mixin/ServerLightingProviderMixin.java:248-299 `wgLightCollectBlocks` | 读 | chunk 方块 → blocks9 int[]（rawId\|lum<<24，884736） | LIGHT_RUST（缺省关）；子路 LIGHT_OLD_COLLECT（缺省关→快路径优先） | 仅 `LIGHT_RUST` 开时可达；**生产 mask 下不可达**（总门关=vanilla） | derived（读入 ThreadLocal 缓冲，无外部副作用） | 空节 fill 0（:273）；收集失败→vanilla（:298 `k==884736`） |
| 同 :303-318 `wgLightNeighbor` | 读 | ChunkProvider 邻 chunk 引用 + ChunkStatus ≥FEATURES 检查 | LIGHT_RUST | 同上，生产 mask 下不可达 | derived | 只读检查，不写 chunk 状态（:313） |
| 同 :263-266 | 读 | ChunkSection[]（getSectionArray，len<24→回退） | LIGHT_RUST | 同上 | derived | 与 bulk 写面共享 section 数组（跨面触点，见二 BulkWb 行） |
| 同 :433-491 `wgLightFillSectionFast` | 读 | PalettedContainer.writePacket 帧（bits/palette/longs，LSB-first 解码） | 缺省路（oldcollect 关） | LIGHT_RUST 开且 oldcollect 关；生产 mask 下不可达 | derived | 帧意外→回旧路；bits 域 4..16 防御 |
| 同 :398-414 `wgLightFillSectionOld` | 读 | section.getBlockState 逐格（rawId+luminance） | LIGHT_OLD_COLLECT（缺省关）或快路径失败回退 | LIGHT_RUST 开时仍可作为快路径回退触达；生产 mask 下不可达 | derived | 回退基准路；对拍已移除（#48 惯例，:282-283 注释） |
| 同 :326-394 `wgLightCollectPacked` | 读 | packed 三段 meta/pal/sto（writePacket 帧，bits 域 4..14，global 节→null） | **!LIGHT_BLOCKABI** 时执行（:684-686）；LIGHT_BLOCKABI 缺省=false ⇒ **缺省=packed 路**（F3，一手核实） | LIGHT_RUST 开（legacy 臂）；生产 mask 下不可达 | derived | 设 `-Dcoreswap.light.blockabi` = blocks9 A/B 回退；无语义反转，开关命名语义=「用 blocks9 ABI」 |
| 同 :419-424 `wgLightEncState` | 读 | Block.STATE_IDS（state id→rawId+lum 编码） | 随收集路 | 同收集路；生产 mask 下不可达 | derived | 未知 id→MIN_VALUE→回退 |
| 同 :200-203 `wgLightHeightOk` + :251/:327 | 读 | 世界高度参数（bottomSection=-4，span=24，bottomY=-64） | LIGHT_RUST | 同上 | derived | 不符→vanilla；blocks9 布局域前提 |
| 同 :810-816 `wgLightNibble` | 读 | out 缓冲 + outFlags（0=数据/1=全0/2=全15） | 随写回（LIGHT_RUST） | 写回触达时；生产 mask 下不可达 | derived（拷出到新 ChunkNibbleArray） | flag 1/2 走 defaultValue 构造 |
| 同 :77-133 各静态 final | 读 | sysprop 门本身 | —（即门） | 类加载即消费，**任何 mask 均可达** | derived（进程级 final，无逆——进程生命周期内不可变，重启即消） | 全部 `!= null` 判定；paldump/gracems 为数值型门（:105、LightDomainBatch:21） |
| bench/LightPalDump.java:24-41 | 读 | PalettedContainer.Data 私有 record（反射 getDeclaredField("data")） | `coreswap.light.paldump`（Integer.getInteger，缺省 **0=关**） | paldump>0 且 LIGHT_RUST 开；生产 mask 下不可达 | derived（只读反射，无写） | #55 反射重映射家族：仅 dev loom/yarn 名域可运行；消费点 mixin:279-281/:757-759 |
| bench/LightDomainBatch.java:21 | 读 | sysprop（gracems） | `coreswap.light.domainbatch.gracems`（缺省 **10_000**） | LIGHT_DOMAIN 开时有语义；生产 mask 下不可达 | derived | 非「存在性门」，数值门 |
| jni_bridge.rs:296-311 thread_local 缓冲（Rust 侧，列此备查） | 读+写（内部 scratch） | b9/ob/os/of/pm/pp/ps | 无（随 JNI 调用） | LIGHT_RUST 开；生产 mask 下不可达 | derived（每次全量覆写，:289-291 声明） | 无跨调用脏数据 |

## 二、Java 写面

| 站点 file:line | 读/写 | 数据域 | 门控 sysprop（默认方向） | 前提条件（生产 mask 可达性） | inverse 状态 | 备注 |
|---|---|---|---|---|---|---|
| mixin/ServerLightingProviderMixin.java:743-747 `wgLightLegacyTakeover` 写回段 | 写 | **blocklight + skylight** nibble（enqueueSectionData ×24 section ×2 通道）→（vanilla LightStorage→持久化 NBT，间接） | LIGHT_RUST（:77，缺省关=vanilla） | 生产 mask 下不可达 | **有逆路径（间接）**：写回经 vanilla `enqueueSectionData` 进入 LightStorage/存档 NBT——可被 vanilla 重算覆盖，但无显式恢复命令；间接触达未做 xref 摸底（scout 诚实声明 #4 继承） | 单一总门；flag 1/2 走 defaultValue 构造（非 null） |
| 同 :750-751 | 写 | chunk.setLightOn(true) + TACS releaseLightTicket（light ticket 状态域） | LIGHT_RUST | 生产 mask 下不可达 | 有逆（vanilla light() 原路径本也执行同二操作，语义等价复刻 :749 注释） | 复刻原 light() 尾部 POST 语义 |
| 同 :566-573 `wgLightDomainWriteBack` | 写 | 双通道 nibble + setLightOn + 摘票 | LIGHT_DOMAIN（:79，缺省关） | LIGHT_RUST 且 LIGHT_DOMAIN 均开；生产 mask 下不可达 | 同上两行（同数据域、不同线程） | 域批任务线程执行；POST 严格在对应 chunk 算完后 |
| 同 :545 `Arrays.copyOf(b9,…)` | 读→写（内存快照） | blocks9 快照 | LIGHT_DOMAIN | 同上 | derived（新数组，提交线程侧快照，260916-01 崩溃修复） | 域任务零容器读 |
| 同 :644 降级重放调 `wgLightLegacyTakeover(…,false)` | 写 | 同 legacy 写回域 | LIGHT_DOMAIN 且域任务失败（:641 注释「不可达路径，loud fail」） | **静态标注前提不可达**（预检过但任务期失败=状态回退）——见 #98 自检 | 同 legacy | 防御分支 |
| 同 :231-245 `wgBetaDump` | 写 | **文件系统**：`<betadump>/<cx>_<cz>.bin`（packed 三段原始字节） | LIGHT_BETADUMP（缺省 null=关）**且须 LIGHT_BETAPROBE 同开**（:698 判据，一手核实） | 双探针门均开；生产 mask 下不可达 | derived（唯一临时区语义的新命名产物，天然合规 §9.8；逆=删除该目录文件） | 采集副作用面；块级一次，不进热路径 |
| 同 :762-766 native-throw 置位 | 写 | lightNativeDead 原子标志 | LIGHT_RUST | 生产 mask 下不可达 | **无逆-显式声明**：进程级一次性置位（compareAndSet false→true），无恢复路径，重启即消 | 置位后永久回退 vanilla |
| bench/LightDomainBatch.java:52-93 | 写 | DOMAINS/futures/chunks/blocks9s 内存登记表 | LIGHT_DOMAIN | 生产 mask 下不可达 | derived（进程内临时登记，seal 幂等 remove(key,st)；任务完即消） | 未注册回调 loud fail |
| BulkWb.java:163-311（java-core/src/main/java/wg/bench/） | 写 | chunk section **方块** PalettedContainer 原地替换 + 三派生计数 | `coreswap.bulkwb`（WgCompat.flag，缺省=**1.20.1 默认开**，"0" 显式关） | **生产 mask 下可达**（缺省开）⚠️ 本表唯一生产可达的写站点（跨面） | 有逆（写回原 section 内容？——未核，bulk 面归 B6 其他试点，此处只登记触点） | ⚠️ 非 light 面，但 light 读面（getSectionArray :263）与 bulk 写面共享 section 数组；是否入 light 声明域由主会话裁决 |
| WgCompat.java:14-17 `flag()` | 读 | sysprop 包装器 | — | — | derived | 「非 "0" 即真」语义 ≠ light 家族 `!= null` 语义（#19 家族风险注记）；light 面无直接消费点 |

## 三、Rust 读面

| 站点 file:line | 读/写 | 数据域 | 门控 | 前提条件（生产 mask 可达性） | inverse 状态 | 备注 |
|---|---|---|---|---|---|---|
| worldgen-core/src/light/mod.rs:106-109 `from_json_file` | 读 | **文件系统**：light_data.json（corewap-light-1 格式） | 无 sysprop（路径参数经 lightInit ← Java `coreswap.light.data`，缺省 hardcode mixin:82-83） | lightInit 被 Java 侧 `wgLightEnsureInit`（mixin:142-164）触达 ⇒ LIGHT_RUST 开；生产 mask 下不可达 | derived（只读） | **light 面唯一 Rust 文件读**（F4 一手核实：mod.rs:106-109 只读 light_data.json；opacity/emission 按 raw block id，:122-133；**不读 blocks.json**） |
| mod.rs:112-139 `from_json_str` + :164-174 `parse_u8_field` | 读 | opacity/emission 查找表（raw id 键，动态扩容） | 无 | 同上 | derived | 负 opacity clamp 0（260905-05，:166-169 注释，无 VoxelShape 有损边界） |
| mod.rs:143-148 `lookup` | 读 | 表（缺 id/负 id→default=table[0]） | 无 | 同上 | derived | air_fast 前提 table[0]==(0,0)（:136） |
| mod.rs:182-192 `light_compute` / :308-338 `light_compute_domain` / :340+ `light_compute_inner` | 读 | blocks9/blocks25 输入数组 | 无（门在 Java 侧） | LIGHT_RUST 开；生产 mask 下不可达 | derived | 纯函数 + per-call Scratch（:189-191）；无世界/文件访问 |
| mod.rs:199-283 区段 packed 解码（`packed_to_blocks9`） | 读 | packed meta/pal/sto（Java 预拆帧） | 无 | LIGHT_RUST 开且缺省（!BLOCKABI）packed 路触达；生产 mask 下不可达 | derived | 解码失败 PackedDecode→JNI rc=-2 |
| jni_bridge.rs:315-327 `lightInit` | 读 | 文件路径字符串→LightEngine | 无 | LIGHT_RUST 开（mixin:149 调用）；生产 mask 下不可达 | derived | 失败返 0→Java 回退 vanilla（mixin:155-158） |
| jni_bridge.rs:346-… `lightCompute`（domain/packed 同族 :380+/424+/475+ 未逐行核，标「未核」） | 读 | JNI 数组（blocks9/blocks25/meta/pal/sto） | 无 | 同上 | derived | catch_unwind 兜底 rc=-3（:344 注释），panic 不跨 FFI |

## 四、Rust 写面

| 站点 file:line | 读/写 | 数据域 | 门控 | 前提条件（生产 mask 可达性） | inverse 状态 | 备注 |
|---|---|---|---|---|---|---|
| mod.rs `light_compute_inner` export 段（经 :191/:293/:335 调用） | 写 | out_block/out_sky（24×2048 B 双通道）+ out_flags（48 B：0/1/2 均质旗标） | 无（Java 侧 LIGHT_RUST/LIGHT_DOMAIN 决定触达） | 生产 mask 下不可达 | derived（写 JNI 缓冲→Java out 数组→enqueueSectionData；**不直接写世界**） | flag 语义 mod.rs:180-181 |
| jni_bridge.rs:331-340 `lightDestroy` | 写（内存） | Box 回收 | 无 | Java 进程关闭/卸载路径；生产 mask 下（vanilla）不触达 | derived | handle=0 no-op（:335） |
| bin-diag（light_bench*/light_golden_dump/light_probe_260914，worldgen-core/src/bin-diag/） | 写 | 诊断 stdout/golden 文件 | 无 sysprop（独立 bin，不参与默认构建） | **生产 mask 下结构性不可达**（不在默认构建产物内） | derived（§八.13 隔离区新命名产物） | 不进生产触达面 |

## 五、门控总表（sysprop 一手消费点直读，本 worker 逐行复核）

| sysprop | 消费点 file:line | 形态 | 默认方向 | 备注 |
|---|---|---|---|---|
| coreswap.light.rust | mixin:77 | `!= null` | **关（vanilla）** | 单一总门（一手核实） |
| coreswap.light.domainbatch | mixin:79 | `!= null` | 关 | 域批 A/B 臂 |
| coreswap.light.domainbatch.gracems | LightDomainBatch:21 | Integer.getInteger | 10_000 | 数值门，非存在性门 |
| coreswap.light.data | mixin:82-83 | String（带缺省值） | hardcode 路径 | light_data.json 位置 |
| coreswap.light.timing | mixin:104 | `!= null` | 关 | chunk 级计时 |
| coreswap.light.paldump | mixin:105 | Integer.getInteger | 0=关 | 消费点 mixin:279/:757 + LightPalDump 反射面 |
| coreswap.light.betaprobe | mixin:115 | `!= null` | 关 | P-β/P-path 探针 |
| coreswap.light.betadump | mixin:120 | String | null=关 | **须与 betaprobe 同开才生效**（:698 一手核实）；文件写域 |
| coreswap.light.oldcollect | mixin:126 | `!= null` | 关 | 强制旧收集路（blocks9 旧采） |
| coreswap.light.blockabi | mixin:133 | `!= null` | **缺省（未设）= packed ABI 路**（F3：:684 `if (!LIGHT_BLOCKABI)` 走 packed）；设值 = blocks9 A/B 回退 | 语义已消歧：无反转，命名语义=「用 blocks9 ABI」；:132 注释「候选 C packed ABI」与消费点取反并存，注释按 F3(b) 解读为历史残留 |
| coreswap.bulkwb | BulkWb:90 经 WgCompat.flag | 「非 "0" 即真」 | **1.20.1 默认开** | 跨面触点（bulk 写 section / light 读 section） |
| coreswap.bulkwblog / wbcontent / bulkwbtest / bulkwbsentinel | BulkWb:91-97 | `!= null` | 关 | bulk 面诊断门（跨面注记） |

- build.gradle `-P` 映射侧未核对（B6-1 check_switch_mapping.py 职责域），Degraded。

## 六、声明汇总（light 面整体 reads: / writes: 集合，一行一域）

**reads:**
- `chunk.blocks`（section 方块，三收集形态：blocks9-fast / blocks9-old / packed——同一 chunk 内可互为回退，F2）
- `chunk.status`（ChunkStatus ≥FEATURES 检查，只读）
- `world.heightparams`（bottomY/-64、section span/24）
- `Block.STATE_IDS`（id→rawId+lum 编码表）
- `sysprop.light.*`（门本身，11 个消费点）
- `fs:light_data.json`（唯一 Rust 数据文件读；**不含 blocks.json**，F4）
- `diagnostic:palette-private-record`（反射读 PalettedContainer.Data，paldump 探针专用）

**writes:**
- `light.blocklight.nibble`（enqueueSectionData BLOCK ×24，legacy/domain 双形态）
- `light.skylight.nibble`（enqueueSectionData SKY ×24，同上）
- `chunk.lightOn + ticket`（setLightOn(true)+releaseLightTicket，POST 语义）
- `mem:domain-registry`（LightDomainBatch 登记表 + blocks9 快照，进程内）
- `mem:nativeDead-flag`（进程级原子一次性标志，无逆-显式声明）
- `fs:betadump/<cx>_<cz>.bin`（双探针门控的诊断文件写）
- `rust.out`（out_block/out_sky/out_flags JNI 缓冲，不直接写世界）
- （跨面已裁决（judge S3 回写，主会话 260918-06）：**不并入 light 域**——辖区按面划分，light 声明只管光照数据域；`chunk.blocks.section` 共享数组归 bulk/写回面 + R9-b「单写者不变量」（#143）论证面。⚠️ **并发义务缺口登记**：R9-b 现有 BulkWb SENTINEL 只覆盖 writer×writer，**不含 light reader × bulk writer**——该义务归属 R9-b 面，列为下块待办，本表不承担。）

---

## ① 诚实声明

1. **全部静态读取，零运行验证**（未跑 server/探针/JNI）——整表 Degraded，所有行 draft。
2. **F3/F4 为继承事实但已一手复核**：mixin:133/:684、mod.rs:106-109 本轮逐行读过，与主会话消歧一致。
3. **未核面**：`jni_bridge.rs:360-566`（domain/packed JNI 体未逐行读，行号承 scout 图 + :346 头部一手核）；`LightPalDump.java` 全文（仅门控与反射头）；`FormProbe.java`、`BenchMod.java`；1.21.6 侧全部；BulkWb.java:163-311 方法体（只核门控行 :90-97 与写域描述，写回逆路径**未核**）；knowledge/INDEX.md 与 discovered #98/#102 原文（本 worker 未读，#98 自检按任务书给定口径执行）。
4. **间接触达未摸底**：写回→vanilla LightStorage→持久化 NBT 的 xref 链未做（scout #4 继承）；heightmap：light 面未发现直接读写（sky 用自算 col_max，mod.rs:87）——声明集合不含 heightmap，为「未发现」级而非「证明不存在」级。
5. **默认方向 = 静态直读**；运行时实际生效值未验证（sysprop 可被启动参数覆盖）。

## ② 「实际触达 ⊆ 声明」比对器比对键建议

比对键 = 三元组规范化字符串 `light:<数据域>:<r|w>:<站点规范名>`；站点规范名取 file 基名+符号名（**不含行号**——行号随演化漂移，域+符号稳定）。运行时触达证据源：`[LIGHT-PATH] path=`（domain/legacy-rust/legacy-fallback/vanilla-init0，mixin:780-801）与 `[LIGHT-BETA] abi=`（packed/blocks9，:696/:722）日志逐 chunk 归约成触达键集合，再校验 ⊆ 本表 reads∪writes 键集。

```python
# 伪代码骨架（供主会话实现）
DECLARED = load_keys(declaration_md)          # 解析六节汇总表 → {"light:chunk.blocks:r:Mixin#wgLightCollectBlocks", ...}
ALIASES  = {                                   # 运行时事件 → 声明键映射（一张事件可以触达多键）
  "path=domain":        ["light:light.blocklight.nibble:w:Mixin#wgLightDomainWriteBack",
                         "light:light.skylight.nibble:w:Mixin#wgLightDomainWriteBack",
                         "light:chunk.lightOn+ticket:w:Mixin#wgLightDomainWriteBack", ...],
  "abi=packed":         ["light:chunk.blocks:r:Mixin#wgLightCollectPacked",
                         "light:fs:light_data.json:r:mod.rs#lightInit", ...],
  "abi=blocks9":        ["light:chunk.blocks:r:Mixin#wgLightCollectBlocks", ...],
}
def actual_keys(log_lines):
    ks = set()
    for ln in log_lines:
        for pat, keys in ALIASES.items():
            if pat in ln: ks |= set(keys)
    return ks
unknown = actual_keys(logs) - DECLARED
report("UNDECLARED", unknown)   # 非空 = 声明表漏登记（比对器唯一 FAIL 判据）；空 = 实际触达 ⊆ 声明
```

## ③ #98 家族消解力自检（「生产门控下前提不可达」标注行）

本表携带「生产 mask 下前提不可达」标注的行（静态标注，非运行证明）：

1. **mixin:644 域批降级重放分支**（二节第 5 行）——源码注释 :641 自证「状态回退，**不可达路径**，loud fail」：唯一一处源码文本级自证的不可达前提行。
2. **bin-diag Rust 诊断 bin**（四节第 3 行）——**结构性不可达**（不在默认构建产物内）。
3. **fs:betadump 写**（二节第 6 行）——双探针门（betaprobe ∧ betadump）生产 mask 下均关，前提不可达（诊断面，属「门控关闭不可达」非「状态机不可达」）。
4. **整面因 LIGHT_RUST 缺省关而不可达**——这是总门缺省，不是 #98 意义上的「前提不可达」行级标注（否则全表皆是，无消解力）；本表行级标注只对「门开之后仍不可达」的分支生效，即上面 1-3。

**#102 terrain_cache 形态**：属另一接管面（terrain/domain face），不在 light 面声明范围内——light 面无 #102 对应实例，**如实登记「无对象」，不凑数**（且 #102 原文未读，见诚实声明 3，此处不对其下任何判定）。
