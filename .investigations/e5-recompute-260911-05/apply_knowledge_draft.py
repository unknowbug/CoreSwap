# -*- coding: utf-8 -*-
# 应用 knowledge-update-draft-260911-05.md 的取代指针块（E1-a..E1-e / E2-a / E2-b / E3 / E4 / E5）
# 每个插入：目标文件 + 插入点行号（1-based，插在该行之后）+ 锚点校验子串 + 草稿内块行区间
import io, os, sys
sys.stdout.reconfigure(encoding="utf-8", errors="replace")

ROOT = r"E:\PYTHON\CoreSwap"
DRAFT = os.path.join(ROOT, r".investigations\e5-recompute-260911-05\knowledge-update-draft-260911-05.md")
draft = open(DRAFT, encoding="utf-8").read().split("\n")

# (目标文件, 插入点行号, 锚点校验子串, 草稿块起行, 草稿块止行)  —— 块行区间为 1-based 闭区间
JOBS = [
    (r"knowledge\discovered\workflow-patterns.md", 1718, "家族索引", 67, 67),      # E1-a
    (r"knowledge\discovered\workflow-patterns.md", 1787, "家族索引", 87, 92),      # E1-b
    (r"knowledge\discovered\workflow-patterns.md", 1851, "家族索引", 113, 119),    # E1-c
    (r"knowledge\discovered\workflow-patterns.md", 1860, "家族索引", 137, 137),    # E1-d
    (r"knowledge\discovered\workflow-patterns.md", 1872, "家族索引", 156, 156),    # E1-e
    (r"knowledge\INDEX.md", 131, "> 260910-04 R3 追加", 174, 174),                  # E2-a
    (r"knowledge\INDEX.md", 133, "> 260910-05 追加", 190, 190),                     # E2-b
    (r"knowledge\discovered\algorithm-fingerprints.md", 469, "家族索引", 231, 233), # E3
    (r"knowledge\discovered\build-tooling.md", 998, "> 该断言已落地在 `diff_1201`", 254, 257),  # E4
    (r"versions\1.21.6\docs\10-timewise-archive.md", 88, "状态：C7-①/②/③", 282, 283),          # E5
]

# 按文件分组，组内按插入点降序（自下而上）
from collections import defaultdict
groups = defaultdict(list)
for j in JOBS:
    groups[j[0]].append(j)

fail = 0
for rel, jobs in groups.items():
    path = os.path.join(ROOT, rel)
    lines = open(path, encoding="utf-8").read().split("\n")
    for (_, ln, expect, bs, be) in sorted(jobs, key=lambda x: -x[1]):
        if ln > len(lines):
            print(f"[FAIL] {rel}: 插入点 {ln} 超出文件行数 {len(lines)}"); fail += 1; continue
        anchor = lines[ln - 1]
        if expect not in anchor:
            print(f"[FAIL] {rel}:{ln} 锚点校验失败\n       期望含: {expect}\n       实际行: {anchor[:80]}"); fail += 1; continue
        block = draft[bs - 1:be]
        # 块前后各留一个空行（草稿块本身以空行开头）
        ins = block if block and block[0].strip() == "" else [""] + block
        if ins and ins[-1].strip() != "":
            ins = ins + [""]
        lines[ln:ln] = ins
        print(f"[OK] {rel}: 在 {ln} 后插入 {len(ins)} 行（锚点 {expect[:20]}…）")
    if fail == 0:
        open(path, "w", encoding="utf-8", newline="\n").write("\n".join(lines))

print(f"\n== 完成：失败 {fail} 项 ==")
sys.exit(1 if fail else 0)
