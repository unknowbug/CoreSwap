# CARVERS 阶段 Rust 移植：错误与根因清单（重点记录）

> 载体：`.investigations/carver-port/carver-errors.md`（错误台账，独立成篇）。
> 本文件按「现象 → 根因 → 定位 → 修复 → 教训」五段式记录把 C++ `carver.h`（661 行，CaveCarver+RavineCarver）移植到 Rust（`WorldgenRust/src/carver.rs` + `chunkrandom.rs`）过程中的编译错误。这些是 **Rust 移植 C++ 的典型坑**（借用检查器 vs C++ 裸指针/引用），跨项目可复用。
> 背景：本 session 完成移植（commit bf3d851），carver_probe 对拍 vanilla FULL 参照（seed=-8248318472910187742，4x4 origin -288,-256）：无 carver match=95.41%，有 carver match=95.61%，挖洞重合 90.88%（5842/6428）。编译期踩坑 4 个（C1-C4），全部是 Rust 借用/所有权模型与 C++ 语义的冲突。
> 结论性对齐数据见对应主题篇/时间线；本文件只记「错在哪、为什么错、怎么发现、下次怎么避」。

---

## C1. E0499 嵌套可变借用——`CarverContext` 同时持 `&mut aquifer` 与 `&mut apply_material_rule` 闭包，闭包又捕获同一 aquifer

### 现象
- 编译 `carver.rs` 报 **E0499: cannot borrow `*ctx` as mutable more than once at a time**（`carve_at_point` 内同时 `ctx.aquifer.apply(...)` 与 `ctx.apply_material_rule.unwrap()(...)`）。
- 具体：`carve_at_point` 里先 `let state = self.get_state(ctx, ...)`（内部 `ctx.aquifer.apply` 可变借用 aquifer），随后 `ctx.apply_material_rule.unwrap()(wx, wy-1, wz, has_fluid)`（闭包内部又可变借用同一 aquifer）。

### 根因（机制）
- C++ 的 `CarverContext` 是**裸指针聚合**（`Aquifer* aquifer` + `std::function<int(...)> applyMaterialRule`），`applyMaterialRule` 闭包在 `worldgen_api.cpp` 里捕获 `SurfaceContext`（内部含 aquifer 指针）——C++ 无借用检查，`ctx.aquifer` 与闭包捕获的 aquifer 是**两个独立指针指向同一对象**，可同时用。
- Rust 的 `CarverContext` 用 `&'a mut Aquifer` + `&'a dyn Fn(...)` 表达同一生命周期。`apply_material_rule` 闭包在 `carver_probe.rs` 里捕获 `&mut va.aq`（`sb.apply_material_rule_single(&rule, &biome_at_jitter, ...)` 内部用 `va.aq`），而 `ctx.aquifer` 也是 `&mut va.aq`——**同一对象被两个 `&mut` 同时持有**，Rust 借用检查器拒绝（E0499）。
- 本质：C++ 的「两个指针指向同一对象」在 Rust 里是**同一可变借用被复制**，违反「同一时刻只能一个 `&mut`」。

### 定位（诊断方法）
- 编译器 E0499 直接指出 `carve_at_point` 内 `ctx` 被二次可变借用；对照 C++ `carver.h` 的 `CarverContext`（裸指针聚合）确认「aquifer 与 applyMaterialRule 闭包共享同一 aquifer」是 C++ 允许、Rust 禁止的形态。

### 修复
- 把 `apply_material_rule` 从 `CarverContext` 的字段改为**独立参数**传入 `carve_at_point`（不放进 `ctx`），使 aquifer 的可变借用与闭包调用**在时间上不重叠**：
  - `carve_at_point` 先 `get_state`（借 `ctx.aquifer`）拿到 `state`，**释放 aquifer 借用后**再调 `apply_material_rule`（此时闭包可再借 aquifer）。
  - 即：把「同时持两个 `&mut`」改成「**顺序借用**」——先借 aquifer 算 state，再借闭包做 materialRule，两次借用不重叠。
