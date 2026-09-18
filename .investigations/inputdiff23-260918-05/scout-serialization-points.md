# inputDiff23 scout：packed payload 序列化点与最小侵入 dump 方案（260918-05）

只读静态勘探（recode-scout 语义），未运行任何构建。置信度：**candidate**（全部 file:line 静态证据，未运行验证）。日期标签 260918-05 由任务指派沿用，未重取 Get-Date（scout 只读，产物名由任务给定）。

## 1. 序列化点定位

### 1.1 vanilla 一手源（yarn，.tmp/scout-260905-08/mcsrc/）

文件：`.tmp/scout-260905-08/mcsrc/net/minecraft/world/chunk/PalettedContainer.java`

| 点 | 位置 | 内容 |
|---|---|---|
| `PalettedContainer.writePacket(PacketByteBuf)` | L217-226 | lock → 委托 `this.data.writePacket(buf)` → unlock |
| `Data.writePacket`（私有 record `Data` L361） | L383-387 | **三段组装**：① `buf.writeByte(storage.getElementBits())` ② `palette.writePacket(buf)` ③ `buf.writeLongArray(storage.getData())` |
| `Data.getPacketSize` | L379-381 | `1 + palette.getPacketSize() + varInt(storage.getSize()) + storage.getData().length*8`（帧组成旁证） |
| `PalettedContainer.readPacket` | ~L196-215（L210 `readLongArray(data.storage.getData())`） | 反序列化对偶 |

palette 各实现的 `writePacket`：
- `BiMapPalette.java` L84-91：`writeVarInt(size)` + 逐项 `writeVarInt(idList.getRawId(entry))` —— **palette 序即此循环的写出顺序**
- `SingularPalette.java` L72-78：裸 `writeVarInt(rawId)`（无 size 前缀；未初始化 palette 抛 `IllegalStateException`）
- `ArrayPalette.java` L94、`IdListPalette.java` L49 同名方法

### 1.2 ChunkSection 序列化调用点（本项目源内）

文件：`versions/1.20.1/java/src/main/java/wg/bench/mixin/ServerLightingProviderMixin.java`
- **L335**：`sec.getBlockStateContainer().writePacket(pbuf)` —— 候选 C packed 收集（`wgLightCollectPacked` L303-371，9 邻 × 24 节全量 writePacket 进 ByteBuf）
- **L415**：`sec.getBlockStateContainer().writePacket(pbuf)` —— 候选 B 快路径（`wgLightFillSectionFast` L410-468，单节 writePacket 后手工解码）
- **L341-365** 已是 packed 帧完整拆解实现：bits（L341 readUnsignedByte）→ singular 分支（L342-349）→ palette size+entries（L352-358）→ long array 长度前缀 + 数据（L360-363）

注意：这是 **light 接管路径的输入快照**（收集 3×3 邻域），不是网络/存档写出点；但对 inputDiff23 而言，两次 reload 走同一收集代码，hash 不稳定即此 writePacket 帧不稳定——dump 点选这里是**零新增注入面**。

### 1.3 现有 mixin 清单（chunk/section/palette/light 相关）

`versions/1.20.1/java/src/main/java/wg/bench/mixin/` 下：

| 类 | 性质 | 与本课题关系 |
|---|---|---|
| `ServerLightingProviderMixin.java` | @Mixin(ServerLightingProvider)，`light(Chunk,boolean)` HEAD 注入（L743-744） | **含全部 writePacket 调用点（L335/L415）+ LIGHT-BETA hash 打点（L673/L695）——首选复用** |
| `ChunkSectionAccessor.java` | @Accessor：blockStateContainer / 三个计数（读写） | 可直接取容器，无需反射 |
| `LightingProviderAccessor.java` / `ChunkLightProviderAccessor.java` | @Accessor | 邻 chunk provider，不涉 payload |
| `ThreadedAnvilChunkStorageAccessor.java` | @Accessor（releaseLightTicket） | 不涉 payload |
| 其余（BlobProbe/ColProfProbe/SurfaceDump/NoiseDump/EstDump/AquiferDump/ConfiguredFeatureProbe/DiagFeatureBiome/…） | worldgen 探针 | 不涉 palette/payload |

## 2. [LIGHT-BETA] 探针现状（inputDiff23 数据源）

全部在 `ServerLightingProviderMixin.java`（env 门控 `-Dcoreswap.light.betaprobe`，L115）：

- **hash 定义**：`wgBetaHash` L202-209（int[] FNV-1a）+ L213-222（long[] FNV-1a，高低 32 位各一步）
- **packed 臂打点** L667-675：`hash = FNV(meta[432]) → FNV(pal[0..packedLens[0]]) → FNV(sto[0..packedLens[1]])`，输出行 `[LIGHT-BETA] chunk(x,z) abi=packed hash=… emptySec=…`
- **blocks9 臂打点** L691-697：`abi=blocks9 hash=… emptySec=…`
- **emptySec 计数**：`WG_BETA_SEC_CUR` 在收集循环空节分支递增（L328 packed 臂 / L248 blocks 臂）

关键判断：**能直接扩展**。packed 臂打点时（L667-675），`WG_META_TL` / `WG_PAL_TL` / `WG_STO_TL` 三缓冲内容尚在（JNI 调用在 L676 之后），meta/pal/sto 全量数据可在此处直接序列化输出，无需新 hook。局限①：emptySec 目前只计数不带节位号（哪个节为空未知——判定候选三「空节集合不稳定」需要位号）；局限②：缓冲是 ThreadLocal 复用，dump 必须在打点处同步写、不能异步；局限③：blocks9 臂只有解包后的 rawId 视图，palette 序/位打包形态在其上不可见——payload 级对比应以 **packed 臂**为准（blocks9 臂 hash 不变 + packed hash 变 ⇔ 帧形态自由度/palette 序问题，本身就是一个判别信号）。

