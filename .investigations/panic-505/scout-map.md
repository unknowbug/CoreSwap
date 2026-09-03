# panic-505 勘探地图（recode-scout，260902，draft）

现象：seed=8576294172403134396，origin chunk (200,200)，64×64 sweep 顺序 fill，~2304-2560 chunk 处
panic 于 surface_rules.rs:505 `self.samplers.get(key).expect("missing noise sampler")`，backtrace 在 build_surface 内。

## 一、管线地图

### 预加载（worldgen_handle.rs WorldgenHandle::create）
- L266-284 预加载分两路：
  - ① base 3 key（L267）：`minecraft:surface` / `minecraft:surface_secondary` / `minecraft:clay_bands_offset`
  - ② `df_ns == "overworld"` 静态清单（L272-274）：`badlands_surface, badlands_pillar, calcite, gravel, powder_snow, packed_ice, ice, surface_swamp`
  - ③ 非 overworld：`collect_noise_keys(settings.surface_rule JSON)` 动态收集（L277-283）
- L299：`samplers = Box::leak(db.noise_samplers().clone())` —— **创建期快照，此后不可再补 key**。
  运行期任何未预加载 key 一律查表 miss。

### 运行期 get_noise(:505) 的全部调用方（surface_rules.rs，穷举 grep）
| key | 调用点 | 触发条件 |
|---|---|---|
| minecraft:surface | sample_run_depth L518 | 每列无条件（首次进新列） |
| minecraft:surface_secondary | sample_secondary_depth L528（dead_code）/ build_surface L1177 / apply_material_rule_single L1334 | build_surface 入口无条件 |
| minecraft:clay_bands_offset | get_terracotta_block L541 | TerracottaBands 规则命中（badlands） |
| minecraft:badlands_surface | place_badlands_pillar L1365 | **仅 eroded_badlands biome 列** |
| minecraft:badlands_pillar | place_badlands_pillar L1366 | 同上 |
| **minecraft:badlands_pillar_roof** | place_badlands_pillar L1372 | 同上，且仅当 e>0 才走到 L1372 |

另：NoiseThreshold 条件（L131）走 `ctx.noise_samplers.get(noise_key)`，miss 时 `warn_unknown_noise_key` 回退 0.0（**不 panic**）。

### panic 触发链（build_surface L1207-1223）
列级循环 → L1219 `if pillar_biome.0 == "minecraft:eroded_badlands"` → L1220 place_badlands_pillar
→ L1365/L1366 get_noise(badlands_surface/badlands_pillar)（已预加载，OK）→ 若 e>0 → L1372
get_noise("minecraft:badlands_pillar_roof") → **不在 overworld 预加载清单（worldgen_handle.rs L272-274）** → :505 expect panic。

### collect_noise_keys（surface_rules.rs L160-186）覆盖面
递归遍历 JSON 节点字段：`sequence[]` / `if_true` / `then_run` / `invert`，收集条件 `type` 含 `noise_threshold` 的 `noise` 字段。
- 只收集 noise_threshold；VerticalGradient（用 splitter 非 sampler）、其他条件无 sampler 依赖 → 对 JSON 维度而言无 :505 风险（JSON 路径的 NoiseThreshold 走 warn 回退，不 panic）。
- overworld 代码规则树不走 collect（无 JSON 源），靠静态清单——清单漏 badlands_pillar_roof 即本 bug 面。

## 二、候选缺失机制清单（互斥候选）

### 候选 a（强，推荐收敛）：overworld 预加载清单缺 `minecraft:badlands_pillar_roof`
- 证据：
  - surface_rules.rs:1372 `self.get_noise("minecraft:badlands_pillar_roof")`（get_noise → :505 expect）
  - worldgen_handle.rs:272-274 overworld 静态清单只有 `badlands_surface, badlands_pillar, calcite, gravel, powder_snow, packed_ice, ice, surface_swamp`——无 pillar_roof
  - noise_params.json:7 `"minecraft:badlands_pillar_roof": {firstOctave:-8, amplitudes:[1.0]}` 存在（可建，只是没建）
  - density_builder.rs:61 registry 有该 key（诊断 bin badlands_probe.rs:52 / beard_cmp.rs:57 均预加载了 3 个 badlands key，生产清单漏第三个）
- 机制自洽性：panic 延迟到 ~2304-2560 chunk = sweep 走到 eroded_badlands biome 区域才进 place_badlands_pillar，且需 e>0 才到 L1372——完美解释「不是 chunk 1 就崩」；backtrace build_surface → pillar → :505 一致。
- 修复方向（一行）：worldgen_handle.rs L272 清单加 `"minecraft:badlands_pillar_roof"`。

### 候选 b（弱，解释力不足）：NoiseThreshold 运行期 key 缺失
- 证据：collect_noise_keys/静态清单可能漏 noise_threshold key（如 calcite/gravel/...）。
- 反证：NoiseThreshold 走 L131 `noise_samplers.get` + warn 回退 0.0（surface_rules.rs:131-137），**不 panic**；
  且 overworld 清单已含全部 8 个 noise_threshold key。排除。

### 候选 c（不适用本案）：非 overworld JSON 维度 collect_noise_keys 覆盖缺口
- JSON 路径 NoiseThreshold 同样走 warn 回退不 panic；base 3 key 已无条件预加载（L267）。
- 与本案（overworld seed sweep）无关，但作为通用机制记录：collect 只认 `noise_threshold` 的 `noise` 字段，
  未来 JSON 节点新增 sampler 消费点（或 parse_surface_cond L1114 空串 key）会静默回退。

## 三、建议
收敛门判定：仅候选 a 有完整证据链（调用点+清单缺项+数据源+延迟触发解释），单假设收敛分析可直接做，
无需 fan-out。修复=主会话一行补清单 + 重跑 sweep 回归。
