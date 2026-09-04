# b1 候选：Rust SURFACE 阶段是 OOB dirt/sand 写者（candidate 草稿，260904-06 fanout-writer）

> 状态：**draft**（静态代码分析 + 证据包数据推演，无运行时验证；验证分层 = Degraded 静态审查）
> 分析者：fan-out worker b1。只读分析，未执行任何命令。
> 引用格式：文件:行号，全部来自本 session 实读。

---

## 0. 结论一句话

**结构层面 b1 成立（高置信）**：stageMask=3 下 mod 臂能写 dirt/sand/gravel 的阶段只剩 Rust `build_surface`（NOISE/ore_vein/aquifer 只写 air/stone/water/lava/铜铁矿，features 被 bit1 跳过）。**机制层面（中置信）**：OOB dirt 的最省假说是 `SurfaceCondC`（above_preliminary_surface）输入分叉——Rust `surface_heights4`（worldgen_handle.rs L582-585，默认走独立重扫 est 路径 L563-579）与 C++ 参照臂的 preliminary surface 估计不一致，使 `block_y >= k + surface_depth - 8` 窗口（surface_rules.rs L269，**对上不封顶**）下移，让"列内每个连续 stone run 的顶部"都被 SurfaceDepth-floor 规则刷 dirt → 一列多点、簇厚 2-3、间距 = run 周期（实测 ~16）。**(244) 列 gravel→sand 单列**则是另一条输入分叉（biome_at）驱动 mr2(sand)/mr3(gravel) 二选一的旁证。16 周期本身的来源（列内 16 间距 run 结构 → 密度函数周期性）**未验证**，idk 标注。

---

## 1. 必做分析 ①：Y 遍历范围是否有 clamp？高 Y 段规则能否命中？

**结论：扫描上界 = heightmap+1（不是 world_top clamp），写入条件只看 `state == stone`，无显式 [min_y, world_top) 写入 clamp——但真正的高 Y 写入必须同时有「heightmap 高」+「那里有 stone」。**

代码事实：

- 扫描起点 `p = heightmap[idx] + 1`（surface_rules.rs L1259，L1271），heightmap 输入 = `cd.surface_height`（worldgen_handle.rs L556）。
- `cd.surface_height` 语义 = **绝对 y**（"首个 solid 的 y"，terrain.rs L239；填充逻辑 terrain.rs L268-281：自顶向下第一个 `d > 0.0` 的 y）——**不是 index**，排除 ±64 (min_y) 整体错位类 off-by-one。
- 循环 `while wy >= min_y`（L1279）只有下界；上界由起点决定。`y >= world_top_y` 处按 air 处理（L1280-1282）→ **不会写 world_top 以上**，但 min_y+height 以内、heightmap 以下的一切 stone 都在扫描域内。
- 写入唯一点 `state == default_block(stone)`（L1317）→ `rule.apply` → `col.at_mut`（L1335-1337）。**只要列在 y=216~295 是 stone（C++ 探针已证 12/12 OOB 样本 ref 侧全 stone，即 mod NOISE 同样给 stone），且规则命中，surface 就会写**。

高 Y 段哪些规则能把 stone 置换为 dirt：

- `SurfaceCondC`（L1048-1051 门控整个 mr9）：`block_y >= k + surface_depth - 8`（L269）——**对上不封顶**。k 来自 `surface_heights4` 四角 lerp2（L255-269）。k 一旦偏低（如 208~224），则 y≥216 全部满足 → 该列高处一切 StoneDepth-floor 命中点都参与置换。
- dirt 的规则来源（mr9 树内，均可在高处命中）：
  - `mr7` 末尾 `b("dirt")`（L753）：前置门 = `mc10 = Water{-6,-1,add_stone_depth:true}`（L612, L987-1019）——**无流体列 `fluid_height == i32::MIN` 时 WaterCond 恒 true（L93-95）**，即干燥列 mc10 免费通过；再过 `StoneDepth{add_surface_depth:true, ceiling:false}`（L998-1001，floor 带，厚度 ~surface_depth≈3-6 → 与簇厚 2-3 吻合）。
  - `mr`（grass_block/dirt seq，L620-623）作为多处 fallback。
- granite→dirt（195,255/271,246）：granite 是 NOISE 侧岩性吗？——注意 ore_vein 只写铜/铁（worldgen_handle.rs L541-551 无 granite 路径；上轮已证 Rust NOISE 块表无 dirt/sand/gravel），样本里 ref 的 "granite" 应来自 C++ 探针的全量链路（其 surface/各自阶段），mod 臂在同位置把 stone 基底写成了 dirt。方向与「mod 多写 dirt」一致。

