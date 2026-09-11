# -*- coding: utf-8 -*-
# 补跑剩余两项：E4（build-tooling，锚点子串已修正）+ E5（1.21.6 时间线）
import os, sys
sys.stdout.reconfigure(encoding="utf-8", errors="replace")
ROOT = r"E:\PYTHON\CoreSwap"
draft = open(os.path.join(ROOT, r".investigations\e5-recompute-260911-05\knowledge-update-draft-260911-05.md"),
             encoding="utf-8").read().split("\n")
JOBS = [
    (r"knowledge\discovered\build-tooling.md", 998, "> 该断言已落地在 `diff_1201.py`", 254, 257),
    (r"versions\1.21.6\docs\10-timewise-archive.md", 88, "状态：C7-①/②/③", 282, 283),
]
fail = 0
for (rel, ln, expect, bs, be) in JOBS:
    path = os.path.join(ROOT, rel)
    lines = open(path, encoding="utf-8").read().split("\n")
    if ln > len(lines):
        print(f"[FAIL] {rel}: 插入点 {ln} 超出 {len(lines)}"); fail += 1; continue
    if expect not in lines[ln - 1]:
        print(f"[FAIL] {rel}:{ln} 锚点失败 -> {lines[ln-1][:80]}"); fail += 1; continue
    block = draft[bs - 1:be]
    ins = block if block and block[0].strip() == "" else [""] + block
    if ins and ins[-1].strip() != "":
        ins = ins + [""]
    lines[ln:ln] = ins
    open(path, "w", encoding="utf-8", newline="\n").write("\n".join(lines))
    print(f"[OK] {rel}: 在 {ln} 后插入 {len(ins)} 行")
sys.exit(1 if fail else 0)
