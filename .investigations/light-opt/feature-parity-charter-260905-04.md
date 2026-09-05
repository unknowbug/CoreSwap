# feature parity 独立课题立项卡（260905-04 用户拍板新立）

## 背景与来源
- G2 收敛结论（confirmed 2026-09-05）：光照内核在 blocks 一致域无缺陷；残差真根因 = worldgen feature 放置分歧。
- judge 全量 447 chunk palette 对比：346/447 chunk 有 blocks 差——签名含 jungle_leaves/oak_leaves/vine/jungle_log 增减（80-350 处/chunk）+ stone↔coal_ore、granite↔andesite 替换层。
- 域归属裁定（2026-09-05 用户）：**新立独立课题**，不并入 260904-15 四项永久挂起。

## 范围
- 树木放置分歧（树叶/树干布局、藤蔓）——feature 放置阶段随机派生/形状生成。
- 矿石/替换层分歧（stone↔coal_ore、granite↔andesite）——ore feature + 替换规则。
- 明确不含：光照内核（已证无缺陷）、D4 接管协议、永久挂起 4 项。

## 判据（建议，立项时细化）
- 4×4@200 seed 8576294172403134396 域内 feature 放置逐位一致（palette 级）。
- 分层切分：先矿石/替换层（确定性更强，可能纯数据/派生层）后树木（形状+随机派生）。

## 已知输入（G2 遗产，可直接继承但开工时须廉价验证）
- 翻转集中 Y=3..4 地表树冠段；显式差剖面 rust=vanilla−1~2（树冠顶多一档衰减 = blocks9 输入差忠实后果）。
- 101/447「影子传播」未解释残差候选（推断级）随本课题 y/柱级验证收口。
- 447 对比脚本：.tmp/light-g1/（judge 复算脚本含 palette 对比）。

## 开工前置
- 读本卡 + 12-lighting.md + g2-convergence 结论；seed/坐标三查铁律全套适用。
- 机制未明初期 MUST recode-scout 勘探（feature 放置管线：NOISE→SURFACE→FEATURE 顺序、随机派生链）——禁直接跳单点定位。
