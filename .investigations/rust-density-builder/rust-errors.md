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

## R5. 参照数据种子污染——`bench.seed` 只写标签不设世界种子，`.density`/`.blocks` 实为 `519...` 世界生成（重点，最易复现）

### 现象（具体数据）
- 目标：验证 Rust finalDensity vs vanilla（种子 -2032795982907864146，chunk(0,0)-(3,3) 16 chunk）。
- `vanilla_cmp_probe`（Rust vs `vanilla_-2032795982907864146_4.density`）得 **matched(<1e-9)=7280/12288、maxDiff=7.727e-1 @(4,24,48)（vanilla=+0.314, rust=-0.458）**；`diff_by_y` 显示 **y∈[-56,80] 地面带 ≥0% 匹配、y≥112 air 区 100%**。
- 即：地下/地面带分分钟不同，air 区却 100%「吻合」。

### 根因（机制）
- **`-Dbench.seed`（gradle `-PbenchSeed`）不会设置世界种子**；世界种子完全来自 `run/server.properties` 的 `level-seed`。
- 当时 `level-seed` 为空 + `run/world/` 是**既有复用目录**（非本次生成）→ 世界种子 = level.dat 里的 **`519481969467018787`**。
- WorldGenBench/BlockProbe 采样 `noiseConfig.getNoiseRouter().finalDensity()` = **世界真实种子`519...`**，写文件时却把 `bench.seed`（`-2032795982907864146`）写进文件名/header。
- 于是 `.density`/`.blocks` 是**`519...` 世界的数据**、却标着 `-2032795982907864146` 标签 → 与用 `-2032` 建树的 Rust 对拍，必然地面带全错。
- **air 区为何"看似吻合"（红色鲱鱼）**：`final_density` 顶层 `min(squeeze(...), caves/noodle)`，`squeeze(-1)=-0.5+1/24=-0.458333` 是**饱和钳位**。air 区输入 ≤ -1 → 两侧都钳到 `-0.45833`，与种子无关 → 早期 y≥112 天然 100%，掩盖了地下带真实误差。

### 定位（诊断链）
1. `colcmp`（单列逐点）→ ground 带 Rust 恒 `-0.4583` 钳位、vanilla 变化且为正 → 不是树 bug 特征（树 bug 不会 air 全区精确吻合）。
2. **`seedtest`**（候选种子[`-2032`, `519...`]各建树对拍 .density）→ `519...` ground 带 14.41% ≫ `-2032` 4.89% → 参照实际由 `519...` 生成。
3. **`colcmp2`**（决定性）→ 用 `519...` 建树，整列 y=-64..312 **逐点 |d|<4e-6**（含此前对不上的 ground 带）→ 铁证.
4. **查日志铁证**：`cherry-blockprobe.log` 同时打印 `seed=-2032795982907864146`（benchSeed 标签）与 `worldSeed=519481969467018787`（世界真实种子）→ 直接对拍「标签 ≠ 实际」。
5. **按 AGENTS.md 探针/参照数据采集核对铁律 #1（seed 三处核对）**复核：`level-seed` 空 + `run/world` 复用 → 世界确实是 `519...`。

### 修复
- 设 `level-seed=-2032795982907864146`（先备份 `server.properties` → `server.properties.bak-levelseed`），**删 `run/world`**（`Move-Item` 备份为 `world.bak-519seed`）强制按 `-2032` 重新生成世界，kill 残留 java。
- 重跑 `-PbenchProbe`/`-PblockProbe`（`-PbenchSeed=-2032795982907864146`），并**三查**：新日志 `worldSeed=-2032795982907864146` ✓、`.density`/`.blocks` header `seed=-2032795982907864146` ✓。
- 之后 Rust vs 新参照：`vanilla_cmp_probe` **matched=10406/12288、maxDiff=6.842e-5**（0.777 巨差消失）；`rust_vs_vanilla` **91.17% / nonAir 73.55%**（污染对照时 18.62%）。

### 教训（可复用判错经验）
- **`bench.seed` ≠ 世界种子**：`-PbenchSeed` 只改文件头标签，世界种子永远来自 `level-seed`。导出参照前先核对 `level-seed` 是否已设、`run/world` 是否被旧目录污染，**确认 `worldSeed`（探针日志）与目标 seed 一致**——这是项目铁律 #1 的初衷，本次正是漏了这一查。
- **「air 区吻合 + ground 带全错」的签名 = 参照/种子配置错**（air 饱和、ground 未饱和才暴露），不是代码 bug——树 bug 不会让 air 全区精确吻合。
- **饱和/钳位节点（squeeze/clamp）会掩蔽差异**：对拍时若某区天然恒定，先判断该区是否被饱和「假吻合」，再从非饱和区找真差异；否则会把「参照错」误判成「代码对」/「代码错」。
- **对拍前先证参照正确（种子/坐标/文件三查），再怀疑代码**——与 R1 同源的第一关。

---

