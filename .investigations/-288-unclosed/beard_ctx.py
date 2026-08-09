# -*- coding: utf-8 -*-
"""看 beard_run.txt 中 [BEARD] 的坐标上下文 + y 到多少"""
import sys
sys.stdout.reconfigure(encoding="utf-8", errors="replace")

fn = r"E:\PYTHON\CoreSwap\.investigations\-288-reopen\cmd-output\beard_run.txt"
lines = open(fn, encoding="utf-8", errors="replace").read().splitlines()
# 找 [BEARD] 段起始前的 20 行 + 全段末尾
start = None
beard = []
for i, l in enumerate(lines):
    if "[BEARD]" in l:
        if start is None:
            start = i
        beard.append((i + 1, l))
print(f"== [BEARD] 段起始 line {start + 1}，共 {len(beard)} 行 ==")
print("== 段前 15 行 ==")
for i in range(max(0, start - 15), start):
    print(f"{i+1}: {lines[i]}")
print("== [BEARD] 末 10 行 ==")
for ln, l in beard[-10:]:
    print(f"{ln}: {l}")
