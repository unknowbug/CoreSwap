# native 光照计算路径机制地图（scout-map）

> 角色：recode/性能 scout（只读勘探）。日期标签沿用父工作块 260919-02。
> 状态：draft（静态读码 + 历史结论回一手核对，未运行任何验证）。
> 继承输入：A1 verdict（confirmed，`.artifacts/a1-260919-01/verdict-260919-01.md`）——域批任务三段 P50：fill=0.302 / native=4.264 / wb=0.079 ms，native ~92%。
> 标注约定：【实证】= 本轮读码/一手文件确认；【推断】= 静态推演，需 T3 采集验证。

## 1. 调用链地图（Java 打点 → JNI 入口 → Rust 内核，file:line）

```
[Java] ServerLightingProviderMixin.wgLightDomainSubmit（Mixin:533）
  ├─ wgLightCollectBlocks（提交线程，逐 chunk 9 次填 b9；Mixin:541）——不计入 fill/native/wb 三段
  ├─ Arrays.copyOf 快照（Mixin:545）+ LightDomainBatch.submit（Mixin:548）
  └─ taskFactory → CompletableFuture.runAsync(wgLightDomainTaskRun, mainWorkerExecutor)（Mixin:501-503）
[Java] wgLightDomainTaskRun（Mixin:579，worker 线程）
  ├─ centers 排序（Mixin:580）＋ wgLightEnsureInit（Mixin:590）
  ├─ blocks25 = new int[2457600]（Mixin:597，~9.8MB/任务）
  ├─ assemble：逐中心 9×System.arraycopy 拼 5×5 帧（Mixin:600-615）……┐
  ├─ _t1（Mixin:620）＝ fill 段上界（fillNs=_t1-t0，Mixin:627）←────┘ fill 段覆盖以上全部
  ├─ wg.CppWorldgen.lightComputeDomain(handle, blocks25, out)（Mixin:621）
  │    └─ [JNI] Java_wg_CppWorldgen_lightComputeDomain（jni_bridge.rs:424）
  │         ├─ 长度校验（jni_bridge.rs:433-437）
  │         ├─ b25 = vec![0i32; 2457600]（jni_bridge.rs:439，~9.8MB 新分配）      ← native 段
  │         ├─ get_int_array_region 拷入 9.8MB（jni_bridge.rs:440）               ← native 段
  │         ├─ outv = vec![0u8; 885168]（jni_bridge.rs:441）                      ← native 段
  │         ├─ catch_unwind(light_compute_domain)（jni_bridge.rs:443-445）        ← native 段
  │         │    └─ [Rust] light_compute_domain（worldgen-core/src/light/mod.rs:308）
  │         │         ├─ Scratch::new（mod.rs:319；空 Vec，见 §2-A3）＋ b9=vec![0i32;884736]（mod.rs:320，~3.5MB）
  │         │         └─ for k in 0..9（mod.rs:321-336）—— 9 中心循环：
  │         │              ├─ 子窗拷贝 b9←blocks25 9×copy_from_slice（mod.rs:325-331，每中心 ~3.5MB）
  │         │              └─ light_compute_inner（mod.rs:335 → :340）：
  │         │                   ① scratch clear+resize ~2.7MB（mod.rs:361-369）
  │         │                   ② fill：884736 格逐格 air_fast( mod.rs:392 )/lookup( mod.rs:398 )/col_max( mod.rs:408 )（mod.rs:376-413）
  │         │                   ③ block BFS bfs_propagate（mod.rs:420 → :503-544；try_spread :546）
  │         │                   ④ sky 直落 col_max 区间写（mod.rs:432-439）
  │         │                   ⑤ sky 种子区间算术（mod.rs:451-482）＋ sky BFS（mod.rs:483）
  │         │                   ⑥ export_center 中心 98304 格 nibble 打包（mod.rs:491 → :569-605）
  │         └─ set_byte_array_region 拷出 885KB（jni_bridge.rs:463）               ← native 段
  ├─ _t2（Mixin:622）＝ native 段下界（nativeNs=_t2-_t1，Mixin:628）
  └─ 逐中心 wgLightDomainWriteBack（Mixin:634-659 → :562-574）＝ wb 段（enqueueSectionData ×48 + setLightOn + releaseLightTicket）
任务行打印：Mixin:660-669（fillMs/nativeMs/wbMs 均 per-center 均值口径）
```

