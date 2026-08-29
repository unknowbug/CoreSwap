# Rust worldgen 功能实现：错误与根因清单（F 系列）

> 载体：`.investigations/rust-mod-load/functional-errors.md`（错误台账，独立成篇，draft 草稿）。
> 本文件按「现象 → 根因 → 定位 → 修复 → 教训」五段式记录「Rust worldgen **整体功能实现**」里程碑（apply_features 补 OCEAN_FLOOR_WG 高度图 / ore_vein 矿脉接入 / Beardifier 接入 / wg_fill_density 实现 / 生成路径锁清理）过程中的错误。
> 背景：Rust 块级管线已跑通（`WorldgenRust/`，见 `rust-mod-errors.md` M 系列与 07 篇「Rust worldgen 作为 mod 运行」）。本 session 把 FEATURES 阶段功能真正接进生成管线并补齐缺失高度图/矿脉/结构修正/density 采样，用户明确「先整体功能实现 + 跑测试记录对齐程度，不纠结为什么没对齐」。
> 编号：本文件用 **F 系列**（functional 功能实现），与 `.investigations/rust-mod-load/rust-mod-errors.md` 的 **M 系列**（mod-load 桥接）区分，避免同课题目录跨文件编号混淆。
> 结论性对齐数据见结论 docs 草稿/主题篇；本文件只记「错在哪、为什么错、怎么发现、下次怎么避」。

---

## F1. OreVeinSampler::apply 用 `&mut self`——生成路径不必要的可变借用（会加锁）

### 现象
- `WorldgenRust/src/ore_vein.rs` 的 `OreVeinSampler::apply` 签名是 **`pub fn apply(&mut self, x, y, z) -> i32`**。
- 矿脉 sampler 作为 `WorldgenHandle` 字段（`ore_vein: OreVeinSampler`），`fill_chunk_blocks` 在块循环里以 `&mut` 借用它调用 apply → 生成路径出现可变借用。
- 在「生成路径零锁」目标（锁清理，见结论 docs）下，`&mut self` 的 apply 会迫使对该 sampler 加锁才能并发访问，**引入不必要的锁竞争**——与整个生成路径无锁化冲突。

### 根因（机制）
- apply 内部实际只用 **只读操作**：density 采样（vein_toggle/vein_ridged/vein_gap 的 `.sample`，都是 `&self`）+ `splitter.split_xyz(...)`（**本来就是 `&self`**）。没有任何需要写 self 的状态。
- 签名写成 `&mut self` 是**过度借用**——接口声明了并不需要的可变性。Rust 借用检查器不会主动收敛这个签名；它只是"能编译通过"。结果：凡是持有 `&mut` 的地方都要求独占访问，在多线程生成（RwLock/Arc 共享 handle）下就变成必须加锁。
- 这是「可变借用不是免费的——它隐含独占访问语义」的典型：**不必要的 `&mut` 不是无害的，它在共享读场景下强加互斥**。

### 定位（诊断方法）
- 在锁清理（ed59f50）规划「生成路径零锁」时**逐函数审查 apply 签名**，发现 `splitter.split_xyz` 与 density sample 都是 `&self`（Rust 编译器在调用点能静态确认）——签名里的 `&mut self` 与实现体实际只读**不自洽**。
- 判定方法：读 apply 函数体，标出所有涉及 self 的操作，看是否有任一 `&mut self` 才能做的写操作；没有 → 签名过度借用。
- **核心判据**：「函数体只有 `&self` 调用，签名却写 `&mut self`」= 签名谎报可变性，必然是不必要的借用。

### 修复
- `ore_vein.rs` 把 `apply(&mut self, ...)` 改为 **`apply(&self, ...)`**，注释明确「&self（只读：density 采样 + splitter.split_xyz 均 &self）——无需锁，并发安全」。
- 因为实现体本来就只读，**改签名不破坏任何逻辑**，纯类型层收敛——编译器验证通过。
- 修复后 sampler 只读共享，生成路径无需对 ore_vein 加锁。

### 教训（可复用判错经验）
- **Rust 里"能编译"不等于"借用正确"**：`&mut self` 如果函数体实际只读，就是过度借用——它在共享读（多线程/Arc/RwLock）下**强加独占语义 → 逼你加锁**。
- **判据**：函数签名声明的可变性 > 函数体实际需要 = `&mut self` → `&self` 收敛；凡是 `&mut self` 但体内只有 `&self` 调用，先改签名，别让它成为锁的借口。
- **锁清理的伴生检查**：声明「生成路径零锁」时，逐字段排查是否有 `&mut` 方法体实际只读——这类"签名谎报"是隐性锁来源。
- 符号级（类型/借用）错误一定是结构错，不是逻辑错——**先查签名/借用，别在采样数值上纠结**（与 AGENTS.md 判错经验同族）。

