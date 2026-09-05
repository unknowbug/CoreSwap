# G2 残差根因 fan-out — 候选 .b2：H2「Rust sky 直落/遮挡系统性错误」

session 标签：260905（fan-out .b2，只读分析；脚本 `.tmp/light-g1/b2_*.py`）
状态：**draft / candidate**（数据层证据充分，confirmed 待人类拍板）

## 裁决

**倾向 DENY（H2 不成立）**——一行理由：448 残差 chunk 的整 section「翻转」实为 **blocks9 输入的树冠方块差（rust 树冠多叶/密）经两侧序列化省略语义不对称放大成「vanilla 渐变 vs rust 缺键」的伪 0↔15 翻转**，Rust 内核 `light/mod.rs` 的直落/BFS/dom 索引/outFlags 逻辑与 vanilla 语义核对一致，无系统性缺陷。

## 机制分析（通读 worldgen-core/src/light/mod.rs）

逐段对照 vanilla 语义，未发现可产生「整 section 0↔15 翻转」的缺陷：

- **sky 直落**（mod.rs:191-203）：域 y0..383 自顶向下 opacity==0 即 15、遇 opacity>0 停，等价 vanilla 高度图柱状直落（域顶 y=319 即世界顶，无 y>191 遮挡语义差）。
- **15 直落不衰减特例**（mod.rs:274-278 `try_spread`）：`downward && l==15 && op==0 → 15`，与 vanilla ChunkSkyLightProvider 一致。
- **dom_index/dom_unpack**（mod.rs:42-51）：`(y*48+z)*48+x` ↔ `(x, z, y)` 互逆一致，无 P2 式 y/z 互换复发；blocks9 布局 `c*98304 + y*256 + z*16 + x` 与 JNI 注释（jni_bridge.rs:278）一致（G1 目标区 exact 100% 也反证索引/布局无错）。
- **outFlags**（mod.rs:319-336）：由实际 min/max 判定，均质也写全值 data。
- opacity 表核查（b2_opacity_table_audit.py）：leaves 全系 opacity=1、water=1、stone/dirt/log=15，与 vanilla 一致；仅 18 个 shulker_box/pointed_dripstone 为 -1（parse_u8_field clamp 到 0，数量级无关本残差，但建议后续修正）。

## 数据验证（b2_sky_flip_stats.py / b2_sky_fulldiff_anatomy.py / b2_key_presence_survey.py / b2_partial_profile.py）

1. **「整 section 翻转」的真实形态**：sky 全 4096 格全差的 section 共 360 个，**无一**是均质 0↔均质 15（v15/r0=0、v0/r15=0）。全部模式 = `vanilla 有 SkyLight 键且值域 0..13 渐变 → rust 侧缺键`（(True,False)=360/360）；Y 分布 = **世界 section Y=3..6，集中 Y=3(169)+Y=4(179)**（y≈48–95，地表+树冠段，非父代提示的 y176-191）；每 chunk 几乎恰为 **(3,4) 相邻两 section**（168/179）。部分差 section 387 个（17-4095 格）。
2. **缺键语义裁决（关键）**：rust MCA 有 14307 个「均质全 15」的 SkyLight 键、0 个「均质全 0」键 → rust 侧序列化只在 flag==1（全 0）时省略键。故 360 个 rust 缺键 section = **rust 实际输出全 0**；vanilla 0..13 渐变 vs rust 全 0。所谓「0↔15 翻转」是对比口径把 rust 缺键按「隐式全 15」填充造成的假象（该口径只对 vanilla「地表以上省略」成立；rust 侧省略语义是「全 0 省略」，两者不对称）。（附带发现：vanilla MCA 地下 Y0-2 缺键实为隐式 0 而非 15，现口径两侧同缺填 15 恰好相等未暴露，属口径隐患。）
3. **方向与偏差签名**：vanilla 值 > rust（rust 更暗）；部分差 section 的柱剖面显示 **rust = vanilla −1～−2、且恰在树冠顶部边界多一档衰减**（例：chunk(22,-9) 柱 x15z11：vanilla Y4 全 15、y63=14；rust y64=14、y63=12 —— rust 树冠比 vanilla 高/多一层叶）。「vanilla 有渐变 0..13 vs rust 全 0」= rust 树冠把该 16×16 柱群完全遮死（vanilla 树冠有缝隙漏光）。
4. G1 4×4@200 exact 100% 的解释：该目标区（200 区 chunk 12..15）无此类树冠差/树冠未全遮 section，blocks9 输入与 vanilla 逐块一致 → 光照自然一致；G1 通过不能为树冠差区域背书。

## 判别性结论

- 若 H2 成立（直落/高度语义/索引错）：翻转应呈 Y 顶部集中或随机均匀分布、且在 blocks9 输入已证一致的区域也应出现。实际：翻转集中地表 section、成对 (3,4)、且部分差剖面精确呈现「多一层 opacity=1 叶」的 −1/−2 平移 —— 全部证据指向 **blocks9 输入差（Rust worldgen 树叶放置与 vanilla 有 ±1 块级差异）**，而非内核错误。
- 内核代码静态审查 + G1 exact 100% + 上部梯度仅差 1~2（若直落/BFS/索引错不可能只差 1）——三重反证 H2。
- 可判别性遗留：残差区无 .blocks 方块导出，无法直接逐块展示树冠差（本结论由光照剖面反演）；建议主会话在残差区补一次 vanilla/rust blocks 导出，逐块确认叶层差（预期：rust 树冠多叶），即可升级为 confirmed 级证据。

## 可疑点（非根因，登记）

- `worldgen-core/src/light/mod.rs:121-125`（parse_u8_field）：`opacity: -1` 被 clamp 成 0；vanilla -1 语义是「按 VoxelShape 判定」，18 个方块（shulker_box×16、pointed_dripstone）语义有损。与 448 残差无关（文件属世界生成不可及方块），建议 light_data.json 生成器改为 0/15 二值显式化。
- 对比口径：rust 侧「缺键=15」填充不成立（rust 省略=全 0），g2_residual_map.py:90-92 的 fill 规则需按侧区分（vanilla 缺键=地表上 15/地下 0；rust 缺键=0）。

## 脚本与产物

- .tmp/light-g1/b2_sky_flip_stats.py — 448 残差 chunk section 级统计
- .tmp/light-g1/b2_sky_fulldiff_anatomy.py — 全差 section 解剖（Y/方向/键模式）
- .tmp/light-g1/b2_key_presence_survey.py — 两侧 MCA SkyLight 键存在性普查
- .tmp/light-g1/b2_partial_profile.py — 部分差 section 柱剖面（−1/−2 签名）
- .tmp/light-g1/b2_opacity_table_audit.py — light_data.json opacity 审计
