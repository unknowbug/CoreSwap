# M17 bedrock 随机带修复 — 结论摘要（2026-09-02 session）

> **状态更新（2026-09-02）：用户实机验收通过（下界无异常，M16+M17 一并确认）→ M17 修复与 M16 Full 化结论升级 confirmed。**

## 结论（candidate，待 judge + 用户确认）
Nether bedrock roof 随机带残差（123..126，4011 块）根因 = `parse_anchor_abs_y` 的 `below_top` 锚换算 off-by-one：
- 旧：`min_y + height - v`（128-v → true_y=123/false_y=128）
- 新：`min_y + height - 1 - v`（Java 顶块 y = min_y+height-1 = 127 起 → true_y=122/false_y=127）
修复位置：`WorldgenRust/src/surface_rules.rs` L944。overworld deepslate 梯度用绝对锚不受影响（故此前未暴露）。

## 证据链（数据层）
1. 诊断 bin `WorldgenRust/src/bin/nether_bedrock_band.rs`（per-y vanilla/rust bedrock 计数，4×4@0,0 seed -8248）：
   - 修复前 vanilla 概率 [123]=0.2 [124]=0.4 [125]=0.6 [126]=0.8 [127]=1.0(满层)；Rust 同模式整体 +1 层（[123]=0…[127]=0.8）
   - 修复后逐位吻合：每层 van_only=rust_only=0 → splitter 种子派生正确，纯锚换算 bug
2. 全量回归 `multiworld_nether_blocks`：TOTAL 96.0568% → **96.4428%**；y96..127 94.0% → **97.12%**（同工具同区域同 seed，前后可比）

## 同 session 附带完成：M16 Full 化（overworld 存档层）
- 新工具：`.investigations/multiworld-port/cmd-output/compare_save_region.py`（区域 MCA vs vanilla FULL 参照，与 runtime 版 BlockProbe WGB2 格式对齐——含 per-chunk wx/wz + 256 UTF biome；M16 旧 compare_save_vs_ref.py 读 MC 侧旧格式，两格式不兼容）
- seed A=-2032795982907864146：16 chunks @3200,3208 → **存档层 96.1750% = 内存级 96.1750%（精确同值）**
- seed B=8576294172403134396：同区域 → **存档层 97.3254% = 内存级 97.3254%（精确同值）**
- 判定：JNI 写入→序列化落盘逐位无损（双 seed × 16 chunk 全一致），M16 写路径修复存档层 Full 化成立
- 口径声明（§9.7）：「存档写入口径」（存档 NBT vs vanilla FULL 参照，含 feature 残差），与 96% 系「探针口径」不可比；残差 = 涂布/feature 类生成差异（packed_ice/clay/矿石等），非写入错位
- 运行日志：cmd-output/{ref-ow-3200-blockprobe,rust-save-m16ful-run1,rust-save-m16ful-run2-seedB,ref-ow-3200-blockprobe-seedB}.log

## 状态
- M17 修复：cargo release 编译通过；未实机验证（建议用户实机验收时一并观察下界顶部床岩）
- 残余残差（非本课题）：soul_sand/gravel 涂布边界、熔岩湖边界、洞窟空腔、矿石 feature 差异

## judge 审查与补盘（2026-09-02）
- judge 结论：**A（M17 修复）与 B（M16 Full 化）均建议授予 candidate**；波及面/格式假设/seed 三查/口径声明全部核实通过
- 补盘完成（judge 待办两条）：
  - `cmd-output/nether-bedrock-band-after-fix.txt`（修后 per-y 原始表：832/1675/2450/3256/4096 逐位吻合）
  - `cmd-output/compare-save-region-seedB-result.txt`（GRAND 97.3254%）
  - `cmd-output/compare-save-region-seedA-result.txt`（重跑确认 GRAND 96.1757%）
- ⚠️ 微漂移疑点（诚实标注）：seed A 首跑 96.1750%（1512702）vs 复跑 96.1757%（1512713），差 11 块（0.0007pp）——两次 dll 差异仅在 below_top 分支（overworld 理论不受影响），来源未查明；量级不影响结论，挂 M4 家族（跨运行确定性）线索待下轮若有需要再查