- JNI 声明侧：`wg.CppWorldgen` native 方法声明（Java 侧入口类，`versions/1.20.1/java/src/main/java/wg/CppWorldgen.java`）；Rust 导出名 `Java_wg_CppWorldgen_lightComputeDomain`【实证，jni_bridge.rs:424】。
- 旁路 ABI（本轮不在测面）：lightCompute per-chunk（jni_bridge.rs:346，走 thread_local 缓冲，legacy 通道）／lightComputePacked（jni_bridge.rs:475，仅 legacy per-chunk 分流，Mixin:696-724）。**域批路径只用 blocks25 宽 ABI，packed 直传未接入域批**【实证】。

## 2. native 段（4.264ms P50）内部构成候选清单

> per-center 口径：native 段一次 JNI 调用 = 9 中心批量。以下块按「每任务（9 中心）」给量级。

| # | 工作块 | 位置 | 量级/复杂度 | 定性 |
|---|---|---|---|---|
| A1 | **JNI 拷入 blocks25** | jni_bridge.rs:440 | 9.8MB 内存搬移；round3 实测 per-chunk 3.5MB 拷入+出 ≈ JNI 边界 ~1.3ms（probe-verdict §1.2）→ 线性外推域批 ~3.4MB 等效拷 + pin/copy 未定 | 实证（结构）+ 推断（耗时外推） |
| A2 | **每次调用新分配**：b25 9.8MB + outv 0.885MB（jni_bridge.rs:439/441）+ b9 3.5MB（mod.rs:320）+ Scratch 五 Vec 首次扩容 ~2.7MB（mod.rs:93-101, 319） | 合计 ~17MB/任务堆分配 + 首触缺页；⚠️ jni_bridge.rs:438 注释「输入帧一次性分配可忽略」假设②在 per-task `new int[...]`（Mixin:597）与 Rust 侧 per-call vec 双侧均未真正复用 | 实证（代码在位）/ 推断（耗时占比） |
| B1 | **子窗重拷** b9←blocks25，9 中心 × 3.5MB（mod.rs:325-331） | 31.5MB/任务纯搬移——blocks25 已含全部数据，b9 是内核 ABI 的冗余二级拷贝 | 实证 |
| B2 | **fill 逐格循环** 884736 格 × 9 中心（mod.rs:376-413） | ~7.96M 格/任务；历史 per-chunk fill ≈ 0.92-0.94ms（12 篇 :52, :99）→ 域批 ≈ 9×0.94 ≈ 8.4ms/任务上限口径（同核外推，需验证）；air 快路径（mod.rs:392）+ 查表 lookup 边界检查（mod.rs:143-148）为主指令面 | 实证（结构）/ 推断（外推值） |
| B3 | **block BFS**（mod.rs:420） | 历史 per-chunk 8.8-13µs（12 篇 :58, :99）——光源种子少，量级可忽略 | 实证（历史） |
| B4 | **sky 直落**（mod.rs:432-439） | round2 后为纯写趟 ~0.30ms/chunk（12 篇 :79, :99）；域批 ≈ ×9 | 实证（历史） |
| B5 | **sky 种子枚举 + sky BFS**（mod.rs:451-483） | round2 区间算术后 ~0.21ms/chunk（12 篇 :80, :99）；域批 ≈ ×9 | 实证（历史） |
| B6 | **export_center**（mod.rs:569-605） | ~0.16-0.21ms/chunk（12 篇 :57, :99）；域批 ×9 = 885KB 输出打包 | 实证（历史） |
| C1 | **拷出 out 885KB**（jni_bridge.rs:463） | 小项，~0.03ms 级（probe-verdict §1.2 出参账） | 推断 |
| C2 | 锁/同步 | **现路径无锁**：Scratch 已 per-call 局部化（CP-1 #150 解粘，mod.rs:14-17, 189-191 注释）；engine 只读 &LightEngine（mod.rs:335）；Mutex 串行化面已消 | 实证 |
| C3 | catch_unwind 包裹（jni_bridge.rs:443） | 单次包装开销可忽略 | 实证（结构）/ 推断 |

