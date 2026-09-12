# scout-interpretation-A1 — A1（javap 字节码）原始输出解读（260912-02 追加件）

> 角色：scout（只读解读；**不裁决**：不写 confirmed/candidate、不下「根因就是 X」定论）。
> 追加纪律：本文件**只追加**，`scout-map.md` 既有正文未改动（如与 scout-map 冲突，以本文件的 A1 字节码证据为准，并在 §6 标注取代关系）。
> 输入（全部由主会话采集，我逐行读过；未做任何运行臂）：
> - `.investigations/shared-java-core-260912-02/evidence/A1-javap-PalettedContainer-{1.20.1,1.21.6}.txt`（862 / 885 行）
> - `.investigations/shared-java-core-260912-02/evidence/A1-javap-PacketByteBuf-{1.20.1,1.21.6}.txt`
> - `.investigations/shared-java-core-260912-02/evidence/A1-commands.txt`
> **A1 载体锚（权威）**：1.21.6 named jar sha256 = `e6f80ac9604aadd3534d4ea46d7267eb28c8c274981684b2d08efe99039b51e4`；1.20.1 = `1d645c50380751f307572b7bf55484600c96d6ebe93963b0e3ec00e1f2d3fda9`（`A1-commands.txt:5-8`）。运行期即此 1.21.6 jar（`log-1.21.6-bulk.txt:737` 帧后缀同源）。
> 引用约定：`PC-1216:x` = `A1-javap-PalettedContainer-1.21.6.txt:x`；`PC-1201:x`；`PBB-1216:x` = `A1-javap-PacketByteBuf-1.21.6.txt:x`；`PBB-1201:x`。
> 验证分层：**Degraded-Partial**——本件结论建立在「权威字节码（javap）+ 我方源码」之上，**无任何运行期新证据**；凡属机制外推的仍标「推断」。

---

## ① b2a 的机制前提是否已被字节码级证实？

**结论（证据面表述）**：**读端（被证实）＋写端（我方源码，权威）两侧的「帧格式不配对」已在字节码级各自落实**；即 b2a 的两个必要前提**均已成立**。剩余未被字节码覆盖的只有「前缀错位→解码出 `{2,8}`」这一段**机制外推**（见 §6 与 ⑤）。

| # | 断言 | 证据 | 支持/不支持 |
|---|---|---|---|
| 1-1 | 1.21.6 `PalettedContainer.readPacket` 的 storage 段读**用定长版**（不消费 VarInt 长度） | `PC-1216:402` 方法头 → `:407 readByte:()B`（offset 5）→ `:413 getCompatibleData`（15）→ `:418 Palette.readPacket`（24）→ `:422 PaletteStorage.getData:()[J`（34）→ **`:423 invokevirtual #405 // PacketByteBuf.readFixedLengthLongArray:([J)[J`（39）** → `:427 putfield data`（45）→ `:429 unlock()` | ✅ 支持 |
| 1-2 | 1.20.1 同一位置**用带前缀版** | `PC-1201:373` 方法头 → `:378 readByte`（5）→ `:384 getCompatibleData`（15）→ `:389 Palette.readPacket`（24）→ `:393 PaletteStorage.getData`（34）→ **`:394 invokevirtual #398 // PacketByteBuf.readLongArray:([J)[J`（39）** | ✅ 支持 |
| 1-3 | 两版 `readPacket` **指令偏移逐条相同**（5/15/24/39/45），唯一差异 = 第 39 条 invokevirtual 目标 | 1-1 与 1-2 并列比对（常量池号不同=编译环境伪差，KB 对拍纪律） | ✅ 支持（单变量差异） |
| 1-4 | 1.21.6 `readFixedLengthLongArray(long[])` 语义 = 读 `values.length` 个 long、**无前缀** | `PBB-1216:658` 声明 → `:662` 委派静态版 → `:691` 静态版方法头 → **`:702 invokevirtual #622 // ByteBuf.readLong:()J`（循环体，无任何 VarInt 调用）**；同文件 `:691` 方法体内无 `VarInts` | ✅ 支持 |
| 1-5 | 1.21.6 **仍然存在**带前缀读 `readLongArray`（语义完整，只是 `readPacket` 不再用它） | `PBB-1216:652 readLongArray()` → `:655` 委派静态 `:665` → **`:668 invokestatic VarInts.read`（读长度）→ `:688 readFixedLengthLongArray`（再读该长度）** | ✅ 支持（且是 ④ 方案评估的关键事实） |
| 1-6 | 1.21.6 `writeLongArray` 仍是「VarInt 前缀 + 定长」（与 1.20.1 同义） | `PBB-1216:600 writeLongArray(long[])` → `:604` 委派静态 `:608` → **`:613 VarInts.write` + `:617 writeFixedLengthLongArray`** | ✅ 支持 |
| 1-7 | 1.20.1 **不存在任何** `*FixedLengthLongArray` 方法（读端只能按「前缀 + longs」解析） | `PBB-1201` 全体签名行（grep `LongArray`）只有 `:717 writeLongArray(long[])`、`:747 readLongArray()`、`:754 readLongArray(long[])`、`:765 readLongArray(long[],int)`；无 fixed 命中 | ✅ 支持 |
| 1-8 | 1.20.1 `writeLongArray` 确写 VarInt 前缀 | `PBB-1201:717` → `:722 invokevirtual #228 // writeVarInt:(I)` | ✅ 支持 |
| 1-9 | **我方写侧按 1.20.1 前缀契约写**（权威 = 我们自己的源码） | `java-core/src/main/java/wg/bench/BulkWb.java:262 pb.writeLongArray(pa.getData())`；`:247 pb.writeLongArray(EMPTY_LONGS)` | ✅ 支持（源码级；我方代码不以字节码为权威） |
| 1-10 | ⇒ **1.21.6 上「我方写的帧」与「vanilla 读的帧」不配对**（写=前缀+longs，读=仅 longs） | 1-6/1-9（写侧语义） × 1-1/1-4（读侧语义） | ✅ 支持（**前提层面**） |
| 1-11 | 「多余 2 字节前缀 ⇒ 解码索引越界 ⇒ `EntryMissingException`」 | **无字节码证据**：属机制外推（`VarInt(256)=0x80 0x02` 2 字节 + 4-bit 打包下 16 bit 位移 = 前缀 nibble 落在每个 long 首尾 16 bit） | ⚠ **推断**（未证实；判别见 ⑤ C1 与 scout-map §6 b2a 反证条件①） |