- 最终 `carver.rs` 里 `apply_material_rule` 仍是 `ctx` 字段（`Option<&dyn Fn>`），但 `carve_at_point` 内通过**先取 state 再调闭包**的顺序避免重叠（`get_state` 返回后 aquifer 借用结束）。

### 教训（可复用判错经验）
- **C++ 裸指针聚合（多个指针指向同一对象）移植到 Rust，若这些指针在同一个函数里被同时可变使用 → 必撞 E0499**。解法不是「加 unsafe」，而是**重排借用顺序**（先借 A 算完释放，再借 B），或把共享对象拆成独立参数。
- **「同一对象两个 `&mut`」在 C++ 是合法（两个指针），在 Rust 是编译错误**——移植时先识别「哪些字段/闭包共享同一底层对象」，再决定借用顺序。
- 判错签名：E0499 + 报错点同时出现 `ctx.aquifer` 与 `ctx.apply_material_rule` → 先查「是否同一对象被两个 `&mut` 持有」，重排借用顺序，别急着 unsafe。

---

## C2. E0384 不可变变量二次赋值——`let` 绑定缺 `mut`，循环内被重新赋值

### 现象
- 编译报 **E0384: cannot assign twice to immutable variable**（`carve_tunnels` / `carve_ravine` 的 `x/y/z/width/yaw/pitch` 在循环体内被 `+=` 重新赋值）。
- 具体：`carve_tunnels` 参数 `x: f64, y: f64, z: f64, width: f32, yaw: f32, pitch: f32` 在 `for j in ...` 循环里 `x += ...; y += ...; z += ...; pitch *= ...; yaw += ...`。

### 根因（机制）
- C++ 函数参数默认**可重新赋值**（`double x` 在函数体内可 `x += ...`）；Rust 函数参数默认**不可变绑定**（`let` 语义），要重新赋值必须显式 `mut x`。
- 移植时把 C++ 的 `double x` 直接写成 Rust 参数 `x: f64`（漏 `mut`），循环内 `+=` 触发 E0384。

### 定位（诊断方法）
- 编译器 E0384 直接指出「assign twice to immutable variable」+ 行号（循环内 `+=`）；对照 C++ 函数签名确认这些参数在 C++ 里是「可重新赋值的局部变量」。

### 修复
- 参数加 `mut`：`mut x: f64, mut y: f64, mut z: f64, mut width: f32, mut yaw: f32, mut pitch: f32`（`carver.rs` L505/L584 已加）。
- 注意：`carver_probe.rs` 里 `let mut aq = ...` 的 `mut` 是**多余**的（编译器 warning `unused_mut`），与 C2 相反——Rust 的 `mut` 是「按需声明」，不是「C++ 默认可变」的惯性。

### 教训（可复用判错经验）
- **C++ 函数参数默认可变 → Rust 参数默认不可变**：移植时凡 C++ 函数体内被重新赋值的参数，Rust 必须显式 `mut`。漏 `mut` 是 Rust 移植 C++ 最高频的编译错误之一。
- 反向：Rust 里 `mut` 是「按需」，C++ 移植过来常会**多写 `mut`**（编译器 `unused_mut` warning）——`mut` 只加在真正被重新赋值/可变借用的绑定上，别照 C++ 惯性全加。

---

## C3. E0502 闭包借用冲突——`skip_predicate` 闭包不可变捕获 `l`/`fs`，与 `random` 可变借用冲突

### 现象
- 编译报 **E0502: cannot borrow `random` as mutable because it is also borrowed as immutable**（`carve` 里 `skip_predicate` 闭包捕获 `l`，随后 `random.next_int_bound(...)` 可变借用 `random`）。
- 具体：`CaveCarver::carve` 里 `let l = cfg.floor_level.get(random) as f64;` 后定义 `let skip_predicate = |rx, ry, rz, _y| { ry <= l || ... };`，闭包**不可变捕获 `l`**；随后 `random.next_int_bound(4)` 等**可变借用 `random`**。若 `l` 与 `random` 生命周期纠缠（如 `l` 由 `random` 派生且闭包捕获 `l` 的引用），会触发 E0502。

