# lossless-accel 错误台账（260903-02 立）

> 五段式：现象 → 根因 → 定位 → 修复 → 教训。末尾附「错误→根因」速查表。

## LL1. Rust 侧 MT3 同款 clamp 结构性串行（P0-② 欠账清偿，260903-02）

- **现象**：架构计划 §0 预置核对项——C++ `worldgen_api.cpp:1323` 有 `if (threads > count) threads = count`（MT3，count=1 时池恒 1 worker = 实机 M=1 结构性串行）；Rust 侧是否同款待核对。
- **定位**：grep `WorldgenRust` → `src/api.rs:38` `threads.min(count).max(1)`（env 覆盖分支 L27 同语义）——同款确认。
- **修复**：`api.rs` adaptive_threads 尾行改为 `if count > 1 { threads.min(count).max(1) } else { threads.max(1) }`——count=1 不 clamp，池按请求线程数建 worker 并保持；count>1 语义不变。`cargo check --lib` 绿（仅既有 267 warnings）。
- **状态**：代码级修复完成（candidate）；实机/批量性能影响随 P2a 端到端验证一并确认。C++ 侧同款修复仍是 worldgen-mt-scaling 课题 candidate 待办（本课题不动）。
- **教训**：跨语言移植的池化/调度参数逻辑（clamp/自适应）是同款 bug 高发位——移植核对项应 grep 两侧同语义表达式而非只看函数名。

## LL2. 参照文件 header origin 与内容坐标不符（260903-02，P0-① 探针踩坑）

- **现象**：FULL 归因探针按 header/文件名假设的 origin (-288,-256) 生成对比 chunk → vanilla 配对全 miss（分解计数 0，与同运行 12321 差异块矛盾）。
- **根因**：`vanilla_..._4_-288_-256_FULL.bak.blocks` 的 header origin 字段与实际 chunk 坐标（-18..-15, -16..-13）不符；文件名/注释同被误导。
- **定位**：同运行内恒等式自检（match 差 vs 分解差）矛盾 → python 直读参照文件逐 chunk 坐标（`/tmp .tmp/refkeys.py` 范式）→ 实际坐标曝光。
- **修复**：探针改按参照文件内坐标生成与配对；handle_probe 历史对比因「用文件内坐标生成」本就自洽，未污染。
- **教训**：参照五要素之外，**header 字段本身也可能是错的**——配对/对比永远以文件内容实测坐标为准；探针必须带恒等式自检（本例 0 vs 12321 矛盾 5 分钟暴露假配对）。

## LL3. Emitter 漏调 `_normal_func/_old_blended_func` → NORMAL_PACK 全占位 → 噪声层恒 0（260903-03）

- **现象**：DFC Rust 后端首跑，FINAL 密度与参照差异呈结构性模式——**格点（采样角点）处整对、其余位置恒常数**，噪声细节完全缺失（噪声层输出恒 0）；差异不是随机散布而是「有信号 vs 无信号」的二值形态。
- **根因**：`dfc_gen.py` 新增的 `gen_tables_rs` emitter 只产出了 DF 节点表/spline 表/坐标 fold 表，**漏调了 `_normal_func(idx, …)` / `_old_blended_func(idx, …)`** 这两个「数据收集」调用（C++ 版 `gen_cpu` :1658-1674 在遍历 noise_instances 时同步调它们填充 normal_meta/old_meta，进而生成 NORMAL_PACK/NORMAL_PACK_F/OLD_PACK）。Rust 后端表中 NORMAL_PACK 是按 noise_instances 索引的 `[n, octBase, splitBase]` 三元组——emitter 没调收集函数时这些槽位全为占位 0。运行时 `dfc_backend.rs` 的 normal 噪声求值按 `NORMAL_PACK[b3]=n`（octave 数）展开循环：n=0 → 循环零次 → 该噪声采样恒返回 0。**机制层面：数据表生成器是「遍历副作用填充元数据」的隐式契约，漏掉的是副作用调用而不是任何显式字段**——这是数据驱动架构特有的错误形态（代码路径显式、数据路径靠遍历填充）。
- **定位**：**分层探针（layered probe）**——不直接对 FINAL 全量对拍（那只会得到一个 12.7% 级 mismatch 无从下手），而是按层级隔离输出：先只让 runtime 输出噪声层采样，与参照噪声逐点对比。判据签名：**「格点整对 + 其余恒常数 = 噪声层零输出签名」**——格点处（2D 噪声经 offset 调制的格点缓存语义）恰好常数项对上，其余点暴露噪声项恒 0。签名一确认，即倒查「噪声输入从哪张表来」→ NORMAL_PACK 全 0 → 回查 emitter 遍历代码发现收集调用缺失。
- **修复**：`gen_tables_rs` 遍历 noise_instances 时补调 `self._old_blended_func(idx, params, octBase, splitBase)` / `self._normal_func(idx, params, octBase, splitBase)`（与 C++ `gen_cpu` :1658-1674 同源对齐），使 NORMAL_PACK/NORMAL_PACK_F/OLD_PACK 被真实填充后生成到 `dfc_cpu_tables.rs`（重生成后 NORMAL_PACK 首项 `9, 0, 0, 9, 18, 108, …`，非全 0）。
- **教训**：
  1. **可复用判错签名：「格点整对 + 其余恒常数 = 噪声层零输出签名」**——密度的结构化差异模式本身就是分层定位信息：整段恒 0/恒常数指向「该层输入表为空/占位」，随机散布才指向索引错位或精度。先分层复现签名，再倒查数据源，不要在 FINAL 全量 diff 上硬啃。
  2. 数据表生成器移植时，**逐一核对 C++ 参考实现的遍历副作用**（收集/填充调用），不能只移植「产出的表结构」——生成器的隐式契约（调用了才填充）不在产出物里，只在参考代码的调用序列里。