**未被 A1 覆盖、但与 b2a 相关的静态缺口（必须如实标注）**：
- `PalettedContainer$Data.writePacket` / `Data.getPacketSize` 的字节码**未采集**（javap 只打印指定顶层类的成员，嵌套类需单独指定类名）⇒ 「vanilla 自身写端也用定长 + 不再计前缀长度」目前仅为**源码级**（`versions/1.21.6/.../PalettedContainer.java:393/:387` vs 1.20.1 `:386/:380`）。补采命令见 §7 A1b。
- `Palette` 实现（`ArrayPalette.readPacket` / `SingularPalette` / `IdListPalette`）字节码**未采集** ⇒ ③ 的 palette 段对齐目前为源码级。补采见 §7 A1b。

---

## ② b2b（位宽协商语义差）现在是否仍然存活？

**状态：形式上仍存活（未被 A1 排除），但证据面已被两重独立削弱；且 A1 显示其**前置通道**在字节码层两版同形。**

| 判据 | A1 证据 | 对 b2b 的作用 |
|---|---|---|
| `readPacket` 里 byte 的**消费与传递路径**两版是否同形 | `PC-1216:407 readByte`（5）→ `:413 getCompatibleData(Data,I)`（15）；`PC-1201:378`→`:384`（同偏移、同签名） | **削弱**：byte→provider 的调用点、参数、顺序逐条相同；若位宽映射语义变了，差异只可能在 `PaletteProvider.createDataProvider` 的**实现**里 |
| `getCompatibleData` 的**决策骨架**两版是否同形 | `PC-1216:191` 方法头 → `:198 PaletteProvider.createDataProvider(IndexedIterable,I)`（9）→ `:204 Data.configuration()`（19）→ `:205 DataProvider.equals`（22）→ `:216 DataProvider.createData(...)`（43）；`PC-1201:162/:169/:175/:176/:187` **偏移与调用序列逐一相同** | **削弱**：容器「复用 vs 新建 Data」的语义两版同构 |
| `PaletteProvider.createDataProvider` 的**实现**（0→SINGULAR / 1..4→ARRAY(4) / 5..8→BI_MAP(bits) / ≥9→ID_LIST） | **A1 未采集**（实现在嵌套类/枚举内） | **无法用 A1 排除** → b2b 存活的唯一缝隙就在这里 |
| 运行期异常索引分布 | 130 条 DIAG 仅 `{2,8}`（scout-map §0.4，`log-1.21.6-bulk.txt` grep 精确计数） | **强削弱**：位宽协商错位预测解码值**散布**在 `0..15`（或 `0..31`）多处；仅两值且恰为 `0x80 0x02` 的 nibble，与该机制**不**自洽 |
| 1.20.1 `getPacketSize` 含 `getVarIntLength`、1.21.6 删该项 | `PBB-1201:32 public static int getVarIntLength(int)` 存在；`PBB-1216` 无该签名命中（被 `VarInts.getSizeInBytes` 取代，源码 `ArrayPalette.java:105`） | 与位宽协商无关，但**佐证** 1-1 的帧变更（同一批 wire 变更） |

**保留 b2b 的判据（仍能区分 b2a 与 b2b 的观察）**：
1. **索引取值集合**：若某次重复跑出现 `{2,8}` 之外的索引值（尤其 3..7 / 9..15 多值散列）⇒ 指向 b2b（或 b1）；若恒 `{2,8}` ⇒ b2a 独有签名。**单变量、零成本（只需既有日志 grep）**。
2. **`storage.getElementBits()` / buffer 首字节 `i` 的一致性探针**：在 `buildContainer` 里于 `pc.readPacket(pb)` 后打印「写的 `bits`（`BulkWb.java:244`）」与「读端实际 storage 位宽」，若二者不符 ⇒ b2b；若相符而内容仍错 ⇒ b2a。
3. **残留字节探针**：`readPacket` 后 `pb.readableBytes()`。b2a 预测 **2**（ARRAY 分支 `VarInt(256)`；SINGULAR 支预测 1）；b2b 若仅是位宽协商差、帧仍按前缀读，预测 **0**（前缀被正常消费）。该探针**一次即可分离 b2a/b2b**。
4. **A1b 字节码**（`PalettedContainer$PaletteProvider` 及其 enum/impl 的 `createDataProvider`）：若与 1.20.1 逐条同形（0/1..4/5..8/≥9 分支与 `bits()` 返回值一致）⇒ b2b 死。

**若已被排除，是哪一条排除的？** —— **尚未**被排除；A1 只把 b2b 的存活缝隙压缩到「未采集的 `createDataProvider` 实现」一处，运行期 `{2,8}` 分布对它是不支持（非排除）。按 scout 纪律，b2b 仍应保留为互斥候选项，直到第 3/4 条判据之一落地。

---

## ③ 我方三处写侧读法与 1.21.6 读方的逐项对齐关系

写侧（我方源码，`BulkWb.java`）与读侧（1.21.6 字节码）**三步同序**：`byte(bits)` → `palette 段` → `storage long 段`；**前两步对齐，第三步错配**。两版 `readPacket` 指令偏移完全相同（5 / 15 / 24 / 39），这是「单变量差异」的字节码级证据。

| 步 | 我方写侧（源码） | 1.21.6 读侧（字节码） | 1.20.1 读侧（字节码） | 对齐判定 |
|---|---|---|---|---|
| ① 位宽字节 | `BulkWb.java:244 pb.writeByte(bits)`（`bits = distinct<=1 ? 0 : ceilLog2(distinct)`，`:235`） | `PC-1216:407 invokevirtual PacketByteBuf.readByte:()B`（offset **5**）→ istore_2 `:408` → `iload_2` 传参 `:412` | `PC-1201:378`（offset 5）同形 | ✅ **对齐**（byte 语义=storage 请求位宽，两版同点消费） |
| ② palette 段 | `:259 pb.writeVarInt(distinct)` + `:260 distinct × pb.writeVarInt(stateRawId(palRaw[k]))`（bits==0 支改 `:246 writeVarInt(stateRawId(palRaw[0]))`） | `PC-1216:418 invokeinterface Palette.readPacket:(PacketByteBuf)V`（offset **24**）→ 实现内按 `VarInt size` + `size × VarInt rawId` 消费 | `PC-1201:389`（offset 24）同形 | ✅ **对齐（契约同）** —— 依据：两版 `ArrayPalette.readPacket` 方法体逐字相同（`versions/1.20.1/.../ArrayPalette.java:85-92` vs `versions/1.21.6/.../ArrayPalette.java:86-92`）；⚠ **A1 未采集 `ArrayPalette` 字节码**，此格为源码级（补采 A1b）。另：我方 `distinct ≤ 16`（实测 `max_distinct=7`，`evidence/log-1.20.1-bulklog.txt:2567`）≤ 读端 `ArrayPalette` 容量 `1<<4=16`（1.21.6 `ArrayPalette.java:23`）⇒ 容量不越界 |
| ③ storage long 段 | `:262 pb.writeLongArray(pa.getData())`（= VarInt 长度 + longs）；`bits==0` 支 `:247 pb.writeLongArray(EMPTY_LONGS)`（= VarInt(0)，1 字节） | `PC-1216:422 PaletteStorage.getData:()[J`（34）→ **`:423 readFixedLengthLongArray:([J)[J`（39）**（**只读 `values.length` 个 long，无前缀**，`PBB-1216:691/:702`） | `PC-1201:393 getData`（34）→ `:394 readLongArray:([J)[J`（39）（**先读 VarInt 长度**，`PBB-1201:765` 三参版读长度后读 long） | ❌ **错配（1.21.6）／✅ 配对（1.20.1）** |
| ④ 装回 | `:266-268` 新建容器 → `pc.readPacket(pb)`；写入侧 `:172 acc.wgSetBlockStateContainer(pc)` | `PC-1216:427 putfield data`（45）→ `:429 unlock`（49） | `PC-1201` 同形 | ✅ 两版同形（读端把新 `Data` 装回 volatile 字段） |

