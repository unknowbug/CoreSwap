# scout-residual9-260904-12 — 残 9 gravel→sand surface 微族勘探（管线地图 + 接线 diff + 候选假设）

- 角色：recode.scout（只读勘探；本产物只进 .investigations/，不含分析定论）
- 日期标签：260904-12（沿用任务书标签；主会话落盘前建议 Get-Date 复核）
- 置信度：全部 **draft/scout 级**；「已验证事实」与「未验证假设」在 §6 分开标注

## 1. 交接材料消化（FACT，来源见括号）

- 残 76 收口根因（confirmed）：Rust surface biome 判定缺 BiomeAccess zoom，ab56706 修复后 decisive 76→12，其中 gravel→sand 49→9（`.artifacts/lossless-accel/residual76-verdict-260904-10.md`）。
- 旧 76 数据形态（`.tmp/p2full/res76-probe-260904-10.csv`，修复前）：g2s 5 个采样点全部 `pre=stone, biome=deep_lukewarm_ocean, r=63, sda=1, applied=sand`；s2g 点 biome=deep_ocean applied=gravel。即旧残差形态 = **Rust 按单元直读 biome 命中 deep_lukewarm → 走 sand 分支；Java 实际 gravel**。修复消除了直读→zoom 的主体差，剩 9 未立案。
- ⚠️ 残 9 的**具体坐标尚无落盘数据**：`off-surfacefix-260904-10/verify-surfacefix-260904-10.out.txt`（实际路径 `.investigations/residual-1830/cmd-output/`）只有计数（gravel→sand: 9），没有逐点坐标。首轮 worker 分析前必须先提取（命令模板见 §5.0）。
- 知识库 #44（workflow-patterns.md L784-789）：跨载具课题先 diff 两侧「同功能站点接线清单」，单侧缺站点即候选根因。
- NEXT_SESSION 待办 2 确认本课题先验假设：(a) zoom 二阶边界效应 (b) 新机制——**均未验证，不当公理**。

## 2. vanilla gravel/sand 判定管线地图（一手源码，src-ext 1.20.1）

### 2.1 生产链路

```
ChunkStatus.SURFACE (SurfaceBuilder.generate)
  └─ materialRuleContext = new MaterialRuleContext(..., chunk::getBiomeForNoiseGen 经 ChunkRegion 包装, ...)
     ChunkRegion.java L102: new BiomeAccess(this, BiomeAccess.hashSeed(seed))  // sha256-asLong
  └─ applySurfaceRule 逐块:
     MaterialRules.java L462-464 initVerticalContext: biomeSupplier = memoize(posToBiome.apply(pos.set(x,y,z)))
       → BiomeAccess.getBiome(精确块坐标, 含 y!): 8 邻域 quart cell jitter 选点 → chunk biome storage 值
```

### 2.2 海底 floor 残差相关分支（VanillaSurfaceRules.createDefaultRule，surface=true）

关键二级结构 materialRule9（L244-271，经 L281 `condition(surface(), materialRule9)` 施加）：

| 优先序 | 条件 | 结果 | 与残 9 相关性 |
|---|---|---|---|
| 1 | STONE_DEPTH_FLOOR_WITH_SURFACE_DEPTH (L258) | materialRule7（DTL 默认 dirt 链） | 次相关 |
| 2 | STONE_DEPTH_FLOOR (L263-270) | sequence(biome(FROZEN_PEAKS,JAGGED_PEAKS)→STONE, **biome(WARM_OCEAN,LUKEWARM_OCEAN,DEEP_LUKEWARM_OCEAN)→materialRule2(SANDSTONE/SAND)**, else **materialRule3(GRAVEL)**) | **主判据**：海底 floor 块 gravel vs sand 完全由 biome 三值切换 |
| — | biome 条件采样点 | 每块精确 (x,y,z)（含 y），经 BiomeAccess zoom | 与修复点同源 |

辅助分支（水面/浅海，次要）：materialCondition14(WARM_OCEAN/BEACH/SNOWY_BEACH)+L259 SANDSTONE、L71 materialRule2（SANDSTONE ceiling/SAND）、STONE_DEPTH_FLOOR_WITH_SURFACE_DEPTH_RANGE_6/30（L259-260）。

判定阈值事实：deep_ocean 落 else → GRAVEL；deep_lukewarm_ocean 落 L267 → SAND。**残 9 g2s（vanilla gravel / Rust sand）在 floor 语境下等价于：Rust 在该块判定出 (warm|lukewarm|deep_lukewarm) 系 biome 而 Java 判定 deep_ocean——或该块根本没走 floor 分支（分支条件/前序规则差异）。**

## 3. Rust 侧接线清单（#44 diff，Rust 视角）

