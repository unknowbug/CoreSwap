# 发布工单契约（RELEASE-TICKET，260906-06 与 CoreSwap-Maint 发布管理 §5 对接）

> 职责分离（CoreSwap-Maint AGENTS.md §5，260906 拍板）：**构建归主工作区，发布归维护 agent，逐次人工确认**。
> 本契约定义主工作区 → 维护 agent 的固定交接载体。方向与 BUG 卡相反（BUG 卡 = Maint 写主工作区读；发布单 = 主工作区写 Maint 读）。

## 1. 载体与位置

- **工单源（唯一事实源）**：`E:\PYTHON\CoreSwap\.artifacts\releases\RELEASE-<version>.md`（主工作区持有——代码、构建、验证证据都在主工作区侧）。
- **投递入口**：`E:\PYTHON\CoreSwap\.artifacts\releases\INDEX.md`（一行一单：版本 | 状态 | 日期；Maint 只读）。
- **回写镜像**：发布完成后由 **Maint agent 在自己工作区**写 `E:\PYTHON\CoreSwap-Maint\.artifacts\releases\status\RELEASE-<version>.json`（tag / release URL / asset 名 / 发布时间），主工作区只读——双方隔离规则均不破坏。

## 2. 工单固定 schema（一版一单，缺项 = Maint 拒单）

```markdown
---
version: 1.0.26            # 与 jar 文件名版本严格一致
status: pending            # pending → published / aborted（Maint 回写镜像为准，票内 status 只由 Maint 改）
date: <YYYY-MM-DD HH:mm>   # 工单创建（Get-Date 锚定）
---
## 1. 构建产物
- jar: <绝对路径>            # build/libs/coreswap-<mc>-<ver>.jar
- jar sha256: <64 位>        # 发布物完整性锚
- dll sha256: <64 位>        # jar 内 worldgen.dll（与 target/release/worldgen.dll 一致性已在主工作区核验）
- 源 commit: <仓库 HEAD>      # 代码可追溯

## 2. 验证记录（判据表 + 证据路径）
- 端到端/逐位/回归判据各一行结论 + 证据文件绝对路径（cmd-output / review 文档）

## 3. Changelog（中英双语全文，可直接贴 Release notes）

## 4. 发布动作清单（Maint 执行）
- tag 名（对齐仓库既有 tag 约定）
- Release 标题 + notes（用 §3）
- asset 上传名（通常与 jar 文件名一致）
- 发布后：关联 bug 卡回填载体版本 + 跟踪 issue `triage:fixed` 回帖（如适用）

## 5. 已知边界 / 降级声明（§9.7 语义，随 Release notes 公开与否由用户定）
```

## 3. 流程状态机

1. 主工作区：开发 → 验证 → judge MUST → **用户 confirmed** → 构建最终 jar → 填单 → INDEX 登记 → **告知用户**（用户是把工单转给 Maint 会话的触发者）。
2. Maint 会话：读 INDEX → 按 §2 校验单完整性（缺项拒单并注明）→ 自行重算 jar sha256 与票内一致 → **用户显式确认发布**（生产动作）→ gh api 执行 → 回写 status 镜像 + 票内 status。
3. 异常路径：任何校验不过 / 用户不确认 → status 镜像写 `aborted` + 原因；主工作区修订重发（版本号不变，单内容更新，date 刷新）。

## 4. 纪律

- 主工作区**不直接发版**（不跑 gh release）；Maint **不构建**（不在主工作区跑构建，只消费工单）。
- 工单里的 sha256 是**唯一完整性判据**——Maint 以自己重算为准，不信任主工作区转述。
- 发布 = 生产动作 = 逐次人工确认，无自动发布（Maint §5 既有铁律，本契约不改变）。

## 5. Hash 双重校对 + 污染反馈环

跨 session 双执行体互不信任对方的构建转述，杜绝 release 构建物污染：

1. **第一重（主工作区，出单前）**：jar sha256 与 jar 内 worldgen.dll sha256 均写入工单；dll 还须与 `target/release/worldgen.dll` 一致（执行体三元组：构建产源 / 打包物 / 运行验证体）。
2. **第二重（Maint，发布前 MUST）**：独立重算 jar 文件 sha256 + 解包重算内部 dll sha256，与工单声明**逐字节比对**——任何不一致 → **拒单**，发布流程 halt。
3. **污染反馈环（mismatch 时）**：Maint 在 status 镜像写 `hash-mismatch`（字段：重算值 / 票内值 / 差异对象 jar|dll）→ 回报主工作区排查。主工作区侧排查清单（按历史家族）：
   - 构建链：gradle processResources 取到的 dll 是否陈旧（#23 rlib/dll 陈旧假绿；mtime 不可信，用内容指纹）
   - 解压链：CoreSwapFixHelper / 解压缓存幂等复用把旧产物当新（E9 家族：size 相等即复用）
   - 转录链：工单 sha 本身手抄错（重算即暴露）
4. **闭环**：主工作区修复后**刷新工单**（重算 sha + date 更新），Maint 重新走第二重；通过才进入人工确认发布。
