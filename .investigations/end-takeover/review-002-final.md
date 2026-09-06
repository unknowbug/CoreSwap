# review-002 — end 接管开发块收尾 MUST 级审查（core.judge，260906-04）

> 总裁决：**推荐 confirmed（可授予）**。以下为 judge 原文意见全文落盘（主会话应用）。

## 一、三源核对结果（judge 亲手独立验证）

| # | 核对项 | 方法 | 结果 |
|---|---|---|---|
| ① | D1/D2 对拍 seed 7691421705105351955 | 逐行数值比对 java/rust-endprobe 双侧 | ✅ 全等（permChecksum/origin/5 simplex/5 erosion 含边界点） |
| ② | D1/D2 对拍 seed 12345 | 同法 | ✅ 全等 |
| ③ | seed 三查 | probe header + server 日志 initEnd seed | ✅ 双侧 7691421705105351955 |
| ④ | ab-final 逐位 | common=36 diff_chunks=0/36 total_mismatch=0；palette↔id 逐块全等比较 | ✅ Full 判据达成 |
| ⑤ | 执行体三元组 | 独立重算 sha256：target/release dll = build/libs jar 内 dll = mods jar 内 dll = **1B5AA1DEA49445A2…**（2160640B）+ 运行时日志自报一致 | ✅ 闭环（**注意：终版哈希为 1B5AA1DE，非早期记录的 96AD0411——后者是 cell 修复前的旧 dll**） |
| ⑥ | git diff 抽查 | terrain/worldgen_handle 通读 | ✅ 与快照一致 |
| ⑦ | 工作区 | git status 干净 | ✅ |

## 二、分条意见

- **PASS-1** Full 验收判据真实达成（逐位，非近似）。
- **PASS-2** §9.7 口径声明充分、分层如实。
- **PASS-3** zeroShape 重命名对 nether 行为无影响（条件集等价）。
- **PASS-4** end/nether 同形判别边界可接受（settings id + endActive 双闸）；WG_TRANSPILER/DFC/GPU 默认关且对拍全程未开。
- **PASS-5** 提交纪律、bin-diag 隔离合规。

## CONCERN（均非阻塞）

1. **哈希声明更正**：终版 = `1B5AA1DEA49445A2…`；早期「96AD0411」为 cell 修复前旧 dll，防记录污染（已由主会话在收尾时更正声明）。
2. **共享路径回归跟进**：with_cells / wrapping / 空列哨兵均为静态可证恒等变换，confirmed 后建议补跑一次 overworld/nether 既有 A/B 存档回归升为运行时证据。
3. **覆盖面小缺口**：y≥128 上半未逐位入判据（vanilla 本就全空气，口径已如实限定 y0..127），知悉即可。
4. **流程瑕疵**：D3 SHOULD judge 未单独执行（收尾 MUST 已覆盖其审查面），记入台账。
5. 备注：对拍输出文件建议内嵌 seed/argv 头注（产物自证）。

**confirmed 由人类拍板。**
