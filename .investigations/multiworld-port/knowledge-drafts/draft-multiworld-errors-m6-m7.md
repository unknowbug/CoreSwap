# 草稿：multiworld-errors.md 追加 M6-M7 + 附记 2 条（subagent 产出，主会话应用）

> **应用位置**：`.investigations/multiworld-port/multiworld-errors.md`——以下 M6/M7 两节 + 附记插入到「## M5. …」节之后、「## 附：错误 → 根因 速查表（一页索引）」**之前**；速查表 2 行追加到现有表格末尾。追加不覆盖。
> 来源：2026-08-30 多世界收尾后半场，commit 879329d，全部实测/源码确认。

---

## M6. nether 卡 74.04%、y32..63 带 7.9% 纹丝不动、legacy_random_source 零效果——JSON 布尔走 `as_f64()` 恒 None → `unwrap_or` 默认值静默生效（本轮最大根因）

### 现象
- nether 块级 match 卡在 **74.04%** 不再上升；按 y 分带：**y32..63 带仅 7.9%** 纹丝不动（熔岩海带带）、y0..31 也偏低。
- `legacy_random_source` 字段加了读取逻辑后**零效果**（legacy 分流从未激活）——「配置写了却不生效」的多字段聚簇。

### 根因（机制）
- `nether.json` 的 `"aquifers_enabled": false` 是 **JSON 布尔**；Rust 读取写的是：
  ```rust
  settings.get("aquifers_enabled").and_then(|v| v.as_f64()).map(|x| x != 0.0).unwrap_or(true)
  ```
- 自研 `json.rs` 的 `as_f64()` 只匹配 `JsonValue::Number`——**Bool 恒返回 None** → `and_then` 链断掉 → **`unwrap_or(true)` 的默认值静默生效**。
- 后果链：下界 aquifers_enabled 被错误当成 true → **下界被错误启用真实含水层** → 6.7 万块水（vanilla 是 air）。同款坑还埋着 `legacy_random_source`（默认 false 生效 → legacy 分流从未激活）和 feature.rs 的 `requires_block_below`。
- 本质：**「optional 读取 + unwrap_or 默认值」组合会把「字段类型不匹配」静默吞成默认行为**——不是「字段缺失」，是「字段在但类型读不到」，代码却按缺失处理走默认值，且默认值方向还恰好与 JSON 真实值相反（false → true）。

### 定位（诊断链）
1. **混淆对直方图（got→want Top 配对）**暴露 `id32=water` 大规模聚集——错误填充的是整层水，指向流体/含水层机制而非噪声值差。
2. **skip 开关二分**锁 stage：跳过 aquifer/流体相关阶段复跑 → 差异消失 → 锁定 stage 1（fill）内的流体填充路径。
3. 反查 classify 分支条件 → aquifer 启用状态的判定输入不对 → 下钻到 JSON 解析层 → 发现 `as_f64()` 对 Bool 恒 None、`unwrap_or(true)` 兜底。

### 修复
- `json.rs` 增加 **`as_bool()`**（Bool 直接读；Number 兼容 `!= 0`）。
- 三处读取（`aquifers_enabled` / `legacy_random_source` / feature.rs `requires_block_below`）由 `as_f64().map(!=0.0)` 改为 `as_bool()`。
- 修后：nether **74.04% → 82.69%**（y32..63 7.9% → **65.8%**，y0..31 59.5% → 79.6%）；overworld 95.40% 零回归。

### 教训（可复用判错经验）
- **「optional 读取 + unwrap_or 默认值」是静默默认值陷阱的标配组合**——默认值必须显式断言类型（读取后打一行日志或 assert 类型），新 JSON 字段接入时验证「读到的是什么」而不是「默认值是什么」。此坑跨语言跨项目通用（任何 self-parsed JSON/配置——Rust/Java/C++ 手写 parser——都会踩），已单独立 discovered 条目（见 knowledge/discovered/build-tooling.md 草稿）。
- **「多个配置字段同时零效果」是解析层错的聚簇签名**——单字段不生效可能是逻辑错，多字段同时「写了没反应」先怀疑共同的解析/读取层，不要逐字段查逻辑。
- **判错路径可复用**：块级混淆对直方图（got→want）定位「错的是什么」→ skip 二分定位「错在哪一段」→ 分支条件反推「输入状态错在哪」→ 才下钻解析层。层层收敛，不直接跳 JSON parser。

---

## M7. 下界熔岩的真正来源——aquifers_enabled=false 时走 `AquiferSampler.seaLevel()` 匿名实现（源码确认机制，docs/09 旧猜测证实）