## R6. Beardifier from_file 解析 piece 索引错位——先消费 tag 后未重数剩余字段 → piece 永远 0 个

> 背景：本 session 移植 Beardifier（`StructureWeightSampler` 结构密度修正）到 `WorldgenRust/src/beardifier.rs`，`beard_probe.rs` 自检断言 `b.pieces.len()==1` 失败（实际 0）。

### 现象
- `beard_probe` 自检断言 `b.pieces.len()==1` 失败（实际 **0**）。
- 加载器 `from_file` 解析出了 chunk，但 **piece 没有 push 进去**（pieces 空）。

### 根因（机制）
- beard file 的 piece 行格式为：`piece <minX> <minY> <minZ> <maxX> <maxY> <maxZ> <terrain 0-3> <groundLevelDelta>`——**共 8 个数**（terrain + groundLevelDelta 共 8 字段）。
- 加载器先 `parts.next()` **消费了 "piece" tag**，此时剩余 8 个数；但门控用了 `v.len() >= 9`（应为 **>=8**），且字段索引整体右移一位：
  - `terrain` 取 `v[7]`（应为 **v[6]**）
  - `ground_level_delta` 取 `v[8]`（**越界**，应为 **v[7]**）
- 索引全部右移一位 → 任何合法 8 字段行都不满足 `len>=9` 门 → **永远不 push** → pieces=0。

### 定位（诊断链）
1. `beard_probe` 自检断言失败（`pieces.len()==1` 期望 vs 实际 0）暴露 pieces 为空。
2. 在 `from_file` 加 `eprintln` 打印每行 tag + 结尾 `out.len()` / `pieces.len()` → 确认 chunk 数正常但 **pieces=0**。
3. 对照 `block_probe.cpp` 的 beard 格式注释确认**正确字段数为 8**（terrain + groundLevelDelta）→ 与代码门控 `>=9` / 索引 v[7]、v[8] 逐位核对 → 确认右移一位。

### 修复
- 门控 `v.len() >= 8`；
- `terrain = v[6]`、`ground_level_delta = v[7]`（索引各左移一位）。
- 修复后 `beard_probe` 断言通过（pieces 正常装载）。

### 教训（可复用判错经验）
- **解析文本格式时，先消费 tag 后剩余字段数要按「tag 已消费」重新数**——数的是 tag 之后的字段，不是原始行字段数；用原始字段数（8 字段但 tag 消费后剩 8 个）容易把门控/索引整体 +1 错位。
- **字段索引对照权威参考（`block_probe.cpp` 格式注释）逐位核对**，不要凭猜——「看着像第 N 个」远不如「对照格式注释逐位数」可靠。

---

## R7. 用 pwsh `Set-Content -Encoding utf8` 做字符串替换，破坏含中文注释的 UTF-8 源文件（'f'→'i' 全局篡改）

### 现象
- `fillbench.rs` / `fillmap.rs` 被破坏：**所有 'f' 字符变成 'i'**（`fill→iill`、`Aquifer→Aquiier`、`Classifier→Classiiier`），编译报语法错误。
- 同批改的 `fillprofile.rs` **未破坏**（该文件无中文注释）。

### 根因（机制）
- 用 PowerShell 的 `Get-Content -Raw` + `Set-Content -Encoding utf8` 对含**非 ASCII（中文注释）**的 UTF-8 文件做**编码往返**时字节错位，导致 ASCII 字符被篡改（'f'→'i'）。
- 具体机制：`Get-Content -Raw` 在错误 code page 下读入 → `Set-Content -Encoding utf8`（Windows PowerShell 的 utf8 = 带 BOM 的 UTF-8）写回，读/写两侧 code page 不一致 → 中文字节被错解 → 字节流错位污染相邻 ASCII。

### 定位（诊断链）
1. `fillbench` / `fillmap` 编译报 `expected ... found MacroBiome` 等**语法错误**（字符被篡改后的解析失败）。
2. read 文件发现 **'f'→'i' 全局替换**（`fill→iill`、`Aquifer→Aquiier`、`Classifier→Classiiier`）。
3. 对比 `fillprofile.rs`（**无中文注释，未破坏**）→ 差异只在「是否有中文注释」→ 确认是**编码往返**问题，非内容逻辑问题。

### 修复
- `fillbench.rs` / `fillmap.rs` 用 **write 工具重建**（从 git/记录恢复正确内容）。
- `fillprofile.rs` 用 **edit 工具修复**（edit 是干净字节写入，不做编码往返）。

### 教训（可复用判错经验）
- **修改含中文（非 ASCII）的 UTF-8 源文件必须用 DSH 的 edit/write 工具（干净字节写入），不要用 pwsh `Set-Content` 做字符串替换**——编码往返会破坏非 ASCII 文件（本实例 'f'→'i'）。
- AGENTS.md §八.1/4 已有相关坑，此为**新实例**（之前是 inline python / Copy-Item 目标目录，本实例是 `Set-Content` 编码往返）。
- **签名**：字符被「规律性全局替换」（尤其 'f'→'i' 这种 ASCII 变体）+ 文件含中文注释 + 无中文注释的同批文件未受影响 → 先怀疑编码往返工具链，不是内容逻辑。

