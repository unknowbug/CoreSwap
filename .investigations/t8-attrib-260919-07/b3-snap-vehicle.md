# .b3 snap 载具差 — 残余子面出清（t8-attrib-260919-07，draft）

- 角色: fan-out worker .b3（snap 载具覆盖/坐标/缺键/序列化语义差）
- 结论: **倾向排除**——snap 载具本身不构成 44.69% diff 的机制（置信度：高，静态一手核对）；
  唯一残余子面「'-' 缺键语义混入」若存在，其**制造点在接管写回语义（.b1 辖区），不在 snap 载具**，
  且形态上无法解释「2 连通矩形 + singleton=0 + 中带零 diff」。
- 验证分层: **Degraded（纯静态审查）**——沙箱无 shell，未跑任何解析/重采。本结论不外推通道/y 剖面（#188 盲区，§4）。

---

## 1. 残余子面出清：'-' 缺键能否系统性混入（问题 1）

### F（一手源码）

1. **snap 解析完全对称**：`.tmp/g3-260905-03/snap_light.py:81-85` —— 两臂同脚本逐 section 取
   `s.get("BlockLight")` / `s.get("SkyLight")`，缺键 → `"-"`；`repr(sig)` 只含 (Y, hex12|"-") 元组，
   不含原始字节 → 序列化字节差（如 gzip vs zlib、NBT 键序）**不可能**混入 hash（:88 整体 sha256 前已折叠为
   确定性字符串）。`if bl else "-"` 对 `b""`（0 长数组）与缺键同判 "-"——该模糊**两臂同侧对称**，不制造臂间差。
2. **接管写回永不传 null**：`versions/1.20.1/java/src/main/java/wg/bench/mixin/ServerLightingProviderMixin.java:894-901`
   —— 注释明示「flag：0=数据有效（拷贝 2048B），1=全 0，2=全 15（均质用 defaultValue 构造，**不传 null**）」；
   写回点 :590-591（域批路径）与 :831-832（legacy 路径）均对 **24 sections × BLOCK/SKY 全量**
   `provider.enqueueSectionData(...)`，无一 null。`.tmp/light-yarn/LightingProvider.java:121-129`：
   enqueueSectionData 接受 @Nullable，非 null 即入 light storage。
3. **ChunkNibbleArray 语义**：`.tmp/light-yarn/ChunkNibbleArray.java:31-33,110-119,164-175` ——
   `new ChunkNibbleArray(0)`（bytes=null, defaultValue=0, `isUninitialized()`=true）与
   `new ChunkNibbleArray(15)`（defaultValue=15）经 `asByteArray()` 物化为 2048×0x00 / 2048×0xFF；
   即**接管臂落盘的 section 键存在面 = 全 24×2**。vanilla 臂 = 同一 Java 服务器同一 ChunkSerializer
   （两臂唯一差 = LIGHT_DOMAIN/LIGHT_RUST 开关 → mixin :859 `if (!LIGHT_RUST) return;` 完全旁路），
   vanilla light storage 中不存在的 section 由 ChunkSerializer 决定落不落键。
4. **key 集一致**：cmp_t8 common=9450、new=0、gone=0（scout-map §1.3）——chunk 级覆盖/坐标零差。

### I（推断，标注置信度）

5. **'-'↔hash 伪翻转通道存在与否，取决于 vanilla ChunkSerializer 对「storage 无此 section / 全零数组」的省略规则**
   ——该一手源（ChunkSerializer.java）工作区无（`versions/1.20.1/java` 只有探针工程；`.tmp/light-yarn/` 七个文件
   无 ChunkSerializer），web 检索仅得 API 文档无方法体 → **静态不可判**。
   但**两分支收敛同一 .b3 结论**：
   - 分支 α（vanilla 也落全零键 / 不省略）：两臂同值 → 同键同 hash，snap 制造不了任何伪差 → .b3 全出清。
   - 分支 β（vanilla 省略全零/缺失 section 键）：接管臂多写「显式零 nibble」键 → diff chunk 出现 '-'（vanilla）vs
     零值 hash（domain）翻转——但该翻转的**制造点是 mixin 写回「不传 null」的接管语义**（F2），snap 只是忠实
     见证人（F1）→ 证据流**让渡 .b1**（域批写回语义），.b3 仍排除。
6. **量级/形态双重反证（分支 β 下）**：
   - 量级：任何 chunk 都必然含全零 BlockLight section（普通空气区）与高空 SkyLight section，
     section 存在性类别近地图均匀 → 系统性 '-'↔零值 hash 翻转应打向 **≈全部 9450 chunk**，不是 4223（44.69%）。
   - 形态：见 §2。
   两项联合：分支 β 即便为真，也只能解释一个「准全覆盖背景差」，与观测的**两个矩形簇**形态不匹配。

**子面判定：不能解释「矩形整片、singleton=0」形态**（置信度：高）。

---

## 2. 形态论证：载具/序列化类差 vs 观测形态（问题 2）

- **F**：diff = 2 个 4-连通矩形簇（A: 616, B: 3607），singleton=0，中带零 diff（继承，勿重验）。
- **I（论证）**：snap 载具差的作用变量是「per-chunk 解析/序列化语义」，对每个 chunk **位置无关地、确定性地**施加——
  同一脚本对同构 mca 输入，行为只依赖 chunk 内字节内容类别，不依赖世界位置。因此载具类差的空间投影只能是：
  ① **内容类别驱动的准均匀撒点**（如 §1-β 的 '-' 翻转，类别近均匀 → 近均匀/高覆盖，且伴随海量孤点簇）；
  ② **随机性语义噪声**（如解析 bug 偶发错位 → 高 singleton、无空间相关）。
  观测 = **2 个地形尺度连通域、0 singleton、簇间完整零差带** → 空间强相关 → 唯一相容解释 = 差源自
  **世界内容/传播本身的地形相关差异**（.b1 域批算法差 /.b2 时序差的辖区），载具差与其形态**不相容**。
