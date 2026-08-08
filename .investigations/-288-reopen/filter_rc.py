# -*- coding: utf-8 -*-
# 过滤 rangechoice_run.txt 的 RANGECHOICE 行：(-278, y, -240) 列 y=-64..30
import sys, re

sys.stdout.reconfigure(encoding="utf-8", errors="replace")

pat = re.compile(r"RANGECHOICE\] pos=\(-278,(-?\d+),-240\) input=([-\d.]+) -> (\w+) \((.*?)\) inRange=(.*)")
for l in open("rangechoice_run.txt", encoding="utf-8", errors="replace"):
    m = pat.search(l)
    if not m:
        continue
    y = int(m.group(1))
    if -64 <= y <= 30:
        print(f"y={y:3d} input={m.group(2)} -> {m.group(3)} ({m.group(4)}) inRange={m.group(5)[:70]}")
