# 草稿：multiworld-errors.md 追加「M9 增补：后续验证定性闭环（Biome6Probe）」

> 产出者：knowledge 落盘 subagent（2026-08-30 深夜，commit edfd145 前后素材）
> 应用方式：作为小节追加到 M9 相关内容末尾（教训段之后、`## 附：错误 → 根因 速查表` 之前）。
> status: **candidate**（BIOME6 探针定性证据支持，逐点对拍未做——见「专项路径」）

---

## M9 增补：后续验证定性闭环（Biome6Probe）——legacy climate visitor 机制定性与净负根因收窄（2026-08-30 深夜，commit edfd145 前后）

### 结论（M9 假设升级为定性结论）

1. **legacy climate visitor 就是 vanilla 下界 biome 分类的真实输入机制**——biome 采样吃的是 visitor 替换后的**固定种子**噪声（temperature/vegetation 特例：`CheckedRandom(0)/(2)` + `createLegacy(-7,[1,1])`）。M9 的 WG_LEGACY_CLIMATE 门控方向**正确**。
2. Rust 开启门控后仍净负（82.72% → 77.01%）的原因 = **Legacy-Perlin 数值实现细节偏差**：Rust `LegacyRandom(0)+new_legacy(-7,[1,1])` 的输出与 Java `CheckedRandom(0)+createLegacy(-7,[1,1])` 不同——不是机制理解错。
3. 专项路径已明确：**对拍校准**——Java `CheckedRandom(0)+createLegacy(-7,[1,1])` vs Rust `LegacyRandom(0)+new_legacy(-7,[1,1])` 同坐标采样值逐点对比，差异只可能落在三段之一：① LCG 输出序列 ② Perlin 构造（origin/permutation）③ 采样公式。

### 证据（BIOME6 探针，Java yarn NoiseRouter 直采 @ 同一批 mismatch 坐标 y=1）

| 维度 | Java | Rust（无特例） |
|---|---|---|
| temperature t | **+0.077 ~ +0.119（正）** | **-0.115（负）** |
| humidity h | -0.149 ~ -0.175 | -0.092 |
| c / e / d / w | 0 | 0 |

**判定后果链**：Java t 为正 → 落 soul_sand_valley 温度区间 → 表面 soul_sand/soul_soil（与 vanilla 参照吻合）；Rust t 为负 → 误判 nether_wastes（netherrack）——与 mismatch 完全对应（连符号都不同，非精度级小偏差）。另：legacy 激活后 first mismatch 从「bedrock 错位」质变为「soul_sand 表面」，bedrock 错位顺带解决（真根因 = 随机源，非反锚序）。

### 教训要点

- **「修对方向但数值偏差」与「机制理解错」是两类问题，判定手段不同**：前者用同坐标逐点对拍收窄（LCG 序列 / Perlin 构造 / 采样公式三段二分），后者靠源码语义核对。M9 曾把两者混在一个「净负」信号里。
- **连符号都不同的采样差 = 结构/种子级差异签名**（不是精度差），优先查随机源派生链与构造参数，别在浮点精度上纠结。
- 固定种子特例噪声（与 worldSeed 无关）是 vanilla 下界 climate/地形主干的输入——这类「固定种子路径」移植时必须单独对拍，不能靠整体 match 分数间接验证。

---

## 速查表追加 1 行（应用时同步到末尾速查表）

| 现象/信号 | 根因 | 判错要点 |
|---|---|---|
| M9 增补：Java/Rust 同坐标 6 维对比 t 符号相反（Java +0.077~+0.119 vs Rust -0.115），soul_sand_valley 误判 nether_wastes；Rust 门控开启仍净负 82.72→77.01 | legacy climate visitor 确为下界 biome 分类真实输入（定性闭环）；净负 = Legacy-Perlin 数值实现偏差（LegacyRandom vs CheckedRandom + createLegacy），非机制错 | **符号级翻转 = 结构/种子级差异**，逐点对拍三段二分（LCG 序列 / Perlin 构造 / 采样公式）；「方向对 + 数值偏」与「机制错」判定手段不同，勿混 |
