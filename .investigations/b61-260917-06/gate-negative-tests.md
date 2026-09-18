# B6-1 门禁有效性验证：双负向测试（260918-01 round 5）

```yaml
status: draft
block: 260918-01
role: 主会话（验证执行，round 5）
purpose: 验证 gate 的**主检测**（漏映射）与**作用域检测**（#47）都能真失败——防 silently-green 门
layer: Partial（静态抽取 + 注入/还原实证；未跑 JVM 端到端）
```

## 0. 为什么必须做（判据来源）

**知识库 #163（silently-green 门）**：判据只落「存在/数量」的门恒绿，其「绿」只证明门跑了，不证明被保护对象没坏。
**推论**：**一个不能失败的 gate 等于没有 gate**。故本轮对 gate 的两个检测维度各做一次**注入式负向测试**。

## 1. 负向测试 A：主检测（漏映射 → ORPHAN/per-version 缺口）

**注入**：删除 `versions/1.20.1/java/build.gradle` 中已修复的 `-PsurfaceDumpDim` → `-Dsurfacedump.dim` 映射行（模拟「有人误删」回归）。

**期望**：1.20.1 per-version 缺口 11 → **12**，且 `surfacedump.dim` 出现在缺口清单；`--strict` 非零退出。

**实测（原文）**：
```
[INJECTED] removed: if (project.findProperty('surfaceDumpDim') != null) run.vmArg "-Dsurfacedump.dim
  surfacedump.dim
  [1.20.1] 声明 127 / 本版(+共享)消费 136 → 缺口 12
  rc=1
```
⇒ **主检测可失败** ✅（声明面 128→127、缺口 11→12、命中项出现、strict rc=1，四项自洽）。

## 2. 负向测试 B：作用域检测（#47）

**注入**：在文件末尾（`benchVmArgs` 闭包**之外**）加一行 `run.vmArg "...-Dscope.violation=1"`。

**实测（原文）**：
```
[OUT-OF-SCOPE] versions\1.20.1\java\build.gradle:293  if (project.findProperty("scopeViolationTest") != null) run.vmArg "-Ds
  strict rc=1
```
⇒ **作用域检测可失败** ✅。

## 3. 还原验证（两次注入均精确还原）

```
[RESTORED] from .tmp/b61-260917-06/bg-negtest-backup.gradle
git diff --stat HEAD -- versions/1.20.1/java/build.gradle   → （空）
CONSISTENT: 125 / [1.20.1] 缺口 11 / [1.21.6] 缺口 21
```
⇒ 还原**逐字节精确**（git diff 为空），gate 回到基线。测试脚本 `.tmp/b61-260917-06/negtest_round5.py`（临时区，不入库）。

## 4. 本轮附带发现：输出可读性缺陷（**主会话自曝误读**）

**现象**：本轮排查中，主会话一度把 **per-version 段**的 `surfacedump.dim` 误读为「union ORPHAN 里仍有它」→ 短暂误判「gate 有 bug」。

**根因**：原输出中 union 段与 per-version 段**并列打印同名项**，两段标题强度相近（`-- ORPHAN --` vs `-- per-version 缺口 --`），且未标注**同名不同义**。

**修复**（已落地）：① 段标题改为 `---- [union 段] ... ----` / `---- [per-version 段] ... ----` 并在开头加**口径提示**（「名字会重叠但含义不同」）；② per-version 段**逐版标注**「其中 N 项不在 union ORPHAN 中（= 另一版已声明，掩盖发生在此）」——**把掩盖量直接打出来**。

**修复后实测**：1.21.6 段显式报告「其中 **12 项不在 union ORPHAN 中**」——量化了掩盖规模（此前需人工集合运算才能得出）。

**教训（可复用）**：**多口径工具的输出必须显式声明「同名项的口径差异」，并把口径差异量化**——否则读者（含 AI）会把不同口径的同名项当作同一结论，**制造「工具坏了」的假警报**（本例）或「一切正常」的假安全（#168 家族）。这与 #163「门禁只判存在/数量」同源：**判据的读数域必须与判据语义绑定**。

## 5. 覆盖面与降级声明（§9.7）

- **覆盖面**：A 覆盖「主检测的命中与退出码」；B 覆盖「作用域检测的命中与退出码」；还原覆盖「注入不残留」。
- **未覆盖**：① 未跑 JVM 端到端（注入后未实跑 `runServer` 观测行为差异）——故为 **Partial**；② 未测 1.21.6 侧的注入还原；③ 未测 DEAD 类注入（DEAD 与 ORPHAN 共用集合运算路径，风险低）。
- **降级声明**：本验证为 **Partial**（注入式静态实证），**不得称 Full**。

## 6. §9.8 副作用与逆

| 副作用 | 类型 | 逆 |
|---|---|---|
| 临时改 `build.gradle`（两次注入） | in-place | **备份件 `.tmp/b61-260917-06/bg-negtest-backup.gradle`** + 还原后 `git diff` 为空实证；另 git 本体可 revert |
| 新脚本 `negtest_round5.py` | derived | identity（.tmp 不入库） |
| 改 `scripts/check_switch_mapping.py`（输出可读性） | in-place | git 提交（可 revert） |