**结构性冗余（最大嫌疑，#L5+L6 同族）**：`light_compute_domain` 每中心对**整个 3×3 域（884736 格）全量重算**，9 中心 = 81 chunk 体积的 fill/传播工作只产出 9 chunk 输出（mod.rs:297-301 契约即如此）——与 12 篇 :141 形态审计「结构总量比 ≥9×」同构，且域批化**没有**消除 per-center 域冗余，只是把 9 次 JNI 合成 1 次【实证（代码结构）】。

## 3. 历史结论盘点（已回一手文件核对，#90 转抄漂移律执行）

| 结论 | 一手出处 | 与 native 段关系 |
|---|---|---|
| 内核 phase 分解（post-round2）：fill 943µs(58%) / block_bfs 8.8 / sky_fall 300 / sky_seed_bfs 206 / export 157 µs/chunk | 12-lighting.md:99（round1 表 :51-58 为 post-opt 前一版） | native 段内核侧的直接预拆基线；域批内核 = 该结构 ×9 中心（B2-B6）【实证（历史测量）+ 推断（外推）】 |
| per-chunk 串行账：collect 0.288 + native 2.651ms（round3 后）；native 2.8 − 内核 1.483 = **JNI 边界 ~1.3ms**（pin/copy 未定） | 12-lighting.md:127; light-round3 probe-verdict §1.2 | JNI 边界成本有实测锚，但载体是 per-chunk ABI（3.5MB）；域批 ABI 9.8MB 未实测【推断】 |
| round3 措施：B palette 展开收集（collect 4.5→0.288）+ C packed 直传（native 2.770→2.651） | 12-lighting.md:115-118 | **C 只接了 legacy per-chunk 分流（Mixin:696），域批 blocks25 宽帧未接 packed**——域批 native 段仍付 9.8MB 拷入 + Rust 端已展开 blocks25 全量【实证】 |
| e2e 判据 FAIL 1.072 收尾（C-gate，用户裁决接受现状）；载体噪声下限使 1s 级缺口在该载体不可判 | 12-lighting.md:123-124; .investigations/light-round3-260914-04/review-final-judge.md | 定位：e2e 载体层面已收尾 ≠ native 内部无构成问题；后续评估须换判据载体（:128「e2e 载体更换」列为剩余面）【实证】 |
| 剩余优化面（未实施）：解码-查表单趟融合 ~0.6-0.8ms + A-② sky_fall 融合 ~0.1-0.2ms | 12-lighting.md:128 | 属 per-chunk 内核面候选，域批下同构存在 ×9；「light round4 纯算力项继续冻结」（07 篇 :1565）为现行排序约束【实证】 |
| 形态审计 L5+L6：全量重算 + 无跨 chunk 缓存 = 结构总量比 ≥9×，CP-1 增量化 = 首候选 | 12-lighting.md:141; 07-block-pipeline.md:1548 | 域批未增量化，冗余原样继承；G3 归因 confirmed 后 CP-1 权重升级（07 篇 :1603, :1610）【实证】 |
| CP-2 解粘/.b2：调度形态是 e2e 7% 回退首席候选，但预验证 .b1/.b2/.b3 判据带内否定 → 降后 | 07 篇 :1549, :1612 | native 段是 worker 线程内实测，线程放置不在 native 计时面内【实证】 |
| #107 单车道教训（同步接管阻塞调度器，异步化 6.64× 修复）与 CP-3/CP-1 Mutex 解粘 | 父会话 confirmed 继承；代码侧现状 = 域批异步 hook（Mixin:501-503）+ scratch per-call（mod.rs:14-17） | 已修形态在位；本轮域批路径无锁可证【实证（代码现状）】 |
| GPU 路线三次否决不重开 | .artifacts/perf-rework/gpu-phase1-verdict-260908-01.md（父会话 confirmed 继承，本轮未重开） | 压缩面评估排除 GPU 轴【实证（纪律）】 |
| G3 drift 基底 = legacy 路径轮次级不收敛；域批一次重载收敛不动点（域批无害，收敛性反而优） | 12-lighting.md:158-181 | 约束：任何 native 压缩改造不得破坏域批「整域快照重算」的收敛性质——增量化/缓存类改造直接触碰此面【实证（结论）+ 推断（约束面）】 |
| memset 23.6µs 实测 / nonAir 32.6% / sky15 区间 66.9%（源 K） | light-round3 probe-verdict §2.1 | fill/直落的数据形态参数，可复用于压缩面估算【实证（历史测量）】 |

