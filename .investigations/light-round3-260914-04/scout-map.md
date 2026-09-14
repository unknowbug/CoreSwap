# 光照 round3 P1 勘探图（scout-map）

> 角色：recode.scout（只读勘探）。日期标签沿用父会话工作块 260914-04。
> 状态：draft（静态读出，未运行验证）。所有行号为本轮实际读出。

## 1. 现状成本账（静态推演）

### 1.1 内核 fill 路径（`worldgen-core/src/light/mod.rs`）

- **查表结构**：`LightEngine.table: Vec<(u8, u8)>`（opacity, emission 同表，mod.rs:57），初始 1024 项、按最大 id 扩容（mod.rs:89-95）。查表入口 `lookup()`（mod.rs:119-124）：`id<0` 分支 + `table.get(id).copied().unwrap_or(table[0])` —— **每次非 air 查表 = 一次边界检查 + Option 解包 + 冷路径回退拷贝**。
- **每格成本**（fill 主循环 mod.rs:210-247，四层 y/z/cx/lx）：读 `blocks9[base+lx]`（i32）→ `air_fast && v==0` 快路径（mod.rs:226-229，air 恒 0 写）→ 非 air：位拆 id/lum（mod.rs:230-231）→ `lookup` → 条件写 opacity/block_light/queue.push/col_max（mod.rs:236-243）。**col_max 已同趟融合**（round2，mod.rs:209 注释）；fill 与 col_max 无再融合空间——剩余可融合面 = opacity 表内联（消 `get`+Option）与「fill×sky_fall 融合」（sky_fall L266-273 目前是独立的 15-区间写循环，读 col_max 后逐格写 sky_light）。
- **scratch 清零成本**：每次调用 `opacity/block_light/sky_light` 三块 884736×1B clear+resize（mod.rs:195-203）≈ 2.65MB/次 memset（opacity 依赖全 0 语义支撑 air 快路径）。
- **export**：`export_center`（mod.rs:403-439）——中心 98304 格 × 双通道 min/max + nibble 打包，无逐格函数调用，纯内联循环；「均质 section 跳过」在 export 侧已由 flag 机制覆盖输出侧，但**输入侧（blocks9）无均质信息**（int[] ABI 不携带）。
- **phase 分解引用**（历史，12-lighting.md L76-77 round2 口径）：fill 943µs(58%)/sky_fall 300µs/sky_seed_bfs 206µs/export 157µs/block_bfs 8.8µs，内核 1.587ms/chunk。本轮静态读出与该结构自洽（fill 是唯一逐格查表面）。

### 1.2 Java 收集循环（`ServerLightingProviderMixin.java`）

- `wgLightCollectBlocks`（mixin L128-183）：空节 `sec.isEmpty()` 短路 `Arrays.fill(0)`（L162-167）；**非空节逐格** `sec.getBlockState(x,ly,z)` × 4096 + `Registries.BLOCK.getRawId(st.getBlock())` + `st.getLuminance()`（L169-178）——**非空节每 chunk 调用数 = 4096×非空节数**（上限 98304/chunk，即任务书数字；实际 ≈ 4096×地表实心节数，surface 区约 24 节中 8-15 节非空，量级 3-6 万/chunk）。每次 getBlockState 走 PalettedContainer.get → palette.get(storage.get(idx)) 三级间接。
- **PalettedContainer 内部可读性（静态判断，未运行验证）**：yarn 一手源 `.tmp/scout-260905-08/mcsrc/net/minecraft/world/chunk/PalettedContainer.java`——`data` 为 `private volatile PalettedContainer.Data<T>`（L36）→ **mixin `@Accessor("data")` 可行**（字段直接命名，无混淆障碍，同项目已有 ChunkSectionAccessor 先例，`versions/1.20.1/java/.../ChunkSectionAccessor.java` L29 已 @Accessor blockStateContainer 成功交付）。`Data` 是 **record**（PalettedContainer.java L361-392）：`storage()`/`palette()` 公共访问器免 accessor。`PaletteStorage.getData() → long[]` + `getElementBits()`（PackedIntegerArray 实现，见 L384-386 writePacket 用法）；`Palette` 接口有 `get(int)` + `getSize()`（L336 `data.palette.getSize()` 用例）。⚠️ 线程安全：`data` 是 volatile，LIGHT 阶段邻 chunk 已静置，读快照一致（静态推断，未运行验证）。