## LL4. PERM_SIZE 漏 ×256（260903-03）

- **现象**：修复 LL3 后噪声层仍不对/perm 表越界或读串——perm 布局表容量与实际写入量不匹配（背靠背 octave 的写入按 `(octBase + r) * 256 + j` 落位，PERM_SIZE 若按 octave 总数分配则只有 1/256 容量）。
- **根因**：Rust 侧 emitter 计算 perm 表大小时直接用了 **octave 总数**，漏乘每 octave 的 perm 项数 256。C++ 蓝本 `gen_cpu`（dfc_gen.py :1713-1716）的正确口径是 `total_octave` 按实例累计（old_blended 计 40 = 16 lower + 16 upper + 8 interp；normal 计 `2 * len(amplitudes)`）后 **`perm_size = total_octave * 256`**。机制：perm 表是「octave 槽 × 每槽 256 项」的二维布局被压成一维，容量常数是布局维度之积，漏掉第二个维度是典型的一维化容量错误。
- **定位**：对照蓝本逐常数核对——生成 `dfc_cpu_tables.rs` 后直接 grep `PERM_SIZE` 与 C++ `gen_cpu` 的 `perm_size = total_octave * 256`（:1716）对拍，发现缺 ×256。生成器类错误不需要跑 runtime，静态对拍常数即可定位（当前生成产物 PERM_SIZE=356352 = 1392×256，口径正确）。
- **修复**：emitter 的 PERM_SIZE 计算改为与 `gen_cpu` 同式（实例遍历累计 octave 后 ×256），重生成 `dfc_cpu_tables.rs`。
- **教训**：**跨语言移植数据表布局时，所有容量/stride/尺寸常数必须与蓝本逐一对拍**（静态 grep 对拍即可，成本低）——这类错误不报编译错、只报运行期数据串位，运行期定位代价远高于静态核对。可复用判据：**「一维化 packed 表的 SIZE 常数 = 各布局维度之积，移植时逐维度核对」**。

## LL5. `collect_perm` normals 索引错位（260903-03）

- **现象**：前两处修复后 perm 表仍有错——normal 实例的 perm 段内容与实际采样器不对应，噪声输出系统性错乱（值非 0 但不对，与 LL3 的恒 0 签名可区分）。
- **根因**：`dfc_backend.rs::collect_perm` 回填 normal 实例 perm 时，**用 noise_instances 的实例序号 `i` 直接索引 `normals[i]`**。但 runtime 的 `normals: Vec<DoublePerlinNoiseSampler>` **只收集了 normal 类实例**（`new()` :96-102 只对 NORMAL_INIT 循环 push），而 NORMAL_PACK 是**按全量 noise_instances 索引**的（含 shift/old 占位槽，如首段 25 个 normal 后跟一排 `[0,0,0]` 占位）——`i` 是全量表序号，`normals` 是压缩后向量，两者错位：第 i 个全量 normal 实例实际是 `normals[k]`（k = 它在 normal 实例中的序号）。机制：**「全量索引表 vs 压缩实例向量」两套编号混用**——表按全量序，Vec 按过滤后序。
- **定位**：源码核对 + 语义注释修复现场（dfc_backend.rs :145-152 现行代码）：`let mut k = 0; for i in 0..NORMAL_INSTANCES { let n = NORMAL_PACK[i*3]…; let noise = &self.normals[k]; … k += 1; }`——normal 实例（n≠0）按遇到顺序取 `normals[k]` 并递增 k，跳过占位槽不消费索引。错位模式在 LL3 修复引入真实 NORMAL_PACK 后才暴露（此前全 0 表走不到 normal 分支），三 bug 呈串联暴露链。
- **修复**：`collect_perm` 的 normal 段改用独立序号 `k` 顺序对应 `normals[k]`（只对 n≠0 的实例递增），与「normals Vec 只含 normal 实例、按序对应非 0 NORMAL_PACK 项」的压缩约定对齐。
- **教训**：**生成数据表（全量索引、含占位槽）与运行时容器（按类过滤压缩）之间必须显式声明索引映射约定**，回填/查表代码写之前先问「这个下标是全量序还是过滤后序」。可复用判据：**「packed 表带占位槽 + runtime 容器按类过滤 ⇒ 一切下标换算需经过过滤映射，禁止直接同下标互查」**。另注：LL3→LL4→LL5 是串联暴露链（上一 bug 修复才让下一 bug 的症状显形）——多 bug 串联时每修一个都要重跑分层签名，不能假设「一次全量对拍通过 = 无残留 bug」。

