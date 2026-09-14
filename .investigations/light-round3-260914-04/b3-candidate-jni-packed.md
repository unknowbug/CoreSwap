# .b3 候选 C：JNI 传参改形（palette + packed storage 直传）

> 角色：core.worker（fan-out P2）。日期标签：260914-04。**status: draft**（静态设计 + 一手源核对，未运行验证，未编译）。
> 任务：候选 C 候选设计（不实施）。前提核对基于 `.investigations/light-round3-260914-04/scout-map.md`（下称 scout）。
> 声明：本候选只交付设计 + 上限推演 + 风险面；e2e 绝对值为推演预期（带宽假设显式声明），非测量值。

---

## 1. JNI 新 ABI 设计

### 1.1 新导出函数签名

Java 侧（`wg.CppWorldgen`，与现 `lightCompute` 并存）：

```java
// 每节 packed 形态：sectionMeta[2*s]=bits(0..16)、sectionMeta[2*s+1]=paletteLen；
// bits==0 && paletteLen==0 → 空节哨兵（全 AIR）；bits==0 && paletteLen==1 → singlet 均质节（无 storage）；
// bits>=4 → storageAll 顺序拼接各节 long[]，节内 long 数 = ceil(4096 / (64/bits))。
static native int lightComputePacked(long handle,
        int[] sectionMeta,   // 216*2 = 432 int（9 chunk × 24 节 × 2）
        int[] paletteIds,    // 各节 palette 扁平拼接：Σ paletteLen 项，每项 = rawId | (luminance<<24)
        long[] storageAll,   // 各节 PackedIntegerArray.getData() 拼接（空/singlet 节贡献 0 项）
        byte[] outBlock, byte[] outSky, byte[] outFlags);  // 出参三件同现 ABI
```

Rust 侧（`versions/1.20.1/rust/src/jni_bridge.rs`，在现 `Java_wg_CppWorldgen_lightCompute`（jni_bridge.rs L334-401）之后新增）：

```rust
#[unsafe(no_mangle)]
pub extern "system" fn Java_wg_CppWorldgen_lightComputePacked<'frame>(
    mut unowned_env: EnvUnowned<'frame>, _class: JClass, handle: jlong,
    section_meta: JIntArray, palette_ids: JIntArray, storage_all: JLongArray,
    out_block: JByteArray, out_sky: JByteArray, out_flags: JByteArray,
) -> jint
```

返回码沿用现约定（jni_bridge.rs L332：0 成功 / -1 handle 空 / -2 长度错 / -3 panic / -4 JNI 错）。

### 1.2 section 元数据布局与哨兵

- 每节 2 int：`[bits, paletteLen]`。节序 = 现 blocks9 节序（chunkIdx=dz*3+dx，节内 s 0..23，mixin L110/L137-160 同构）——Rust 侧展开后落位与现 ABI 逐节对齐，出参/flags 布局零改动。
- **空节哨兵** `bits=0, paletteLen=0`：对应 mixin 现路 `sec.isEmpty()` → `Arrays.fill(0)`（mixin L162-166）；Rust 侧对目标区间直接零填（等价论证见 §4.2）。
- **singlet 均质节** `bits=0, paletteLen=1`：对应 `SingularPalette + EmptyPaletteStorage`（PalettedContainer.java L424/L400——bits==0 时 storage 即 EmptyPaletteStorage，`getElementBits()==0`、`getData()` 空数组）；Rust 侧用 paletteIds 单项整节填充。
- **bits≥4**：storage long 数 = `ceil(4096 / elementsPerLong)`，`elementsPerLong = 64/bits`（PackedIntegerArray.java L261/L266）；Rust 用运行游标切 storageAll。

### 1.3 paletteIds 编码（rawId|lum 打包规则）

- 沿用现 blocks9 单元 ABI：**低 24 位 = `Registries.BLOCK.getRawId(st.getBlock())`，高 8 位 = `st.getLuminance()`**（mixin L173-175）——Rust 内核 `fill` 的位拆（mod.rs:230-231）与 air 快路径 `v==0`（mod.rs:226-229）语义原样保留，内核零改动即可吃 packed 输入的展开结果。
- Java 侧每 palette 项（≤ 数十/节）做一次 `palette.get(i)` → `rawId|lum` 预编码，**消 4096×非空节数 的逐格 getBlockState/getRawId/getLuminance**（mixin L169-178）→ 降为 ΣpaletteLen 次（≈ 每节 2-15 项，scout §1.2）。

