# S6 欠账清偿草稿——DFC Rust 后端三 bug（260903-03，subagent 产出，待主会话应用）

> 载体：追加到 `.investigations/lossless-accel/lossless-accel-errors.md`（LL3/LL4/LL5，接续现有 LL1/LL2 编号与五段式格式）；速查表补 3 行。
> 证据源：p2a-design-260903-03.md §0 末行、cmd-output/dfc-probe-260903-03.txt、cmd-output/dfc-verify-260903-03/04.txt、dfc_gen.py（gen_tables_rs / _normal_func / _old_blended_func / gen_cpu :1651,1713-1716）、WorldgenRust/src/dfc_backend.rs（:76-160, :258-262）、generated/dfc_cpu_tables.rs（PERM_SIZE=356352, NORMAL_PACK/NORMAL_PACK_F）。

---

## LL3. Emitter 漏调 `_normal_func/_old_blended_func` → NORMAL_PACK 全占位 → 噪声层恒 0（260903-03）

- **现象**：DFC Rust 后端首跑，FINAL 密度与参照差异呈结构性模式——**格点（采样角点）处整对、其余位置恒常数**，噪声细节完全缺失（噪声层输出恒 0）；差异不是随机散布而是「有信号 vs 无信号」的二值形态。
- **根因**：`dfc_gen.py` 新增的 `gen_tables_rs` emitter 只产出了 DF 节点表/spline 表/坐标 fold 表，**漏调了 `_normal_func(idx, …)` / `_old_blended_func(idx, …)`** 这两个「数据收集」调用（C++ 版 `gen_cpu` :1658-1674 在遍历 noise_instances 时同步调它们填充 normal_meta/old_meta，进而生成 NORMAL_PACK/NORMAL_PACK_F/OLD_PACK）。Rust 后端表中 NORMAL_PACK 是按 noise_instances 索引的 `[n, octBase, splitBase]` 三元组——emitter 没调收集函数时这些槽位全为占位 0。运行时 `dfc_backend.rs` 的 normal 噪声求值按 `NORMAL_PACK[b3]=n`（octave 数）展开循环：n=0 → 循环零次 → 该噪声采样恒返回 0。**机制层面：数据表生成器是「遍历副作用填充元数据」的隐式契约，漏掉的是副作用调用而不是任何显式字段**——这是数据驱动架构特有的错误形态（代码路径显式、数据路径靠遍历填充）。
- **定位**：**分层探针（layered probe）**——不直接对 FINAL 全量对拍（那只会得到一个 12.7% 级 mismatch 无从下手），而是按层级隔离输出：先只让 runtime 输出噪声层采样（`dbg_normal_raw` 类 hook），与参照噪声逐点对比。判据签名：**「格点整对 + 其余恒常数 = 噪声层零输出签名」**——格点处（2D 噪声经 offset 调制的格点缓存语义）恰好常数项对上，其余点暴露噪声项恒 0。签名一确认，即倒查「噪声输入从哪张表来」→ NORMAL_PACK 全 0 → 回查 emitter 遍历代码发现收集调用缺失。
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

---

## 速查表补行（追加到现有表末尾）

| 错误 | 根因 |
|---|---|
| LL3 噪声层恒 0 | emitter 漏调 `_normal_func/_old_blended_func` → NORMAL_PACK 全占位 0 → octave 循环零次；「格点整对+其余恒常数 = 噪声层零输出签名」 |
| LL4 PERM_SIZE 错 | 容量漏 ×256（一维化 packed 表 SIZE=各维度之积，与 gen_cpu :1716 口径脱钩） |
| LL5 collect_perm 错位 | 全量索引表（NORMAL_PACK 含占位槽）vs 压缩 normals Vec 直接同下标互查，缺过滤映射（k 序号） |
