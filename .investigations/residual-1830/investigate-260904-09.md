# residual-1830 排查记录（260904-09 工作块 · .investigations 中间产物）

## 课题
残留 1830（off-fixed-08 后残差，99.884%）：流体族 ~700（水洞群 x196-216/y14-30/z236-242，CS 干洞 vs vanilla 水洞）+ deepslate→air 106 + surface 小族 ~300。

## 链路
1. **廉价独立验证**：dll 三元组 ✓（0D247E03 双处一致）；witness 坐标（205,22,239）实机+直读器口径 ✓。
2. **scout 勘探**（subagent 915e5264）：scout-aquifer-map.md——公式/结构层逐行零分歧；缓存臂闭（精确值缓存无语义差）；纠正「est 4 角插值」过时假设（两侧均为列扫描）；关键结构性发现：残差**双向并存** → 指 D4 blob 三元组差或阈值震荡。
3. **判别探针**（worker 8bcf0826 交付，主会话应用）：Rust WG_AQDUMP（aquifer.rs 点文件驱动全链 dump）+ Java AquiferDumpProbeMixin（@Shadow 复算 + RETURN 捕获 vanilla decision）。12 点判别：**opq/r/s/t 12/12 全异 → D4 实锤**。
4. **根因**：`worldgen_handle.rs` aquifer splitter 直传顶层 `random_deriver()`，漏 `split_str("minecraft:aquifer").next_splitter()`（Java NoiseConfig.java:54 对应链；ore 管线 L261 有同构 split 对照）。splitter 种子错 → 全部 blob 随机偏移错 → 液面/距离场全链分叉 → 干/水双向互换 + carver 域（aquifer.apply(pos,0)）投影为 deepslate→air。
5. **修复**：worldgen_handle.rs 一行（split_str("minecraft:aquifer").next_splitter()）。
6. **探针复验**：修复后 12/12 点 opq/r/fl2/decision 全对齐（残差字段均为 Rust 惰性早退 na vs Java 旁路全算，无真实数值分歧）。
7. **decisive probe**（全新 world 重导，dll 5E2ACB7F，21:11，worldSeed 三查 ✓）：**1830 → 76**（99.995%）；流体族/deepslate→air 全消；(205,239) 列 vanilla 水面精确恢复。
8. **新残留 76**：gravel→sand 49 / sand→gravel 24 / 零星 3——surface 材质微族，与 aquifer 无关，独立小课题（未立项）。

## 探针/工具链 bug 台账（错误优先）
| # | 现象 | 根因 | 定位 | 修复 | 教训 |
|---|---|---|---|---|---|
| 1 | WG_AQDUMP 门控零输出（stderr 无 enabled 行） | worker 草稿 `aqdump_hit` 先查 AQDUMP_ON（false）才初始化 points（唯一置位点）——鸡生蛋死锁 | stderr 无 [AQDUMP] enabled → 门控未激活 | env 存在性 OnceLock 先判 | 诊断门控「flag 在初始化器内置位」模式必须先判 env 存在性；上机前静态走一遍首次调用路径 |
| 2 | Java 启动即 IllegalClassLoadError: WgCap cannot be referenced directly | mixin 包（wg.bench.mixin.*）禁止非 mixin 类（嵌套类 WgCap 被字节码引用） | 混淆栈直接指 mixin transformer | WgCap 移到 wg.bench.AquiferDumpProbe 公有嵌套类 | mixin 包内不放任何辅助类（含 static nested） |
| 3 | DimensionType.field_35479 中间名映射风险 | worker 预警的 mappings 不确定性 | 编译前人工核对 | 按源码语义替换字面量 -32512 | 中间名常量优先用字面量+注释 |
| 4 | cmp 脚本 0 配对 | 键含 density（4 元组）vs 3 元组查找 + density 三位舍入作键（density 本身是对比字段） | 手动集合交集核对 | 坐标配对 + density 就近 | 对比脚本「键」不得含待对比字段 |

## 命令记录
- rustc 单编 driver：`rustc --edition 2021 -O src\bin-diag\aqdump_driver.rs -o .tmp-bin\aqdump_driver.exe --extern WorldgenRust=<rlib> -L target\release\deps`（先建 -o 父目录）
- WG_AQDUMP=.tmp/aqdump/points.txt 运行 driver → rust.txt
- Java：gradle runServer -PcppVanilla=true -PaqDump=1 -PaqDumpPoints/-PaqDumpOut（build.gradle 已加映射）
- 导出：gradle runServer -PcppReplace -PcppLib -PblockProbe=1 -PblockProbeFull=true -PbenchSeed/-benchSize=4/-benchOriginX=200/-benchOriginZ=200 -PbenchOut=.tmp/p2full/off-aquifix-260904-09
- 对比：python .tmp/aqdump/cmp_fixed.py（探针级）/ python .tmp/p2full/verify_aquifix_260904-09.py（decisive）

## 状态
- 修复 = **candidate**（decisive probe + 探针复验双过，待 judge + 用户 confirmed）
- 现役生产 dll 基线待更新：0D247E03 → 5E2ACB7F（resources 同步待 confirmed 后执行）
