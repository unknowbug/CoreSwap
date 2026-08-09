# -*- coding: utf-8 -*-
"""搜索 beard_run.txt / noiseblk_run3.txt 中的 [BEARD] StructureWeightSampler 采样"""
import sys
sys.stdout.reconfigure(encoding="utf-8", errors="replace")

for fn in [r"E:\PYTHON\CoreSwap\.investigations\-288-reopen\cmd-output\beard_run.txt",
           r"E:\PYTHON\CoreSwap\.investigations\-288-reopen\cmd-output\noiseblk_run3.txt",
           r"E:\PYTHON\CoreSwap\.investigations\-288-reopen\cmd-output\noiseblk_run2.txt"]:
    print("=" * 20, fn, "=" * 20)
    try:
        lines = open(fn, encoding="utf-8", errors="replace").read().splitlines()
    except FileNotFoundError:
        print("MISSING")
        continue
    for i, l in enumerate(lines):
        if "BEARD" in l or "beard" in l.lower() or "StructureWeight" in l or "ocean_ruin" in l or "weight" in l.lower():
            print(f"{i+1}: {l}")
