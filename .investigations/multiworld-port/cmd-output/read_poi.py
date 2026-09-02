# -*- coding: utf-8 -*-
# read_poi.py — 扫 poi region，列出 portal 记录位置
import sys, os, zlib, gzip
sys.stdout.reconfigure(encoding="utf-8", errors="replace")
_ns = {}
_src = open(r"E:\PYTHON\CoreSwap\.investigations\multiworld-port\parse_mca_chunk.py", encoding="utf-8").read()
_src = _src.replace("\nmain()\n", "\n")
exec(compile(_src, "p", "exec"), _ns)
read_mca_chunk = _ns["read_mca_chunk"]; parse_nbt_named = _ns["parse_nbt_named"]

def collect(d, out):
    if isinstance(d, dict):
        if d.get("type") == "minecraft:portal" and "pos" in d:
            p = d["pos"]
            if isinstance(p, int):
                # BlockPos.asLong: x 26 bits, z 26 bits, y 12 bits
                x = p >> 38
                y = (p >> 12) & 0xFFF
                z = (p << 26) >> 38
                if x >= 2**25: x -= 2**26
                if z >= 2**25: z -= 2**26
                if y >= 2**11: y -= 2**12
                out.append((x, y, z))
            else:
                out.append(tuple(p))
        for v in d.values():
            collect(v, out)
    elif isinstance(d, list):
        for v in d:
            collect(v, out)

poi_dir = sys.argv[1]
for fn in sorted(os.listdir(poi_dir)):
    if not fn.endswith(".mca"):
        continue
    data = open(os.path.join(poi_dir, fn), "rb").read()
    for i in range(1024):
        idx = i * 4
        off = int.from_bytes(data[idx:idx+3], "big")
        if off == 0:
            continue
        start = off * 4096
        length = int.from_bytes(data[start:start+4], "big")
        comp = data[start+4]
        raw = data[start+5:start+4+length]
        try:
            raw = zlib.decompress(raw) if comp != 1 else gzip.decompress(raw)
        except Exception:
            continue
        try:
            _, root = parse_nbt_named(raw)
        except Exception:
            continue
        out = []
        collect(root, out)
        for p in out:
            cx = i % 32 + 32 * int(fn.split(".")[1])
            cz = i // 32 + 32 * int(fn.split(".")[2])
            print("%s chunk(%d,%d): portal @ %s" % (fn, cx, cz, p))