### 现象
- M6 修复后 nether 熔岩仍与 vanilla 有残差——需要弄清 vanilla 下界熔岩的确切生成机制（docs/09 此前仅有「可能来自 fillFromNoise」的猜测）。

### 根因（机制，Java 源码确认）
- `aquifers_enabled=false` 时 `ChunkNoiseSampler` 用 **`AquiferSampler.seaLevel()` 匿名实现**（不是关闭 aquifer 就没有流体来源）：
  - `density > 0` → 返回 null（填 default_block）；
  - `density ≤ 0` → `FluidLevel(sea_level, default_fluid).getBlockState(y)` = **`y < sea_level ? lava : air`**（严格 `<`；无噪声参与；无上下界概念）。
- `buildSurface` **跳过流体格**（SurfaceBuilder L136 只记录液面、不应用表面规则）——表面规则不会覆盖熔岩。
- docs/09 旧猜测「熔岩来自 fillFromNoise」「buildSurface 跳过流体格」**均证实**（前者即 sea_level 实现，在 noise 填充阶段内生效）。

### 定位（诊断方法）
- 直接读 Java 源码（yarn sources）`ChunkNoiseSampler` / `AquiferSampler` / `SurfaceBuilder`，逐条落证据——机制类问题源码是权威，不做猜测性实验。

### 修复
- Rust 侧 `VanillaAquifer` 加 `sea_level`（从 settings 数据驱动读取）；`!enabled` 分支实现同语义：`y < sea_level → Lava else Air`。
- 修复后 nether 82.69%（y 分带详见 docs/08 增补段）；熔岩海带带 7.9% → 65.8% 的主要贡献源。

### 教训（可复用判错经验）
- **「开关关闭 ≠ 机制消失」**——vanilla 里 `aquifers_enabled=false` 不是「不跑流体逻辑」，而是**切换到 sea_level 简化实现**；移植开关语义前必须读 false 分支的实际实现，不能按名字直觉理解。
- 机制类定论（本条）由源码逐条证据支撑并落盘 `.investigations/multiworld-port/analysis-nether-lava-mechanism.md`——旧 docs 猜测（docs/09 🔍 lava 项）得以证实/结案，猜测→验证链条闭环。

---

## 附记（worker 发现，简记，未修单开课题）

1. **Hole 语义 Rust/Java 不一致**：Rust `SurfaceCond::Hole` 用 `surface_depth <= 0`；Java `HoleCondition` = `stoneDepthAbove <= 0`（C++ L251 写法才对）——Rust 侧注释声称「对齐 Java runDepth」是错的。影响 nether 的 lake/not(hole) 门控。未修，单开课题。
2. **三个已登记隐患**：① mixin `@Shadow` 够不到父类字段（biomeSource 在 ChunkGenerator）→ 用**缓存反射**（已用于末地保护，M5 附记同源）；② `parse_surface_rule` 未知 cond 走 `?` **静默吞掉整条分支**；③ surface rule 解析失败回退 `Block(0)` 会**写 id 0 进输出**——两者待加告警（静默降级是后续排查的隐形坑）。

---

## 速查表追加 2 行（追加到 `multiworld-errors.md` 现有速查表末尾）

| 现象/信号 | 根因 | 判错要点 |
|---|---|---|
| nether 卡 74.04%、y32..63 带 7.9% 不动、legacy_random_source 零效果（M6） | `nether.json` 的 `false` 是 JSON 布尔；Rust 走 `as_f64()`（只匹配 Number，Bool 恒 None）→ `unwrap_or(true)` 默认值静默生效 → 下界被错误启用真实含水层（6.7 万块水 vs vanilla air）。同款坑：legacy_random_source、requires_block_below | **「optional 读取 + unwrap_or 默认值」会把「字段类型不匹配」静默吞成默认行为**——默认值必须显式断言类型/打日志验证「读到的是什么」；多配置字段同时零效果 = 先查共同解析层；判错路径：混淆对直方图 → skip 二分 → 分支条件反推 → 才下钻解析层 |
| 下界熔岩分布与 vanilla 残差（M7） | `aquifers_enabled=false` 时 vanilla 用 `AquiferSampler.seaLevel()` 匿名实现：density≤0 → `y < sea_level ? lava : air`（严格 <，无噪声）；buildSurface 跳过流体格（SurfaceBuilder L136）。docs/09 旧猜测均证实 | **「开关关闭 ≠ 机制消失」**——false 分支是切换到简化实现而非跳过，移植前必读 false 分支实际源码；机制定论以源码逐条证据落盘（analysis-nether-lava-mechanism.md），猜测→验证闭环 |