## 3. 最小侵入 dump 方案候选

**候选 A（推荐）：扩展现有 LIGHT-BETA 打点输出 payload 字段**
- 位置：`ServerLightingProviderMixin.java` L667-675（packed 臂）与 L691-697（blocks9 臂）
- 做法：hash 计算点旁把 `meta`、`pal[0..n]`、`sto[0..n]` 以 hex/十进制逐元素写入输出（或写到独立 dump 文件，env 门控如 `-Dcoreswap.light.betadump=<dir>`），并补 emptySec 的**节位号列表**（L328 处把 `s` 记入 bitset）
- 侵入面：仅探针分支（`LIGHT_BETAPROBE` 门控内），生产路径零变化；无新 mixin、无新注入点
- 证据：L335（writePacket 已在此路径）、L668-670（三缓冲在打点时仍可读）

**候选 B：新增 mixin 注入 writePacket 点**
- 做法：@Mixin(PalettedContainer.class) 或对 ChunkSection 注入，在 `writePacket` 返回时把 ByteBuf 快照 dump
- 优点：覆盖所有 writePacket 调用者（含 vanilla 网络/存档路径）
- 缺点：PalettedContainer.Data 是私有 record（LightPalDump.java L2-4 注释明确反射原因），mixin 目标应为 `PalettedContainer.writePacket`（公开方法，可注入）；但该点在 vanilla 网络发送/ChunkSerializer 也会触发，dump 量大且混入非 light 路径流量；另外**引入新 mixin = 新注入面**，违背「最小侵入」
- 证据：PalettedContainer.java L218（公开注入目标）；现役项目 mixin 全部集中在 wg.bench.mixin（26 文件，glob 证据），无 PalettedContainer mixin 先例

**候选 C（补充面）：复用 LightPalDump 反射桥做全量 palette 表 dump**
- 位置：`LightPalDump.java` L64-79 `section()`——已通过反射拿到 `Data.palette`/`Data.storage` 实例
- 做法：直方图计数旁输出 `pal.getSize()` 全表 + `storage.getData()` long 全量（`PaletteStorage` 公有接口，无反射风险）
- 定位：候选 A 的交叉验证面（反射视角 vs writePacket 字节流视角，双通道对拍防 dump 自身出错）

结论：**A 为主、C 为对拍、B 不取**。

## 4. writePacket 帧组成与三候选的字节段映射

`Data.writePacket`（PalettedContainer.java L383-387）产出的节级字节流：

```
[byte  bits] [palette 段] [varInt longLen] [longLen × 8 字节 long array]
```

| 候选机制 | 对应 dump 字段 | 观察方法 |
|---|---|---|
| **palette 序不稳定** | palette 段内的 `varInt rawId` 序列（BiMapPalette.java L84-91 写出顺序 = Palette 内部 map 序；`meta[si+1]`=psz） | 对比同 chunk 两轮 dump：若 **long array 逐字节相同而 palette 表元素顺序不同** → palette 序问题（表与位流索引解耦错位）。注意 BiMapPalette 是 `net.minecraft.util.collection.Int2ObjectBiMap`/LinkedObjBiMap 风bage，其遍历序受插入序/扰动影响是已知自由度 |
| **位打包形态自由度** | `bits` 字节（= `meta[si]`）+ long array 段（`sto[]`） | bits 变化（palette 增缩触发 resize → 位数变化）会使同一 4096 方块集产生完全不同的 long 流；bits 相同而 long 流不同 = 位流内容/序真差（或 PalettedContainer resize 后索引重排自由度） |
| **空节集合不稳定** | dump 行的 `emptySec` 计数 + （扩展后）节位号集合；packed 帧中空节**不产生任何字节**（L327-332 直接 skip，meta[si]=meta[si+1]=0） | 两轮 dump：某节一轮有帧、另一轮为空（或反之）⇔ `sec.isEmpty()` 瞬态差异——**这不是 writePacket 字节内的字段，而是「该节是否出现在 payload 中」的存在性字段**；48 节中 ±1 个空节翻转即可改变 hash 而方块内容恒等（空节语义 = 全 AIR 等价） |

meta 缓冲布局（L329-330/L364-365）：每节 2 int = `{bits, psz}`，空节为 `{0,0}`——meta 本身已编码「空节集合」（全 0 对）+「bits 形态」，故候选 A 只 dump meta+pal+sto 三段即可完整重建全部三候选的判别面。

## 5. 附注与风险

- 候选 C 收集失败/global 节（bits≥15）会整 chunk 回退 blocks9 臂（L351 `bits>14 return null`）——两轮若一轮 packed 一轮 blocks9，hash 语义不同，dump 对比 MUST 按 abi 字段分组（§9.7 等价分层：packed 臂与 blocks9 臂不可比）。
- 域批路径（`LIGHT_DOMAIN`，L510-531）blocks9 收集在提交线程、无 LIGHT-BETA 打点——inputDiff23 复跑 MUST 关 domainbatch 或确认其开关状态，否则打点缺失。
- LightDomainBatch.java L44 注释：域任务线程零 PalettedContainer 访问——dump 逻辑不得移入域任务线程。
