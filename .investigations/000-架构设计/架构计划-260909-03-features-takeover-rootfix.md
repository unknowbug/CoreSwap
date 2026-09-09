---
编号: 000
任务: mc-1216 features 接管——根因修复（biome_registry_order）+ b2 五缺陷 + 双臂重对拍 + mask 翻转重提
任务类型: 缺陷修复 + 验证对拍（re-code + swe 混合，主链为收敛型修复闭环）
模式档位: 轻量
状态: 待批准
日期锚: Get-Date 2026-09-09 14:59（标签 260909-03）
前序: NEXT_SESSION 260909-02 交接；verdict confirmed（用户拍板）；提交锚 00ce23d
---

## 范围（含明确不做什么）

- 做：① 生成 1.21.6 版 `biome_registry_order.json`（一手导出，禁止照抄 1.20.1）+ Rust 侧加载验证 + chunk(0,0) step9 (k,p,fid) 序列与 Java 对齐；② b2 五个静态 PROVEN 缺陷逐一修复（Disk break / Geode isAir / Ore isExposedToAir / Lake isSolid / emerald_ore）；③ 双臂 + 噪声基线重对拍（信噪比回落噪声量级 + step9 序列全同为验收判据）；④ 序列收敛后重提 mask 翻转（0b011→0b001）走 judge → 用户 confirmed。
- 不做：残差 9 项（§9.7 单列，随重对拍顺带观察，不单独立题）；缓装台账 B7-B12；1.20.1 主线任何改动。

## 任务拆解（子任务 → 预期产物）

1. **T0 交接结论廉价独立验证**（纪律要求，开工第一步）：复现「biome_registry_order.json 缺失 → 字典序回退」直接证据（复读 probe-rustfeat-err-snapshot.log + Rust 加载路径源码核对），验证通过才继承根因方向。
2. **T1 根因修复**：从 1.21.6 注册表导出 biome 顺序（覆盖面逐项核对，含 pale_garden 等新 biome）→ Rust registry_order 加载生效验证 → chunk(0,0) 双侧 (k,p,fid) 对拍（复用 WG_FEATURELOG + javafeat 探针，.tmp 脚本在位）。
3. **T2 b2 五缺陷修复**：每修一项跑对应探针 P1-P6（见 fanout b2 §四）；Geode 修复附带 CheckedRandom vs Legacy 噪声流等价性验证；Ore isExposedToAir 选型（邻 chunk 读 vs 保守 discard）若两案各有代价 → 触发 fan-out 判定。
4. **T3 重对拍**：`chunky_arm_260909-02.ps1` 双臂 + 噪声基线重跑（mask/seed 三查照旧）。
5. **T4 mask 翻转重提**：judge → 用户 confirmed（新结论走新流程）。

## 验证方式

- T1/T2：Full/Partial 探针逐项（WG_FEATURELOG 序列、P1-P6 各自探针），每项带 seed/坐标三查。
- T3：验收判据 = 信噪比回落噪声基线量级（terrain/veg 两口径）+ step9 (k,p,fid) 全同；§9.7 声明对比口径。
- 工程修复不消耗 evidence saturation 计数；连续 3 轮无新数据层证据 → 回数据层/升级。

## judge 预置

- T3 对拍结论 candidate 授予：SHOULD judge（审查对象：verdict 草稿 + 对拍原始数据 + git diff）。
- mask 翻转重提 + 收尾交付：MUST judge（三源核对）。

## fan-out 预置

- 潜在分叉点：T2-Ore isExposedToAir 实现选型（邻 chunk 读 vs 保守 discard）若非单选 → .bN 候选并行，禁止主会话自推。
- 其余收敛型修复主会话直接做（v0.8 收敛门）。

## 知识库更新

- 结论性 docs/discovered 写入：subagent 产出草稿（先读 SUBAGENT-KNOWLEDGE-GUIDE.md）+ 主会话应用验证；临时排查记录主会话可写。

## 子角色介入点

- scout: 否（管线地图已有，根因已定位；机制未明场景未出现）。
- worker: T2 若某缺陷修复涉及发散解读/代码交付隔离 → subagent；T4 judge。
- fan-out: 仅上述 Ore 选型分叉点。
- judge: T3 candidate SHOULD + T4/T4 收尾 MUST（预置见上）。
- knowledge: 各 Phase 末尾结论性落盘 subagent 产出。