### 1.3 JNI 边界（`versions/1.20.1/rust/src/jni_bridge.rs`）

- `Java_wg_CppWorldgen_lightCompute`（jni_bridge.rs:334-401）：长度校验（L345-350）→ `get_int_array_region` 整块拷 blocks9（**884736×4B ≈ 3.5MB/chunk**，L357）→ thread_local 缓冲（L290-292，round2 已免堆分配）→ `light_compute` → 三次 `set_byte_array_region` 回写（L391-393，98KB+98KB+48B）。
- **拷贝次数/chunk**：Java TL 缓冲（Java 侧 7MB/线程，mixin L118-126）→ JNI get 拷入 Rust TL → 结果 set 拷出 = **blocks9 一次跨界大拷（3.5MB）+ 输出两次小拷**。数组是 JNI critical/region 拷贝语义（get_int_array_region 必拷或 pin，实现相关）。

## 2. 候选前提一手核对

### 候选 A：内核 fill 降维 —— **成立（部分），待运行验证收益**
- opacity 表内联：表已是 `Vec<(u8,u8)>`（u8 域本就满足，**无需 u8 改造**，mod.rs:57）；「内联」实指消 `lookup` 的 `get().unwrap_or`（mod.rs:123）→ 预扩容到固定长度后直接 `unsafe get_unchecked` 或 split opacity-only 紧表（1003 项 × 1B，缓存友好）。opacity 域 0..15（parse_u8_field clamp，mod.rs:135），u8 域成立。
- fill-col_max 融合：**已存在**（mod.rs:209 注释 + L241-242 同趟）——该子项无剩余空间；可换目标是 **fill×sky_fall 融合**（sky_fall L266-273 独立循环 → fill 已知 col_max 可在 y 循环内联写 sky 15 区间，但 sky 区间依赖最终 col_max，需倒序 y 或两趟，静态可行待推演）。
- 均质 section 跳过：**前提不成立于现 ABI**——blocks9 int[] 不携带 section 边界/均质信息；内核无法识别均质段（除非 sentinel 值协议，属候选 C 改形面）。
- light_data.json：33455 B，blocks 条目 **1003 项**（本轮 python 读出）——表规模小，全表驻 L1/L2 可行。

### 候选 B：Java palette 级批量展开 —— **静态成立，未运行验证**
- 空节 isEmpty 短路已在位（mixin L162-167）；非空节 palette 规模：BLOCK_STATE PaletteProvider（PalettedContainer.java L420-430）——1-15 项走 ArrayPalette（bits 4），即典型非空节 palette 1-15 项，**远小于 4096**。
- 展开路：@Accessor 拿 `data` → record 访问器 `palette()`/`storage()`（L361）→ 枚举 palette（`getSize()`×`get(i)` 得 BlockState → 预编码 rawid|lum ≤15 项小表）→ `storage.getData()` long[] 位流解码 4096 项（bits=storage.getElementBits()；PackedIntegerArray 索引序 = mojang 序，与 blocks9 的 (y,z,x) 行主序一致性**需核对 PackedIntegerArray 逐位序**——未核对，列开放问题）。Java 侧每格成本从「3 级间接 getBlockState + 2 次注册表查询」降为「位流移位 + 小表索引」。
- 降调用次数：`getBlockState` 98304→0（非空节）；`Registries.BLOCK.getRawId` 98304→paletteSize×节数（≈几十/节）。
- **readPacket 不适用**（那是网络反序列化入口，L203-215，语义不同）；正确抓手是 data record 直读。静态可行性依据：字段/访问器链全公有可及（私有 data 一层 @Accessor 即通）。

### 候选 C：JNI 传参改形（palette+packed storage 直传）—— **静态成立，改动面中等，未运行验证**
- 前提同 B 的 accessor 链（Java 侧取数成本与 B 共享）。
- 新 JNI 形态设想（勘探列举，不设计）：`lightComputePacked(handle, int[] sectionMeta /*每节 [bits, paletteLen]×216*/, int[] paletteIds /*rawid|lum 扁平*/, long[] storageAll /*各节 packed long 拼接*/)` —— 传参量 ≈ paletteIds(≤~3k int) + storage（bits4×4096×24节 ≈ 48KB long[] + 空节 0）vs 现 3.5MB，**降 ~2 个量级**。
- Rust 侧需新增：PackedIntegerArray 解码器（mojong 位序）+ palette 映射展开 → 仍填现有 blocks9/直接填 opacity（可与候选 A 融合成「展开+查表+col_max 单趟」）。
- 两侧改动面：Java mixin 收集段重写（L160-179）+ 新 accessor（PalettedContainer.data）+ jni_bridge 新导出 + worldgen-core 新解码入口；blocks9 旧 ABI 可并存（回退保留）。
- 位序风险：PackedIntegerArray 的 mojang 除魔数索引（PackedIntegerArray.java L17-31 INDEX_PARAMETERS）与 (y<<8|z<<4|x) 的对应关系需实测对拍（开放问题，golden 门覆盖）。