**逐项结论（事实面）**：
- 步① 与步② 在 1.21.6 上**无需改动**（契约同、偏移同、容量不越界）。
- 步③ 是**唯一**错配点：`BulkWb.java:262` 写的是「前缀 + 256 longs」，1.21.6 读端只取 256 个 long ⇒ 读端**不消费**那 2 字节前缀（ARRAY 分支 `4096×4/64 = 256 longs`，`VarInt(256)=0x80 0x02`），而缓冲区实际还有 `2` 字节未读（scout-map §6 b2a 预测）。
- 1.21.6 **保留了** `readLongArray`（前缀版，`PBB-1216:652/665/668/688`）——说明错配**不是**「1.21.6 移除了前缀能力」，而是「`PalettedContainer.readPacket` 换成了定长版」。
- 1.20.1 **完全没有**定长版（`PBB-1201` 无任何 `*FixedLengthLongArray`）⇒ 写侧不能无条件改成定长（会打断 1.20.1 出货线）。

---

## ④ 最小修复缝候选清单（**只列方案与判据，未实施**）

前提约束（来自计划 §1）：① 1.20.1 出货线**字节/行为不变**（V1b 复用）；② 1.21.6 默认臂**不得退化**；③ 变更若落共享源须带分版缝。验收判据组（父会话给定）：`1.21.6 默认臂不退化` + `1.21.6 bulk 臂 [WG-CONTENT-WB] 恢复 625` + `两臂逐 chunk hash/dh 多重集相等` + `1.20.1 V2/V3 重跑`。

### 方案 F1｜分版缝 · 帧写入形态（共享源内按分版常量二选一）— **最小改动**
- 改动面：`BulkWb.java:262`（ARRAY/BI_MAP/ID_LIST 支）与 `:247`（SINGULAR/`EMPTY_LONGS` 支）——把 `pb.writeLongArray(...)` 换成「按版本选 `writeLongArray`（1.20.1）/`writeFixedLengthLongArray`（1.21.6）」，版本常量走 `WgCompat` 同族分版缝（`WgCompat.java` 为**分版件**，`BulkWb` 保持共享）。
- 对 **1.20.1** 影响：走原分支 ⇒ **字节与行为均不变**（静态 final ⇒ JIT 折叠，热路径零成本）。判据「1.20.1 V2/V3 重跑」**可先由 V1b（条目级 sha 零差）预判**，再跑运行臂确认。
- 是否需分版件：**需要**（`WgCompat` 常量；不改 `coreswap*.mixins.json`，无 mixin 侧影响）。
- 判据覆盖：四条全覆盖（默认臂路径不触碰；bulk 臂 `[WG-CONTENT-WB]` 与两臂多重集直接判；1.20.1 由条目 sha + V2/V3 判）。
- 残余风险：SINGULAR 支（`:247`）若**不**同步分版，1.21.6 上仍会多留 `VarInt(0)`=1 字节未消费（读端读 0 个 long ⇒ **无功能影响**，但属帧不洁；建议同批统一，避免留下第二个不可解释的残留字节）。

### 方案 F2｜运行期能力自探测（免分版件、跨版本自动适配）
- 改动面：`BulkWb.buildContainer` 首跑时探测读端帧语义（例：先按前缀帧 `readPacket` 后校验 `pb.readableBytes()==0` / `pc.get` 抽样比对，不匹配则重试定长帧），结果缓存进静态 final/AtomicBoolean。
- 对 **1.20.1** 影响：**行为不变**（探测结果=前缀帧），但**类文件字节会变** ⇒ V1b「条目级 sha 零差」不再成立，须改用运行级对照（V2/V3）+ 内容门。
- 是否需分版件：**不需要**。
- 判据覆盖：四条均可覆盖，但「1.20.1 出货线字节不变」这条**降级**为「行为不变（运行臂实证）」。
- 残余风险：探测代码自身需被 judge 审（新失败面）；首跑多一次打包开销（可忽略）；本质是「用复杂度换免分版件」，**架构收益可疑**。

### 方案 F3｜分版件 · 抽 `BulkWb` 帧写入函数（F1 的架构化变体）
- 改动面：共享 `BulkWb` 只留调用点（`BulkWbFrame.writeStorage(pb, pa)`），实现放 `versions/1.21.6/java/.../BulkWbFrame.java` 与 1.20.1 同名分版件。
- 对 **1.20.1** 影响：零（分版实现各写各的）。
- 是否需分版件：**需要**（新增分版 java 文件——须确认 `build.gradle` 的共享源集合是否自动包含，属构建配置核对项）。
- 判据覆盖：同 F1。
- 残余风险：新增分版文件与共享源集合的接线（KB 构建家族坑）；收益=把版本差异从共享源彻底移出，代价=文件数与构建配置复杂度。

### 方案 F4｜绕开 `readPacket` 的整段构造（**不推荐**，仅列全）
- 思路：不用 vanilla `readPacket` 解码，而用 `PackedIntegerArray`(+palette) 直接构造 `Data`（需反射/mixin 触及包私有 record `DataProvider`，KB 已证包外不可命名）或以自建 storage 灌入。
- 对 **1.20.1** 影响：若共享则**行为面大改**，V1b 判据失效；需两版各自实现。
- 判据覆盖：可覆盖但验证成本最高、跨版本最脆，**且与既有 C 线「零布局转换」设计相悖**。

**方案比较（供 HOOK-C 决策，非裁决）**：按「改动最小 + 1.20.1 字节零变化 + 判据可判」三轴，**F1 最优、F3 次之（架构更干净但引入分版文件接线）**；F2 在「1.20.1 字节不变」轴上降级；F4 不建议。三者都不触碰 `writeSections` 的换容器/计数语义，也都不触碰默认逐块臂 ⇒ 「默认臂不退化」是结构性成立（而非仅靠测试）。

---

## ⑤ 运行探针 C1（`-Dcoreswap.bulkwbtest=1`）是否仍为决定性判别？

