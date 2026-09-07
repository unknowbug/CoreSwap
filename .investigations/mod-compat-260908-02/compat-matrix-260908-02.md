# CoreSwap × Voxy / Distant Horizons / Epic Terrain 兼容性评估（260908-02，draft）

> 输入：scout-mods-external.md（web 调研，URL 齐全）+ scout-coreswap-boundary.md（本地边界盘点，代码/实测出处齐全）。
> 性质：决策支持评估，非交付结论；draft，judge 未审（如需立项某条支持路线再补 judge）。
> 评级口径：绿=机制正交或已实测兼容；黄=机制可兼容但有条件/未实测/有已知坑；红=现形态不兼容（有明确撞点）。

## 总矩阵

| mod | 评级 | 机制关系 | 关键依据 |
|---|---|---|---|
| **Voxy**（本体） | **绿**（附注记） | 纯 client 渲染 LOD（Vulkan），不触碰 worldgen——与 CoreSwap（server 侧接管）正交 | scout-external §1 |
| **Voxy WorldGen addon** | **黄⚠️** | 进入 worldgen 领域的 Voxy 生态组件，LOD 生成走不走服务器 chunk 管线**未查到细节**——走管线=LOD 自动继承 Rust 地形（好）；自算=LOD 与实际地形不一致（坏） | scout-external §1（iSeeEthan/voxy_worldgen_v2） |
| **Distant Horizons** | **黄**（条件绿） | `distantGeneratorMode` 决定：**FEATURE_GENERATOR 模式驱动 vanilla worldgen 管线 → LOD 自动继承 CoreSwap 接管结果（机制绿）**；INTERNAL 模式走 DH 内部简化生成 → 远景≠实际地形（红）；客户端不复刻 worldgen（服务器不装 DH 远处无数据） | scout-external §2（DH API javadoc 一手） |
| **Epic Terrain（[ETN] 史诗地形）** | **红**（现形态） | 它是 **worldgen 数据包**（noise_settings/density_function/biome JSON）——CoreSwap 三个撞点全中：⑦ 只消费 wgDir 解包目录不读 datapack；② 接管集硬编码（ETN 的 overworld settings id 不在集内→放行 vanilla 或不一致）；overworld 代码 surface rule 特权（ETN 的 surface_rule 被忽略） | scout-coreswap §②⑦ + worldgen_handle.rs L399-417 |

## 分项说明

### Voxy（绿，注记两条）
1. 1.20.1 Forge 无官方版，依赖 XingPeng-Pixel/voxy-1.20.1 第三方移植（**alpha 线**）——风险在移植质量不在 CoreSwap；
2. Vulkan 共存无冲突面：CoreSwap gpu_ffi 生产不加载（GPU 线已搁置），无资源竞争。
3. mixin 注入面一手清单未查到——实机共存测试前保留黄字 caution。

### Distant Horizons（黄，支持成本最低的一个）
- **机制红利**：FEATURE_GENERATOR 模式下 DH 的 LOD 生成会**调用服务器真实 chunk 管线**——CoreSwap 接管后 LOD 远景自动与实际地形逐位一致，无需任何适配代码。这是三个里唯一「可能免费兼容」的。
- 条件：① 配置必须钉 FEATURE_GENERATOR（INTERNAL = 远景不一致红）；② 未实测——需一轮 e2e（Chunky/DH 装上飞一圈对拍 LOD vs 实际 chunk，#26 载具可复用）。
- DH 3.0.x Beta on 1.20.1 Forge 活跃，版本面 OK。

### Epic Terrain（红，支持成本最高）
它是数据包形态，撞的全是 CoreSwap 已知的硬编码欠账（= NEXT_SESSION 开工点 5 + 遗留 idk ①）。要支持需四件套：
1. **datapack → wgDir 解包转换器**（⑦ 无 datapack 入口）；
2. **接管集 opt-in 声明入口**（② 硬编码放行 vanilla）——即开工点 5 的 ① opt-in 清单；
3. **surface rule JSON 路径对 overworld 生效**（代码特权让位给 settings 数据）；
4. **biome climate → biome_params 转换 + 遗留 idk ① 实测**（mod 自带 biome_params 未实测）；另 biome_temperature 硬编码 vanilla 名单 → ETN 自定义 biome 温度默认 0.5，表面规则（雪/草）会失真。
⑥ feature 类型白名单对数据包形态**风险较低**（数据包只能用 vanilla feature 类型，无法加新类型）——这是 ETN 侧唯一的好消息。

## 建议排序（如要投入支持）
1. **DH**：先做一轮 FEATURE_GENERATOR e2e（半天级）——可能白拿兼容；
2. Voxy 实机共存冒烟（纯渲染正交验证，半天级）；
3. Epic Terrain：本质 = 开工点 5（维度/数据包参数化三步）的第一个真实需求方——建议等参数化课题立项时把 ETN 当验收用例，不单独立项。
