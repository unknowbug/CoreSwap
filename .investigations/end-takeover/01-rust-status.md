# T1 — Rust end 分支现状盘点（260906-04 摸底）

> 验证分层：Partial（静态源码/数据审查，未运行）。状态：draft。
> 一手核实：NEXT_SESSION 方向描述「create_for_dim 硬编码 overworld」**已过时/不准确**——260905-01 拆分后 `create_for_dim` 已全参数化（nether 已实证跑通），end 的缺口比交接描述小且更具体。

## 已验证事实（附证据）

### 1. create_for_dim 参数化现状（worldgen_handle.rs）

| 维度参数 | end.json 值 | Rust 支持 | 证据 |
|---|---|---|---|
| min_y / height | 0 / 128 | ✅ settings.noise 读取（默认 -64/384，end 值可覆盖） | worldgen_handle.rs:164-166 |
| aquifers_enabled | false | ✅ settings 读取，false 跳过 aquifer | worldgen_handle.rs:168, :55 |
| legacy_random_source | true | ✅ 已有（nether 实证同路径） | worldgen_handle.rs:170-171, 211 |
| sea_level | **0**（end.json 有 `"sea_level": 0` 字段，judge 抽查更正——初稿误记「无字段兜底 63」） | ✅ settings 读取命中 0，无风险 | worldgen_handle.rs:325; end.json sea_level=0 |
| surface_rule | 单条 `minecraft:block → end_stone`（平凡规则） | ✅ 非 overworld 走 settings.surface_rule JSON 解析，fail-fast；单 block 节点平凡可解 | worldgen_handle.rs:329-345; end.json surface_rule |
| vein_toggle/ridged/gap | 在位 | ✅ 不触发 router.get None 早退 | end.json noise_router 字段实测 15 项全在（与 nether 相同全集） |
| density_function 目录 | `density_function/end/` | ✅ df_ns 参数化（settings_name 去 .json） | worldgen_handle.rs:151-154, 217 |

### 2. 硬缺口（真正的开发工作）

1. **`minecraft:end_islands` density function 节点未实现**——end.json 中出现 2 次（erosion router = cache_2d(end_islands)，直接决定外岛形状）。density_builder.rs 无此节点类型（grep 全 src 无 match 分支）。Java 侧对应 `DensityFunctions.EndIslandDensityFunction`（1.20.1），其内部 OctavePerlin 噪声为代码构造（不走 noise_params 注册表——实测 data/noise_params.json **无** minecraft:end_islands 条目），amplitudes/octave 为 Java 代码硬编码 → Rust 需新写：节点分支 + 专用噪声构造 + 算法复刻（含 legacy random 派生）。
2. **end biome 判定全新组件**——end 无 MultiNoise（biome_params 文件不存在于 data/，实测只有 biome_params.json / biome_params_nether.json）；Java `TheEndBiomeSource` 为中心/高地/边缘位置判定，非 MultiNoise → 现有 `BiomeClassifier::load(biome_params)` 路径不适用，需新 classifier（规则提取见 02-end-biome-rules.md，subagent 产出中）。MacroBiome 的 temp/hum/cont 等 router 输入在 end.json 全为 0.0 常量（erosion 除外=end_islands），即使硬跑 MultiNoise 也无区分度，属结构性缺口非参数缺口。
3. **non-blocker 备注**：WG_TRANSPILER/DFC/GPU 通道硬编码 overworld（base_3d_noise overworld 路径 worldgen_handle.rs:192；DfcDensity/GpuDensity overworld 专用），但默认关（零退化铁律）——end 走默认 macro_sampler 树路径，不阻塞；end 对拍时禁开这些 env。

### 3. 资源闭包（详见 03-resource-closure.md）

end.json 引用闭包自包含：无 `minecraft:reference` 外部 df 文件；节点类型全集 = add×9 / mul×5 / y_clamped_gradient×4 / cache_2d×2 / end_islands×2 / blend_density / interpolated / squeeze / block——除 `end_islands` 外 density_builder 均已支持（density_builder.rs:317 cache_2d, :349 y_clamped_gradient）。`density_function/end/base_3d_noise.json`、`sloped_cheese.json` 在盘但未被 end.json 引用（冗余文件，闭包无关）。

## 摸底结论（待 T2 合入后出 verdict）

管线侧缺口 = **1 个新 density function 节点（end_islands）**；biome 侧缺口 = **1 个新位置判定 classifier**；其余全部复用既有 create_for_dim 数据驱动路径。缺口定性偏小，倾向轻量架构（待 T2 量级确认）。