---

## F2. apply_features 的 `ocean_floor=None` 导致 getOceanFloorTopY 返回 -65——ore/disk/spring 放置判断异常

### 现象
- `worldgen_handle.rs` `apply_features` 构造 `FeatureContext` 时 `ocean_floor: None`（未构建 OCEAN_FLOOR_WG 高度图）。
- Java 侧 `getOceanFloorTopY`（OreFeature/DiskFeature/SpringFeature 判放置位置用）在 ocean_floor 未提供时返回 **min_y-1 = -65**（哨兵值）。
- 结果：依赖海洋底部高度判断放置的 ore（水下矿）/disk（圆盘）/spring（泉水）放置位置判断异常。
- 此前 FEATURES 跑通（6934ea4）但**未对齐海洋底语义**——水下 ore/disk/spring 的 y 位置是错的。

### 根因（机制）
- OCEAN_FLOOR_WG 是 Java 在 carver 前构建的**最高非水/非空气固体 y 高度图**（`Heightmap.Types.OCEAN_FLOOR_WG`），供 FEATURES 阶段判断「该列是否在水下/底部在哪」。
- Rust 的 `FeatureContext.ocean_floor` 是 `Option<&[...]>`，`None` 时 `getOceanFloorTopY` 返回 `min_y - 1`（哨兵——表示"无有效海底高度"）。
- ore/disk/spring 的放置逻辑读这个 y 判断：`blockY <= getOceanFloorTopY(x,z)` 类条件在哨兵 -65 下**对所有真实 y 恒成立/恒不成立**的错误方向——功能没接入正确的高度信息，等于"功能在但在错误坐标上跑"。

### 定位（诊断方法）
- 对比 Java `Heightmap.Types.OCEAN_FLOOR_WG` 构建时机（carver 前）+ `getOceanFloorTopY` 的 None 回退行为（`min_y - 1`）。
- 功能验证（features_probe/vein_probe）时水下 ore/disk/spring 放置位置与 vanilla 明显错位 → 追 `ocean_floor` 是否提供 → 发现 `FeatureContext.ocean_floor: None`。
- **判据**：FEATURES 阶段功能接入了但水下类（ore/disk/spring）位置错 → 先查高度图（OCEAN_FLOOR_WG）是否构建传入，None/sentinel 是默认嫌疑。

### 修复
- `worldgen_handle.rs` `apply_features` 在构建 FeatureContext 前补齐 **OCEAN_FLOOR_WG 高度图**（`ocean_floor` 数组，256 项，每列从顶向下扫，跳过 air/water/lava，取第一个固体 y = 海底/地表）。
- 构建后用 `ocean_floor: Some(&ocean_floor)` 传入 FeatureContext（原 `ocean_floor: None` 改为 Some）。
- 修复后 ore/disk/spring 能按海洋底部实际高度判断放置，水下功能不再跑在哨兵 -65 上。

### 教训（可复用判错经验）
- **"功能已接入但坐标错"先查高度图/上下文是否提供**：FEATURES 阶段的 OceanFloore/disk/spring 依赖 OCEAN_FLOOR_WG（carver 前构建的最高非水/空气固体 y），`None` 时 getOceanFloorTopY 返回 `min_y-1` 哨兵，导致放置判断恒错方向。
- **Java 的 `Option`/哨兵回退语义要还原**：Rust 侧 `Option<&[...]>` 的 `None` 对应 Java 的「未提供高度图」，不能直接忽略——要还原 None 时的哨兵返回值（min_y-1）及其对下游判断的影响。
- **功能验证不能只看"函数跑通/返回非空"，要看位置语义**：features_probe 对齐率低但功能"在跑"，要分层核对（哪类 feature、哪种高度图依赖）才能定位错位源。

---

## F3. Beardifier 接入引入 Mutex 锁（beardifiers）——生成路径加锁

### 现象
- 接入 Beardifier（结构密度修正，a6a53f7）前面临：`WorldgenHandle.beardifiers` 字段类型是 **`Mutex<HashMap<(i32,i32), Beardifier>>`**（此前 set_beardifier 用 `lock().unwrap().insert`，clear 用 `lock().unwrap().clear`）。
- 若 fill_chunk 读 beardifier 也走这个 Mutex，则**生成路径每 chunk 读都要拿 Mutex**——与「生成路径零锁」目标冲突，且读并发场景（多线程 fill）下 Mutex 会序列化所有读。

