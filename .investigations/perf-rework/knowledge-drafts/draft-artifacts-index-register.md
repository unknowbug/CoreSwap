# .artifacts/index.yaml 登记草稿（root-cause-draft + review-rootcause）

> **目标路径**：`.artifacts/index.yaml`
> **应用方式（主会话）**：在文件末尾追加两个 entries。marker 为现有末尾两条 perf-rework 条目（L157-164，`- id: 're-code:perf-rework:requirements-doc'` 到 `status: candidate`），在其后追加下方新条目。
> **用途**：满足 judge 审查要点 5（产物契约，spec §1.3）：root-cause-draft.md / review-rootcause.md 已落盘但未登记。kind/status 按审查对象现状（根因分析 draft 已定论，登记状态按产物惯例标 candidate，用户拍板 + 修复闭环后再升）。

## 定位 marker（现有文本，在其后追加）

```yaml
  - id: 're-code:perf-rework:requirements-doc'
    path: '../.investigations/perf-rework/requirements-doc.md'
    kind: plan
    status: candidate
  - id: 're-code:perf-rework:static-audit'
    path: '../.investigations/perf-rework/static-audit.md'
    kind: analysis
    status: candidate
```

## 追加的新文本（紧接 marker 之后）

```yaml
  - id: 're-code:perf-rework:root-cause-draft'
    path: '../.investigations/perf-rework/root-cause-draft.md'
    kind: analysis
    status: candidate
  - id: 're-code:perf-rework:review-rootcause'
    path: '../.investigations/perf-rework/review-rootcause.md'
    kind: review
    status: candidate
```