| 站点 | Java 机制 | Rust 现状（工作区文件实测） | 接线判定 |
|---|---|---|---|
| surface biome 输入 | hashSeed+zoom 精确块坐标 | `worldgen_handle.rs L620-624 biome_at_surface`：`biome_pick_cell(self.biome_access_seed, x,y,z)` → `biomesrc.biome((px<<2,py<<2,pz<<2))` | ✅ 已接（ab56706） |
| surface 缓存 key | — | L631-635 packed u32 截断（对齐 C++ biomeCellKey） | ✅ |
| **carver** jitter | hashSeed+zoom | `L722-725 biome_at_jitter`：**biome_access_seed**（hashSeed）+ zoom | ✅ 已接（见 §6 风险注） |
| **features** jitter | hashSeed+zoom | `L849-853 biome_at_jitter`：**biome_access_seed** + zoom | ✅ 已接（同上） |
| surface 规则结构 | VanillaSurfaceRules L244-284 | `surface_rules.rs L1020-1038`：STONE_DEPTH_FLOOR(off=0,no add,range 0,ceiling=false) → seq(peaks→stone, 三值海洋 biome→mr2, gravel) 与 Java 1:1；L1048 SurfaceCondC(surface()) 包 mr9 | ✅ 结构对齐 |
| biome 分类器 | MultiNoiseBiomeSource SearchTree | `biome.rs` KD-tree 7 维（6 参数+offset），biome_of 采样 6 个 density fn 在 (px<<2) 块坐标 | ⚠️ 见候选 H2 |
| zoom 选点算法 | BiomeAccess.getBiome | `biome.rs L293-322 biome_pick_cell`（8 邻域、mix_seed/jitter、strict `<` 平局） | ⚠️ 见候选 H1 |
| features biome 选择面 | 3×3 chunk biome sections | `worldgen_handle.rs L817` 注释自认「简化：当前 chunk biome（biome_at_no_jitter chunk 角, y=0）」 | ❌ **已知简化，候选 H4 入口** |
| disk feature | DiskFeature（深海 biome 有 disk_sand/disk_gravel） | `feature_loader.rs L41-42/248-249` 已实装，放置用 OCEAN_FLOOR + `pos_to_biome=biome_at_jitter` | ⚠️ 见候选 H3 |

**接线 diff 结论**：surface 站点结构层已 1:1； zoom 已接满 4 站点。残 9 不太可能是「整层漏接」型根因（旧根因形态），更像**边界选点/分类差**或 **surface 之外阶段写 sand**。

## 4. 候选假设清单（draft，含互斥性与判别实验）

- **H1 zoom 选点残余差（先验 a 的具体化）**：`biome_pick_cell` 在某些子单元偏移（负坐标/极端 jitter）选点与 Java/C++ 不同 → Rust 选中 deep_lukewarm 侧 cell。C++ biome.h biomePickCell 已 confirmed 对齐 Java，故判别 = Rust pick vs C++ pick 同点对比。与 H2 互斥（选点差 vs 同点分类差）。
- **H2 同 cell 分类差**：pick 相同但 `biomesrc.biome` 在 deep_ocean/deep_lukewarm 参数边界（两者参数极近）与 Java chunk-storage 值不同（MultiNoise 采样口径/浮点/SearchTree 剪枝平局）。与 H1 互斥。
- **H3 非 surface 阶段写 sand（先验 b）**：FEATURES 阶段 disk_sand（深海 biome feature 表含 disk）在 Rust 覆写了 Java 保留的 gravel（Java 该 biome 无此 disk 或 disk 未命中该块）。判定特征：残差点若 `surface applied=gravel` 但最终存档=sand 即坐实。与 H1/H2 互斥。
- **H4 features biome 选择面简化引入**：apply_features 用 chunk 角无 jitter 单点定 cur_features（Java 是 3×3 chunk section 集合）→ chunk 角 biome 判差导致 disk_sand 放置开关翻转。与 H3 不互斥（H4 是 H3 的一个上游原因），但与 H1/H2 互斥。
- 排除候选（已知事实直接排除）：规则结构缺分支（§3 已对齐）；bare-seed 接线（工作区已全 hashed）。

≥2 互斥候选（H1|H2 与 H3|H4 两轴）→ **需 fan-out .bN 并行**（建议 .b1=H1+H2 合并的「biome 通道双臂 dump」、.b2=H3+H4 合并的「阶段剥离 dump」；两组并行、组内收敛）。

## 5. 判别实验设计（decisive probe；动态验证均「主会话执行」）