## LL6. 预注册判据「0 diff」未达（rounded6 96.08%）——判据形式违规与口径裁定（260903-04）

- **现象**：架构计划 260903-04 W4 预注册「GPU 角点 vs DFC-CPU oracle `{:.6}` 舍入内 0 diff」；实测 rounded6 仅 96.08%（6144 点 5903 对），max_diff=9.18e-6。
- **根因**：预注册判据把「f32 ULP 级微差在 6 位舍入边界的翻转」误写成「0 diff」——f32 vs f32 的 ULP 差（~1e-6 量级）落在 `{:.6}` 舍入边界上是必然事件，判据本身口径错位，非实现错误。
- **定位**：tri-cut3 重编后双 seed 23 点 major_diff(>1e-4)=0 + max_diff 9.18e-6 与已知 f32 ULP 量级吻合 → 差异全部为精度级，无语义级。
- **裁定**：按计划「f32 口径既定判定规则」（p2a-design §3：舍入边界翻转回数据层取证，不得静默放宽）——本次以 major_diff(>1e-4)=0 为主判据、max_diff 量级为辅证，rounded6 降为参考指标。**教训：预注册判据必须与数值精度口径自洽——f32 对拍写「0 diff」前先算 ULP 量级落在舍入边界的期望翻转率。**
- **状态**：candidate（judge ④-2 指出的形式违规，本条即补录）。

## LL7. final_density.spv 陈旧产物（D23 修复前语义）——生成器多产物部分更新失配（260903-04）

- **现象**：GPU 引擎 vs DFC-CPU oracle 系统性 diff（6144 点 f32_exact 43.26%、max_diff 0.5533）；tri-cut 证明 C++ CPU 与 GPU 自身 major diff。已知值哨兵点 (784,160,-408)（历史 seed）GPU 输出 0.0453032888 = 时间线 L1386 记录的 D23 修复**前**错误值。
- **根因**：spv 编译于 D23 提交（cc58e05 08-15 19:21）前 5 小时，9de661e 提交了修复前 spv（git blob 取证：.bak-pre-d23 与 9de661e 提交 blob 哈希逐字节相同）；08-23 comp/cpu_backend.h 同批重生成但 spv 未重编——生成器多产物（comp/cpu_backend.h/spv）部分更新，glslc 编译步骤脱节。
- **定位**：双 seed 切分（历史 seed 正 x 已验证点复现历史错值）+ mtime/提交时间 5 小时窗交叉；重编（gen_final_density.py → glslc → 部署 .bak-pre-d23 备份）后双 seed 23 点 major=0、6144 点 max_diff=9.18e-6——闭环。
- **修复**：重编部署；判据固化（多产物原子更新 + 哨兵结论配已知值哨兵点）→ knowledge/discovered/build-tooling.md 发现 #12。
- **教训**：二进制产物无法从内容/新鲜时间戳判断新旧；「逐位一致」只证明一致域内语义相同，域外产物可能陈旧。
- **状态**：candidate（judge 三重独立证据 PASS）。

## LL10. 探针 fill_n=8 口径错 + 0.61× 单 shot 测量伪影（260903-08，P-C2 复测）