**判定：surface 引擎本身无越界写 bug（不写 top 之上、只写 stone 之上），OOB dirt 必然 = 规则谓词在「本不该命中的 y」命中 → 指向谓词输入（heights4/biome/stone_depth）分叉，而不是遍历范围泄漏。**

---

## 2. 必做分析 ②：(195,199) 列 y≥216 周期 ~16 簇形态解释

### 2.1 输入链路（heightmap / heights4 从哪来）

- heightmap：`fill_chunk` 宏观产物 `cd.surface_height`（worldgen_handle.rs L556 ← terrain.rs L281）。
- `surface_heights4`：worldgen_handle.rs L582-585，四角坐标 `(cx*16, cz*16) / (+16,0) / (0,+16) / (+16,+16)`——对齐 Java `chunkToBlockCoord(i+1) = (i+1)<<4`（L580-581 注释，260903-13 修过 +15→+16 的 off-by-one，#25）。
- 每角 est：`est_at`（L563-579）**默认走独立重扫路径**（`WG_EST_SHARED` 默认关，L563）：`y` 从 `min_y+noise_height`(=320) 起步长 8 向下，`self.init.sample > 0.390625` 首个 y（L571-577）。
- 消费点：`surface_cond_c_test`（surface_rules.rs L255-270）：`k = lerp2(fx, fz, e[0..3]).floor()`，条件 `block_y >= k + surface_depth - 8`。

### 2.2 16 周期机制假说（本候选的核心推演）

`SurfaceCondC` 窗口对上不封顶（L269 只设下界）→ **k 确定后，列内每一个「连续 stone run 的顶部」都满足 StoneDepth-floor**（`stone_depth_above = q`，L1312-1315：run 顶 q=1，逐块 +1；`q <= 1 + surface_depth + k`，L100-108）→ 若列内 stone 被周期性 air/fluid 打断成 16 间距的 run，则 dirt 恰好刷在每个 run 顶 → **多点、周期、每点厚 2-3（surface_depth 带）**。

实测簇 y 数据核对（evidence-pack L24）：

| 簇 | y | y mod 16 |
|---|---|---|
| 1 | 216 | 8 |
| 2 | 229,232 | 5, 8 |
| 3 | 246,247,248 | 6,7,8 |
| 4 | 262,263,264 | 6,7,8 |
| 5 | 278,279,280 | 6,7,8 |
| 6 | 294,295 | 6,7 |

**y ≡ 6..8 (mod 16) 高度自洽**：即列内 y≡9 起存在 ~8 厚的间断（air 或 aquifer fluid——fluid 不重置 q 但打断 s/run 结构，air 重置 q，L1287-1296），run 顶稳定落在 ≡6..8。且 (195,255,246) 与 (195,271,246) 的 granite→dirt 同样 +16 重复，同一周期。16 = 2×density y-cell(8)，密度函数在 d∈(0, 0.39] 段（stone 但低于 est 阈值 0.390625）做周期性穿越是形态学上自洽的。

**为什么 vanilla 没有**：vanilla 该列 y≥216 非 stone（vanilla 表面 ~200，dirt @184/200，evidence-pack L23）——vanilla 的 run 结构里根本没有这些高位 run。**即 mod（和 C++ 探针臂）NOISE 在此列 y≥216 产出了 vanilla 没有的 stone**，Rust surface 在其上合法地刷了 dirt。⚠️ 这意味着证据包里可能叠加了第二个问题（NOISE 阶段高位 stone 过量，C++ 与 mod 共有）——它不属于 b1（写者）范畴，但 judge 收敛时必须分离：**「谁写了 dirt」= surface；「为什么那里有 stone 可写」= 另一条线索**。

**为什么 C++ 参照同位置是 stone 而非 dirt**：同一 stone run 顶，C++ surface 不刷 dirt → 两臂 `SurfaceCondC` 判定分叉（k 值不同）或 C++ 树形/参数差。最廉价分叉源：Rust est 默认独立重扫路径（`self.init.sample`，L574）vs C++ 的 preliminary surface 实现——**若 `self.init` 含 jaggedness 项或步长/边界差 ±8，k 平移 → 窗口下缘平移 → 恰好跨 216 边界的 run 翻转 dirt/stone**。（⚠️ 未读 `self.init` 的构造，无法确认是否含 jaggedness——idk，见 §5。）

