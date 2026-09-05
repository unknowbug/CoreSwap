# phase4-apply-report-260905-05 — tree patch 机械搬运应用报告

> 应用对象：`.investigations/feature-parity/phase4-tree-patch-260905-05.md` → src（worldgen-core）。
> 执行角色：代码应用 worker（机械搬运，禁语义改动；未跑编译——本会话无 shell）。
> 锚点核对：所有目标锚点**均命中**，无 skipped hunk。

## 一、逐文件 hunk 清单

### 1. `worldgen-core/src/tree.rs`（新建，patch §1 全文）
| hunk | 状态 |
|---|---|
| §1 全文（L12-703 rust 块） | applied（逐字搬运） |

- 新增 use：文件自带（crate::blocks / chunkrandom / feature::OreFeatureContext / json / placement::IntProvider as IntProv）——无额外新增。
- 「应用时需补」项落实：
  1. **weighted total=0 守卫**（patch §5.3 明示「应用时补上」）：`BlockStateProvider::get` Weighted 分支加 `if *total_weight <= 0 { return entries[0].0; }`（在 next_int_bound 之前，0 消费）。
  2. **`can_replace_pub` 公开包装**：patch §2.2 simple_block 分支引用 `crate::tree::can_replace_pub`，但 §1 未交付该函数——编译必需项，新增 3 行 `pub fn can_replace_pub(...) -> bool { can_replace(...) }` 原样转发（语义零变化）。

### 2. `worldgen-core/src/lib.rs`
| hunk | 状态 |
|---|---|
| `pub mod tree;`（加在 feature_loader 声明后，照现有 mod 声明格式） | applied |

### 3. `worldgen-core/src/feature_loader.rs`
| hunk | 状态 |
|---|---|
| §2.1 ConfiguredFeature 结构体 + parse 全文替换（含 supersedes 注释解除原 L50 拍板注） | applied |
| §2.2 generate_configured 全文替换（签名 +`cache: &FeatureCache`）+ generate_nested 新函数 | applied |
| §2.3 preload_all 增补段（内嵌 placed 递归预加载）插在 configured 预加载 `if let Ok(croot)` 块之后、`self.placed.insert` 之前 | applied |

- 顶部 use：**无需新增**——patch 全部用 `crate::tree::…` 全路径，FeatureCache/FeaturePlacementContext 本文件已有（use 行原样保留，未动）。

### 4. `worldgen-core/src/worldgen_handle.rs`
| hunk | 状态 |
|---|---|
| §2.4 generate_configured 调用点（原 L914-915 闭包）追加 `&FeatureCache` 实参 | applied |
| §3.8 配套：FeaturePlacementContext 构造点（原 L887，全仓唯一构造）增 `anchor_biome: Some(cur_biome_id.clone())` | applied |

- 全仓 grep 确认：`generate_configured(` 唯一调用点 = worldgen_handle 闭包（F-1 教训核对通过）；`FeaturePlacementContext {` 唯一构造点同处；versions\1.20.1\rust 薄壳无引用（grep 仅命中已归档 C++）。

### 5. `worldgen-core/src/placement.rs`
| hunk | 状态 |
|---|---|
| §3.1 enum 增项：SurfaceWaterDepthFilter(i32) / EnvironmentScan{..} 追加；BlockPredicateFilter 变体重构为 `{ predicate: BlockPredicate }` | applied |
| §3.2 BlockPredicate 枚举 + parse + test 新段（插在 FeaturePlacementContext 之后、PlacementModifier enum 之前） | applied |
| §3.3 get_positions 三新分支；旧 BlockPredicateFilter 分支（原 L182-189）删除 | applied |
| §3.4 parse：surface_water_depth_filter / environment_scan / block_predicate_filter(谓词树) 三新分支；旧 matching_fluids/matching_blocks 写死分支（原 L234-254）删除；尾部 `None` → 未知 type 显式告警 + None | applied |
| §3.5 IntProvider::Trapezoid / BiasedToBottom get() 修正替换 | applied |
| §3.7 PlacedFeature::parse_inline 新增（并入现有 impl PlacedFeature 尾部） | applied |
| §3.8 FeaturePlacementContext 增 `pub anchor_biome: Option<String>`；Biome 分支替换为 anchor 对比实现 | applied |

### 6. `worldgen-core/src/carver.rs`
| hunk | 状态 |
|---|---|
| §3.6 HeightProvider 全文替换（增 trapezoid/plateau + trapezoid get 实装） | applied |
| 配套构造点同步（编译必需）：CarverConfig 默认值 2 处 struct 字面量补 `trapezoid: false, plateau: 0` | applied |

## 二、适配点汇总（编译必需 / patch 明示，均记录）

1. **tree.rs `can_replace_pub`**：patch §2.2 引用但 §1 未交付——新增公开转发包装。
2. **tree.rs weighted total=0 守卫**：patch §5.3 明示应用时补上——已补。
3. **BlockPredicate::test 的 air 常量**：patch 写 `crate::constants::AIR_ID` 并自注「若无常量：接线点」——本仓无 constants 模块，改用 `crate::blocks::AIR`（=0，语义等价）。
4. **BlockPredicate::test 的 block_at 调用**：patch 直写 `ctx.block_at(x,y,z)`，但 `FeaturePlacementContext.block_at` 是 `Option<&dyn Fn>` 不可直调（旧代码用 `.unwrap()()`）——包一层闭包 `block_at(bx,by,bz) = ctx.block_at.map_or(-1, |f| f(...))`（None → -1 = patch 声明的「不可读 → false」保守语义）。
5. **§3.3 max_depth==0 early-return 删除**：patch §3.3 注明该短路是「错误保留项」并建议实现时删掉——已按注删除，只走精确路径（两形态无行为差异）。
6. **carver.rs HeightProvider 字面量构造 2 处**：新字段 trapezoid/plateau 补默认值 `false/0`（保持 Copy + 原行为）。
7. **worldgen_handle anchor_biome 实参**：patch 说「由 worldgen_handle 闭包填入」未给代码——用当前 chunk biome `cur_biome_id.clone()`（与 biome_at 同源采样，None 直通路径保留）。
8. **feature_loader 顶部 use**：确认无需新增（全路径引用）。

## 三、存疑项（不阻塞，移交 judge/S5+）

1. **未编译验证**：全部搬运为静态拼接，按 patch 原状「未编译验证」；须 `cargo build --offline -p worldgen --release`。重点风险：§2.2 selector 闭包对 `octx`（&mut）的捕获与后续借用时序（patch §5.2 自身已登记 RefCell 备选）；`generate_nested` 为占位实现（S6/S7 接线点）。
2. **idk-7 占位**：selector/patch generate 公式为推断占位，结果禁入对拍（patch §5.7）。
3. **数据边界**：REPLACEABLE_BY_TREES / LOGS / DIRT tag / soil 集合为硬编码主体近似（R-3 tag 数据待接线）；cocoa age / vine face / leaf distance 属性位缺失（R-4），palette 对比已知偏差源。
4. **§3.5/§3.6 随机序列变更**：Trapezoid/BiasedToBottom 修正会改变现有 height_range 类 feature 的随机序列（patch §6 预告），S5 需 palette 回归定界。
5. **anchor_biome 收紧面**：Biome modifier 由「恒直通」变为「biome_at+anchor 均在时过滤」——anchor=chunk biome 与 Java posToBiome(8 邻域 jitter) 口径不同；biome_at 缺席时仍直通（不收紧）。对拍时注意此口径差。

（draft，260905-05，apply worker。patch 原文未改动。）
