# -*- coding: utf-8 -*-
# read_level_dat.py — 读 level.dat 的玩家坐标/维度（integrated server 存档）
import sys, struct
sys.stdout.reconfigure(encoding="utf-8", errors="replace")
import gzip
_ns = {}
_src = open(r"E:\PYTHON\CoreSwap\.investigations\multiworld-port\parse_mca_chunk.py", encoding="utf-8").read()
_src = _src.replace("\nmain()\n", "\n")
exec(compile(_src, "p", "exec"), _ns)
parse_nbt_named = _ns["parse_nbt_named"]

data = gzip.open(sys.argv[1], "rb").read()
_, root = parse_nbt_named(data)
def walk(d, path=""):
    if isinstance(d, dict):
        for k, v in d.items():
            yield from walk(v, path + "/" + k)
    elif isinstance(d, list):
        yield path, d
    else:
        yield path, d
for p, v in walk(root):
    lp = p.lower()
    if lp.endswith("/pos") or lp.endswith("/dimension") or "player" in lp.lower() and p.endswith("/pos"):
        print(p, "=", v)