## 3. 代码漂移清单（260905 round2 基线 → 今）

`git log --since=2026-09-05` 三文件路径：

| commit | 日期 | 影响 |
|---|---|---|
| 76c2104 | 09-05 11:55 | round2 本身（基线） |
| 44d8ef7 | 09-05 15:32 | feature parity 4b（触碰 worldgen-core，**light/mod.rs 末次改动即此**——内容以 git show 为准，本轮未逐行 diff；phase 分解数字引用自 12 篇，载体为 round2 后构建，需 P4 重建时用内容指纹哨兵核对 #30） |
| 40e406a / 2f05cfd | 09-07 | jni_bridge.rs 增 wg_register_block（**光照导出函数未动**，本轮读出 L334-401 与 12 篇描述一致） |
| d5151a1 | 09-10 22:56 | Java 工程路径迁移 runtime/→versions/（**纯 rename，ServerLightingProviderMixin 逻辑零改动**，git log --follow 确认无中间提交） |

**结论**：光照三文件自 round2（76c2104/44d8ef7）后无逻辑漂移；旧 phase 分解数字（fill 943µs 等）**可引用**，但载体为当时 rlib，重建微基准仍须 #30 指纹哨兵。

## 4. e2e 复现面

- ON 臂拦截点：`ServerLightingProvider.light(Lnet/minecraft/world/chunk/Chunk;Z)` HEAD + `cir.setReturnValue`（mixin L185-187）——即绕过 vanilla `propagateLight`（yarn LightingProvider L97/L79，mixin 头注释 L7）；OFF 臂 = 不设 gate 属性时 mixin L188 直接 return = **完全 vanilla 路径**（propagateLight 原样）。
- 四臂脚本**仍在**：`.tmp/d3-opt-260905-04/e2e_run.ps1`（参数 `-Arm N -GateOn`，删 world + `-PlightRust=1` gate + [LightRust] 行计数校验 fallback=0）+ `e2e_round2_seq.ps1`（交替序）。运行环境 `runtime/1.20.1/java/run`。**wall 口径可复现**（脚本未删、gate 语义未变、12 篇 L94 口径 = ON 中位 17.4 vs OFF 13.9）。注意 Java 工程已迁 versions/1.20.1/java（d5151a1），脚本内 $run 路径需 P4 复核一次。

## 5. 开放问题（只列不定论）

1. PackedIntegerArray 的元素位序（long 内跨项 packing 顺序）与 blocks9 (y,z,x) 行主序是否逐位一致？（决定 B 的 Java 解码与 C 的 Rust 解码正确性；golden 对拍可覆盖）
2. 非空 section 的实际 palette 规模分布（真实地形，预期 2-15 项/节）——需 dump 实测，才能定 B/C 收益上限。
3. `get_int_array_region` 在 JVM（TEMURIN 17?）是否走 pin 还是拷贝——影响 JNI 拷贝次数账（静态无法定）。
4. fill×sky_fall 融合的等价性（sky 15-区间需列完整 col_max，正序 y 单趟不可得）——worker 推演。
5. 非 air 且 id=0（lum>0 光源）在真实数据中的占比——影响 air 快路径命中率的余量估算。
6. `lightNativeDead`/fallback 计数在四臂 ON 臂是否恒 0（脚本已计数，P4 引用）。

## 6. 本轮静态读出 vs 历史引用标注

- 静态读出：查表结构/每格成本形态、col_max 已融合、JNI 拷贝形态、accessor 链可行性、脚本在位、漂移清单、light_data 1003 项。
- 历史引用：phase 分解数字（12 篇 L76-77）、e2e 17.4/13.9（L94）、Java 收集 5.6ms 级历史量级（L49，round1 口径已被 round2 收窄）。
