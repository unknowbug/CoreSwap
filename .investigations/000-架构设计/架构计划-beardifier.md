---
编号: 000-beardifier
任务: Beardifier 移植——StructureWeightSampler 结构密度修正（C++ 实现 + 接入 density 链 + 闭合 -288 海底边界 ≈6710 块）
任务类型: 算法还原（Java → C++）+ 验证
模式档位: 轻量
状态: 待批准
日期: 2026-08-09
---

## 背景（已确认，verdict-04 candidate + 用户拍板列入范围内）

- **根因**：-288 海底边界 ≈6710 块 = C++ 缺失 Beardifier（StructureWeightSampler 结构密度修正）。Java density 链 = `CellCache.add(DensityInterpolator(finalDensity), Beardifier)`；C++ 只实现了 finalDensity 部分（worldgen_api.cpp L570 注释自认）
- **Java 机制**（已读源码确认）：
  - `NoiseChunkGenerator.populateNoise` → `doFill` → `StructureWeightSampler.createStructureWeightSampler(world, chunkPos)`（L106）
  - `ChunkNoiseSampler.getActualDensityFunction` L469-470：`Beardifier.INSTANCE` → 替换为真实 `beardifying`（StructureWeightSampler）
  - `StructureWeightSampler.sample(pos)`：对每个 Piece（bbox + terrainAdaptation + groundLevelDelta）+ JigsawJunction，按 BURY / BEARD_THIN / BEARD_BOX / NONE 分支累加权重；24³ 预计算 STRUCTURE_WEIGHT_TABLE
  - CoreSwap mixin 在 `populateNoise` HEAD 拦截 → C++ 接管，vanilla StructureWeightSampler 从未构造 → C++ 无 Beardifier
- **关键约束**：C++ 无结构系统（structure starts 由 vanilla 机制产生）→ **Beardifier 输入数据（piece/junction 列表）必须由 Java 侧喂入**（mixin 处 StructureAccessor 可用，与 vanilla 时机一致）

## 范围

**做**：
1. C++ 纯算法移植：`beardifier.h`（STRUCTURE_WEIGHT_TABLE 24³ 预计算 + Piece/Junction 结构 + `sample()` 全分支 + `getMagnitudeWeight`/`getStructureWeight` 逐位对齐）
2. C++ density 链接入：worldgen_api.cpp 补 `add(finalDensity, Beardifier)`（CellCache 语义）；句柄持有 piece/junction 数据 + `wg_set_beardifier` 接口
3. JNI 桥：`CppWorldgen.setBeardifier`（Java mixin 处构造 vanilla `StructureWeightSampler` → 提取 piece/junction → int[] 传 C++）
4. MC 工程 mixin 改动：`populateNoise` HEAD 拦截处，fillChunk 前构造 StructureWeightSampler 并喂 C++
5. 验证：block_probe -288 海底边界闭合（≈6710 块）+ 8576/3200 零退化 + scan_cpp_anchors invalid=0
6. 知识库更新（subagent 产出）：06 篇/04 篇追加 + 10 时间线

**不做**：
- ❌ 不移植结构生成器/StructureLocator（结构布局 = vanilla 机制喂入，C++ 不重写结构系统）
- ❌ 不实现其他 FEATURE（carvers/岩石替换/装饰层——独立立项）
- ❌ 不做 JigsawJunction 的生成逻辑（仅消费 vanilla 传人的 junction 数据）
- ❌ 不动 -288 的 gravel/表面规则剩余差异（独立子课题）

## 任务拆解（子任务 → 预期产物）

1. **算法移植**：`cpp/worldgen/src/beardifier.h`——24³ 表预计算（`calculateStructureWeight`→`structureWeight` 逐位）+ `sample()` 4 分支 + Piece/Junction 结构。产物：beardifier.h
   - 对拍锚点：verdict-04 BEARD-244 8 点（y=55..62 val 已知）+ `beard_ctx.py`（已有采样上下文脚本）
