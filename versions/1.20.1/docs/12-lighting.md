# 12 · 光照引擎（Rust 重写）——P2 收口状态

> 状态：candidate（G1/G3/G3c PASS 判据已过，G2 转向 + D3 FAIL 待后续处理；judge APPROVE-WITH-CONDITIONS；confirmed 待用户拍板）
> 本篇只记结论与判据；过程/被推翻假说归 10 时间线。

## 结论链（猜测→验证→排除→发现）
1. **猜测**：G2 残差 = Rust fallback 收不到 propagateLight（光照传播调用缺失）。**验证/排除**：fan-out .b1 DENY——残差 0/448 全在 pregen 边界 + vanilla 传播是拉取语义（无主动 propagate 链）。
2. **发现（G2 真根因）**：残差 = **worldgen feature 放置分歧**（树叶/藤蔓/矿石/安山岩）；judge 全量 447 chunk palette 对比归因。光照内核在 blocks 一致域**无缺陷**。
3. **G1（内核正确性）**：blocks 一致域光照 exact 100%（口径：4×4@200 树冠一致区 16 chunks 光照 MCA 逐位 + 静态审查；**不为树冠差区域背书**）。
4. **G2 收口口径**：feature 放置分歧闭合后重测；当前残差归因已明，光照侧无待修项。
5. **G3（round-trip）二次收敛口径**：1.20.1 存档无 isLightOn 键 + 每次启动必 relight（vanilla 亦然）→「光照不变」严格判据对该引擎**不成立**；有效判据 = **二次重启收敛 0 + 相对基线**。首载漂移 vanilla 17/2025、rust 123/2025（**7×，待办**——相对基线差，二次重启均收敛 0；口径：2025 chunk pregen，首载 vs 二次重启）。
6. **G3c（nether/end sanity）**：PASS——64+64 chunks 生成、高度守卫回退 vanilla（bottomY=0 span=16 打点正确）、无新 crash。
7. **D3（性能）双 FAIL**：内核 6.88ms/chunk（合成数据）；e2e 回退 2.4×（29.7 vs 12.4s / 2025 chunks，gate ON vs OFF 全量 wall）。开销主体 = 内核 BFS。

## 已知语义问题
- light_data.json opacity:-1 经 parse_u8_field clamp 成 0（18 条目；与残差无关，语义有损，见 knowledge/discovered/compiler-idioms.md #14）。
- @anchor.idk（降级）：LightStorage.enqueueSectionData 再传播语义未核（round-trip 二次收敛已实证，风险降级；随 7× 漂移待办一并跟踪）。

## 对比判据（复用）
- 跨实现光照 MCA 对比 MUST 先核两侧缺键语义（vanilla 缺 SkyLight=隐式 15 / rust 缺键=flag1 全 0；统一填充 = 1.47M 假差异），见 workflow-patterns #46。
- MCA 解析坑（无 xPos / unpack_from 不推进）见 build-tooling #20；源码参照核版本见 f5-bugs #5。

## 被排除假说（一行排除清单）
- ❌ fallback 收不到 propagateLight——.b1 DENY（残差在 pregen 边界 + 拉取语义）
- ❌ 光照内核 BFS 缺陷——G1 exact 100%（blocks 一致域）
- ❌ isLightOn 缺失是 rust 侧 bug——vanilla/rust 双侧存档均无此键（.tmp/net 源树疑非 1.20.1）