### 2.3 索引换算 off-by-one 排查（任务提示点）

- heights4 角坐标 +16 已修（L580-585，#25 第三例），现值对齐 Java。
- heightmap 索引 `idx = l*16 + k` = z*16+x（L1257），steep 读同序（L240-250），内部一致。
- est 扫描域 `min_y+noise_height .. min_y` 含下界、步 8（L571-577），注释称对齐 Java NoiseChunk.computePreliminarySurfaceLevel（260903-13 judge A1 修过 319→320）。
- **未发现现存 off-by-one**；本候选主张的分叉是「est 输入值分叉」（Rust init 采样 vs C++ 实现），非索引错位。

---

## 3. 必做分析 ③：OOB 样本点逐点推算

chunk 换算：column (x,z)=(195,199) → chunk (12,12)，(lx,lz)=(3,7)，heightmap idx = 7*16+3=115。
column (244,244) → chunk (15,15)，(lx,lz)=(4,4)。column (211,231) → chunk (13,14)，(lx,lz)=(3,7)。

**heights4 索引与角**：列 (195,199) 在角 (192,192)-(208,208) 的 lerp2 内：fx=(195&15)/16=3/16，fz=(199&15)/16=7/16；k ≈ 双线性(e00,e10,e01,e03)（surface_rules.rs L260-262）。k≈208~224 时窗口下缘 = k+surface_depth-8 ≈ 203~222 → 恰覆盖 216 首簇、不覆盖 200 以下 → **与「簇从 216 开始」定量自洽**。

「surface 判定输入读错位置」能否解释「本该 stone 的位置被置换」：

- 方向核对：样本 ref(C++)=stone / mod=dirt = **mod 多写**。机制上要求 mod 臂 SurfaceCondC=true 而 C++ 臂=false → est 输入 k 差 ≥ 窗口带宽即可，成立。
- (244,-60..-54,244) gravel→sand 单列：gravel/sand 都出自 ceiling 段 `mr2(sand)/mr3(gravel)`（L624-637），选择器是 biome 条件（mc14 beach/warm_ocean→sand、mc15 desert→sand，L673-674）vs stony_shore/windswept_gravelly→gravel（L656-665, L731-747）。**mod 在该列选了 sand 分支 = biome_at 输入分叉**（biome_at 量化 (x>>2)<<2，worldgen_handle.rs L602-605）。这是 surface 写者框架下的第二类输入分叉（biome 而非 est），与 b1 不矛盾——**b1 的主张精确化为「Rust SURFACE 用分叉的输入在多写」**。

---

## 4. 五段式证据链

| 段 | 内容 |
|---|---|
| **现象** | stageMask=3 双跑后 OOB stone→dirt/sand 未消失（43 HI + 11 LO，evidence-pack L17）；(195,199) 列 y≥216 mod 独有周期 ~16 dirt 簇；(244,-60..-54,244) 单列 gravel→sand |
| **根因（机制，候选）** | ① 写者消去：mask=3 下唯一能写 dirt/sand/gravel 的 mod 阶段 = Rust build_surface（NOISE→air/stone/water/lava，terrain.rs L277-278；ore_vein→铜铁，worldgen_handle.rs L548-551；aquifer→water/lava，terrain.rs L232；features 被 bit1 跳过）。② dirt 多写机制 = SurfaceCondC 窗口（对上不封顶，surface_rules.rs L269）× 列内多 run 顶 × est 输入（heights4 默认独立重扫路径，worldgen_handle.rs L563-579）两臂分叉。③ sand/gravel 二选一 = biome_at 输入分叉 |
| **定位（怎么找到的）** | 静态：surface_rules.rs L269（无上界）+ L93-95（无流体 Water 恒真）+ L100-108（floor 带厚度）→ 「一列多 run 顶多簇」唯一自洽形态；簇 y mod 16 ∈ {5..8} 定量自洽；worldgen_handle.rs L608-612 证 mask=3 SURFACE 仍在跑 |
| **修复（未做——本候选只定位，修复待 judge 收敛后）** | 候选修复方向：est 路径对齐 C++（或默认 WG_EST_SHARED=1 验证）；biome_at 高位 y 采样对齐 |
| **教训** | 「OOB 写」先分「写入域泄漏」vs「谓词在异常输入上合法命中」——本例是后者；且双跑消去 features 后，「写者」消去链要数尽所有能写目标 block 的阶段（含 ore_vein 块表核对） |

## 5. 自检清单