### 根因（机制）
- Mutex 是**互斥锁**：同一时刻只允许一个读者/写者。而 beardifier 的使用模式是 **set/clear（写）极低频（CppBridge 只在切 chunk/config 时 set）** + **fill（读）高频并发**——典型「写少读多」。
- Mutex 无法区分读写，读也会互斥 → 高频读在并发下争同一把锁。
- 正确并发原语是 **RwLock（读写锁）**：写独占、**读可共享无争用**——与「写少读多」模式匹配，fill 的读并发不阻塞。

### 定位（诊断方法）
- 在规划「生成路径零锁」时审查 beardifier 访问模式：set/clear 低频写 + fill 高频读 → 应选 `RwLock` 而非 `Mutex`。
- **判据**：一个被「低频写 + 高频并发读」访问的共享状态，用 Mutex 就是把读也互斥化 → 改成 RwLock（读共享无争用）。
- 同时发现：即使 RwLock，fill_chunk 若**持读锁跨整个 fill_chunk 调用**，会把锁持有时间拉长，且 fill 里可能再调用需要该 handle 的操作造成重入风险——需要 clone 释放锁。

### 修复
- `beardifiers` 字段类型 `Mutex<HashMap<...>>` → **`std::sync::RwLock<HashMap<...>>`**。
- set_beardifier/clear_beardifier 用 `write().unwrap()`（低频写，独占），fill 读用 `read().unwrap()`（高频读，共享无争用）。
- **fill_chunk_blocks 读当前 chunk beardifier 后 clone**（`self.beardifiers.read().unwrap().get(&(cx,cz)).cloned()`），**不持锁跨 fill_chunk 调用**——`beard.as_ref()` 传入，锁在 clone 后立即释放，避免长持有与重入。
- 修复后生成路径对 beardifier 只做「read + clone + 释放」的瞬时读，无长锁。

### 教训（可复用判错经验）
- **锁类型要匹配访问模式**：「低频写 + 高频并发读」用 **RwLock**（读共享无争用），**不要用 Mutex**（把读也互斥化，并发下序列化）。
- **持锁跨度要最小化**：即使 RwLock，也不该持读锁跨大函数（fill_chunk）——**读出来 clone 就释放锁**，避免锁持有时间过长 + 重入风险。
- **「并发生成路径零锁」的检查清单**：逐共享字段核（① 有无 `&mut` 方法实际只读→F1；② 有无 Option 高度图 None→F2；③ 有无 Mutex 低频写高频读→F3 改 RwLock；④ 有无跨大函数持锁→clone 释放）。这三条构成"零锁功能接入"的完整闭合。
- 符号级错误（锁类型/借用）先查结构，不在算法数值上纠结。

---

## 附：错误 → 根因 速查表（一页索引）

| 现象/信号 | 根因 | 判错要点 |
|---|---|---|
| `OreVeinSampler::apply` 签名 `&mut self`，生成路径要给它加锁（F1） | 函数体实际只读（density `.sample` + `splitter.split_xyz` 均 `&self`），签名谎报可变性 = **过度借用** | **Rust「能编译」≠「借用正确」**：函数签名可变性 > 函数体实际需要 → `&mut self` 改 `&self`；共享读场景下不必要的 `&mut` 逼你加锁 |
| `apply_features` 的 `ocean_floor=None`，水下 ore/disk/spring 放置位置错（F2） | 未构建 OCEAN_FLOOR_WG 高度图，`getOceanFloorTopY` 返回 `min_y-1=-65` 哨兵 | **功能接入但坐标错 → 先查高度图/上下文是否提供**；Rust `Option` 的 None 要还原 Java 的哨兵回退语义 |
| Beardifier 接入手持 `Mutex`，fill 读会互斥（F3） | 「低频写(set/clear) + 高频并发读(fill)」用 Mutex = 把读也互斥化；应选 RwLock（读共享无争用） | **锁类型匹配访问模式**：写少读多用 RwLock；**持锁跨度最小化**：读出来 clone 释放，不跨 fill_chunk 长持有 |
| 生成路径有锁（整体诊断，F1-F3 共用） | `&mut` 实际只读 / Option 高度图 None / Mutex 读互斥 / 跨大函数持锁 | **「并发生成路径零锁」四查**：可变签名自洽性 → Option 高度图 → 锁类型 → 持锁跨度 |