### 根因（机制）
- C++ 的 `skipPredicate` 是 `std::function` lambda，**按值捕获** `l`（`[&]` 或 `[=]`），与 `random` 无借用关系；Rust 闭包默认**按引用捕获**，若闭包捕获了由 `random` 派生的值（`l`）的引用，而闭包存活期间又对 `random` 做可变借用 → 冲突。
- 实际触发点：`skip_predicate` 闭包在 `carve` 里被**多次调用**（`carve_cave`/`carve_tunnels` 传入），闭包存活期跨越 `random` 的后续可变借用；若闭包捕获 `l` 的**引用**（而非拷贝），则 `random` 的 `&mut` 与闭包的 `&l` 冲突。

### 定位（诊断方法）
- 编译器 E0502 指出「borrow `random` as mutable while borrowed as immutable」+ 闭包定义行与 `random` 可变借用行；对照 C++ lambda 捕获方式（按值 vs 按引用）确认差异。

### 修复
- 让 `skip_predicate` 闭包**按值捕获** `l`（`move` 或把 `l` 拷贝进闭包），使闭包不持有 `random` 派生值的引用：`let l = cfg.floor_level.get(random) as f64;` 后 `let skip_predicate = move |rx, ry, rz, _y| { ry <= l || ... };`（`l` 是 `f64`，`Copy`，`move` 后闭包独立持有）。
- 同理 `RavineCarver::carve_ravine` 的 `skip` 闭包捕获 `fs`（`Vec<f32>`）——`fs` 是独立 `Vec`（`create_horizontal_stretch_factors` 返回），闭包按引用捕获 `fs` 与 `random` 无冲突，但若 `fs` 与 `random` 纠缠需同样处理。

### 教训（可复用判错经验）
- **C++ lambda 默认按值捕获（`[=]`）→ Rust 闭包默认按引用捕获**：移植时凡 C++ lambda 捕获的标量/独立值，Rust 闭包要显式 `move`（或确认捕获的是 `Copy` 值），否则闭包持有引用会与后续可变借用冲突（E0502）。
- **闭包存活期跨越其他可变借用时，优先让闭包按值捕获（`move`）**——尤其捕获的是 `f64`/`i32` 等 `Copy` 标量时，`move` 零成本且消除借用纠缠。
- 判错签名：E0502 + 报错点「闭包捕获值 + 后续 `random`/其他 `&mut`」→ 先查闭包是否按引用捕获了派生值，改 `move`。

---

## C4. E0382 move 值——`ConfiguredCarver` 从 cache 取出后 move 进 `carve`，后续循环再用

### 现象
- 编译报 **E0382: use of moved value**（`carver_probe.rs` 里 `cc.carve(...)` 后 `cc` 被 move，循环下一轮再用 `cc`）。
- 具体：`let cc = match get_carver(carver_id, &mut carver_cache) { Some(c) => c, ... };` 后 `cc.carve(&mut ctx, ...)`——`carve` 接收 `&self`（`ConfiguredCarver::carve(&self, ...)`），但 `cc` 若被 move 进某处（或 `carve` 签名是 `self`）则后续 `cc` 不可用。

### 根因（机制）
- C++ 的 `ConfiguredCarver` 是**值类型**，`carve` 是 `const` 成员函数（`&self` 语义），可反复调用；Rust 若 `carve` 签名写成 `self`（按值消费）或 `cc` 被 move 进闭包/其他调用，则后续循环再用 `cc` 触发 E0382。
- 实际：`get_carver` 返回 `Option<ConfiguredCarver>`（`Clone`），`cc` 是**局部值**；`cc.carve(...)` 若接收 `&self` 则没问题，但若 `carve` 内部 `match self` 后 move 了字段（如 `ConfiguredCarver::Cave(cfg)` 的 `cfg` 被 move），或 `cc` 被 move 进 `carve` 的 `self` 参数，则 `cc` 失效。