- [x] 引用全部带文件:行号，未读不引（`self.init` 是否含 jaggedness **未读** → 显式 idk，不做断言）
- [x] 置换方向核对：ref stone / mod dirt = mod 多写（非「mod 少删」）
- [x] stageMask=3 下 SURFACE 确认仍在跑（worldgen_handle.rs L609-612 只查 bit2）
- [x] 16 周期来源（列内 run 结构的成因）**未验证** → idk：为何密度函数在该列呈 16 周期穿越 d∈(0,0.39]（cell 高 8 的倍频？jaggedness？）——需 §6-E4 数据
- [x] 证据包内已发现的前后矛盾已标注：C++ 探针列全 stone（y≥216）+ vanilla 该高度非 stone → **叠加嫌疑：C++/mod 共有的 NOISE 高位 stone 过量**（非 b1 范畴，交 judge 分流）
- [x] 置信度 draft；confirmed 留给人类
- [ ] 运行时验证：未做（worker 无 shell，命令模板见 §6 交主会话）

## 6. 可判定实验（命令模板，交主会话执行；worker 只解读原始输出）

| # | 实验 | 命令模板 | 判据 |
|---|---|---|---|
| E1（**决定性：写者消去**） | mod 臂 env 跳过 Rust SURFACE 后重导 A 域 | mod 导出命令不变，加 env：`$env:WG_SKIP_SURFACE="1"`（worldgen_handle.rs L609 已支持）再导 (200..204)² | OOB dirt/sand/gravel **全部消失** → b1 写者确证；仍在 → 写者另有其人（b2 方向），b1 被推翻 |
| E2（决定性：est 输入分叉） | est 路径 A/B | 同上但改 `$env:WG_EST_SHARED="1"`（L563）重导 | y≥216 dirt 簇消失或位移 → heights4/est 输入分叉确证；不动 → est 无嫌疑，转 biome/规则树差 |
| E3（定点规则分支） | 生产 ctx dump 命中点 | 点文件 `.tmp/b1-points.txt` 写行 `195 216 199` / `195 229 199` / `195 246 199` / `244 -58 244`，`$env:WG_SOUL_CTX_DUMP=".tmp/b1-points.txt"`（surface_rules.rs L466-487）跑单 chunk | stderr `[SOUL-CTX]` 行给出 biome/sda/sdb/surface_depth/fluid/applied → 直接看是哪条规则、什么输入命中 |
| E4（16 周期来源） | 该列 NOISE-only 纵剖面 | `$env:WG_SKIP_SURFACE="1"` 导单 chunk 后脚本打印 (195,199) 列 y∈[200,320] 块序列；同点跑 C++ NOISE-only（block_probe 关 surface 开关，如有）与 vanilla 对照 | 确认「高位 stone + 16 周期 run 结构」在哪一臂产生（mod-only / C+++mod 共有 / 三方差异） |
| E5（heights4 数值） | est 四角 dump | `$env:WG_EST_DUMP=".tmp/est-dump.csv"`（L586-601）跑 chunk(12,12)、(15,15) | 与 C++ preliminary surface 估算对比（若 C++ 无对应 dump，用 E3 的 k 反推）；k 期望 ~208-224（§3 推算） |
| E6（biome 分叉） | (244) 列 biome 对比 | WG_BIOMEDUMP / -biomeDump 定点 (244,-60..-54,244)，Rust vs C++ 各一次（注意三套坐标口径，AGENTS.md 探针核对铁律） | biome 不一致 → mr2/mr3 选择差解释 gravel→sand |

原始输出按命令委托契约落 `.investigations/lossless-accel/fanout-writer-260904-06/cmd-output/`，回传 worker 解读。

---

## 7. 置信度声明

- **写者身份（Rust SURFACE 写了 OOB dirt/sand/gravel）**：**E1 后升级：运行时确证（写者层面）**——E1 臂（WG_SKIP_SURFACE=1）(195,199) 列 dirt/sand/gravel = 空（cmd-output/e1-skipsurface-result.txt L22），dirt 随 surface 关闭消失。
- **机制（SurfaceCondC est 输入分叉 + run 顶多簇）**：draft 级（部分更新见 §8；E2/E3 可判定）。
- **16 周期成因**：idk 部分收敛——E1 显示 C++ 参照臂同区域（列 195,198）也有 ~16 间距高位 dirt 簇 → run 结构在共享 NOISE 基底上，非 Rust 特有。
- 本文档 = candidate 草稿，永不自标 confirmed。

---

## 8. E1 运行时验证回填（260904-06，主会话执行，worker 解读）