### 1.4 旧 ABI 并存回退策略

- `lightCompute`（jni_bridge.rs L334）**原样保留**，作为回退路径与对拍基准（双采集对拍 #48 的"旧路"侧）。
- 切换 = Java 侧静态属性（对齐现 gate 惯例，如 `[LightRust].packed=1` 默认关）：关 → 走旧 blocks9 路；开 → 走 packed 路，任何长度校验失败（rc=-2）打 `wgLightFallback` 回退 vanilla（mixin L100-106 惯例）。gate 默认关 ⇒ 出厂零风险（架构文件 §6 回退条款）。
- 注意：切换属**收集路二选一**，与 `LIGHT_RUST` 总 gate 正交（总 gate 关 = 完全 vanilla，mixin L188）。

### 1.5 四层改动面清单

| 层 | 文件 | 改动 |
|---|---|---|
| mixin 收集段 | `versions/1.20.1/java/.../ServerLightingProviderMixin.java` L160-179 | 非空节改读 `data.palette()/storage()`（via 新 accessor），写 sectionMeta/paletteIds/storageAll；TL 缓冲三件新增（~432 int + ≤~16K int + ≤~2MB long 上限，实心 bits4 时 ~48KB/节集）；空节写哨兵 |
| 新 accessor | 新文件 `PalettedContainerAccessor.java`（`@Accessor("data")`，返回 `PalettedContainer.Data<BlockState>`；record 访问器 `palette()`/`storage()` 公有免 accessor，PalettedContainer.java L361） | 先例：ChunkSectionAccessor 已 @Accessor blockStateContainer 成功交付（scout §1.2） |
| jni_bridge | `versions/1.20.1/rust/src/jni_bridge.rs` L334 后 | 新导出 `lightComputePacked`：长度校验（sectionMeta=432 / 出参三件同现）→ TL 缓冲拷入（get_int_array_region ×2 + get_long_array_region ×1，量级 ~KB 级）→ 调 worldgen-core 新入口 → 出参 set 三件复用 L391-393 |
| worldgen-core | `worldgen-core/src/light/mod.rs`（新入口 `light_compute_packed`） | 见 §2：解码 + 展开 + 融合 fill（吸收候选 A 的 fill 面） |

---

## 2. 位序核对（最高优先）——一手源结论

### 2.1 静态结论：**元素不跨 long 边界；小端序；索引序与 blocks9 一致**

一手源 `.tmp/scout-260905-08/mcsrc/net/minecraft/util/collection/PackedIntegerArray.java`：

1. **不跨界**：`elementsPerLong = 64 / elementBits`（L261，整除下取整）——每 long 恰装整数个元素，元素**永不跨 long**；"跨 long 边界"形态不存在，尾部余数元素单独在末 long 低位（L266 `j = ceil(size/epl)`，writePaletteIndices L361-369 同构）。⇒ 解码器无跨 long 拼接分支，风险面比预期小。
2. **long 内小端序**：`get(index)`（L307-313）= `data[index/epl] >> ((index % epl) * elementBits) & maxValue`——第 0 个元素在 long 最低位，索引递增向高位推进。`swap`/`set`（L285-304）同一位移公式。构造器（L223-250）从 int[] 打包时 `k` 从高往低 `l <<= bits`，与 get 公式自洽。
3. **除魔数**：`getStorageIndex`（L278-282）用 INDEX_PARAMETERS（L20-213）魔数除法，数学上 ≡ `index / elementsPerLong`（常数除法优化，注释 L8-19）。Rust 侧直接用原生除法即可，无需复刻魔数表。
4. **索引序对齐**：`computeIndex(x,y,z) = (y<<4|z)<<4|x = y<<8|z<<4|x`（PalettedContainer.java L465-467，BLOCK_STATE paletteProvider edgeBits=4，L420/L445-446）——与 blocks9 节内布局 `(y+64)*256+z*16+x` 的节内序（mixin L110/L168-171 ly/z/x 循环）**逐位一致**。scout 开放问题 1 静态闭合（仍须运行对拍，见 2.3）。