2. **density 链接入**：worldgen_api.cpp——handle 增加 beardifier 实例 + `wg_set_beardifier(handle, pieces, junctions)` + fill 路径 `add(finalDensity, beard)`。产物：worldgen_api.cpp diff
   - 对拍锚点：CellCache 等式 8/8（C++ finalDensity + Beard = Java AQF dCC ≤3e-6）
3. **JNI + mixin**：jni_bridge.cpp 新接口 + CppWorldgen.java + NoiseChunkGeneratorMixin（fillChunk 前构造 StructureWeightSampler → 序列化 piece/junction 喂 C++）。产物：MC 工程 diff（本地 M，勿 push）
   - ⚠️ 注意：`StructureWeightSampler.createStructureWeightSampler` 需要 `StructureAccessor`——mixin 参数里有（populateNoise 签名第 4 参）
4. **block_probe 验证**：
   - 参照：-288 区域（-288,-256 4×4 chunk）structure 布局 dump（Java 探针扩展导出 piece/junction 文件）
   - block_probe 支持 beard 输入文件 → 跑 -288 对比 → 海底边界闭合块数（目标 ≈6710）
   - 8576/3200 零退化（该区域应无结构 → Beardifier=0 → 结果不变）
5. **知识库更新（subagent 产出）**：04/06 篇 + 10 时间线 + discovered（Beardifier 机制/表驱动权重）

## 验证方式

- **分层 Full**：block_probe -288（海底边界 6710 块闭合验证，主载体）
- **单元对拍**：beardifier.h 单测——BEARD-244 8 点 + 表值抽样（Java 探针 BEARD 扩展输出 24³ 表或代表性点）
- **零退化铁律**：block_probe 8576（99.9994%）+ 3200（99.9997%）无退化
- **门禁**：`python scripts\scan_cpp_anchors.py` invalid=0（beardifier.h 核心函数 MUST @anchor.test + source）

## judge 预置

- 各阶段结论（算法移植 candidate、闭合量 candidate）授予前 SHOULD judge
- **收尾交付 MUST judge**（三源核对：.artifacts 快照 + git diff + regression 记录）
- 计划阶段已预置（本项）

## fan-out 预置

- 单机制单假设（Beardifier 缺失已验证 candidate）——无互斥分叉，不触发
- 若验证出现「海底边界未闭合」或「闭合量与预期差大」→ 分叉时触发 fan-out（candidate：a) 算法移植错位 b) 数据喂入不完整 c) 结构布局 dump 与实机不一致）

## 知识库更新

- 结论性 docs（04/06 篇、10 时间线、discovered）：**subagent 产出草稿（core.worker）+ 主会话应用验证**
- 临时记录（.investigations/ cmd-output/）：主会话可写

## 风险 & 回退

- **R1：block_probe 结构布局来源**——block_probe 是纯 C++ 离线工具，无 Java。需先扩展 Java 探针 dump -288 区域 structure 布局（piece/junction）。若 dump 不可行 → 回退：仅算法单测（BEARD-244 8 点对拍）+ 实机 JNI 验证（客户端加载村庄区域看地形）
- **R2：JNI 序列化成本**——piece/junction 数据量小（每 chunk 数十条），int[] 传参可接受；若 mixin 处构造失败（StructureAccessor 时机问题）→ 回退：仅 C++ 算法 + block_probe 参照验证，实机接入下个迭代
- **R3：结构布局时机差异**——vanilla 在 doFill 内构造（populateNoise 之后），mixin 在 populateNoise HEAD 构造——structure starts 可能尚未生成（getStructureStarts 会触发同步定位，与 vanilla 同源，预期一致，但需实测确认）
- **R4：8576/3200 零退化风险**——若该区域实际存在结构（Beardifier≠0），修复后这些块会变化。审计：8576 区域（720,-432）附近无村庄/结构记录；3200 区域（4×4 chunks 3200..3263）需确认。若变化 → 重新核算基线（诚实声明）