**原始数据**：cmd-output/e1-skipsurface-result.txt（stageMask=3 + WG_SKIP_SURFACE=1，dll ec4a9aed…，seed 三查过）。【judge C1 补正】skip 生效证据 = **行为证据**：①(195,199) 列 dirt/sand/gravel 全清空；②vs C++ 臂 OOB 族方向翻转（243/111，C++ dirt 留 E1 stone）——log `[Mixin] buildSurface skipped` 是 mixin 常规 cancel 日志（NoiseChunkGeneratorMixin.java L95，各臂均打），**不作为** WG_SKIP_SURFACE 证据。改进建议：Rust skip 分支（worldgen_handle.rs L609-626）加独立一次性日志（如 [SURFACE-SKIP] handle=… mask=…）。

### 8.1 判据核对：写者消去 ✅ 确证

- E1 臂 (195,199) 列 dirt/sand/gravel = **空**（L22）——mod 原臂该列的 mod 独有 dirt 簇（216/229-232/246-248/262-264/278-280/294-295）全部随 Rust surface 关闭而消失 → **该列 OOB dirt 的写者 = Rust build_surface，运行时确证**（E1 判据满足「全部消失」分支）。

### 8.2 243/111「反向增长」口径解释（⚠️ 不可比，声明 §9.7）

- 方向翻转：本次 ra=E1 臂（无 Rust surface）/ rb=C++ NOISE+SURFACE → n=243(HI)/111(LO) 计的是 **C++ surface 写了 dirt 而 E1 臂留 stone** 的点（样例行 L2-17 全部 ref 侧 dirt / E1 侧 stone），**不是 E1 臂多写**。
- 口径三要素不可比（§9.7）：① 载体变了（E1=Rust NOISE-only vs 原 A=mod FULL 含 surface）；② 差异方向反了（原 43/11 = mod 多写；本次 = C++ 多写）；③ 两臂同时差「surface 有无」+「surface 实现」两个维度。**243/111 不与 43/11 同口径对比，不构成矛盾**。

### 8.3 机制更新

1. **C++ surface 也在高位刷周期 dirt**：HI 样本 y=214-215, 230-231, 247, 263, 279-280, 293-296, 309-312（列 195,**198**）——同样 ~16 间距 run 顶形态。→ 「16 周期 run 结构」在共享 NOISE 基底上（C++ NOISE 同样给高位 stone），**非 Rust 伪影**；强化 §2.2 的「NOISE 高位 stone 过量为 C++/mod 共有叠加问题」flag。
2. mod-vs-C++ 真差异（原 43/11）收窄为：**同一 stone run 顶，Rust surface 刷 dirt 而 C++ surface 不刷（或反之）的列级/点级翻转**——与 est/biome 输入分叉机制一致（相邻列 198 vs 199 两臂各刷各的，正是 k 窗口/输入平移的表现形态）。
3. granite：E1 样本 ref 侧出现 granite（L5-6 等）→ granite 是 C++ 探针链路自身产物；mod NOISE 块表无 granite（上轮已证），置换方向陈述不变。

### 8.4 下一步实验建议

- **E3（最高优先）**：WG_SOUL_CTX_DUMP 定点 `(195,216,199) (195,246,199)`（Rust 臂命中点）+ `(195,247,198) (195,263,198)`（C++ 臂命中点对照）→ 直接看同一 run 顶两臂 biome/surface_depth/stone_depth/applied 差 → 判定 est vs biome 哪条输入分叉。
- **E2（仍需，廉价）**：WG_EST_SHARED=1 重导 → dirt 簇位移/消失即锁定 heights4/est 路径。
- **E5**：WG_EST_DUMP 取 chunk(12,12) 四角 k 值，与 §3 推算窗口（k≈208-224）核对。
- E4/E6 降级为可选（E4 的「NOISE 高位 stone 共有」已被 E1 间接证实一半；E6 待 E3 结果后决定）。

---

## 9. E2'/E3/E5 回填 + 机制修订（260904-06，worker 解读；含 §15.4 取代记录）

> 原始数据：cmd-output/e2-estshared0-result.txt、cmd-output/e3-e5-run-full.log、.tmp/p2full/e3-est-dump.csv。

### 9.0 取代记录（supersedes，原文不改，按 §15.4）

