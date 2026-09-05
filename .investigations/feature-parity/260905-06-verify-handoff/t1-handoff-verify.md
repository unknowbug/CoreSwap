# T1 交接廉价独立验证（260905-06，实际 2026-09-05 15:4x，主会话）

## 执行体三元组核对（知识库 #36）

- `target/release/worldgen.dll`：2026-09-05 15:21:46，2,056,704 bytes，sha256 E3D24AFF…（8ecaa68 提交于 15:32:08，构建先于提交，源内容一致）。
- `runtime/1.20.1/java/src/main/resources/native/worldgen.dll`：14:18:24 / 2,059,776 bytes —— **暂为旧版**，但 build.gradle processResources doFirst 会在 runServer 时从 target/release 强制同步（inputs.file 已声明 dll），设计内自愈，非过期风险。实际执行体 = 下次 gradle run 打包产物。
- 上轮 A/B 快照生成时执行体经 phase5-interim §1 核验（D97995B9… hash 一致 + 同步日志）。

## 基线复现（239,594 口径）

- 命令：`python .tmp/feature-parity-260905-05/v8_ab_verdict.py`（对现存 ab-vanilla-region / ab-takeover-region 快照重跑）
- 结果：**双侧地形完整=3009  diff chunks=1829  total=239594** —— 与 NEXT_SESSION/judge 独立复算基线逐位一致 ✅
- top deltas 签名与 phase5 判决一致：树族（oak/jungle leaves、vine，Y4-6 主导）+ 双向 ore 小残差（andesite/granite/diorite 双向均衡 1.0-1.2 万）。
- §9.7 口径声明：载体 = v8 name 域 region 对比（v3 口径）；覆盖面 = 受控 A/B 双臂 5313 chunk 中双侧地形完整交集 3009；与 standing 基线可比（同脚本同快照同域）。

## 结论

交接结论「基线 239,594/1829/3009 + 树族主签名」验证通过，可继承为公理推进 T2/T3/T4。

**未验证假设（分离落盘）**：
1. idk-7（selector/patch 占位 + generate_nested 未接线）确为树族残差主来源——本轮 T3 实装验证。
2. R-1（HashSet 桶序）影响面未定量——T4 定界。
3. dll 的 opacity/light 修复（14:18→15:21 重建）不影响 feature 对比——feature 装饰不读 light 数据，暂判无关，若后续见怪签名再核。
