# 候选 B：Java 侧 palette 级批量展开（fan-out .b2）

> 角色：core.worker（P2 fan-out 候选设计，不实施）。工作块 260914-04。
> 状态：**draft**（静态推演 + 一手源核对，未运行验证）。
> 一手源：`.tmp/scout-260905-08/mcsrc/`（yarn 1.20.1 sources，下称 PC.java / PIA.java）、`versions/1.20.1/java/src/main/java/wg/bench/mixin/ServerLightingProviderMixin.java`（下称 Mixin）、`versions/1.20.1/docs/12-lighting.md`（下称 12篇）。

## 0. 结论速览

- **位序可纯静态闭环**（§1）：PackedIntegerArray 元素 LSB-first、long 内对齐不跨界、索引序 = computeIndex = y<<8|z<<4|x，与 blocks9 段内 (ly,z,x) 行主序**逐元素一致**。Java 移位循环可等价复刻，无需魔数（魔数 = 常数除法，见 §1.3）。对拍探针仍列为 SHOULD（保险带，非 MUST）。
- **成本上限（#124 口径）**：round2 后 Java 收集段残余 ≈ **0.3s/2025 chunks（减法推断口径，未直接计时，标 open）**；B 上限消其 60-75% ≈ **e2e 绝对值 ~0.2s**（17.4→~17.2s，1.25×→~1.235×）——**单靠 B 不达 ≤1.05× 判据**，属「小而确定」收益，与 A（内核 943µs fill）非同量级。但若 §2.3 的直接计时推翻 0.3s 减法口径，上限上修。
- **最大风险**：volatile data 快照一致性（§4.3，一次性局部读可闭合）与 ID_LIST 大 bits 形态（≤15，静态覆盖）。

## 1. 位序核对（最高优先——结论：静态闭环成立）

### 1.1 索引序（PC.java）

- `PalettedContainer.get(x,y,z)` → `get(computeIndex(x,y,z))`（PC.java:180-182 → 184-187：`data.palette.get(data.storage.get(index))`）。
- `computeIndex = (y << edgeBits | z) << edgeBits | x`，BLOCK_STATE provider edgeBits=4（PC.java:465-467 + 420）⇒ **index = y<<8 | z<<4 | x**。
- Mixin 现收集路逐格序 = `ly,z,x` 三重循环（Mixin:169-178），blocks9 段内偏移即 y<<8|z<<4|x。**旧路 getBlockState 本身就按此序取**，故 palette 解码按同序填 `blocks9[k++]` 即逐元素对齐——索引序闭合。

### 1.2 位 packing（PIA.java）

`PackedIntegerArray.get(index)`（PIA.java:307-313）：

```java
int i = getStorageIndex(index);          // = index / elementsPerLong
long l = this.data[i];
int j = (index - i * elementsPerLong) * elementBits;
return (int)(l >> j & this.maxValue);    // maxValue = (1L << elementBits) - 1（PIA.java:260）
```

- **LSB-first**：构造侧打包（PIA.java:228-249）从低位到高位顺序放 element j..j+epl-1；`writePaletteIndices`（PIA.java:346-370）独立同证（逐 long `l & maxValue; l >>= elementBits`）。
- **不跨 long**：`elementsPerLong = 64 / elementBits`（PIA.java:261），`offset = (index mod epl) * bits`，且 `bits*epl ≤ 64` ⇒ 每元素完整落在单个 long 内，无跨界拼位。bits=4 时 epl=16 恰好 64 位填满；bits=5（epl=12×5=60）等形态 long 尾部留空位，解码不受影响。
- `getData()` 直接返回内部 long[]（PIA.java:316-318），`getElementBits()` 返回 bits（PIA.java:326-328）——Java 侧可直接取原始位流。

### 1.3 魔数核对与「免魔数」论证

`getStorageIndex`（PIA.java:278-282）= `(index * scale + offset) >> 32 >> shift`，常数取自 `INDEX_PARAMETERS[3*(epl-1) + 0/1/2]`（PIA.java:20-213, 262-265）。抽验：

- bits=4 → epl=16 → k=45..47（PIA.java:66-68）= (Integer.MIN_VALUE, 0, 3)：index×2³¹ >>35 = index/16，验 index=16→1 ✓（无符号读法，PIA.java:279-280 `toUnsignedLong`）。
- bits=5 → epl=12 → k=33..35（PIA.java:54-56）= (357913941, 357913941, 0)：≈2³²/12 ✓。
- bits=16 → epl=4 → k=9..11（PIA.java:30-32）= (MIN, 0, 1)：index×2³¹>>33 = index/4 ✓。

