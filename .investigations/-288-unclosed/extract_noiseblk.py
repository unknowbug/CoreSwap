# -*- coding: utf-8 -*-
"""提取 noiseblk_blockprobe.txt 中 (-244,-256) 列完整 NOISE 阶段形态"""
import sys
sys.stdout.reconfigure(encoding="utf-8", errors="replace")

lines = open(r"E:\PYTHON\CoreSwap\.investigations\-288-reopen\cmd-output\noiseblk_blockprobe.txt", encoding="utf-8", errors="replace").read().splitlines()
for i, l in enumerate(lines):
    if "-244" in l or "NOISE-BLK" in l or "status" in l or "chunk(" in l:
        print(f"{i+1}: {l}")
