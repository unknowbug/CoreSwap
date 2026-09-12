# 提案：merge_index 片段顶层契约容错（裸列表根 + 逐文件隔离 + 注释保全）

- 提案对象：RE-Framework 工具链 `scripts/merge_index.py`（= `ref_merge_index` 工具的实现；`core-artifact` §5.1 配套）
- 提案人：CoreSwap 260912-02 session（2026-09-12）
- 状态：draft（待维护 agent 评估）
- 实证来源：CoreSwap 错误台账 `.investigations/shared-java-core-260912-02/errors-260912-02.md` **W13** + judge 交付前确认残留 **R4**（`.investigations/shared-java-core-260912-02/judge-260912-02.md:404`）；本块实测（`--dry-run` 复现 + 源码三点定位 + 逐文件读 legacy 片段 + 工具 sha 比对）

## 1. 问题陈述

`ref_merge_index` 在 CoreSwap 仓库**当前完全不可用**：任何一次合并（含 `--dry-run`）都在 `norm_entries` 处抛异常并整体 fail-fast，退出码 1、**不写任何文件**。后果是 `core-artifact` 的产物登记（`.artifacts/` 条目）在工具路径上被完全阻断，项目只能人工绕过。

三层缺口叠加（任一层单独存在都不致命）：

1. **片段顶层契约无类型分支**：`norm_entries()` 假定片段顶层是 mapping（`schema_version` / `project` / `module` / `entries:`），对解析结果直接调 `.get('entries', [])`。
2. **历史遗留片段形态不合契约**：仓库内存在 **5 个「裸列表根」片段**（顶层是 YAML 序列，没有 `entries:` 包裹）。
3. **采集无 per-file 容错**：`collect_fragments()` 递归 glob 全部 `index-entry.yaml`，任一文件不合规即让整个项目的合并中止（fail-fast 无隔离）。

此外有**同源副作用**：写回路径是「parse → 白名单化 → 重建整个文档 → dump」，只保留 `{id, path, kind, status}` 四字段 ⇒ **注释必然丢失**（YAML 加载器不保留注释是语言层面事实；`ENTRY_KEYS` / `norm_entries` / 写回 dict 三处叠加），而本项目 `.artifacts/index.yaml` 的惯例是**注释承载结论摘要与证据指针**。

## 2. 证据（可复现）

### 2.1 现象（一手运行记录）

```
AttributeError: 'list' object has no attribute 'get'
  栈：main():95 → norm_entries():52 → data.get('entries', [])
  退出码 1，未写任何文件
```

来源：`errors-260912-02.md:326-327`（`--dry-run` 即崩溃）。本项目 `scripts/merge_index.py` 副本与 RE-Framework 上游版 **sha256 前 16 位同为 `299598f46ebe1269`** ⇒ 非项目侧改动所致。

### 2.2 源码三点定位（本项目副本行号）

| 点 | 位置 | 内容 |
|---|---|---|
| ① 无类型分支 | `scripts/merge_index.py:52` | `for e in data.get('entries', []) or []:` —— `data` 为 `list` 时直接 AttributeError |
| ② 递归采集无容错 | `:59-60`（调用点 `:95`） | `glob('.artifacts/**/index-entry.yaml', recursive=True)`，无 try / except、无跳过 |
| ③ 写回丢注释 | `:37` + `:55` + `:129-139` | `ENTRY_KEYS = ('id','path','kind','status')` → `{k: e.get(k,'') …}` 白名单化 → 重建 `{schema_version, project, module, entries}` 后 `yaml.safe_dump` **覆写**根文件 |

### 2.3 5 个 legacy 裸列表根片段（逐文件实读）

`.artifacts/8576-24blocks/{aquifer-wateredge,biome-fix,biome-terracotta,followup,surface-plus1}/index-entry.yaml` —— 均为「注释行 + 直接进入序列」（如 `biome-fix/index-entry.yaml:4` 起即 `- id: …`，**无 `entries:` 映射键**）；行数 12 / 43 / 12 / 13 / 12。⇒ 解析结果为 `list`，命中缺口 ①。

### 2.4 影响面量化（注释即权威载体）

本项目 `.artifacts/index.yaml` 中**以 `#` 开头的注释行占相当比例**（260912-02 会话实测：全文 **1531 行、其中 `#` 注释行 504 行**，约 33%），且根 `index.yaml:1190` 已就地写下「勿跑该工具」警告 ⇒ 一旦工具被采用并写回，带注释的权威条目会被降级为 4 字段裸条目、叙事静默丢失。

### 2.5 本块的正当绕行（非项目侧缺陷）

① 主会话**手工**把 260912-02 的 4 条（带注释）追加进根 `.artifacts/index.yaml:1462-1524`；② 本地幂等校验脚本 `.tmp/shared-java-core-260912-02/check_index.py` 比对「片段 ↔ 根 index」的 `id/path/kind/status` 逐字段一致 + 幂等（实测 `IDEMPOTENT-CONSISTENT`）；③ 片段改置**标准采集路径** `.artifacts/shared-java-core-260912-02/index-entry.yaml`（工具修好后可直接采集，届时因 id / path / status 相同判「已存在，跳过（幂等）」）。

## 3. 建议（最小修复，按优先级）

1. **`norm_entries` 加 list 根容错**（一行）：`if isinstance(data, list): entries = data` 再取 entries；对既非 mapping 也非 list 的顶层给明确报错（不得静默当空）。
2. **采集按文件 try / except，跳过并报告**：单文件不合规 → 记入跳过清单并在输出打印「本次跳过 N 个片段（路径 + 原因）」，退出码仍可为 0（不静默、不 fail-fast）。
3. **写回保留注释 / 改「只追加缺失条目」策略**：可选 `--preserve-comments`；或把写回从「重建整档」改为「只追加缺失条目 + 原地保留原文」（本项目注释即权威载体的前提）。
4. **文档化片段顶层契约 + 给出合规样例**（`entries:` 包裹示例）并写入 `core-artifact` §5.1；输出行同时打印「片段总数 / 新增 / 幂等跳过 / 冲突 / 跳过文件」。

## 4. 影响面

- 工具在 `re-framework` preset 内嵌、**跨项目复用** ⇒ 缺陷不是 CoreSwap 局部问题：任何含历史遗留片段的项目都会遇到「工具完全不可用」。
- 报错指向 **YAML 内部类型**（`'list' object has no attribute 'get'`），不含片段路径与修复指引 ⇒ 使用者倾向怀疑「自己的片段写错了」，自查成本高（本项目已两次独立遇到同一坑：260910-04 与 260912-02）。
- 写回丢注释是**静默数据降级**（无告警），比崩溃更隐蔽。

## 5. 验收判据（建议）

① 对含 5 个裸列表根片段的仓库跑 `--dry-run`：**不再抛异常**，输出「跳过 5 个片段 + 路径清单」，退出码 0；② 修复后对合规片段合并**幂等**（二次运行 新增 = 0 / 跳过 = N）；③ 若仍采用写回路径，根 `index.yaml` 的注释行数**不减少**，或工具显式声明「不保全注释」且默认不写盘。
