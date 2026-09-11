# RELEASE INDEX — 发布工单入口（Maint 只读本目录；状态回写见 status\ 镜像）

| 版本 | 工单 | 状态 | 创建（Get-Date 锚定） |
|---|---|---|---|
| 1.0.26 | [RELEASE-1.0.26.md](./RELEASE-1.0.26.md) | published（2026-09-06，tag v1.0.26；回执 = 工单 head status 行，status 镜像目录缺失已提醒 Maint 在 1.0.27 补） | 2026-09-06 |
| 1.0.27 | [RELEASE-1.0.27.md](./RELEASE-1.0.27.md) | published（2026-09-07，tag `coreswap-1.20.1-1.0.27` 已重建至 `26fc23e` 并核对 master 谱系；回执 = 工单 head status 行；status 镜像目录仍缺，Maint 侧待补建） | 2026-09-07 |
| 1.0.28 | [RELEASE-1.0.28.md](./RELEASE-1.0.28.md) | pending（judge PASS-with-conditions 已应用；待用户 confirmed 后转 Maint） | 2026-09-11 |

- 契约：[RELEASE-CONTRACT.md](./RELEASE-CONTRACT.md)（schema / 状态机 / hash 双重校对 + 污染反馈环）
- Maint 消费提示词（用户粘贴到维护会话）：
  ```
  读取 E:\PYTHON\CoreSwap\.artifacts\releases\INDEX.md 及 pending 工单，按 RELEASE-CONTRACT.md 执行：先双重 hash 校验（jar + 内部 dll 独立重算，与票内逐字节比对），通过后准备发布（tag/notes/asset），用户显式确认才执行发布；发布后回写 status 镜像。hash 不一致 → halt 并按契约 §5 反馈主工作区。
  ```
