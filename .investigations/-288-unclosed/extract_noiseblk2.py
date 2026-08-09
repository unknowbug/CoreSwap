# -*- coding: utf-8 -*-
"""提取 noiseblk_blockprobe.txt 中 chunk(-16,-16) (-244,-256) 列完整 NOISE-BLK 输出"""
import sys
sys.stdout.reconfigure(encoding="utf-8", errors="replace")

lines = open(r"E:\PYTHON\CoreSwap\.investigations\-288-reopen\cmd-output\noiseblk_blockprobe.txt",
             encoding="utf-8", errors="replace").read().splitlines()
print("== chunk(-16,-16) 相关行（NOISE-BLK + status + EstDiagN）==")
for i, l in enumerate(lines):
    if "(-16,-16)" in l or "(-244" in l or "status=" in l:
        print(l)
