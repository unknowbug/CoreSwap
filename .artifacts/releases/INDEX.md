# RELEASE INDEX — 发布工单入口（Maint 只读本目录；状态回写见 status\ 镜像）

| 版本 | 工单 | 状态 | 创建（Get-Date 锚定） |
|---|---|---|---|
| 1.0.26 | [RELEASE-1.0.26.md](./RELEASE-1.0.26.md) | pending | 2026-09-06 |

- 契约：[RELEASE-CONTRACT.md](./RELEASE-CONTRACT.md)（schema / 状态机 / hash 双重校对 + 污染反馈环）
- Maint 消费提示词（用户粘贴到维护会话）：
  ```
  读取 E:\PYTHON\CoreSwap\.artifacts\releases\INDEX.md 及 pending 工单，按 RELEASE-CONTRACT.md 执行：先双重 hash 校验（jar + 内部 dll 独立重算，与票内逐字节比对），通过后准备发布（tag/notes/asset），用户显式确认才执行发布；发布后回写 status 镜像。hash 不一致 → halt 并按契约 §5 反馈主工作区。
  ```