| id | 被取代结论（原文位置） | 取代结论 | 理由 |
|---|---|---|---|
| R1 | §8.4「E2：WG_EST_SHARED=1 反转开启」——默认关（§2.1 对 L563 的解读） | **est shared 默认开，`=0` 才关**（env_enabled 语义；worldgen_handle.rs L511-515 注释模式佐证）。主会话已按修正口径执行 | 源码 L559 注释 + 主会话核对 |
| R2 | §3「k≈208~224，窗口下缘 203~222，与首簇 216 定量自洽」 | **实测四角 est=24（chunk 12,12；dump 值 = 绝对 y，worldgen_handle.rs L591 直印 surface_heights4[i]）** → k≈24，SurfaceCondC 窗口下缘 ≈ 24+sd-8 ≈ 17 → **y≥17 全部在窗口内**。原推算作废；机制方向不变、数值基础更换（见 9.3） | E5 实测 |
| R3 | §2.2/§8.4「est 路径选择（shared/独立）是两臂分叉候选源」 | **证伪**：E2' 关 shared 后导出 sha256 与 off 逐字节相同（e2 L1），本区域零语义差 → 分叉不在 Rust est 路径选择 | E2' |
| R4 | §3「(244) gravel→sand = biome_at 输入分叉（mod 侧）」 | **修订（待定）**：E3 实测 mod 该 cell biome=deep_lukewarm_ocean，而 deep_lukewarm_ocean 在 final 段命中 mr2(**sand**)（surface_rules.rs L1030-1033）→ **mod 的 sand 对其 biome 是正确输出**；ref 侧 gravel 暗示 C++ 探针臂该 cell 的 biome/分支不同 → 更像**参照（探针）侧差异**而非 mod bug。但 b2 报告称两臂 biome 全同——张力未解，见 9.4-(c) | E3 + b2 结论冲突 |

### 9.1 E2' 解读

- sha256 相同（2ff71249…）+ (195,199) 列零差异 → **Rust est 两路径语义同一**（与 worldgen_handle.rs L23「四臂 hash 零语义差」一致）。est 路径选择退出嫌疑。
- E2 臂全列口径 y>200 dirt/sand n=4618 —— 两臂（off/e2）都在高位大量刷 dirt：mod surface 在「stone 幕帘」上大规模正常作画，坐实「幕帘共享、差异仅边界翻转」框架。

### 9.2 E3 解读

- 唯一输出行 `244,-58,244, biome=deep_lukewarm_ocean, sda=48, sdb=7, surface_depth=1, fluid_height=MIN, applied=id=970`：
  - biome 与 mod 该列 sand 输出自洽（deep_lukewarm_ocean → mr2 sand，L1030-1033）；
  - ceiling_ok/floor_ok 均 false 而 applied≠none → 命中的是**无条件 fallback 或 vertical_gradient 段**（y=-58 ≤ true_y 0 → deepslate gradient 必真，L1052-1056；id=970 疑为 deepslate/sand，**需 id→name 表核对**，9.4-(a)）；
  - sda=48 说明 q 跨水层连续计数（fluid 不重置 q，L1290-1293；仅 air 重置 L1287-1289）：**海洋列的 stone_depth_above 是「跨水累计」**——floor 谓词在深处难命中，但无条件 fallback 规则仍作画。
- **195 列 4 点零输出判读**（挂点核对：钩子在 `state == default_block` 分支内，surface_rules.rs L1317-1334，点集命中即 dump，无其他门）：零输出 = mod 扫描在这些点**要么 state≠stone（air/water），要么扫描起点 heightmap+1 低于该 y（未到达）**。与「mod 在 216+ 有 dirt」表面矛盾 → 两种可能：①E3 运行链路与原 dirt 产生轮不同（幕帘边界微移/dll 代次差）；②原 mod dirt 点存在坐标口径差。**不可臆断，需 9.4-(b) 裁决。**

### 9.3 机制修订（E5 之后的世界图景）

est=24 语义：preliminary surface（阈值 0.39）落在 y≈24，而 NOISE 在 y≈214-312 仍有 d∈(0,0.39] 的 **stone 幕帘**（fill 的 solid 判据是 d>0，terrain.rs L279）→ **幕帘整体位于 SurfaceCondC 窗口内（k≈24 ≪ 幕帘 y）** → 两臂 surface 都会在幕帘 run 顶刷 dirt（E1 的 C++ 侧 243 点、E2 臂 4618 点均此形态）。于是：

