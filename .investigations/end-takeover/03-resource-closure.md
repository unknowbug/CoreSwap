# T3 — density_function/end/* 资源闭包核对（260906-04 摸底）

> 验证分层：Partial（静态文件核对）。状态：draft。数据根：`versions/1.20.1/data/worldgen/data/minecraft/worldgen/`

## 核对结果

| 资源 | 在位 | end.json 引用 | 备注 |
|---|---|---|---|
| noise_settings/end.json | ✅ | — | aquifers_enabled=false, legacy_random_source=true, min_y 0/height 128, default_block=end_stone, default_fluid=air |
| density_function/end/ 目录 | ✅（base_3d_noise.json, sloped_cheese.json） | **sloped_cheese 在闭包内** | judge 抽查更正：end.json final_density 含 `"minecraft:end/sloped_cheese"` 引用（df_ns 路径解析形态，非显式 `minecraft:reference` 节点，初稿误判为冗余）。base_3d_noise 暂未见引用。⚠️ 该引用形态能否被 Rust external_loader/resolve_ref 正确解析，摸底未验证——列入开发块待验项（本仓库两文件恰好在盘所以不缺，但「未引用=冗余」结论已收回，禁止按旧稿清理） |
| end.json 引用闭包 | ✅ 自包含 | — | 节点类型全集：add×9, mul×5, y_clamped_gradient×4, cache_2d×2, end_islands×2, blend_density×1, interpolated×1, squeeze×1, block×1 |
| 节点类型 Rust 支持 | 除 end_islands 全支持 | — | cache_2d @density_builder.rs:317, y_clamped_gradient @density_builder.rs:349；**end_islands 无分支（硬缺口）** |
| noise_params.json minecraft:end_islands | ❌ 不存在 | — | Java EndIsland 噪声为代码构造非注册表，Rust 复刻需同构代码构造 |
| end biome JSON ×5（the_end/highlands/midlands/barrens/small_end_islands） | ✅ biome/ 目录 | — | biome 加载（carvers/features）路径可复用 |
| biome_params（end 版） | ❌ 不存在 | — | end 不走 MultiNoise，预期不需要；见 02 篇判定规则 |
| surface_rule | ✅ 内嵌 end.json | 单 block→end_stone | 平凡规则，JSON 解析路径直接覆盖 |

## 结论

资源层**无缺失文件**；唯一缺口是 `minecraft:end_islands` 节点实现（代码层，非数据层）。数据侧无需新增采集，闭包核对 PASS。