**结论**：魔数实现的正是「常数 epl 的精确无符号除法」。Java 复刻**无需复刻魔数**——`index / epl`、`index % epl`（epl 为循环不变量常数）在 JIT 下编译为等价乘移序列，语义与 getStorageIndex 在 0..4095 域恒等（open 级残余风险极低，§1.5 对拍覆盖）。

### 1.4 解码循环设计（伪代码）

对每非空节（bits = storage.getElementBits()；本方案只在 storage instanceof PackedIntegerArray 时走此路）：

```
// 一次性预处理（每节）：
Data<BlockState> d = accessor.getData();            // volatile 读一次，局部快照（§4.3）
Palette<BlockState> pal = d.palette();              // record 公有访问器（PC.java:361）
long[] words = ((PackedIntegerArray) d.storage()).getData();
int bits = d.storage().getElementBits();
int epl = 64 / bits;  long mask = (1L << bits) - 1;
int psz = pal.getSize();
int[] tab = new int[psz];                          // rawid|lum 预编码小表
for (i in 0..psz-1) { BlockState st = pal.get(i);
                      tab[i] = Registries.BLOCK.getRawId(st.getBlock()) | (st.getLuminance() << 24); }

// 主解码（4096 项，序 = y<<8|z<<4|x = 数组自然序）：
for (idx = 0; idx < 4096; idx++) {
    int w = idx / epl;                             // 常数除法，JIT 乘移化
    int v = (int)(words[w] >>> ((idx - w * epl) * bits)) & mask;   // mask 前先 & 0xFFFF... 由 mask 收
    blocks9[k++] = tab[v];
}
```

**bits 形态覆盖**（BLOCK_STATE provider，PC.java:420-430）：
- bits=0（SINGULAR，1 项）→ storage 是 `EmptyPaletteStorage` **不是** PackedIntegerArray（PC.java:400）——**必须特判**：`Arrays.fill(blocks9, k, k+4096, tab[0])`（§4.2）。
- bits=4（ARRAY，1-15 项）→ epl=16。
- bits=5..8（BI_MAP）→ epl=12/10/9/8（64/9=7，即 bits=9 仅出现在 ID_LIST 协商后；BI_MAP 上限 8）。
- bits=9..15（ID_LIST，`ceilLog2(idList.size())`，1.20.1 方块状态注册表 rawId 域 >2¹⁴ → 通常 15）→ epl=7..4，通式成立（open：真实 ID_LIST 节占比极低，dump 探针 §3 顺带观测）。

### 1.5 对拍探针（SHOULD，保险带）

静态已闭环，但按 #48 惯例仍建议在实施时保留临时双采集（前 4 chunk）：新 palette 路 vs 旧 `getBlockState` 路，逐元素 int 对比 98304 项，4×MATCH 后移除并复编（12篇:93 先例形态）。探针即「位序 + 特判面」的运行时兜底，成本一次性。

## 2. 成本上限推演（#124 先量后改）

### 2.1 round2 后 Java 段残余（口径声明：**减法推断，未直接计时**）

12篇:94：e2e 绝对开销 3.5s（17.4-13.9），其中内核预测 1.59ms×2025 ≈ 3.2s ⇒ **Java 收集+JNI 残余 ≈ 0.3s（≈0.15ms/chunk）**。口径三点（§9.7）：载体 = e2e 四臂 wall（ON/OFF 中位，2025 chunks）；覆盖面 = 收集循环+JNI 拷贝+回写的合计减法余项，**无法区分收集循环 vs JNI 拷贝各占多少**（open，scout 开放问题 3 的 pin/copy 未定加剧此模糊）；可比性 = 与 12篇 L94 同口径。
另注：该 0.3s 是「预测内核份额的减法余」，含 OFF 臂漂移 ±1.5s 级噪声（12篇:101）——**残余甚至可能为负（Java 段≈0）**，此口径下 B 的 e2e 上限本身不确定。

### 2.2 指令数推演（每格，非空节）

| 路径 | 每格指令构成（一手行号） | 估计 |
|---|---|---|
| 旧（Mixin:172-175） | `sec.getBlockState`→`PalettedContainer.get`：computeIndex（2 shift+2 or，PC.java:465）+ `storage.get`：Validate 边界×2（PIA.java:308）+ 魔数乘移（PIA.java:279-281）+ 数组载入+shift+mask（PIA.java:310-312）+ `palette.get`（ArrayPalette 越界检查+数组载入）+ `getRawId`（注册表查询）+ `getLuminance`（state 字段链）+ ABI 编码 | ~25-40 |
| 新（§1.4） | 常数除（乘移）+ rem + shift + mask + words 载入 + tab 载入 + blocks9 写 | ~8-12 |

