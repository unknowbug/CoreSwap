```yaml
id: pbeta-260917-05:timeline-draft
block: 260917-05
status: draft          # 追记行草稿 → versions/1.20.1/docs/10-timewise-archive.md 末尾 260917-05 块
worker: subagent (core.worker)
```

# 10-timewise-archive.md 追记行草稿（260917-05 块，追加到文件末尾，格式对齐 3312 行 260917-04 块）

```markdown
## 260917-05（实际 2026-09-17；P-β/P-path 分辨探针——β 与 F3 干净重载协议下均 QUIET，inputDiff23=46 立附带发现）🔍 candidate（judge PASS-with-conditions S1-S5 已应用；confirmed 留用户）

> 承接 260917-04 下块开工点（P-β/P-path 分辨探针，β 主嫌疑复核）；判据预登记 criteria-260917-05.md
> （时序锚 eaa0a5f @19:09:42 + 采集修复 eb3988f @19:10:37，均先于采集）。三轮采集：pbeta05a VOID
> （编译错 rc=1）、pbeta05b VOID（#42 家族第三犯：tmpdir 未固化 → lightInit threw → 整臂 vanilla，
> SELFCERT 正面拦截）、**pbeta05c 有效**（SELFCERT 三 boot 全绿：Done≥1 / lightInit ok / probe armed /
> fallback=0 / domain_hook=0 / dll 6f7fa3ae；path census 每 boot legacy-rust 529）。

- ✅ **C-β 判定（candidate，n=1）**：β（空节瞬态读）在干净重载协议下 **QUIET**——emptySec 逐 chunk
  两 run 恒等 + 全局 run2=run3=71871（run1=71900）；F3（信 stored 未重算）**未检出**（changed23=0，
  存在性不证伪，只证本载具本协议无现象）。与 260917-04 新基线（legacy 干净链 run2→run3=0）自洽。
- ⚠️ **独立推翻一次汇总定性（§16.3 交接验证）**：主会话机械交叉「inputDiff23=46/46 = packed↔blocks9
  ABI 切换」被 verdict worker 日志抽样推翻——全日志 `abi=blocks9` 0 行，10 个 diff chunk 三 run 均
  abi=packed、hash 两两不同、emptySec 恒等。46 重定性为**附带发现**：packed payload 跨重载不稳定、
  output-neutral、机制 open（palette 序/位打包/空节集合形态三候选未分辨，@anchor.idk 在案）。
- 📌 预登记分支覆盖缺口如实声明（#154 家族）：「inputDiff>0 且 changed=0 且输入恒等性可证」未单列，
  驱动机械退出码 1 的分支 3「第四通道」措辞失去对象，未开 fan-out（数据面无互斥分叉）。
- 📌 通用模式 → workflow-patterns 新发现一条（汇总交叉定性 MUST 回原始日志抽样 + 计数恒等≠集合恒等 +
  hash 探针随行打印口径；subagent 草稿待主会话应用定号）+ build-tooling #42 补充案例第三犯一行。
- 过程产物 `.investigations/pbeta-260917-05/`（criteria + cmd-output/ pbeta05a/b/c + knowledge-drafts/ 三份
  subagent 草稿）；§9.7：载具 = snap_light+Done+60s（C-1 系可比）+ [LIGHT-BETA]/[LIGHT-PATH] 新口径
  （无历史可比）；n=1 单 seed 单区域，不外推。
```
