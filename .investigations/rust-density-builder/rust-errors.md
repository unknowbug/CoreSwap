# Rust worldgen 重写：错误与根因清单（重点记录）

> 载体：`.investigations/rust-density-builder/rust-errors.md`（错误台账，独立成篇）。
> 本文件按「现象 → 根因 → 定位 → 修复 → 教训」五段式记录 Rust（WorldgenRust/）重写对齐 C++ density_builder 过程中的错误。结论性对齐数据见对应主题篇/时间线；本文件只记「错在哪、为什么错、怎么发现、下次怎么避」。
> 背景：WorldgenRust/ 已完成 density_builder/finalDensity 并对齐 C++。本 session 排查踩坑 4 个（R1-R4），其中 R1 有完整排查链（疑似 bug → 排除 → 确因参照文件错误）。

---

## R1. Beardifier/过期参照坑——对拍误判「Rust bug」实为历史参照文件配置不同（重点，最完整排查链）

### 现象
- 目标：验证 Rust 块级 y-column 填充（`chunkfill_probe.rs` 采样 seed=8576294172403134396 的 final_density 树，chunk(45,-26) row(8,8) → world 列 (x=728, z=-408)，y=-64..319 共 **384 点**）。
- 用**历史参照文件** `versions/1.20.1/data/cpp_density_8576_45_-26_b8_8.txt`（C++ 输出，y 降序逐行 `y val`）对拍，得 **matched=256/384、maxDiff=1.515e-2 @y=-8（ref=0.012826, got=-0.0023230549714523385）**；128 个点（主要 y∈[-40,240] 可变密度区）明显不一致。
- 常量区一致：y≥108 恒 **-0.458333**、y≥252 恒 **-0.024995**（这两段与 Rust 对齐）。

### 根因（机制）
- 一度怀疑是 Rust 的插值/range_choice 在该区段有 bug。**随后把 6 点加入当前 C++ 参照（`rust_ref_check.cpp` final_density 采样）**：`(728,-64,-408)/(728,-40,-408)/(728,-8,-408)/(728,0,-408)/(728,120,-408)/(728,319,-408)`，得**当前 C++ = -0.00232305 与 Rust 完全一致**——即**当前 C++ 与 Rust 一致，是历史参照文件不对**。
- 真正的机制差：`cpp_density_8576_45_-26_b8_8.txt` 是由**含 Beardifier 结构密度修正**的完整 C++ worldgen 生成；而 `density_builder.h` 的 buildNode（= Rust = 当前 C++ `rust_ref_check`）**不含 Beardifier**（`@anchor.idk`「结构 Beardifier 密度修正未实现…」，D23 段；结构附近如 (728,-8,-408) 邻近 (784,160,-408) 结构区，二者差 ~0.015）。
- 二者是**不同密度配置**，不可混用——一个「完整 worldgen 配置」、一个「buildNode 配置」，参照文件配置 ≠ 被测代码配置。

### 定位（诊断链）
1. **Rust vs 历史 cpp_density 对拍** → 差异点集中在 y∈[-40,240] 可变密度区（256/384 匹配）。
2. **加同点进当前 C++ `rust_ref_check` 采样**（6 点覆盖常量/可变/结构区）→ 当前 C++ == Rust 完全一致。
3. **判定参照文件配置不同**（历史 cpp_density 含 Beardifier，buildNode/Rust 不含）。
4. **用当前 C++ 全列 dump 重新作参照**：`versions/1.20.1/cpp/build-msvc/bin/cpp_col728.txt`（`COL y val` 格式）。

### 修复
- 对拍 buildNode 一致性改**用当前 C++ 重编译的列 dump（`cpp_col728.txt`）**作参照，**弃用历史 `cpp_density_*` 文件**。
- 修复后对拍 **384/384 一致、maxDiff=3.582e-9**（float32 级）。