⇒ 非空节收集段降 **~60-75%**。调用量：`getBlockState` 3-6万/chunk→0；`getRawId` 同量级→paletteSize×节数（每节 ≤15 次 + ID_LIST 节例外）。

### 2.3 e2e 绝对值预期与上限修正手段

- 基线口径（2.1 成立时）：B 省 ≈ 0.3s×(0.6-0.75) ≈ **0.2s**；e2e ON 17.4→~17.2s，回退比 1.25×→~1.235×。**单 B 不达 ≤1.05×**。
- **前置修正动作（建议排在实施前，成本≈0）**：Java 段直接计时——在 Mixin 收集前后加 chunk 级 `System.nanoTime`（env 门控 `-Dcoreswap.light.timing`，chunk 级一次，不逐格，守「诊断不进热路径每点」铁律），跑 200 chunk 得收集段真值。若实测 ≫0.15ms/chunk（减法口径失真），B 上限相应上修；若 ≈0，B 降级为「与 C 共享的铺垫件」，主投资转 A。
- 预期真实收益的第二来源：收集段现值可能被 round2「section 直采」收窄后仍占 0.5-1ms/chunk 量级（旧口径 5.6ms/chunk 是 round1 前，12篇:49；round2 无独立微基准，12篇:101 降级声明）。**此为 open，靠上述计时定。**

## 3. palette 规模分布 open（scout 开放问题 2）——最小实测方案

- **探针形态**：复用 §1.4 预处理段，env 门控 `-Dcoreswap.light.paldump=<N>`（N=前 N 个 chunk，建议 200）。每非空节记录一行进预分配 long[] 直方图（key=paletteSize 0..15 与 16+（ID_LIST），另记 bits 值分布），N chunk 后一次性 `System.out` 汇总直方图 + 非空节数/节。零日志热路径成本（内存累加），N 后自动停。
- **产出判据**：确认典型非空节 palette 2-15 项（scout 预期，scout-map §2 B）、ID_LIST 节占比、singular(bits=0) 非空节占比（影响 §4.2 特判面命中率）。
- **执行约束**：subagent 无 shell（AGENTS §八.12）——探针代码由本候选附带设计，主会话 P4 前实机跑，原始输出落 `.investigations/light-round3-260914-04/cmd-output/`。
- **顺带产出**：同一直方图可同时观测 bits 分布（4/5-8/15），为 C 的 sectionMeta 布局与 A 的均质跳过铺垫。

## 4. 等价性论证（与旧路 getBlockState 语义等价面）

1. **空节**：`sec==null || sec.isEmpty()` 短路 `Arrays.fill(0)` 原样保留（Mixin:162-167），不经 palette 路——等价面不动。
2. **SingularPalette 快路径**：bits=0 时 storage 为 EmptyPaletteStorage（PC.java:400）、palette.getSize()==1（PC.java:336 同判据先例）⇒ 全 4096 格 = palette.get(0) 编码值，`Arrays.fill` 特判（§1.4）。注意**非空节也可能是 singular**（整节单一非 air 方块，如深层石头节），此特判命中率高且是 B 的重要加速子面（整节 fill 免解码循环）。
3. **bits 协商形态**：BLOCK_STATE provider 全形态（0/4/5-8/ID_LIST 9-15）由 §1.4 通式 + §4.2 特判覆盖；bits 取自 `storage.getElementBits()` 运行时真值而非推导，天然跟随协商结果（含 onResize 后的新 Data，PC.java:133-139）。
4. **volatile data 读时序**：`data` 为 `private volatile Data<T>`（PC.java:36）。**MUST 单次局部快照** `Data<BlockState> d = ...getData()`，palette 与 storage 取自**同一 record 实例**（record 字段 final，PC.java:361，构造后不可变）——否则 resize 竞态下 palette/storage 撕裂（旧索引对新表）。LIGHT 阶段邻 chunk 已静置（scout-map §1.2 静态推断，且 vanilla propagateLight 同前提读 chunk）⇒ 快照读安全；此论证与旧路 `getBlockState` 内部单 volatile 读（PC.java:185）等强。
5. **数值面**：tab 预编码 = 旧路逐格 `getRawId | luminance<<24`（Mixin:174-175）的逐 palette 项等价重排——每格值 = tab[storage 索引]，与 `palette.get(storage.get(idx))` 链（PC.java:186）逐步同构。

## 5. 实施风险与改动面