- **原 43/11「OOB」本质收窄为：两臂在同一幕帘上的作画边界微差（43 点 / 数万幕帘点）**——不是 mod 特有病理。
- **对 vanilla 的真根因上移到 NOISE**：vanilla 该区域 y≥~201 无 stone（vanilla 表面 ~200），而 C++/Rust NOISE 都给出 d>0 幕帘 → **「0<d≤0.39 幕帘」是 C++ 与 Rust 共有的 NOISE↔vanilla 分叉，是比 surface 写者更大的靶子**。与上轮「Rust NOISE 对齐 C++」不矛盾——两者可能一起偏离 vanilla（C++ 探针与 vanilla 未在幕帘区逐位对比过）。
  > ⚠️ **已被取代（260904-06 幕帘线）**：「vanilla y≥201 无 stone」为参照误读（P4 实测 131 stone 族块），幕帘 = vanilla 正常机制（aquifer barrier margin，d 实测全负）；本段推论作废。见 `.artifacts/lossless-accel/curtain-verdict-260904-06.md`。

### 9.4 剩余分叉与下一步（收敛建议）

剩余候选（针对 43 点翻转 + 零输出谜题）：
- (m1) **NOISE 幕帘边界微差**（C++ vs Rust 在 d≈0 / d≈0.39 阈值附近的 Rock/Air 分类差 → run 顶不同）——当前最强。
- (m2) C++ preliminary surface 语义差 vs Rust est（阈值/步长/jaggedness）——R3 削弱了「Rust 内部路径」版本，但「Rust vs C++ 实现」版本未排除。
- (m3) (244) 参照侧 biome/探针差——R4。

**建议不做 b3 全轮 fan-out**（已从「写者是谁」收敛到「边界微差在哪」，属收敛型追踪），改做四个廉价定点动作：
- (a) **id 970 查表**：BlockRegistry id→name（主会话一行脚本）→ 锁定 E3 命中分支。
- (b) **E4-lite 列剖面**：WG_SKIP_SURFACE=1 打印 (195,199) 与 (195,198) 两列 y∈[-64,320] 块序列 + 每列 heightmap → 同时裁决幕帘结构、两列差异、E3 零输出之谜（扫描是否到达 216）。
- (c) **(244) 参照 biome 复核**：C++ -biomeDump 定点 (244,-60..-54,244) vs mod biome（注意坐标口径）→ ref≠deep_lukewarm_ocean 则 R4 坐实；同则查 NoiseThreshold("minecraft:surface") 采样点差（surface_rules.rs L148 y 强制 0.0，与 C++ 对照）。
- (d)（静态，可并行）读 C++ `worldgen/src/surface.h` preliminary surface 实现（阈值/步长/域）与 Rust est（worldgen_handle.rs L571-577）逐行对拍 → 直接裁决 (m2)，无需运行。

### 9.5 收敛结论（交主会话/judge）

- **「写者 = Rust SURFACE」可升 strong candidate**：结构消去链 + E1 运行时确证 + E2' 排除混淆变量。建议 judge 审查后授予 candidate。
- **课题重心应转移**：mod↔C++ 的 43/11 已定性为幕帘边界微差；**mod↔vanilla 真根因在 NOISE 幕帘（C++/Rust 共有）**——建议主会话立新调查线（「0<d≤0.39 幕帘的 NOISE↔vanilla 分叉」），b1 写者结论归档。
- 本节判读置信度 draft；数值全部来自主会话落盘原始数据，无编造。

---

## 10. (a)(b)(c) 裁决回填 + 最终 candidate 申报（260904-06，worker 判读）

> 原始数据：cmd-output/e4lite-id970-columns.txt、cmd-output/cpp-biomedump-244col.txt。

### 10.1 取代/更新记录（续 §15.4 链）

| id | 内容 | 依据 |
|---|---|---|
| R5 | §9.2「applied id=970 疑 deepslate/sand」→ **确定 = deepslate**（blocks.json 唯一映射；y=-58 ≤ true_y 0，deepslate gradient 必真，surface_rules.rs L1052-1056）→ E3 唯一输出行完全合理，钩子本身工作正常 | (a) |
| R6 | §9.4-(c) R4「参照侧 biome 差」→ **证伪**：C++ -biomeDump 七点全部 = deep_lukewarm_ocean，与 mod 一致 → (244) gravel→sand 的分叉在 **C++ surface 规则分支**：final 段 StoneDepth-floor 的 warm-ocean 条目（surface_rules.rs L1030-1033 → mr2 **sand**）在 C++ surface.h 未正确命中，落到 mr3 **gravel** fallback。**按 vanilla 语义（Java VanillaSurfaceRules：deep_lukewarm_ocean 地板 = sand），mod 的 sand 才是对的——C++ 参照自身在此点偏离 vanilla** | (c) |
| R7 | 16 周期来源部分收敛：**col(195,198) aquifer 水口袋 @197/212/228/244/261/277（间距 15/16/16/17/16）**；C++ HI dirt 点（214-215/230-231/247/263/279-280）**恰好全部在口袋正下方首个 stone** → C++ 臂的高位周期 dirt = 水口袋驱动的作画。**但 col(195,199) 高位无水口袋（y>190 仅 @198 一格水），mod dirt 簇（216/229-232/246-248/…）落在连续 stone/granite run 内部** → col(199) 的 mod 侧 16 周期触发输入仍未定位（biome 带 4 格？noise y=0 采样？），E3 复跑前不定论 | (b) |