### 教训（可复用判错经验）
- **对拍 buildNode 必须用「当前 C++ 构建新生成的参照」，不能沿用历史 `cpp_density_*` 文件**——那些含 Beardifier，属完整 worldgen 配置，非 buildNode 配置。
- 结构附近 buildNode 与完整 worldgen 差 ~0.015（Beardifier 未实现，是 `@anchor.idk` 已知边界）；**差值落在「结构区附近」+「可变密度区」而非「常量区一致」时，先怀疑参照配置**。
- **区分「参照文件配置」与「被测代码配置」是判断「是不是 bug」的第一关**——先证参照正确，再去怀疑代码。本次若直接信历史参照，会把「当前 C++==Rust 一致」的正确实现误判成「Rust bug」。

---

## R2. ABS/SQUARE mn 边界写成 `max(0, |imin|)` → 下界被抬高 → 范围塌缩成常量

### 现象
- `overworld_probe`（Rust）vs `rust_ref_check`（C++）对拍时，`abs(ridges)` 范围**塌缩成常量（5.7143）**，`ridges_folded` 的 **min==max=-14.1429**（退化错误，明显不是精度差）。

### 根因（机制）
- Rust `un()` 的 ABS/SQUARE 分支 `mn` 写成 `impl.max(0.0)`（= **|imin|**）；C++ `UnaryOperation::create`（L184-188）是 `mn = std::max(0.0, imin)`（用**原始 imin**）。
- 当 imin<0 时，`max(0, imin)=0`，而 `|imin|` 是 imin 的**绝对值**（正数）——下界被错误抬高 → 范围塌缩。ABS 的合法值域下界应恒为 0（取绝对值后非负），但峰值/上界由山脊形状决定，mn 取错直接导致 min/max 列退化。

### 定位（诊断方法）
- 对拍 min/max 列退化（min==max）→ 核对 `UnaryOperation::create` 的计算式（沿用 D21 减法二分类似思路：**锁定边界公式**，逐字符对拍，不靠直觉等价式）。

### 修复
- `mn = imin.max(0.0)`（Rust），**不是** `apply_unary(op, imin).max(0.0)`——后者把 `abs(imin)` 又对 0 取 max，仍是 |imin|，同样错。

### 教训（可复用判错经验）
- **复刻边界/min/max 公式要逐字符对齐 C++**，不能凭「更对称/更保守」的直觉等价式（`mn=|imin|` 看着对称实为错——C++ 用的是原始 imin 与 0 的 max，非绝对值）。
- **符号级/范围级错误（min==max 塌缩、成常量、符号都反）一定是结构错不是精度错，先查公式再谈精度**。

---

## R3. clamp 节点字段名读错 `argument`（应为 `input`）→ build_node(Null) → panic

### 现象
- 构建 `caves/entrances`（含 `minecraft:clamp` 节点）时 build_node panic：`resolve minecraft:overworld/caves/entrances failed: unsupported density type '' on node Null`。

### 根因（机制）
- Rust build_node 的 `minecraft:clamp` 分支读 `self.arg(v,"argument")`；但 C++ `buildObject` L92 用 `arg("input")`——**clamp 节点的字段是 `"input"` 而非 `"argument"`**。
- 读缺字段 → `arg` 返回 `&JsonValue::Null` → `build_node(Null)` → `unsupported density type ''`。顶层 10 个 overworld 文件**不用 clamp**（所以对拍 10 文件时没暴露），只有 caves 用。

### 定位（诊断方法）
- **instrumented error（`node Null`）直接暴露缺字段** → 对照 C++ `density_builder.h` L92 确认字段名（`input`）。

### 修复
- clamp 分支改读 `self.arg(v,"input")`。

### 教训（可复用判错经验）
- **读取 JSON 的字段名 key 要与 C++ 逐字符对齐（`input` vs `argument`）**——字段名差一个词就是 Null → unsupported。
- **覆盖测试（caves）才暴露这个分支**——初版 10 文件对拍没覆盖到，是**覆盖不全**的教训：对拍通过 ≠ 分支全覆盖，坏掉的路径恰是没采样到的路径。

---

## R4. InterpolatedDF min/max 用 `-max`（错误自算边界）→ min_value 差 0.15

