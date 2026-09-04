# Judge Review: oob-redo-verdict-260904-06（260904-08 工作块 T1）

> reviewer: core.judge (subagent, 隔离) | date: 2026-09-04 18:4x (Get-Date 实取)
> 对象: `.artifacts/lossless-accel/oob-redo-verdict-260904-06.md` (candidate) + 依据 incident-layout-260904-06.md + 重推导脚本
> 方法: workflow-patterns #41 独立复算 —— 从 blocks.h:69 定义**从零另写** y-major 变换（不 import coltool.py、不复用任何 x-major 脚本），对裁决三关键读数独立复算 + 一个 x-major 负对照。**未降级**：pwsh+python 实际运行成功。

## 总结论: PASS（有 2 项修改建议，均不动摇裁决主体）

## 独立复算结果（一手，落盘 `.tmp/p2full/judge-independent-260904-08.out.txt`）

| # | 检查 | 裁决主张 | 独立复算 | 判定 |
|---|------|---------|---------|------|
| sanity | 布局判别 | — | 4 臂 col(195,199) y62 water / y63 air / 顶面 y62 一致（海平面自检过；y-major 读出合理地形剖面） | PASS |
| CHECK1 | ref 列 (195,199) y180-319 全 air | 全 air (140) | `{'air': 140}`，cppNS 同全 air；全列 diff = 22 且模式逐条吻合（y−45..−44 cave_air↔deepslate、y6..31 andesite/diorite↔stone） | PASS（数值+模式双重吻合） |
| CHECK2 | mod↔ref FULL OOB (y>200\|y<−32) | 0 diff | 0 | PASS |
| CHECK3 | E1 y>200 贡献 | 0 | **y>200 = 0**；y<−32 = 118918（bedrock/deepslate↔stone，= 裁决第 2 条声明的预期 surface 规则差，与主张一致） | PASS |
| CHECK4 | cppNS y>200 | 0 diff | 0 | PASS |
| CHECK5 | 负对照（x-major 污染读法） | 幻影机制存在 | 8×8 列即出 238 个 y>200 幻影 diff（正读 0）——实证旧脚本读法确实制造幻影数据 | PASS |

复算实现独立要点：自写 header 解析（struct 大端）、自实现 `index=(y-MINY)*256+z*16+x`、自写 biome 段跳读、海平面 sanity——与 coltool.py 零代码复用。

## 逐项核对

1. **证据链完整性**: PASS。一手重推导输出路径在 verdict 内列明且文件存在；本次独立复算独立吻合。incident 记录（根因/污染清单/成立与作废清单）完整可引用。
2. **candidate 置信度**: 有据。数据层一手重推导 + 本次 judge 独立复算双源一致；verdict 明确 candidate（judge 待审，confirmed 留人类）——状态合法，无越权。
3. **supersedes 双指针**: **主体完整，但回指针缺失（修改项 M1）**——oob-redo-verdict 自身 supersedes 三份旧 verdict（正确）；但三份旧 verdict 正文均未加 superseded-by 回指针：writer-verdict-260905 无 superseded-by 行；writer-verdict-260904-06 的 superseded-by 仍只指 curtain-verdict；curtain-verdict-260904-06 仍写「superseded-by:（无）」——后者已失真。§15.4 双指针要求原结论补回指针（先例：residual-signature-verdict-260904-04 的 superseded_by 追加行）。index.yaml 侧：oob-redo 条目 supersedes 列表完整，但三个被取代条目无 superseded_by 注记（writer-verdict-260905 条目仅 ⚠️ 布局事故 comment，未点明已被 oob-redo 取代）。
4. **index.yaml 登记**: PASS。oob-redo-verdict-260904-06 已登记（id/path/kind/status/supersedes 齐全）；git 已提交（ae7908c），工作区 .artifacts 无未提交漂移（三源核对之 git 侧通过）。
5. **static-only 断言（#42）**: PASS（附注）。裁决第 3/4 条引用的 aquifer/est/heightmap 静态对拍均显式声明「布局无关」；未发现把静态读码包装成运行时实测的断言。第 4 条 ore 族坐标归因明确标注「待 coltool 重算」为 idk/待办，未超层。
6. **§9.7 可比性声明**: PASS。载体/覆盖面/与既有口径可比性三要素齐备，且诚实声明「读数变换已修正，可比性以 incident 记录为准」。
7. **噪声卡/retry cap**: 本次审查对象为分析产物非代码 anchor，无未解决噪声卡牵连；重推导为单轮数据层重采（新证据），无 cap 问题。

## 修改项清单（不改 status，交主会话应用）

- **M1（§15.4 回指针补齐）**: ① writer-verdict-260905.md 与 curtain-verdict-260904-06.md 头部补 `superseded-by: oob-redo-verdict-260904-06.md`（curtain 的「（无）」必须改）；② writer-verdict-260904-06.md 的 superseded-by 追加 oob-redo-verdict（保留既有 curtain 指针）；③ index.yaml 三个被取代条目补 superseded_by 注记。原正文不改写，只加指针行（先例 residual-signature-verdict-260904-04）。
- **M2（勘误注记，低优先）**: dump 文件 chunk keys 为 (12..15,12..15)（blocks x/z 192..255），与 header origin=(200,200) 及文件名 "_200_200" 语义不一致（本次复算四臂 key 一致故不影响任何结论）——建议在 incident 记录或 coltool 注释加一行说明，防后续导出脚本按 origin 误推世界坐标。

## 推荐状态

oob-redo-verdict-260904-06 **建议维持/授予 candidate**（复算支持；M1/M2 为文书修补不构成驳回理由）。confirmed 留人类拍板。