---

## 附：错误 → 根因 速查表（一页索引）

| 现象/信号 | 根因 | 判错要点 |
|---|---|---|
| matched=256/384、maxDiff=1.515e-2 @y=-8、128 点不一致、常量区一致（R1） | 历史参照文件 `cpp_density_*` 含 Beardifier 结构密度修正（完整 worldgen 配置）；buildNode（Rust/当前 C++）不含——**两个不同密度配置不可混用**。当前 C++==Rust 一致，是参照文件不对。结构附近差 ~0.015（`@anchor.idk` 已知边界） | **对拍先证参照正确**：buildNode 必须用「当前 C++ 重编译的新参照」（`cpp_col728.txt`），弃用历史 `cpp_density_*`。差异集中在结构区/可变密度区而非常量区时，先查参照配置再疑代码 |
| `abs(ridges)` 塌缩成常量 5.7143、`ridges_folded` min==max=-14.1429（R2） | Rust `un()` ABS/SQUARE 的 `mn` 写成 `impl.max(0.0)`（= |imin|）；C++ `std::max(0.0, imin)` 用**原始 imin**。imin<0 时 `max(0,imin)=0` ≠ |imin| → 下界抬高 → 范围塌缩 | **符号级/范围级退化（min==max / 成常量）= 结构错不是精度错，先查边界公式逐字符对齐 C++**。不等价式（`mn=|imin|` 看着对称实为错）不凭直觉 |
| build_node panic `unsupported density type '' on node Null`（R3） | clamp 分支读 `self.arg(v,"argument")`；C++ 是 `arg("input")`——字段名差一个词 → 读缺字段返回 Null → build_node(Null) | **JSON 字段名 key 与 C++ 逐字符对齐**；对拍通过 ≠ 分支全覆盖（顶层 10 文件不用 clamp 掩盖，caves 才暴露） |
| `caves/noodle` min_value 差 0.15（Rust -0.0083 vs C++ -0.1583），max/sample 一致（R4） | `InterpolatedDF` min/max 写成 `-arg.max_value()`（自算边界+取负）；C++ L560-561 是 `arg->minValue()/maxValue()`（**委托 arg**）。`InterpolatedNoiseDF`（old_blended）的 `-maxVal`（L474）是**另一类**，语义正确不改 | **每个类/变体的 min/max 语义去 C++ 逐类核对**（委托 arg / 自算边界 / -max 三选）；同类名相似不同义（InterpolatedDF vs InterpolatedNoiseDF），不能套用 |
| finalDensity vs vanilla：ground 带(y∈[-56,80]) 0% 匹配、air 区(y≥112) 100%、maxDiff 0.777 @(4,24,48)（R5） | **参照数据种子污染**：`bench.seed` 不改世界种子（世界来自 `level-seed`）；`level-seed` 空 + `run/world` 复用 → 世界=**519...**，但 `.density`/`.blocks` 标成 `-2032`。air 区"吻合"是 `squeeze` 饱和（`squeeze(-1)=-0.45833`）与种子无关的假吻合 | **`bench.seed`≠世界种子**；导出参照前查 `level-seed`+`run/world` 是否被旧目录污染，用探针日志 `worldSeed` 三查。**「air 区吻合+ground 带全错」签名=参照/种子配置错非代码 bug**；饱和节点会掩蔽差异，先判非饱和区再定位 |
| `beard_probe` 断言 `pieces.len()==1` 失败（实际 0），chunk 解析正常但 piece 没 push（R6） | Beardifier `from_file` 先 `parts.next()` 消费 "piece" tag 后剩余 **8** 个字段，却用门控 `v.len()>=9`（应 >=8）+ 索引右移一位（terrain 取 v[7] 应 v[6]、ground_level_delta 取 v[8] 越界应 v[7]）→ 合法 8 字段行永不过门 → 不 push | **先消费 tag 后重新数剩余字段数（按「tag 已消费」数，非原始行字段数）**；字段索引对照权威参考（`block_probe.cpp` 格式注释）逐位核对，不凭猜 |
| `fillbench.rs`/`fillmap.rs` 编译语法错误，所有 'f'→'i'（fill→iill、Aquifer→Aquiier、Classifier→Classiiier）（R7） | 用 pwsh `Get-Content -Raw` + `Set-Content -Encoding utf8` 对含中文注释的 UTF-8 文件做**编码往返**字节错位 → ASCII 被篡改；`fillprofile.rs`（无中文注释）未破坏佐证 | **改含中文的 UTF-8 源文件用 DSH edit/write 工具（干净字节写入），不用 pwsh Set-Content**；签名：字符规律性全局替换 + 文件含中文 + 同批无中文文件未受影响 → 先疑编码往返工具链 |