### 现象
- `caves/noodle` 的 min_value 差 **0.15**（Rust **-0.0083** vs C++ **-0.1583**）；max/sample 一致。

### 根因（机制）
- Rust `DensityFunction::Interpolated(id)` 的 min/max 写成 `-id.arg.max_value()` / `id.arg.max_value()`（自算边界 + 取负）；C++ `InterpolatedDF` L560-561 是 `arg->minValue() / arg->maxValue()`（**委托 arg**，即直接用 delegate 的 min/max）。
- ⚠️ **`InterpolatedNoiseDF`（old_blended_noise）的 `-maxVal`（L474）是另一个类**，其语义正确、**不改**——两个类对 min/max 的约定不同，不能想当然套用。

### 定位（诊断方法）
- 对拍 min 列 → 核对 C++ `InterpolatedDF` 的 min/max（L560-561）→ 发现是「委托 arg」，不是「-max 自算」。

### 修复
- `Interpolated(id)` 的 min=`id.arg.min_value()`、max=`id.arg.max_value()`（委托 arg）。

### 教训（可复用判错经验）
- **每个类/变体的 min/max 语义要去 C++ 逐类核对**（「委托 arg」vs「自算边界」vs「-max」），不能想当然——同类名相似（InterpolatedDF vs InterpolatedNoiseDF）但语义不同。
- **语义看似相同、结构实为两类的节点（尤其 min/max/边界）最易踩**：先到 C++ 对应类逐类核对，再写等价表达式。

---

## 附：错误 → 根因 速查表（一页索引）

| 现象/信号 | 根因 | 判错要点 |
|---|---|---|
| matched=256/384、maxDiff=1.515e-2 @y=-8、128 点不一致、常量区一致（R1） | 历史参照文件 `cpp_density_*` 含 Beardifier 结构密度修正（完整 worldgen 配置）；buildNode（Rust/当前 C++）不含——**两个不同密度配置不可混用**。当前 C++==Rust 一致，是参照文件不对。结构附近差 ~0.015（`@anchor.idk` 已知边界） | **对拍先证参照正确**：buildNode 必须用「当前 C++ 重编译的新参照」（`cpp_col728.txt`），弃用历史 `cpp_density_*`。差异集中在结构区/可变密度区而非常量区时，先查参照配置再疑代码 |
| `abs(ridges)` 塌缩成常量 5.7143、`ridges_folded` min==max=-14.1429（R2） | Rust `un()` ABS/SQUARE 的 `mn` 写成 `impl.max(0.0)`（= |imin|）；C++ `std::max(0.0, imin)` 用**原始 imin**。imin<0 时 `max(0,imin)=0` ≠ |imin| → 下界抬高 → 范围塌缩 | **符号级/范围级退化（min==max / 成常量）= 结构错不是精度错，先查边界公式逐字符对齐 C++**。不等价式（`mn=|imin|` 看着对称实为错）不凭直觉 |
| build_node panic `unsupported density type '' on node Null`（R3） | clamp 分支读 `self.arg(v,"argument")`；C++ 是 `arg("input")`——字段名差一个词 → 读缺字段返回 Null → build_node(Null) | **JSON 字段名 key 与 C++ 逐字符对齐**；对拍通过 ≠ 分支全覆盖（顶层 10 文件不用 clamp 掩盖，caves 才暴露） |
| `caves/noodle` min_value 差 0.15（Rust -0.0083 vs C++ -0.1583），max/sample 一致（R4） | `InterpolatedDF` min/max 写成 `-arg.max_value()`（自算边界+取负）；C++ L560-561 是 `arg->minValue()/maxValue()`（**委托 arg**）。`InterpolatedNoiseDF`（old_blended）的 `-maxVal`（L474）是**另一类**，语义正确不改 | **每个类/变体的 min/max 语义去 C++ 逐类核对**（委托 arg / 自算边界 / -max 三选）；同类名相似不同义（InterpolatedDF vs InterpolatedNoiseDF），不能套用 |
