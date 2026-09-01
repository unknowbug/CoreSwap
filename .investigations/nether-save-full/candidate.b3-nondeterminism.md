# 候选 b3：Rust nether 生成 / Java feature 阶段跨运行非确定（M4 家族）

**status: draft**（worker b3 候选产物，2026-09-04，静态审查 + 矛盾现场推演；无 shell，全部证据来自工作区文件）

## 结论倾向：**弱不支持（倾向于推翻「Rust 生成非确定」子命题；Java 侧仍-running 的 vanilla CARVER/FEATURE 阶段 + probe 读取时机竞态是更优解释）**

核心判断：b3 作为「Rust nether 生成器本身跨运行非确定」**缺少活的机制**——静态审查未找到现存顺序/线程敏感路径，且 Rust 全代码 **零次写入 cave_air**（grep cave_air 无命中），cave_air 类差异根本不可能源自 Rust 生成。b3 唯一能救活的形态是「probe 读取时机 vs vanilla 尾随阶段写入」的竞态（见下），但那是**读取时序问题，不是生成非确定**。

## 证据链

### A. 静态审查：Rust nether 路径无现存跨运行非确定机制
1. **多线程分区确定**（`WorldgenRust/src/api.rs` L98-127）：`wg_fill_blocks_multi` 按 `threads` 个 scoped 线程交错分片（i += nthreads），每 chunk 只被一个线程写一次固定 out，与完成顺序无关；历史 `mt_probe.rs`/`mt_fill.rs`（bin-diag）就是「多线程 vs 串行 mismatch=0」的验证载体。
2. **随机派生全部按 key/坐标定值、无顺序消费链**：
   - noise sampler 惰性创建 = `random_deriver.split_str(key)`（density_builder.rs L152-179）——splitter 按噪声 key 派生，创建顺序不影响结果；
   - carver：`set_carver_seed(seed+l, cx2, cz2)`（worldgen_handle.rs L478/L508）；
   - features：`set_population_seed(seed, cx*16, cz*16)` → `set_decorator_seed(population_seed, p, k)`（worldgen_handle.rs L588-605）——逐 chunk 封闭，不依赖其他 chunk 先后。
3. **历史非确定 bug 已修**：`biome.rs` L182-185 注释——原 `HashMap` 的 `all_features_lists` 迭代序每进程随机 → PlacedFeatureIndexer 编号随机 → nether features 运行间不确定（2026-08-30，2796 块差）；现 `BTreeMap` 按键序确定。M17 dll（C5AC5309）在此修复之后。
4. **残余 HashMap 排查**：feature 路径其余 HashMap 均为查找用或按下标写入（feature_loader.rs L95 `all_features[gidx]=fid` 写入按下标、序无关）；`density_builder.rs` L193 lazy_refs 填充按 key 匹配、序无关。**未发现第二个活体顺序敏感点**。

### B. cave_air 归属：Rust 不写 cave_air
- `grep cave_air` 全 `WorldgenRust/`（含 src/bin）**零命中**；Rust 块输出只有 `blocks.json` 注册表里的 id。
- mixin 只拦截 NOISE（populateNoise）+ SURFACE（buildSurface）两阶段（`NoiseChunkGeneratorMixin.java`）——**Java vanilla 的 CARVERS / FEATURES / lighting 等后续阶段照常在 Rust 输出上跑**。
- ⇒ gen1 内存 / gen2 存档里的 cave_air 只能是 Java 侧尾随阶段（vanilla nether cave carver 最可疑）或读/比较工具链写入/翻译的。

### C. 矛盾现场的时序模型（竞态假说）
流程「cppReplace + readWorldProbe **同跑**」：若 readWorldProbe 在某些 chunk 的 CARVERS 阶段完成前读取，读到的是「carver 前」状态（无 cave_air）；存档/晚读捕获「carver 后」状态（有 cave_air）。
- gen1（20:09）内存 131 差含 130 cave_air → 读得晚，carver 已落；
- gen2（20:13）内存 1 差无 cave_air → 读得早，carver 未落；
- gen1 内存 ≈ gen2 存档（现场观察 #4）——恰好是「同一最终状态的两个晚读」；
- 1×quartz→gold 四处读全部一致（gen1/gen2内存/gen2存档/reconfirm）→ 稳定确定性残差（独立对齐 bug，与 b3 无关）。