### 2.2 Rust 解码器伪代码（bits∈{4,5..8,9..15/16} 全形态）

```rust
// per section (s), 输入: bits, palette: &[i32](rawId|lum), packed: &[i64], out: &mut [i32](4096)
let epl = 64 / bits;                    // bits>=4 恒成立（§4.3 协商规则）
let mask: u64 = (1u64 << bits) - 1;
let mut idx = 0usize;
for &l in packed {                      // ceil(4096/epl) 个 long
    let mut v = l as u64;
    for _ in 0..epl {
        if idx == 4096 { break; }       // 尾部余数 long 的高位垃圾不取
        let p = (v & mask) as usize;
        out[idx] = palette[p];          // 融合形态：此处直接查 opacity 表 + 写 scratch + col_max（吸收 A 的 fill 面）
        v >>= bits;
        idx += 1;
    }
}
// bits==0 分支：哨兵/均质，见 §1.2
```

注意：bits≥9 时 palette 为 ID_LIST（全局 state id，PalettedContainer.java L427），但 Java 侧已把 palette 项统一预编码成 rawId|lum——Rust 不感知 palette 类型，**解码器对 4..15/16 形态统一**。bits 上界 = `ceilLog2(Block.STATE_IDS.size())`（L427）——1.20.1 具体 state 数未静态读出（**open**，Java 侧 dump 一次即可定；预计 15-16，Rust 解码器按 bits≤16 泛化无成本）。

### 2.3 必须运行对拍的点（golden 门）

- **解码器单元 golden**：Java 侧 dump 每节 `storage.writePaletteIndices(int[4096])`（PackedIntegerArray.java L346-370，mojang 权威展开）+ palette 映射后的 blocks9 期望值；Rust 解码展开逐 int 对比（4 chunk × 216 节全量）。这是位序断言的**唯一运行级证据**，静态结论（§2.1）不替代。
- golden 逐位不变（4 用例 FNV，C4 惯例）+ 双采集对拍（#48：packed 路 vs 旧 blocks9 路 4×MATCH）。

---

## 3. 成本上限推演（#124 先量后改）

**显式带宽假设**：单核有效内存带宽 ~10GB/s 量级（DDR4/5 单核流访问的保守量级；未实测，**open**——用作上限推演的分母，非测量值）。

现 ABI 每 chunk 传输/写侧环节（scout §1.2/§1.3）：

| 环节 | 量 | 10GB/s 下上限 |
|---|---|---|
| ① Java 写 blocks9（非空节逐格 + 空节 fill） | 3.5MB 写 + 逐格间接成本 | ~0.35ms 带宽 + getBlockState 间接（B 候选主战场） |
| ② JNI `get_int_array_region` 拷/pin | 3.5MB（pin 或拷实现相关，scout open 3） | ~0.35ms（若拷） |
| ③ Rust fill 读 blocks9 | 3.5MB 读 | ~0.35ms |
| ④（对照）Java 收集逐格调用 | 4096×非空节数 次 getBlockState+2 注册表查询 | 间接跳转主导，带宽模型不覆盖 |

C 直接消 **①②③ 的字节量**（→ ~50-250KB/chunk：paletteIds ≤~3k int≈12KB + storage bits4×24节×4096×0.5B≈48KB/实心，bits≥9 稀疏节最大 ~8KB/节）——三环节带宽上限合计 ~1.05ms/chunk → **C 的传输/写侧净收益上限 ≈ 1.0ms/chunk**（①中带宽份额 + ② + ③；①的逐格间接成本归 B/C 共有消除面）。

