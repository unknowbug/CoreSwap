# 草稿：docs/09 第八节增补段（subagent 产出，主会话应用）

> **应用位置**：`versions/1.20.1/docs/09-multi-dimension.md`——「## 八、Rust 多世界落地（2026-08-30 Phase A/B/C…）」小节**末尾追加**（「### 遗留课题」之后）。追加不覆盖。
> status: **candidate**。

---

### 补遗（2026-08-30 深夜）：JSON 布尔读取修复 + 熔岩机制 + 数字更新

> 错误台账 M6/M7：`.investigations/multiworld-port/multiworld-errors.md`（含速查表）；熔岩机制源码证据：`.investigations/multiworld-port/analysis-nether-lava-mechanism.md`。

**修复 3（JSON 布尔，M6）**：`nether.json` 的 `"aquifers_enabled": false` 是 JSON 布尔，Rust 读取走 `as_f64()`（自研 json.rs 只匹配 Number，Bool 恒 None）→ `unwrap_or(true)` 默认值静默生效 → 下界被错误启用真实含水层（6.7 万块水 vs vanilla air）。同款坑连带 `legacy_random_source`（legacy 分流从未激活）与 feature.rs `requires_block_below`。修复：json.rs 加 `as_bool()`（Bool 直读 + Number 兼容 !=0），三处读取改 as_bool。

**修复 4（熔岩机制，M7，源码确认）**：vanilla 下界熔岩 = `aquifers_enabled=false` 时 `ChunkNoiseSampler` 走 `AquiferSampler.seaLevel()` 匿名实现——density≤0 → `y < sea_level ? lava : air`（严格 <，无噪声参与）；buildSurface 跳过流体格（SurfaceBuilder L136 只记录液面）。docs/09 旧猜测「来自 fillFromNoise」「buildSurface 跳过流体格」均证实。Rust：VanillaAquifer 加 settings 数据驱动的 sea_level，`!enabled` 分支同语义实现。

**数字更新（本节此前数字作废，以本条为准）**：
- nether match **82.69%**（M6/M7 修复前 74.04%）：y≥128 **100%** / y0..31 **79.6%** / y32..63 **65.8%**（修复前 7.9%）/ y64..95 55.2% / y96..127 61.0%。
- overworld **95.40% 零回归**；两次运行**逐位一致**（确定性保持）。

**遗留课题更新**（第八节原「遗留课题」之上追加）：
- soul_sand_valley 表面残差（y=1..2）；
- legacy 分流激活验证（as_bool 修复后读取已通，块级输出仍未变）；
- Hole 语义不一致（Rust `surface_depth <= 0` vs Java `stoneDepthAbove <= 0`，C++ L251 才对——Rust 注释声称对齐 Java 是错的，影响 nether lake/not(hole) 门控，单开课题）；
- 末地引擎未启动（同前）。