### D. gen2 存档 vs reconfirm（同文件两读不同）——工具/文件新鲜度污染嫌疑
- gen2 存档的 103 cave_air **与 gen1 的簇完全同位置**（chunk(203,200) y70-71）；reconfirm 从盘读同一 world 却无 cave_air。
- 最省假设：compare_save_region.py 解的 r.6.6.mca 是 **gen1 运行的落盘**（world 未在 gen2 前彻底清理 / compare 与 reconfirm 读的不是同一代文件），或 MCA palette 解析 bug。这不是生成非确定，是**对比载体污染**——b3 完全解释不了这条，反而被它削弱。

## 能解释的观察
- **gen1 内存(131) vs gen2 内存(1)**：可用「probe 读取时机竞态」解释（carver 尾随阶段落块早晚），也可用 M4 家族微漂移解释——但静态审查找不到 Rust 侧活机制，竞态更省。（M16 微漂移前科 11 块仍是 M4 存在性的旁证，但量级 11 vs 130 且当时 dll 版本不同，传导力弱。）
- **gen2 内存(1) vs gen2 存档(104)**：可用竞态解释——存档是全部阶段完成后的终态，内存 probe 是中途读。这是任务书允许的 loophole，且本例有明确的落块者（Java carver）。

## 不能解释的观察
- **gen2 存档(104, 有 cave_air) vs reconfirm 读同一盘(1, 无 cave_air)**：b3 任何形态（生成非确定或读取时机）都解释不了同一 MCA 两次读取不同——这指向 compare_save_region.py 解析 bug 或 mca 文件代际错配（gen1 的盘被当 gen2 的存档比较）。此条是 b3 的最强反证。
- b3 无法解释 cave_air 的**写入者**（Rust 零写入，Java vanilla 尾随阶段才是候选）——即 b3 把「写入者非确定」安错了头。

## 建议主会话执行的决定性廉价探针（1-2 个）
1. **双时刻内存读（竞态判定，最便宜）**：重跑 gen2（清 world），① gen 完成后立即 readWorldProbe 数 cave_air；② server stop / 全部 chunk status 走完后二次 readWorldProbe 再数。若 ②出现 ~103 cave_air 且 ①没有 → 竞态成立，b3「生成非确定」推翻；若两次都有/都无 → 回到 M4 家族继续查。（顺带 `WG_FEATURELOG=1` 看 chunk(203,200) 的 [FEATURE] 行数两次是否一致，直接验证 Rust 侧确定性。）
2. **MCA 新鲜度 + 解析一致性（污染判定）**：gen 前后对 `r.6.6.mca` 记 sha256 + mtime；用 compare_save_region.py 与独立解析（dump_chunk_203_200.py）对**同一份字节**各解一次 chunk(203,200)——两者不一致 = 解析 bug；一致且文件是 gen2 落盘 → 存档确有 cave_air（此时与 reconfirm 的矛盾转向 ReadWorldProbe 读盘路径）。

## 附注
- 「Rust 写 cave_air 本身是 bug」子问题：不成立——Rust 根本不写 cave_air；若最终确认 Java carver 在 Rust 输出上写出 cave_air 而 vanilla 参照同位置是 air，则疑点转为「carver 输入块状态差」或「vanilla 参照导出时机」，需单独立候选。
- seed B（gen=save 精确同值 96.7478%）与本模型自洽：seed B 那次读取时机晚/无竞态。

---

# 前提修正后的再评估（2026-09-04 追加；不改原结论正文）

## 前提修正内容
新数据层证据：所有 gen run 的 **CppBridge 均 enabled=false**——`cppWorldgenDir` 传错一层（传了 `.tmp-coreswap-data\worldgen\data`，正确为 `.tmp-coreswap-data\worldgen` 这层，即 wg_dir=含 `data/` 的目录）；ctypes 直连 `wg_create` 已复现：传错层返回 0（NULL），传对层返回非 0。
⇒ **gen1/gen2 世界全是 vanilla 全管线生成，从未经过 Rust nether 代码路径**。本文正文的 §A-§C（Rust 侧静态审查结论仍然成立，但与本次实验无因果）；§B「cave_air 写入者」自动收敛为 **vanilla 自己的 CARVER 阶段**（无 Rust 参与的歧义）。

