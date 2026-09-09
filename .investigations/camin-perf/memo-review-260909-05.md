# 260909-05 memo 立项评审记录（中间产物，主会话维护）

## Step 1 — #101 三证廉价独立验证（2026-09-09 ~19:25，HEAD 70c38d8）

- 执行体：cargo build --offline -p WorldgenRust（Finished 0.25s 缓存，rlib=deps/libWorldgenRust-96a33b096de64ab8.rlib mtime 18:15:44 > newest src worldgen_handle.rs 18:14:56，#30 判据过）；rustc 单编 .tmp/camin_bench_26090905.exe。
- 载体：camin_bench 16×16=256 chunks，seed -8248318472910187742，wg=versions/1.21.6/data/worldgen，origin(0,0)，串行。
- 结果：
  - on（默认）：166.89ms/chunk hash=6908dbfc9a79c40a（上轮 177.9ms 同 hash）
  - off（WG_CA_MIN=0）：127.60ms/chunk hash=115641b86711d9cd（上轮 131.8ms 同 hash）
  - 残差 +30.8%（上轮 +35%，run 级波动量级内）
- 判定：hash 双臂逐位同 = 同执行体同行为面 → 上轮 E4b/E4c 读数（853k 读/chunk、114× 放大、seagrass≈53%）与 #101 三证**继承合法**，复核通过。

## Step 2 — 写侧判别实验（进行中）

- 目标未知量：越界读的列级重复分布（memo 命中潜力上界）+ 越界写频率（失效压力）。
- 手段：worldgen_handle.rs block_at_col 越界分支加门控计数（WG_CA_MEMODIAG）：total reads / unique cols / per-col 读直方图（sum、sum²、top-k）/ pending_cross 写计数，chunk 级输出。

## Step 2 — 写侧判别实验（完成，2026-09-09 ~19:45）

- 改动：worldgen_handle.rs 加 WG_CA_MEMODIAG 门控诊断（thread_local 列级/点级读直方图，chunk 级输出，诊断关闭时行为 hash 恒等已验 6908dbfc）。
- 载体：同 Step 1（16×16，on 臂，诊断开启 184.1ms/chunk——含 hashmap 统计开销，仅用于分布不用于计时）。
- 读数（257 chunks 含 warmup）：
  - 越界读合计 216,826,945 → **≈843k/chunk**（与 E4b 853k 吻合，sanity ✅）
  - 唯一列合计 73,922 → **≈288 列/chunk**；列级重复率 ≈100%
  - 点级 (x,y,z) 重复率 **avg 98%**（唯一点仅 ~2%）
  - 列读桶（合计）：1 次=1343 / 2-10=46827 / 11-100=22593 / **>100=3159**（热点列 ≈12 条/chunk，top 列 ~550-574 读/chunk）
  - **pending_writes 全程 = 0**（16×16 origin(0,0) region 无跨 chunk 越界写）
- 判定：
  1. **#101「单读 51ns 近下限」前提被部分证伪**——51ns = mutex lock + hash + Arc + 数组读的同步结构成本；98% 重复读每次重付该成本。
  2. 写失效压力在该口径为零；量级上 pending 写（placed 级）vs 读（843k）差 5 个量级，失效频率天然不构成主导风险。
  3. 读热点集中（12 列 >100 读），「消每读同步开销」与「读量语义固有」是两层——#101 混层。
- **§9.7 口径声明（judge 条件 2）**：① 本组读数为 bench 串行 region 口径（seed -8248318472910187742，16×16 origin(0,0)），与存档 dump 口径不可比；② 257 chunks 含 warmup chunk(0,0) 双计（分母偏差 ~0.4%，不动摇结论）；③ **pending_writes=0 为单 region 单样本**，仅支持「该口径下失效压力为零」，外推到其他区域/维度需复核；④ 51ns 单读价为残差÷读数的摊销推算值，可能含 miss 重生成摊入，不得当独立实测单价引用。
- 结论：判定树分叉实锤 → fan-out 三候选（.b1 点级 memo / .b2 不立项维持 #101 / .b3 结构去同步预取）。

## Step 3 — fan-out（进行中）

- worker B1 (ea4e0912) / B2 (fca8141c) / B3 (e4f32d9b) 并行，产物 .artifacts/camin-perf/memo-decision-26090905.b{1,2,3}.yaml

