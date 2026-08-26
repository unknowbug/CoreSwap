# Rust 移植语言空间注意（CoreSwap worldgen——逆向对齐 Java）

> 主会话 | 2026-08-24 后续 | 状态：draft（实战记录，防重踩）
> 背景：Rust 重写中，Rust 语言空间的数值语义与 C++/Java 有差异，逆向对齐时须逐位复刻 C++（C++ 已对齐 Java）的语义，特别是**浮点/溢出/移位/模**。以下是实测案例。

## 实测案例：next_double float vs f64（✅ 已修复，Phase 2 noise 对齐）

**现象**：Rust `DoublePerlinNoiseSampler` 输出与 C++ 差 ~3e-6（列 3 点差 2.3-3.1e-6，非 ulp），md5 已对齐（octave 种子对）但 noise 值偏。

**根因**（C++ noise.h L54）：`nextDouble() { return (double)((impl.next() >> 11) * 1.110223E-16F); }`——**C++ 用 `1.110223E-16F`（float literal）**，`(uint64)(next>>11) * 1.110223E-16F` 是 **float 数学**（uint64 转 float（精度损失）+ float 乘法）。

Rust 初版写成 `(next>>11) as f64 * 1.110223E-16`（**f64 常量，更高精度**）——**更精确 ≠ 对齐**（Java 用 float 1.110223E-16F，origin = nextDouble*256 因 float 精度差 ~1e-6 → noise 差）。

**修复**：`pub fn next_double(&mut self) -> f64 { (((self.impl_pp.next() >> 11) as f32) * 1.110223E-16f32) as f64 }` —— 复刻 C++ float 数学（uint64→f32 截断 + f32 乘法 → f64）。

**教训**：**Java/C++ 用 float 常量的地方，Rust 必须用 f32（或 f32 常量）**，不能用 f64（更精确但不对齐）。逆向对齐是复刻语义（含精度损失），不是用更高精度。

## 实测案例：UnaryOperation ABS/SQUARE 的 mn 边界（✅ 已修复，Phase 3 density_builder）

> 本小节为「Rust 移植语言空间注意」的新增行（对应下方语言空间表格第 `mn=max(0,imin)` 行）——Phase 3 density_builder 对齐时发现的 min/max 边界复刻错误。

**现象**：Rust 复刻 C++ `UnaryOperation::create` 时，ABS/SQUARE 的 mn 写成 `|imin|`（`mn.max(0.0)`），导致 `abs(ridges)` 范围塌缩成常量（5.7143），`ridges_folded` 的 min==max=-14.1429（退化错误）。

**根因**（C++ `UnaryOperation::create` L184-188）：`mn = std::max(0.0, imin)` —— 用的是**原始 imin**（`max(0.0, imin)`），不是 `|imin|`。当 `imin<0` 时 `max(0.0, imin)=0`，而 `|imin|` 会错（`imin=-max` 时 `|imin|=max` 而非 0），于是下界被错误抬高、范围塌缩。

**定位**：`overworld_probe`（Rust）vs `rust_ref_check`（C++）对拍时，`abs(ridges)`/`ridges_folded` 的 `min`/`max` 列输出呈退化状（范围塌缩、min==max），检查 `UnaryOperation::create` 的 mn/mx 计算式定位到边界公式与 C++ 不一致。

**修复**：`mn = imin.max(0.0)`（Rust），非 `apply_unary(op, imin).max(0.0)`。

**教训**：复刻边界/范围（min/max/margin）时，**常量**的下界/上界公式要和 C++ 逐字符对齐，不能凭直觉用"更对称/更保守"的等价式（`mn=|imin|` 看着更对称，实为错）。符号级错误一定是结构错不是精度错，先查公式再谈精度。

> 🔗 语言空间表格新增行：C++ `mn = std::max(0.0, imin)` → Rust `imin.max(0.0)`（见下表）。

## Rust 语言空间注意（移植 C++ → Rust 时逐项核对）

| C++/Java 语义 | Rust 对应 | 注意 |
|---|---|---|
| **整数补码溢出**（int/uint64 wrap）| `.wrapping_add/.wrapping_mul`（显式）| Rust debug 默认 panic（release wrap），须显式 wrapping 复刻 C++ 补码 wrap |
| **浮点→整** `(int)v` / `(long)v` | `v as i32/i64` | Rust `as` 向零截断（同 C++ cast）；但 `floorD`（向负无穷）须 `as` 后修正 |
| **算术右移**（有符号 `>>`）| 同（`i64 >>`）| Rust 有符号 `>>` = 算术右移（同 C++/Java）|
| **模 `%`**（余数，符号同被除数）| 同 | Rust `%` = 余数（同 C++）；Java `remainderUnsigned`/`floorMod` 需特殊处理（如 nextInt(bound) 的拒绝取样）|
| **float 常量精度**（Java/C++ `1.110223E-16F`）| **f32 常量** | 用 f64 常量会差（不精确≠对齐），须复刻 float 精度 |
| **整数→float**（`uint64 * float`）| `as f32` | Rust `as f32`（截断）同 C++；大整数→float 精度损失须复刻 |
| **min/max 边界式**（`mn=max(0,imin)`）| `imin.max(0.0)` | 用**原始 imin**，不是 `apply_unary(op, imin).max(0.0)`；`mn=\|imin\|` 错（本案例，Phase 3）|

## 核对清单（noise 移植已用）
- `floor_d`（向负无穷）：`v as i32` + `v < i ? i-1 : i`（对齐 C++ floorD）
- `wrapping_mul/add`（补码溢出）
- `maintain_precision`：`(v/3.3554432e7+0.5) as i64` 再 `v - i64*3.3554432e7`（对齐 Java/C++）
- `>> 16`（hash_xyz 算术右移）
- `next_double`：f32 数学（**本条修复**）

> **通用**：逆向对齐 Rust 移植，**凡是 C++/Java 有 float/整数/移位/模的语言差异，Rust 必须逐位复刻**（含精度损失），不能"更精确"——对齐是复刻语义。核对清单见上。

## 引用
- noise.rs / xoroshiro.rs（Rust 移植，Phase 2 逐位对齐通过）
- noise_check_cpp.cpp（C++ 参照，已对齐 Java）
- rust-rewrite-plan.md（Phase 里程碑）
