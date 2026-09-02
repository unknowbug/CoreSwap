# -*- coding: utf-8 -*-
# read_seed.py — 读 level.dat 的 WorldGenSettings seed
import sys, gzip
sys.stdout.reconfigure(encoding="utf-8", errors="replace")
_ns = {}
_src = open(r"E:\PYTHON\CoreSwap\.investigations\multiworld-port\parse_mca_chunk.py", encoding="utf-8").read()
_src = _src.replace("\nmain()\n", "\n")
exec(compile(_src, "p", "exec"), _ns)
parse_nbt_named = _ns["parse_nbt_named"]
data = gzip.open(sys.argv[1], "rb").read()
_, root = parse_nbt_named(data)
d = root.get("Data", {})
wg = d.get("WorldGenSettings", {})
print("seed =", wg.get("seed", d.get("RandomSeed", "?")))
print("LevelName =", d.get("LevelName", "?"))