## 4. 热点候选与测量缺口

**现役打点**只有 fill/native/wb 三段（Mixin:620-633），native 段 4.264ms 是黑盒。缺口分两层：

- **可静态先估（用 §2+§3 历史锚外推，不必先打点）**：
  - 内核 9 中心合计 ≈ 9 × (fill 0.94 + sky_fall 0.30 + sky_seed 0.21 + export 0.16 + block_bfs 0.01) ≈ **~14.6ms**（per-chunk 历史值线性外推；与实测 4.264ms per-center = 38.4ms/任务相比偏低 ~2.6×——差值要么是历史基准机差/数据差，要么是分配+拷贝贡献被低估，**两解释互斥，见 §5**）【推断】。
  - JNI 拷入 9.8MB + b9 重拷 31.5MB ≈ 41MB 纯搬移/任务，10GB/s 带宽下 ~4ms/任务 ≈ 0.45ms per-center【推断，带宽假设未实测】。
  - 分配 ~17MB/任务（A2）缺页/分配器成本未量化【推断】。
- **需门控打点才能拆分（T3 采集）**：
  1. JNI 内分段：拷入 / 内核 / 拷出（jni_bridge.rs:440/444/463 三点计时，env 门控，注意「测量/探针污染铁律」——chunk 级或任务级一次，不放热路径逐格）；
  2. Rust 内核 phase：`light_compute_phased`（mod.rs:284-295）机制现成但**只接了 per-chunk 入口，域批入口 light_compute_domain（mod.rs:335）传 None**——扩到域批即得 9 中心聚合 phase 账，改动极小；
  3. 子窗拷贝（mod.rs:325-331）单独计时；
  4. 分配成本：light_compute_domain/jni_bridge 内 vec 换可复用缓冲（thread_local 化，per-chunk 路 round2 已有先例 jni_bridge.rs:289-311）前后的 A/B——先测后改；
  5. 系统级：worker 线程被调度抢占的贡献（nativeNs 含非 CPU 时间）——FormProbe 线程 id/粘线分布探针（07 篇 :1560 预验证 3 同族）可复用。

## 5. 互斥机制候选预标（供 fan-out 预置，.bN）

「native 段 4.264ms/center（38.4ms/任务）」的构成至少有两个**量级上互斥**的主解释（可并行采证裁断）：

- **.b1 算法量级（9× 域冗余主导）**：81 chunk 体积 fill/传播 ÷ 实机有效吞吐 ≈ 主导项。判据：域批内核 phase 打点（§4-T2）中 fill+sky 合计占比 ≥70% 且与 81×体积模型自洽。若成立 → 压缩面 = 增量化/域共享（CP-1 族，大工程），且受 G3 收敛性质约束（§3 末行）。
- **.b2 JNI 边界 + 内存搬移/分配主导**：9.8MB 拷入 + 41MB 重拷 + 17MB 分配/任务 + pin/copy 语义未定。判据：JNI 内分段打点中非内核段 ≥1.5ms/任务或拷入段超带宽模型 2×。若成立 → 压缩面 = packed 直传接域批（round3 C 的域批化，输入降 ~2 个量级，probe-verdict §2.3）+ 缓冲复用（中工程，不触算法）。
- **.b3（补充轴，非全互斥）测量口径/调度污染**：nativeNs 含 worker 抢占/缺页等待；timedOut=115 低尾行已知进分布（A1 verdict §6.3）。判据：P50 与 trim 后均值差 + 线程粘线探针。成本最低，建议随 .b1/.b2 同轮顺带排除。

竞争判据预登记建议：三候选共用一轮域批分段探针采集（§4-T1/T2/T3 一次采齐），各 .bN 以「占比 ≥ 预登记阈值」判主次，避免主会话自推（core.fanout 规则）。

## 6. 自检声明

- 全部 file:line 为本轮实际读出（Mixin 829 行文件、jni_bridge.rs 270-566、light/mod.rs 全文 709 行）。
- 历史数字均回一手文件（12-lighting.md / probe-verdict-260914-04.md / verdict-260919-01.md）核对行号，未照抄任何转述。
- 无运行时验证动作（scout 只读，subagent 无 shell）；所有耗时外推已标【推断】。
- 本文件为唯一写入产物，未改任何代码/配置。
