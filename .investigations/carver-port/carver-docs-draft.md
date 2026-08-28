# CARVERS 阶段 Rust 移植：结论性 docs 草稿（供主会话应用）

> 载体：本文件是**草稿**（`.investigations/carver-port/`），由主会话按价值门筛选后应用到对应主题篇/时间线。
> 按 SUBAGENT-KNOWLEDGE-GUIDE.md 价值门：**高价值语义点**（CheckedRandom 内部递归、mathSin 查表、float π）详记；**中价值算法指纹**（carveRegion 边界、CarvingMask 索引、setCarverSeed 派生）简记；**低价值一次性数值**（95.41%→95.61%、90.88% 重合、8430/6428 挖洞数）**不写 docs**（只留 `.investigations/carver-port/cmd-output/carver_probe.txt` 作验证记录）。
> 建议落点：CARVERS 阶段无独立主题篇，建议追加到 `07-block-pipeline.md`（块级流水线阶段）或新建 carver 小节；`setCarverSeed`/`CheckedRandom` 语义可并入 `02-random.md`。

---

## 一、CARVERS 阶段语义要点（高价值，详记）

### 1.1 CheckedRandom 内部递归——`carveTunnels`/`carveRavine` 用 48 位 LCG，非 Xoroshiro（重点）

- **语义**：CARVERS 阶段外层 `ChunkRandom` 基类是 `CheckedRandom`（48 位 LCG，`java.util.Random` 算法）。`setCarverSeed(worldSeed, chunkX, chunkZ)` 派生种子后，`CaveCarver::carveTunnels` / `RavineCarver::carveRavine` 内部用 `Random.create(seed)` 创建**新的 CheckedRandom**（48 位 LCG），**不是 Xoroshiro**。
- **为什么关键**：这是 C++ 移植的已知根因（2026-08-10）——C++ 曾误用 `XoroshiroRandom` → 漂移序列全错 → 挖洞位置不重合。Rust 移植必须同样用 `CheckedRandom::new(seed)`（`carver.rs` L512/L589）。
- **可复用判据**：MC 里 `Random.create(seed)` 的默认实现是 `new CheckedRandom(seed)`（48 位 LCG），**不是** `XoroshiroRandom`。凡看到 `Random.create(...)` 派生内部随机源，先确认是 LCG 而非 Xoroshiro——这是 MC 随机派生链里最易踩的「同 API 不同实现」坑。

### 1.2 mathSin/mathCos 用 MC 65536 项 SINE_TABLE 查表，非 std::sin（重点）

- **语义**：`MathHelper.sin/cos` 是 **65536 项 SINE_TABLE 查表**（`table[(int)(value * 10430.378F) & 65535]`），**不是** `std::sin`。`mathCos(value) = table[(int)(value * 10430.378F + 16384.0F) & 65535]`。
- **为什么关键**：carve 漂移逐位对齐的关键。C++ 旧实现曾用 `std::sin`（double 精度）→ 与 MC 查表（float 索引）差 → 111 步漂移累积数格（挖洞位置偏移根因）。
- **可复用判据**：MC 的 `MathHelper.sin/cos` 一律查表（65536 项，索引 `(int)(value * 10430.378F) & 65535`），**任何用 `std::sin`/`f64::sin` 替代的实现都会在长循环里累积漂移**。查表索引是 float 乘法 + int 截断 + & 65535，不是 double。

### 1.3 carveTunnels 里 `(float)Math.PI` 全程 float（重点）

- **语义**：`CaveCarver::carveTunnels` 的 `d = 1.5 + mathSin(3.1415927F * j / branchCount) * width`——`(float)Math.PI = 3.1415927F`（float π），**全程 float**，不是 double π。
- **为什么关键**：float π 与 double π 在 `mathSin` 查表索引上差 1 位 → 111 步漂移累积数格（C++ 旧实现用 double π 全程 double 是挖洞位置偏移根因之一）。
- **可复用判据**：MC 里 `(float)Math.PI` 是 `3.1415927F`（float 截断），与 `3.14159265358979323846`（double）不同。凡 Java 源码出现 `(float)Math.PI`，Rust/C++ 必须用 `3.1415927f32`，不能写 double π。

---

## 二、CARVERS 阶段算法指纹（中价值，简记）

### 2.1 setCarverSeed 派生（并入 02-random.md）

```
setSeed(worldSeed); l = nextLong(); m = nextLong();
n = chunkX*l ^ chunkZ*m ^ worldSeed; setSeed(n)
```
- `nextLong()` = `(long)next(32) << 32 + next(32)`（**有符号拼接**，MC-239059：j<0 时高 32 位被 0xFFFFFFFF 填充，非无符号位拼接）。
- `chunkX*l` 是 `int * long` → long 乘法（chunkX 符号扩展）。

### 2.2 CarvingMask 索引

```
index = (x & 15) | ((z & 15) << 4) | ((y - bottomY) << 8)
```
- 256*height 位集（`Vec<u64>`），get/set 按位。

### 2.3 carveRegion 边界（Java Carver.carveRegion）

- 范围判断：`f = 16.0 + width*2.0`；`cx = chunkX*16 + 8.0`；`|x-cx| > f || |z-cz| > f` → 返回 false。
- 循环边界：`k = max(floor(x-width) - i2 - 1, 0)`；`l = min(floor(x+width) - i2, 15)`；`m = max(floor(y-height) - 1, minY+1)`；`n = 7`（`hasBelowZeroRetrogen=false`）；`o = min(floor(y+height) + 1, minY+height-1-n)`；`p/q` 同 k/l（z 侧）。
- 归一化：`g = (s + 0.5 - x) / width`；`h = (u + 0.5 - z) / width`；`w = (v - 0.5 - y) / height`；`g*g + h*h >= 1.0` 跳过。
- y 从 `o` 递减到 `m+1`（`for v = o; v > m; v--`）。
- **洞穴中心 x/y/z 用邻域 chunk（chunkX/chunkZ 参数仅用于范围判断）；carveRegion 写方块用 targetChunkX/Z（当前 chunk）**——两套坐标，易混。

### 2.4 getState / replaceable / applyMaterialRule

- `getState`：`y <= lavaLevel.getY(minY+8=-56)` → lava；否则 `aquifer.apply(pos, 0.0)`（density=0.0）。
- `replaceable` tag：`#minecraft:overworld_carver_replaceables`（**含 water！**）。
- `applyMaterialRule`（grass 被挖后 dirt 替换）：`SurfaceContext.initVertical(1,1,fluidHeight) + rule.apply`；`hasFluid ? j+1 : INT32_MIN`。

---

## 三、验证记录（低价值，不写 docs，仅留 cmd-output）

- 对拍 vanilla FULL 参照 `vanilla_-8248318472910187742_4_-288_-256_FULL.bak.blocks`（seed=-8248318472910187742，4x4 origin -288,-256）：
  - 无 carver（surface-only）：match=95.41%，nonAir=86.89%
  - 有 carver：match=95.61%，nonAir=86.34%，Rust carved=8430，vanilla carved=6428，挖洞重合 90.88%（5842/6428）
- 这些是**一次性数值/当前对齐状态快照**，按价值门**不写 docs**；完整输出见 `.investigations/carver-port/cmd-output/carver_probe.txt`。