## 再评估：vanilla-vs-vanilla（参照 vs 存档）的 gen1 131 差 vs gen2 1 差

关键逻辑反转：**CppBridge=false 时，gen run 与 vanilla 参照在语义上应逐位相同**（同 seed 同算法同数据）。而实测仍有 1 差（gen2）和 131 差（gen1），且跨运行不一致——

1. **这几乎排除「生成器非确定」解释，强烈指向观察层伪影**。1.20.1 vanilla nether worldgen 是逐 chunk 纯函数式的（噪声/表面/carver/feature 种子全部按 (worldSeed, chunkPos[, key]) 派生，无跨 chunk 顺序消费链；feature 跨 chunk 蔓延由 chunk status 依赖序约束）。Java 侧能想到的顺序/线程敏感点（ServerChunkManager worker 线程调度、decoration 邻 chunk 状态等待）在 1.20.1 均不改变最终块结果——只改变**完成时刻**。
2. 因此 gen1(131，含 130 cave_air) vs gen2(1，无 cave_air) 的差异，最优解释仍是**读取时机落在生成进度线的不同位置**（readWorldProbe 同跑时部分 chunk 的 CARVERS/FEATURES 尚未完成；vanilla nether carver 正是 cave_air 的合法写入者）——同一次运行内部读两次也会不同，与「跨运行」表象无关。本文正文 §C 的时序模型原样成立，只是写入者从「Java 尾随阶段（歧义）」明确为「vanilla carver」。
3. quartz→gold 1 差在全 vanilla 下更可疑：vanilla vs vanilla 不该有此差 → 另一半嫌疑落在**比较载体**上（参照 blocks 导出与本次 world 的差异，如参照导出时机/版本/region 清理残留），与 §D 的 mca 新鲜度污染嫌疑同族。**建议主会话把「参照导出与 gen run 的载体同源性核查」（导出日志 seed/时间/mtime 比对）列为前置核查项**。
4. b3 的 M4 家族定位（跨运行确定性风险）在本次实验中**不成立也不能升级**——观察到的跨运行差异没有一条需要非确定生成来解释；候选应保持推翻倾向，降级为「风险登记」而非「现象解释」。

## Rust 侧真正接管后的同类风险清单（静态审查外推，非本次实验证据）

基于正文 §A 的审查，接管后若出现同形态「gen1≠gen2」差异，按风险排序：
1. **HashMap 迭代序进种子链**（已发生一次：2026-08-30 biome.rs all_features_lists，2796 块差）——代码库对该反模式已有一次前科，任何新代码把 HashMap 的 `.values()/.iter()` 结果喂进 PlacedFeatureIndexer/seed 派生即复发。防御判据：**凡进入随机种子派生或索引编号的集合遍历，必须 BTreeMap/Vec 显式排序**。
2. **`std::fs::read_dir` 顺序**（biome.rs load_features 读 biome/*.json）——当前经 BTreeMap 中转无害；若未来有人绕过 BTreeMap 直接消费 read_dir 序（如按文件顺序建 featureIndex），就变成机器间/运行间不确定点。
3. **thread_local 缓存跨界污染**（surface_rules.rs L60-63 NOISE_THRESH_CACHE 按 (noise_key, col_key) 键控）——键含坐标即安全；若未来某缓存键漏掉坐标/seed 成分，跨 chunk 复用即产生「取决于哪个线程先算过哪列」的差异（多线程下非确定）。
4. **多线程本身不是风险**（api.rs 分片 + 每 chunk 单写者 + mt_probe/mt_fill mismatch=0 历史验证），前提是保持「chunk → 固定线程 → 固定 out」不变；若改成动态任务队列 + chunk 间有数据依赖（如 feature 跨 chunk 蔓延写入邻居），会引入真顺序敏感。
5. **观察层风险与 vanilla 同族且更大**：接管后 probe 读取时机（CppBridge fill 与 Java 尾随 carver/feature 的相对时刻）同样是竞态来源——本次实验恰恰证明观察层就能独立制造「跨运行差异」假象，Rust 接管后对比实验必须先排除观察层（双时刻读 + 载体新鲜度核查）再指控生成器。

**修正后结论倾向：推翻 b3 作为本次矛盾的解释；M4 家族（Rust 侧）保持为「接管后风险登记」，候选保持 draft，待双时刻读 + 载体同源性两探针落定后由主会话裁决取代或归档。**