### 10.2 E3 零输出矛盾复核（钩子逻辑核查 + 剩余解释；R8 已裁决）

| ID | 取代内容 | 理由 |
|---|---|---|
| R8 | §10.2 解释 #1「E3 未生成 chunk(12,12)」→ **证伪**：E3 复跑（e3-rerun-full.log，7 点扩列）chunk(12,12) 已生成且 surface 已跑（log L682），(195,216/217/218/230/246/247,199) 仍全部零输出，唯 (244,-58,244) 输出——与首轮模式相同 →「未生成」不再成立。剩余解释（钩子只在 (15,15) chunk 的执行路径生效 / OnceLock+多 worker 竞争 / 点坐标→chunk 归属在 dump 判定的映射差）**全部明示 idk**，随新调查线排查 | E3 复跑直接实证（judge C2） |

历史复核记录（#1 已被 R8 取代，#2/#3 降权）：
1. ~~E3 运行未生成 chunk(12,12)~~（R8 证伪）；
2. E3 运行链路/dll 代次与原 dirt 轮不同（幕帘边界微移致该点非 stone）——E4-lite 已证该点 pre-surface=stone，降权；
3. 点文件格式异常（与 244 行同格式，可能性低）。

### 10.3 最终 candidate 申报文本（交 judge）

**候选标题**：b1 — Rust SURFACE 阶段是 OOB dirt/sand 写者（部分 OOB 亦是 C++ 参照自身的 vanilla 偏离）。

**写者结论（strong candidate，申报 candidate 授予）**：
1. stageMask=3 下 mod 臂能写 dirt/sand/gravel 的阶段唯一 = Rust `build_surface`（结构消去：NOISE/ore_vein/aquifer 块表核实、features 被 bit1 跳过）。
2. E1 运行时确证：WG_SKIP_SURFACE=1 后 (195,199) 列 dirt/sand/gravel 清空（e1 L22）。
3. E2' 排除混淆变量：est 路径选择零语义差（sha 逐字节同）。

**机制现状（draft，两臂作画边界差 + 参照侧缺陷）**：
4. 世界图景：NOISE 在高位存在 d∈(0,0.39] stone 幕帘（E1 臂最高 solid y=318；est=24 ≪ 幕帘 y → 幕帘整体在 SurfaceCondC 窗口内）→ 两臂 surface 都在幕帘上正常作画；43/11「OOB」= 两臂作画点集的边界微差。
5. col(198) 臂差异点由 ~16 间距 aquifer 水口袋驱动（C++ 在每个口袋下方 stone 刷 dirt）；col(199) mod 侧 16 周期触发输入未定位（idk）。
6. (244) gravel→sand：biome 两臂一致（deep_lukewarm_ocean），C++ 写 gravel = C++ surface 规则分支缺陷（R6），**mod sand 疑为 vanilla 正确值**——建议对 (244,-60..-54,244) 做一次 vanilla 列对照定案（新线首实验即可捎带）。

**移交建议**：
- 本候选（写者身份）交 judge 审查授予 candidate 后归档；剩余机制细项（col(199) mod 16 周期触发输入、E3 复跑、(d) preliminary surface 对拍）随新线继续。
- **立新调查线：「NOISE stone 幕帘（0<d≤0.39）的 C++/Rust 共有 vanilla 偏离」**——这是 mod↔vanilla 差异的真根因层，优先级高于 surface 侧残余微差；首动作建议：vanilla vs C++-NOISE-only vs Rust-NOISE-only 三方在该区域列剖面逐位对比（含 density 值采样，定位 d 阈值穿越差异）。

**置信度**：写者 = strong candidate；机制细项 = draft/idk（已逐条标注）。本文件由 fan-out worker b1 产出，never self-confirm。