- **现象**：260903-04 gpu_corner_probe.rs 实测「串行 5.0µs/pt，双线程 0.61×」；复测（gpu_mt_wall_retest.rs，n=8 原实参口径 + n=6144 全批对照，5 轮 S/P 交替中位数）两口径均 ≈1.0×（1.006×/0.989×），0.61× 未复现。且派生指标「5.0µs/pt」口径错误——`fill_n=8` 实为**每次 fill 8 个点**，注释「8 chunk 批量」与派生分母（按 6144 点算）错位，真实量级 ≈3.8ms/pt。
- **根因**：① 派生指标的分子（实测耗时）与分母（假定点数）来自两处——实参 `fill_n=8` 与注释「8 chunk 批量」脱节，µs/pt 沿用了注释口径的错误分母；② 0.61× 来自**单 shot、无轮次、无中位数、时钟/调度状态未控**的测量，落在正常轮间波动（0.964-1.065×）分布之外，是测量伪影而非机制（Mutex 真串行化成立：sync-check mismatch=0/6144 + 静态锁/fence + 双线程 wall ≈1.0× 三方自洽）。
- **定位**：① 复测设计核对实参时暴露——读探针源码发现 fill_n=8 是 fill 实参而注释宣称批量语义；② 0.61× 按 route2-260903-05 judge 两步走处置：无探针整批 wall + 计数断言（total=412 结构自洽：每口径 6 预热 + 5×(20+20) ×2）+ 多轮中位，原值落分布外即定性。
- **修复**：复测探针双口径设计 + 计数断言 + 中位数主判据；260903-04 [fact2] 0.61× 按 §15.4 标注被本复测候选取代（按「测量伪影」结案，不引入新机制假设）。派生 µs/pt 作废（历史 txt 不改，错误在此记录）。
- **教训**：① **探针派生指标的分子分母要与实参口径核对，注释不算数**；② **单 shot 数字只出候选不出结论，整批 wall + 计数 + 多轮中位才可判**——反常比值（0.61×）先怀疑测量侧，复测未复现即按伪影结案，不为它发明新机制。
- **状态**：candidate（judge 建议通过，C1 计数构成补注已清偿）。

## LL11. Rust 2021 闭包字段级捕获裸指针 → `*mut c_void cannot be sent between threads safely`（260903-08）

- **现象**：`move` 闭包体内写 `hp.0`（H 包装体的裸指针字段），即便 H 已 `unsafe impl Send`，编译器仍报「`*mut c_void` cannot be sent between threads safely」。
- **根因**：**Rust 2021 edition 闭包按字段离散捕获**——只引用 `hp.0` 时捕获的是字段 `*mut c_void` 本身而非整个 H；Send 实现挂在 H 上，裸指针不继承。同段代码跨 edition 语义静默变化（2021 前捕获整变量）。
- **定位**：判错签名 =「已 unsafe impl Send 还报裸指针 Send 错」→ 查闭包体内对包装体的字段访问（hp.0 触发字段级捕获）。
- **修复**：普通 helper 函数按值传包装体（`fn work(h: H, ...)`，H 整体按值移动 → Send 生效）。
- **教训**：跨线程闭包不直接解引用包装体取裸指针字段；把裸指针获取移进被调函数内部，线程边界只传 Send 包装体。
- **状态**：candidate。

## 速查表

| 错误 | 根因 |
|---|---|
| LL1 Rust MT3 串行 | `threads.min(count)` 在 count=1 时把池 clamp 到 1 worker（C++ 同款移植） |
| LL2 归因探针全 miss | 参照 header origin 与内容坐标不符；配对用了硬编码坐标而非文件内坐标 |
| LL3 噪声层恒 0 | emitter 漏调 `_normal_func/_old_blended_func` → NORMAL_PACK 全占位 0 → octave 循环零次；「格点整对+其余恒常数 = 噪声层零输出签名」 |
| LL4 PERM_SIZE 错 | 容量漏 ×256（一维化 packed 表 SIZE=各维度之积，与 gen_cpu :1716 口径脱钩） |
| LL5 collect_perm 错位 | 全量索引表（NORMAL_PACK 含占位槽）vs 压缩 normals Vec 直接同下标互查，缺过滤映射（k 序号） |
| LL6 rounded6 96.08% 未达「0 diff」 | 预注册判据与 f32 ULP 口径错位（ULP 差落 6 位舍入边界必然翻转）；主判据改 major_diff(>1e-4)=0 |
| LL7 spv 陈旧产物 | 多产物部分更新失配（spv 编译早于 D23 提交 5h，commit 9de661e 提交修复前 spv；重编即愈）→ build-tooling #12 |



| LL8 三方对拍假残差 | 跨探针对比未钉死同列同坐标（GPU z=0 vs macro z=16 误作同点） |
| LL9 transpiler ch0 线性化 | NoiseSet 漏设 blended_noise（坑记在诊断注释侧未吸收进生产构造）；.bA 静态配平归因误判 |
| LL10 fill_n=8 口径错 + 0.61× | ① 派生指标分母沿用注释口径（8 点/次非 6144）未与实参核对；② 单 shot 无轮次测量出测量伪影——「派生指标分子分母对实参核对 + 整批 wall+计数+多轮中位」 |
| LL11 闭包裸指针 Send 报错 | Rust 2021 字段级捕获：闭包写 hp.0 捕获 *mut c_void 而非 Send 包装体 H；修法=helper 函数按值传包装体 |