### 5.0 第 0 步（前置，无它一切免谈）：提取残 9 逐点坐标
```powershell
# 在 verify_surfacefix_260904-10.py 基础上加逐点导出（.tmp/p2full/ 下新建 residual12_points.py）：
# 对 ref<->off-surfacefix 全量 cmp 中 va=gravel && vb=sand 的点输出 "x y z" 到 .tmp/p2full/residual12-points.txt
python .tmp/p2full/residual12_points.py   # seed 三查：两 .blocks 文件名内嵌 seed 8576294172403134396 一致才继续
```

### 5.1 探针 A（判 H1/H2 vs H3/H4，一次 dump 双答）
扩展 `WorldgenRust/src/bin-diag/res76_probe.rs`（或新 bin-diag probe）对 9 点输出：
`surf_biome_zoom`（biome_at_surface 判定）、`pick_cell`（px,py,pz）、`surf_applied`（surface 规则 applied）、`final`（全管线后）、`6d_params`（该 cell 的 t/h/c/e/d/w）。
主会话命令模板：
```powershell
cargo build --release -p WorldgenRust --bin res12_probe
# 运行（seed=8576294172403134396，与参照一致——seed 三查第一查）
.\WorldgenRust\target\release\res12_probe.exe --seed 8576294172403134396 --points .tmp/p2full/residual12-points.txt
```
C++ 对照臂（pick_cell/biome 权威）：复用 WG_BIOMEDUMP / biomeCellKey 诊断路径输出同 9 点。**判读**：
- pick_cell Rust≠C++ → H1；pick 同但 biome 名不同 → H2；pick 同 biome 同但 final=sand ≠ surf_applied=gravel → H3/H4。
- `6d_params` 落 deep_ocean/deep_lukewarm 边界距离 <参数分辨率 → H2 强佐证。

### 5.2 探针 B（判 H3/H4 内部：阶段剥离）
```powershell
$env:WG_SKIP_FEATURES=1   # 或 FLAG_SKIP_FEATURES
# 重导同 seed 同范围 blocks → 与 off-surfacefix-260904-10 对比：
# 残 9 消失 → H3/H4 坐实（feature 写 sand）；不变 → 回 H1/H2。
```
WG_SKIP_FEATURES 会连带去 ore/spring——残 9 判读只看 g2s 计数是否归零即可，不受干扰。

### 5.3 探针 C（H4 专项，若 5.2 坐实）
对 9 点所在 chunk 打 `cur_biome_id`（chunk 角 y=0 无 jitter）与 Java 3×3 section biome 对比（Java 臂用既有 biome dump 工具），验证「chunk 角判差 → disk_sand 开关翻转」。

### 5.4 期望签名（用于三查）
seed=8576294172403134396；9 点 ref=gravel / off-surfacefix=sand；区域先验在 chunk(14,12) 一带（旧 76 采样点 (236-245, 192-249) 邻域，**未验证假设**——残 9 可能不在同区域）。

## 6. 已验证事实 vs 未验证假设

**已验证事实（有落盘证据）**：
1. 76→12 decisive 数字与 g2s 9 计数（verify-surfacefix-260904-10.out.txt，seed 三查已过）。
2. vanilla floor 分支结构 = biome 三值切换 gravel/sand（VanillaSurfaceRules.java L263-270）。
3. biome 采样 = 精确块坐标 + BiomeAccess zoom（MaterialRules.java L462-464 + ChunkRegion.java L102）。
4. Rust 工作区 4 站点（surface/carver/features）zoom+hashSeed 均已接；surface 规则结构与 Java 1:1（本报告 §3，文件行号实测）。
5. 旧 76 g2s 形态 = biome 判差驱动（res76-probe CSV）。

**未验证假设（不得当公理）**：
1. 残 9 点坐标/分布（数据未提取，§5.0）。
2. H1~H4 全部候选（无任何动态证据）。
3. 残 9 与旧 76 主族同区域、同机制（先验 a）——scout 静态证据反而提示 **H3/H4（feature disk）与 H1/H2 同权重**，先验 (a):(b) 建议按 50:50 起步。
4. 「carver/features 已接 hashed seed」是否已提交（工作区 vs HEAD）——verdict §5 的 open 疑点描述与当前工作区代码不符（疑似 ab56706 后又一 commit 已修）；**主会话 `git log --oneline -5 -- WorldgenRust/src/worldgen_handle.rs` 核对**，影响 H3/H4 判读基线。
5. biomesrc 重算式 biome（按需采样）与 Java chunk-storage 填充值完全同源（H2 的前提面）。

## 7. 待深入点 / 建议下一步

1. 主会话执行 §5.0 → 得 9 点坐标后立即 §5.1 探针 A（一轮双答）。
2. fan-out 前置：探针 A 数据回来后再分叉（避免无数据 fan-out）。
3. 顺手项：git log 核对 carver/features hashed-seed 修复 commit 是否入库（§6-4）。
