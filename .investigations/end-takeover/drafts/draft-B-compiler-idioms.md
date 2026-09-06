# draft-B — compiler-idioms.md 追加草稿（260906-04 end 接管开发块）

> 目标文件：`knowledge/discovered/compiler-idioms.md`（现最大 #17，本稿续接 #18 起）。
> 素材：`.investigations/end-takeover/260906-04-errors.md` E2 + `02-end-biome-rules.md`（SimplexNoiseSampler 移植要点）。
> 主会话应用后须同步 INDEX.md。状态：draft（subagent 草稿，待主会话应用 + judge）。

---

## 发现 #18: 复刻 Java long 算术一律 wrapping_*——Rust debug 溢出 panic 是免费的 Java 回绕审计器（260906-04）

- **发现时间**：260906-04；**置信度**：candidate（panic 行号定位 + 修复后 debug/release 同语义实证）；**module**：re-code / Rust↔Java 算术语义。

### 观察/根因
fill 大坐标 chunk（100,100）时 panic `attempt to multiply with overflow` @ `chunkrandom.rs` 的 `set_population_seed`/`set_carver_seed`。Java long 乘法天然回绕（JLS 语义，静默取低位）；Rust debug profile 算术溢出即 panic。`next_long()` 可取满 i64 域，`chunk_x * l` 超域是必然事件——此前 overworld/nether 未炸只因测试 chunk 坐标小。release profile 静默回绕，语义碰巧与 Java 一致，缺陷被掩盖。

### 修复
三处 `*`/`+` 改 `wrapping_mul`/`wrapping_add`——位模式与 Java 完全一致，非行为变更。

### 如何利用
1. **判据（MUST）**：复刻 Java long/int 算术一律 `wrapping_*`——使 debug/release 同语义，不依赖 profile 巧合。判据适用面：种子派生、hash 组合、坐标×大常量（`x * 3129871` 类，AGENTS.md 易错点清单既有条目）。
2. **正向利用（判错经验）**：新维度大坐标/新 seed 派生链首跑**必须先过 debug profile**——debug 溢出断言等价于免费的「Java 回绕点审计器」，逐个 panic 点就是一处需要 wrapping 的位置；release 只会静默吞掉。
3. 交叉引用：compiler-idioms #4（MSVC long=32 位——同一「Java long 语义在系统语言侧不成立」家族的 C++ 面）。

### 证据
`.investigations/end-takeover/260906-04-errors.md` E2；`chunkrandom.rs` set_population_seed/set_carver_seed。

---

## 发现 #19: SimplexNoiseSampler 移植要点——float/double 域边界逐行对齐 + nextDouble 乘法语义（260906-04）

- **发现时间**：260906-04；**置信度**：candidate（静态源码审查 + 同 seed Java↔Rust 置换表/origin/simplex/erosion 全等对拍实证，judge 三源核对通过）；**module**：re-code / 浮点域移植。

### 观察/根因
EndIslands 密度函数是本工程首个 SimplexNoiseSampler 移植点，其数值域构成三层混合，任何一层提前收窄/扩张都会引入与 Java 不同的舍入：
1. **构造消费序列**：`SimplexNoiseSampler.java` L33-36 —— originX/Y/Z = `random.nextDouble() * 256.0`。nextDouble 的语义 = 两个 next 拼接后 **× 2^-53 double 常量**（乘法发生在 double 域，BaseRandom.java L51-56 引证）；Rust 复刻必须先合成完整 double 再乘 256.0，禁止分域近似。
2. **采样内层是 float 域**：EndIslands `sample` 中 `f = 100.0F - sqrt(x*x + z*z) * 8.0F`、`g = (|o|*3439.0F + |p|*147.0F) % 13.0F + 9.0F`、clamp `[-100.0F, 80.0F]`、邻域阈值 `-0.9F`——全部 float 字面量（DensityFunctionTypes.java L637-661）。Rust 对应 `f32` 逐行对齐，禁止「顺手用 f64 更精确」——精度更高反而是错的。
3. **输出归一是 double 域**：最终 `(sample(...) - 8.0) / 128.0`（L665）与阈值判定（0.25 / -0.0625 / -0.21875，TheEndBiomeSource.java L73-84）发生在 double 域。

### 如何利用
1. **判据**：移植带浮点的 Java 算法，先画「域地图」——逐常量标注 F 后缀（float 域）与无后缀（double 域），Rust 侧 `f32`/`f64` 严格随行；字面量后缀就是域边界的权威标注。
2. **int 除法注意**：`x/2`、`x%2` 是 Java 截断向零除法/取模（i 域），与负坐标 floorDiv/floorMod 不同域（compiler-idioms #1），本函数内两族并存勿混。
3. **对拍分层隔离**（本轮实证有效）：先对置换表/origin 值（构造域），再对单点 simplex 采样值（采样域），最后对 erosion/biome 阈值边界（应用域）——精度问题可定位到层，不逐位盲对。
4. 交叉引用：workflow-patterns #59（对拍必须数值化比较——本轮对拍方法前提）；#40 家族的「域混淆」同型。

### 证据
`02-end-biome-rules.md` §2.2/§2.4（DensityFunctionTypes.java L626-682、SimplexNoiseSampler.java L33-36、NoiseConfig.java L105 种子直传无 split）；对拍记录：review-002-final.md ①②（seed 7691421705105351955 / 12345 双侧全等）。
