# 候选 b1：cave_air 由 chunk 管线某阶段在 ReadWorldProbe 读取之后才写入（时序/异步写入类）

status: draft（worker b1 分析产物，待 judge / 主会话收敛）

## 结论倾向

**倾向推翻（b1 最多解释 4 条观察中的 2 条半，且撞上致命反证：reconfirm）**。
核心理由：ReadWorldProbe 用 `world.getChunk(wx, wz, ChunkStatus.FULL, true)`（ReadWorldProbe.java:55），1.20.1 chunk status 管线 NOISE→SURFACE→CARVERS→FEATURES→…→FULL 中 **FULL 是链尾，getChunk(FULL) 同步阻塞到全部前序阶段完成才返回**——不存在「chunk 已是 FULL 但 carvers 还没跑」的读取窗口。cave_air 的写入者（nether carver，NetherCaveCarver 填 cave_air）在 CARVERS 阶段，**先于** FULL，因此先于 probe 读取。

## 证据链

1. **Mixin 侧（NoiseChunkGeneratorMixin.java:41-78）**：populateNoise @HEAD cancel 后 `completedFuture(chunk)` 立即返回——这只是把 NOISE 阶段替换为同步 C++ 产物，**不影响后续 status 推进**。vanilla 的 ChunkStatus 依赖链（ThreadedAnvilChunkStorage / ChunkStatusgetAllBefore）保证调用方 `getChunk(..., FULL, true)` 返回时 CARVERS/FEATURES/INITS/LIGHT 全部已 apply。HEAD cancel 不会让 chunk「提前 FULL」——status 状态机的推进者仍是 vanilla 的 chunk 加载器，不是 populateNoise 的返回值。
2. **cave_air 来源**：nether 洞穴 carver 在 CARVERS 阶段以 cave_air 填充；差异簇（chunk(203,200) y70-72，cave 数量 4/23/53 随深度递增）形态与洞穴剖面一致。C++ dll（cppReplace）只接管 NOISE+SURFACE，CARVERS 仍为 vanilla → cave_air 属于管线内合法产物，不是「管线外延迟写入」。
3. **BlockProbe.java:459-470 历史教训**：carver 遍历 17×17 邻域，邻域未生成则**静默跳过**（不是延迟写入）。这解释的是「cave_air 缺失（被跳过）」而非「cave_air 晚到」——历史教训支持的是 b1 的**反面机制**（carver 边界跳过类，如候选时序相反方向：vanilla 参照导出时邻域不完整 → 参照缺 cave_air，world 侧全邻域生成 → world 有 cave_air，即 air→cave_air 方向）。
4. **readWorldProbe 与 cppReplace 同跑（gen2）**：probe 是 server 启动后对同一 world 的第二次 getChunk(FULL)——此时 chunk 早已落盘/在内存 FULL，读取是纯快照，无管线在途。

## 能解释的观察

- **观察 2（gen2 内存无 cave_air）**：勉强可解释为「读取早于写入」——但需要违反上述 FULL 屏障才能成立，实际是**被证据 1 否定**的伪解释。
- **观察 1（gen1 内存有 cave_air）/ 观察 3（gen2 存档有）**：在 b1 框架下只能靠「gen1 的 probe 恰好读得晚」这种**无机制的运气论**，且 gen1/gen2 用同一份代码同参数（facts 文件第 8 行），无理由产生不同时序。

## 不能解释的观察（b1 致命伤）

- **观察 4（reconfirm 读盘无 cave_air）+ 矛盾点 3「同文件两读不同」**：b1 是**写入时序相对读取**的理论。gen2 存档 MCA 里已经物理存在 cave_air（独立 MCA 解析确认 (3263,70,3211)=cave_air，facts 第 15 行）——文件内容不会因任何「写入晚于读取」而改变。reconfirm 若读到这份 MCA，必然报 ~104 差；它只报 1 差（quartz→gold），说明 **reconfirm 读取时呈现的内存状态不含 cave_air**。要让 b1 兼容这一点，必须额外假设「加载时 cave_air 被移除」——那已是另一个机制（load 侧改写，如 chunk 加载后 tick/light/block-update/再生成），不是时序写入。**结论：b1 无法解释观察 4，也无法解释观察 3↔4 的同文件两读矛盾。**
- 顺带：b1 同样无力解释 seed B（gen 内存 = 存档 MCA 精确同值）为什么在正常 case 下时序问题消失。

## 建议主会话执行的 1-2 个决定性廉价探针

1. **MCA 前后双拷贝对比（最便宜、直接裁决矛盾点 3）**：reconfirm 启动前先拷贝 r.6.6.mca → r.6.6.mca.before；reconfirm 停服后再跑一次 compare_save_region.py/dump_chunk_203_200.py 解「after」版 MCA。若 after 版 cave_air 从 103 → 0：证明 cave_air 在 reconfirm 的 load→save 周期中被移除（指向 load 侧改写/再生成类候选）；若仍是 103：说明 compare 工具或 reconfirm 读取路径有选择性读取问题。
2. **reconfirm 侧注入坐标打印（一行级改动）**：ReadWorldProbe 对 chunk(203,200) 读取前打印 `(3263,70,3211)` 的 blockstate（或对 mismatch 行放开 shown 上限到 cave 簇）。同时核对 reconfirm 日志里是否出现 `[Mixin] populateNoise(nether) intercepted chunk(203,200)`——若出现，说明 reconfirm 服务器**重新生成**了该 chunk（没从盘加载），「读盘」前提本身被打破，矛盾整体换轨到「world 目录/gen 条件」类候选。