**关键限定（任务书强调）**：Rust 侧展开仍是逐格成本（decode+查表+写 scratch），不因 packed 而消失；scratch 三块 2.65MB memset（mod.rs:195-203）也不变。C 的净收益 = 传输与写侧，不是展开侧。且 round2 e2e 口径显示 gap 3.5s ≈ 内核份额 3.2s（12-lighting.md L94）——即 ①② 实测已被 round2 的 section 直采 + TL 缓冲大幅吃掉，**C 的独立增量预期应按上限的 0.5-1.0ms/chunk 折半估计**（部分环节 round2 后实测接近零，12 篇"Java 收集/JNI 侧开销已基本消除"）。

**e2e 绝对值预期（对照回退 3.5s，ON 17.4/OFF 13.9）**：C 独立（不带 A）≈ 省 0.5-1.0ms×2025 ≈ **1.0-2.0s**；C+A（融合展开+表内联，fill 面合并）才有机会把 3.5s gap 压到噪声带（1.05× 达标线 ≈ 0.7s 余量）。**结论：C 单独不足以达标，必须与 A 组合。**

---

## 4. 等价性论证

### 4.1 展开等价（palette 映射 vs getBlockState）
`getBlockState(x,y,z)` → `data.palette.get(data.storage.get(computeIndex(x,y,z)))`（PalettedContainer.java L180-187）——C 的展开 = 同一公式逐格施加（storage.get 即 §2.1 位解码，palette.get 由 Java 预编码表替代）。**构造性等价**，无语义分支。

### 4.2 空节 / singlet 等价
- 空节：`sec.isEmpty()` ⇔ 全节 air（ChunkSection 计数语义）⇔ 现路 `Arrays.fill(0)`（mixin L162-166 注释已对齐 vanilla WorldChunk#getBlockState 空节返回 AIR）；哨兵零填充 = 同值。**注意 AIR 的 luminance==0、rawId==0，v==0 满足 air 快路径**（mod.rs:226）。
- singlet：bits==0 时 provider 给 SINGULAR + EmptyPaletteStorage（L424/L400）——整节同态，`paletteIds[单值]` 填充 = getBlockState 恒返同态。⚠️ 唯一须防的陷阱：**singlet 态可能是非 air 光源**（如全 glowstone 玩家放置层？天然地形罕见但 ABI 必须支持）——故 §1.2 哨兵必须区分 paletteLen==0（AIR）与 ==1（均质任意态），不能合并。

### 4.3 bits 协商形态全覆盖（一手源引用）
BLOCK_STATE PaletteProvider（PalettedContainer.java L420-430）：请求 bits 0 → SINGULAR/0；**1-4 → 一律 ARRAY/bits=4**（L425——即 storage 实际 elementBits 恒为 4，不存在 1/2/3 形态）；5-8 → BI_MAP/bits=5..8（L426）；≥9 → ID_LIST/ceilLog2(stateIds.size())（L427）。⇒ **运行时以 `storage.getElementBits()` 为权威**，Java 侧读实值传 sectionMeta，Rust 永不猜测——协商逻辑零复刻，天然全覆盖。bits 可能值域 = {0} ∪ {4..15/16}。

### 4.4 volatile 读时序前提
`data` 为 volatile（PalettedContainer.java L36），`onResize` 换 Data 对象整体替换（L133-139）——Java 侧**每节一次快照读**（局部变量持 Data 引用，再取 palette/storage），要么旧 Data 要么新 Data，不会读到撕裂的 palette/storage 组合（record 字段 final）。前提：LIGHT 阶段邻 chunk 已静置（scout §1.2 静态推断，未运行验证——**open**，双采集对拍可顺带覆盖：对拍期两次读 data 快照 hash 一致即可实证）。

---

## 5. 实施风险与改动面

### 5.1 四层风险
1. **位序**：静态已闭合（§2.1），残余风险 = 尾部余数 long 高位垃圾位（伪代码已防）与 bits 上界（open）。golden 单元对拍兜底。
2. **空节/singlet 形态**：哨兵设计已区分（§4.2）；风险 = Java 侧把 `getElementBits()==0` 误判为空节（正确判法 = 先查 `sec.isEmpty()` 再读 data，与现路同序）。
3. **JNI 长度校验**：storageAll/paletteIds 长度与 ΣsectionMeta 一致性须 Rust 侧逐节游标校验，失配返 -2 回退（防恶意/漂移布局）。
4. **1.21.6 波及**：`versions/1.21.6/rust/src/jni_bridge.rs` 与 1.20.1 同构（本轮 grep：L334 lightCompute/L283 LIGHT_BLOCKS9_LEN 同行号同构）——新导出双版本各加一份薄壳（参数校验+TL 拷入），解码核在 worldgen-core/light/mod.rs 单源共享；1.21.6 只跑 sanity（golden + gate ON 冒烟，架构文件 §6）。