| 项 | 内容 |
|---|---|
| 改动面 | 新增 `PalettedContainerAccessor`（@Accessor("data")，先例 ChunkSectionAccessor.java:29）；Mixin `wgLightCollectBlocks` 非空节分支重写（Mixin:168-178 → §1.4）。**JNI ABI / Rust 侧 / 内核零改动**。 |
| 风险 R1 | EmptyPaletteStorage/singular 特判漏写 → 全节错值（golden/对拍秒抓）；防御：`storage instanceof PackedIntegerArray` 否则走 fill 特判。 |
| 风险 R2 | data 快照撕裂（§4.3）——单局部读纪律写进实现注释。 |
| 风险 R3 | 收益不达减法口径预期（§2.1 残余可能≈0）——前置计时探针定夺，不盲实施。 |
| 风险 R4 | ID_LIST bits=15 节（若真实地形出现）解码成本略升 vs 旧路仍降（无注册表查询）；通式覆盖无正确性风险。 |
| 回退 | 收集路新旧分支 env 门控（`-Dcoreswap.light.palette`）可 A/B，默认新；异常即回旧分支。 |

## 6. 与 A/C 组合关系

- **B ∩ C 共享 accessor 链**（PalettedContainerAccessor + Data record 读 + palette 小表预编码 + §3 dump 探针），**去处分道**：B 的终点 = Java 解码填 blocks9 int[]，保持现 JNI ABI（lightCompute 签名不动）；C 的终点 = packed long[] + palette 表直传，Rust 解码。**互斥点 = 解码执行位置 + JNI ABI 形态**——C 采纳则 B 的解码主循环（§1.4 后半）作废，但 B 的前半（accessor/小表/特判/快照纪律）全部复用。⇒ B 是 C 的严格子集铺垫 + 独立可交付的小收益。
- **B ∩ A 正交**：B 改 Java 段、A 改内核 fill；B 不改变 blocks9 语义 ⇒ A 的 golden 基线不受 B 影响（可分别 commit、分别 revert，架构 §6 回退条款）。
- 排序建议（供 judge/主会话）：若 §2.3 计时证实收集段 ≥0.5ms/chunk → B 先行（小改动面快收益，且为 C 铺路）；若 ≈0.15ms 减法口径成立 → B 缓行，A/C 优先，B 并入 C 实施。

## 7. 判据预登记建议

1. **行为门**：golden 4 用例 FNV 逐位不变（C4 惯例，12篇:92）+ **双采集对拍**（§1.5：palette 路 vs getBlockState 路，前 4 chunk × 98304 逐元素，4×MATCH=0 diff；对拍后诊断移除并复编，#48 惯例）。
2. **性能门（分层）**：① Java 段 chunk 级计时（§2.3 探针，env 门控）前后对比，预期非空节段降 ≥50%；② e2e 四臂交替（架构 §3 主判据同口径），B 单独 commit 的增量 = ON 臂中位变化（预期 ~0.2s 量级，须 >噪声带——若 <噪声带则 B 的 e2e 证据降级为「无害 + Java 段微基准证明」，不虚报）。
3. **1.21.6**：B 不触内核/JNI ⇒ 仅需 1.20.1 验证 + 1.21.6 编译面不涉及（架构 §1 sanity 条款自动满足，声明即可）。

## 8. 引用清单（断言 → 文件:行号）

- PC.java = `.tmp/scout-260905-08/mcsrc/net/minecraft/world/chunk/PalettedContainer.java`：volatile data L36；get 链 L180-187；readPacket L203-215；Data record L361-392；EmptyPaletteStorage L400；BLOCK_STATE provider L420-430；computeIndex L465-467。
- PIA.java = `.tmp/scout-260905-08/mcsrc/net/minecraft/util/collection/PackedIntegerArray.java`：INDEX_PARAMETERS L20-213；maxValue L260；elementsPerLong L261；魔数取值 L262-265；getStorageIndex L278-282；get L307-313；getData L316-318；getElementBits L326-328；打包序 L228-249；writePaletteIndices L346-370。
- Mixin = `versions/1.20.1/java/src/main/java/wg/bench/mixin/ServerLightingProviderMixin.java`：收集循环 L128-183；空节短路 L162-167；逐格三重循环 L169-178；ABI 编码 L174-175。
- 12篇 = `versions/1.20.1/docs/12-lighting.md`：round1 残余 L49；e2e 3.5s/3.2s 减法 L94；对拍先例 L93；phase 分解 L99；降级声明 L101。
- 先例：`versions/1.20.1/java/src/main/java/wg/bench/mixin/ChunkSectionAccessor.java` L29。
- open 项：① §2.1 Java 段真值（计时探针定）；② ID_LIST/singular 节真实占比（§3 dump）；③ scout 开放问题 3（JNI pin/copy）不因 B 改变。