**是，仍是决定性且非冗余**——A1 只证实了「读端帧格式」与「写端源码帧格式」两侧前提，**没有**证实 1-11 那一段机制外推（前缀错位→解码越界）。C1 是当前唯一能把「buffer 契约」单独隔离出来检验的载体（合成 section、无地形、无 carver、无 Rust）。

**A1 让 C1 的预测更硬（可先静态预言，再用运行臂验证）**：写端=前缀帧（`BulkWb.java:262`）、读端=定长帧（`PC-1216:423`）⇒
- **1.21.6 期望签名**：`SINGULAR 支（distinct=1）PASS`（bits=0、读 0 个 long、前缀残留无害）→ 紧接 **ARRAY 支（distinct=3）FAIL 或抛异常**（`AssertionError [WG-BULKWB-TEST] FAIL branch distinct=3 …`，`BulkWb.java:313-314`；或 `pc.get` 直接抛 `EntryMissingException`，`:310`）→ 后续支不再执行。
- **1.20.1 期望签名**：4×`[WG-BULKWB-TEST] PASS` + `[WG-BULKWB-TEST] all branches PASS`（`BulkWb.java:317-321`）。

**反证条件是否仍成立**：**成立**。若 1.21.6 四支**全 PASS**，则「我方 buffer + 1.21.6 读端」在合成条件下可正确往返 ⇒ b2a 的错位机制被否（届时须转 b2b/b1，或怀疑既有 `{2,8}` 另有来源）。**注**：该反证条件只在「1.20.1 对照臂也全 PASS（证明探针自身可用）」时才具判别力——若 1.20.1 探针自身不 PASS，先修探针。

### 最小运行命令（两版）

**1.21.6（决定性臂）**——`Radius` 只需覆盖首个 chunk（自检在首个 `writeSections` 触发，`BulkWb.java:141`）：
```powershell
$env:CORESWAP_EXTRA_JVM = "-Dcoreswap.bulkwb=1 -Dcoreswap.wbcontent=1 -Dcoreswap.bulkwblog=1 -Dcoreswap.bulkwbtest=1"
pwsh .investigations\shared-java-core-260912-01\evidence\tool-run_1216_w2.ps1 -Tag probe-b2a-test -Radius 16
```
（`CORESWAP_EXTRA_JVM` 由 `tool-run_1216_w2.ps1:27-29` 追加到 `JAVA_TOOL_OPTIONS`；日志 `.tmp\shared-java-core-260912-01\w2\1216-probe-b2a-test.log`）
**硬门（血统）**：日志 `[CppBridge] dll= … sha256=abd7d8893d22e030…`（与 `log-1.21.6-bulk.txt:94` 同值）。
**判读**：grep `WG-BULKWB-TEST`（期望 PASS distinct=1 + FAIL/异常）、`EntryMissingException`（期望出现且索引∈{2,8}）、`[WG-BULKWB]`（期望 `max_distinct≤8`、`idlist_hits=0`）。

**1.20.1（反面对照臂）**——该脚本 `:24` 未接 `CORESWAP_EXTRA_JVM`，最小做法是复制一份并追加 flags（不改 tracked 文件）：
```powershell
Copy-Item .investigations\shared-java-core-260912-01\evidence\tool-run_pre_1201.ps1 .tmp\run_pre_1201_probe.ps1 -Force
(Get-Content .tmp\run_pre_1201_probe.ps1) -replace '(-Dcoreswap\.wbcontent=1)"', '$1 -Dcoreswap.bulkwb=1 -Dcoreswap.bulkwbtest=1"' | Set-Content .tmp\run_pre_1201_probe.ps1
pwsh .tmp\run_pre_1201_probe.ps1 -Tag probe-b2a-test-1201 -Radius 16
```
**判读**：`[WG-BULKWB-TEST] PASS distinct=1/3/20/300` + `all branches PASS` ⇒ 探针可用且 1.20.1 帧配对 ✅。**注意**：该脚本在 `.tmp/w2/pre-wt` worktree 内运行（E2 纪律：worktree 仅可用于类文件/运行期判据，不可作条目级基线）——本探针属**运行期/类文件**范畴，故可用；若需主树 1.20.1 臂，改用 `versions\1.20.1\java` + `gradle runServer -PcppReplace=true -PcppWorldgenDir=…\versions\1.20.1\data\worldgen` 同 `JAVA_TOOL_OPTIONS`。

**C1 被 A1 取代了吗？** 没有。A1 关闭的是「读端是否定长」这个问题（已关闭、无需运行臂）；C1 要回答的是「我方 buffer 与读端组合是否真的往返失败」，两者不可互替。

---

## ⑥ 与 scout-map.md 的关系（取代/收敛声明，不改既有正文）

| scout-map 原表述 | A1 后状态 |
|---|---|
| §2「1.21.6 读端为定长帧 / 1.20.1 为前缀帧」 | **升级为字节码级证据**（1-1/1-2/1-4/1-7），原表述不删、此处增强 |
| §6 b2a「机制链两端都有行号」 | 仍是「行号/源码级」；A1 把**读端**升到字节码级；**机制外推（1-11）仍未证实**（→ ⑤ C1） |
| §6 b2a 反证条件②「若 javap 显示 1.21.6 仍用 `readLongArray` ⇒ 死」 | **已判不成立该反证**：`PC-1216:423` 用 fixed（读端反证条件②**已被排除**，b2a 存活） |
| §6 b2b「未被支持但未排除」 | **维持**（存活缝隙仅剩 `createDataProvider` 实现字节码，见 ②） |
| §5「反编译源行号不可比」 | 维持；A1 未采集 `-l` 行表（A3 仍待跑，U3 未闭） |

## ⑦ A1 未覆盖项 → 补采命令（供主会话，纯静态）

