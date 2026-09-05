# judge-verdict-260905-06 — 树族残差收敛 idk-7 实装 交付审查

- 角色：core.judge（subagent，只出意见不改 status；confirmed 留宿主人类）
- 审查对象：git 8ecaa68..244d680（worldgen-core/ 4 文件）+ .tmp/feature-parity-260905-06 验证链 + 方法论主张
- 三源核对：① diff 应用版（已逐行读 git diff）② scout 取证 + yarn/mojmap 一手源对拍 ③ 验证记录（盘上实测复算）
- **结论：APPROVE-WITH-CONDITIONS**

## 1. 代码语义核对（逐项）

| 核对点 | 结论 |
|---|---|
| RandomSelectorFeature（逐项 nextFloat<chance 即选即返；default 不抽选择 RNG；嵌套 generate_nested 接线 placed 优先/configured 兜底） | ✅ 与 scout §1/mojmap 一手源一致；default 字段名修复（"default" 兼容旧 "default_feature"）正确 |
| RandomPatchFeature（tries 来自 feature JSON；每 try 恒 6 次 nextInt 差分 j=xz+1/k=y+1，序 x→x→y→y→z→z；返回 i>0；xz/y_spread 由 IntProv 改 plain int） | ✅ 与 scout §2/yarn 一致 |
| BlockPredicate::parse 读 "type" | ✅ 修复正确（带 predicate_type fallback，安全；恒空→全灭链路解释成立） |
| Heightmap +1（ChunkRegion.getTopY 语义：实体方块上方第一位；k<=min_y 截断对应 Java k>bottomY） | ✅ |
| mega_jungle_trunk / bush / jungle foliage | ✅ 主链对拍：GiantTrunk 2×2 柱顶列仅 (0,0)（iy<height-1）+ dirt 四角；MegaJungle 分支 i=h-2-nextInt(4)、步长 2+nextInt(4)、nextFloat×2π、l/2 整除、TreeNode(-2, 非 giant)；bush 公式（无 /2 无 max0、角判无 y==0 子句）；jungle 非 giant 首层 1 次 nextInt(2)、ax+az>=7 判；generate_square giant ext=1；短路消费序（blob 角判 nextInt 恒消费）均与 yarn 一致 |
| worldgen_handle：block_at 接本 chunk 列 + ocean_floor 接入 | ⚠️ 方向正确（此前谓词全灭根因链成立），但见 C-3 两处语义偏差登记 |

## 2. 验证记录核对（judge 独立复算）

- **v9 跨 run vanilla 噪声 241,080**（盘上 out.txt）✅ ——与 v8 信号 248,054 同阶，支撑「v8 口径只能验无回归、不能裁决树族收敛」。方法论主张成立。
- **v8 判决 06 批 248,054/1938/3009** ✅ 落盘一致。
- **v10（树残差 old 171,442 → new 118,697，−30.8%）**：⚠️ **盘上 v10-verdict.out.txt 是 stale 记录**（16:49，早于 region_new.bin 再生成 17:07），内容显示 delta=0，与交付声明矛盾。judge 以盘上最终两份 dump 重跑：**old=171,442 → new=119,198，−30.5%**——方向与量级成立，但声称数值 118,697 不可从留盘产物复现（差 501 块，疑为中间版 dump 口径）。数值方向可采信，精确值须按 C-1 重存。
- **v11（新旧 dump 逐块 diff）**：judge 实测 = **740 chunks / 164,309 块差**（oak_leaves→air 90,171、jungle_leaves→air 22,097、vine→air 17,270 主导）——「同 vanilla 快照 + 纯 Rust 确定性 + 代码 diff 唯一变量」载体成立 ✅，确定性 dump 载体方法论认可。
- **v12 树族双向分解**：脚本在，**输出无落盘**；数值仅见于 index.yaml standing（与简报一致，无法独立复算原 run，但方向被 v10/v11 交叉印证）。
- **region_clean.bin 清理后不变性**：**无任何 hash 落盘记录**（全仓 grep 0 命中）——该声明当前不可核，见 C-2。

## 3. 遗留项（确认登记，非追责）

R-1 HashSet 桶序、vines feature、leaves distance 位、树位置对齐（oak rust多 38,250 / jungle rust少 22,611）——均已在 index.yaml standing 登记，与盘上证据一致 ✅。v10/−30.5% 是**进展指标不是收敛证据**，遗留项不得因百分比下降而降级。

## 4. 条件清单（APPROVE-WITH-CONDITIONS）

- **C-1（MUST）**：以最终 idk7_dump 二进制重跑 v10/v11/v12 并将三份输出落盘（当前 v10-verdict.out.txt 为 stale+自相矛盾记录，必须替换或标注 supersedes）；把 standing 中的 118,697/−30.8% 修正为可复现值（judge 实测 119,198/−30.5%，若原始 118,697 有据须给出口径差异说明）。
- **C-2（MUST）**：region_clean.bin 清理前后 hash 计算与对比记录落盘（.tmp out.txt 或 cmd-output/），补齐 §9.7 可比性三要素声明（载体/覆盖面/口径）。
- **C-3（SHOULD，语义偏差登记）**：① Rust MegaJungle 分支原木只 set_block 不入 trunk_set——Java getAndSetState 会加入 logs set，影响 TrunkVine decorator 候选集（vine 本身在 R-1 遗留内，须并入登记）；② block_at 闭包限本 chunk 列、越界返回 -1（≠air）——Java 会读邻 chunk 实况方块，跨 chunk 边界的谓词（offset/patch 出界）行为有差异，未登记，须补 @anchor.idk。
- **C-4（SHOULD）**：unsafe 指针闭包（col_ptr）依赖「apply_features 单 chunk 独占列」前提，建议加注释引用该并发不变量的出处文档，防后续多线程改动破坏。

## 5. 建议状态

代码 + 方法论达到 candidate 水准（验证=Full 层确定性 dump + judge 独立复算），条件 C-1/C-2 属证据落盘补全非结果推翻。建议：代码 candidate；standing「idk-7 已实装」在 C-1/C-2 完成后可提请用户 confirmed。

*judge 独立复算记录：v11 实测与 v10 重跑为本审查当场执行（PowerShell，2026-09-05，脚本 = .tmp/feature-parity-260905-06/{v11_dump_diff,v10_dump_vs_region}.py，输入 = 盘上 region_old/region_new.bin + ab-vanilla-region）。*