### 5.2 与 A/B 组合关系
- **C 吸收 A 的 fill 面（部分）**：Rust 侧「decode → palette 小表 → opacity 查表 → col_max」天然单趟融合（palette ≤16 项常驻 L1），替代现「读 blocks9 i32 → lookup」——A 的「fill 降维」在 C 形态下自动获得；**A 的 opacity 表内联（消 get().unwrap_or，mod.rs:123）仍独立有效**，两者正交可叠加。
- **B 是 C 的 Java 侧子集/踏脚石**：B = 同 accessor 链 + Java 侧展开，仍写 3.5MB blocks9 + 3.5MB JNI 拷（③仍在）；C = 展开搬到 Rust，①②③全消。共享件 = accessor + palette 预编码 + 双采集对拍设施。
- **路线对比**：**先 B 后 C** = Java-only 小 diff、可独立验证 accessor 链与位序（B 的 Java 解码器与 §2.1 同一断言），再上 C 时风险已拆解；代价 = 两次 e2e 迭代、B 自身收益可能近零（round2 后 Java 侧已"基本消除"，§3）——**B 作为独立优化大概率不划算，只作为 C 的 de-risk 步骤**。**直接 C** = 一步到位，风险集中在位序 + 四层同调，golden 单元对拍（§2.3）先行即可压住。**建议直接 C**（B 的验证价值可由「Java 侧 dump 期望值对拍 Rust 解码器」吸收，无需实施 B）。
- 排序建议（供 judge/用户）：**C+A 组合 > C 单独 > A 单独 > B**（C 单独不达标 §3；B 独立收益最薄）。

### 5.3 判据预登记建议
1. **解码器单元 golden**：dump `writePaletteIndices` × 216 节 × 4 chunk，Rust 展开逐 int MATCH（0 diff）——位序断言运行级证据（§2.3）。
2. **双采集对拍**（#48 惯例）：packed 路 vs 旧 blocks9 路，4 chunk × 884736 项 4×MATCH；对拍后移除诊断复编。
3. **golden 逐位**：4 用例 FNV 全等（C4 惯例，现 rlib 现场重冻）。
4. **JNI 段计时**：chunk 级一次（禁逐格诊断，探针污染铁律），收集段/packed 传入/内核分相，env 门控默认关。
5. **e2e 四臂交替**：判据与噪声带同时预登记（#103/#119），达标线 ON/OFF 中位比 <1.05×。
6. 1.21.6 sanity：golden 双版本 + gate ON 冒烟。

## 6. 开放问题（open 清单）
- bits 上界 = ceilLog2(Block.STATE_IDS.size()) 具体值（1.20.1 静态未读出，须 dump；预计 15-16）。
- `get_int_array_region` pin vs 拷（scout open 3）——只影响 §3 上限的 ② 是否全额兑现，不影响 C 方向成立。
- 非 air singlet 节在真实 pregen 中的出现率（影响 §4.2 哨兵分支的实际命中，非正确性问题）。
- volatile 静置前提的运行级实证（双采集对拍顺带覆盖，§4.4）。

## 7. 自检声明
- 本文件为候选设计（draft），未实施、未编译、未运行；所有行号为本轮实际读出（read 工具）。
- 一手源：PalettedContainer.java / PackedIntegerArray.java（.tmp/scout-260905-08/mcsrc/，yarn sources）、jni_bridge.rs（两版本）、ServerLightingProviderMixin.java、light/mod.rs（行号转引自 scout，未重读 mod.rs 原文——标 🔗 转引：mod.rs:57/89-95/119-124/195-203/209-247 均出自 scout §1.1）。
- 带宽假设显式声明（§3），e2e 绝对值为推演预期非测量。