- 逻辑地位：这是**形态排除**（exclusion-by-morphology），非直接证伪；但它对「载具机制为主嫌」给出强反证。

---

## 3. snap 坐标推导对称性复核（问题 3）

- **F**：`snap_light.py:6`（rx/rz 从文件名解析）+:87（`cx,cz = rx*32+(i&31), rz*32+(i>>5)`）——两臂同一函数、
  同一 region 布局（两侧均由同一 Java TACS 写盘，.mca 头/槽位惯例同源）。
- **F**：9450=9450、new=gone=0 → 若坐标推导任一侧有非对称路径（偏移/翻转/索引序错），key 集必不匹配；
  key 集逐位一致在数学上**封死**坐标非对称（置信度：高）。
- **F**：死代码 `snap_light.py:23`（file 级 hash 计算后被覆盖，未入 snap）——已被 scout §1.4 记录，非对称面为零。
- **I**：残余理论路径仅剩「NBT 解析器 bug」（payload():39-60，手写 NBT 读取）——但同一解析器跑两臂，
  任何 bug 对两臂同构输入同现，制造的是假**同**不是假**异**；且两臂 vanilla 侧 hash 若被解析 bug 扰动，
  E1 重复臂对拍（G17 旁证 0.8%）不会稳定——非对称路径不成立（置信度：中高，E1 旁证不作 E2 上界，judge N2 口径）。

---

## 4. 结论与让渡

### 辖区结论：**倾向排除**（.b3 = snap 载具不是 44.69% diff 的机制）

置信度：高（静态一手 + 形态反证 + key 集封死）。降级声明：Degraded（纯静态）；
「分支 β 下让渡 .b1 的伪翻转子面」为条件化残留，需 §5 实验定谳。

### 排除证据清单（供 judge 三源核对）

| # | 证据 | file:line | 类型 |
|---|---|---|---|
| E1 | snap 解析/哈希折叠对称，序列化字节差不入 hash | .tmp/g3-260905-03/snap_light.py:81-88 | F |
| E2 | key 集逐位一致封死坐标/覆盖非对称 | cmp_t8 输出（scout-map §1.3 继承） | F |
| E3 | 接管写回 24×2 全量非 null（缺键面不对称只可能来自引擎侧） | ServerLightingProviderMixin.java:590-591,831-832,894-901 | F |
| E4 | 两臂唯一差 = LIGHT_RUST 开关旁路，序列化器同为 vanilla Java | ServerLightingProviderMixin.java:856-859 | F |
| E5 | ChunkNibbleArray(0/15) 物化语义（asByteArray 填充） | .tmp/light-yarn/ChunkNibbleArray.java:31-33,110-119 | F |
| E6 | 形态不相容：位置无关载具差 ⊬ 2 矩形连通域 + 0 singleton | scout-map §3 分布结论（继承） | F+I |
| E7 | 分支 β 量级反证：类别均匀 → 预期 ≈全量 diff，观测 44.69% | 本文件 §1-I6 | I |

### 让渡清单

1. **→ .b1**（唯一条件化残留）：若 §5 实验显示 diff chunk 存在臂间 '-' 计数差，
   该伪翻转制造点 = 接管写回「不传 null」语义，归 .b1 辖区（写回序列化存在性语义差）。
2. **→ judge**：本产物 Degraded 静态结论，等待 .b1/.b2 候选 + 形态证据汇合后合审。
3. **盲区声明（不外推）**：|d| 量级 / y 剖面 / 通道分离不可得（scout §1.4）；本结论不含通道级断言（#188）。

## 5. 主会话执行命令模板（最小判别实验：'-' 计数剖面，只读既有 mca）

> 前置：按 scout §4.1，region 若被后续 run 覆写需先重走 #161 world 身份链；.tmp snap json 为稳定证据体，
> 本实验解析的 mca 必须仍是生成两臂 snap json 的同一批文件（mtime 核对）。

```python
# -*- coding: utf-8 -*-
# t8_b3_keypresence.py — 两臂 per-section 缺键('-')计数剖面（只读，零采集）
# 判读: diff chunk 上 |'-'计数差|=0 → 分支α, .b3 残留子面全出清;
#       差≠0 → 分支β, 翻转面定量, 让渡 .b1（写回语义差），.b3 仍非机制。
import struct, zlib, gzip, os, json
from collections import Counter
sys_dir_a = r"E:\PYTHON\CoreSwap\runtime\<armA>\run\world\region"   # 主会话填: 两臂 region 目录
sys_dir_b = r"E:\PYTHON\CoreSwap\runtime\<armB>\run\world\region"   # （与当时 snap 输入同一目录）

# 复用 snap_light.py 的 R/payload/per_chunk_light 解析（同一解析器口径），
# 但 sig 阶段改为计数: per chunk 记 (n_bl_missing, n_sk_missing, n_sections)。
# 与 t8-*-r1-light.json 的 diff key 集合 join，输出:
#   diff_chunks 中 '-' 计数不同的 chunk 数 / 相同的 chunk 数
#   非diff_chunks 中同指标（对照组）
# 模板要点（实现细节让渡主会话）: import snap_light 模块或复制其 R/payload 两个函数，
# 循环 per_chunk_light 的 sig 段替换为缺失计数。禁止写回任何 region/json 原文件。
```

- 运行环境: python 直跑即可（snap_light.py 同款依赖，stdlib only）；输出落 `.tmp/`，回传 .b3/judge 解读。

---
*status: draft（置信度状态机：AI 不自标 candidate/confirmed）*
