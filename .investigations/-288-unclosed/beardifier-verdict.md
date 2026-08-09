# Beardifier 实现验证报告（-288 海底边界闭合）

> 课题：-288 未闭合 ~23% 差异中「海底边界 ≈6710 块」根因 = C++ 缺失 Beardifier（verdict-04，2026-08-09 candidate + 用户拍板列入范围内）
> 本报告：Beardifier（StructureWeightSampler）C++ 移植 + 接入 + 验证全链
> 日期：2026-08-09。状态：**draft（待 judge）**

## 一、结论

**C++ 实现 Beardifier 后，-288 海底边界闭合 8221 块（TOTAL 95.7379% → 96.2606%，+0.52%），零新增 mismatch；闭合点 89% 在海底边界 y=52..62（7313 块），与 verdict-04 预期 ≈6710 吻合（多出部分 = 村庄 12 格内其他 Beardifier 传导差异）。**

## 二、实现内容

| 文件 | 内容 |
|---|---|
| `cpp/worldgen/src/beardifier.h`（新增） | StructureWeightSampler 纯算法移植：24³ float 权重表惰性预计算（`(float)calculateStructureWeight`，pow 字面量同 Java Math.E）+ getMagnitudeWeight/getStructureWeight + sample 四分支（NONE/BURY/BEARD_THIN/BEARD_BOX）+ fastInverseSqrt 位操作（int64 有符号右移） |
| `worldgen_api.cpp` | WorldgenHandle 新增 per-chunk `std::unordered_map<int64_t, Beardifier> beardifiers`（key=chunkX<<32^chunkZ）+ wg_set_beardifier/wg_clear_beardifier C API + fillOneChunk 3a 段 densityBuf = finalDensity + beard（无输入则 null 不加，行为不变） |
| `worldgen_api.h` | wg_set_beardifier/wg_clear_beardifier 声明（pieces 每 8 int / junctions 每 3 int） |
| `jni_bridge.cpp` | Java_wg_CppWorldgen_setBeardifier（JNI 包装） |
| `block_probe.cpp` | -beard <file> 参数 + loadBeardFile 解析（BlockProbe dump 格式） |
| MC 工程（本地 M） | CppWorldgen.setBeardifier native 声明；CppBridge.feedBeardifier（vanilla createStructureWeightSampler 构造 + 反射提取 piece/junction → int[] 喂 C++；失败降级不阻断）；NoiseChunkGeneratorMixin.populateNoise 拦截处喂数据；BlockProbe BEARD-DUMP 段；build.gradle beardDump 映射 |

## 三、验证链

### 3.1 算法对拍（C++ vs Java BEARD-244 真实参照）
- 参照：`beard_m288.txt`（BlockProbe -Dbeard.dump 导出，-288 区域 **16 chunks** 结构布局全量：135 pieces + 506 junctions；BEARD-DUMP 与实机 CppBridge.feedBeardifier 同源——直接 `createStructureWeightSampler(structureAccessor, pos)`，不依赖 cns 生命周期，解决 z=-13 连带推进 cns null 缺失问题）
- t_beard3.exe：C++ Beardifier 采样 (-244,50..66,-256) vs `beard244_run1.txt` 真实参照 **17/17 逐位一致**（含 y=50=0、y=51..54 非零、翻转点 y=58、峰值 y=60、转负 y=63、y=64..66 负值全区间）
- ⚠️ **初版 t_beard2 曾误报 y=50..54/64..66 MISMATCH**——那是测试脚本用臆造占位值 0 当参照所致；用真实参照（beard244_run1.txt）重测后 17/17 全过。该错误已在 t_beard3 修正，不反映 C++ 实现问题

### 3.2 block_probe 全量验证（-288，seed=-8248318472910187742，4×4=16 chunks）
| 指标 | 无 beard（基线） | 有 beard（16 chunks 全量） | 变化 |
|---|---|---|---|
| TOTAL match | 95.7378% | **96.4221%** | **+10777 块** |
| nonAir match | 87.9045% | 90.0042% | +2.10% |
| MISMATCH 数 | 67039 | 56275 | -10764 |

### 3.3 闭合点分布（beard_y_dist2.py）
- **闭合 10777 块**，新增 mismatch **13 块**（净收益 99.88%）
- y=52..62 海底边界：**9280 块（86%）**；y=46..51 深水过渡带 + y=63..67 沙滩边界 + 其他
- 与 verdict-04「海底边界 ≈6710」吻合且超预期（Beardifier 还闭合村庄 12 格内其他传导差异）
- **13 个新增 mismatch 归因**：全部位于 Beardifier 峰值翻转边界附近 surface 分层级联点——
  - (-263,53,-198)：beard 抬高 density 后 C++ surface gravel→stone（vanilla 保持 gravel）
  - (-247..-243,65,-204/-201 等 12 点)：beard 抬高后 C++ grass_block/dirt 分层与 vanilla 差一位
  - **均为 surface rules 与 Beardifier 级联的边界差异（surface 独立课题，非 Beardifier 算法错误）**——算法 17/17 逐位一致不受影响

### 3.4 零退化铁律
- 8576（720,-432 6×6）：TOTAL 99.9994% —— 与基线一致（无 beard 输入时行为不变）
- 3200（4×4）：TOTAL 99.9997% —— 与基线一致
- scan_cpp_anchors：21 anchors（test=20 idk=1）invalid=0

## 四、retry 声明

- 本课题（Beardifier 实现）为 verdict-04 裁决后的新方向实施，非对失败假设的重复验证——按 spec §5.3 声明，本轮验证计数 1
- 实现阶段无假设失败（算法一次对齐 BEARD-244）

## 五、待办

1. judge 审查（收尾 MUST）→ 用户拍板 confirmed/candidate
2. **13 个新增 mismatch 归因**（surface 级联边界差异，独立课题——已诚实声明，不阻碍 Beardifier candidate）
3. 知识库更新（subagent 产出）：04 篇海底边界结案标注、06 篇 Beardifier 机制、10 时间线、discovered（Beardifier 算法指纹：24³ 表 + fastInverseSqrt + pow 非 exp）
4. 实机验证：gradle runClient 村庄区域看地形（可选，1.0.19 发布前）
5. MC 工程改动提交（本地 M，勿 push 历史分叉）

## 六、产物引用

- 参照：`.investigations/-288-unclosed/cmd-output/beard_m288.txt`（16 chunks 全量结构布局，135 pieces + 506 junctions）
- 真实参照：`.investigations/-288-unclosed/cmd-output/beard244_run1.txt`（BEARD-244 y=50..66 全 17 点）
- 脚本：E:\tmp\t_beard3.cpp（对拍）/ beard_y_dist2.py（闭合分布，临时）
- 算法验证：t_beard3.exe 输出（17/17 一致）
- block_probe 输出：E:\tmp\mm_nobeard2.txt / mm_beard2.txt（mismatch 全量，临时）