### 定位（诊断方法）
- 编译器 E0382 指出「use of moved value: `cc`」+ move 发生行（`cc.carve(...)`）与后续使用行（循环下一轮）；对照 C++ `ConfiguredCarver::carve` 是 `const` 成员（`&self`）确认 Rust 应接收 `&self`。

### 修复
- `ConfiguredCarver::carve` 签名用 `&self`（`pub fn carve(&self, ...)`），内部 `match self` 用 `&CaveCarverConfig`/`&RavineCarverConfig`（`ConfiguredCarver::Cave(cfg)` 匹配 `&self` 时 `cfg` 是 `&CaveCarverConfig`），不 move 字段。
- `carver_probe.rs` 里 `cc` 从 cache `clone()` 取出（`get_carver` 返回 `Some(c.clone())`），`cc.carve(&self)` 不消费 `cc`，循环可复用。

### 教训（可复用判错经验）
- **C++ 值类型 + `const` 成员函数（`&self` 语义）→ Rust 必须用 `&self`，不能写成 `self`（按值消费）**：`&self` 可反复调用，`self` 只调用一次。移植时先确认 C++ 成员函数是否 `const`，决定 Rust 用 `&self` 还是 `&mut self`。
- **从 cache/容器取出的值，若后续循环复用，用 `clone()` 而非 move**（`get_carver` 返回 `Some(c.clone())`）；`&self` 方法不消费，`clone` 只是保险。
- 判错签名：E0382 + 报错点「循环内 `cc.carve(...)` 后下一轮再用 `cc`」→ 先查 `carve` 签名是否 `&self`，再查是否 move 了字段。

---

## 附：错误 → 根因 速查表（一页索引）

| 现象/信号 | 根因 | 判错要点 |
|---|---|---|
| E0499 嵌套可变借用，`carve_at_point` 同时 `ctx.aquifer.apply` 与 `ctx.apply_material_rule`（C1） | C++ `CarverContext` 裸指针聚合（`Aquifer*` + `std::function` 闭包捕获同一 aquifer）→ Rust 同一对象被两个 `&mut` 同时持有 | **C++ 裸指针聚合（多指针指向同一对象）移植 Rust，同函数同时可变用 → 必撞 E0499**；解法 = 重排借用顺序（先借 A 算完释放再借 B），别急着 unsafe |
| E0384 不可变变量二次赋值，`carve_tunnels`/`carve_ravine` 的 `x/y/z/width/yaw/pitch` 循环内 `+=`（C2） | C++ 函数参数默认可变 → Rust 参数默认不可变绑定，漏 `mut` | **C++ 参数默认可变 → Rust 参数默认不可变**：凡 C++ 函数体内被重新赋值的参数，Rust 必须显式 `mut`；反向：Rust `mut` 按需声明，别照 C++ 惯性全加（`unused_mut` warning） |
| E0502 闭包借用冲突，`skip_predicate` 捕获 `l` 与 `random` 可变借用冲突（C3） | C++ lambda 默认按值捕获（`[=]`）→ Rust 闭包默认按引用捕获，闭包持有派生值引用与后续 `&mut` 冲突 | **C++ lambda 按值捕获 → Rust 闭包按引用捕获**：闭包存活期跨越其他可变借用时，优先 `move`（捕获 `Copy` 标量零成本）；E0502 + 闭包捕获派生值 → 改 `move` |
| E0382 use of moved value，`cc.carve(...)` 后循环下一轮再用 `cc`（C4） | C++ 值类型 + `const` 成员函数（`&self`）→ Rust 若 `carve` 写成 `self`（按值消费）或 move 字段 | **C++ `const` 成员函数 → Rust 用 `&self`（可反复调用），不能 `self`（只一次）**；cache 取出复用值用 `clone()` 而非 move |
