# -*- coding: utf-8 -*-
# find_portals.py — 扫 region palette 找 nether_portal 所在 chunk
import sys, os, zlib, gzip, struct
sys.stdout.reconfigure(encoding="utf-8", errors="replace")
_ns = {}
src = open(r"E:\PYTHON\CoreSwap\.investigations\multiworld-port\parse_mca_chunk.py", encoding="utf-8").read().replace("\nmain()\n", "\n")
exec(compile(src, "p", "exec"), _ns)
read_mca_chunk = _ns["read_mca_chunk"]; parse_nbt_named = _ns["parse_nbt_named"]

def scan(region_dir, dim):
    for fn in sorted(os.listdir(region_dir)):
        if not fn.endswith(".mca"):
            continue
        parts = fn.split(".")
        rx, rz = int(parts[1]), int(parts[2])
        data = open(os.path.join(region_dir, fn), "rb").read()
        for i in range(1024):
            idx = i * 4
            off = int.from_bytes(data[idx:idx+3], "big")
            if off == 0:
                continue
            start = off * 4096
            length = int.from_bytes(data[start:start+4], "big")
            raw = data[start+5:start+4+length]
            try:
                raw = zlib.decompress(raw) if data[start+4] != 1 else gzip.decompress(raw)
            except Exception:
                continue
            try:
                _, root = parse_nbt_named(raw)
            except Exception:
                continue
            cx = i % 32 + rx * 32
            cz = i // 32 + rz * 32
            hits = []
            for s in root.get("sections", []):
                if not isinstance(s, dict):
                    continue
                bs = s.get("block_states", {}) or {}
                for p in bs.get("palette", []):
                    if isinstance(p, dict):
                        nm = p.get("Name", "")
                        if "portal" in nm or nm in ("minecraft:obsidian",):
                            hits.append((nm, s.get("Y")))
            if hits:
                kinds = sorted(set(h[0] for h in hits))
                print("%s chunk(%d,%d): %s" % (fn, cx, cz, kinds))

print("=== nether DIM-1 portals:")
scan(sys.argv[1], "nether")
print("=== overworld portals:")
scan(sys.argv[2], "over")