**A1b（补 ③ 的第②步与 vanilla 写端自洽性；嵌套类必须显式指定类名）**
```powershell
$j1216 = ".gradle-home\caches\fabric-loom\minecraftMaven\net\minecraft\minecraft-merged\1.21.6-net.fabricmc.yarn.1_21_6.1.21.6+build.1-v2\minecraft-merged-1.21.6-net.fabricmc.yarn.1_21_6.1.21.6+build.1-v2.jar"
javap -p -c -classpath $j1216 "net.minecraft.world.chunk.PalettedContainer`$Data" | Select-String -Pattern "writePacket|getPacketSize|LongArray|getVarIntLength" -Context 1,4
javap -p -c -classpath $j1216 net.minecraft.world.chunk.ArrayPalette | Select-String -Pattern "readPacket|readVarInt|getOrThrow" -Context 0,4
```
判读：`Data.writePacket` 应出现 `writeFixedLengthLongArray`（1.20.1 对应文件应出现 `writeLongArray`）⇒ vanilla 写读自洽；`ArrayPalette.readPacket` 应出现 `readVarInt` 两次语义（size 与逐项 id）⇒ ③ 步②对齐升级为字节码级。若 `Data.getPacketSize` 内出现 `getVarIntLength`（1.21.6）则与源码 `:387` 冲突，需回头核。

**A3（行号口径，仍待跑）**：`javap -l -p -classpath $j1216 net.minecraft.world.chunk.ChunkSection`（期望 `blockStateContainer.get` 行号 = 51）。

---

## ⑧ 本件「已验证事实 vs 推断」分列

**事实（A1 原始输出行号可查）**：1-1…1-10 全部（读端定长/前缀、指令偏移同形、fixed 语义无 VarInt、1.21.6 仍存前缀版读、写侧 `writeLongArray` 语义、1.20.1 无 fixed 方法、`getVarIntLength` 仅 1.20.1、我方写侧 `BulkWb.java:247/:262` 用前缀版）；③ 步①/步③ 对齐关系及其字节码行号。
**推断（显式）**：1-11（前缀错位→`{2,8}`→越界的完整机制）；③ 步②的「对齐」当前为源码级（A1 未采 `ArrayPalette` 字节码）；② 中「`{2,8}` 分布不支持 b2b」为分布-机制自洽性论证，非排除证明。
**本件不主张**：不主张根因已定；不写 confirmed/candidate；不建议具体实施方案（④ 只列方案与判据）。

---

## ⑨ 追加（同一 A1 解读的收尾核查）：分版缝事实 → **对 ④ 的一处自我修正** + 新候选 F5

> 触发：为落实 ④「是否需要分版件」，我读了既有分版缝（只读）。结论导致 ④ F1/F3 的「1.20.1 字节不变」表述**必须修正**（属我自己的表述错，故显式取代，不删原文）。

### 9.1 既有分版缝事实（`file:line`）
| 事实 | 证据 |
|---|---|
| 分版缝 = **每版本一个同名文件**（非共享） | `versions/1.20.1/java/src/main/java/wg/bench/WgCompat.java`（18 行）与 `versions/1.21.6/java/src/main/java/wg/bench/WgCompat.java`（20 行），声明注释 `:3` 明示「共享文件不得出现版本分支/映射差异，全部收进本类」 |
| 两版**内容差异仅两个 `public static final boolean`** | 1.20.1 `:8 BULKWB_ON = true` / `:11 SKIPAIR_ON = true`；1.21.6 `:9 BULKWB_ON = false` / `:13 SKIPAIR_ON = false` |
| 既有 property 覆盖范式（可直接复用） | `:16-19`（1.20.1 `:14-17`）`flag(String, boolean)`：未设取分版缺省，设了「非 "0" 即真」 |
| 常量是 **compile-time constant** ⇒ 会被内联进引用方 class | `BulkWb.java:73 ON = WgCompat.flag("coreswap.bulkwb", WgCompat.BULKWB_ON)`、`CppBridge.java:642` 同族 |

### 9.2 ⚠ **自我修正**（取代 ④ 中 F1/F3 的「1.20.1 字节与行为均不变」）
**修正后表述**：F1/F3（以及任何落在共享 `java-core` 的修复）**必然改变 1.20.1 构建产物字节**——① 共享源 `BulkWb.java` 改动 ⇒ `BulkWb.class` 必变；② 若在 `WgCompat` 新增常量 ⇒ `WgCompat.class` 必变（且新常量会被内联进 `BulkWb.class` 常量池）。**能保证的不是「字节不变」，而是「行为/写出帧逐字节相同」**（1.20.1 分支仍调用 `writeLongArray`，同一 `PacketByteBuf` 输出内容）。
⇒ **V1b 判据必须相应降级**：从「1.20.1 jar/条目级 sha 零差」改为「**1.20.1 写出帧与运行期行为等价**」（可用既有 V2/V3 臂 + `[WG-CONTENT]`/`[WG-CONTENT-WB]` + 逐 chunk hash/dh 多重集覆盖）。**这一点会直接影响 HOOK-C 的方案选择与判据设计**，故单列。

### 9.3 新候选 **F5｜1.21.6-only mixin 改写读端调用**（唯一能对 1.20.1 做到**真零字节影响**的方案）
- 思路：**不动共享源**，在 `versions/1.21.6/java/.../mixin/` 新增 mixin，对 `PalettedContainer.readPacket` 内 `PacketByteBuf.readFixedLengthLongArray([J)[J`（`PC-1216:423`，offset 39）做 `@Redirect`，handler 改为「读 VarInt 长度 + 读该长度 long 到**传入的数组**」，从而接受我方 1.20.1 式前缀帧。
- 对 **1.20.1** 影响：**真零字节**（该 mixin 只存在于 1.21.6 分版 resources/源集）——这是 F1/F3 做不到的。
- 需分版件：**需要**（1.21.6 mixin 类 + `coreswap.mixins.json` 新增条目，`required:true` 下未命中会装载失败 ⇒ 需 refmap 目标核对）。
- 判据覆盖：四条均可覆盖；「1.20.1 V2/V3 重跑」变为**不必要**（1.20.1 无改动），但**1.21.6 的 mixin 目标解析**成为新的前置风险项。
- 关键风险（必须核，非推断可过）：`@Redirect` 的 handler 必须**写回传入数组**（readPacket 原码 `PC-1216:424 pop` 丢弃返回值 ⇒ 若 handler 用 `buf.readLongArray()` 返回新数组，storage 将保持全零）；且 mixin 的 `@At` 定位/remap 需在 1.21.6 mapped 环境下验证（KB mixin 家族坑）。
- 方案对比补充：F1 = 「写出读端期望的帧」（改我方写入，语义直白，1.20.1 行为不变/字节变）；F5 = 「把 vanilla 读端掰回我方帧」（改 vanilla 解码，1.20.1 真零字节，但 blast radius 落在原版热路径 + mixin 维护面）。**二者互斥，属架构决策（HOOK-C）**；我不选边。

### 9.4 修正后的方案速览（供 HOOK-C）
| 方案 | 1.20.1 影响 | 分版件 | 主要风险 |
|---|---|---|---|
| F1 分版常量选帧写入 | 行为不变 / **字节变**（9.2） | 需（`WgCompat`+常量） | 低（我方写入路径） |
| F3 抽分版帧写入函数 | 行为不变 / **字节变** | 需（+分版 java 文件与构建源集接线） | 中（构建配置） |
| F2 运行期自探测 | 行为不变（须运行级实证）/ 字节变 | 免 | 中（新探测失败面） |
| **F5 1.21.6-only mixin redirect** | **真零字节 / 行为不变** | 需（mixin + mixins.json） | 中高（vanilla 解码路径 + remap + `pop` 语义） |
| F4 绕开 readPacket | 大改 / 判据失效 | 需 | 高（不建议） |

**9.x 分层声明**：9.1 为事实（`file:line` 可查）；9.2 为对既有表述的**修正**（取代）；9.3 的方案可行性含推断成分（mixins.json/remap/`pop` 语义需实测或静态核 `PC-1216:419-424` 后确认——`pop` 一条已有字节码证据）。

---

## ⑩ C1 运行级解读（260912-02 Phase 2.5 追加件；§1–§9 正文未改）

> 输入（主会话采集，我逐行读过）：`.investigations/shared-java-core-260912-02/evidence/C1-log-1216-probe-b2a-WBTEST.log`（505 行）、`fp-1.21.6-probe-b2a-WBTEST.txt`（49 行）、`C1-commands.txt`、`tool-run_1216_arm.ps1`；对照臂 = `.investigations/bulk-writeback-260911-05/evidence/raw-c-bulk.log:115-136`（既有 tracked，未重跑）。
> 纪律：只读解读，不写 confirmed/candidate；凡「判定/排除」均限定为**证据面**表述。
> 分层：**运行级（Full-ish）+ 静态字节码**——本轮首次有运行期往返证据；量化主张均给出可查行号。

### 10.1 原始行独立复核（我重新计数，与主会话一致）
| 量 | 值 | 复核依据 |
|---|---|---|
| `[WG-BULKWB-TEST]` | **3 命中**：`:138 PASS distinct=1 (SINGULAR)`、`:140 FAIL branch distinct=3 i=0 got=granite want=stone`、`:139` 是 `[WG-BULKWB] state_ids_size=27946 idlist_bits=15`（非 TEST 行） | grep `WG-BULKWB-TEST\|\[WG-BULKWB\]\|all branches` = 3 命中 |
| 缺 `distinct=20/300`、缺 `all branches PASS` | **成立** | 同上（自检在 ARRAY 支首位置即抛，`TESTED.compareAndSet`，`BulkWb.java:294`） |
| `EntryMissingException` 总数 = **50** | 48 条 DIAG + `:292`（崩溃报告顶层）+ `:472`（`Caused by`） | grep 逐行枚举 + `:286` `---- Minecraft Crash Report ----` |
| `index 2` × **41**、`index 8` × **9** | `index 8` 行号 = `141,145,171,181,194,196,254,292,472`（9 条）⇒ 其余 41 = index 2 | 逐行枚举 |
| `DIAG write threw` = **49** | = 1×AssertionError(`:140`) + 48×EntryMissing | 交叉 |
| `[WG-CONTENT] chunk(` = **49**、`[WG-CONTENT-WB] chunk(` = **0** | 成立 | 臂指标（`tool-run_1216_arm.ps1:113/:114` 计数）＋ fp 文件全 49 行均为 `[WG-CONTENT]` |
| dll 自证 | `[CppBridge] dll=…sha256=abd7d8893d22e030…` = 1.21.6 权威前 16 位 ✓（血统门通过） | `C1-commands.txt:8` |
| 对照臂（1.20.1） | `raw-c-bulk.log:131-136` = 4×`PASS`（distinct=1/3/20/300）+ `all branches PASS`；`:116` dll sha `838e89794a54e19d…`（**1.20.1 专属**，与 1.21.6 `abd7d889…` 不同 ⇒ 两版各自血统自证） | 直接读 |
| 对照臂遗留 caveat | 该 log 属**上一工作块**（260911-05，另一 commit）；selfTest 行格式与当前代码（`BulkWb.java:317-321`）逐字一致 ⇒ 大概率同版探针，但 **commit 同一性未核**（建议 `git log -1 --format=%H -- java-core/src/main/java/wg/bench/BulkWb.java`，或对同代码重跑 1.20.1 臂闭合） | 格式比对 |

### 10.2 ① 四条预登记预测逐条判定 + b2a 双端落实 + b2b 排除

| 预登记预测（我在 §5/消息中给出） | 判定 | 证据 |
|---|---|---|
| P1 1.21.6：`PASS distinct=1` → 紧接 ARRAY 支 FAIL/抛 | **成立** | `:138` PASS(SINGULAR) → `:140` FAIL(distinct=3) |
| P2 无 `FAIL distinct=20/300`、无 `all branches PASS` | **成立** | grep 3 命中（见 10.1） |
| P3 异常索引 ⊆ {2,8}（`VarInt(256)=0x80 0x02` 的 nibble） | **成立** | 41×2 + 9×8，无第三取值 |
| P4 bulk 臂 `[WG-CONTENT-WB]` = 0、`[WG-CONTENT]` 正常 | **成立** | 0 / 49 |
| P5（对照臂）1.20.1 四支全 PASS | **成立（既有证据）** | `raw-c-bulk.log:131-136` |
| P6「`readPacket` 后残留 2 字节」 | **未测**（本臂无该探针） | — |

- **b2a 是否「静态 + 运行级往返」双端落实**：**是**。静态端（A1）＝读端 `readFixedLengthLongArray`（`PC-1216:423`）vs 写端 `writeLongArray` 前缀帧（`BulkWb.java:247/:262`）；运行端（C1）＝同一 buffer 在 1.21.6 上**SINGULAR 支完全往返正确**（`:138`）、**ARRAY 支在第一个位置就解错且错值指向 +4 位移**（`:140` ＋ 10.3 算术）。两端独立且互证。
- **b2b 是否可判定排除**：**证据面可排除**（不再只是「不支持」）。判据链：
  1. A1：`readPacket` 的 byte 消费与 `getCompatibleData` 决策骨架两版**指令偏移逐条相同**（5/15/24/39；`:407/:413` vs `:378/:384`；`:198/:204/:205/:216` vs `:169/:175/:176/:187`）。
  2. U7（主会话闭合）：`PaletteProvider$1` 分支序列逐项相同、`$2` 三支相同、`getBits` 逐字节码相同 ⇒ 「0→SINGULAR / 1..4→ARRAY(4) / 5..8→BI_MAP / ≥9→ID_LIST」的**实现层无差异**。
  3. 运行级：若 b2b 成立（读端位宽与写端不符），`readFixedLengthLongArray(storage.getData())` 的长度将与写端 long 数不符 ⇒ 解码值**散列**且残留字节数 ≠ 2；实测异常取值**只有 {2,8}** 且 SINGULAR 支完全正确，与 b2a 的单变量预测吻合。
  ⇒ 三条合起来关闭了 b2b 的全部可能位置（读端 byte 消费、`getCompatibleData`、`createDataProvider` 实现）——**本缺陷机制上排除 b2b**。

### 10.3 ② 逐位解释 `i=0 got=granite want=stone`，并解决「位移 vs 离散两点」的张力

**代码事实（先钉死坐标与用例）**：`BulkWb.java:295-311` —— 用例 `{1,2,3}`；`src[i] = ids[i % 3]`；位置映射 **`x=i&15, z=(i>>4)&15, y=i>>8`** ⇒ 线性索引 `i = (y<<8)|(z<<4)|x`；读回用 `pc.get(x,y,z)`（`:310`）；`want = CppBridge.stateById(src[i])`（`:311`）。
**raw id 锚（本轮新取的独立证据）**：`versions/1.21.6/data/blocks.json:926 "minecraft:stone": 1`、`:400 "minecraft:granite": 2`。
⇒ `want` = stateById(src[0]) = **stone(1)** ✓；`got` = **granite(2)** = stateById(raw id 2) = stateById(`src[4]`)（因 `4 % 3 = 1` → `ids[1] = 2`）。
**⇒ reader 位置 0 解出的正是 writer 位置 4 写入的值** —— 这是「+4 局部位移」的**逐位实证**，而非「任意损坏」。

**位移全貌（推导，nibble 级；`PackedIntegerArray.java:307-311`：元素 k 位于 `bits[4k, 4k+4)`，低位优先）**：
- 4-bit、16 元素/long；writer 字节流 = `[0x80,0x02] ‖ W0[0..7] ‖ W1[0..7] ‖ …`（前缀由 `writeLongArray` 写入）。
- reader long 0 = `0x80 0x02 W0[0] W0[1] W0[2] W0[3] W0[4] W0[5]`；
  reader long m≥1 = `W(m-1)[6] W(m-1)[7] Wm[0] Wm[1] Wm[2] Wm[3] Wm[4] Wm[5]`。
- nibble 展开 ⇒ **reader 局部 k≤11 ← writer 同 long 局部 k+4**（全局 **+4**）；**reader 局部 k=12..15 ← writer 上一个 long 局部 0..3**（全局 **−28**，顺序反转）。
⇒ **每个位置都解错**（随 i 连续变化）——这正是「`i=0` 就 mismatch」的原因，与「移位 4 个 4-bit 位置」的直觉**一致**。

**为何异常取值只有 {2,8}（离散）**：越界条件 = 解出的值 **≥ 该 section 的 `palette.size`**。
- `k≤11` 与（m≥1 的）`k≥12` 解出的都是 **writer 的真实 palette 索引**（`0..size-1`）⇒ **永不越界**（错但不抛）。
- 唯一非 writer 索引的来源 = **section 首个 long 的 `k=12..15`**：那里没有「上一个 long」，取到的是**前缀字节** `0x80 0x02` 的 nibble = `e15=8, e14=0, e13=0, e12=2` = **{8,0,0,2}**；0 恒有效 ⇒ 越界集合 **⊆ {2,8}**，且 **2 需 `size ≤ 2`**、**8 需 `size ≤ 8`**。
- 扫描顺序（contentLine `BulkWb.java:360-366`：section 外层，然后 `y→z→x` = 线性索引递增）决定「首个越界位置」= 该 section 线性 **12**（size=2 → 抛 index 2）或 **15**（size 3..8 → 抛 index 8）。
⇒ **两种说法都对，是同一 2 字节错位的两个投影**：**位移解释**回答「值域整体错」（连续、每格都错、i=0=granite）；**nibble 解释**回答「**异常**取值离散」（只有首 long 尾 4 位可能越界，其来源是前缀 nibble）。

**⚠ 对 scout-map §6 I2 的修正（显式取代，不改原文）**：I2 原文「污染位置 = 线性索引 12..15」**不准确**——受影响的不是 4 个位置而是**整个 4096 元素数组**（全局 +4 / −28 位移）；12..15 只是**可能越界**的位置。修正后 I2′：`{2,8}` 是**越界值集合**，不是**受影响位置集合**。

**本机制附带的三个可判预测（未测，可作后续单变量判据）**：
1. section `palette.size ≥ 9` 时该 section **不产生异常**（8 有效）——但同 chunk 更小的 section 仍会先抛。
2. 41 个 chunk 的**首个非空 section** `size = 2`；9 个 `size ∈ 3..8`（分布自洽；与 1.20.1 `max_distinct=7`（`log-1.20.1-bulklog.txt:2567`）的取值域一致）。
3. 位置级终极探针：在自检后打印同一容器 `pc.get(0,0,0)` 与 `pc.get(4,0,0)`，本机制预测二者**相等**（均为 writer 位置 4 的值）。

**vanilla carver 同点再证**：本臂 `:286-307` 又是完整崩溃报告（`Exception generating new chunk`、index 8），栈顶 `ArrayPalette.get(:76) ← PalettedContainer.get(:163/:157) ← ChunkSection.getBlockState(:51) ← ProtoChunk.getBlockState(:109) ← Carver.carveAtPoint(:133)`；`:478` 帧带 `~[minecraft-merged-b07cf08c30-1.21.6-net.fabricmc.yarn.1_21_6.1.21.6+build.1-v2.jar:?]` ⇒ 与 A1 输入 jar **同版本同映射 build 名**（严格 sha 同一性未核，见 U9）。

### 10.4 ③ U1/U4/U6/U7 逐项更新（＋新增 U9）
| 编号 | 原状态 | 现状态 | 依据 |
|---|---|---|---|
| **U4**（运行探针未执行） | 未闭 | **闭合** | C1 已跑，b2a 运行级落实（10.2） |
| **U7**（`createDataProvider` 实现未核 = b2b 唯一缝隙） | 未闭 | **闭合（主会话）** ⇒ b2b 证据面排除 | 10.2 判据链 |
| **U1**（DIAG 抛点归属 Heightmap vs contentLine） | 未闭 | **仍不可判，且已降级为不重要** | 两读点（`CppBridge.java:660` / `BulkWb.java:365`）**产出同一 {2,8} 签名**且都在 `writeChunk` 内（异常被 `CppBridge.java:495-499` 归因到 chunk 层）；**新增证据**：contentLine 的打印点（`:381`）在 nz 扫描（`:365`）**之后** ⇒ `[WG-CONTENT-WB]`=0 **同时兼容**「heightmap 抛」与「contentLine nz 抛」，**不能**用来定归属 |
| **U6**（1.21.6 bulk 臂 `max_distinct`） | 未闭 | **仍缺数据，但降级为非判据（不再需要）** | 修正后的机制（10.3）**自身**即保证越界值 ⊆{2,8}，与 `max_distinct` 无关（后者只影响哪个索引先出现）。附带诊断：本臂未打印 `[WG-BULKWB] calls=` 的真因是 **destroy/shutdown hook 未完成**（`CppBridge.java:736 destroy` → `:746 BulkWb.reportSummary()`，注册于 `:818-824` 的 `coreswap-destroy` 钩子），**不是**「未启用分块编码器」——`LOG` 已开（`:139` 的 `state_ids_size=` 行来自 LOG 门控的 `reportOnce`，`BulkWb.java:325-332`） |
| **U9**（新） | — | 未闭 | C1 崩溃帧只给出 loom 的 `minecraft-merged-b07cf08c30-…jar` 名；A1 记录的 jar sha 为 `e6f80ac9604aadd3…`（`A1-commands.txt:6`）⇒ 建议核 `Get-FileHash runtime\1.21.6\java\.gradle\loom-cache\…\minecraft-merged-b07cf08c30-…jar` 与 A1 jar 是否同一 sha |

### 10.5 ④ 运行台 `tool-run_1216_arm.ps1` 复核（哪些缺陷会影响我判读）

| 缺陷 | 是否影响本轮判读 | 说明/证据 |
|---|---|---|
| **D1 fp 口径混淆** | **影响（已计入）** | `:121` 把 `[WG-CONTENT]`（**Rust buf 层**，`CppBridge.java:487-493`）与 `[WG-CONTENT-WB]`（**Java 读回层**，`BulkWb.readbackHash:338-355`）收进同一文件，且两层 `hash=` 是**不同哈希**。本臂 `wb=0` ⇒ 该 fp **实际只含 Rust buf 层**（实测 49 行全为 `[WG-CONTENT]`）⇒ **不可**作写回内容指纹，**不可**与修复后 `[WG-CONTENT-WB]` fp 比对（违反 §9.7 三要素） |
| **D2 fp 前缀未剥净** | 影响机械比对 | `:121` 的 `-replace '^\[[^\]]*\]\s*',''` 只剥掉**时间戳**（日志行首为 `[16:30:24] [Worker-Main-N/INFO] (Minecraft) [STDOUT]: …`）⇒ 实测 fp 行首仍为 `[Worker-Main-1/INFO] (Minecraft) [STDOUT]: [WG-CONTENT] chunk(…)`。跨臂线程分配不同即假差异 |
| **D3 早退截断** | **影响覆盖面主张** | `:13/:101 ThrowAbortAt=3` + 10s 轮询 ⇒ cascade 全部发生在同一秒（16:30:24），49 chunk 后即停 ⇒ 计数型判据**只代表前 ~49 chunk**（不可说「所有 chunk」）；且 destroy 未完成 ⇒ `calls=` 缺失（见 U6） |
| **D4 status 语义** | 无（本臂实际 EARLY-ABORT 正确） | `:111` 的 `EARLY-ABORT` 不等于「未完成」，仅表示轮询到 cascade |
| **D5 ARM-VOID 与 fp 的耦合** | 无（本轮已修） | `:82` 正则已修（曾因 `dll= sha256=` 误标）；**残留建议**：`:120-122` 在 `-not $dllOk` 时仍生成 fp ⇒ ARM-VOID 臂的 fp 应一并作废（`if(-not $dllOk){不写 fp}`） |
| 其余（`:28-31` 磁盘 dll 硬门、`:82/:85` in-log 血统自证、`:50` 删 world、`:53` seed、`:54` 清 native tmp） | **无缺陷**，符合 E1/seed 三查纪律 | 直接读 |

**净影响**：D1/D2 **不影响** ①②③ 的结论（这些只依赖 in-log 行与 WBTEST 行）；D3 使「49/49 全覆盖」类主张不成立（改用「49 个 chunk 各抛 1 次」），但不影响 `{2,8}` 与 `i=0` 判读。

### 10.6 ⑤ F1/F2/F3/F5 排序与「F1 是否落在 HOOK-1/HOOK-2 已批准范式内」的依据

排序**不变**，但依据加强：C1 的 SINGULAR 支通过说明除 storage 段外**其余各步已对齐**（byte→palette→storage 三步中前两步在两版同形且 1.21.6 上往返正确），故最小改动面**确实**落在 `BulkWb.java:247/:262` 两处。

**F1 与「共享超集 + 分版常量门控」范式的吻合性（只给依据，决策归 HOOK-C）**：
- **逐条吻合的事实**：① `WgCompat.java` 就是该范式的**官方分版缝**，其 `:3` 注释即「共享文件不得出现版本分支/映射差异，全部收进本类」；② 其形态 = **两个 `public static final boolean` + 一个 `flag()`**（1.20.1 `:8/:11/:14-17`，1.21.6 `:9/:13/:16-19`）；③ F1 的落点是**共享超集** `BulkWb`（`java-core`），门控量取自**分版件** `WgCompat` ⇒ 与范式**一一对应，无新增架构元素**；④ 常量是 compile-time constant ⇒ 被内联（`BulkWb.java:73` 同族），JIT 可折叠出不可达分支，1.20.1 分支**无热路径成本**。
- **范式内但需写清的一点**：该范式**不保证「1.20.1 产物字节不变」**（§9.2 自我修正）——范式保证的是「共享源无版本分支、差异集中于分版类」。因此 **V1b 判据应表述为行为/写出帧等价**；若判据坚持「1.20.1 字节零差」，则**唯一**满足者是 **F5**（1.20.1 不参与构建）。
- **需与 F1 同批处理的细节**：`EMPTY_LONGS` 支（`:247`）必须同步分版，否则 1.21.6 残留 1 字节（SINGULAR 无功能影响，但会给「残留字节」探针留下第二个不可解释值）。
- **方案归属判读**：**F1 = 范式内**；**F5 = 范式内但把差异从「我方写入」移到「vanilla 读取」**（新增 1.21.6-only mixin 条目，属范式既有的分版件形态，但改的是原版热路径）；**F3 = 范式内**（分版件 + 构建源集接线）；**F2 = 范式外**（引入运行期自探测，破坏「分版常量静态可读」的设计意图）；**F4 = 范式外且不建议**。

### 10.7 本 §「事实 vs 推断」分列
- **事实（行号/计数可查）**：10.1 全表；`:138/:139/:140` 三行原文；41×2/9×8/50/49/0；`blocks.json:926 stone=1`、`:400 granite=2`；`BulkWb.java:294/:295-311/:317-321/:360-366/:381/:325-332`；`CppBridge.java:487-493/:495-499/:660/:670/:736/:746/:818-824`；`PackedIntegerArray.java:307-311`；fp 文件 49 行内容与行首形态；`:286-307/:478` 崩溃栈与 jar 名；`raw-c-bulk.log:116/:131-136`。
- **推断（显式）**：10.3 的 nibble 位移全貌推导（关键一步「reader 0 ← writer 4」已被 granite 观测**证实**；其余 long 的 −28 映射为推导）；「41 个 chunk 首个非空 section size=2 / 9 个 3..8」为分布推断；「palette ≥9 不抛」为本机制的可判预测（未测）；U9 的 jar 同一性。
- **本 §不主张**：不主张根因已 confirmed；不写 candidate；不做方案决策（10.6 只给范式吻合性依据）。
